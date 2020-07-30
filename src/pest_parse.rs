
// use pest::Parser;
// use crate::ast::*;
// use pest::iterators::Pair;

// #[derive(Parser)]
// #[grammar = "tempura.pest"]
// pub struct TempuraParser;

// pub fn parse_tempura(src: &str) -> Result<TempuraAST, &str> {

//     let parsed = TempuraParser::parse(Rule::program, src);

//     match parsed {
//         Ok(mut res) => makeAST(res.next().expect("Should only be 1 program.")),
//         Err(err) => {
//             println!("Parse failure: {}", err);
//             Err("Parse failure.")
//         }
//     }
// }

// fn makeAST(pair: Pair<Rule>) -> Result<TempuraAST, &str> {

//     assert_eq!(pair.as_rule(), Rule::program);
    
//     Ok(TempuraAST { assignments : pair.into_inner().map( assignmentAST ).collect() })
// }

// // fn inner_rules<N>(pairs: Pairs<Rule>, rules:[Rule; N]) -> [Rule; N] {
// //     let mut scanner = 0;
// //     let mut found = [None; N];

// //     for p in pairs {
// //         if (p.as_rule() == rules[scanner]) {
// //             found[scanner] = Some(p);
// //         }
// //         scanner += 1;
// //     }
// // }

// fn nameAST(pair: Pair<Rule>) -> Name {

//     assert_eq!(pair.as_rule(), Rule::ident);
    
//     Name(pair.as_span().as_str().to_string())
// }

// fn typeAST(pair: Pair<Rule>) -> Type {

//     assert_eq!(pair.as_rule(), Rule::ttype);

//     typeAST_inner(pair.into_inner().next().unwrap())
// }
    
// fn typeAST_inner(pair: Pair<Rule>) -> Type {
//     match pair.as_rule() {
//         Rule::fntype => {
//             let mut inner = pair.into_inner();
//             let result = typeAST_inner(inner.next().unwrap());

//             let mut args = inner.map(typeAST_inner).collect();
            
//             Type::Function(args, Box::new(result))
//         },
//         Rule::simpletype => {
//             match pair.as_span().as_str() {
//                 "int" => Type::PrimInt,
//                 "string" => Type::PrimString,
//                 _ => panic!("Unknown type!")
//             }
//         },
//         _ => panic!("Illegal rule.")
//     }
// }

// fn exprAST(pair: Pair<Rule>) -> Expression {
//     assert_eq!(pair.as_rule(), Rule::expr);
    
//     Expression::ConstString("TODO".to_string())

//     // match pair.as_rule() {
//     //     Rule::fntype => {
//     //         let inner = pair.into_inner();
//     //         let mut args = Vec::new();
//     //         while (inner.len() >= 2) {
//     //             args.push(typeAST(inner.next().unwrap()));
//     //         }
//     //         let result = typeAST(inner.next().unwrap());
//     //         Type::Function(args, result)
//     //     },
//     //     Rule::simpletype => {
//     //         match pair.as_span().as_str() {
//     //             "int" => Type::SimpleInt,
//     //             "string" => Type::SimpleString,
//     //             _ => panic!("Unknown type!")
//     //         }
//     //     }
//     // }
// }

// fn assignmentAST(pair: Pair<Rule>) -> Assignment {

//     assert_eq!(pair.as_rule(), Rule::valuedec);
    
//     let mut inner = pair.into_inner();

//     println!("!!! {:?}", inner);

//     let typedec = if inner.peek().unwrap().as_rule() == Rule::ttype {
//         Some(typeAST(inner.next().unwrap()))
//     } else {
//         None
//     };

//     let mut untyped_valuedec = inner.next().unwrap().into_inner();
//     let mut arg_idents = Vec::new();
//     while untyped_valuedec.peek().unwrap().as_rule() == Rule::ident {
//         nameAST(untyped_valuedec.next().unwrap()); 
//     }
//     let value_expr = arg_idents.pop();

//     Assignment(value_ident, arg_idents, typedec, value_expr)
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_parse_typedec() {
//         let successful_parse = TempuraParser::parse(Rule::typedec, "fb : int -> string");
//         // println!("{:?}", successful_parse);
//         successful_parse.unwrap();
//     }

//     #[test]
//     fn test_parse_untyped_valuedec() {
//         let successful_parse = TempuraParser::parse(Rule::untyped_valuedec, "fb i = \"hello\"");
//         // println!("{:?}", successful_parse);
//         successful_parse.unwrap();
//     }

//     #[test]
//     fn test_parse_typed_valuedec() {
//         let successful_parse = TempuraParser::parse(Rule::valuedec, "fb : int -> string\nfb i = \"hello\"");
//         println!("{:?}", successful_parse);
//         match successful_parse {
//             Ok(res) => println!("{}", res),
//             Err(res) => {
//                 println!("{}", res);
//                 assert!(false);
//             }
//         }
        
//     }
// }
