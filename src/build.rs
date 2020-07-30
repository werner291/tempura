use crate::ast::*;
use crate::compute::*;
use generational_arena::{Arena, Index};
use std::collections::HashMap;
use std::rc::Rc;
use topological_sort::TopologicalSort;

fn index_assignments<'a>(ast: TempuraAST<'a>) -> Result<HashMap<&'a str, Assignment<'a>>, &str> {
    // Map of assignments, from name to the AST entry.
    // This is basically ast.assignments but in map form without duplicates.
    let mut assignments_astnodes: HashMap<&'a str, Assignment<'a>> = HashMap::new();

    for assgt in ast.assignments.into_iter() {
        // Insert assignment into map for fast lookup.
        if let Some(_old) = assignments_astnodes.insert(assgt.name.0, assgt) {
            return Err("Duplicate assignment.");
        }
    }

    Ok(assignments_astnodes)
}

fn build_value<'a>(
    expr: Expression<'a>,
    rt: &mut RuntimeEnv,
) -> Index {
    use Operation::*;
    // use compute::VarType;

    match expr {
        Expression::ConstInteger(i) => rt.node_from_operation(Const(Rc::new(VarType::Int(i)))),
        Expression::ConstString(s) => {
            let charvec = s.chars().map(|c| rt.node_from_operation(Const(Rc::new(VarType::Char(c))))).collect();
            rt.node_from_operation(Vector(charvec))
        },
        Expression::ValueRef(Name(n)) => rt.by_name[n],
        Expression::FunctionApplication { function, arguments } => {
            panic!("Not yet implemented!")
        }
        Expression::IfElse {
            guard,
            body,
            else_body,
        } => {
            let guard_idx = build_value(*guard, rt);
            let body_idx = build_value(*body, rt);
            let else_idx = build_value(*else_body, rt);

            rt.node_from_operation(IfElse(guard_idx, body_idx, else_idx))
        }

        Expression::Range { from:_, to: _ } => panic!("Not yet implemented!"),
    }
}

pub fn build<'a>(ast: TempuraAST<'a>) -> Result<RuntimeEnv, &str> {
    // let mut assignments = HashMap::new();

    // let stdin = nodes.insert("The quick brown fox jumped over the lazy dog.".to_string());
    // assignments.insert("stdin", stdin);

    let mut ast_index = index_assignments(ast)?;

    let mut ts = TopologicalSort::<&'a str>::new();

    for (name, assgt) in ast_index.iter() {
        for ref_to in assgt.expr.collect_dependencies() {
            ts.add_dependency(ref_to, *name);
        }
    }

    let mut re = RuntimeEnv {
        nodes: Arena::new(),
        by_name: HashMap::new()
    };

    println!("AST: {:?}", ast_index);

    while let Some(name) = ts.pop() {
        if !re.by_name.contains_key(name) {
            println!("{}", name);
            let val = build_value(
                ast_index.remove(name).unwrap().expr,
                &mut re,
            );
            re.by_name.insert(name.to_string(), val);
        }
    }

    if !ts.is_empty() {
        return Err("Circular dependency!");
    }

    Ok(re)
}
