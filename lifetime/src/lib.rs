pub fn strtok<'a>(s: &mut &'a str, delimiter: char) -> &'a str {
    if let Some(i) = s.find(delimiter) {
        let prefix = &s[..i];
        let suffix = &s[(i + delimiter.len_utf8())..];
        *s = suffix;
        prefix
    } else {
        let prefix = *s;
        *s = "";
        prefix
    }
}

pub struct StrSplit<'r> {
    remainder: Option<&'r str>,
    delimiter: char,
}

impl<'r> StrSplit<'r> {
    pub fn new(remainder: &'r str, delimiter: char) -> Self {
        Self {
            remainder: Some(remainder),
            delimiter,
        }
    }
}

impl<'r> Iterator for StrSplit<'r> {
    type Item = &'r str;

    fn next(&mut self) -> Option<Self::Item> {
        let remainder = self.remainder.as_mut()?;

        if let Some(next_delim) = remainder.find(self.delimiter) {
            let until_delimiter = &remainder[..next_delim];
            *remainder = &remainder[next_delim + self.delimiter.len_utf8()..];
            Some(until_delimiter)
        } else {
            println!("remainder {:?}", self.remainder);
            self.remainder.take()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strtok_should_work() {
        let s = "hello world".to_owned();
        let mut s1 = s.as_str();
        let prefix = strtok(&mut s1, ' ');
        assert_eq!("hello", prefix);
        assert_eq!("world", s1);
        assert_eq!("hello world", s.as_str());
    }

    #[test]
    fn str_split_should_work() {
        #[allow(non_camel_case_types)]
        struct testcase {
            input: &'static str,
            output: Vec<&'static str>,
        }

        let testcases = vec![
            testcase {
                input: "a b c d",
                output: vec!["a", "b", "c", "d"],
            },
            testcase {
                input: "a",
                output: vec!["a"],
            },
            testcase {
                input: "a b ",
                output: vec!["a", "b", ""],
            },
            testcase {
                input: "👿",
                output: vec!["👿"],
            },
        ];

        testcases.into_iter().for_each(|testcase| {
            let parts: Vec<_> = StrSplit::new(testcase.input, ' ').collect();
            assert_eq!(testcase.output, parts);
        })
    }
}
