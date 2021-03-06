use crate::program::*;
/// Contains code necessary to run a Tempura program in built form.
use generational_arena::{Arena};
use std::rc::Rc;
use crate::ast;


pub struct Node {
    last_update: Time,
    value_cache: Option<VarType>,
    operation: Operation<NodeIndex>,
    dependents: Vec<NodeIndex>,
    listeners: Vec<Box<dyn Fn(Time, &VarType)>>,
    being_computed: bool,
}

pub struct RuntimeEnv {
    current_time: Time,
    nodes: Arena<Node>,
    pub stdout: Option<NodeIndex>,
    pub stdin: Option<NodeIndex>,
    pub clock: Option<NodeIndex>,
}

type Time = u64;

impl RuntimeEnv {
    pub fn new() -> RuntimeEnv {
        RuntimeEnv {
            nodes: Arena::new(),
            stdout: None,
            stdin: None,
            clock: None,
            current_time: 0,
        }
    }

    pub fn node_from_operation(&mut self, operation: Operation<NodeIndex>) -> NodeIndex {
        let dependencies = operation.dependencies();

        let node = NodeIndex(self.nodes.insert(Node {
            value_cache: None,
            operation,
            dependents: Vec::new(),
            listeners: Vec::new(),
            being_computed: false,
            last_update: 0,
        }));

        for dep in dependencies {
            self.nodes[dep.0].dependents.push(node)
        }

        node
    }

    fn compute_value(&mut self, idx: NodeIndex) -> VarType {
        use Operation::*;

        let mut node = &mut self.nodes[idx.0];

        if node.being_computed {
            panic!("Circular dependency detected!");
        }

        node.being_computed = true;

        let new_val = match node.operation.clone() {
            External => node.value_cache.clone().unwrap_or(VarType::Null),
            Const(v) => v,
            Vector(v) => VarType::Vector(Rc::new(
                v.iter().map(|idx_1| self.pull_once(*idx_1)).collect(),
            )),
            BinaryOp(a, b, opr) => {
                let aa = self.pull_once(a);
                let bb = self.pull_once(b);

                match opr {
                    ast::BinaryOp::Sum => VarType::Int(aa.unpack_int().unwrap() + bb.unpack_int().unwrap()),
                    ast::BinaryOp::Concat => VarType::Vector(Rc::new(
                        aa.unpack_vector().unwrap().iter().cloned()
                            .chain(bb.unpack_vector().unwrap().iter().cloned()).collect(),
                    )),
                    ast::BinaryOp::Range => unimplemented!(),
                    ast::BinaryOp::Eq => VarType::Bool(match aa {
                        VarType::Bool(aa) => aa == bb.unpack_bool().unwrap(),
                        VarType::Char(aa) => aa == bb.unpack_char().unwrap(),
                        VarType::Int(aa) => aa == bb.unpack_int().unwrap(),
                        _ => panic!("Comparison unsupported for vartype.")
                    }),
                    ast::BinaryOp::Gt  => VarType::Bool(aa.unpack_int().unwrap() > bb.unpack_int().unwrap()),
                    ast::BinaryOp::Geq => VarType::Bool(aa.unpack_int().unwrap() >= bb.unpack_int().unwrap()),
                    ast::BinaryOp::Lt  => VarType::Bool(aa.unpack_int().unwrap() < bb.unpack_int().unwrap()),
                    ast::BinaryOp::Leq => VarType::Bool(aa.unpack_int().unwrap() <= bb.unpack_int().unwrap()),
                    ast::BinaryOp::Index =>
                        aa.unpack_vector().expect("can only index into a vector")[bb.unpack_int().expect("can only index with an int index") as usize].clone()
                }
            },
            ToString(a) => VarType::from_string(&self.pull_once(a).render_as_string()),
            IfElse(g, b, eb) => {
                if self.pull_once(g).unpack_bool().unwrap() {
                    self.pull_once(b)
                } else {
                    self.pull_once(eb)
                }
            }
            ApplyFragment(fref, args) => {
                let fragref = self.pull_once(fref).unpack_fragment().unwrap().clone();
                let outref = self.instantiate_fragment(fragref.as_ref(), args);
                self.pull_once(outref)
            }
        };
        self.nodes[idx.0].value_cache = Some(new_val.clone());
        self.nodes[idx.0].being_computed = false;
        self.nodes[idx.0].last_update = self.current_time;
        new_val
    }

    pub fn pull_once(&mut self, idx: NodeIndex) -> VarType {
        let node = &mut self.nodes[idx.0];

        match &node.value_cache {
            Some(x) => x.clone(),
            None => self.compute_value(idx),
        }
    }

    pub fn listen(
        &mut self,
        idx: NodeIndex,
        include_current: bool,
        cb: Box<dyn Fn(Time, &VarType)>,
    ) {
        if include_current {
            cb(self.current_time, &self.pull_once(idx))
        }
        self.nodes[idx.0].listeners.push(cb);
    }

    pub fn put_current(&mut self, idx: NodeIndex, value: VarType) -> Time {
        self.current_time += 1;
        self.nodes[idx.0].value_cache = Some(value);
        self.nodes[idx.0].last_update = self.current_time;
        self.update_dependents(idx);
        self.current_time
    }

    pub fn update_dependents(&mut self, idx: NodeIndex) {
        for dep in self.nodes[idx.0].dependents.clone() {
            if self.nodes[dep.0].last_update < self.current_time {
                self.compute_value(dep);
                self.update_dependents(dep);
            }
        }
        let cur = self.pull_once(idx);
        for cb in self.nodes[idx.0].listeners.iter() {
            cb(self.current_time, &cur);
        }
    }

    pub fn instantiate_fragment(
        &mut self,
        frag: &Fragment<LacunaryRef>,
        arguments: Vec<NodeIndex>,
    ) -> NodeIndex {
        let indices = self.nodes.insert_many_with(frag.nodes.len(), |indices| {
            let noderefs: Vec<NodeIndex> = indices.iter().cloned().map(NodeIndex).collect();

            frag.fill_in(noderefs.as_slice(), arguments.as_slice(), 0)
                .finalize()
                .nodes
                .into_iter()
                .map(|op| Node {
                    value_cache: None,
                    being_computed: false,
                    operation: op,
                    dependents: vec![],
                    listeners: vec![],
                    last_update: 0,
                })
                .collect()
        });

        for idx in indices.iter() {
            for dep in self.nodes[*idx].operation.dependencies() {
                self.nodes[dep.0].dependents.push(NodeIndex(*idx))
            }
        }

        let noderefs: Vec<NodeIndex> = indices.iter().cloned().map(NodeIndex).collect();

        frag.output
            .fill_in(noderefs.as_slice(), arguments.as_slice(), 0)
            .finalize()
    }
}
