mod async_lifetime;

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

struct Interface<'a, 'b: 'a> {
    manager: &'a mut Manager<'b>,
}

impl<'a, 'b: 'a> Drop for Interface<'a, 'b> {
    fn drop(&mut self) {
        println!("interface drop")
    }
}

impl<'a, 'b: 'a> Interface<'a, 'b> {
    pub fn noop(self) {
        println!("interface consumed");
    }
}

struct Manager<'a> {
    text: &'a str,
}

struct List<'a> {
    manager: Manager<'a>,
}

impl Drop for List<'_> {
    fn drop(&mut self) {
        println!("list drop")
    }
}

impl<'a> List<'a> {
    pub fn get_interface<'b>(&'b mut self) -> Interface<'b, 'a>
    where
        'a: 'b,
    {
        Interface {
            manager: &mut self.manager,
        }
    }
}

fn use_list(list: &List) {
    println!("{}", list.manager.text);
}

#[cfg(test)]
mod tests {
    use super::*;

    // fn return_first<'a>(a: &'a mut [u8], b: &'a mut [u8]) -> &'a mut [u8] {
    //     a
    // }

    // b has a different lifetime
    fn return_first<'a>(a: &'a mut [u8], b: &'_ mut [u8]) -> &'a mut [u8] {
        a
    }

    #[test]
    fn test_mut_lifetime() {
        let mut a1 = [1, 2, 3];
        let mut a2 = [4, 5, 6];
        let r = return_first(&mut a1, &mut a2);
        a2[0] = 100;
        r[0] = 100;
    }

    #[test]
    fn borrow_should_work() {
        let mut list = List {
            manager: Manager { text: "hello" },
        };

        list.get_interface().noop();

        println!("Interface should be dropped here and the borrow released");

        use_list(&list);
    }

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

    #[test]
    fn copy_test() {
        let mut a = Some(Box::new(4));
        let b = a.as_ref(); // create a new option
        let c = &b;
        let c1 = &b;
        let d = (*c).unwrap();
        // println!("{:?}", a);
        println!("{:?}", b);
        println!("{:p}", c);
        // is_copy(b);
        println!("{:p}", c1);
        println!("{:?}", d);

        let point = Point { x: 1, y: 2 };
        is_copy(point);
    }

    fn is_copy<T: Copy>(_: T) {}

    #[derive(Clone, Copy)]
    struct Point {
        x: i32,
        y: i32,
    }
}
