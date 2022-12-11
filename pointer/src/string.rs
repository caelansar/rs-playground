use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

const MAX_LENGTH: usize = 30;

pub struct MiniString {
    len: u8,
    data: [u8; MAX_LENGTH],
}

impl MiniString {
    fn new(v: impl AsRef<str>) -> Self {
        let bytes = v.as_ref().as_bytes();
        let len = bytes.len();
        let mut data = [0u8; MAX_LENGTH];
        data[..len].copy_from_slice(bytes);
        Self {
            len: len as u8,
            data,
        }
    }
}

impl Deref for MiniString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        std::str::from_utf8(&self.data[..self.len as usize]).unwrap()
    }
}

impl Debug for MiniString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.deref())
    }
}

/// when length of SmartString is less than MAX_LENGTH, use
/// bytes in stack directly, otherwise, use `String`
///
// memory layout
//
//                      +-----+-----+------------------------------------------------------------------------------------+
//SmartString::Inline   |  0  | len |                                 data                                               |
//                      +-----+-----+------------------------------------------------------------------------------------+
//
//                      +-----+-----------------------------+--------------------+-----------------+---------------------+
//SmartString::Standard |  1  |         padding             |   pointer          |     cap         |        len          |
//                      +-----+-----------------------------+--------------------+-----------------+---------------------+
#[derive(Debug)]
pub enum SmartString {
    Inline(MiniString),
    Standard(String),
}

impl Deref for SmartString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match *self {
            Self::Inline(ref v) => v.deref(),
            Self::Standard(ref v) => v.deref(),
        }
    }
}
impl From<&str> for SmartString {
    fn from(s: &str) -> Self {
        if s.len() > MAX_LENGTH {
            Self::Standard(s.to_owned())
        } else {
            Self::Inline(MiniString::new(s))
        }
    }
}

impl From<String> for SmartString {
    fn from(s: String) -> Self {
        if s.len() > MAX_LENGTH {
            Self::Standard(s)
        } else {
            Self::Inline(MiniString::new(s))
        }
    }
}

impl Display for SmartString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.deref())
    }
}
