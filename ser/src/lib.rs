#![feature(test)]
#![allow(dead_code)]

extern crate test;

use std::{fmt::Debug, marker::PhantomData};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct ParseError;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Point {
    x: f64,
    y: f64,
}

trait ConfigParser<'a> {
    type Cfg: Deserialize<'a> + Debug;

    fn parse(&self, data: &'a str) -> Result<Self::Cfg, ParseError>;
}

struct JsonParser<T> {
    _dst: PhantomData<T>,
}

impl<T> JsonParser<T> {
    fn new() -> Self {
        Self { _dst: PhantomData }
    }
}

impl<'a, T: Deserialize<'a> + Debug> ConfigParser<'a> for JsonParser<T> {
    type Cfg = T;

    fn parse(&self, data: &'a str) -> Result<Self::Cfg, ParseError> {
        let data: T = serde_json::from_str(data).map_err(|err| {
            println!("err: {:?}", err);
            ParseError
        })?;
        Ok(data)
    }
}

struct YamlParser<T> {
    _dst: PhantomData<T>,
}

impl<T> YamlParser<T> {
    fn new() -> Self {
        Self { _dst: PhantomData }
    }
}

impl<'a, T: Deserialize<'a> + Debug> ConfigParser<'a> for YamlParser<T> {
    type Cfg = T;

    fn parse(&self, data: &'a str) -> Result<Self::Cfg, ParseError> {
        let data: T = serde_yaml::from_str(data).map_err(|err| {
            println!("err: {:?}", err);
            ParseError
        })?;
        Ok(data)
    }
}

fn parse<'a, P: ConfigParser<'a>>(data: &'a str, parser: P) -> Result<P::Cfg, ParseError> {
    parser.parse(data)
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, hint::black_box};

    use test::Bencher;

    use super::*;

    #[derive(Debug, Deserialize)]
    struct CowUser<'a> {
        #[serde(borrow)]
        name: Cow<'a, str>,
    }

    #[derive(Debug, Deserialize)]
    struct RefUser<'a> {
        name: &'a str,
    }

    #[derive(Debug, Deserialize)]
    struct OwnedUser {
        name: String,
    }

    #[test]
    fn parse_yaml_should_works() {
        let point = Point { x: 1.0, y: 2.0 };

        let yaml = "x: 1.0\ny: 2.0\n";
        let parser: YamlParser<Point> = YamlParser::new();
        let cfg = parse(yaml, parser).unwrap();
        assert_eq!(point, cfg);
    }

    #[test]
    fn parse_json_should_works() {
        let point = Point { x: 1.0, y: 2.0 };

        let json = r#"{"x": 1.0, "y": 2.0}"#;
        let parser: JsonParser<Point> = JsonParser::new();
        let cfg = parse(json, parser).unwrap();
        assert_eq!(point, cfg);
    }

    static JSON_USER: &str = r#"{"name": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "age": 20}"#;

    fn bench_user<'a, T: Deserialize<'a> + Debug>(b: &mut Bencher) {
        b.iter(|| {
            let parser: JsonParser<T> = JsonParser::new();
            let user = parse(&JSON_USER, parser).unwrap();
            black_box(user);
        })
    }

    #[bench]
    fn bench_owned_user(b: &mut Bencher) {
        bench_user::<OwnedUser>(b)
    }

    #[bench]
    fn bench_ref_user(b: &mut Bencher) {
        bench_user::<RefUser>(b)
    }

    #[bench]
    fn bench_cow_user(b: &mut Bencher) {
        bench_user::<CowUser>(b)
    }

    #[bench]
    fn bench_string_clone(b: &mut Bencher) {
        let data = [0u8; 102400];
        let s = String::from_utf8_lossy(&data).into_owned();
        b.iter(|| black_box(s.clone()))
    }

    #[bench]
    fn bench_mem_take(b: &mut Bencher) {
        let data = [0u8; 102400];
        let mut s = String::from_utf8_lossy(&data).into_owned();
        b.iter(|| black_box(std::mem::take(&mut s)))
    }
}
