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
pub struct GitRefLock {
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(rename = "rev")]
    pub revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawGitRepo {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitForgeRepo<T> {
    pub instance: T,
    pub owner: String,
    pub repo: String,
    pub phantom: PhantomData<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Codeberg(pub Option<String>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Github;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gitlab(pub Option<String>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSource<T> {
    #[serde(flatten)]
    pub repo: T,
    #[serde(flatten)]
    pub git_ref: GitRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLock<T> {
    #[serde(flatten)]
    pub repo: T,
    #[serde(flatten)]
    pub git_ref_lock: GitRefLock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Source {
    #[serde(rename = "git")]
    GitRaw(GitSource<RawGitRepo>),
    #[serde(rename = "codeberg")]
    Codeberg(GitSource<GitForgeRepo<Codeberg>>),
    #[serde(rename = "github")]
    Github(GitSource<GitForgeRepo<Github>>),
    #[serde(rename = "gitlab")]
    Gitlab(GitSource<GitForgeRepo<Gitlab>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LockSpecifics {
    #[serde(rename = "git")]
    GitRaw(GitLock<RawGitRepo>),
    #[serde(rename = "codeberg")]
    Codeberg(GitLock<GitForgeRepo<Codeberg>>),
    #[serde(rename = "github")]
    Github(GitLock<GitForgeRepo<Github>>),
    #[serde(rename = "gitlab")]
    Gitlab(GitLock<GitForgeRepo<Gitlab>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lock {
    #[serde(flatten)]
    pub specs: LockSpecifics,
    pub sha256: String,
}
