use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
struct MyString(String);

impl From<&str> for MyString {
    fn from(s: &str) -> Self {
        MyString(s.to_string())
    }
}

impl Drop for MyString {
    fn drop(&mut self) {
        println!("drop: {}", self.0);
    }
}

impl Deref for MyString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MyString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[test]
fn test_manually() {
    let mut s = ManuallyDrop::new(MyString::from("Hello World!"));

    s.truncate(5);
    println!("s: {:?}", s);

    // comment this
    let _: MyString = ManuallyDrop::into_inner(s);
}
