#![feature(impl_trait_in_assoc_type)]

use futures::future::BoxFuture;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{sleep, Sleep};
use tower::Service;

#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    response_future: F,
    #[pin]
    sleep: Sleep,
}

impl<F: Future> Future for ResponseFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let response_future = this.response_future;
        let sleep = this.sleep;

        match response_future.poll(cx) {
            Poll::Ready(data) => return Poll::Ready(data),
            Poll::Pending => {}
        }

        match sleep.poll(cx) {
            Poll::Ready(_) => {
                panic!("timeout")
            }
            Poll::Pending => {}
        }

        Poll::Pending
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Timeout<S> {
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
struct Timeout2<S> {
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

#[cfg(test)]
mod tests {
    use super::*;

    struct TestService;

    impl Service<()> for TestService {
        type Response = i64;
        type Error = String;
        type Future = impl Future<Output = Result<i64, String>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _req: ()) -> Self::Future {
            async {
                sleep(Duration::from_secs(2)).await;
                println!("TestService");
                Ok(1)
            }
        }
    }

    #[tokio::test]
    async fn service_works() {
        let mut svc = TestService;

        let data = svc.call(()).await.unwrap();
        assert_eq!(1, data);
    }

    #[tokio::test]
    async fn test_timeout_not_reached() {
        let svc = TestService;

        let mut timeout_svc = Timeout::new(svc, Duration::from_secs(3));

        let data = timeout_svc.call(()).await.unwrap();
        assert_eq!(1, data);
    }

    #[tokio::test]
    #[should_panic]
    async fn test_timeout_reached() {
        let svc = TestService;

        let mut timeout_svc = Timeout2::new(svc, Duration::from_secs(1));

        let data = timeout_svc.call(()).await.unwrap();
        assert_eq!(1, data);
    }
}
