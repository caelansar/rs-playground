pub trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}

pub struct WindowsMut<'t, T> {
    slice: &'t mut [T],
    start: usize,
    window_size: usize,
}

impl<'t, T> WindowsMut<'t, T> {
    pub fn new(slice: &'t mut [T], start: usize, window_size: usize) -> Self {
        Self {
            slice,
            start,
            window_size,
        }
    }
}

impl<'t, T> LendingIterator for WindowsMut<'t, T> {
    type Item<'a> = &'a mut [T] where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let retval = self.slice[self.start..].get_mut(..self.window_size)?;
        self.start += self.window_size;
        Some(retval)
    }
}

pub trait IteratorExt: Iterator {
    fn window_count(self, count: u32) -> Windows<Self>
    where
        Self: Sized,
    {
        Windows { iter: self, count }
    }
}

impl<T: ?Sized> IteratorExt for T where T: Iterator {}

pub struct Windows<I> {
    pub(crate) iter: I,
    count: u32,
}

impl<I: Iterator> Iterator for Windows<I> {
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let data = (0..self.count)
            .filter_map(|_| self.iter.next())
            .collect::<Vec<_>>();
        if data.is_empty() {
            None
        } else {
            Some(data)
        }
    }
}

struct ByteIter<'a> {
    data: &'a [u8],
}

// impl<'a> ByteIter<'a> {
//     fn next<'b>(&'b mut self) -> Option<&'b u8> {
//         if self.data.is_empty() {
//             None
//         } else {
//             let byte = &self.data[0];
//             self.data = &self.data[1..];
//             Some(byte)
//         }
//     }
// }

impl<'a> ByteIter<'a> {
    fn next<'b>(&'b mut self) -> Option<&'a u8> {
        if self.data.is_empty() {
            None
        } else {
            let byte = &self.data[0];
            self.data = &self.data[1..];
            Some(byte)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ByteIter;

    #[test]
    fn byte_iter_should_work() {
        let mut byte_iter = ByteIter { data: b"112233" };
        let b1 = byte_iter.next();
        let b2 = byte_iter.next();
        assert!(b1 == b2);
    }
}
