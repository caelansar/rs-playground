struct TraitObject {
    data: *mut (),
    vtable: *mut (),
}

struct FooVtable {
    size: usize,
    align: usize,
    method: fn(*const ()) -> String,
}

trait Foo {
    fn method(&self) -> String;
}

impl Foo for &u8 {
    fn method(&self) -> String {
        "u8".to_string()
    }
}

impl Foo for &String {
    fn method(&self) -> String {
        "String".to_string()
    }
}

// u8:

fn call_method_on_u8(x: *const ()) -> String {
    // the compiler guarantees that this function is only called
    // with `x` pointing to a u8
    let byte: &u8 = unsafe { &*(x as *const u8) };

    byte.method()
}

static mut Foo_for_u8_vtable: FooVtable = FooVtable {
    size: 1,
    align: 1,

    // cast to a function pointer
    method: call_method_on_u8 as fn(*const ()) -> String,
};

// String:

fn call_method_on_String(x: *const ()) -> String {
    // the compiler guarantees that this function is only called
    // with `x` pointing to a String
    let string: &String = unsafe { &*(x as *const String) };

    string.method()
}

static mut Foo_for_String_vtable: FooVtable = FooVtable {
    // values for a 64-bit computer, halve them for 32-bit ones
    size: 24,
    align: 8,

    method: call_method_on_String as fn(*const ()) -> String,
};

#[cfg(test)]
mod tests {
    use super::*;

    use std::fmt::{Debug, Display, Error, Formatter};
    use std::mem::transmute;
    use std::ops::Deref;

    #[test]
    fn test_trait_object() {
        let mut a = "foo".to_string();

        // let b: &Foo = &a;
        let b = TraitObject {
            // store the data
            data: a.as_mut_ptr() as *mut (),
            // store the methods
            vtable: unsafe { &mut Foo_for_String_vtable as *mut FooVtable as *mut () },
        };
        // b.method();
        let r = unsafe { ((*(b.vtable as *mut FooVtable)).method)(b.data as *const ()) };
        println!("string trait object: {}", r);

        let x = 1u8;

        // let y: &Foo = x;
        let y = TraitObject {
            // store the data
            data: x as *mut (),
            // store the methods
            vtable: unsafe { &mut Foo_for_u8_vtable as *mut FooVtable as *mut () },
        };
        // y.method();
        let r = unsafe { ((*(y.vtable as *mut FooVtable)).method)(y.data as *const ()) };
        println!("u8 trait object: {}", r);
    }

    #[test]
    fn test_trait_object2() {
        let v = vec![1, 2, 3, 4];

        let a: &Vec<i32> = &v;
        let b0: &[i32] = &v;
        let b1: &[i32] = v.deref();
        let c: &dyn Debug = &v;

        println!("a: {}", a as *const _ as usize);
        println!("b0: {:?}", unsafe { transmute::<_, (usize, usize)>(b0) });
        println!("b1: {:?}", unsafe { transmute::<_, (usize, usize)>(b1) });
        println!("c: {:?}", unsafe { transmute::<_, (usize, usize)>(c) });

        struct Wrap<'a, T>(&'a fn(&T, &mut Formatter) -> Result<(), Error>, &'a T);

        impl<'a, T> Debug for Wrap<'a, T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let _ = f.write_str(">>Wrap Debug<<");
                self.0(self.1, f)
            }
        }

        let (_, vtable) = unsafe { transmute::<_, (usize, usize)>(c) };

        let vtable_layout = unsafe { &*(vtable as *const [usize; 4]) };
        println!(
            "drop: {}, size: {}, align: {}",
            vtable_layout[0], vtable_layout[1], vtable_layout[2]
        );
        println!("method {:?}", &vtable_layout[3]);

        println!("method (by offset){}", unsafe {
            &*((vtable as *const usize).offset(3) as *const usize)
        });

        let fmt_fn = unsafe {
            &*((&vtable_layout[3] as *const usize)
                as *const fn(&Vec<i32>, &mut Formatter) -> Result<(), Error>)
        };
        println!("{:?}", Wrap(fmt_fn, &v));
    }

    #[test]
    fn test_trait_object3() {
        let s1 = String::from("hello world!");
        let s2 = String::from("goodbye world!");

        let w1: &dyn Display = &s1;
        let w2: &dyn Debug = &s1;

        let w3: &dyn Display = &s2;
        let w4: &dyn Debug = &s2;

        let (addr1, vtable1) = unsafe { transmute::<_, (usize, usize)>(w1) };
        let (addr2, vtable2) = unsafe { transmute::<_, (usize, usize)>(w2) };
        let (addr3, vtable3) = unsafe { transmute::<_, (usize, usize)>(w3) };
        let (addr4, vtable4) = unsafe { transmute::<_, (usize, usize)>(w4) };

        println!("s1: {:p}, s2: {:p}", &s1, &s2);

        println!("addr1: 0x{:x}, vtable1: 0x{:x}", addr1, vtable1);
        println!("addr2: 0x{:x}, vtable2: 0x{:x}", addr2, vtable2);
        println!("addr3: 0x{:x}, vtable3: 0x{:x}", addr3, vtable3);
        println!("addr4: 0x{:x}, vtable4: 0x{:x}", addr4, vtable4);

        assert_eq!(addr1, addr2);
        assert_eq!(addr3, addr4);

        assert_eq!(vtable1, vtable3);
        assert_eq!(vtable2, vtable4);
    }
}
