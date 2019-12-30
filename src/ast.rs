use std::fmt::{Display, Formatter, Error};

pub type Ident = String;

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Type {
    Int,
    Bool,
    Str,
    Void,
    Unknown,
    Invalid,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Type::Int => write!(f, "int"),
            Type::Bool => write!(f, "boolean"),
            Type::Str => write!(f, "string"),
            Type::Void => write!(f, "void"),
            _ => write!(f, "<invalid/unknown>")
        }
    }
}

impl Type {
    pub fn is_valid(&self) -> bool {
        *self != Type::Invalid && *self != Type::Unknown
    }
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum UnaryOp {
    Neg,
    Not
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum BinaryOp {
    Or,
    And,
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Add,
    Sub,
    Mul,
    Div,
    Mod
}

#[derive(Debug)]
pub enum Exp {
    Unary(UnaryOp, Box<TypedExp>),
    Binary(Box<TypedExp>, BinaryOp, Box<TypedExp>),
    Call(Ident, Vec<Box<TypedExp>>),
    Int(i32),
    Bool(bool),
    Str(String),
    Var(Ident),
}

#[derive(Debug)]
pub struct TypedExp {
    pub exp: Exp,
    pub etype: Type
}

impl Exp {
    pub fn new(exp: Exp) -> Box<TypedExp> {
        Box::new(TypedExp{ exp: exp, etype: Type::Unknown})
    }
}

#[derive(Debug)]
pub struct VarDecl(pub Type, pub Ident, pub Option<Box<TypedExp>>);

#[derive(Debug)]
pub struct Block(pub Vec<Box<Stmt>>);

#[derive(Debug)]
pub struct Program(pub Vec<FnDef>);

#[derive(Debug)]
pub struct FnDef(pub Type, pub Ident, pub Vec<VarDecl>, pub Block);

impl FnDef {
    pub fn get_signature(&self) -> (Type, Vec<Type>) {
        let FnDef(ttype, _, params, _) = self;
        (*ttype, params.iter().map(|VarDecl(t, _, _)| *t).collect())
    }

}

#[derive(Debug)]
pub enum Stmt {
    BStmt(Block),
    Decl(Vec<VarDecl>),
    Ass(Ident, Box<TypedExp>),
    Incr(Ident),
    Decr(Ident),
    Ret(Box<TypedExp>),
    VRet,
    Cond(Box<TypedExp>, Block, Option<Block>),
    While(Box<TypedExp>, Block),
    EStmt(Box<TypedExp>),
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let ch = match self {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!"
        };
        write!(f, "{}", ch)
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let ch = match self {
            BinaryOp::Or => "||",
            BinaryOp::And => "&&",
            BinaryOp::Eq => "==",
            BinaryOp::Neq => "!=",
            BinaryOp::Gt => ">",
            BinaryOp::Gte => ">=",
            BinaryOp::Lt => "<",
            BinaryOp::Lte => "<=",
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
        };
        write!(f, "{}", ch)
    }
}

impl Display for Exp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Exp::Unary(op, e) => write!(f, "{}{}", op, e),
            Exp::Binary(l, op, r) => write!(f, "({} {} {})", l, op, r),
            Exp::Call(ident, args) => {
                let mut args_text = String::new();
                for arg in args.iter() {
                    args_text.push_str(&(**arg).to_string());
                    args_text.push_str(", ");
                }
                let args_text = &args_text[0..args_text.len()-2];
                write!(f, "{}({})", ident, args_text)
            }
            Exp::Int(v) => write!(f, "{}", v),
            Exp::Bool(v) => write!(f, "b{}", v),
            Exp::Str(v) => write!(f, "{}", v),
            Exp::Var(v) => write!(f, "{}", v),
        }
    }
}

impl Display for TypedExp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        return self.exp.fmt(f);
    }
}

// Helper struct for parsing, not a part of the ast.
pub struct DeclBody(pub Ident, pub Option<Box<TypedExp>>);