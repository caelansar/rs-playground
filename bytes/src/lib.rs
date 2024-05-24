#![feature(slice_ptr_get)]

use std::sync::Arc;

struct Bytes {
    ptr: usize,
}

const KIND_MASK: usize = 0b1;

impl Bytes {
    fn new_vec(data: Vec<u8>) -> Self {
        println!("{}", data.as_ptr() as usize);
        let ptr = data.as_ptr() as usize | 0x1;
        std::mem::forget(data);
        Bytes { ptr }
    }

    fn new_arc(data: Arc<[u8]>) -> Self {
        let ptr = Arc::into_raw(data).as_ptr() as usize;
        Bytes { ptr }
    }

    fn is_vec(&self) -> bool {
        self.ptr & 0x1 == 1
    }

    fn is_arc(&self) -> bool {
        self.ptr & 0x1 == 0
    }

    fn as_vec(&self) -> Vec<u8> {
        assert!(self.is_vec());
        println!("{}", self.ptr & !KIND_MASK);
        unsafe { Vec::from_raw_parts((self.ptr & !KIND_MASK) as *mut u8, 5, 6) }
    }

    fn as_arc(&self) -> Arc<[u8]> {
        assert!(self.is_arc());

        let slice = unsafe { std::slice::from_raw_parts(self.ptr as *const u8, 5) };
        unsafe { Arc::from_raw(slice) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes() {
        let vec_data = vec![1, 2, 3, 4, 5];
        let arc_data: Arc<[u8]> = Arc::from([6, 7, 8, 9, 10]);

        let bytes_vec = Bytes::new_vec(vec_data);
        let bytes_arc = Bytes::new_arc(arc_data);

        if bytes_vec.is_vec() {
            assert_eq!(vec![1, 2, 3, 4, 5], bytes_vec.as_vec());
        }

        if bytes_arc.is_arc() {
            assert_eq!(Arc::from(vec![6, 7, 8, 9, 10]), bytes_arc.as_arc());
        }
    }
}
