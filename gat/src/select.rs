pub trait Select<T> {
    type Output<'a>
    where
        T: 'a,
        Self: 'a;

    fn select<'a>(self, slice: &'a [T]) -> Self::Output<'a>;
}

impl<T> Select<T> for usize {
    type Output<'a>
        = &'a T
    where
        T: 'a,
        Self: 'a;

    fn select<'a>(self, slice: &'a [T]) -> Self::Output<'a> {
        &slice[self]
    }
}

impl<T> Select<T> for &[usize] {
    type Output<'a>
        = impl Iterator<Item = &'a T>
    where
        T: 'a,
        Self: 'a;

    fn select<'a>(self, slice: &'a [T]) -> Self::Output<'a> {
        self.iter().map(|i| &slice[*i])
    }
}

impl<T, const N: usize> Select<T> for [usize; N] {
    type Output<'a>
        = impl Iterator<Item = &'a T>
    where
        T: 'a,
        Self: 'a;

    fn select<'a>(self, slice: &'a [T]) -> Self::Output<'a> {
        self.into_iter().map(|i| &slice[i])
    }
}

#[allow(dead_code)]
pub fn get<T, S: Select<T>>(data: &[T], selector: S) -> S::Output<'_> {
    selector.select(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select() {
        let v = get(&[1, 2, 3, 4], 3);
        assert_eq!(*v, 4);

        let select = [1, 2];
        let mut s = get(&[1, 2, 4], select.as_slice());
        assert_eq!(s.next().map(|x| *x), Some(2));
        assert_eq!(s.next().map(|x| *x), Some(4));

        let mut s = get(&[1, 2, 4], select);
        assert_eq!(s.next().map(|x| *x), Some(2));
        assert_eq!(s.next().map(|x| *x), Some(4));
    }
}
