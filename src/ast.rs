use std::fmt::{Display, Formatter, Error};
use std::convert::TryInto;

#[derive(Debug)]
pub struct Program {
    pub span: Span,
    pub functions:  Vec<FnDef>,
}

#[derive(Debug)]
pub struct FnDef {
    pub span: Span,
    pub type_spec: TypeSpecifier,
    pub ident: Ident,
    pub params: Vec<VarDecl>,
    pub body: Box<StmtNode>,
}

#[derive(Debug)]
pub struct TypeSpecifier {
    pub span: Span,
    pub ttype: Type,
}

#[derive(Debug)]
pub struct VarDecl {
    pub span: Span,
    pub type_spec: TypeSpecifier,
    pub vars: Vec<DeclBody>
}

// Helper struct for parsing, not a part of the ast.
#[derive(Debug)]
pub struct DeclBody {
    pub span: Span,
    pub ident: Ident,
    pub init: Option<Box<ExpNode>>
}

#[derive(Debug,Clone)]
pub enum ExpTypeVal {
    Int(Option<i32>),
    Bool(Option<bool>),
    Str(Option<String>),
    Void,
    Invalid,
}

impl TryInto<Type> for &ExpTypeVal {
    type Error = ();

    fn try_into(self) -> Result<Type, Self::Error> {
        match self {
            ExpTypeVal::Invalid => Err(()),
            ExpTypeVal::Void => Ok(Type::Void),
            ExpTypeVal::Int(_) => Ok(Type::Int),
            ExpTypeVal::Bool(_) => Ok(Type::Bool),
            ExpTypeVal::Str(_) => Ok(Type::Str),
        }
    }
}

impl ExpTypeVal {
    pub fn from_type(ttype: &Type) -> ExpTypeVal {
        match ttype {
            Type::Int     => ExpTypeVal::Int(None),
            Type::Bool    => ExpTypeVal::Bool(None),
            Type::Str     => ExpTypeVal::Str(None),
            Type::Void    => ExpTypeVal::Void,
        }
    }

    pub fn has_type(&self, ttype: &Type) -> bool {
        match (self, ttype) {
            (ExpTypeVal::Int(_), &Type::Int) => true,
            (ExpTypeVal::Bool(_), &Type::Bool) => true,
            (ExpTypeVal::Str(_), &Type::Str) => true,
            (ExpTypeVal::Void, &Type::Void) => true,
            _ => false,
        }
    }

    pub fn has_valid_type(&self) -> bool {
        match self {
            ExpTypeVal::Invalid => false,
            _ => true
        }
    }
}

#[derive(Debug)]
pub struct ExpNode {
    pub exp: Exp,
    pub span: Span,
    pub typeval: Option<ExpTypeVal>,
}

#[derive(Debug)]
pub enum Exp {
    Unary(UnaryOp, Box<ExpNode>),
    Binary(Box<ExpNode>, BinaryOp, Box<ExpNode>),
    Call(Ident, Vec<Box<ExpNode>>),
    Int(i32),
    Bool(bool),
    Str(String),
    Var(Ident),
}

#[derive(Debug)]
pub struct StmtNode {
    pub span: Span,
    pub stmt: Stmt,
    pub will_return: Option<bool> // always returns
}

#[derive(Debug)]
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

pub type FnSignature = (Type, Vec<Type>);

#[derive(Debug,Clone,Copy)]
pub struct Span(pub usize, pub usize);

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Type {
    Int,
    Bool,
    Str,
    Void, // meh
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum UnaryOp {
    Neg,
    Not
}

#[derive(Debug,Clone,Copy,PartialEq)]
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
        Box::new(ExpNode { exp, typeval: None, span: Span(l, r) })
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
        Box::new(StmtNode {span: Span(l, r), stmt, will_return: None})
    }

    pub fn block(stmt: Box<StmtNode>) -> Box<StmtNode> {
        StmtNode::new(stmt.span.0, stmt.span.1, Stmt::BStmt(vec![stmt]))
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

//impl Display for Exp {
//    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
//        match self {
//            Exp::Unary(op, e) => write!(f, "{}{}", op, e),
//            Exp::Binary(l, op, r) => write!(f, "({} {} {})", l, op, r),
//            Exp::Call(ident, args) => {
//                let mut args_text = String::new();
//                for arg in args.iter() {
//                    args_text.push_str(&(**arg).to_string());
//                    args_text.push_str(", ");
//                }
//                let args_text = &args_text[0..args_text.len()-2];
//                write!(f, "{}({})", ident, args_text)
//            }
//            Exp::Int(v) => write!(f, "{}", v),
//            Exp::Bool(v) => write!(f, "b{}", v),
//            Exp::Str(v) => write!(f, "{}", v),
//            Exp::Var(v) => write!(f, "{}", v),
//        }
//    }
//}

//impl Type {
//
//}

impl Display for ExpTypeVal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ExpTypeVal::Int(_) => write!(f, "int"),
            ExpTypeVal::Bool(_) => write!(f, "boolean"),
            ExpTypeVal::Str(_) => write!(f, "string"),
            ExpTypeVal::Void => write!(f, "void"),
            ExpTypeVal::Invalid => write!(f, "<invalid>"),
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
        }
    }
}

//impl Debug for Type {
//    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
//        self.fmt(f)
//    }
//}

//impl Display for ExpNode {
//    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
//        match self.node {
//            ExpTypeVal::Int => write!(f, "int"),
//            ExpTypeVal::Bool => write!(f, "boolean"),
//            ExpTypeVal::Str => write!(f, "string"),
//            ExpTypeVal::Void => write!(f, "void"),
//        }
//    }
//}
