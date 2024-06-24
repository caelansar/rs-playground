use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::PollSemaphore;

use crate::future::ResponseFuture;
use std::task::ready;
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tower::Service;

/// Enforces a limit on the concurrent number of requests the underlying
/// service can handle.
#[derive(Debug)]
pub struct ConcurrencyLimit<T> {
    inner: T,
    semaphore: PollSemaphore,
    /// The currently acquired semaphore permit, if there is sufficient
    /// concurrency to send a new request.
    ///
    /// The permit is acquired in `poll_ready`, and taken in `call` when sending
    /// a new request.
    permit: Option<OwnedSemaphorePermit>,
}

impl<T> ConcurrencyLimit<T> {
    /// Create a new concurrency limiter.
    pub fn new(inner: T, max: usize) -> Self {
        Self::with_semaphore(inner, Arc::new(Semaphore::new(max)))
    }

    /// Create a new concurrency limiter with a provided shared semaphore
    pub fn with_semaphore(inner: T, semaphore: Arc<Semaphore>) -> Self {
        ConcurrencyLimit {
            inner,
            semaphore: PollSemaphore::new(semaphore),
            permit: None,
        }
    }

    /// Get a reference to the inner service
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Get a mutable reference to the inner service
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Consume `self`, returning the inner service
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<S, Request> Service<Request> for ConcurrencyLimit<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // If we haven't already acquired a permit from the semaphore, try to
        // acquire one first.
        if self.permit.is_none() {
            self.permit = ready!(self.semaphore.poll_acquire(cx));
            debug_assert!(
                self.permit.is_some(),
                "ConcurrencyLimit semaphore is never closed, so `poll_acquire` \
                 should never fail",
            );
        }

        // Once we've acquired a permit (or if we already had one), poll the
        // inner service.
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        // Take the permit
        let permit = self
            .permit
            .take()
            .expect("max requests in-flight; poll_ready must be called first");

        // Call the inner service
        let future = self.inner.call(request);

        ResponseFuture::new(future, permit)
    }
}
