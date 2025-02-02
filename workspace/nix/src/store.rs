use nix_daemon::{
    BuildMode, Progress, Stderr, StderrField, StderrResult, StderrResultType, Store as StoreTrait,
};
use serde::{de::DeserializeOwned, Deserialize};
use tokio::{
    io::{AsyncWrite, AsyncWriteExt},
    process::Command,
};

use crate::error::*;

const NIX_DAEMON_SOCKET: &str = "/nix/var/nix/daemon-socket/socket";

pub(crate) type StoreInner = nix_daemon::nix::DaemonStore<tokio::net::UnixStream>;
// pub(crate) type StoreError = <StoreInner as StoreTrait>::Error;
pub(crate) type StoreError = nix_daemon::Error;
pub struct Store(StoreInner);

impl Store {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self(
            StoreInner::builder()
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

    // pub async fn lock(&mut self, source: &Source) -> Result<Lock, Error> {
    //     source.lock(self).await
    // match source {
    //     Source::Git {
    //         url,
    //         reference,
    //         revision,
    //     } => {
    //         let url = url.clone();
    //         let reference = reference.clone().unwrap_or("HEAD".to_owned());
    //         let revision = match revision {
    //             Some(revision) => revision.clone(),
    //             None => {
    //                 eval(&format!(
    //                     "(builtins.fetchGit {{ url = \"{url}\"; ref = \"{reference}\"; }}).rev"
    //                 ))
    //                 .await?
    //             }
    //         };
    //         let out_path: String = eval(&format!("(builtins.fetchGit {{ url = \"{url}\"; ref = \"{reference}\"; rev = \"{revision}\"; }}).outPath")).await?;
    //         Ok((
    //             Lock::Git {
    //                 url,
    //                 reference,
    //                 revision,
    //             },
    //             out_path,
    //         ))
    //     }
    //     Source::Codeberg { .. } => todo!(),
    //     Source::GitHub {
    //         owner,
    //         repo,
    //         branch,
    //         revision,
    //     } => {
    //         #[derive(Deserialize)]
    //         struct RepoInfo {
    //             default_branch: String,
    //         }
    //         #[derive(Deserialize)]
    //         struct CommitInfo {
    //             sha: String,
    //         }
    //         #[derive(Deserialize)]
    //         struct BranchInfo {
    //             commit: CommitInfo,
    //         }
    //         let client = reqwest::ClientBuilder::new()
    //             .user_agent("Typhon")
    //             .build()
    //             .map_err(|_| Error::ToDo)?;
    //         let owner = owner.clone();
    //         let repo = repo.clone();
    //         let branch = match branch {
    //             Some(branch) => branch,
    //             None => {
    //                 &client
    //                     .get(format!("https://api.github.com/repos/{owner}/{repo}"))
    //                     .send()
    //                     .await
    //                     .map_err(|err| Error::NetworkUnavailable(err.to_string()))?
    //                     .json::<RepoInfo>()
    //                     .await
    //                     .map_err(|err| Error::UnexpectedResponse(err.to_string()))?
    //                     .default_branch
    //             }
    //         };
    //         let revision = match revision {
    //             Some(rev) => rev.clone(),
    //             None => {
    //                 client
    //                     .get(format!(
    //                         "https://api.github.com/repos/{owner}/{repo}/branches/{branch}"
    //                     ))
    //                     .send()
    //                     .await
    //                     .map_err(|err| Error::NetworkUnavailable(err.to_string()))?
    //                     .json::<BranchInfo>()
    //                     .await
    //                     .map_err(|err| Error::UnexpectedResponse(err.to_string()))?
    //                     .commit
    //                     .sha
    //             }
    //         };
    //         let url = format!("https://github.com/{owner}/{repo}/archive/{revision}.tar.gz");
    //         let out_path: String = cmd(
    //             "nix-instantiate",
    //             &[
    //                 "--eval",
    //                 "--expr",
    //                 &format!("builtins.fetchTarball {{ url = \"{url}\"; }}"),
    //             ],
    //         )
    //         .await?;
    //         let sha256 = self.hash(&out_path).await?;
    //         Ok((
    //             Lock::GitHub {
    //                 owner,
    //                 repo,
    //                 revision,
    //                 sha256,
    //             },
    //             out_path,
    //         ))
    //     }
    // }
    // }
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
