mod my_rc;
mod string;

#[cfg(test)]
mod tests {

    use crate::my_rc::Rc as Rc1;
    use crate::string::*;
    use std::rc::Rc;

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

    #[test]
    fn my_rc_should_work() {
        let rc = Rc1::new(1);
        let rc1 = rc.clone();
        assert_eq!(2, *rc1 + 1);
        assert_eq!(1, *rc);
        drop(rc1);
    }

    #[test]
    fn std_rc_should_work() {
        let rc = Rc::new(1);
        let rc1 = rc.clone();
        assert_eq!(2, *rc1 + 1);
        assert_eq!(1, *rc);
        drop(rc1);
    }
}
