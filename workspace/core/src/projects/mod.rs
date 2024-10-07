mod actor;
mod models;

use crate::prelude::*;

#[derive(Clone)]
pub struct Addr(typhon_actors::Addr<actor::Cmd>);

#[derive(Debug)]
pub enum Error {
    JobsetNotFound(String),
    LockInProgress,
    NoLockInProgress,
    ProjectIsNotDisabled,
    ProjectIsNotEnabled,
    SourceNotSet,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl serde::de::StdError for Error {}

pub(crate) fn new(parent_scope: &mut Scope, pool: Pool, name: String) -> Addr {
    let scope = parent_scope.sub_scope();
    Addr(parent_scope.spawn_actor(
        async move {
            let mut conn = pool.get().await;
            let new_project = models::NewProject { name: &name };
            diesel::insert_into(schema::projects::table)
                .values(&new_project)
                .execute(&mut conn)
                .await
                .unwrap();
            drop(conn);
            actor::St::new(scope, pool, name).await
        },
        BUFFER,
    ))
}

pub(crate) fn load(parent_scope: &mut Scope, pool: Pool, name: String) -> Addr {
    let scope = parent_scope.sub_scope();
    Addr(parent_scope.spawn_actor(actor::St::new(scope, pool, name), BUFFER))
}

impl Addr {
    pub async fn disable(&mut self) -> Result<(), Error> {
        self.0.get(|ret| actor::Cmd::Disable { ret }).await
    }
    pub async fn edit_secret(&mut self, key: String, value: Option<String>) {
        self.0
            .get(|ret| actor::Cmd::EditSecret { key, value, ret })
            .await
    }
    pub async fn enable(&mut self) -> Result<(), Error> {
        self.0.get(|ret| actor::Cmd::Enable { ret }).await
    }
    pub async fn lock(&mut self) -> Result<(), Error> {
        self.0.get(|ret| actor::Cmd::Lock { ret }).await
    }
    // pub async fn lock_error(&mut self) -> Option<String> {
    //     self.0.get(|ret| actor::Cmd::LockError { ret }).await
    // }
    pub async fn secrets(&mut self) -> Vec<String> {
        self.0.get(|ret| actor::Cmd::Secrets { ret }).await
    }
    pub async fn set_path(&mut self, path: String) -> Result<(), Error> {
        self.0.get(|ret| actor::Cmd::SetPath { path, ret }).await
    }
    pub async fn set_source(&mut self, source: Source) -> Result<(), Error> {
        self.0
            .get(|ret| actor::Cmd::SetSource { source, ret })
            .await
    }

    // ...

    pub(crate) async fn wait(self) {
        self.0.wait().await
    }
}
