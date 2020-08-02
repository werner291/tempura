
use std::rc::Rc;
use std::iter;

#[derive(Debug, Clone)]
pub enum VarType {
    Int(i64),
    Bool(bool),
    Char(char),
    Vector(Rc<Vec<VarType>>),
    Fragment(Rc<Fragment>)
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

    pub fn unpack_vector(&self) -> Option<&Vec<VarType>> {
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
                    None => return None,
                }
            }
            Some(result)
        })
    }


    pub fn unpack_fragment(&self) -> Option<&Rc<Fragment>> {
        if let VarType::Fragment(f) = self {
            Some(f)
        } else {
            None
        }
    }

    
}

use std::hash::Hash;
use std::fmt::Debug;

// #[derive(PartialEq, Eq, Debug)]
#[derive(Clone, Debug)]
pub enum Operation<I : Clone + Copy + Debug> {
    Const(VarType),
    Vector(Vec<I>),
    Sum(I, I),
    IfElse(I, I, I),
    ApplyFragment(I, Vec<I>)
}

impl<I : Copy + Debug> Operation<I> {
    pub fn dependencies(&self) -> Vec<I> {
        use Operation::*;
        match self {
            Const(_) => Vec::new(),
            Vector(v) => v.clone(),
            Sum(a, b) => vec![*a, *b],
            IfElse(a, b, c) => vec![*a, *b, *c],
            ApplyFragment(f, args) => iter::once(*f).chain(args.iter().cloned()).collect()
        }
    }

    pub fn map_ref<O : Copy + Debug, Ft: Fn(&I) -> O>(&self, mfn : Ft) -> Operation<O> {
        use Operation::*;
        match self {
            Const(c) => Const(c.clone()),
            Vector(v) => Vector(v.iter().map(mfn).collect()),
            Sum(a, b) => Sum(mfn(a),mfn(b)),
            IfElse(a, b, c) => IfElse(mfn(a), mfn(b), mfn(c)),
            ApplyFragment(f,args) => ApplyFragment(mfn(f), args.iter().map(mfn).collect()),
            // ApplyModule(_m,args) => args.clone(),
        }
    }
}

// pub struct RuntimeModule(pub Box<dyn Fn(Vec<NodeIndex>, &mut RuntimeEnv) -> NodeIndex>);



#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum ValueRef {
    InputRef { depth: usize, index: usize },
    ContextRef { depth: usize, index: usize }
}

impl ValueRef {
    pub fn depth(&self) -> usize {
        match self {
            ValueRef::InputRef {depth,index:_} => *depth,
            ValueRef::ContextRef {depth,index:_} => *depth
        }
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct FragmentRef {
    pub depth: usize,
    pub index: usize
}

#[derive(Clone, Debug)]
pub struct Fragment {
    pub nodes: Vec<Operation<ValueRef>>,
    // pub fragments: Vec<Fragment>,
    pub output: ValueRef
}

