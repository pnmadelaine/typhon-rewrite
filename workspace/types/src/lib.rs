use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRef {
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(rename = "rev", skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLock {
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(rename = "rev")]
    pub revision: String,
}

pub trait FetchGit {
    const EXPR: &'static str;
    fn default_reference(&self) -> String;
    fn lock(&self, git: &GitRef) -> GitLock;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawGitRepo {
    pub url: String,
}
impl FetchGit for RawGitRepo {
    const EXPR: &'static str =
        "({ url, ref, rev, .. }: builtins.fetchGit { inherit url ref rev, })";
    fn default_reference(&self) -> String {
        todo!()
    }
    fn lock(&self, _git: &GitRef) -> GitLock {
        todo!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeGitRepo<T> {
    pub instance: T,
    pub owner: String,
    pub repo: String,
    pub phantom: PhantomData<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Codeberg(Option<String>);
impl FetchGit for ForgeGitRepo<Codeberg> {
    const EXPR: &'static str =
        "({ instance ? \"codeberg.org\", owner, repo, rev, sha256, ... }: builtins.fetchTarball { url = \"https://${instance}/${owner}/${repo}/archive/${rev}.tar.gz\"; inherit sha256; })";
    fn default_reference(&self) -> String {
        todo!()
    }
    fn lock(&self, _git: &GitRef) -> GitLock {
        todo!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Github;
impl FetchGit for ForgeGitRepo<Github> {
    const EXPR: &'static str =
        "({ owner, repo, rev, sha256, ... }: builtins.fetchTarball { url = \"https://github.com/${owner}/${repo}/archive/${rev}.tar.gz\"; inherit sha256; })";
    fn default_reference(&self) -> String {
        todo!()
    }
    fn lock(&self, _git: &GitRef) -> GitLock {
        todo!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gitlab(Option<String>);
impl FetchGit for ForgeGitRepo<Gitlab> {
    const EXPR: &'static str =
        "({ instance ? \"gitlab.com\", owner, repo, rev, sha256, ... }: builtins.fetchTarball { url = \"https://${instance}/${owner}/${repo}/archive/${rev}.tar.gz\"; inherit sha256; })";
    fn default_reference(&self) -> String {
        todo!()
    }
    fn lock(&self, _git: &GitRef) -> GitLock {
        todo!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeGitSource<T> {
    #[serde(flatten)]
    repo: ForgeGitRepo<T>,
    #[serde(flatten)]
    git: GitRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeGitLock<T> {
    #[serde(flatten)]
    repo: ForgeGitRepo<T>,
    #[serde(flatten)]
    git: GitLock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Source {
    #[serde(rename = "git")]
    Git(ForgeGitSource<RawGitRepo>),
    #[serde(rename = "codeberg")]
    Codeberg(ForgeGitSource<Codeberg>),
    #[serde(rename = "github")]
    Github(ForgeGitSource<Github>),
    #[serde(rename = "gitlab")]
    Gitlab(ForgeGitSource<Gitlab>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Lock {
    #[serde(rename = "git")]
    Git(ForgeGitLock<RawGitRepo>),
    #[serde(rename = "codeberg")]
    Codeberg(ForgeGitLock<Codeberg>),
    #[serde(rename = "github")]
    Github(ForgeGitLock<Github>),
    #[serde(rename = "gitlab")]
    Gitlab(ForgeGitLock<Gitlab>),
}

impl Source {
    pub fn expr(&self) -> String {
        match self {
            Self::Git(_src) => RawGitRepo::EXPR.to_owned(),
            _ => todo!(),
        }
    }
}
