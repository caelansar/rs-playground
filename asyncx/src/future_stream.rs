#![allow(unused)]

use futures::stream;
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::pin;
use tokio::time::sleep;

async fn async_work(x: i32) -> i32 {
    // to make sure buffered and buffer_unordered are different
    let s = 10 - x;

    sleep(Duration::from_millis(s as u64 * 100)).await;
    x
}

async fn async_predicate(x: i32) -> Option<i32> {
    sleep(Duration::from_millis(100)).await;
    (x % 2 == 0).then_some(x)
}

async fn buffered_example() {
    let mut stream = stream::iter(0..10).map(async_work).buffered(3);

    while let Some(next) = stream.next().await {
        println!("next: {}", next);
    }
}

async fn unordered_example() {
    let mut stream = stream::iter(0..10).map(async_work).buffer_unordered(3);

    while let Some(next) = stream.next().await {
        println!("next: {}", next);
    }
}

async fn buffered_filter_example() {
    let stream = stream::iter(0..10)
        .map(async_work)
        .buffered(3)
        .filter_map(async_predicate);

    pin!(stream);

    while let Some(next) = stream.next().await {
        println!("next: {}", next);
    }
}

async fn concurrent_filter_example() {
    let stream = stream::iter(0..10)
        .map(async_predicate)
        .buffered(3)
        .filter_map(futures::future::ready);

    pin!(stream);

    while let Some(next) = stream.next().await {
        println!("next: {}", next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_buffered_example() {
        let now = std::time::Instant::now();

        buffered_example().await;

        println!("test_buffered_example elapsed: {:?}", now.elapsed());
    }

    #[tokio::test]
    async fn test_unordered_example() {
        let now = std::time::Instant::now();

        unordered_example().await;

        println!("test_unordered_example elapsed: {:?}", now.elapsed());
    }

    #[tokio::test]
    async fn test_buffered_filter_example() {
        let now = std::time::Instant::now();

        buffered_filter_example().await;

        println!("test_buffered_filter_example elapsed: {:?}", now.elapsed());
    }

    #[tokio::test]
    async fn test_concurrent_filter_example() {
        let now = std::time::Instant::now();

        concurrent_filter_example().await;

        println!(
            "test_concurrent_filter_example elapsed: {:?}",
            now.elapsed()
        );
    }
}
