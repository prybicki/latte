use std::fmt::{Display, Formatter, Error};

type Ident = String;

#[derive(Debug,Clone,Copy)]
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
    Call(Ident, Vec<Box<Expr>>),
    Int(i32),
    Bool(bool),
    Str(String),
    Var(String),
}

#[derive(Debug)]
pub struct VarDecl(pub Type, pub Ident, pub Option<Box<Expr>>);

#[derive(Debug)]
pub struct Block(pub Vec<Box<Stmt>>);

#[derive(Debug)]
pub struct Program(pub Vec<TopDef>);


#[derive(Debug)]
pub enum Stmt {
    BStmt(Block),
    Decl(Vec<VarDecl>),
    Ass(Ident, Box<Expr>),
    Incr(Ident),
    Decr(Ident),
    Ret(Box<Expr>),
    VRet,
    Cond(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    While(Box<Expr>, Box<Stmt>),
    EStmt(Box<Expr>),
}

// Helper struct
pub struct DeclBody(pub Ident, pub Option<Box<Expr>>);

#[derive(Debug)]
pub enum TopDef {
    FnDef(Type, String, Vec<VarDecl>, Block)
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
            Expr::Bool(v) => write!(f, "b{}", v),
            Expr::Str(v) => write!(f, "{}", v),
            Expr::Var(v) => write!(f, "{}", v),
        }
    }
}

//impl Display for Expr