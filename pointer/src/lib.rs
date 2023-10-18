mod cell;
mod rc;
mod refcell;
mod string;

#[cfg(test)]
mod tests {

    use crate::cell::Cell;
    use crate::rc::Rc as Rc1;
    use crate::refcell::{RefCell, RefMut};
    use crate::string::*;
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::rc::{Rc, Weak};

    #[test]
    fn test_autoref() {
        #[derive(Clone)]
        struct Container<T>(Arc<T>);

        struct Container1<T>(Arc<T>);

        impl<T> Clone for Container1<T> {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }

        let foo = &Container(Arc::new(1i32));
        // SmartString is not Clone
        let bar = &Container(Arc::new(SmartString::Standard("ss".to_string())));

        // what is the type of _foo_clone & _bar_clone
        let _foo_clone = foo.clone();
        let _bar_clone = bar.clone();

        let foo1 = &Container1(Arc::new(1i32));
        // SmartString is not Clone
        let bar1 = &Container1(Arc::new(SmartString::Standard("ss".to_string())));

        // what is the type of _foo_clone1 & _bar_clone1
        let _foo1_clone = foo1.clone();
        let _bar1_clone = bar1.clone();
    }

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
        assert_eq!(2, Rc::strong_count(&rc));
        drop(rc1);
        assert_eq!(1, Rc::strong_count(&rc));
    }

    #[test]
    fn weak_ptr_should_work() {
        let rc = Rc::new(1);
        let rc1 = rc.clone();
        assert_eq!(2, Rc::strong_count(&rc));
        assert_eq!(0, Rc::weak_count(&rc));
        drop(rc1);
        assert_eq!(1, Rc::strong_count(&rc));
        assert_eq!(0, Rc::weak_count(&rc));
        let rc_weak = Rc::downgrade(&rc);
        assert_eq!(1, Rc::strong_count(&rc));
        assert_eq!(1, Rc::weak_count(&rc));
        assert_eq!(Some(Rc::new(1)), rc_weak.upgrade());
        drop(rc);
        assert_eq!(None, rc_weak.upgrade());
    }

    #[test]
    fn weak_ptr_drop() {
        #[allow(dead_code)]
        struct Foo {
            s: &'static str,
        }

        impl Drop for Foo {
            fn drop(&mut self) {
                println!("foo drop");
            }
        }

        let foo = Rc::new(Foo { s: "hello" });
        let foo1 = Rc::downgrade(&foo);
        let foo2 = Weak::clone(&foo1);

        drop(foo1);
        drop(foo);

        assert!(foo2.upgrade().is_none());
    }

    #[test]
    fn cell_should_work() {
        #[allow(dead_code)]
        struct SomeStruct {
            regular_field: u8,
            special_field: Cell<u8>,
        }

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

    #[test]
    fn refcell_should_work() {
        let shared_map: Rc1<RefCell<_>> = Rc1::new(RefCell::new(HashMap::new()));
        // Create a new block to limit the scope of the dynamic borrow
        {
            let mut map: RefMut<_> = shared_map.borrow_mut().unwrap();
            map.insert("africa", 92388);
            map.insert("kyoto", 11837);
            map.insert("piccadilly", 11826);
            map.insert("marbles", 38);
        }

        // Note that if we had not let the previous borrow of the cache fall out
        // of scope then the subsequent borrow would cause a dynamic thread panic.
        // This is the major hazard of using `RefCell`.
        let total: i32 = shared_map.borrow().unwrap().values().sum();
        assert_eq!(116089, total);
    }

    #[test]
    fn no_refcell_should_work() {
        let mut shared_map = HashMap::new();

        shared_map.insert("africa", 92388);
        shared_map.insert("kyoto", 11837);
        shared_map.insert("piccadilly", 11826);
        shared_map.insert("marbles", 38);

        let total: i32 = shared_map.values().sum();
        assert_eq!(116089, total);
    }

    struct A {
        reference: Rc<B>,
    }

    impl A {
        fn fn_a(&self) -> usize {
            1
        }
    }

    struct B {
        reference: RefCell1<Weak<A>>,
    }

    impl B {
        fn fn_b(&self) -> usize {
            2
        }
    }

    use std::cell::RefCell as RefCell1;
    use std::sync::Arc;

    #[test]
    fn circular_reference_should_work() {
        let b = Rc::new(B {
            reference: RefCell1::new(Weak::new()),
        });

        let a = A {
            reference: b.clone(),
        };

        let rc_a = Rc::new(a);

        *b.reference.borrow_mut() = Rc::downgrade(&rc_a);

        assert_eq!(2, b.fn_b());
        assert_eq!(1, b.reference.borrow().upgrade().unwrap().fn_a());
    }

    #[test]
    fn url_cow_should_work() {
        let url = url::Url::parse("http://cae.com/?a=foo&b=bar&c=hello%20world").unwrap();
        let mut pairs = url.query_pairs();

        assert_eq!(
            pairs.next(),
            Some((Cow::Borrowed("a"), Cow::Borrowed("foo")))
        );

        assert_eq!(
            pairs.next(),
            Some((Cow::Borrowed("b"), Cow::Borrowed("bar")))
        );

        assert_eq!(
            pairs.next(),
            Some((Cow::Borrowed("c"), Cow::Borrowed("hello world")))
        );
    }

    #[test]
    fn test_mut_borrow() {
        let mut vec: Vec<i32> = vec![0, 1, 2];
        let (ref0, ref1): (&mut Vec<i32>, &i32) = unsafe {
            let ptr0 = (&mut vec) as *mut Vec<i32>;
            let ptr1 = &vec[1];
            (&mut *ptr0, ptr1)
        };
        assert_eq!(*ref1, 1);

        for i in 0..100000000 {
            ref0.push(i);
        }
        // undefined behavior
        assert_ne!(*ref1, 1);
    }
}
