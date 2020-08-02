/// Contains code necessary to run a Tempura program in built form.

use generational_arena::{Arena, Index};
use crate::program::*;
use std::rc::Rc;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct NodeIndex(Index);

// #[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
// pub struct FragNodeIndex(pub usize);

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct FragIndex(pub usize);

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

impl RuntimeEnv {
    pub fn new() -> RuntimeEnv {
        RuntimeEnv {
            nodes: Arena::new(),
        }
    }

    // pub fn alloc_node(&mut self, node: Node) -> NodeIndex {
    //     NodeIndex(self.nodes.insert(node))
    // }

    // pub fn alloc_module(&mut self, modu: RuntimeModule) -> ModuleIndex {
    //     ModuleIndex(self.modules.insert(modu))
    // }

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
            Vector(v) => VarType::Vector(
                Rc::new(v.iter().map(|idx_1| self.pull(*idx_1)).collect()),
            ),
            Sum(a, b) => VarType::Int(
                self.pull(a).unpack_int().unwrap() + self.pull(b).unpack_int().unwrap(),
            ),
            IfElse(g, b, eb) => {
                if self.pull(g).unpack_bool().unwrap() {
                    self.pull(b)
                } else {
                    self.pull(eb)
                }
            },
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

    pub fn instantiate_fragment(&mut self, frag: &Fragment, arguments: Vec<NodeIndex>) -> NodeIndex {
        
        // self.nodes.insert_many_with(n: usize, create: F)

            // let translate_ref = |from: ValueRef| {
            //     if from.depth() < env.depth {
            //         from
            //     } else {
            //         match from {
            //             ValueRef::ContextRef { depth:d, index } => 
            //                 ValueRef::ContextRef { depth:d-1, index: index_from + index },
            //             ValueRef::InputRef { depth:d, index } => 
            //                 argrefs[index]
            //         }
            //     }
            // };

            // for (idx, sub_val) in frag.nodes.into_iter().enumerate() {
            //     env.values.push(sub_val.map_ref(|r| translate_ref(*r)));
            // }

            // translate_ref(frag.output)

            // frag
            // let frag = env.fragments[frag];

            // match frag_ref {
            //     Fra
            // }
            // .nodes.iter().map(|n| match n {

            // });

        panic!("TODO")
    }
}
