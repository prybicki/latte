use std::fmt::{Display, Formatter, Error};

pub struct Program {
    pub span: Span,
    pub functions:  Vec<FnDef>,
}

pub struct FnDef {
    pub span: Span,
    pub type_spec: TypeSpecifier,
    pub ident: Ident,
    pub params: Vec<VarDecls>,
    pub block: Block,
}

pub struct TypeSpecifier {
    pub span: Span,
    pub ttype: Type,
}

pub struct VarDecls {
    pub span: Span,
    pub type_spec: TypeSpecifier,
    pub vars: Vec<DeclBody>
}

// Helper struct for parsing, not a part of the ast.
pub struct DeclBody {
    pub span: Span,
    pub ident: Ident,
    pub init: Option<Box<Exp>>
}

pub struct Block {
    pub span: Span,
    pub stmts: Vec<Box<Stmt>>,
}

pub struct Exp {
    pub exp: ExpData,
    pub ttype: Option<Type>,
    pub span: Span,
}

pub enum ExpData {
    Unary(UnaryOp, Box<Exp>),
    Binary(Box<Exp>, BinaryOp, Box<Exp>),
    Call(Ident, Vec<Box<Exp>>),
    Int(i32),
    Bool(bool),
    Str(String),
    Var(Ident),
}

pub struct Stmt {
    pub span: Span,
    pub stmt: StmtData,
}

pub enum StmtData {
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

// *** *** *** Minors *** *** *** //

pub type Ident = String;

#[derive(Clone,Copy)]
pub struct Span(pub usize, pub usize);

#[derive(Clone,Copy,PartialEq)]
pub enum Type {
    Int,
    Bool,
    Str,
    Void,
    Invalid,
}

#[derive(Clone,Copy,PartialEq)]
pub enum UnaryOp {
    Neg,
    Not
}

#[derive(Clone,Copy,PartialEq)]
pub enum BinaryOp {
    Or, And,
    Eq, Neq, Gt, Gte, Lt, Lte,
    Add, Sub, Mul, Div, Mod
}

// *** *** *** Impls *** *** *** //

impl TypeSpecifier {
    pub fn new(ttype: Type, l: usize, r: usize) -> TypeSpecifier {
        TypeSpecifier { span: Span(l, r), ttype}
    }
}

impl Exp {
    pub fn new(l: usize, r: usize, exp: ExpData) -> Box<Exp> {
        Box::new(Exp { exp: exp, ttype: None, span: Span(l, r) })
    }

    pub fn new_un(l: usize, r: usize, op: Op, exp: Box<Exp>) -> Box<Exp> {
        Self::new(l, r, ExpData::Unary(op, exp))
    }

    pub fn new_bin(l: usize, r:usize, op: Op, lexp: Box<Exp>, rexp: Box<Exp>) -> Box<Exp> {
        Self::new(l, r, ExpData::Binary(lexp, op, rexp))
    }
}

impl Stmt {
    pub fn new(l: usize, r: usize, stmt: StmtData) -> Box<Stmt> {
        Box::new(Stmt {span: Span(l, r), stmt})
    }
}

impl Block {
    pub fn new(l: usize, r: usize, stmts: Vec<Box<Stmt>>) -> Block {
        Block {span: Span(l, r), stmts}
    }
}

impl Type {
    pub fn is_valid(&self) -> bool {
        *self != Type::Invalid && *self != Type::Unknown
    }
}

impl FnDef {
    pub fn get_signature(&self) -> (Type, Vec<Type>) {
        (self.type_spec.ttype, self.params.iter().map(|VarDecls {type_spec: ts, ..}| ts.ttype).collect())
    }
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

impl Display for ExpData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ExpData::Unary(op, e) => write!(f, "{}{}", op, e),
            ExpData::Binary(l, op, r) => write!(f, "({} {} {})", l, op, r),
            ExpData::Call(ident, args) => {
                let mut args_text = String::new();
                for arg in args.iter() {
                    args_text.push_str(&(**arg).to_string());
                    args_text.push_str(", ");
                }
                let args_text = &args_text[0..args_text.len()-2];
                write!(f, "{}({})", ident, args_text)
            }
            ExpData::Int(v) => write!(f, "{}", v),
            ExpData::Bool(v) => write!(f, "b{}", v),
            ExpData::Str(v) => write!(f, "{}", v),
            ExpData::Var(v) => write!(f, "{}", v),
        }
    }
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

impl Display for Exp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        return self.exp.fmt(f);
    }
}
