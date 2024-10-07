mod actor;
mod models;
mod queue;

use actor::Cmd;

use crate::prelude::*;

pub type Id = i64;
pub type Priority = u64;

#[derive(Debug, Clone, Copy)]
pub enum Outcome {
    Cancel,
    Fail,
    Success,
}

#[derive(Debug, Clone)]
pub enum Status {
    Unknown, // Remove
    Pending,
    InQueue,
    Skipped,
    DependencyFail {
        id: Id,
    },
    Running {
        started_at: SystemTime,
    },
    Finished {
        started_at: SystemTime,
        ended_at: SystemTime,
        result: Outcome,
    },
}

#[derive(Debug, Clone)]
pub struct Info {
    pub id: Id,
    pub drv: String,
    pub status: Status,
}

pub struct Watch(Vec<actor::WatchBuild>);

#[derive(Debug)]
pub enum Error {
    BuildNotFound(Id),
    StderrNotFound(Id),
}

#[derive(Clone)]
pub struct Addr(typhon_actors::Addr<Cmd>);

impl Addr {
    pub async fn build(&mut self, drv: String, priority: Priority) -> Watch {
        self.0.get(|ret| Cmd::Build { drv, priority, ret }).await
    }

    pub async fn info(&mut self, id: Id) -> Option<Info> {
        self.0.get(|ret| Cmd::Info { id, ret }).await
    }

    pub async fn stderr(&mut self, id: Id) -> Result<Read, Error> {
        self.0.get(|ret| Cmd::Stderr { id, ret }).await
    }
}
