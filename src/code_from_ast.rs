use crate::ast::*;

pub trait TempuraAST {
    fn gen_code(&self) -> String;
}

impl TempuraAST for AssignmentAST {
    fn gen_code(&self) -> String {
        format!("{} = {}", self.name.gen_code(), self.expr.gen_code())
    }
}

impl TempuraAST for FragmentAST {
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

        let submodules = self.submodules.iter().map(FragmentAST::gen_code).join("\n");

        let valuedecls = self.assignments.iter().map(AssignmentAST::gen_code).join("\n");

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

impl TempuraAST for Expression {
    fn gen_code(&self) -> String {
        match self {
            Expression::ConstString(s) => format!("\"{}\"", s),
            Expression::ConstInteger(i) => i.to_string(),
            Expression::ConstBoolean(b) => (if *b { "true" } else { "false" }).to_string(),
            Expression::LacunaryRef(n) => n.gen_code(),
            Expression::ContainerIndexing(c, i) => {
                format!("\"{}\"[{}]", c.gen_code(), i.gen_code())
            }
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

impl TempuraAST for Name {
    fn gen_code(&self) -> String {
        self.0.clone()
    }
}