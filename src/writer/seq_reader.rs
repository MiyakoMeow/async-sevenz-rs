use futures::io::AsyncRead;
use std::ops::Deref;

pub(crate) struct SeqReader<R> {
    readers: Vec<R>,
    current: usize,
}

impl<R> Deref for SeqReader<R> {
    type Target = [R];

    fn deref(&self) -> &Self::Target {
        &self.readers
    }
}

impl<R> SeqReader<R> {
    pub(crate) fn new(readers: Vec<R>) -> Self {
        Self {
            readers,
            current: 0,
        }
    }

    pub(crate) fn reader_len(&self) -> usize {
        self.readers.len()
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for SeqReader<R> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        loop {
            if self.current >= self.readers.len() {
                return std::task::Poll::Ready(Ok(0));
            }
            let cur = self.current;
            let poll = {
                let r = &mut self.readers[cur];
                std::pin::Pin::new(r).poll_read(cx, buf)
            };
            match poll {
                std::task::Poll::Ready(Ok(0)) => {
                    self.current += 1;
                    continue;
                }
                _ => return poll,
            }
        }
    }
}
