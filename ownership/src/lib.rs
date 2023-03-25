#![feature(vec_into_raw_parts)]

use std::{fmt::Debug, mem};

fn extend_vec(v: &mut Vec<i32>) {
    (2..6).into_iter().for_each(|i| v.push(i));
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

        print_vec(v);
    }
}
