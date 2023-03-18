#[derive(Debug, PartialEq)]
pub struct Buffer {
    buffer: String,
}

pub struct Render {
    current_buffer: Buffer,
    next_buffer: Option<Buffer>,
}

impl Render {
    fn update_buffer(&mut self, buf: String) {
        if self.next_buffer.is_some() {
            self.current_buffer = self.next_buffer.take().unwrap();
            self.next_buffer = Some(Buffer { buffer: buf });
        }
    }
}

pub struct Render1 {
    current_buffer: Buffer,
    next_buffer: Buffer,
}

impl Render1 {
    fn update_buffer(&mut self, buf: String) {
        self.current_buffer = std::mem::replace(&mut self.next_buffer, Buffer { buffer: buf })
    }
}

#[cfg(test)]
mod tests {
    use crate::{Buffer, Render, Render1};

    #[test]
    fn update_buffer_should_works() {
        let mut r = Render {
            current_buffer: Buffer {
                buffer: "a".to_string(),
            },
            next_buffer: Some(Buffer {
                buffer: "b".to_string(),
            }),
        };
        r.update_buffer("c".to_string());
        assert_eq!(
            Buffer {
                buffer: "b".to_string()
            },
            r.current_buffer
        );
        assert_eq!(
            Some(Buffer {
                buffer: "c".to_string()
            }),
            r.next_buffer
        );

        let mut r1 = Render1 {
            current_buffer: Buffer {
                buffer: "a".to_string(),
            },
            next_buffer: Buffer {
                buffer: "b".to_string(),
            },
        };
        r1.update_buffer("c".to_string());
        assert_eq!(
            Buffer {
                buffer: "b".to_string()
            },
            r1.current_buffer
        );
        assert_eq!(
            Buffer {
                buffer: "c".to_string()
            },
            r1.next_buffer
        );
    }
}
