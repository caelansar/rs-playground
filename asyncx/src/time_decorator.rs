use std::{
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures::Future;
use pin_project::pin_project;

#[pin_project]
pub struct TimeDecorator<F: Future> {
    start: Option<Instant>,
    #[pin]
    future: F,
}

impl<F: Future> TimeDecorator<F> {
    #[allow(dead_code)]
    pub fn new(future: F) -> Self {
        Self {
            future,
            start: None,
        }
    }
}

impl<F: Future> Future for TimeDecorator<F> {
    type Output = (F::Output, Duration);

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();

        let start = this.start.get_or_insert_with(Instant::now);
        let inner_poll = this.future.poll(cx);
        let elapsed = start.elapsed();

        match inner_poll {
            Poll::Pending => Poll::Pending,
            Poll::Ready(output) => Poll::Ready((output, elapsed)),
        }
    }
}

pub struct TimeDecorator1<F: Future> {
    start: Option<Instant>,
    future: F,
}

impl<F: Future> TimeDecorator1<F> {
    #[allow(dead_code)]
    pub fn new(future: F) -> Self {
        Self {
            future,
            start: Some(Instant::now()),
        }
    }
}

impl<F: Future> Future for TimeDecorator1<F> {
    type Output = (F::Output, Duration);

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let start = self.start;

        let f = unsafe { self.map_unchecked_mut(|x| &mut x.future) };
        let inner_poll = f.poll(cx);
        let elapsed = start.unwrap().elapsed();

        match inner_poll {
            Poll::Pending => Poll::Pending,
            Poll::Ready(output) => Poll::Ready((output, elapsed)),
        }
    }
}
