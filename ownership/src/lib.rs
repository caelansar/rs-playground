use std::{fmt::Debug, mem};

fn extend_vec(v: &mut Vec<i32>) {
    (2..6).into_iter().for_each(|i| v.push(i));
}

fn print_vec<T: Debug>(data: Vec<T>) {
    println!("data: {:?}", data);
    let p = unsafe { mem::transmute::<_, [usize; 3]>(data) };
    println!("cap: {}, ptr: 0x{:x}, size: {}", p[0], p[1], p[2]);
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
