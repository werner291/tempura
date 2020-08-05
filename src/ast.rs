use std::iter;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AssignmentAST {
    pub name: Name,
    pub valtype: Option<Type>,
    pub expr: Expression,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct FragmentAST {
    pub name: Name,
    pub inputs: Vec<ModuleInput>,
    pub assignments: Vec<AssignmentAST>,
    pub submodules: Vec<FragmentAST>,
    pub output: Expression,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ModuleInput {
    pub name: Name,
    pub input_type: Type,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Sum,Geq,Leq,Eq,Lt,Gt,Concat,Index,Range
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Expression {
    ConstString(String),
    ConstInteger(i64),
    ConstBoolean(bool),
    BinaryOp(Box<Expression>, Box<Expression>, BinaryOp),
    ModuleApplication {
        mod_name: Name,
        arguments: Vec<Expression>,
    },
    LacunaryRef(Name),
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

impl Expression {
    pub fn collect_dependencies(&self) -> Vec<Dependency> {
        match self {
            Expression::ConstString(_) => vec![],
            Expression::ConstInteger(_) => vec![],
            Expression::ConstBoolean(_) => vec![],
            Expression::BinaryOp(a, b, _) => a
                .collect_dependencies()
                .into_iter()
                .chain(b.collect_dependencies().into_iter())
                .collect(),
            Expression::LacunaryRef(n) => vec![Dependency::Value(n.0.clone())],
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

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TypeDecl(pub Name, pub Type);

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Type {
    PrimInt,
    PrimString,
}

#[derive(Hash, Debug, Eq, PartialEq, Clone)]
pub struct Name(pub String);
