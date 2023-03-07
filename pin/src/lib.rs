#![allow(unused_variables)]
#![feature(negative_impls)]

use std::pin::Pin;

#[derive(Debug)]
pub struct Test {
    a: String,
    b: *const String,
}

impl !Unpin for Test {}

impl Test {
    fn new(txt: &str) -> Self {
        let a = String::from(txt);
        Test {
            a,
            b: std::ptr::null(),
        }
    }
    fn init<'a>(self: Pin<&'a mut Self>) {
        let self_ptr: *const String = &self.a;
        let this = unsafe { self.get_unchecked_mut() };
        this.b = self_ptr;
    }

    fn a<'a>(self: Pin<&'a Self>) -> &'a str {
        &self.get_ref().a
    }

    fn b<'a>(self: Pin<&'a Self>) -> &'a String {
        unsafe { &*(self.b) }
    }
}

#[cfg(test)]
mod tests {
    use std::pin::Pin;

    use crate::Test;

    #[test]
    fn it_works() {
        let mut test1 = Test::new("test1");
        let mut test1 = unsafe { Pin::new_unchecked(&mut test1) };
        Test::init(test1.as_mut());

        let mut test2 = Test::new("test2");
        let mut test2 = unsafe { Pin::new_unchecked(&mut test2) };
        Test::init(test2.as_mut());

        println!(
            "a: {}, b: {}",
            Test::a(test1.as_ref()),
            Test::b(test1.as_ref())
        );
        println!(
            "a: {}, b: {}",
            Test::a(test2.as_ref()),
            Test::b(test2.as_ref())
        );
    }
}
