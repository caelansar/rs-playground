pub trait Mapper {
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
