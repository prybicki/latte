use std::fmt::{Display, Formatter, Error};

type Ident = String;

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Type {
    Int,
    Bool,
    Str,
    Void,
    Unknown, // TODO use Option
    Invalid,
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
pub enum ExpRaw {
    Unary(UnaryOp, Box<Exp>),
    Binary(Box<Exp>, BinaryOp, Box<Exp>),
    Call(Ident, Vec<Box<Exp>>),
    Int(i32),
    Bool(bool),
    Str(String),
    Var(String),
}

#[derive(Debug)]
pub struct Exp {
    raw: ExpRaw,
    typ: Type,
}

impl Exp {
    pub fn new(raw: ExpRaw) -> Box<Exp> {
        Box::new(Exp {raw, typ: Type::Unknown})
    }
}

#[derive(Debug)]
pub struct VarDecl(pub Type, pub Ident, pub Option<Box<Exp>>);

#[derive(Debug)]
pub struct Block(pub Vec<Box<Stmt>>);

#[derive(Debug)]
pub struct Program(pub Vec<FnDef>);

#[derive(Debug)]
pub struct FnDef(pub Type, pub String, pub Vec<VarDecl>, pub Block);

#[derive(Debug)]
pub enum Stmt {
    BStmt(Block),
    Decl(Vec<VarDecl>),
    Ass(Ident, Box<Exp>),
    Incr(Ident),
    Decr(Ident),
    Ret(Box<Exp>),
    VRet,
    Cond(Box<Exp>, Block, Option<Block>),
    While(Box<Exp>, Block),
    EStmt(Box<Exp>),
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

impl Display for ExpRaw {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ExpRaw::Unary(op, e) => write!(f, "{}{}", op, e),
            ExpRaw::Binary(l, op, r) => write!(f, "({} {} {})", l, op, r),
            ExpRaw::Call(ident, args) => {
                let mut args_text = String::new();
                for arg in args.iter() {
                    args_text.push_str(&(**arg).to_string());
                    args_text.push_str(", ");
                }
                let args_text = &args_text[0..args_text.len()-2];
                write!(f, "{}({})", ident, args_text)
            }
            ExpRaw::Int(v) => write!(f, "{}", v),
            ExpRaw::Bool(v) => write!(f, "b{}", v),
            ExpRaw::Str(v) => write!(f, "{}", v),
            ExpRaw::Var(v) => write!(f, "{}", v),
        }
    }
}

impl Display for Exp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let Exp{raw: raw, ..} = self;
        return raw.fmt(f);
    }
}

// Helper struct for parsing, not a part of the ast.
pub struct DeclBody(pub Ident, pub Option<Box<Exp>>);