mod unique;

use std::collections::HashMap;
use std::hash::Hash;

pub trait FromIterator<T> {
    fn my_from_iter<I>(iter: I) -> Self
    where
        I: Iterator<Item = T>;
}

pub trait Iterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;

    fn collect<B>(self) -> B
    where
        B: FromIterator<Self::Item>,
        Self: Sized,
    {
        B::my_from_iter(self)
    }

    fn map<B, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> B,
    {
        Map::new(self, f)
    }

    fn filter<P>(self, predicate: P) -> Filter<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        Filter::new(self, predicate)
    }
}

impl<T> FromIterator<T> for Vec<T> {
    fn my_from_iter<I>(mut iter: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        let mut vec = Vec::new();

        while let Some(x) = iter.next() {
            vec.push(x);
        }

        vec
    }
}

impl<K, V> FromIterator<(K, V)> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn my_from_iter<I: Iterator<Item = (K, V)>>(mut iter: I) -> HashMap<K, V> {
        let mut map = HashMap::new();
        while let Some((k, v)) = iter.next() {
            map.insert(k, v);
        }
        map
    }
}

pub struct Map<I, F> {
    iter: I,
    f: F,
}

impl<I, F> Map<I, F> {
    fn new(iter: I, f: F) -> Self {
        Self { iter, f }
    }
}

impl<B, I, F> Iterator for Map<I, F>
where
    I: Iterator,
    F: FnMut(I::Item) -> B,
{
    type Item = B;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(x) = self.iter.next() {
            Some((self.f)(x))
        } else {
            None
        }
    }
}

pub struct Filter<I, P> {
    iter: I,
    p: P,
}

impl<I, P> crate::Filter<I, P> {
    fn new(iter: I, p: P) -> Self {
        Self { iter, p }
    }
}

impl<I, P> Iterator for crate::Filter<I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(x) = self.iter.next() {
            if (self.p)(&x) {
                return Some(x);
            }
        }
        None
    }
}

pub struct SliceIterator<'a, T> {
    data: &'a [T],
    pos: usize,
}

impl<'a, T> SliceIterator<'a, T> {
    pub(crate) fn new(data: &'a [T]) -> Self {
        SliceIterator { data, pos: 0 }
    }
}

impl<T: Clone> Iterator for SliceIterator<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.data.len() {
            None
        } else {
            let result = Some(&self.data[self.pos]);
            self.pos += 1;
            result.cloned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter_works() {
        let mut slice_iter = SliceIterator::new(&[1, 2, 3, 4, 5]);
        assert_eq!(slice_iter.next(), Some(1));

        let rest: Vec<_> = slice_iter.collect();
        assert_eq!(rest, vec![2, 3, 4, 5]);

        let tuples = &[("a".to_string(), 1), ("b".to_string(), 2)];
        let tuples_iter = SliceIterator::new(tuples);

        let map: HashMap<_, _> = tuples_iter.collect();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("a"), Some(&1));
        assert_eq!(map.get("b"), Some(&2));
    }

    #[test]
    fn map_works() {
        let slice_iter = SliceIterator::new(&[1, 2, 3, 4, 5]);

        let map_list: Vec<_> = slice_iter.map(|x| x - 1).collect();
        assert_eq!(map_list, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn filter_works() {
        let slice_iter = SliceIterator::new(&[1, 2, 3, 4, 5]);

        let filter_list: Vec<_> = slice_iter.filter(|x| *x > 3).collect();
        assert_eq!(filter_list, vec![4, 5]);
    }
}
