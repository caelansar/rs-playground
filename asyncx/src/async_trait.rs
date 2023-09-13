use futures::Future;

pub trait KvIterator {
    type Next<'a>: Future<Output = Option<(&'a str, &'a str)>>
    where
        Self: 'a;

    fn next(&mut self) -> Self::Next<'_>;
}

pub struct TestIterator {
    idx: usize,
    max: usize,
}

impl KvIterator for TestIterator {
    type Next<'a> = impl Future<Output = Option<(&'a str, &'a str)>>;

    fn next(&mut self) -> Self::Next<'_> {
        async move {
            self.idx += 1;
            if self.idx > self.max {
                return None;
            }
            Some(("k", "v"))
        }
    }
}

#[allow(dead_code)]
async fn iterator(mut iter: impl KvIterator) {
    while let Some(data) = iter.next().await {
        println!("key: {}, value: {}", data.0, data.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_trait() {
        let iter = TestIterator { idx: 0, max: 5 };

        iterator(iter).await;
    }
}
