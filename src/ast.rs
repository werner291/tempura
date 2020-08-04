use itertools::Itertools;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::iter;

pub trait TempuraAST {
    fn gen_code(&self) -> String;
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Assignment {
    pub name: Name,
    pub valtype: Option<Type>,
    pub expr: Expression,
}

impl TempuraAST for Assignment {
    fn gen_code(&self) -> String {
        format!("{} = {}", self.name.gen_code(), self.expr.gen_code())
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Module {
    pub name: Name,
    pub inputs: Vec<ModuleInput>,
    pub assignments: Vec<Assignment>,
    pub submodules: Vec<Module>,
    pub output: Expression,
}

impl Module {
    pub fn collect_dependencies(&self) -> Vec<Dependency> {
        let bound_in_context: HashSet<&str> =
            self.inputs.iter().map(|inp| inp.name.0.borrow()).collect();

        self.submodules
            .iter()
            .flat_map(|sm| sm.collect_dependencies())
            .chain(
                self.assignments
                    .iter()
                    .flat_map(|a| a.expr.collect_dependencies()),
            )
            .chain(self.output.collect_dependencies())
            .filter(|dep| !bound_in_context.contains(&dep.get_name()))
            .collect()
    }
}

impl TempuraAST for Module {
    fn gen_code(&self) -> String {
        let args = self
            .inputs
            .iter()
            .map(|mi| {
                format!(
                    "{} : {}",
                    mi.name.gen_code(),
                    match mi.input_type {
                        Type::PrimInt => "int",
                        Type::PrimString => "str",
                    }
                )
            })
            .join(", ");

        let submodules = self.submodules.iter().map(Module::gen_code).join("\n");

        let valuedecls = self.assignments.iter().map(Assignment::gen_code).join("\n");

        let output = self.output.gen_code();

        let declarations = format!("{}\n{}\n{}", submodules, valuedecls, output);

        format!(
            "mod {modname}({args}) {{ \n {declarations} \n }}",
            modname = self.name.gen_code(),
            args = args,
            declarations = declarations
        )
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ModuleInput {
    pub name: Name,
    pub input_type: Type,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Expression {
    ConstString(String),
    ConstInteger(i64),
    ConstBoolean(bool),
    ModuleApplication {
        mod_name: Name,
        arguments: Vec<Expression>,
    },
    Sum {
        a: Box<Expression>,
        b: Box<Expression>,
    },
    Range {
        from: Box<Expression>,
        to: Box<Expression>,
    },
    ValueRef(Name),
    IfElse {
        guard: Box<Expression>,
        body: Box<Expression>,
        else_body: Box<Expression>,
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

impl Expression {
    pub fn collect_dependencies(&self) -> Vec<Dependency> {
        match self {
            Expression::ConstString(_) => vec![],
            Expression::ConstInteger(_) => vec![],
            Expression::ConstBoolean(_) => vec![],
            Expression::ValueRef(n) => vec![Dependency::Value(n.0.clone())],
            Expression::Range { from, to } => from
                .collect_dependencies()
                .into_iter()
                .chain(to.collect_dependencies().into_iter())
                .collect(),
            Expression::Sum { a, b } => a
                .collect_dependencies()
                .into_iter()
                .chain(b.collect_dependencies().into_iter())
                .collect(),
            Expression::ModuleApplication {
                mod_name,
                arguments,
            } => iter::once(Dependency::Module(mod_name.0.clone()))
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

impl TempuraAST for Expression {
    fn gen_code(&self) -> String {
        match self {
            Expression::ConstString(s) => format!("\"{}\"", s),
            Expression::ConstInteger(i) => i.to_string(),
            Expression::ConstBoolean(b) => (if *b { "true" } else { "false" }).to_string(),
            Expression::ValueRef(n) => n.gen_code(),
            Expression::Range { from, to } => format!("{}..{}", from.gen_code(), to.gen_code()),
            Expression::Sum { a, b } => format!("{}..{}", a.gen_code(), b.gen_code()),
            Expression::ModuleApplication {
                mod_name,
                arguments,
            } => format!(
                "{}({})",
                mod_name.gen_code(),
                arguments.iter().map(TempuraAST::gen_code).join(", ")
            ),
            Expression::IfElse {
                guard,
                body,
                else_body,
            } => format!(
                "if {} then {} else {}",
                guard.gen_code(),
                body.gen_code(),
                else_body.gen_code()
            ),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TypeDecl(pub Name, pub Type);

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Type {
    PrimInt,
    PrimString,
}

#[derive(Hash, Debug, Eq, PartialEq, Clone)]
pub struct Name(pub String);

impl TempuraAST for Name {
    fn gen_code(&self) -> String {
        self.0.clone()
    }
}
