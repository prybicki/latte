use std::fmt::{Display, Debug, Formatter, Error};
use std::io::Read;

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
pub enum Expr {
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Call(String, Vec<Box<Expr>>),
    Int(i32),
    Bool(bool),
    Str(String),
    Var(String),
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

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Expr::Unary(op, e) => write!(f, "{}{}", op, e),
            Expr::Binary(l, op, r) => write!(f, "({} {} {})", l, op, r),
            Expr::Call(ident, args) => {
                let mut args_text = String::new();
                for arg in args.iter() {
                    args_text.push_str(&(**arg).to_string());
                    args_text.push_str(", ");
                }
                let args_text = &args_text[0..args_text.len()-2];
                write!(f, "{}({})", ident, args_text)
            }
            Expr::Int(v) => write!(f, "{}", v),
            Expr::Bool(v) => write!(f, "{}", v),
            Expr::Str(v) => write!(f, "{}", v),
            Expr::Var(v) => write!(f, "{}", v),
        }
    }
}

//impl Display for Expr