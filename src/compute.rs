use generational_arena::{Arena, Index};
use std::any::Any;
use std::rc::Rc;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug)]
pub enum VarType {
    Int(i64),
    Bool(bool),
    Char(char),
    Vector(Vec<Rc<VarType>>)
}

impl VarType {
    
    pub fn unpack_int(&self) -> Option<i64> {
        if let VarType::Int(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn unpack_bool(&self) -> Option<bool> {
        if let VarType::Bool(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn unpack_char(&self) -> Option<char> {
        if let VarType::Char(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn unpack_vector(&self) -> Option<&Vec<Rc<VarType>>> {
        if let VarType::Vector(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn stringify(&self) -> Option<String> {
        self.unpack_vector().and_then(|v| {
            let mut result = "".to_string();
            for c in v.iter() {
                match c.unpack_char() {
                    Some(cc) => result.push(cc),
                    None => return None
                }
            }
            Some(result)
        })
    }
}

pub struct Node {
    value_cache: Option<Rc<VarType>>,
    operation: Operation,
    dependents: Vec<Index>,
    being_computed: bool
}

#[derive(Clone)]
pub enum Operation {
    Const(Rc<VarType>),
    Vector(Vec<Index>),
    Sum(Index, Index),
    IfElse(Index, Index, Index),
}

impl Operation {
    fn dependencies(&self) -> Vec<Index> {
        use Operation::*;
        match self {
            Const(_) => Vec::new(),
            Vector(v) => v.clone(),
            Sum(a, b) => vec![*a, *b],
            IfElse(a, b, c) => vec![*a, *b, *c],
        }
    }
}

pub struct RuntimeEnv {
    pub nodes: Arena<Node>,
    pub by_name: HashMap<String, Index>,
}

impl RuntimeEnv {
    pub fn node_from_operation(&mut self, operation: Operation) -> Index {
        let dependencies = operation.dependencies();

        let node = self.nodes.insert(Node {
            value_cache: None,
            operation,
            dependents: Vec::new(),
            being_computed: false,
        });

        for dep in dependencies {
            self.nodes[dep].dependents.push(node)
        }

        node
    }

    fn compute_value(&mut self, idx: Index) -> Rc<VarType> {
        use Operation::*;

        let mut node = &mut self.nodes[idx];

        if node.being_computed {
            panic!("Circular dependency detected!");
        }

        node.being_computed = true;

        let new_val = match node.operation.clone() {
            Const(v) => v,
            Vector(v) => Rc::new(
                VarType::Vector(v.iter()
                    .map(|idx_1| self.pull(*idx_1))
                    .collect()),
            ),
            Sum(a, b) => Rc::new(VarType::Int(
                self.pull(a).unpack_int().unwrap() + self.pull(b).unpack_int().unwrap(),
            )),
            IfElse(g, b, eb) => {
                if self.pull(g).unpack_bool().unwrap() {
                    self.pull(b)
                } else {
                    self.pull(eb)
                }
            }
        };
        self.nodes[idx].value_cache = Some(new_val.clone());
        self.nodes[idx].being_computed = false;
        new_val
    }

    pub fn pull(&mut self, idx: Index) -> Rc<VarType> {
        let node = &mut self.nodes[idx];

        match &node.value_cache {
            Some(x) => x.clone(),
            None => self.compute_value(idx),
        }
    }
}