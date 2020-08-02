use std::collections::HashSet;
use std::iter;

// #[derive(Debug, Eq, PartialEq)]
// pub struct TempuraAST<'a> {
//     modules: Vec<Module<'a>>
// }

#[derive(Debug, Eq, PartialEq)]
pub struct Assignment<'a> {
    pub name: Name<'a>,
    pub valtype: Option<Type>,
    pub expr: Expression<'a>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Module<'a> {
    pub name: Name<'a>,
    pub inputs: Vec<ModuleInput<'a>>,
    pub assignments: Vec<Assignment<'a>>,
    pub submodules: Vec<Module<'a>>,
    pub output: Expression<'a>,
}

impl<'a> Module<'a> {
    pub fn collect_dependencies(&'a self) -> Vec<Dependency> {
        let bound_in_context: HashSet<&str> = self.inputs.iter().map(|inp| inp.name.0).collect();

        self.submodules
            .iter()
            .flat_map(|sm| sm.collect_dependencies())
            .chain(
                self.assignments
                    .iter()
                    .flat_map(|a| a.expr.collect_dependencies()),
            )
            .filter(|dep| !bound_in_context.contains(&dep.get_name()))
            .collect()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ModuleInput<'a> {
    pub name: Name<'a>,
    pub input_type: Type,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Expression<'a> {
    ConstString(String),
    ConstInteger(i64),
    ModuleApplication {
        mod_name: Name<'a>,
        arguments: Vec<Expression<'a>>,
    },
    Range {
        from: Box<Expression<'a>>,
        to: Box<Expression<'a>>,
    },
    ValueRef(Name<'a>),
    IfElse {
        guard: Box<Expression<'a>>,
        body: Box<Expression<'a>>,
        else_body: Box<Expression<'a>>,
    },
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Dependency {
    Value(String),
    Module(String),
}

impl Dependency {
    fn get_name(&self) -> &str {
        match self {
            Dependency::Value(n) => &n,
            Dependency::Module(n) => &n,
        }
    }
}

impl<'a> Expression<'a> {
    pub fn collect_dependencies(&self) -> Vec<Dependency> {
        match self {
            Expression::ConstString(_) => vec![],
            Expression::ConstInteger(_) => vec![],
            Expression::ValueRef(n) => vec![Dependency::Value(n.0.to_string())],
            Expression::Range { from, to } => from
                .collect_dependencies()
                .into_iter()
                .chain(to.collect_dependencies().into_iter())
                .collect(),
            Expression::ModuleApplication {
                mod_name,
                arguments,
            } => iter::once(Dependency::Module(mod_name.0.to_string()))
                .chain(
                    arguments
                        .iter()
                        .flat_map(|arg| arg.collect_dependencies().into_iter()),
                )
                .collect(),
            Expression::IfElse {
                guard,
                body,
                else_body,
            } => guard
                .collect_dependencies()
                .into_iter()
                .chain(body.collect_dependencies().into_iter())
                .chain(else_body.collect_dependencies().into_iter())
                .collect(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TypeDecl<'a>(pub Name<'a>, pub Type);

#[derive(Debug, Eq, PartialEq)]
pub enum Type {
    PrimInt,
    PrimString,
}

#[derive(Hash, Debug, Eq, PartialEq, Copy, Clone)]
pub struct Name<'a>(pub &'a str);
