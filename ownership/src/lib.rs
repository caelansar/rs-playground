#![feature(vec_into_raw_parts)]

use std::fmt::Debug;

fn extend_vec(v: &mut Vec<i32>) {
    v.extend(2..33)
}

fn print_vec<T: Debug>(data: Vec<T>) {
    println!("data: {:?}", data);
    let (ptr, len, cap) = data.into_raw_parts();
    println!(
        "ptr: 0x{:x}, len: {}, cap: {}, ",
        ptr as *const usize as usize, len, cap
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_realloc() {
        let mut v = vec![1];

        println!("heap start: {:p}", &v[0] as *const i32);

        // allocate a larger size array in the heap
        // copy the elements from the current location to the new array
        // update the pointer to the new memory location
        extend_vec(&mut v);

        println!("new heap start: {:p}", &v[0] as *const i32);

        v.truncate(10);
        assert_eq!(v.capacity(), 32);

        v.shrink_to_fit();
        assert_eq!(v.capacity(), 10);

        print_vec(v);
    }
}
