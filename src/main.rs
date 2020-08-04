extern crate nom;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod ast;
mod build;
mod gen_ast;
mod nom_parse;
mod program;
mod quoted_string;
mod run;

extern crate term_size;
// extern crate pest;
// #[macro_use]
// extern crate pest_derive;

use nom::error::VerboseError;
use run::*;
use std::process::exit;
use std::{env, fs};
use std::io;
use program::VarType;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let filename = &args[1];

    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    let result = match nom_parse::parse_tempura::<VerboseError<&str>>(&contents) {
        Ok((_, result)) => {
            result
        }
        Err(e) => {
            println!("{}", e);
            exit(1);
        }
    };

    println!("Building...");

    let mut rte = build::build_runtime(result).expect("Build failed.");

    // let stdout = main_module.0(vec![stdin], &mut rte);

    println!("\u{001B}[32mBuild successful...");

    rte.listen(rte.stdout.unwrap(), true, Box::new(|_t,c| {
        match c {
            VarType::Char(c) => println!("{}",c),
            VarType::Null => (),
            _ => panic!("stdout should be char stream")
        }
        
    }));

    let mut input = String::new();
    loop{
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                for c in input.chars() {
                    rte.put_current(rte.stdin.unwrap(), VarType::Char(c));
                }
            }
            Err(error) => println!("error: {}", error),
        }
    }
    // println!("{}", .stringify().unwrap());
}
