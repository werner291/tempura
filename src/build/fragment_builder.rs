use crate::program::{Fragment, Operation, LacunaryRef, VarType};
use std::collections::HashMap;
use std::rc::Rc;
// use crate::ast::Type;

// #[derive(Eq, PartialEq, Hash, Copy, Clone)]
// pub struct FragmentRef {
//     pub up: usize,
//     pub index: usize,
// }

type Type = ();

struct NodeScaffold {
    operation: Operation<LacunaryRef>,
    node_type: Type
}

pub struct FragmentBuilder<'a> {
    name: String,
    pub values_by_name: HashMap<String, LacunaryRef>,
    values: Vec<NodeScaffold>,
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

    pub fn lookup_value(&self, name: &str) -> Option<LacunaryRef> {
        match self.values_by_name.get(name) {
            Some(idx) => Some(*idx),
            None => match self.parent {
                Some(par) => par.lookup_value(name).map(|vr| match vr {
                    LacunaryRef::ContextRef { up, index } => {
                        LacunaryRef::ContextRef { up: up + 1, index }
                    }
                    LacunaryRef::InputRef { up, index } => LacunaryRef::ContextRef { up: up + 1, index },
                    LacunaryRef::InstanciatedRef(ni) => LacunaryRef::InstanciatedRef(ni),
                }),
                None => None,
            },
        }
    }

    pub fn alloc_value(&mut self, operation: Operation<LacunaryRef>) -> LacunaryRef {

        self.values.push(NodeScaffold{ operation, node_type: ()});

        LacunaryRef::ContextRef {
            up: 0,
            index: self.values.len() - 1,
        }
    }

    pub fn alloc_fragment(&mut self, frag: Fragment<LacunaryRef>) -> LacunaryRef {
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

    pub fn build(self, output: LacunaryRef) -> Fragment<LacunaryRef> {
        Fragment {
            name: self.name,
            nodes: self.values.into_iter().map(|v| v.operation).collect(),
            output,
        }
    }
}
