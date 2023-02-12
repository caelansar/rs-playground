use std::io::Result;

use futures::{stream, Stream};
use pin_project::pin_project;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, BufReader, Lines};

#[pin_project]
pub struct LineStream<R> {
    #[pin]
    inner: Lines<BufReader<R>>,
}

impl<R: AsyncRead> LineStream<R> {
    pub fn new(reader: BufReader<R>) -> Self {
        Self {
            inner: reader.lines(),
        }
    }
}

impl<R: AsyncBufRead> Stream for LineStream<R> {
    type Item = Result<String>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project()
            .inner
            .poll_next_line(cx)
            .map(Result::transpose)
    }
}

pub fn line_stream<R: AsyncBufRead + Unpin>(
    reader: Lines<BufReader<R>>,
) -> impl Stream<Item = String> {
    stream::unfold(reader, |mut reader| async move {
        reader
            .next_line()
            .await
            .transpose()
            .map(|x| (x.unwrap(), reader))
    })
}
