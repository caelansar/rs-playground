trait Mapper {
    type Item;

    type Result<U>;

    fn map<F, U>(self, f: F) -> Self::Result<U>
    where
        F: FnMut(Self::Item) -> U;
}

impl<T> Mapper for Option<T> {
    type Item = T;

    type Result<U> = Option<U>;

    fn map<F, U>(self, f: F) -> Self::Result<U>
    where
        F: FnMut(Self::Item) -> U,
    {
        Self::map(self, f)
    }
}

impl<T, E> Mapper for Result<T, E> {
    type Item = T;

    type Result<U> = Result<U, E>;

    fn map<F, U>(self, f: F) -> Self::Result<U>
    where
        F: FnMut(Self::Item) -> U,
    {
        Self::map(self, f)
    }
}

impl<T> Mapper for Vec<T> {
    type Item = T;

    type Result<U> = Vec<U>;

    fn map<F, U>(self, f: F) -> Self::Result<U>
    where
        F: FnMut(Self::Item) -> U,
    {
        self.into_iter().map(f).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uni_map<T, U, M, F>(mapper: M, f: F) -> M::Result<U>
    where
        M: Mapper<Item = T>,
        F: FnMut(T) -> U,
    {
        mapper.map(f)
    }

    #[test]
    fn map_option() {
        let v = Some(1);

        let v1 = uni_map(v, |x| x + 1);
        assert_eq!(Some(2), v1)
    }

    #[test]
    fn map_vec() {
        let v = vec![1, 2, 3];

        let v1 = uni_map(v, |x| x + 1);
        assert_eq!(vec![2, 3, 4], v1)
    }

    #[test]
    fn map_result() {
        let v: Result<i32, &str> = Ok(1);

        let v1 = uni_map(v, |x| x + 1);
        assert_eq!(Ok(2), v1)
    }
}
