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
        self.start += 1;
        Some(retval)
    }
}
