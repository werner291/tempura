extern crate nom;

mod ast;
mod build;
mod compute;
mod nom_parse;
mod quoted_string;

extern crate term_size;
// extern crate pest;
// #[macro_use]
// extern crate pest_derive;

use std::{env, fs};
use std::process::exit;

// enum RuntimeRef {
//     StringRef(Index),
//     BooleanRef(Index),
//     IntegerRef(Index),
// }

// impl RuntimeRef {

// fn is_string(&self) -> bool {
//     if let RuntimeRef::StringRef(_) = self { true } else { false }
// }

// fn is_boolean(&self) -> bool {
//     if let RuntimeRef::BooleanRef(_) = self { true } else { false }
// }

// fn is_integer(&self) -> bool {
//     if let RuntimeRef::IntegerRef(_) = self { true } else { false }
// }
// }

// struct RuntimeVar {
//     cache: Option<Rc<dyn Any>>,
//     compute: Box<dyn Fn(&mut Runtime) -> Rc<dyn Any>>,
//     dependents: Vec<Index>,
// }

// impl RuntimeVar {
//     fn from_compute(compute: Box<dyn Fn(&mut Runtime) -> Rc<dyn Any>>) -> RuntimeVar {
//         RuntimeVar {
//             cache: None,
//             compute,
//             dependents: Vec::new(),
//         }
//     }
// }

// struct Runtime {
//     variables: Arena<RuntimeVar>, // integers: Arena<RuntimeVar<i64>>,
//                                   // booleans: Arena<RuntimeVar<bool>>
// }

// impl Runtime {
//     fn new() -> Runtime {
//         Runtime {
//             variables: Arena::new(),
//         }
//     }

//     fn update(&mut self, idx: Index) -> Rc<dyn Any> {
//         let computed = (self.variables[idx].compute)(rt);
//         self.cache = Some(computed);
//         self.cache.as_ref().unwrap().clone()
//     }

//     fn pull(&mut self, idx: Index) -> Rc<dyn Any> {
//         match self.cache.as_ref() {
//             Some(x) => x.clone(),
//             None => self.update(rt),
//         }
//     }

//     fn pull_var(&mut self, idx: Index) -> Rc<dyn Any> {
//         self.variables[idx].pull(self)
//     }

//     // fn alloc_string(&mut self, var: RuntimeVar<String>) -> RuntimeRef {
//     //     RuntimeRef::StringRef(self.strings.insert(var))
//     // }

//     // fn get_boolean(&mut self, rref: RuntimeRef) -> Result<&mut RuntimeVar<bool>, &str> {
//     //     if let RuntimeRef::BooleanRef(idx) = rref {
//     //         Ok(&mut self.booleans[idx])
//     //     } else {
//     //         Err("Not a boolean reference.")
//     //     }
//     // }

//     // fn get_string(&mut self, rref: RuntimeRef) -> Result<&mut RuntimeVar<String>, &str> {
//     //     if let RuntimeRef::BooleanRef(idx) = rref {
//     //         Ok(&mut self.strings[idx])
//     //     } else {
//     //         Err("Not a string reference.")
//     //     }
//     // }

//     // fn get_integer(&mut self, rref: RuntimeRef) -> Result<&mut RuntimeVar<i64>, &str> {
//     //     if let RuntimeRef::BooleanRef(idx) = rref {
//     //         Ok(&mut self.integers[idx])
//     //     } else {
//     //         Err("Not a integer reference.")
//     //     }
//     // }
// }

// fn build_value<'a>(
//     expr: Expression<'a>,
//     name_index: &HashMap<&str, Index>,
//     vars: &mut Runtime,
// ) -> Result<Index, &'static str> {
//     match expr {
//         Expression::ConstString(val) => Ok(vars
//             .variables
//             .insert(RuntimeVar::from_compute(Box::new(move |_| Rc::new(val))))),
//         Expression::ConstInteger(i) => Ok(vars
//             .variables
//             .insert(RuntimeVar::from_compute(Box::new(move |_| Rc::new(i))))),
//         Expression::ValueRef(Name(val)) => Ok(name_index[val]),
//         Expression::IfElse {
//             guard,
//             body,
//             else_body,
//         } => {
//             // Build the guard expression into a value and obtain a reference.
//             let guard_idx = build_value(*guard, name_index, vars)?;

//             // if !(vars.variables[guard_idx].vartype == TypeId::of::<bool>()) {
//             //     return Err("Guard of if-expression must be boolean.");
//             // }

//             let body_idx = build_value(*body, name_index, vars)?;
//             let else_body_idx = build_value(*else_body, name_index, vars)?;

//             // if !(vars.variables[body_idx].vartype == vars.variables[else_body_idx].vartype) {
//             //     return Err("Both branches of the if-statement must be of the same type.");
//             // }

//             Ok(vars
//                 .variables
//                 .insert(RuntimeVar::from_compute(Box::new(move |rt| {
//                     if *rt.pull_var(guard_idx).downcast_ref::<bool>().unwrap() {
//                         rt.pull_var(body_idx)
//                     } else {
//                         rt.pull_var(else_body_idx)
//                     }
//                 }))))

//             // assignments.insert(name, referred_node);
//         }

//         Expression::Range { from, to } => {
//             panic!("Not done!");
//         }
//     }
// }

// fn build_runtime<'a>(ast: TempuraAST<'a>) -> Result<Runtime, String> {
//     let mut assignments = HashMap::new();
//     let mut nodes: Arena<String> = Arena::new();

//     let stdin = nodes.insert("The quick brown fox jumped over the lazy dog.".to_string());
//     assignments.insert("stdin", stdin);

//     // Map of assignments, from name to the AST entry.
//     // This is basically ast.assignments but in map form without duplicates.
//     let mut assignments_astnodes: HashMap<&'a str, Expression<'a>> = HashMap::new();

//     for Assignment {
//         name: Name(name),
//         expr,
//         args: _,
//         valtype: _,
//     } in ast.assignments.into_iter()
//     {
//         // Insert assignment into map for fast lookup.
//         if let Some(_old) = assignments_astnodes.insert(name, expr) {
//             return Err(format!("Duplicate assignment to name: {}.", name));
//         }
//     }

//     let mut ts = TopologicalSort::<&'a str>::new();

//     for (name, expr) in assignments_astnodes.iter() {
//         for ref_to in expr.collect_dependencies() {
//             ts.add_dependency(ref_to, *name);
//         }
//     }

//     let mut rt = Runtime::new();

//     while let Some(name) = ts.pop() {
//         if name != "stdin" {
//             let expr = assignments_astnodes
//                 .remove(name)
//                 .expect("astnodes should have a key for all values");

//             let value_idx = build_value(expr, &assignments, &mut rt)?;
//             assignments.insert(name, value_idx);

//             // {
//             //     Expression::ConstString(val) => {
//             //         assignments.insert(name, nodes.insert(val));
//             //     }
//             //     Expression::ValueRef(Name(val)) => {
//             //         let referred_node = *assignments
//             //             .get(&*val)
//             //             .expect("Topological sort should have satisfied all dependencies.");
//             //         assignments.insert(name, referred_node);
//             //     }
//             //     Expression::IfElse {
//             //         guard,
//             //         body,
//             //         else_body,
//             //     } => {
//             //         // assignments.insert(name, referred_node);
//             //     }
//             //     Expression::ConstInteger(i) => {
//             //         panic!("Not done!");
//             //     }
//             //     Expression::Range { from, to } => {
//             //         panic!("Not done!");
//             //     }
//             // }
//         }
//     }

//     if !ts.is_empty() {
//         return Err("A circular dependency exists!".to_string());
//     }

//     Ok(rt)
// }

// fn run(runtime: Runtime) {
//     // if let Some(node_id) = runtime.stdout {
//     //     println!("{}", runtime.nodes[node_id]);
//     // }
// }

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let filename = &args[1];

    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    let result = match nom_parse::parse_tempura(&contents) {
        Ok((_, result)) => {
            println!("Parse successful: {:?}", result);
            result
        },
        Err(e) => {
            println!("{}", e);
            exit(1);
        }
    };

    println!("Building...");

    let mut runtime = build::build(result).expect("Build failed.");

    // let runtime = build_runtime(result).expect("Build failed.");

    println!("\u{001B}[32mBuild successful...");

    println!(
        "{:?}",
        runtime
            .pull(runtime.by_name["stdout"])
            .downcast_ref::<String>()
            .unwrap()
    );

    // run(runtime);

    // let (w, h) = term_size::dimensions().expect("Cannot get terminal size!");

    // let ten_millis = time::Duration::from_millis(1000);

    // let phrases = ["Hello there!", "How are you doing?", "Feeling loopy?"];

    // print!("\x1B[2J");

    // let mut i = 0;
    // loop {
    //     thread::sleep(ten_millis);

    //     print!("i = {}", phrases[i % phrases.len()]);

    //     for j in phrases[i % phrases.len()].len() - 1..w - 1 {
    //         print!(" ");
    //     }

    //     print!("\r");

    //     // for y in 0..h {
    //     //     for x in 0..w {
    //     //     }
    //     //     println!("i = {}\r", i);
    //     // }
    //     i += 1;
    //     io::stdout().flush().unwrap();
    // }
}
