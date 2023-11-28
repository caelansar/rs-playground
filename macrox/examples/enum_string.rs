use macrox::EnumString;
use std::str::FromStr;

#[derive(EnumString, Eq, PartialEq, Debug)]
enum MyEnum {
    One,
    Two,
    Three,
}

fn main() {
    assert_eq!("One", MyEnum::One.to_string());
    assert_eq!(MyEnum::One, MyEnum::from_str("One").unwrap());
}
