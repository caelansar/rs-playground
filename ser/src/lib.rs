#![feature(test)]
#![allow(dead_code)]

mod byteorder;

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

    use serde_json::json;
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

    fn bench_user<'a: 'b, 'b, T: Deserialize<'b> + Debug>(b: &mut Bencher, s: &'a str) {
        b.iter(|| {
            let parser: JsonParser<T> = JsonParser::new();
            let user = parse(&s, parser).unwrap();
            black_box(user);
        })
    }

    fn gen_json_string() -> String {
        let data = [65u8; 1024];
        let s = String::from_utf8_lossy(&data).into_owned();
        let v = json!({"name": s, "age": 20});
        serde_json::to_string(&v).unwrap()
    }

    #[bench]
    fn bench_owned_user(b: &mut Bencher) {
        let s = gen_json_string();
        bench_user::<OwnedUser>(b, &s)
    }

    #[bench]
    fn bench_ref_user(b: &mut Bencher) {
        let s = gen_json_string();
        bench_user::<RefUser>(b, s.as_str())
    }

    #[bench]
    fn bench_cow_user(b: &mut Bencher) {
        let s = gen_json_string();
        bench_user::<CowUser>(b, &s)
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
