async fn fib(n: u32) -> u32 {
    match n {
        1 | 2 => 1,
        _ => Box::pin(fib(n - 1)).await + Box::pin(fib(n - 2)).await,
    }
}

#[cfg(test)]
mod tests {
    use crate::recursion::fib;

    #[tokio::test]
    async fn test_recursion() {
        let v = fib(10).await;
        assert_eq!(55, v)
    }
}
