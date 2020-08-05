use generational_arena::Index;
use itertools::join;
use std::iter;
use std::rc::Rc;
use crate::ast::BinaryOp;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct NodeIndex(pub Index);

#[derive(Debug, Clone)]
pub enum VarType {
    Null,
    Int(i64),
    Bool(bool),
    Char(char),
    Vector(Rc<Vec<VarType>>),
    Fragment(Rc<Fragment<LacunaryRef>>),
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

    pub fn unpack_vector(&self) -> Option<Rc<Vec<VarType>>> {
        if let VarType::Vector(v) = self {
            Some(v.clone())
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
                    None => return None,
                }
            }
            Some(result)
        })
    }

    pub fn from_string(string: &str) -> VarType {
        VarType::Vector(Rc::new(string.chars().map(VarType::Char).collect()))
    }

    pub fn unpack_fragment(&self) -> Option<&Rc<Fragment<LacunaryRef>>> {
        if let VarType::Fragment(f) = self {
            Some(f)
        } else {
            None
        }
    }

    pub fn render_as_string(&self) -> String {
        match self {
            VarType::Null => "null".to_string(),
            VarType::Int(i) => i.to_string(),
            VarType::Bool(b) => b.to_string(),
            VarType::Char(c) => c.to_string(),
            VarType::Fragment(f) => format!("{:?}", f),
            VarType::Vector(v) => {
                format!("[{}]", join(v.iter().map(VarType::render_as_string), ","))
            }
        }
    }
}

use std::fmt::Debug;
use std::hash::Hash;

// #[derive(PartialEq, Eq, Debug)]
#[derive(Clone, Debug)]
pub enum Operation<I: Clone + Copy + Debug> {
    External,
    Const(VarType),
    Vector(Vec<I>),
    BinaryOp(I, I, BinaryOp),
    ToString(I),
    IfElse(I, I, I),
    ApplyFragment(I, Vec<I>),
}

impl<I: Copy + Debug> Operation<I> {
    pub fn dependencies(&self) -> Vec<I> {
        use Operation::*;
        match self {
            External => vec![],
            Const(_) => Vec::new(),
            Vector(v) => v.clone(),
            BinaryOp(a,b,_) => vec![*a, *b],
            ToString(a) => vec![*a],
            IfElse(a, b, c) => vec![*a, *b, *c],
            ApplyFragment(f, args) => iter::once(*f).chain(args.iter().cloned()).collect(),
        }
    }
}

// pub struct RuntimeModule(pub Box<dyn Fn(Vec<NodeIndex>, &mut RuntimeEnv) -> NodeIndex>);

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum LacunaryRef {
    InputRef { up: usize, index: usize },
    ContextRef { up: usize, index: usize },
    InstanciatedRef(NodeIndex),
}

impl LacunaryRef {
    /// How many parent contexts up in which to look for the value being referenced to.
    /// Returns None if the reference has already been fulfilled.
    pub fn up(&self) -> Option<usize> {
        use LacunaryRef::*;
        match self {
            InputRef { up, index: _ } => Some(*up),
            ContextRef { up, index: _ } => Some(*up),
            InstanciatedRef(_) => None,
        }
    }
}

pub trait Lacunary<F> {
    fn fill_in(&self, nodes: &[NodeIndex], inputs: &[NodeIndex], depth: usize) -> Self;

    fn finalize(self) -> F;
}

#[derive(Clone, Debug)]
pub struct Fragment<I: Copy + Debug> {
    // TODO maybe add an index of stuff to be filled in?
    // Current algorithm is kinda expensive.
    pub name: String,
    pub nodes: Vec<Operation<I>>,
    pub output: I,
}

impl Lacunary<Fragment<NodeIndex>> for Fragment<LacunaryRef> {
    fn fill_in(
        &self,
        nodes: &[NodeIndex],
        inputs: &[NodeIndex],
        depth: usize,
    ) -> Fragment<LacunaryRef> {
        Fragment {
            name: self.name.clone(),
            nodes: self
                .nodes
                .iter()
                .map(|n| n.fill_in(nodes, inputs, depth))
                .collect(),
            output: self.output.fill_in(nodes, inputs, depth),
        }
    }

    fn finalize(self) -> Fragment<NodeIndex> {
        Fragment {
            name: self.name,
            nodes: self.nodes.into_iter().map(|n| n.finalize()).collect(),
            // nodes: self.nodes.iter().map(|n| match n {
            //     LacunaryRef::InstanciatedRef(n) => n,
            //     _ => panic!("cannot finalize with remaining hole")
            // }).collect(),
            output: self.output.finalize(),
        }
    }
}

impl Lacunary<Operation<NodeIndex>> for Operation<LacunaryRef> {
    fn fill_in(
        &self,
        indices: &[NodeIndex],
        inputs: &[NodeIndex],
        depth: usize,
    ) -> Operation<LacunaryRef> {
        use Operation::*;

        match self {
            External => External,
            Const(c) => Const(c.fill_in(indices, inputs, depth)),
            Vector(v) => Vector(
                v.iter()
                    .map(|n| n.fill_in(indices, inputs, depth))
                    .collect(),
            ),
            BinaryOp(a, b, op) => BinaryOp(
                a.fill_in(indices, inputs, depth),
                b.fill_in(indices, inputs, depth),
                *op
            ),
            ToString(a) => ToString(a.fill_in(indices, inputs, depth)),
            IfElse(a, b, c) => IfElse(
                a.fill_in(indices, inputs, depth),
                b.fill_in(indices, inputs, depth),
                c.fill_in(indices, inputs, depth),
            ),
            ApplyFragment(f, args) => ApplyFragment(
                f.fill_in(indices, inputs, depth),
                args.iter()
                    .map(|n| n.fill_in(indices, inputs, depth))
                    .collect(),
            )
        }
    }

    fn finalize(self) -> Operation<NodeIndex> {
        use Operation::*;

        match self {
            External => External,
            Const(c) => Const(c.finalize()),
            Vector(v) => Vector(v.iter().map(|n| n.finalize()).collect()),
            BinaryOp(a, b, op) => BinaryOp(a.finalize(), b.finalize(), op),
            ToString(a) => ToString(a.finalize()),
            IfElse(a, b, c) => IfElse(a.finalize(), b.finalize(), c.finalize()),
            ApplyFragment(f, args) => {
                ApplyFragment(f.finalize(), args.iter().map(|n| n.finalize()).collect())
            }
        }
    }
}

impl Lacunary<VarType> for VarType {
    fn fill_in(&self, indices: &[NodeIndex], inputs: &[NodeIndex], depth: usize) -> VarType {
        match self {
            VarType::Null => VarType::Null,
            VarType::Fragment(f) => {
                VarType::Fragment(Rc::new(f.fill_in(indices, inputs, depth + 1)))
            }
            VarType::Bool(b) => VarType::Bool(*b),
            VarType::Int(i) => VarType::Int(*i),
            VarType::Char(c) => VarType::Char(*c),
            VarType::Vector(v) => VarType::Vector(Rc::new(
                v.iter()
                    .map(|n| n.fill_in(indices, inputs, depth))
                    .collect(),
            )),
        }
    }

    fn finalize(self) -> VarType {
        match self {
            VarType::Null => VarType::Null,
            VarType::Fragment(f) => VarType::Fragment(f),
            VarType::Bool(b) => VarType::Bool(b),
            VarType::Int(i) => VarType::Int(i),
            VarType::Char(c) => VarType::Char(c),
            VarType::Vector(v) => {
                VarType::Vector(Rc::new(v.iter().cloned().map(|n| n.finalize()).collect()))
            }
        }
    }
}

impl Lacunary<NodeIndex> for LacunaryRef {
    fn fill_in(&self, nodes: &[NodeIndex], inputs: &[NodeIndex], depth: usize) -> LacunaryRef {
        if self.up() == Some(depth) {
            LacunaryRef::InstanciatedRef(match self {
                LacunaryRef::ContextRef { up: _, index } => nodes[*index],
                LacunaryRef::InputRef { up: _, index } => inputs[*index],
                LacunaryRef::InstanciatedRef(ni) => *ni,
            })
        } else {
            *self
        }
    }

    fn finalize(self) -> NodeIndex {
        match self {
            LacunaryRef::InstanciatedRef(ni) => ni,
            _ => panic!("trying to finalize an incomplete LacunaryRef: {:?}", self),
        }
    }
}
