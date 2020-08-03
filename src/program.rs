use generational_arena::Index;
use std::iter;
use std::rc::Rc;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct NodeIndex(pub Index);

// #[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
// pub struct FragNodeIndex(pub usize);

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct FragIndex(pub usize);

#[derive(Debug, Clone)]
pub enum VarType {
    Int(i64),
    Bool(bool),
    Char(char),
    Vector(Rc<Vec<VarType>>),
    Fragment(Rc<Fragment<ValueRef>>),
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

    pub fn unpack_fragment(&self) -> Option<&Rc<Fragment<ValueRef>>> {
        if let VarType::Fragment(f) = self {
            Some(f)
        } else {
            None
        }
    }
}

use std::fmt::Debug;
use std::hash::Hash;

// #[derive(PartialEq, Eq, Debug)]
#[derive(Clone, Debug)]
pub enum Operation<I: Clone + Copy + Debug> {
    Const(VarType),
    Vector(Vec<I>),
    Sum(I, I),
    Concat(I, I),
    ToString(I),
    IfElse(I, I, I),
    ApplyFragment(I, Vec<I>),
}

impl<I: Copy + Debug> Operation<I> {
    pub fn dependencies(&self) -> Vec<I> {
        use Operation::*;
        match self {
            Const(_) => Vec::new(),
            Vector(v) => v.clone(),
            Sum(a, b) => vec![*a, *b],
            Concat(a, b) => vec![*a, *b],
            ToString(a) => vec![*a],
            IfElse(a, b, c) => vec![*a, *b, *c],
            ApplyFragment(f, args) => iter::once(*f).chain(args.iter().cloned()).collect(),
        }
    }
}

// pub struct RuntimeModule(pub Box<dyn Fn(Vec<NodeIndex>, &mut RuntimeEnv) -> NodeIndex>);

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum ValueRef {
    InputRef { up: usize, index: usize },
    ContextRef { up: usize, index: usize },
    InstanciatedRef(NodeIndex),
}

impl ValueRef {
    /// How many parent contexts up in which to look for the value being referenced to.
    /// Returns None if the reference has already been fulfilled.
    pub fn up(&self) -> Option<usize> {
        use ValueRef::*;
        match self {
            InputRef { up, index: _ } => Some(*up),
            ContextRef { up, index: _ } => Some(*up),
            InstanciatedRef(I) => None,
        }
    }
}

pub trait Lacunary<F> {

    fn fill_in(&self, nodes: &[NodeIndex], inputs: &[NodeIndex], depth: usize) -> Self;

    fn finalize(self) -> F;

}

#[derive(Clone, Debug)]
pub struct Fragment<I:Copy+Debug> {
    // TODO maybe add an index of stuff to be filled in?
    // Current algorithm is kinda expensive.
    pub nodes: Vec<Operation<I>>,
    pub output: usize,
}

impl Lacunary<Fragment<NodeIndex>> for Fragment<ValueRef> {

    fn fill_in(&self, nodes: &[NodeIndex], inputs: &[NodeIndex], depth: usize) -> Fragment<ValueRef> {
        Fragment {
            nodes: self.nodes.iter().map(|n| n.fill_in(nodes, inputs, depth)).collect(),
            output: self.output
        }
    }

    fn finalize(self) -> Fragment<NodeIndex> {
        Fragment {
            nodes: self.nodes.into_iter().map(|n| n.finalize()).collect(),
            // nodes: self.nodes.iter().map(|n| match n {
            //     ValueRef::InstanciatedRef(n) => n,
            //     _ => panic!("cannot finalize with remaining hole")
            // }).collect(),
            output: self.output
        }
    }

}

impl Lacunary<Operation<NodeIndex>> for Operation<ValueRef> {

    fn fill_in(&self, indices: &[NodeIndex], inputs: &[NodeIndex], depth: usize) -> Operation<ValueRef> {
        use Operation::*;

        match self {
            Const(c) => Const(c.fill_in(indices, inputs, depth)),
            Vector(v) => Vector(
                v.iter()
                    .map(|n| n.fill_in(indices, inputs, depth))
                    .collect(),
            ),
            Sum(a, b) => Sum(
                a.fill_in(indices, inputs, depth),
                b.fill_in(indices, inputs, depth),
            ),
            Concat(a, b) => Concat(
                a.fill_in(indices, inputs, depth),
                b.fill_in(indices, inputs, depth),
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
            Const(c) => Const(c.finalize()),
            Vector(v) => Vector(
                v.iter()
                    .map(|n| n.finalize())
                    .collect(),
            ),
            Sum(a, b) => Sum(
                a.finalize(),
                b.finalize(),
            ),
            Concat(a, b) => Concat(
                a.finalize(),
                b.finalize(),
            ),
            ToString(a) => ToString(a.finalize()),
            IfElse(a, b, c) => IfElse(
                a.finalize(),
                b.finalize(),
                c.finalize(),
            ),
            ApplyFragment(f, args) => ApplyFragment(
                f.finalize(),
                args.iter()
                    .map(|n| n.finalize())
                    .collect(),
            )
        }
    }
}

impl Lacunary<VarType> for VarType {

    fn fill_in(&self, indices: &[NodeIndex], inputs: &[NodeIndex], depth: usize) -> VarType {
        match self {
            VarType::Fragment(f) => {
                let ff = Fragment {
                    nodes: f
                        .nodes
                        .iter()
                        .map(|n| n.fill_in(indices, inputs, depth))
                        .collect(),
                    output: f.output,
                };
                VarType::Fragment(Rc::new(ff.fill_in(indices, inputs, depth+1)))
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
            VarType::Fragment(f) => VarType::Fragment(f),
            VarType::Bool(b) => VarType::Bool(b),
            VarType::Int(i) => VarType::Int(i),
            VarType::Char(c) => VarType::Char(c),
            VarType::Vector(v) => VarType::Vector(Rc::new(
                v.iter().cloned()
                    .map(|n| n.finalize())
                    .collect(),
            )),
        }
    }
}

impl Lacunary<NodeIndex> for ValueRef {

    fn fill_in(&self, nodes: &[NodeIndex], inputs: &[NodeIndex], depth: usize) -> ValueRef {
        if self.up() == Some(depth) {
            ValueRef::InstanciatedRef(match self {
                ValueRef::ContextRef { up:_, index } => nodes[*index],
                ValueRef::InputRef { up:_, index } => inputs[*index],
                ValueRef::InstanciatedRef(ni) => *ni,
            })
        } else {
            *self
        }
    }

    fn finalize(self) -> NodeIndex {
        match self {
            ValueRef::InstanciatedRef(ni) => ni,
            _ => panic!("trying to finalize an incomplete ValueRef")
        }
    }

}