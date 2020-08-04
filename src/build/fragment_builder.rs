use crate::program::{Fragment, Operation, ValueRef, VarType};
use std::collections::HashMap;
use std::rc::Rc;

// #[derive(Eq, PartialEq, Hash, Copy, Clone)]
// pub struct FragmentRef {
//     pub up: usize,
//     pub index: usize,
// }

pub struct FragmentBuilder<'a> {
    name: String,
    pub values_by_name: HashMap<String, ValueRef>,
    values: Vec<Operation<ValueRef>>,
    parent: Option<&'a FragmentBuilder<'a>>,
}

impl<'a> FragmentBuilder<'a> {
    pub fn new(name: String) -> FragmentBuilder<'static> {
        FragmentBuilder {
            name,
            values_by_name: HashMap::new(),
            values: Vec::new(),
            parent: None,
        }
    }

    pub fn lookup_value(&self, name: &str) -> Option<ValueRef> {
        match self.values_by_name.get(name) {
            Some(idx) => Some(*idx),
            None => match self.parent {
                Some(par) => par.lookup_value(name).map(|vr| match vr {
                    ValueRef::ContextRef { up, index } => ValueRef::ContextRef { up:up+1, index },
                    ValueRef::InputRef { up, index } => ValueRef::ContextRef { up:up+1, index },
                    ValueRef::InstanciatedRef(ni) => ValueRef::InstanciatedRef(ni)
                }),
                None => None,
            },
        }
    }

    pub fn alloc_value(&mut self, value: Operation<ValueRef>) -> ValueRef {
        self.values.push(value);
        ValueRef::ContextRef {
            up: 0,
            index: self.values.len() - 1,
        }
    }

    pub fn alloc_fragment(&mut self, frag: Fragment<ValueRef>) -> ValueRef {
        self.alloc_value(Operation::Const(VarType::Fragment(Rc::new(frag))))
    }

    pub fn derive_child(&'a self, name: String) -> FragmentBuilder<'a> {
        FragmentBuilder {
            name,
            values_by_name: HashMap::new(),
            values: Vec::new(),
            parent: Some(self),
        }
    }

    pub fn build(self, output: ValueRef) -> Fragment<ValueRef> {
        Fragment {
            name: self.name,
            nodes: self.values,
            output: match output {
                ValueRef::ContextRef { up: 0, index } => index,
                _ => panic!("Output must be a same-level contextref."),
            },
        }
    }
}
