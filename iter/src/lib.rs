use std::collections::HashMap;
use std::hash::Hash;

pub trait MyFromIterator<T> {
    fn my_from_iter<I>(iter: I) -> Self
    where
        I: MyIterator<Item = T>;
}

pub trait MyIterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;

    fn collect<B>(self) -> B
    where
        B: MyFromIterator<Self::Item>,
        Self: Sized,
    {
        B::my_from_iter(self)
    }
}

impl<T> MyFromIterator<T> for Vec<T> {
    fn my_from_iter<I>(mut iter: I) -> Self
    where
        I: MyIterator<Item = T>,
    {
        let mut vec = Vec::new();

        while let Some(x) = iter.next() {
            vec.push(x);
        }

        vec
    }
}

impl<K, V> MyFromIterator<(K, V)> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn my_from_iter<I: MyIterator<Item = (K, V)>>(mut iter: I) -> HashMap<K, V> {
        let mut map = HashMap::new();
        while let Some((k, v)) = iter.next() {
            map.insert(k, v);
        }
        map
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

impl<T: Clone> MyIterator for SliceIterator<'_, T> {
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
        let mut tuples_iter = SliceIterator::new(tuples);

        let map: HashMap<_, _> = tuples_iter.collect();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("a"), Some(&1));
        assert_eq!(map.get("b"), Some(&2));
    }
}
