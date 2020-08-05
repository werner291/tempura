extern crate nom;

// #[cfg(test)]
// extern crate quickcheck;
// #[cfg(test)]
// #[macro_use(quickcheck)]
// extern crate quickcheck_macros;

mod ast;
mod build;
// mod gen_ast;
mod nom_parse;
mod program;
mod quoted_string;
mod run;
mod verifier;

extern crate term_size;
// extern crate pest;
// #[macro_use]
// extern crate pest_derive;

use nom::error::VerboseError;
use program::VarType;
use std::io::{self, Write};
use std::process::exit;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use std::{env, fs};


fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let filename = &args[1];

    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    let result = match nom_parse::parse_tempura::<VerboseError<&str>>(&contents) {
        Ok((_, result)) => result,
        Err(e) => {
            println!("{}", e);
            exit(1);
        }
    };

    println!("Building...");

    let mut rte = build::build_runtime(result).expect("Build failed.");

    // let stdout = main_module.0(vec![stdin], &mut rte);

    println!("\u{001B}[32mBuild successful...");

    rte.listen(
        rte.stdout.unwrap(),
        false,
        Box::new(|_t, c| match c {
            VarType::Char(c) => {
                print!("{}", c);
                io::stdout().flush().unwrap();
            }
            VarType::Null => (),
            _ => panic!("stdout should be char stream"),
        }),
    );

    enum Event {
        Stdin(char),
        ClockTick(u64),
    }

    let (tx, rx): (Sender<Event>, Receiver<Event>) = mpsc::channel();
    let tx2 = tx.clone();

    // Spawn one second timer
    thread::spawn(move || {
        let mut t = 0;
        loop {
            thread::sleep(Duration::from_millis(100));
            tx.send(Event::ClockTick(t)).unwrap();
            t += 1;
        }
    });

    // Spawn one second timer
    thread::spawn(move || loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_n) => {
                for c in input.chars() {
                    tx2.send(Event::Stdin(c)).unwrap()
                }
            }
            Err(error) => println!("error: {}", error),
        }
    });

    loop {
        match rx.recv().unwrap() {
            Event::Stdin(c) => {
                rte.put_current(rte.stdin.unwrap(), VarType::Char(c));
            }
            Event::ClockTick(t) => {
                rte.put_current(rte.clock.unwrap(), VarType::Int(t as i64));
            }
        }
    }
}
