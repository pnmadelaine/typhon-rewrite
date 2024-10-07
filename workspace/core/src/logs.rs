use std::{
    future::Future,
    io::Error as IoError,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::fs::File;

use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct NotFound;

pub struct Read {
    recv: watch::Receiver<bool>,
    file: File,
}

#[derive(Clone)]
pub(crate) struct Subscribe {
    recv: watch::Receiver<bool>,
    path: String,
}

pub(crate) struct Write {
    send: watch::Sender<bool>,
    file: File,
}

impl Subscribe {
    pub(crate) fn new(path: String) -> Self {
        let (send, recv) = watch::channel(true);
        drop(send);
        Self { recv, path }
    }

    pub(crate) async fn read(self) -> Result<Read, NotFound> {
        let Self { recv, path } = self;
        let file = File::open(path).await.map_err(|_| NotFound)?;
        Ok(Read { recv, file })
    }
}

pub(crate) fn new(path: String) -> (impl Future<Output = Write>, Subscribe) {
    let (send, recv) = watch::channel(false);
    let subscribe = Subscribe {
        path: path.clone(),
        recv,
    };
    let write = async move {
        let file = File::open(&path).await.unwrap();
        Write { send, file }
    };
    (write, subscribe)
}

impl AsyncRead for Read {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<Result<(), IoError>> {
        match Pin::new(&mut self.file).poll_read(cx, buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Ready(Ok(())) => {
                if *self.recv.borrow() {
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

impl AsyncWrite for Write {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, IoError>> {
        Pin::new(&mut self.file).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
        Pin::new(&mut self.file).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
        let res = Pin::new(&mut self.file).poll_shutdown(cx);
        self.send.send_replace(true);
        res
    }
}
