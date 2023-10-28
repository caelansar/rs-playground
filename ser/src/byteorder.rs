use std::{mem, slice};

trait AsSlice {
    unsafe fn as_u8_slice(&self) -> &[u8];
}

impl<T: Sized> AsSlice for T {
    unsafe fn as_u8_slice(&self) -> &[u8] {
        slice::from_raw_parts((self as *const T) as *const u8, mem::size_of::<T>())
    }
}

#[test]
fn test_to_u8_slice() {
    #[derive(Debug)]
    struct MyStruct {
        id: u8,
        data: [u8; 10],
    }
    let my_struct = MyStruct {
        id: 0,
        data: [2; 10],
    };
    let bytes: &[u8] = unsafe { my_struct.as_u8_slice() };
    assert_eq!([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0], bytes);
}
