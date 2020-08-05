use crate::ast::*;
// use nom::error::VerboseError;
// use quickcheck::quickcheck;
use quickcheck::{Arbitrary, Gen};
use rand::Rng;

impl Arbitrary for FragmentAST {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        arbitrary_module(g, 0)
    }
}

fn arbitrary_module<G: Gen>(g: &mut G, depth: usize) -> FragmentAST {
    FragmentAST {
        name: Arbitrary::arbitrary(g),
        inputs: Arbitrary::arbitrary(g),
        assignments: Arbitrary::arbitrary(g),
        submodules: if g.gen() && depth < 3 {
            (0..g.gen_range(0, 5u32))
                .map(|_| arbitrary_module(g, depth + 1))
                .collect()
        } else {
            vec![]
        },
        output: arbitrary_expression(g, 0),
    }
}

impl quickcheck::Arbitrary for ModuleInput {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        ModuleInput {
            name: Arbitrary::arbitrary(g),
            input_type: Arbitrary::arbitrary(g),
        }
    }
}

impl quickcheck::Arbitrary for AssignmentAST {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        AssignmentAST {
            name: Arbitrary::arbitrary(g),
            valtype: Arbitrary::arbitrary(g),
            expr: arbitrary_expression(g, 0),
        }
    }
}

impl quickcheck::Arbitrary for Type {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        use Type::*;
        if Arbitrary::arbitrary(g) {
            PrimInt
        } else {
            PrimString
        }
    }
}

impl Arbitrary for Name {
    fn arbitrary<G: Gen>(g: &mut G) -> Name {
        let az = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

        let s: String = (1u32..g.gen_range(1, 30))
            .map(|_| az[g.gen_range(0, az.len())] as char)
            .collect();
        Name(s)
    }
}

fn arbitrary_expression<G: Gen>(g: &mut G, depth: usize) -> Expression {
    if g.gen() && depth < 2 {
        // Can only generate complex structures if less than 3 deep.
        match g.gen_range(0, 4) {
            0 => Expression::IfElse {
                guard: Box::new(arbitrary_expression(g, depth + 1)),
                body: Box::new(arbitrary_expression(g, depth + 1)),
                else_body: Box::new(arbitrary_expression(g, depth + 1)),
            },
            1 => Expression::ModuleApplication {
                mod_name: Arbitrary::arbitrary(g),
                arguments: (0..g.gen_range(0, 3))
                    .map(|_| arbitrary_expression(g, depth + 1))
                    .collect(),
            },
            2 => Expression::Range {
                from: Box::new(arbitrary_expression(g, depth + 1)),
                to: Box::new(arbitrary_expression(g, depth + 1)),
            },
            3 => Expression::Sum {
                a: Box::new(arbitrary_expression(g, depth + 1)),
                b: Box::new(arbitrary_expression(g, depth + 1)),
            },
            _ => panic!("option should never be generated"),
        }
    } else {
        match g.gen_range(0, 4) {
            0 => Expression::ConstBoolean(g.gen()),
            1 => Expression::ConstInteger(g.gen()),
            2 => Expression::ConstString(Arbitrary::arbitrary(g)),
            3 => Expression::LacunaryRef(Arbitrary::arbitrary(g)),
            _ => panic!("option should never be generated"),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[quickcheck]
//     fn generator_parser_reversal(xs: Module) -> bool {
//         let code = xs.gen_code();

//         // println!("{}", code);

//         match module::<VerboseError<&str>>(&code) {
//             Ok((_, r)) => xs == r,
//             _ => false,
//         }
//     }
// }
