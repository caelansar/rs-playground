use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Future;
use tokio::task::{JoinError, JoinHandle};

/// Spawn a new tokio Task and cancel it on drop.
pub fn spawn<T>(future: T) -> CancelDecorator<T::Output>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    CancelDecorator(tokio::spawn(future))
}

/// Cancels the wrapped tokio Task on Drop.
pub struct CancelDecorator<T>(JoinHandle<T>);

impl<T> Future for CancelDecorator<T> {
    type Output = Result<T, JoinError>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { Pin::new_unchecked(&mut self.0) }.poll(cx)
    }
}

impl<T> Drop for CancelDecorator<T> {
    fn drop(&mut self) {
        // do `let _ = self.0.cancel()` for `async_std::task::Task`
        self.0.abort();
    }
}
