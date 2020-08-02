use crate::quoted_string;
use nom::{
    branch::alt,
    // branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1},
    combinator::{map, opt},
    error::{context, convert_error, ParseError, VerboseError},
    multi::separated_list,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    AsChar,
    IResult,
};

use crate::ast::*;

// Whitespace

fn whitespace<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";
    take_while(move |c| chars.contains(c))(src)
}

//region Name

fn name<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&str, Name, E> {
    map(
        take_while1(|item: char| item.is_alphanum() || item == '_'),
        Name,
    )(src)
}
//endregion

//region Expression

fn ifelse<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&'a str, Expression<'a>, E> {
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

enum AssigmentOrSubmodule<'a> {
    Assignment(Assignment<'a>),
    Submodule(Module<'a>),
}

fn module<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&'a str, Module<'a>, E> {
    use AssigmentOrSubmodule::*;

    let mod_input = map(
        separated_pair(
            preceded(whitespace, name),
            preceded(whitespace, char(':')),
            preceded(whitespace, ttype),
        ),
        |(name, input_type)| ModuleInput { name, input_type },
    );

    let parameter_list = context(
        "Parameter list",
        delimited(
            char('('),
            separated_list(char(','), mod_input),
            preceded(whitespace, char(')')),
        ),
    );

    let decls = context(
        "declarations",
        separated_list(
            char('\n'),
            preceded(
                whitespace,
                alt((map(assignment, Assignment), map(module, Submodule))),
            ),
        ),
    );

    context(
        "module",
        map(
            tuple((
                preceded(whitespace, tag("mod")),
                preceded(whitespace, name),
                parameter_list,
                preceded(whitespace, char('{')),
                opt(terminated(decls, char('\n'))),
                context(
                    "output expression",
                    preceded(whitespace, expression)
                ),
                preceded(whitespace, char('}')),
            )),
            |(_mod, name, inputs, _, decls, output, _)| {
                let mut assignments = Vec::new();
                let mut submodules = Vec::new();

                if let Some(d) = decls {
                    for dec in d {
                        match dec {
                            AssigmentOrSubmodule::Assignment(ass) => assignments.push(ass),
                            AssigmentOrSubmodule::Submodule(smod) => submodules.push(smod),
                        }
                    }
                }

                Module {
                    name,
                    inputs,
                    assignments,
                    submodules,
                    output,
                }
            },
        ),
    )(src)
}

fn ttype<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&'a str, Type, E> {
    context(
        "type",
        alt((
            map(preceded(whitespace, tag("int")), |_| Type::PrimInt),
            map(preceded(whitespace, tag("str")), |_| Type::PrimString),
        )),
    )(src)
}

fn function_application<'a, E: ParseError<&'a str>>(
    src: &'a str,
) -> nom::IResult<&str, Expression<'a>, E> {
    // Adjust to make sure priority works as expected.

    let arglist = context(
        "arglist",
        delimited(
            char('('),
            separated_list(char(','), preceded(whitespace,expression)),
            preceded(whitespace, char(')')),
        ),
    );

    context(
        "function_application",
        map(pair(name, arglist), |(mod_name, arguments)| {
            Expression::ModuleApplication {
                mod_name,
                arguments,
            }
        }),
    )(src)
}

fn expression<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&str, Expression<'a>, E> {
    alt((function_application, range, ifelse, single_expression))(src)
}

fn single_expression<'a, E: ParseError<&'a str>>(
    src: &'a str,
) -> nom::IResult<&str, Expression<'a>, E> {
    alt((
        string,
        integer,
        valueref,
        delimited(char('('), expression, char(')')),
    ))(src)
}

fn valueref<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&str, Expression, E> {
    map(name, Expression::ValueRef)(src)
}

fn parse_int<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&str, i64, E> {
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

fn range<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&str, Expression, E> {
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

fn string<'a, E: ParseError<&'a str>>(src: &'a str) -> IResult<&str, Expression, E> {
    map(quoted_string::parse_string, Expression::ConstString)(src)
}

fn integer<'a, E: ParseError<&'a str>>(src: &'a str) -> IResult<&str, Expression, E> {
    map(parse_int, Expression::ConstInteger)(src)
}

//endregion

pub fn assignment<'a, E: ParseError<&'a str>>(
    src: &'a str,
) -> nom::IResult<&'a str, Assignment<'a>, E> {
    context(
        "Value Assignment",
        map(
            tuple((
                preceded(whitespace, name),
                opt(preceded(tuple((whitespace, char(':'))), ttype)),
                preceded(whitespace, char('=')),
                preceded(whitespace, expression),
            )),
            |(name, typedecl, _, expr)| Assignment {
                name,
                valtype: typedecl,
                expr,
            },
        ),
    )(src)
}

// pub fn assignment<'a><'a, E: ParseError<&'a str>>(src: &'a  str) -> nom::IResult<&'a str, Assignment<'a, E>> {

//     let typedecl = context(
//         "Type declaration.",
//         opt(preceded(tuple((whitespace, char(':'))), ttype)),
//     );
//     map(tuple((typedecl, assignment_untyped)), |(t, a)| Assignment {
//         valtype: t,
//         ..a
//     })(src)
// }

pub fn parse_tempura<'a, E: ParseError<&'a str>>(
    src: &'a str,
) -> nom::IResult<&'a str, Module<'a>, E> {
    module(src)
    // context(
    //     "Top-level",
    //     map(separated_list(char('\n'), module), |modules| TempuraAST {
    //         modules
    //     }),
    // )(src)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ifelse() {
        assert_eq!(
            ifelse::<VerboseError<&str>>("if foo then bar else baz"),
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
            range::<VerboseError<&str>>("5..9"),
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
        assert_eq!(name::<VerboseError<&str>>("hello"), Ok(("", Name("hello"))));
        assert_eq!(
            name::<VerboseError<&str>>("hello world"),
            Ok((" world", Name("hello")))
        );
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(
            whitespace::<VerboseError<&str>>("\t\r\n   hello\t     worldhello"),
            Ok(("hello\t     worldhello", "\t\r\n   "))
        );
    }

    #[test]
    fn test_ttype() {
        assert_eq!(
            ttype::<VerboseError<&str>>("str \n the rest"),
            Ok((" \n the rest", Type::PrimString))
        );
    }

    #[test]
    fn text_expr1() {
        assert_eq!(
            expression::<VerboseError<&str>>("map(fb,0..99) "),
            Ok((
                " ",
                Expression::ModuleApplication {
                    mod_name: Name("map"),
                    arguments: vec![
                        Expression::ValueRef(Name("fb")),
                        Expression::Range {
                            from: Box::new(Expression::ConstInteger(0)),
                            to: Box::new(Expression::ConstInteger(99))
                        }
                    ]
                }
            ))
        );
    }

    #[test]
    fn text_expr2() {
        let src2 = r#"concat("Hello world: ", to_string(i))"#;
        let res2 = function_application::<VerboseError<&str>>(src2);

        check_result(
            src2,
            res2,
            Expression::ModuleApplication {
                mod_name: Name("concat"),
                arguments: vec![
                    Expression::ConstString("Hello world: ".to_string()),
                    expression::<VerboseError<&str>>("to_string(i)").unwrap().1,
                ],
            },
        );

        assert_eq!(
            function_application::<VerboseError<&str>>("lines(map(fb,0..99)) "),
            Ok((
                " ",
                Expression::ModuleApplication {
                    mod_name: Name("lines"),
                    arguments: vec![Expression::ModuleApplication {
                        mod_name: Name("map"),
                        arguments: vec![
                            Expression::ValueRef(Name("fb")),
                            Expression::Range {
                                from: Box::new(Expression::ConstInteger(0)),
                                to: Box::new(Expression::ConstInteger(99))
                            }
                        ]
                    }]
                }
            ))
        );
    }

    #[test]
    fn test_valuedec() {
        assert_eq!(
            assignment::<VerboseError<&str>>("hello_world : str= \"Hello!\""),
            Ok((
                "",
                Assignment {
                    expr: Expression::ConstString("Hello!".to_string()),
                    name: Name("hello_world"),
                    valtype: Some(Type::PrimString),
                }
            ))
        );

        assert_eq!(
            assignment::<VerboseError<&str>>("stdout = lines(map(fb,5..9))"),
            Ok((
                "",
                Assignment {
                    expr: expression::<VerboseError<&str>>("lines(map(fb,5..9))")
                        .unwrap()
                        .1,
                    name: Name("stdout"),
                    valtype: None,
                }
            ))
        )
    }

    #[test]
    fn test_integer() {
        assert_eq!(parse_int::<VerboseError<&str>>("1337"), Ok(("", 1337)));
        assert_eq!(parse_int::<VerboseError<&str>>("-1337"), Ok(("", -1337)));
        assert_eq!(parse_int::<VerboseError<&str>>("1337  "), Ok(("  ", 1337)));
        assert!(parse_int::<VerboseError<&str>>("  -13 37").is_err());
    }

    #[test]
    fn test_module() {
        let src = r#"mod fb(i : int) {
            concat("Hello world: ", to_string(i))
        }"#;

        check_result(
            src,
            module(src),
            Module {
                name: Name("fb"),
                inputs: vec![ModuleInput {
                    name: Name("i"),
                    input_type: Type::PrimInt,
                }],
                submodules: vec![],
                assignments: vec![],
                output: expression::<VerboseError<&str>>(
                    r#"concat("Hello world: ", to_string(i))"#,
                )
                .unwrap()
                .1,
            },
        );
    }

    fn check_result<T: Eq + std::fmt::Debug>(
        src: &str,
        res: nom::IResult<&str, T, VerboseError<&str>>,
        expected: T,
    ) {
        match res {
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("Parse error: {}", convert_error(src, e));
                assert!(false);
            }
            Ok((_, res)) => assert_eq!(res, expected),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_toplevel() {
        let src = r##"mod main(stdin : str) {

            mod fb(i : int) {
                concat("Hello world: ", to_string(i))
            }
        
            fb(500)
        }"##;

        check_result(
            src,
            parse_tempura(src),
            Module {
                name: Name("main"),
                inputs: vec![ModuleInput {
                    name: Name("stdin"),
                    input_type: Type::PrimString,
                }],
                submodules: vec![module::<VerboseError<&str>>(r#"mod fb(i : int) {
                    concat("Hello world: ", to_string(i))
                }"#).unwrap().1],
                assignments: vec![],
                output: expression::<VerboseError<&str>>("fb(500)").unwrap().1,
            },
        );
    }
}
