use crate::Color;
use winnow::combinator::{separated, trace};
use winnow::stream::AsChar;
use winnow::token::take_while;
use winnow::{seq, PResult, Parser};

fn parser_num(input: &mut &str) -> PResult<u8> {
    take_while(2, AsChar::is_hex_digit)
        .map(|x| u8::from_str_radix(x, 16).unwrap())
        .parse_next(input)
}

fn parse_hex(data: &mut &str) -> PResult<Color> {
    "#".parse_next(data)?;

    let (red, green, blue) = (parser_num, parser_num, parser_num).parse_next(data)?;

    Ok(Color { red, green, blue })
}

fn parse_list(input: &mut &str) -> PResult<Vec<Color>> {
    separated(0.., parse_hex, ", ").parse_next(input)
}

#[test]
fn test_parse_list() {
    let mut input = "(#355770, #355770, #355770)";

    // --features winnow/debug
    let data = trace("parse_hex_list", seq!(_: '(', parse_list, _: ')'))
        .parse(&mut input)
        .unwrap()
        .0;

    assert_eq!(data.len(), 3);
    assert_eq!(
        data[0],
        Color {
            red: 53,
            green: 87,
            blue: 112,
        }
    );
    assert_eq!(
        data[1],
        Color {
            red: 53,
            green: 87,
            blue: 112,
        }
    );
    assert_eq!(
        data[2],
        Color {
            red: 53,
            green: 87,
            blue: 112,
        }
    );
}
