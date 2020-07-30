use generational_arena::{Arena, Index};
use std::any::Any;
use std::rc::Rc;

pub struct Node {
    value_cache: Option<Rc<dyn Any>>,
    operation: Element,
    dependents: Vec<Index>,
    being_computed: bool,
}

#[derive(Clone)]
pub enum Element {
    Const(Rc<dyn Any>),
    Vector(Vec<Index>),
    Sum(Index, Index),
    IfElse(Index, Index, Index),
}

impl Element {
    fn dependencies(&self) -> Vec<Index> {
        use Element::*;
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
    pub stdout: Option<Index>,
}

impl RuntimeEnv {
    pub fn node_from_operation(&mut self, operation: Element) -> Index {
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

    fn compute_value(&mut self, idx: Index) -> Rc<dyn Any> {
        use Element::*;

        let mut node = &mut self.nodes[idx];

        if node.being_computed {
            panic!("Circular dependency detected!");
        }

        node.being_computed = true;

        let new_val = match node.operation.clone() {
            Const(v) => v,
            Vector(v) => Rc::new(
                v.iter()
                    .map(|idx_1| self.pull(*idx_1))
                    .collect::<Vec<Rc<dyn Any>>>(),
            ),
            Sum(a, b) => Rc::new(
                self.pull(a).downcast_ref::<i64>().unwrap()
                    + self.pull(b).downcast_ref::<i64>().unwrap(),
            ),
            IfElse(g, b, eb) => {
                if *self.pull(g).downcast_ref::<bool>().unwrap() {
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

    pub fn pull(&mut self, idx: Index) -> Rc<dyn Any> {
        let node = &mut self.nodes[idx];

        match &node.value_cache {
            Some(x) => x.clone(),
            None => self.compute_value(idx),
        }
    }
}
