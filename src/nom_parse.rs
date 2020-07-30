use crate::quoted_string;
use nom::{
    branch::alt,
    // branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1},
    combinator::{cut, map, opt},
    error::context,
    multi::{many1, separated_list},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
    AsChar,
    IResult,
    sep
};

use crate::ast::*;

// Whitespace

fn whitespace(src: &str) -> nom::IResult<&str, &str> {
    let chars = " \t\r\n";
    take_while(move |c| chars.contains(c))(src)
}

//region Name

fn name(src: &str) -> nom::IResult<&str, Name> {
    map(
        take_while1(|item: char| item.is_alphanum() || item == '_'),
        Name,
    )(src)
}
//endregion

//region Expression

fn ifelse<'a>(src: &'a str) -> nom::IResult<&str, Expression<'a>> {
    context(
        "if-then-else",
        map(
            tuple((
                preceded(
                    tuple((whitespace, tag("if"))),
                    context("if guard", preceded(whitespace, expression)),
                ),
                preceded(
                    tuple((whitespace, tag("then"))),
                    context("if body", preceded(whitespace, expression)),
                ),
                preceded(
                    tuple((whitespace, tag("else"))),
                    context("else body", preceded(whitespace, expression)),
                ),
            )),
            |(guard, body, elsebody)| Expression::IfElse {
                guard: Box::new(guard),
                body: Box::new(body),
                else_body: Box::new(elsebody),
            },
        ),
    )(src)
}

fn functiontype<'a>(src: &'a str) -> nom::IResult<&str, Type> {
    context(
        "Function type.",
        map(
            separated_pair(
                preceded(whitespace, primtype),
                preceded(whitespace, tag("->")),
                preceded(whitespace, primtype),
            ),
            |(from, to)| Type::Function {
                arg_types: vec![from],
                result_type: Box::new(to),
            },
        ),
    )(src)
}

fn ttype<'a>(src: &'a str) -> nom::IResult<&str, Type> {
    alt((functiontype, primtype))(src)
}

fn primtype<'a>(src: &'a str) -> nom::IResult<&str, Type> {
    alt((
        map(preceded(whitespace, tag("int")), |_| Type::PrimInt),
        map(preceded(whitespace, tag("str")), |_| Type::PrimString),
    ))(src)
}

fn function_application<'a>(src: &'a str) -> nom::IResult<&str, Expression<'a>> {
    // Adjust to make sure priority works as expected.

    let arglist = delimited(char('('), separated_list(char(','), expression), preceded(whitespace, char(')')));

    map(pair(single_expression, arglist), 
        |(function,arguments)| Expression::FunctionApplication {function:Box::new(function), arguments})(src)
}

fn expression<'a>(src: &'a str) -> nom::IResult<&str, Expression<'a>> {
    alt((function_application, range, ifelse, single_expression))(src)
}



fn single_expression<'a>(src: &'a str) -> nom::IResult<&str, Expression<'a>> {
    alt((
        string,
        integer,
        valueref,
        delimited(char('('), expression, char(')')),
    ))(src)
}

fn valueref(src: &str) -> nom::IResult<&str, Expression> {
    map(name, Expression::ValueRef)(src)
}

fn parse_int(src: &str) -> nom::IResult<&str, i64> {
    map(
        pair(opt(char('-')), digit1),
        |(sgn, digits): (Option<char>, &str)| {
            if sgn.is_some() {
                -digits.parse::<i64>().unwrap()
            } else {
                digits.parse::<i64>().unwrap()
            }
        },
    )(src)
}

fn range(src: &str) -> nom::IResult<&str, Expression> {
    map(
        separated_pair(single_expression, tag(".."), single_expression),
        |(from, until)| Expression::Range {
            from: Box::new(from),
            to: Box::new(until),
        },
    )(src)
}

// // String
// fn parse_str(src: &str) -> IResult<&str, &str> {
//     escaped(alphanumeric, '\\', one_of("\"n\\"))(src)
// }

fn string(src: &str) -> IResult<&str, Expression> {
    map(quoted_string::parse_string, Expression::ConstString)(src)
}

fn integer(src: &str) -> IResult<&str, Expression> {
    map(parse_int, Expression::ConstInteger)(src)
}

//endregion

pub fn assignment_untyped<'a>(src: &'a str) -> nom::IResult<&'a str, Assignment<'a>> {
    context(
        "Value Assignment",
        map(
            tuple((
                preceded(whitespace, name),
                separated_list(whitespace, preceded(whitespace, name)),
                preceded(whitespace, char('=')),
                preceded(whitespace, expression),
            )),
            |(name, args, _, expr)| Assignment {
                name,
                args,
                valtype: None,
                expr,
            },
        ),
    )(src)
}

pub fn assignment<'a>(src: &'a str) -> nom::IResult<&'a str, Assignment<'a>> {
    let typedecl = context(
        "Type declaration.",
        opt(preceded(tuple((whitespace, tag("::"))), cut(ttype))),
    );
    map(tuple((typedecl, assignment_untyped)), |(t, a)| Assignment {
        valtype: t,
        ..a
    })(src)
}

pub fn parse_tempura<'a>(src: &'a str) -> nom::IResult<&'a str, TempuraAST<'a>> {
    context(
        "Top-level",
        map(separated_list(char('\n'), assignment), |ass| TempuraAST { assignments: ass }),
    )(src)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ifelse() {
        assert_eq!(
            ifelse("if foo then bar else baz"),
            Ok((
                "",
                Expression::IfElse {
                    guard: Box::new(Expression::ValueRef(Name("foo"))),
                    body: Box::new(Expression::ValueRef(Name("bar"))),
                    else_body: Box::new(Expression::ValueRef(Name("baz")))
                }
            ))
        );
    }

    #[test]
    fn test_range() {
        assert_eq!(
            range("5..9"),
            Ok((
                "",
                Expression::Range {
                    from: Box::new(Expression::ConstInteger(5)),
                    to: Box::new(Expression::ConstInteger(9))
                }
            ))
        );
    }

    #[test]
    fn test_name() {
        assert_eq!(name("hello"), Ok(("", Name("hello"))));
        assert_eq!(name("hello world"), Ok((" world", Name("hello"))));
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(
            whitespace("\t\r\n   hello\t     worldhello"),
            Ok(("hello\t     worldhello", "\t\r\n   "))
        );
    }

    #[test]
    fn test_ttype() {
        assert_eq!(
            ttype("str \n the rest"),
            Ok((" \n the rest", Type::PrimString))
        );
        assert_eq!(
            ttype("str -> int\n the rest"),
            Ok((
                "\n the rest",
                Type::Function {
                    arg_types: vec![Type::PrimString],
                    result_type: Box::new(Type::PrimInt)
                }
            ))
        );
    }

    #[test]
    fn test_function_application() {
        assert_eq!(
            function_application("map(fb,0..99) "),
            Ok((" ", Expression::FunctionApplication {
                        function: Box::new(Expression::ValueRef(Name("map"))),
                        arguments: vec![
                            Expression::ValueRef(Name("fb")),
                            Expression::Range {
                                from: Box::new(Expression::ConstInteger(0)),
                                to: Box::new(Expression::ConstInteger(99))
                            }
                        ]
                    }
            )));

            assert_eq!(
                function_application("lines(map(fb,0..99)) "),
                Ok((" ", Expression::FunctionApplication {
                    function: Box::new(Expression::ValueRef(Name("lines"))),
                    arguments: vec![
                        Expression::FunctionApplication {
                            function: Box::new(Expression::ValueRef(Name("map"))),
                            arguments: vec![
                                Expression::ValueRef(Name("fb")),
                                Expression::Range {
                                    from: Box::new(Expression::ConstInteger(0)),
                                    to: Box::new(Expression::ConstInteger(99))
                                }
                            ]
                        }
                    ]
                })));
    }

    #[test]
    fn test_valuedec() {
        assert_eq!(
            assignment("hello_world n = \"Hello!\""),
            Ok((
                "",
                Assignment {
                    args: vec![Name("n")],
                    expr: Expression::ConstString("Hello!".to_string()),
                    name: Name("hello_world"),
                    valtype: None
                }
            ))
        );

        assert_eq!(
            assignment("stdout = lines(map(fb,5..9))"),
            Ok(("", Assignment {
                args: vec![],
                expr: expression("lines(map(fb,5..9))").unwrap().1,
                name: Name("stdout"),
                valtype: None
            }))
        )
    }

    #[test]
    fn test_integer() {
        assert_eq!(parse_int("1337"), Ok(("", 1337)));
        assert_eq!(parse_int("-1337"), Ok(("", -1337)));
        assert_eq!(parse_int("1337  "), Ok(("  ", 1337)));
        assert!(parse_int("  -13 37").is_err());
    }

    #[test]
    fn test_valuedec_typed() {
        assert_eq!(
            assignment(":: int -> str\nhello_world n = \"Hello!\""),
            Ok((
                "",
                Assignment {
                    args: vec![Name("n")],
                    expr: Expression::ConstString("Hello!".to_string()),
                    name: Name("hello_world"),
                    valtype: Some(Type::Function {
                        arg_types: vec![Type::PrimInt],
                        result_type: Box::new(Type::PrimString)
                    })
                }
            ))
        );
    }

    #[test]
    fn test_toplevel() {
        assert_eq!(
            parse_tempura(r##":: int -> str
                fb i = "hello"
                
                :: str
                stdout = lines(map(fb,5..9))"##),
            Ok(("", TempuraAST { assignments : vec![
                assignment(":: int -> str\nfb i = \"hello\"").unwrap().1,
                assignment(":: str\nstdout = lines(map(fb,5..9))").unwrap().1
            ] }))
        )
    }
}
