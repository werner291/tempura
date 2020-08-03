use crate::program::*;
/// Contains code necessary to run a Tempura program in built form.
use generational_arena::{Arena, Index};
use std::fmt::Debug;
use std::rc::Rc;

pub struct Node {
    value_cache: Option<VarType>,
    operation: Operation<NodeIndex>,
    dependents: Vec<Index>,
    being_computed: bool,
}

pub struct RuntimeEnv {
    nodes: Arena<Node>,
    // pub modules: Arena<RuntimeModule>
}

// pub fn lift_reference_into_context(
//     v: &ValueRef,
//     indices: &[Index],
//     inputs: &[NodeIndex],
//     depth: usize
// ) -> ValueRef {
//     if v.up() == Some(depth) {
//         ValueRef::InstanciatedRef(match v {
//             ValueRef::ContextRef { up:_, index } => NodeIndex(indices[*index]),
//             ValueRef::InputRef { up:_, index } => inputs[*index],
//             ValueRef::InstanciatedRef(ni) => *ni,
//         })
//     } else {
//         v
//     }
// }

// pub fn lift_reference_into_runtime(
//     v: &ValueRef,
//     indices: &[Index],
//     inputs: &[NodeIndex],
// ) -> NodeIndex {
//     if v.up() != 1 {
//         panic!("Cannot lift non-top reference.");
//     }

//     match v {
//         ValueRef::ContextRef { up, index } => NodeIndex(indices[*index]),
//         ValueRef::InputRef { up, index } => inputs[*index],
//         ValueRef::InstanciatedRef(ni) => *ni,
//     }
// }

// pub fn lift_variable_into_context(f: &VarType, indices: &[Index], inputs: &[NodeIndex]) -> VarType {
//     match f {
//         VarType::Fragment(f) => {
//             let ff = Fragment {
//                 nodes: f
//                     .nodes
//                     .iter()
//                     .map(|n| lift_operation_into_context(n, indices, inputs))
//                     .collect(),
//                 output: lift_reference_into_context(&f.output, indices, inputs),
//             };
//             VarType::Fragment(Rc::new(ff))
//         }
//         VarType::Bool(b) => VarType::Bool(*b),
//         VarType::Int(i) => VarType::Int(*i),
//         VarType::Char(c) => VarType::Char(*c),
//         VarType::Vector(v) => VarType::Vector(Rc::new(
//             v.iter()
//                 .map(|n| lift_variable_into_context(n, indices, inputs))
//                 .collect(),
//         )),
//     }
// }

// pub fn lift_operation_into_context(
//     op: &Operation<ValueRef>,
//     indices: &[Index],
//     inputs: &[NodeIndex],
// ) -> Operation<ValueRef> {
//     use Operation::*;
//     match op {
//         Const(c) => Const(lift_variable_into_context(c, indices, inputs)),
//         Vector(v) => Vector(
//             v.iter()
//                 .map(|n| lift_reference_into_context(n, indices, inputs))
//                 .collect(),
//         ),
//         Sum(a, b) => Sum(
//             lift_reference_into_context(a, indices, inputs),
//             lift_reference_into_context(b, indices, inputs),
//         ),
//         Concat(a, b) => Concat(
//             lift_reference_into_context(a, indices, inputs),
//             lift_reference_into_context(b, indices, inputs),
//         ),
//         ToString(a) => ToString(lift_reference_into_context(a, indices, inputs)),
//         IfElse(a, b, c) => IfElse(
//             lift_reference_into_context(a, indices, inputs),
//             lift_reference_into_context(b, indices, inputs),
//             lift_reference_into_context(c, indices, inputs),
//         ),
//         ApplyFragment(f, args) => ApplyFragment(
//             lift_reference_into_context(f, indices, inputs),
//             args.iter()
//                 .map(|n| lift_reference_into_context(n, indices, inputs))
//                 .collect(),
//         ),
//         // ApplyModule(_m,args) => args.clone(),
//     }
// }

// pub fn lift_operation_into_runtime(
//     op: &Operation<ValueRef>,
//     indices: &[Index],
//     inputs: &[NodeIndex],
// ) -> Operation<NodeIndex> {
//     use Operation::*;
//     match op {
//         Const(c) => Const(lift_variable_into_context(c, indices, inputs)),
//         Vector(v) => Vector(
//             v.iter()
//                 .map(|n| lift_reference_into_runtime(n, indices, inputs))
//                 .collect(),
//         ),
//         Sum(a, b) => Sum(
//             lift_reference_into_runtime(a, indices, inputs),
//             lift_reference_into_runtime(b, indices, inputs),
//         ),
//         Concat(a, b) => Concat(
//             lift_reference_into_runtime(a, indices, inputs),
//             lift_reference_into_runtime(b, indices, inputs),
//         ),
//         ToString(a) => ToString(lift_reference_into_runtime(a, indices, inputs)),
//         IfElse(a, b, c) => IfElse(
//             lift_reference_into_runtime(a, indices, inputs),
//             lift_reference_into_runtime(b, indices, inputs),
//             lift_reference_into_runtime(c, indices, inputs),
//         ),
//         ApplyFragment(f, args) => ApplyFragment(
//             lift_reference_into_runtime(f, indices, inputs),
//             args.iter()
//                 .map(|n| lift_reference_into_runtime(n, indices, inputs))
//                 .collect(),
//         ),
//         // ApplyModule(_m,args) => args.clone(),
//     }
// }

impl RuntimeEnv {
    pub fn new() -> RuntimeEnv {
        RuntimeEnv {
            nodes: Arena::new(),
        }
    }

    pub fn node_from_operation(&mut self, operation: Operation<NodeIndex>) -> NodeIndex {
        let dependencies = operation.dependencies();

        let node = self.nodes.insert(Node {
            value_cache: None,
            operation,
            dependents: Vec::new(),
            being_computed: false,
        });

        for dep in dependencies {
            self.nodes[dep.0].dependents.push(node)
        }

        NodeIndex(node)
    }

    fn compute_value(&mut self, idx: NodeIndex) -> VarType {
        use Operation::*;

        let mut node = &mut self.nodes[idx.0];

        if node.being_computed {
            panic!("Circular dependency detected!");
        }

        node.being_computed = true;

        let new_val = match node.operation.clone() {
            Const(v) => v,
            Vector(v) => {
                VarType::Vector(Rc::new(v.iter().map(|idx_1| self.pull(*idx_1)).collect()))
            }
            Sum(a, b) => VarType::Int(
                self.pull(a).unpack_int().unwrap() + self.pull(b).unpack_int().unwrap(),
            ),
            Concat(a, b) => {
                let va: Rc<Vec<VarType>> = self
                    .pull(a)
                    .unpack_vector()
                    .expect("can only concat vectors");
                let vb: Rc<Vec<VarType>> = self
                    .pull(b)
                    .unpack_vector()
                    .expect("can only concat vectors");
                VarType::Vector(Rc::new(
                    va.iter().cloned().chain(vb.iter().cloned()).collect(),
                ))
            }
            ToString(a) => {
                // match  {
                let s = format!("{:?}", self.pull(a));
                VarType::from_string(&s)
                // VarType::Int(i) => VarType::from_string(i.to_string()),
                // VarType::Bool(b) => VarType::from_string(b.to_string()),
                // VarType::Char(c) =>  VarType::from_string(c.to_string()),
                // VarType::Fragment(f) => VarType::from_string("<fragment>"),
                // VarType::Vector(v) => v.iter().map()
                // }
            }

            //.unpack_int().unwrap() + self.pull(b).unpack_int().unwrap(),
            ,
            IfElse(g, b, eb) => {
                if self.pull(g).unpack_bool().unwrap() {
                    self.pull(b)
                } else {
                    self.pull(eb)
                }
            }
            ApplyFragment(fref, args) => {
                let fragref = self.pull(fref).unpack_fragment().unwrap().clone();
                let outref = self.instantiate_fragment(fragref.as_ref(), args);
                self.pull(outref)
            }
        };
        self.nodes[idx.0].value_cache = Some(new_val.clone());
        self.nodes[idx.0].being_computed = false;
        new_val
    }

    pub fn pull(&mut self, idx: NodeIndex) -> VarType {
        let node = &mut self.nodes[idx.0];

        match &node.value_cache {
            Some(x) => x.clone(),
            None => self.compute_value(idx),
        }
    }

    pub fn instantiate_fragment(
        &mut self,
        frag: &Fragment<ValueRef>,
        arguments: Vec<NodeIndex>,
    ) -> NodeIndex {

        let indices = self.nodes.insert_many_with(frag.nodes.len(), |indices| {

            let noderefs : Vec<NodeIndex> = indices.iter().cloned().map(NodeIndex).collect();

            frag.fill_in(noderefs.as_slice(), arguments.as_slice(), 0)
                .finalize()
                .nodes.into_iter()
                .map(|op| {
                    Node {
                        value_cache: None,
                        being_computed: false,
                        operation: op,
                        dependents: vec![],
                    }
                })
                .collect()
        });

        NodeIndex(indices[frag.output])
    }
}
