#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn test_hashmap_layout() {
        let map = HashMap::new();
        let mut map = explain("start", map);

        map.insert('a', 1);
        let mut map = explain("add a", map);
        map.insert('b', 2);
        map.insert('c', 3);

        let mut map = explain("add c", map);

        map.insert('d', 4);

        let mut map = explain("add d", map);

        map.remove(&'a');

        explain("end", map);
    }

    // struct RawTableInner<A> {
    // // Mask to get an index from a hash value. The value is one less than the
    // // number of buckets in the table.
    // bucket_mask: usize,

    // // [Padding], T1, T2, ..., Tlast, C1, C2, ...
    // //                                ^ points here
    // ctrl: NonNull<u8>,

    // // Number of elements that can be inserted before we need to grow the table
    // growth_left: usize,

    // // Number of elements in the table, only really used by len()
    // items: usize,

    // alloc: A,
    // }
    fn explain<K, V>(name: &str, map: HashMap<K, V>) -> HashMap<K, V> {
        let arr: [usize; 6] = unsafe { std::mem::transmute(map) };
        // The memory layout of a struct is undefined by default to allow for compiler
        // optimizations like field reordering. On my computer, HashMap is laid out as follows
        println!(
            "{}: bucket_mask 0x{:x}, ctrl 0x{:x}, growth_left: {}, items: {}",
            name, arr[1], arr[0], arr[2], arr[3]
        );
        unsafe { std::mem::transmute(arr) }
    }
}
