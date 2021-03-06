/// This module contains the code necessary to transform
/// an AST fresh from the parser into a form that can
/// be executed.
use crate::ast::*;
use crate::program::*;
use std::collections::HashMap;
use std::rc::Rc;
use topological_sort::TopologicalSort;

pub mod fragment_builder;
use crate::run::RuntimeEnv;
use fragment_builder::*;

trait Named {
    fn name(&self) -> &Name;
}

impl Named for AssignmentAST {
    fn name(&self) -> &Name {
        &self.name
    }
}

impl Named for FragmentAST {
    fn name(&self) -> &Name {
        &self.name
    }
}

/// Take a Vector of Named, and return a HashMap keyed by name.
fn index_named<T: Named>(assignments: Vec<T>) -> Result<HashMap<String, T>, &'static str> {
    // Map of assignments, from name to the AST entry.
    // This is basically ast.assignments but in map form without duplicates.
    let mut assignments_astnodes = HashMap::new();

    for assgt in assignments.into_iter() {
        // Insert assignment into map for fast lookup.
        if let Some(_old) = assignments_astnodes.insert(assgt.name().0.clone(), assgt) {
            return Err("Duplicate assignment.");
        }
    }

    Ok(assignments_astnodes)
}

fn build_value(expr: Expression, env: &mut FragmentBuilder) -> Result<LacunaryRef, &'static str> {
    use Operation::*;
    // use compute::VarType;

    Ok(match expr {
        Expression::ConstInteger(i) => env.alloc_value(Operation::Const(VarType::Int(i))),
        Expression::ConstBoolean(b) => env.alloc_value(Operation::Const(VarType::Bool(b))),
        Expression::ConstString(s) => {
            let charvec = s
                .chars()
                .map(|c| env.alloc_value(Const(VarType::Char(c))))
                .collect();
            env.alloc_value(Vector(charvec))
        },
        Expression::LacunaryRef(Name(n)) => match env.lookup_value(&n) {
            Some(r) => r,
            None => panic!("Reference to non-existent value `{}`.", n),
        },
        Expression::ModuleApplication {
            mod_name,
            arguments,
        } => {
            let argrefs: Vec<LacunaryRef> = arguments
                .into_iter()
                .map(|arg| build_value(arg, env))
                .collect::<Result<Vec<_>, _>>()?;

            let fragref = env.lookup_value(&mod_name.0).expect("module not found");

            env.alloc_value(Operation::ApplyFragment(fragref, argrefs))
        },
        Expression::IfElse {
            guard,
            body,
            else_body,
        } => {
            let guard_idx = build_value(*guard, env)?;
            let body_idx = build_value(*body, env)?;
            let else_idx = build_value(*else_body, env)?;

            env.alloc_value(IfElse(guard_idx, body_idx, else_idx))
        },
        Expression::BinaryOp(a,b,op) => {
            let a_idx = build_value(*a, env)?;
            let b_idx = build_value(*b, env)?;

            env.alloc_value(BinaryOp(a_idx, b_idx, op))
        }
    })
}

pub fn build_module(
    modu: FragmentAST,
    parent_env: &FragmentBuilder,
) -> Result<Fragment<LacunaryRef>, &'static str> {
    let mut ast_index = index_named(modu.assignments)?;
    let mut mod_index = index_named(modu.submodules)?;

    let mut fb = parent_env.derive_child(modu.name.0);

    for (index, mi) in modu.inputs.iter().enumerate() {
        fb.values_by_name
            .insert(mi.name.0.to_string(), LacunaryRef::InputRef { up: 0, index });
    }

    let mut ts = TopologicalSort::<Dependency>::new();

    for (name, assgt) in ast_index.iter() {
        for ref_to in assgt.expr.collect_dependencies() {
            ts.add_dependency(ref_to, Dependency::Value(name.to_string()));
        }
    }

    while let Some(dep) = ts.pop() {
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
                    let val_built = build_value(val.expr, &mut fb)?;
                    fb.values_by_name.insert(valname, val_built);
                }
            }
        }
    }

    for dep in modu.output.collect_dependencies() {
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
                    let val_built = build_value(val.expr, &mut fb)?;
                    fb.values_by_name.insert(valname, val_built);
                }
            }
        }
    }

    let output = build_value(modu.output, &mut fb)?;

    Ok(fb.build(output))
}

pub fn build_runtime(main_module: FragmentAST) -> Result<RuntimeEnv, &'static str> {
    let stdlib = vec![
        Fragment {
            name: "to_string".to_string(),
            nodes: vec![Operation::ToString(LacunaryRef::InputRef { up: 0, index: 0 })],
            output: LacunaryRef::ContextRef { up: 0, index: 0 },
        },
        Fragment {
            name: "concat".to_string(),
            nodes: vec![Operation::BinaryOp(
                LacunaryRef::InputRef { up: 0, index: 0 },
                LacunaryRef::InputRef { up: 0, index: 1 },
                BinaryOp::Concat
            )],
            output: LacunaryRef::ContextRef { up: 0, index: 0 },
        },
    ];

    let mut re = RuntimeEnv::new();
    let mut fb = FragmentBuilder::new("".to_string());

    for f in stdlib {
        let name = f.name.clone();
        let n = re.node_from_operation(Operation::Const(VarType::Fragment(Rc::new(f))));
        fb.values_by_name.insert(name, LacunaryRef::InstanciatedRef(n));
    }

    let stdin = re.node_from_operation(Operation::External);
    fb.values_by_name
        .insert("stdin".to_string(), LacunaryRef::InstanciatedRef(stdin));

    let clock = re.node_from_operation(Operation::External);
    re.put_current(clock, VarType::Int(0));
    fb.values_by_name
        .insert("clock".to_string(), LacunaryRef::InstanciatedRef(clock));

    let mainmod = build_module(main_module, &fb).unwrap();

    let stdout = re.instantiate_fragment(&mainmod, vec![]);

    re.stdout = Some(stdout);
    re.stdin = Some(stdin);
    re.clock = Some(clock);

    Ok(re)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nom_parse as prs;
    use nom::error::VerboseError;

    #[test]
    fn test_typeerror() {
        
        let mut env = FragmentBuilder::new("test_environment".to_string());

        assert!(build_value(prs::expression::<VerboseError<&str>>(r#"1 + "apple""#).unwrap().1, &mut env).is_err());

    }

}