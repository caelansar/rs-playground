use futures::Future;
use std::io::Write;

pub trait KvIterator {
    type Next<'a>: Future<Output = Option<(&'a str, &'a str)>>
    where
        Self: 'a;

    fn next(&mut self) -> Self::Next<'_>;
}

pub struct TestIterator {
    idx: usize,
    max: usize,
    key: Vec<u8>,
    value: Vec<u8>,
}

#[allow(dead_code)]
impl TestIterator {
    pub fn new(idx: usize, max: usize) -> Self {
        Self {
            idx,
            max,
            key: Vec::new(),
            value: Vec::new(),
        }
    }
}

impl KvIterator for TestIterator {
    type Next<'a> = impl Future<Output = Option<(&'a str, &'a str)>>;

    fn next(&mut self) -> Self::Next<'_> {
        async move {
            self.idx += 1;
            if self.idx > self.max {
                return None;
            }

            // manipulate the keys and values without repeatedly allocating and deallocating memory
            // for them, instead reusing the same allocated space as much as possible

            self.key.clear();
            write!(&mut self.key, "key_{}", self.idx).unwrap();

            self.value.clear();
            write!(&mut self.value, "value_{}", self.idx).unwrap();

            unsafe {
                Some((
                    std::str::from_utf8_unchecked(&self.key[..]),
                    std::str::from_utf8_unchecked(&self.value[..]),
                ))
            }
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
        let iter = TestIterator::new(0, 5);

        iterator(iter).await;
    }
}
