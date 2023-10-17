use std::os::raw::c_char;

extern "C" {
    fn strlen(s: *const c_char) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn strlen_works() {
        let s = "hello";
        let cs = CString::new(s).unwrap();

        assert_eq!(unsafe { strlen(cs.as_ptr()) }, 5);
    }
}
