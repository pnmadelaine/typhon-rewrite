use tokio::task::spawn_blocking;

use super::*;
use crate::{eval::eval, Error, Store};

impl Source {
    pub async fn lock_specs(&self) -> Result<LockSpecifics, Error> {
        todo!()
    }
}

impl Store {
    pub async fn lock(&mut self, source: &Source) -> Result<Lock, Error> {
        let specs = source.lock_specs().await?;
        let sha256 = self.fetch(&specs).await?;
        Ok(Lock { specs, sha256 })
    }

    pub async fn fetch(&mut self, specs: &LockSpecifics) -> Result<String, Error> {
        todo!()
    }
}

trait FetchGitRepo {
    fn url(&self) -> String;
}

impl FetchGitRepo {
    fn sync_default_reference(self) -> Result<String, Error> {
        use git2::{Direction, Remote};

        let url = self.url();

        let mut remote = Remote::create_detached(url)?;
        remote.connect(Direction::Fetch)?;

        let default_branch = remote.default_branch()?;
        Ok(default_branch
            .as_str()
            .ok_or_else(|| Error::NotUtf8(default_branch.to_vec()))?
            .to_owned())
    }

    fn sync_lock(self, git_ref: GitRef) -> Result<GitRefLock, Error> {
        use git2::{Direction, Remote};

        let url = self.url();

        let mut remote = Remote::create_detached(url)?;
        remote.connect(Direction::Fetch)?;

        let GitRef {
            reference,
            revision,
        } = git_ref;

        let reference = match reference {
            Some(reference) => reference,
            None => Self::sync_default_reference(url)?,
        };

        let revision = match revision {
            Some(revision) => revision,
            None => remote
                .list()?
                .iter()
                .find(|head| head.name() == reference)
                .ok_or_else(|| Error::ToDo)?
                .oid()
                .to_string(),
        };

        Ok(GitRefLock {
            reference,
            revision,
        })
    }

    async fn default_reference(&self) -> Result<String, Error> {
        let repo = self.clone();
        Ok(spawn_blocking(move || repo.sync_default_reference()).await??)
    }

    async fn lock(&self, git_ref: &GitRef) -> Result<GitRefLock, Error> {
        let repo = self.clone();
        let git_ref = git_ref.clone();
        Ok(spawn_blocking(move || repo.sync_lock(git_ref)).await??)
    }
}

impl FetchGitRepo for RawGitRepo {
    fn url(&self) -> String {
        self.url.clone()
    }
}

impl FetchGitRepo for GitForgeRepo<Codeberg> {
    fn url(&self) -> String {
        let Self {
            instance,
            owner,
            repo,
            phantom: _,
        } = self;
        format!(
            "https://{}/{owner}/{repo}",
            instance.0.clone().unwrap_or("codeberg.org".to_owned())
        )
    }
}

impl FetchGitRepo for GitForgeRepo<Github> {
    fn url(&self) -> String {
        let Self {
            instance: _,
            owner,
            repo,
            phantom: _,
        } = self;
        format!("https://github.com/{owner}/{repo}",)
    }
}

impl FetchGitRepo for GitForgeRepo<Gitlab> {
    fn url(&self) -> String {
        let Self {
            instance,
            owner,
            repo,
            phantom: _,
        } = self;
        format!(
            "https://{}/{owner}/{repo}",
            instance.0.clone().unwrap_or("codeberg.org".to_owned())
        )
    }
}
