use core::fmt::Debug;
use futures::Future;
use std::io::Write;

pub trait KvIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next(&mut self) -> impl Future<Output = Option<Self::Item<'_>>>;
}

pub trait AsyncKvIterator {
    type Item<'a>: Debug
    where
        Self: 'a;
    async fn next(&mut self) -> Option<Self::Item<'_>>;
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
    type Item<'a> = (&'a str, &'a str);

    fn next(&mut self) -> impl Future<Output = Option<Self::Item<'_>>> {
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

impl AsyncKvIterator for TestIterator {
    type Item<'a> = (&'a str, &'a str);

    async fn next(&mut self) -> Option<Self::Item<'_>> {
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

#[allow(dead_code)]
async fn iterator(mut iter: impl for<'a> KvIterator<Item<'a> = (&'a str, &'a str)> + 'static) {
    while let Some(data) = iter.next().await {
        println!("key: {}, value: {}", data.0, data.1);
    }
}

#[allow(dead_code)]
async fn native_iterator(
    mut iter: impl for<'a> AsyncKvIterator<Item<'a> = (&'a str, &'a str)> + 'static,
) {
    while let Some(data) = iter.next().await {
        println!("key: {}, value: {}", data.0, data.1);
    }
}

async fn native_iterator1<A: AsyncKvIterator>(mut iter: A)
where
    for<'a> <A as AsyncKvIterator>::Item<'a>: Debug,
{
    while let Some(data) = iter.next().await {
        println!("data: {:?}", data);
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

    #[tokio::test]
    async fn test_native_async_trait() {
        let iter = TestIterator::new(0, 5);

        // native_iterator(iter).await;
        native_iterator1(iter).await;
    }
}
