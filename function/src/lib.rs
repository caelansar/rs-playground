#![feature(fn_traits)]
#![feature(unboxed_closures)]

#[cfg(test)]
mod tests {
    use std::mem::{size_of_val, transmute};

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
        let mut x: fn(i32) -> i32 = foo; // coercion
        println!("{}", type_name_of_val(&x));
        assert_eq!(8, size_of_val(&x));
        x = bar;
        println!("{}", type_name_of_val(&x));
        assert_eq!(8, size_of_val(&x));
    }

    #[test]
    fn test_cast() {
        let ptr = foo as fn(i32) -> i32;
        let ptr_addr = ptr as usize;

        println!("{}", ptr_addr);

        let raw_ptr = ptr_addr as *const ();
        let ptr = unsafe { transmute::<_, fn(i32) -> i32>(raw_ptr) };
        assert_eq!(0, ptr(1));
    }

    #[test]
    fn test_manually_closure_struct() {
        let name = "cae".to_string();
        let vec = vec![1, 2, 3];
        let v = &vec;
        let data = (1, 2, 3);
        let closure = move |x: i32| {
            println!("param: {x}");
            println!("data: {:?}", data);
            println!("v: {:?}, name: {:?}", v, name.clone());
        };
        assert_eq!(
            size_of_val(&closure),
            24/*name*/ + 12/*data*/ + 4/*v*/ + 8 /*padding*/
        );
        closure(1);
        println!("vec: {:?}", vec); // still valid

        struct Closure<Captured> {
            captured: Captured,
        }

        impl<'a> FnOnce<(i32,)> for Closure<((i32, i32, i32), &'a Vec<i32>, String)> {
            type Output = ();
            extern "rust-call" fn call_once(self, args: (i32,)) {
                self.call(args)
            }
        }

        impl<'a> FnMut<(i32,)> for Closure<((i32, i32, i32), &'a Vec<i32>, String)> {
            extern "rust-call" fn call_mut(&mut self, args: (i32,)) {
                self.call(args)
            }
        }

        impl<'a> Fn<(i32,)> for Closure<((i32, i32, i32), &'a Vec<i32>, String)> {
            extern "rust-call" fn call(&self, (x,): (i32,)) {
                println!("param: {x}");
                println!("data: {:?}", self.captured.0);
                println!("v: {:?}, name: {:?}", self.captured.1, self.captured.2);
            }
        }

        let name = "cae".to_string();
        let vec = vec![1, 2, 3];
        let v = &vec;
        let data = (1, 2, 3);
        let manually_closure = Closure {
            captured: (data, v, name),
        };
        manually_closure(1);
        println!("vec: {:?}", vec); // still valid
    }
}
