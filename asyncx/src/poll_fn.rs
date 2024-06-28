use std::future::{self, Future};
use std::pin::pin;
use std::task::Poll;

/// Resolves to the first future that completes. In the event of a tie, `a` wins.
fn naive_select<T>(
    a: impl Future<Output = T>,
    b: impl Future<Output = T>,
) -> impl Future<Output = T> {
    async {
        let (mut a, mut b) = (pin!(a), pin!(b));
        future::poll_fn(move |cx| {
            if let Poll::Ready(r) = a.as_mut().poll(cx) {
                Poll::Ready(r)
            } else if let Poll::Ready(r) = b.as_mut().poll(cx) {
                Poll::Ready(r)
            } else {
                Poll::Pending
            }
        })
        .await
    }
}

#[tokio::test]
async fn test_poll_fn() {
    let a = async { 42 };
    let b = future::pending();
    let v = naive_select(a, b).await;
    assert_eq!(v, 42);

    let a = future::pending();
    let b = async { 27 };
    let v = naive_select(a, b).await;
    assert_eq!(v, 27);

    let a = async { 42 };
    let b = async { 27 };
    let v = naive_select(a, b).await;
    assert_eq!(v, 42); // biased towards `a` in case of tie!
}
