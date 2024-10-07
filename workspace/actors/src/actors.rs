use crate::*;

pub trait Exec
where
    Self: Sized + Send + 'static,
{
    type St: Send;
    fn exec(self, state: &mut Self::St, send: &WeakAddr<Self>);
    fn finish(state: Self::St) -> impl Future<Output = ()> + Send;
}

/// An actor is a long running task, it owns some state and will execute commands sent to it.
pub struct Actor<T: Exec> {
    addr: Addr<T>,
    recv: mpsc::Receiver<T>,
    guard: watch::Sender<()>,
}

impl<T: Exec> Actor<T> {
    pub fn new(buffer: usize) -> Self {
        let (send, recv) = mpsc::channel(buffer);
        let (guard, wait) = watch::channel(());
        let addr = Addr { send, wait };
        Self { addr, recv, guard }
    }

    pub fn address(&self) -> Addr<T> {
        self.addr.clone()
    }

    pub fn run(self, mut state: T::St) -> impl Future<Output = ()> + Send {
        async {
            let Self {
                addr,
                mut recv,
                guard,
            } = self;
            let addr = addr.weak();
            while let Some(cmd) = recv.recv().await {
                cmd.exec(&mut state, &addr);
            }
            T::finish(state).await;
            drop(guard);
        }
    }
}

/// An address is an endpoint to a running actor.
/// An actor will live until all addresses pointing to it are dropped.
pub struct Addr<T: Exec> {
    send: mpsc::Sender<T>,
    wait: watch::Receiver<()>,
}

impl<T: Exec> std::clone::Clone for Addr<T> {
    fn clone(&self) -> Self {
        let send = self.send.clone();
        let wait = self.wait.clone();
        Self { send, wait }
    }
}

impl<T: Exec> Addr<T> {
    pub async fn send(&mut self, cmd: T) {
        let _: Result<(), _> = self.send.send(cmd).await;
    }

    pub async fn wait(self) {
        let Self { send, mut wait } = self;
        drop(send);
        assert!(wait.changed().await.is_err());
    }

    pub fn weak(self) -> WeakAddr<T> {
        WeakAddr {
            send: self.send.downgrade(),
            wait: self.wait,
        }
    }

    pub async fn get<O, F>(&mut self, f: F) -> O
    where
        F: FnOnce(Ret<O>) -> T,
    {
        let (send, recv) = tokio::sync::oneshot::channel();
        self.send(f(Ret(Some(send)))).await;
        recv.await.unwrap()
    }
}

/// A weak address does not keep the actor alive.
pub struct WeakAddr<T: Exec> {
    send: mpsc::WeakSender<T>,
    wait: watch::Receiver<()>,
}

impl<T: Exec> std::clone::Clone for WeakAddr<T> {
    fn clone(&self) -> Self {
        let send = self.send.clone();
        let wait = self.wait.clone();
        Self { send, wait }
    }
}

impl<T: Exec> WeakAddr<T> {
    pub async fn send(&self, cmd: T) -> Option<()> {
        let _ = self.send.upgrade()?.send(cmd).await;
        Some(())
    }

    pub async fn wait(mut self) {
        assert!(self.wait.changed().await.is_err());
    }

    pub async fn get<O, F>(&mut self, f: F) -> Option<O>
    where
        F: FnOnce(Ret<O>) -> T,
    {
        let (send, recv) = tokio::sync::oneshot::channel();
        self.send(f(Ret(Some(send)))).await?;
        Some(recv.await.unwrap())
    }
}

/// A helper type to define return values for commands.
pub struct Ret<T>(Option<tokio::sync::oneshot::Sender<T>>);

impl<T> Drop for Ret<T> {
    fn drop(&mut self) {
        assert!(self.0.is_none());
    }
}

impl<T: Send> Ret<T> {
    pub fn send(mut self, value: T) {
        let _: Result<(), _> = self.0.take().unwrap().send(value);
    }
}

pub type Res<T, E> = Ret<Result<T, E>>;

impl<T: Send, E: Send> Res<T, E> {
    pub fn ok(self, x: T) {
        self.send(Ok(x));
    }
    pub fn err(self, err: E) {
        self.send(Err(err));
    }
    pub fn do_send(self, f: impl FnOnce() -> Result<T, E>) {
        self.send(f());
    }
}
