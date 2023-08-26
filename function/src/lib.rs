#[cfg(test)]
mod tests {
    use std::mem::size_of_val;

    use std::any::type_name;

    fn type_name_of_val<T: ?Sized>(_: &T) -> &'static str {
        type_name::<T>()
    }

    fn foo(_: i32) -> i32 {
        0
    }

    fn bar(_: i32) -> i32 {
        1
    }

    #[test]
    fn test_function_item() {
        #[allow(unused_mut)]
        let mut x = foo;
        println!("{}", type_name_of_val(&x));
        assert_eq!(0, size_of_val(&x));
        // x = bar; ❌
    }

    #[test]
    fn test_function_pointer() {
        let mut x: fn(i32) -> i32 = foo;
        println!("{}", type_name_of_val(&x));
        assert_eq!(8, size_of_val(&x));
        x = bar;
        println!("{}", type_name_of_val(&x));
        assert_eq!(8, size_of_val(&x));
    }
}
