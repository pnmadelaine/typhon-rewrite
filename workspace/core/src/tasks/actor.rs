use tokio::fs;

use super::{logs::*, *};
use crate::prelude::*;

pub(super) enum Cmd {
    Result {
        ret: Ret<watch::Receiver<Option<bool>>>,
    },
    Start,
    Stderr {
        ret: Ret<ReadLog>,
    },
}

pub(super) struct St {
    id: i32,
    send_abort: Option<oneshot::Sender<()>>,
    send_start: Option<oneshot::Sender<()>>,
    recv_finished: watch::Receiver<bool>,
    recv_success: watch::Receiver<Option<bool>>,
}

impl St {
    fn result(&mut self, ret: Ret<watch::Receiver<Option<bool>>>) {
        ret.send(self.recv_success.clone());
    }

    fn start(&mut self) {
        self.send_start.take().map(|send| send.send(()));
    }

    fn stderr(&mut self, ret: Ret<ReadLog>) {
        let recv_finished = self.recv_finished.clone();
        let path = format!("logs/{}.stderr", self.id);
        let fut = Box::pin(fs::File::open(path));
        let file = None;
        ret.send(ReadLog {
            recv_finished,
            fut,
            file,
        });
    }

    // ...

    pub(super) async fn new<T>(mut scope: Scope, pool: Pool, task: T) -> Self
    where
        T: Task + Send + 'static,
    {
        let mut conn = pool.get().await;
        let id = diesel::insert_into(schema::tasks::table)
            .default_values()
            .returning(schema::tasks::id)
            .get_result(&mut conn)
            .await
            .unwrap();
        drop(conn);

        let path = format!("logs/{id}.stderr");
        let (send_abort, recv_abort) = oneshot::channel();
        let send_abort = Some(send_abort);
        let (send_start, recv_start) = oneshot::channel();
        let send_start = Some(send_start);
        let (send_finished, recv_finished) = watch::channel(false);
        let (send_success, recv_success) = watch::channel(None);
        let file = fs::File::create(path).await.unwrap();
        let mut log_writer = WriteLog {
            send_finished,
            file,
        };

        scope.spawn(async move {
            let Ok(()) = recv_start.await else { return };
            {
                let mut conn = pool.get().await;
                let started_at = Some(SystemTime::now());
                diesel::update(schema::tasks::table.filter(schema::tasks::id.eq(id)))
                    .set(schema::tasks::started_at.eq(started_at))
                    .execute(&mut conn)
                    .await
                    .unwrap();
                drop(conn);
            }
            let success = tokio::select! {
                x = task.run(&mut log_writer) => Some(x),
                _ = recv_abort => None,
            };
            {
                let mut conn = pool.get().await;
                let ended_at = Some(SystemTime::now());
                diesel::update(schema::tasks::table.filter(schema::tasks::id.eq(id)))
                    .set((
                        schema::tasks::ended_at.eq(ended_at),
                        schema::tasks::success.eq(success),
                    ))
                    .execute(&mut conn)
                    .await
                    .unwrap();
                drop(conn);
            }
            let _ = send_success.send(success);
            log_writer.shutdown().await.unwrap();
        });

        Self {
            id,
            send_abort,
            send_start,
            recv_finished,
            recv_success,
        }
    }
}

impl Exec for Cmd {
    type St = St;

    fn exec(self, state: &mut Self::St, _: &WeakAddr<Self>) {
        match self {
            Cmd::Result { ret } => state.result(ret),
            Cmd::Start => state.start(),
            Cmd::Stderr { ret } => state.stderr(ret),
        }
    }

    async fn finish(state: Self::St) {
        let St {
            id: _,
            send_abort,
            send_start,
            mut recv_finished,
            mut recv_success,
        } = state;
        drop(send_start);
        drop(send_abort);
        let _ = recv_finished.changed().await;
        let _ = recv_success.changed().await;
    }
}
