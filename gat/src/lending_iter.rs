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

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>> {
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
