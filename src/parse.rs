use nom::{
    // branch::alt,
    bytes::complete::take_while,
    character::complete::char,
    combinator::map,
    sequence::{preceded, separated_pair},
    IResult,
};
use crate::quoted_string;

// Whitespace

fn whitespace(src: &str) -> nom::IResult<&str, &str> {
    let chars = " \t\r\n";
    take_while(move |c| chars.contains(c))(src)
}

//region Name
#[derive(Debug)]
pub struct Name<'a>(&'a str);

fn name(src: &str) -> nom::IResult<&str, Name> {
    map(nom::character::complete::alphanumeric1, Name)(src)
}
//endregion

//region Expression
#[derive(Debug)]
pub enum Expression {
    ConstString(String),
}

fn expression(src: &str) -> nom::IResult<&str, Expression> {
    string(src)
}

// // String
// fn parse_str(src: &str) -> IResult<&str, &str> {
//     escaped(alphanumeric, '\\', one_of("\"n\\"))(src)
// }

fn string(src: &str) -> IResult<&str, Expression> {
    map(quoted_string::parse_string, Expression::ConstString)(src)
}


//endregion

#[derive(Debug)]
pub struct Assignment<'a>(Name<'a>, Expression);

pub fn tempura(src: &str) -> nom::IResult<&str, Assignment> {
    map(
        separated_pair(
            preceded(whitespace, name),
            preceded(whitespace, char('=')),
            preceded(whitespace, expression),
        ),
        |(n, e)| Assignment(n, e),
    )(src)
}