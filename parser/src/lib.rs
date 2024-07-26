mod winnow;

use nom::bytes::complete::{tag, take_while_m_n};
use nom::character::complete::i32;
use nom::combinator::map_res;
use nom::sequence::{delimited, separated_pair, tuple};
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

fn parse_integer_pair(input: &str) -> IResult<&str, (i32, i32)> {
    separated_pair(i32, tag(", "), i32)(input)
}

fn parse_coordinate(input: &str) -> IResult<&str, Coordinate> {
    let (remaining, (x, y)) = delimited(tag("("), parse_integer_pair, tag(")"))(input)?;

    Ok((remaining, Coordinate { x, y }))
}

#[derive(Debug, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

fn hex_color(input: &str) -> IResult<&str, Color> {
    let (input, _) = tag("#")(input)?;
    println!("input: {}", input);
    let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

    Ok((input, Color { red, green, blue }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_coordinate_works() {
        let (_, parsed) = parse_coordinate("(3, 5)").unwrap();
        assert_eq!(parsed, Coordinate { x: 3, y: 5 });

        let (_, parsed) = parse_coordinate("(2, -4)").unwrap();
        assert_eq!(parsed, Coordinate { x: 2, y: -4 });

        let err = parse_coordinate("(3,)");
        assert!(err.is_err());

        let err = parse_coordinate("(,3)");
        assert!(err.is_err());

        let err = parse_coordinate("Ferris");
        assert!(err.is_err());
    }

    #[test]
    fn parse_color_works() {
        assert_eq!(
            hex_color("#355770"),
            Ok((
                "",
                Color {
                    red: 53,
                    green: 87,
                    blue: 112,
                }
            ))
        );
    }
}
