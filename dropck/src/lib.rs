#![feature(dropck_eyepatch)]

use std::fmt::Debug;
use std::marker::PhantomData;

struct MyBox<T> {
    name: *mut T,
    _phantom: PhantomData<T>,
}
impl<T> MyBox<T> {
    fn new(init: T) -> Self {
        Self {
            name: Box::into_raw(Box::new(init)),
            _phantom: Default::default(),
        }
    }
}

unsafe impl<#[may_dangle] T> Drop for MyBox<T> {
    fn drop(&mut self) {
        {
            unsafe {
                let _ = Box::from_raw(self.name);
            }
        }
    }
}

struct Bad<T: Debug>(T);
impl<T: Debug> Drop for Bad<T> {
    fn drop(&mut self) {
        println!("{:?}", self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropck_works() {
        let _a;
        let _s = "hello".to_owned();
        _a = MyBox::new(_s);
    }

    #[test]
    fn dropck_with_ref_works() {
        let _a;
        let _s = "hello".to_owned();
        _a = MyBox::new(&_s);
    }

    // #[test]
    // fn t_is_drop_wont_compile() {
    //     let _a;
    //     let _s = "bad".to_owned();
    //     let _b = Bad(&_s); // borrowed value does not live long enough
    //     _a = MyBox::new(_b);
    // }
}
