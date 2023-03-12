pub trait RefIterator {
    type Item<'a>
    where
        Self: 'a;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}

pub struct StrRefHolder<'a> {
    strings: &'a str,
    i: usize,
}

impl<'a> StrRefHolder<'a> {
    pub fn new(s: &'a str) -> StrRefHolder<'a> {
        Self { strings: s, i: 0 }
    }
}

impl<'s> RefIterator for StrRefHolder<'s> {
    type Item<'a> = &'a str where 's: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.i >= self.strings.len() {
            None
        } else {
            self.i += 1;
            Some(&self.strings[..=self.i - 1])
        }
    }
}
