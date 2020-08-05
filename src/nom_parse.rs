use crate::quoted_string;

// Compiler keeps waning about `convert_error` and `VerboseError` despite them being used.
#[allow(unused_imports)]
use nom::error::{context, convert_error, ParseError, VerboseError};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1},
    combinator::{map, opt},
    multi::separated_list,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    AsChar, IResult,
};

use crate::ast::*;

// Whitespace

pub fn whitespace<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";
    take_while(move |c| chars.contains(c))(src)
}

//region Name

pub fn name<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&str, Name, E> {
    map(
        take_while1(|item: char| item.is_alphanum() || item == '_'),
        |s: &str| Name(s.to_string()),
    )(src)
}
//endregion

//region Expression

pub fn ifelse<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&'a str, Expression, E> {
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

enum AssigmentOrSubmodule {
    Assignment(AssignmentAST),
    Submodule(FragmentAST),
}

pub fn module<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&'a str, FragmentAST, E> {
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
                context("output expression", preceded(whitespace, expression)),
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

                FragmentAST {
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

pub fn ttype<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&'a str, Type, E> {
    context(
        "type",
        alt((
            map(preceded(whitespace, tag("int")), |_| Type::PrimInt),
            map(preceded(whitespace, tag("str")), |_| Type::PrimString),
        )),
    )(src)
}

pub fn function_application<'a, E: ParseError<&'a str>>(
    src: &'a str,
) -> nom::IResult<&str, Expression, E> {
    // Adjust to make sure priority works as expected.

    let arglist = context(
        "arglist",
        delimited(
            char('('),
            separated_list(char(','), preceded(whitespace, expression)),
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

pub fn container_index<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&str, Expression, E> {
    context(
        "container index",
        map(
            pair(
                single_expression,
                delimited(char('['), expression, preceded(whitespace, char(']'))),
            ),
            |(cont, idx)| Expression::BinaryOp(Box::new(cont), Box::new(idx), BinaryOp::Index),
        ),
    )(src)
}

pub fn expression<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&str, Expression, E> {
    alt((
        function_application,
        container_index,
        infix_operation,
        ifelse,
        single_expression,
    ))(src)
}

pub fn single_expression<'a, E: ParseError<&'a str>>(
    src: &'a str,
) -> nom::IResult<&str, Expression, E> {
    alt((
        string,
        integer,
        boolean,
        valueref,
        delimited(char('('), expression, char(')')),
    ))(src)
}

pub fn valueref<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&str, Expression, E> {
    map(name, Expression::LacunaryRef)(src)
}

pub fn parse_int<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&str, i64, E> {
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

pub fn infix_operation<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&str, Expression, E> {
    
    let operator = alt((
        map(tag(".."), |_| BinaryOp::Range),
        map(tag("+"),  |_| BinaryOp::Sum),
        map(tag("<>"), |_| BinaryOp::Concat),
        map(tag(">="), |_| BinaryOp::Geq),
        map(tag("=="), |_| BinaryOp::Eq),
        map(tag("<="), |_| BinaryOp::Leq),
        map(tag(">"), |_| BinaryOp::Lt),
        map(tag("<"), |_| BinaryOp::Gt)
    ));

    map(
        tuple((preceded(whitespace, single_expression), preceded(whitespace, operator), preceded(whitespace, single_expression))),
        |(a, o, b)| Expression::BinaryOp(Box::new(a),Box::new(b),o)
    )(src)
}

pub fn string<'a, E: ParseError<&'a str>>(src: &'a str) -> IResult<&str, Expression, E> {
    map(quoted_string::parse_string, Expression::ConstString)(src)
}

pub fn integer<'a, E: ParseError<&'a str>>(src: &'a str) -> IResult<&str, Expression, E> {
    map(parse_int, Expression::ConstInteger)(src)
}

pub fn boolean<'a, E: ParseError<&'a str>>(src: &'a str) -> IResult<&str, Expression, E> {
    alt((
        map(tag("true"), |_| Expression::ConstBoolean(true)),
        map(tag("false"), |_| Expression::ConstBoolean(false)),
    ))(src)
}

//endregion

pub fn assignment<'a, E: ParseError<&'a str>>(
    src: &'a str,
) -> nom::IResult<&'a str, AssignmentAST, E> {
    context(
        "Value Assignment",
        map(
            tuple((
                preceded(whitespace, name),
                opt(preceded(tuple((whitespace, char(':'))), ttype)),
                preceded(whitespace, char('=')),
                preceded(whitespace, expression),
            )),
            |(name, typedecl, _, expr)| AssignmentAST {
                name,
                valtype: typedecl,
                expr,
            },
        ),
    )(src)
}

pub fn parse_tempura<'a, E: ParseError<&'a str>>(src: &'a str) -> nom::IResult<&'a str, FragmentAST, E> {
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
                    guard: Box::new(Expression::LacunaryRef(Name("foo".to_string()))),
                    body: Box::new(Expression::LacunaryRef(Name("bar".to_string()))),
                    else_body: Box::new(Expression::LacunaryRef(Name("baz".to_string())))
                }
            ))
        );
    }

    #[test]
    fn test_name() {
        assert_eq!(
            name::<VerboseError<&str>>("hello"),
            Ok(("", Name("hello".to_string())))
        );
        assert_eq!(
            name::<VerboseError<&str>>("hello world"),
            Ok((" world", Name("hello".to_string())))
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
                    mod_name: Name("map".to_string()),
                    arguments: vec![
                        Expression::LacunaryRef(Name("fb".to_string())),
                        Expression::BinaryOp(
                            Box::new(Expression::ConstInteger(0)),
                            Box::new(Expression::ConstInteger(99)),
                            BinaryOp::Range)
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
                mod_name: Name("concat".to_string()),
                arguments: vec![
                    Expression::ConstString("Hello world: ".to_string()),
                    expression::<VerboseError<&str>>("to_string(i)").unwrap().1,
                ],
            },
        );
    }

    #[test]
    fn test_valuedec() {
        assert_eq!(
            assignment::<VerboseError<&str>>("hello_world : str= \"Hello!\""),
            Ok((
                "",
                AssignmentAST {
                    expr: Expression::ConstString("Hello!".to_string()),
                    name: Name("hello_world".to_string()),
                    valtype: Some(Type::PrimString),
                }
            ))
        );

        assert_eq!(
            assignment::<VerboseError<&str>>("stdout = lines(map(fb,5..9))"),
            Ok((
                "",
                AssignmentAST {
                    expr: expression::<VerboseError<&str>>("lines(map(fb,5..9))")
                        .unwrap()
                        .1,
                    name: Name("stdout".to_string()),
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
            FragmentAST {
                name: Name("fb".to_string()),
                inputs: vec![ModuleInput {
                    name: Name("i".to_string()),
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
                panic!("Failure.")
            }
            Ok((_, res)) => assert_eq!(res, expected),
            _ => panic!("Failure."),
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
            FragmentAST {
                name: Name("main".to_string()),
                inputs: vec![ModuleInput {
                    name: Name("stdin".to_string()),
                    input_type: Type::PrimString,
                }],
                submodules: vec![
                    module::<VerboseError<&str>>(
                        r#"mod fb(i : int) {
                    concat("Hello world: ", to_string(i))
                }"#,
                    )
                    .unwrap()
                    .1,
                ],
                assignments: vec![],
                output: expression::<VerboseError<&str>>("fb(500)").unwrap().1,
            },
        );
    }
}
