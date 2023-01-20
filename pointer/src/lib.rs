mod cell;
mod rc;
mod string;

#[cfg(test)]
mod tests {

    use crate::cell::Cell;
    use crate::rc::Rc as Rc1;
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
    fn rc_should_work() {
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

    struct SomeStruct {
        regular_field: u8,
        special_field: Cell<u8>,
    }

    #[test]
    fn cell_should_work() {
        let my_struct = SomeStruct {
            regular_field: 0,
            special_field: Cell::new(1),
        };

        let new_value = 100;

        // ERROR: `my_struct` is immutable
        // my_struct.regular_field = new_value;

        // WORKS: although `my_struct` is immutable, `special_field` is a `Cell`,
        // which can always be mutated
        my_struct.special_field.set(new_value);
        assert_eq!(my_struct.special_field.get(), new_value);
    }
}
