use std::fmt::{Debug, Formatter, Error};

#[derive(Debug)]
pub struct IntLit(pub i32);

#[derive(Debug)]
pub struct BoolLit(pub bool);

//#[derive(Debug)]
pub struct StrLit(pub String);

#[derive(Debug)]
pub struct Ident(pub String);

#[derive(Debug)]
pub enum Type {
    Int,
    Bool,
    Str,
    Void,
}

#[derive(Debug)]
pub enum UnaryOp {
    Neg,
    Not
}

#[derive(Debug)]
pub enum BOp {
    Or,
    And,
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Debug)]
pub enum AOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod
}

#[derive(Debug)]
pub enum BinaryOp {
    Boolean(BOp),
    Arithmetic(AOp),
}

#[derive(Debug)]
pub enum Expr {
    Unary(Box<Expr>, UnaryOp),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Application(Box<Ident>, Vec<Box<Expr>>),
    Int(IntLit), // TODO change
}


impl Debug for StrLit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let StrLit(s) = self;
        write!(f, "\"{}\"", s)
    }
}
