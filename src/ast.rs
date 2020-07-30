#[derive(Debug, Eq, PartialEq)]
pub struct Assignment<'a> {
    pub name: Name<'a>,
    pub args: Vec<Name<'a>>,
    pub valtype: Option<Type>,
    pub expr: Expression<'a>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TempuraAST<'a> {
    pub assignments: Vec<Assignment<'a>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Expression<'a> {
    ConstString(String),
    ConstInteger(i64),
    FunctionApplication {
        function: Box<Expression<'a>>,
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

impl<'a> Expression<'a> {
    pub fn collect_dependencies(&self) -> Vec<&'a str> {
        match self {
            Expression::ConstString(_) => vec![],
            Expression::ConstInteger(_) => vec![],
            Expression::ValueRef(Name(n)) => vec![n],
            Expression::Range { from, to } => from
                .collect_dependencies()
                .into_iter()
                .chain(to.collect_dependencies().into_iter())
                .collect(),
            Expression::FunctionApplication {function,arguments} => 
                function.collect_dependencies().into_iter().chain(
                    arguments.iter().flat_map(|arg| arg.collect_dependencies().into_iter())
                ).collect(),
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
    Function {
        arg_types: Vec<Type>,
        result_type: Box<Type>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct Name<'a>(pub &'a str);
