mod string;

#[cfg(test)]
mod tests {

    use crate::string::*;

    #[test]
    fn smart_string_should_work() {
        let a: SmartString = "aaaaa".into();
        let b: SmartString = "b".repeat(31).into();

        println!("{}", std::mem::size_of::<MiniString>());
        println!("{}", std::mem::size_of::<SmartString>());
        println!("{}", a);
        println!("{}", b);

        assert!(a.starts_with("aa"));
        assert!(b.starts_with("bb"))
    }
}
