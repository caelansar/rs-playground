use crate::ResponseFuture;
use std::future::Future;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::sleep;
use tower::Service;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Timeout<S> {
    inner: S,
    timeout: Duration,
}

impl<S> Timeout<S> {
    #[allow(dead_code)]
    pub fn new(inner: S, timeout: Duration) -> Self {
        Timeout { inner, timeout }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Timeout2<S> {
    inner: S,
    timeout: Duration,
}

impl<S> Timeout2<S> {
    #[allow(dead_code)]
    pub fn new(inner: S, timeout: Duration) -> Self {
        Timeout2 { inner, timeout }
    }
}

impl<S, Request> Service<Request> for Timeout<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let response_future = self.inner.call(req);

        let sleep = sleep(self.timeout);

        ResponseFuture {
            response_future,
            sleep,
        }
    }
}

impl<S, Request> Service<Request> for Timeout2<S>
where
    S: Service<Request> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = S::Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = impl Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|e| e.into())
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let fut = self.inner.call(req);
        let timeout = self.timeout;

        async move {
            match tokio::time::timeout(timeout, fut).await {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(e)) => Err(e.into()),
                Err(e) => {
                    println!("tokio timeout");
                    Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                }
            }
        }
    }
}
