use crate::ast::*;
use crate::program::*;
use std::collections::HashMap;
use topological_sort::TopologicalSort;

mod fragment_builder;
use fragment_builder::*;

trait Named<'a> {
    fn name(&self) -> Name<'a>;
}

impl<'a> Named<'a> for Assignment<'a> {
    fn name(&self) -> Name<'a> {
        self.name
    }
}

impl<'a> Named<'a> for Module<'a> {
    fn name(&self) -> Name<'a> {
        self.name
    }
}

fn index_named<'a, T: Named<'a>>(assignments: Vec<T>) -> Result<HashMap<&'a str, T>, &'static str> {
    // Map of assignments, from name to the AST entry.
    // This is basically ast.assignments but in map form without duplicates.
    let mut assignments_astnodes = HashMap::new();

    for assgt in assignments.into_iter() {
        // Insert assignment into map for fast lookup.
        if let Some(_old) = assignments_astnodes.insert(assgt.name().0, assgt) {
            return Err("Duplicate assignment.");
        }
    }

    Ok(assignments_astnodes)
}

fn build_value<'a>(expr: Expression<'a>, env: &mut FragmentBuilder<'a>) -> ValueRef {
    use Operation::*;
    // use compute::VarType;

    match expr {
        Expression::ConstInteger(i) => env.alloc_value(Operation::Const(VarType::Int(i))),
        Expression::ConstBoolean(b) => env.alloc_value(Operation::Const(VarType::Bool(b))),
        Expression::ConstString(s) => {
            let charvec = s
                .chars()
                .map(|c| env.alloc_value(Const(VarType::Char(c))))
                .collect();
            env.alloc_value(Vector(charvec))
        }
        Expression::ValueRef(Name(n)) => env.lookup_value(n).unwrap(),
        Expression::ModuleApplication {
            mod_name,
            arguments,
        } => {
            let argrefs: Vec<ValueRef> = arguments
                .into_iter()
                .map(|arg| build_value(arg, env))
                .collect();

            let fragref = env.lookup_value(mod_name.0).expect("module not found");

            env.alloc_value(Operation::ApplyFragment(fragref, argrefs))
        }
        Expression::IfElse {
            guard,
            body,
            else_body,
        } => {
            let guard_idx = build_value(*guard, env);
            let body_idx = build_value(*body, env);
            let else_idx = build_value(*else_body, env);

            env.alloc_value(IfElse(guard_idx, body_idx, else_idx))
        }

        Expression::Range { from: _, to: _ } => panic!("Not yet implemented!"),
    }
}

pub fn build_module<'a>(
    modu: Module<'a>,
    parent_env: &FragmentBuilder,
) -> Result<Fragment<ValueRef>, &'static str> {
    let mut ast_index: HashMap<&str, Assignment> = index_named(modu.assignments)?;
    let mut mod_index: HashMap<&str, Module> = index_named(modu.submodules)?;

    let mut fb = parent_env.derive_child();

    for (index, mi) in modu.inputs.iter().enumerate() {
        fb.values_by_name
            .insert(mi.name.0.to_string(), ValueRef::InputRef { up: 0, index });
    }

    let mut ts = TopologicalSort::<Dependency>::new();

    for (name, assgt) in ast_index.iter() {
        for ref_to in assgt.expr.collect_dependencies() {
            ts.add_dependency(ref_to, Dependency::Value(name.to_string()));
        }
    }

    for (name, modl) in mod_index.iter() {
        for ref_to in modl.collect_dependencies() {
            ts.add_dependency(Dependency::Module(name.to_string()), ref_to);
        }
    }

    while let Some(dep) = ts.pop() {
        println!("Dep: {:?}", dep);
        match dep {
            Dependency::Module(modname) => {
                if fb.lookup_value(&modname).is_none() {
                    let modl = mod_index.remove(modname.as_str()).unwrap();
                    let frag = build_module(modl, &fb)?;
                    let fref = fb.alloc_fragment(frag);
                    fb.values_by_name.insert(modname, fref);
                }
            }
            Dependency::Value(valname) => {
                if fb.lookup_value(&valname).is_none() {
                    let val = ast_index.remove(valname.as_str()).unwrap();
                    let val_built = build_value(val.expr, &mut fb);
                    fb.values_by_name.insert(valname, val_built);
                }
            }
        }
    }

    let output = build_value(modu.output, &mut fb);

    Ok(fb.build(output))
}

pub fn build_toplevel_module<'a>(modu: Module<'a>) -> Result<Fragment<ValueRef>, &'static str> {
    let mut fb = FragmentBuilder::new();

    let to_string = fb.alloc_fragment(Fragment {
        nodes: vec![Operation::ToString(ValueRef::InputRef { up: 0, index: 0 })],
        output: 0,
    });

    fb.values_by_name.insert("to_string".to_string(), to_string);

    let concat = fb.alloc_fragment(Fragment {
        nodes: vec![Operation::Concat(
            ValueRef::InputRef { up: 0, index: 0 },
            ValueRef::InputRef { up: 0, index: 1 },
        )],
        output: 0,
    });

    fb.values_by_name.insert("concat".to_string(), concat);

    build_module(modu, &fb)
}