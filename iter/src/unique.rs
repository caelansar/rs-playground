use std::collections::HashSet;
use std::hash::Hash;

pub trait UniqueIterator: Iterator
where
    Self: Sized,
{
    fn unique(self) -> Unique<Self> {
        Unique {
            iter: self,
            seen: HashSet::new(),
        }
    }
}

pub struct Unique<I>
where
    I: Iterator,
{
    iter: I,
    seen: HashSet<I::Item>,
}

impl<I> Iterator for Unique<I>
where
    I: Iterator,
    I::Item: Clone + Eq + Hash,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find(|item| self.seen.insert(item.clone()))
    }
}

impl<T> UniqueIterator for T
where
    T: Iterator,
    T::Item: Clone,
{
}

#[cfg(test)]
mod tests {
    use super::UniqueIterator;

    #[test]
    fn test_unique_iterator() {
        let list = vec![1, 1, 2, 2, 4];
        let unique_list = list.into_iter().unique().collect::<Vec<_>>();
        assert_eq!(unique_list, vec![1, 2, 4]);

        let list = vec!["aa", "aa", "bb"];
        let unique_list = list.into_iter().unique().collect::<Vec<_>>();
        assert_eq!(unique_list, vec!["aa", "bb"]);
    }
}
