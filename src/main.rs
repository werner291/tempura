extern crate nom;

mod quoted_string;
mod parse;

use std::env;
use std::fs;


fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let filename = &args[1];

    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    let result = parse::tempura(&contents);

    println!("{:?}", result);
}
