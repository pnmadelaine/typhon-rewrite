mod actor;
mod logs;

use std::future::Future;

use crate::prelude::*;

pub(crate) trait Task {
    fn run<W>(self, w: &mut W) -> impl Future<Output = bool> + Send
    where
        W: AsyncWrite + Send + Unpin;
}

pub(crate) struct Addr(typhon_actors::Addr<actor::Cmd>);

pub(crate) fn new<T>(parent_scope: &mut Scope, pool: Pool, task: T) -> Addr
where
    T: Task + Send + 'static,
{
    let scope = parent_scope.clone();
    Addr(parent_scope.spawn_actor(actor::St::new(scope, pool, task), BUFFER))
}

impl Addr {
    pub(crate) async fn result(&mut self) -> Option<bool> {
        let mut recv = self.0.get(|ret| actor::Cmd::Result { ret }).await;
        recv.changed().await.unwrap();
        let res = *recv.borrow();
        res
    }
    pub(crate) async fn start(&mut self) {
        self.0.send(actor::Cmd::Start).await;
    }
    pub(crate) async fn stderr(&mut self) -> impl AsyncRead {
        self.0.get(|ret| actor::Cmd::Stderr { ret }).await
    }
}
