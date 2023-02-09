use std::{ops::Deref, rc::Rc, sync::Arc};

pub trait PointerFamily {
    type Pointer<T>: Deref<Target = T>;

    fn new<T>(value: T) -> Self::Pointer<T>;
}

pub struct ArcFamily;
pub struct RcFamily;

impl PointerFamily for ArcFamily {
    type Pointer<T> = Arc<T>;

    fn new<T>(value: T) -> Self::Pointer<T> {
        Arc::new(value)
    }
}
impl PointerFamily for RcFamily {
    type Pointer<T> = Rc<T>;

    fn new<T>(value: T) -> Self::Pointer<T> {
        Rc::new(value)
    }
}
