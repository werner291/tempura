extern crate nom;

mod ast;
mod build;
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

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let filename = &args[1];

    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    let result = match nom_parse::parse_tempura::<VerboseError<&str>>(&contents) {
        Ok((_, result)) => {
            println!("Parse successful: {:?}", result);
            result
        }
        Err(e) => {
            println!("{}", e);
            exit(1);
        }
    };

    println!("Building...");

    let mut rte = RuntimeEnv::new();

    let main_module = build::build_toplevel_module(result).expect("Build failed.");

    let charvec = "This is some input"
        .chars()
        .map(|c| rte.node_from_operation(program::Operation::Const(program::VarType::Char(c))))
        .collect();

    let stdin = rte.node_from_operation(program::Operation::Vector(charvec));

    let stdout = rte.instantiate_fragment(&main_module, vec![stdin]);

    // let stdout = main_module.0(vec![stdin], &mut rte);

    println!("\u{001B}[32mBuild successful...");

    println!("{}", rte.pull(stdout).stringify().unwrap());
}
