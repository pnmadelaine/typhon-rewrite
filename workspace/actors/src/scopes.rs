use std::future::{Future, IntoFuture};

use tokio::{
    sync::{mpsc, watch},
    task::{AbortHandle, JoinHandle, JoinSet},
};

use crate::*;

#[derive(Clone)]
pub struct Scope {
    send: mpsc::UnboundedSender<JoinHandle<()>>,
    watch: watch::Receiver<()>,
}

impl Scope {
    pub fn new() -> (Self, JoinHandle<()>) {
        let mut set = JoinSet::new();
        let (send, recv) = mpsc::unbounded_channel();
        let (guard, watch) = watch::channel(());
        let aux = |mut recv: mpsc::UnboundedReceiver<JoinHandle<()>>| async {
            recv.recv().await.map(|h| (h, recv))
        };
        set.spawn(aux(recv));
        let handle = tokio::spawn(async move {
            while let Some(x) = set.join_next().await {
                match x.unwrap() {
                    Some((h, recv)) => {
                        set.spawn(async {
                            h.await.unwrap();
                            None
                        });
                        set.spawn(aux(recv));
                    }
                    None => (),
                }
            }
            drop(guard);
        });
        (Self { send, watch }, handle)
    }

    pub fn spawn<F>(&mut self, f: F) -> AbortHandle
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(f);
        let abort = handle.abort_handle();
        self.send.send(handle.into_future()).unwrap();
        abort
    }

    pub async fn wait(self) {
        let Self { send, mut watch } = self;
        drop(send);
        assert!(watch.changed().await.is_err());
    }

    pub fn sub_scope(&mut self) -> Self {
        let mut set = JoinSet::new();
        let (send, recv) = mpsc::unbounded_channel();
        let (guard, watch) = watch::channel(());
        let aux = |mut recv: mpsc::UnboundedReceiver<JoinHandle<()>>| async {
            recv.recv().await.map(|h| (h, recv))
        };
        set.spawn(aux(recv));
        self.spawn(async move {
            while let Some(x) = set.join_next().await {
                match x.unwrap() {
                    Some((h, recv)) => {
                        set.spawn(async {
                            h.await.unwrap();
                            None
                        });
                        set.spawn(aux(recv));
                    }
                    None => (),
                }
            }
            drop(guard);
        });
        Self { send, watch }
    }

    pub fn spawn_actor<T, F>(&mut self, state: F, buffer: usize) -> Addr<T>
    where
        T: Exec,
        F: Future<Output = T::St> + Send + 'static,
    {
        let actor = Actor::new(buffer);
        let addr = actor.address();
        self.spawn(async { actor.run(state.await).await });
        addr
    }
}
