use std::path::PathBuf;

use async_fs as afs;
use async_io::block_on;
use futures::io::AsyncRead;
use futures_lite::AsyncReadExt;

pub(crate) struct LazyFileReader {
    path: PathBuf,
    reader: Option<afs::File>,
    end: bool,
}

impl LazyFileReader {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            reader: None,
            end: false,
        }
    }
}

impl AsyncRead for LazyFileReader {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        if self.end {
            return std::task::Poll::Ready(Ok(0));
        }
        if self.reader.is_none() {
            match block_on(afs::File::open(&self.path)) {
                Ok(f) => self.reader = Some(f),
                Err(e) => return std::task::Poll::Ready(Err(e)),
            }
        }
        match block_on(self.reader.as_mut().unwrap().read(buf)) {
            Ok(n) => {
                if n == 0 {
                    self.end = true;
                    self.reader = None;
                }
                std::task::Poll::Ready(Ok(n))
            }
            Err(e) => std::task::Poll::Ready(Err(e)),
        }
    }
}
