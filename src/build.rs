use crate::ast::*;
use crate::program::*;
use std::collections::HashMap;
use std::rc::Rc;
use topological_sort::TopologicalSort;
use generational_arena::Arena;

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

fn build_value<'a>(expr: Expression<'a>, 
                   env: &mut FragmentBuilder<'a>) -> ValueRef {

    use Operation::*;
    // use compute::VarType;

    match expr {
        Expression::ConstInteger(i) => {
            env.alloc_value(Operation::Const(VarType::Int(i)))
        },
        Expression::ConstString(s) => {
            let charvec = s
                .chars()
                .map(|c| {
                    env.alloc_value(Const(VarType::Char(c)))
                })
                .collect();
            env.alloc_value(Vector(charvec))
        }
        Expression::ValueRef(Name(n)) => env.lookup_value(n).unwrap(),
        Expression::ModuleApplication {
            mod_name,
            arguments,
        } => {
            
            let argrefs : Vec<ValueRef> = arguments
                .into_iter()
                .map(|arg| build_value(arg, env))
                .collect();

            // TODO Avoid redundancy here.

            let fragref = env.resolve_fragment_name(mod_name.0).unwrap();

            let frag = env.alloc_value(Operation::Const(VarType::Fragment(env.get_fragment(&fragref).unwrap().clone())));

            env.alloc_value(Operation::ApplyFragment(frag, argrefs))
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

// pub fn build_module<'a>(declarations : Vec<Assignment>) -> {

// }

pub struct FragmentBuilder<'a> {
    pub values_by_name: HashMap<String, ValueRef>,
    pub values : Vec<Operation<ValueRef>>,
    pub fragments_by_name: HashMap<String, FragmentRef>,
    pub fragments : Vec<Rc<Fragment>>,
    parent: Option<&'a FragmentBuilder<'a>>,
    depth: usize
}

impl<'a> FragmentBuilder<'a> {

    pub fn new() -> FragmentBuilder<'static> {
        FragmentBuilder {
            values_by_name: HashMap::new(),
            fragments_by_name: HashMap::new(),
            values: Vec::new(),
            fragments: Vec::new(),
            parent: None,
            depth: 0
        }
    }

    pub fn lookup_value(&self, name: &str) -> Option<ValueRef> {
        match self.values_by_name.get(name) {
            Some(idx) => Some(*idx),
            None => match self.parent {
                Some(par) => {
                    par.lookup_value(name)
                }
                None => None,
            },
        }
    }

    pub fn resolve_fragment_name(&self, name: &str) -> Option<FragmentRef> {
        match self.fragments_by_name.get(name) {
            Some(idx) => Some(*idx),
            None => match self.parent {
                Some(par) => {
                    par.resolve_fragment_name(name)
                }
                None => None,
            },
        }
    }

    pub fn get_fragment(&self, fref: &FragmentRef) -> Option<&Rc<Fragment>> {
        if fref.depth < self.depth {
            self.parent.unwrap().get_fragment(fref)
        } else {
            self.fragments.get(fref.index)
        }
    }

    pub fn alloc_value(&mut self, value: Operation<ValueRef>) -> ValueRef {
        self.values.push(value);
        ValueRef::ContextRef {
            depth: self.depth,
            index: self.values.len() - 1
        }
    }

    pub fn alloc_fragment(&mut self, frag: Fragment) -> FragmentRef {
        self.fragments.push(Rc::new(frag));
        FragmentRef {
            depth: self.depth,
            index: self.fragments.len() - 1
        }
    }

    fn derive_child(&'a self) -> FragmentBuilder<'a> {
        FragmentBuilder {
            values_by_name: HashMap::new(),
            values: Vec::new(),
            fragments_by_name: HashMap::new(),
            fragments: Vec::new(),
            parent: Some(self),
            depth: self.depth+1
        }
    }

    // pub fn pull_fragment_into_environment(&self, fragref: &FragmentRef) -> Fragment {
    //     match fragref {
    //         FragmentRef::SiblingRef(direct) => self.fragments[*direct].clone(),
    //         FragmentRef::InParentRef(ref_in_parent) => {
    //             let frag = self.parent.unwrap().pull_fragment_into_environment(&*ref_in_parent);
    //             Fragment {
    //                 nodes: frag.nodes.into_iter().map(|op| op.ref_map(|val_ref| {
    //                     match val_ref {
    //                         ValueRef::InputRef(i) => ValueRef::InputRef(i),
    //                         x => ValueRef::InParentRef(Box::new(x))
    //                     }
    //                 })).collect(),
    //                 fragments: frag.
    //                 output: ValueRef::InParentRef(Box::new(frag.output))
    //             }
    //         }
    //     }
    // }
}

pub fn build_module<'a>(
    modu: Module<'a>,
    parent_env: &FragmentBuilder,
) -> Result<Fragment, &'static str> {

    let mut ast_index: HashMap<&str, Assignment> = index_named(modu.assignments)?;
    let mut mod_index: HashMap<&str, Module> = index_named(modu.submodules)?;

    let mut fb = parent_env.derive_child();

    let mut ts = TopologicalSort::<Dependency>::new();

    for (name, assgt) in ast_index.iter() {
        for ref_to in assgt.expr.collect_dependencies() {
            ts.add_dependency(ref_to, Dependency::Value(name.to_string()));
        }
    }

    for (name, modl) in mod_index.iter() {
        for ref_to in modl.collect_dependencies() {
            ts.add_dependency( Dependency::Module(name.to_string()), ref_to);
        }
    }

    while let Some(dep) = ts.pop() {
        match dep {
            Dependency::Module(modname) => {
                if fb.resolve_fragment_name(&modname).is_none() {
                    let modl = mod_index.remove(modname.as_str()).unwrap();
                    let frag = build_module(modl, &fb)?;
                    let fref = fb.alloc_fragment(frag);
                    fb.fragments_by_name.insert(modname, fref);
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

    Ok(Fragment {
        nodes: fb.values,
        output
    })


    // }));
}

pub fn build_toplevel_module<'a>(
    modu: Module<'a>
) -> Result<Fragment, &'static str> {

    build_module(modu, &FragmentBuilder::new())
}

// pub fn build(ast: Module) -> Result<(RuntimeEnv, EnvStack), &str> {
//     assert_eq!(ast.name, Name("main"));

//     // let mut assignments = HashMap::new();

//     // let stdin = nodes.insert("The quick brown fox jumped over the lazy dog.".to_string());
//     // assignments.insert("stdin", stdin);

//     let mut value_name_env = EnvStack::new();

//     let mut re = RuntimeEnv::new();

//     value_name_env.insert_module(
//         "main".to_string(),
//         build_module(ast, &mut re, &value_name_env).expect("Build failed."),
//     );

//     Ok((re, value_name_env))
// }
