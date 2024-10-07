mod structs;

use nix_daemon::{BuildMode, Progress, Stderr, StderrField, StderrResult, StderrResultType, Store};
use serde::{de::DeserializeOwned, Deserialize};
use tokio::{
    io::{AsyncWrite, AsyncWriteExt},
    process::Command,
};

pub use crate::structs::*;

const NIX_DAEMON_SOCKET: &str = "/nix/var/nix/daemon-socket/socket";

type InnerStore = nix_daemon::nix::DaemonStore<tokio::net::UnixStream>;
type StoreError = <InnerStore as Store>::Error;
pub struct NixStore(InnerStore);

pub async fn derivation(drv: &str) -> Result<RecursiveDerivation, Error> {
    let mut cmd = tokio::process::Command::new("nix");
    cmd.args(["derivation", "show", "-r", drv]);
    let res = cmd.output().await.map_err(|_| Error::ToDo)?.stdout;
    serde_json::from_slice(&res).map_err(|_| Error::ToDo)
}

impl NixStore {
    pub async fn new() -> Result<NixStore, Error> {
        Ok(Self(
            InnerStore::builder()
                .connect_unix(NIX_DAEMON_SOCKET)
                .await
                .map_err(Error::NixDaemon)?,
        ))
    }

    pub async fn missing(&mut self, drv: &str) -> Result<Vec<String>, Error> {
        self.0
            .query_missing([format!("{drv}!*")])
            .result()
            .await
            .map(|missing| missing.will_build)
            .map_err(|_| Error::ToDo)
    }

    pub async fn build<'a, 'b, W>(
        &mut self,
        drv: &str,
        stderr: &mut W,
    ) -> Result<Result<(), ()>, Error>
    where
        W: AsyncWrite + Unpin,
    {
        let path = format!("{drv}!*");
        let mut progress = self.0.build_paths([path], BuildMode::Normal);
        loop {
            match progress.next().await {
                Ok(Some(Stderr::Result(StderrResult {
                    kind: StderrResultType::BuildLogLine,
                    fields,
                    ..
                }))) => {
                    let Some(StderrField::String(line)) = fields.first() else {
                        return Err(Error::ToDo);
                    };
                    let _ = stderr.write_all(line.as_bytes()).await;
                    let _ = stderr.write_u8('\n' as u8).await;
                }
                Ok(Some(_)) => (),
                Ok(None) => {
                    return Ok(Ok(()));
                }
                Err(StoreError::NixError(_)) => {
                    return Ok(Err(()));
                }
                Err(err) => {
                    return Err(Error::NixDaemon(err));
                }
            }
        }
    }

    pub async fn hash(&mut self, path: &str) -> Result<String, Error> {
        Ok(self
            .0
            .query_pathinfo(path)
            .result()
            .await
            .map_err(Error::NixDaemon)?
            .ok_or(Error::ToDo)?
            .nar_hash)
    }

    pub async fn lock(&mut self, source: &Source) -> Result<(Lock, String), Error> {
        match source {
            Source::Git {
                url,
                reference,
                revision,
            } => {
                let url = url.clone();
                let reference = reference.clone().unwrap_or("HEAD".to_owned());
                let revision = match revision {
                    Some(revision) => revision.clone(),
                    None => {
                        eval(&format!(
                            "(builtins.fetchGit {{ url = \"{url}\"; ref = \"{reference}\"; }}).rev"
                        ))
                        .await?
                    }
                };
                let out_path: String = eval(&format!("(builtins.fetchGit {{ url = \"{url}\"; ref = \"{reference}\"; rev = \"{revision}\"; }}).outPath")).await?;
                Ok((
                    Lock::Git {
                        url,
                        reference,
                        revision,
                    },
                    out_path,
                ))
            }
            Source::Codeberg { .. } => todo!(),
            Source::GitHub {
                owner,
                repo,
                branch,
                revision,
            } => {
                #[derive(Deserialize)]
                struct RepoInfo {
                    default_branch: String,
                }
                #[derive(Deserialize)]
                struct CommitInfo {
                    sha: String,
                }
                #[derive(Deserialize)]
                struct BranchInfo {
                    commit: CommitInfo,
                }
                let client = reqwest::ClientBuilder::new()
                    .user_agent("Typhon")
                    .build()
                    .map_err(|_| Error::ToDo)?;
                let owner = owner.clone();
                let repo = repo.clone();
                let branch = match branch {
                    Some(branch) => branch,
                    None => {
                        &client
                            .get(format!("https://api.github.com/repos/{owner}/{repo}"))
                            .send()
                            .await
                            .map_err(|err| Error::NetworkUnavailable(err.to_string()))?
                            .json::<RepoInfo>()
                            .await
                            .map_err(|err| Error::UnexpectedResponse(err.to_string()))?
                            .default_branch
                    }
                };
                let revision = match revision {
                    Some(rev) => rev.clone(),
                    None => {
                        client
                            .get(format!(
                                "https://api.github.com/repos/{owner}/{repo}/branches/{branch}"
                            ))
                            .send()
                            .await
                            .map_err(|err| Error::NetworkUnavailable(err.to_string()))?
                            .json::<BranchInfo>()
                            .await
                            .map_err(|err| Error::UnexpectedResponse(err.to_string()))?
                            .commit
                            .sha
                    }
                };
                let url = format!("https://github.com/{owner}/{repo}/archive/{revision}.tar.gz");
                let out_path: String = cmd(
                    "nix-instantiate",
                    &[
                        "--eval",
                        "--expr",
                        &format!("builtins.fetchTarball {{ url = \"{url}\"; }}"),
                    ],
                )
                .await?;
                let sha256 = self.hash(&out_path).await?;
                Ok((
                    Lock::GitHub {
                        owner,
                        repo,
                        revision,
                        sha256,
                    },
                    out_path,
                ))
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Whatever(String),
    NonUtf8Output(String),
    NixNotFound,
    NetworkUnavailable(String),
    UnexpectedResponse(String),
    NixDaemon(<InnerStore as Store>::Error),
    DeserializationError(serde_json::Error),
    ToDo,
}

fn from_utf8(bytes: Vec<u8>) -> Result<String, Error> {
    String::from_utf8(bytes).map_err(|err| Error::NonUtf8Output(err.to_string()))
}

async fn cmd<T: DeserializeOwned>(cmd: &str, args: &[&str]) -> Result<T, Error> {
    let mut cmd = Command::new(cmd);
    cmd.args(args);
    let output = cmd.output().await.map_err(|_| Error::NixNotFound)?;
    if !output.status.success() {
        return Err(Error::Whatever(from_utf8(output.stderr)?));
    }
    let stdout = from_utf8(output.stdout)?;
    serde_json::from_str(&stdout).map_err(Error::DeserializationError)
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(tag = "type")]
// pub enum Source {
//     #[serde(rename = "github")]
//     GitHub {
//         owner: String,
//         repo: String,
//         #[serde(skip_serializing_if = "Option::is_none")]
//         branch: Option<String>,
//         #[serde(skip_serializing_if = "Option::is_none")]
//         rev: Option<String>,
//     },
// }
pub use typhon_types::*;

pub fn json(source: &Source) -> Result<String, ()> {
    serde_json::to_string(source).map_err(|_| ())
}

pub async fn eval<T: DeserializeOwned>(expr: &str) -> Result<T, Error> {
    Ok(cmd("nix-instantiate", &["--eval", "--expr", expr]).await?)
}
