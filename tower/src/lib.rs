#![feature(impl_trait_in_assoc_type)]

mod concurrency;
mod future;
mod timeout;

use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::time::Sleep;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::concurrency::ConcurrencyLimit;
    use crate::timeout::{Timeout, Timeout2};
    use std::time::Duration;
    use tokio::time::sleep;
    use tower::{Service, ServiceExt};

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

    #[tokio::test]
    #[should_panic]
    async fn test_concurrency_failed() {
        let svc = TestService;

        let mut concurrency_svc = ConcurrencyLimit::new(svc, 2);

        for _ in 0..3 {
            let data = concurrency_svc.call(()).await.unwrap();
            assert_eq!(1, data);
        }
    }

    #[tokio::test]
    async fn test_concurrency_permitted() {
        let svc = TestService;

        let mut concurrency_svc = ConcurrencyLimit::new(svc, 2);

        for _ in 0..3 {
            let data = concurrency_svc
                .ready()
                .await
                .unwrap()
                .call(())
                .await
                .unwrap();
            assert_eq!(1, data);
        }
    }
}
