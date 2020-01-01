use std::fmt::{Display, Formatter, Error, Debug};

pub struct Program {
    pub span: Span,
    pub functions:  Vec<FnDef>,
}

pub struct FnDef {
    pub span: Span,
    pub type_spec: TypeSpecifier,
    pub ident: Ident,
    pub params: Vec<VarDecl>,
    pub body: Box<StmtNode>,
}

pub struct TypeSpecifier {
    pub span: Span,
    pub ttype: Type,
}

pub struct VarDecl {
    pub span: Span,
    pub type_spec: TypeSpecifier,
    pub vars: Vec<DeclBody>
}

// Helper struct for parsing, not a part of the ast.
pub struct DeclBody {
    pub span: Span,
    pub ident: Ident,
    pub init: Option<Box<ExpNode>>
}

pub struct ExpNode {
    pub exp: Exp,
    pub ttype: Option<Type>,
    pub span: Span,
//    pub optimization: bool
}

pub enum Exp {
    Unary(UnaryOp, Box<ExpNode>),
    Binary(Box<ExpNode>, BinaryOp, Box<ExpNode>),
    Call(Ident, Vec<Box<ExpNode>>),
    Int(i32),
    Bool(bool),
    Str(String),
    Var(Ident),
}

pub struct StmtNode {
    pub span: Span,
    pub stmt: Stmt,
//    pub ret: bool // always returns
}

pub enum Stmt {
    BStmt(Vec<Box<StmtNode>>),
    Decl(VarDecl),
    Ass(Ident, Box<ExpNode>),
    Incr(Ident),
    Decr(Ident),
    Ret(Box<ExpNode>),
    VRet,
    Cond(Box<ExpNode>, Box<StmtNode>, Option<Box<StmtNode>>),
    While(Box<ExpNode>, Box<StmtNode>),
    EStmt(Box<ExpNode>),
}

// *** *** *** Minors *** *** *** //

pub type Ident = String;

type FnSignature = (Type, Vec<Type>);

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
    pub fn new(l: usize, r: usize, ttype: Type) -> TypeSpecifier {
        TypeSpecifier { span: Span(l, r), ttype}
    }
}

impl ExpNode {
    pub fn new(l: usize, r: usize, exp: Exp) -> Box<ExpNode> {
        Box::new(ExpNode { exp, ttype: None, span: Span(l, r) })
    }

    pub fn new_un(l: usize, r: usize, op: UnaryOp, exp: Box<ExpNode>) -> Box<ExpNode> {
        Self::new(l, r, Exp::Unary(op, exp))
    }

    pub fn new_bin(l: usize, r:usize, op: BinaryOp, lexp: Box<ExpNode>, rexp: Box<ExpNode>) -> Box<ExpNode> {
        Self::new(l, r, Exp::Binary(lexp, op, rexp))
    }
}

impl StmtNode {
    pub fn new(l: usize, r: usize, stmt: Stmt) -> Box<StmtNode> {
        Box::new(StmtNode {span: Span(l, r), stmt})
    }
}

pub fn is_valid(ttype: &Option<Type>) -> bool {
    match ttype {
        Some(t) if t != &Type::Invalid => true,
        _ => false
    }
}

impl FnDef {
    pub fn get_signature(&self) -> FnSignature {
        (self.type_spec.ttype, self.params.iter().map(|VarDecl {type_spec: ts, ..}| ts.ttype).collect())
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

impl Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Type::Int => write!(f, "int"),
            Type::Bool => write!(f, "boolean"),
            Type::Str => write!(f, "string"),
            Type::Void => write!(f, "void"),
            Type::Invalid => write!(f, "<invalid>")
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.fmt(f)
    }
}

impl Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.fmt(f)
    }
}

impl Display for ExpNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        return self.exp.fmt(f);
    }
}
