use nom::bytes::complete::tag;
use nom::character::complete::i32;
use nom::sequence::{delimited, separated_pair};
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
}
