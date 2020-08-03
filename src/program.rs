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
    Fragment(Rc<Fragment>),
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

    pub fn unpack_fragment(&self) -> Option<&Rc<Fragment>> {
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
    pub fn up(&self) -> usize {
        use ValueRef::*;
        match self {
            InputRef { up, index: _ } => *up,
            ContextRef { up, index: _ } => *up,
            InstanciatedRef(I) => 0,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct FragmentRef {
    pub depth: usize,
    pub index: usize,
}

#[derive(Clone, Debug)]
pub struct Fragment {
    pub nodes: Vec<Operation<ValueRef>>,
    // pub fragments: Vec<Fragment>,
    pub output: ValueRef,
}

// trait NetworkEnv<I:Copy+Debug> {
    
//     /// Allocate a node in the envionment and return a reference to it.
//     fn node_from_operation(op: Operation<I>) -> I;

// }

// impl NetworkEnv<ValueRef> for Fragment {

//     fn node_from_operation(op: Operation<ValueRef>) -> ValueRef {

//     }

// }