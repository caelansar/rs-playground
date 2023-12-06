use std::io::{self, Write};

struct PartialWrite;

impl Write for PartialWrite {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if !buf.is_empty() {
            Ok(buf.len() / 2)
        } else {
            Ok(0)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::PartialWrite;
    use std::io::Write;

    #[test]
    fn test_write() {
        let mut writer = PartialWrite;
        let data = [1, 2, 3, 4, 5, 6, 7, 8];
        let bytes_written = writer.write(&data).unwrap();

        assert_eq!(data.len(), 8);
        assert_eq!(bytes_written, 4);
    }

    #[test]
    #[should_panic]
    fn test_write_all() {
        let mut writer = PartialWrite;
        let data = [1, 2, 3, 4, 5, 6, 7, 8];
        writer.write_all(&data).unwrap();
    }
}
