use std::{
    future::Future,
    io::Error as IoError,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::fs::File;

use crate::prelude::*;

pub(super) struct LogWriter {
    pub(super) send_finished: watch::Sender<bool>,
    pub(super) file: File,
}

impl AsyncWrite for LogWriter {
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
        self.send_finished.send_replace(true);
        res
    }
}

pub(super) struct LogReader {
    pub(super) recv_finished: watch::Receiver<bool>,
    pub(super) fut: Pin<Box<dyn Future<Output = Result<File, IoError>> + Send>>,
    pub(super) file: Option<File>,
}

impl AsyncRead for LogReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<Result<(), IoError>> {
        let before = buf.filled().len();
        let file = match self.file.as_mut() {
            Some(file) => file,
            None => match self.fut.as_mut().poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Ok(file)) => self.file.insert(file),
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
            },
        };
        match Pin::new(file).poll_read(cx, buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => {
                if buf.filled().len() == before && !*self.recv_finished.borrow() {
                    Poll::Pending
                } else {
                    Poll::Ready(Ok(()))
                }
            }
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
        }
    }
}
