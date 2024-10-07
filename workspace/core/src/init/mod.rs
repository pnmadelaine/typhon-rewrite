mod actor;
mod helpers;

use crate::{prelude::*, projects, Settings};

#[derive(Clone)]
pub struct Addr(typhon_actors::Addr<actor::Cmd>);

#[derive(Debug)]
pub enum Error {
    IllegalProjectName(String),
    ProjectAlreadyExists(String),
    ProjectIsBeingDeleted(String),
    ProjectNotFound(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl serde::de::StdError for Error {}

pub(crate) fn init(settings: Settings) -> (Addr, JoinHandle<()>) {
    let pool = Pool::new(&settings.database_url);
    let (scope, handle) = Scope::new();
    let actor = Actor::new(BUFFER);
    let addr = actor.address();
    let handle = tokio::spawn(async move {
        actor.run(actor::St::new(scope, pool).await).await;
        handle.await.unwrap();
    });
    (Addr(addr), handle)
}

impl Addr {
    pub async fn delete_project(&mut self, name: String) -> Result<(), Error> {
        self.0
            .get(|ret| actor::Cmd::DeleteProject { name, ret })
            .await
    }
    pub async fn new_project(&mut self, name: String) -> Result<(), Error> {
        self.0.get(|ret| actor::Cmd::NewProject { name, ret }).await
    }
    pub async fn project(&mut self, name: String) -> Result<projects::Addr, Error> {
        self.0.get(|ret| actor::Cmd::Project { name, ret }).await
    }
    pub async fn projects(&mut self) -> Vec<String> {
        self.0.get(|ret| actor::Cmd::Projects { ret }).await
    }
    pub async fn rename_project(&mut self, from: String, to: String) -> Result<(), Error> {
        self.0
            .get(|ret| actor::Cmd::RenameProject { from, to, ret })
            .await
    }
    pub async fn wait(self) {
        self.0.wait().await
    }
}
