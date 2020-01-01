use crate::ast::*;
use crate::diag;
use std::collections::HashMap;

pub fn remove_comments(text: &str) -> String {
    #[derive(Debug)]
    enum PrimaryState {
        InCode,
        AfterForwardSlash,
        InSingleLineComment,
        InMultiLineComment,
        InMultiLineAfterAsterisk,
    };
    #[derive(Debug)]
    enum SecondaryState {
        NotInString,
        InString,
        InStringAfterEscape,
    };
    let mut s1 = PrimaryState::InCode;
    let mut s2 = SecondaryState::NotInString;
    let mut output = String::new();
    for ch in text.chars() {
        match (&s2, ch) {
            (SecondaryState::NotInString, '"')      => s2 = SecondaryState::InString,
            (SecondaryState::InString, '\\') => s2 = SecondaryState::InStringAfterEscape,
            (SecondaryState::InStringAfterEscape, _) => s2 = SecondaryState::InString,
            (SecondaryState::InString, '"' ) => s2 = SecondaryState::NotInString,
            (_, _) => ()
        };

        if let SecondaryState::NotInString = s2 {
            match (&s1, ch) {
                (PrimaryState::InCode, '#') => s1 = PrimaryState::InSingleLineComment,
                (PrimaryState::InCode, '/') => s1 = PrimaryState::AfterForwardSlash,
                (PrimaryState::AfterForwardSlash, '/') => s1 = PrimaryState::InSingleLineComment,
                (PrimaryState::AfterForwardSlash, '*') => s1 = PrimaryState::InMultiLineComment,
                (PrimaryState::AfterForwardSlash, _) => {
                    s1 = PrimaryState::InCode;
                    output.push('/');
                }
                (PrimaryState::InSingleLineComment, '\n') => s1 = PrimaryState::InCode,
                (PrimaryState::InMultiLineComment, '*') => s1 = PrimaryState::InMultiLineAfterAsterisk,
                (PrimaryState::InMultiLineAfterAsterisk, '/') => {
                    s1 = PrimaryState::InCode;
                    continue;
                },
                (PrimaryState::InMultiLineAfterAsterisk, _) => s1 = PrimaryState::InMultiLineComment,
                (_,_) => (),
            }
        }
        output.push(match s1 {
            PrimaryState::InCode => ch,
            _ => if ch == '\n' { '\n' } else { ' ' },
        });
    }
    return output;
}

// Bad things:
// uninitialized variable
// repeated definition
// undeclared variable
// undeclared function
// dead-code return
// no return
// TODO declared variable must not be void

// first pass -> expression types
// second pass -> valid expression types in statements

// mismatched types (expressions, variables, returns, parameters)

// TODO:
// no return (especially dead return: if(false) return) / return only in some cases

struct ScopedMap<K,V> {
    scopes: Vec<HashMap<K,V>>,
}

impl<K,V> ScopedMap<K,V>
    where K: std::cmp::Eq + std::hash::Hash
{
    fn new() -> ScopedMap<K,V> {
        ScopedMap {scopes: Vec::new()}
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::<K, V>::new())
    }

    fn pop_scope(&mut self) -> Option<HashMap<K,V>> {
        self.scopes.pop()
    }

    fn get(&self, key: &K) -> Option<&V> {
        for scope in self.scopes.iter().rev() {
            let x = scope.get(key);
            if x.is_some() {
                return x
            }
        }
        return None
    }

    fn insert(&mut self, key: K, val: V) -> Option<V> {
        if self.scopes.last().is_none() {
            self.push_scope();
        }
        let top_scope = self.scopes.last_mut().unwrap();
        return top_scope.insert(key, val);
    }
}

type Env<'a> = ScopedMap<Ident, Type>;
type FEnv<'a> = HashMap<Ident, (Type, Vec<Type>)>;
type Diags = Vec<diag::Diagnostic>;

fn get_unary_op_type(op: &UnaryOp, exp_type: &Option<Type>) -> Option<Type> {
    Some(match exp_type {
        Some(exp_type) => {
            match (op, exp_type) {
                (UnaryOp::Neg, Type::Int) =>  Type::Int,
                (UnaryOp::Not, Type::Bool) => Type::Bool,
                _ => Type::Invalid
            }
        },
        _ => Type::Invalid,
    })
}

fn get_binary_op_type(op: &BinaryOp, lexp_type: &Option<Type>, rexp_type: &Option<Type>) -> Option<Type> {
    Some(match (lexp_type, rexp_type) {
        (Some(lexp_type), Some(rexp_type)) => {
            match (op, lexp_type, rexp_type) {
                (BinaryOp::Eq,  l, r) => if l == r { Type::Bool } else { Type::Invalid },
                (BinaryOp::Neq, l, r) => if l == r { Type::Bool } else { Type::Invalid },
                (BinaryOp::Or,  Type::Bool, Type::Bool) => Type::Bool,
                (BinaryOp::And, Type::Bool, Type::Bool) => Type::Bool,
                (BinaryOp::Gt,  Type::Int, Type::Int) => Type::Bool,
                (BinaryOp::Gte, Type::Int, Type::Int) => Type::Bool,
                (BinaryOp::Lt,  Type::Int, Type::Int) => Type::Bool,
                (BinaryOp::Lte, Type::Int, Type::Int) => Type::Bool,
                (BinaryOp::Add, Type::Int, Type::Int) => Type::Int,
                (BinaryOp::Add, Type::Str, Type::Str) => Type::Str,
                (BinaryOp::Sub, Type::Int, Type::Int) => Type::Int,
                (BinaryOp::Mul, Type::Int, Type::Int) => Type::Int,
                (BinaryOp::Div, Type::Int, Type::Int) => Type::Int,
                (BinaryOp::Mod, Type::Int, Type::Int) => Type::Int,
                _ => Type::Invalid
            }
        },
        _ => Type::Invalid,
    })
}

fn verify_exp(exp_node: &mut ExpNode, fenv: &FEnv, env: &Env, diags: &mut Diags) {
    exp_node.ttype = match &mut exp_node.exp {
        Exp::Unary(op, inner) => {
            verify_exp(inner, fenv, env, diags);
            let this_type = get_unary_op_type(op, &inner.ttype);
            if is_valid(&inner.ttype) && !is_valid(&this_type) {
                diags.push(diag::Diagnostic{
                    message: format!("invalid expression type for operand {}", op),
                    details: Some((inner.span, format!("expression has type {}", inner.ttype.unwrap())))
                });
            }
            this_type
        },
        Exp::Binary(lexp_node, op, rexp_node) => {
            verify_exp(lexp_node, fenv, env, diags);
            verify_exp(rexp_node, fenv, env, diags);
            let this_type = get_binary_op_type(op, &lexp_node.ttype, &rexp_node.ttype);
            if is_valid(&lexp_node.ttype) && is_valid(&rexp_node.ttype) && !is_valid(&this_type) {
                diags.push(diag::Diagnostic{
                    message: format!("invalid expressions types for operand {}", op),
                    details: Some((exp_node.span, format!("expression has type {} {} {}", lexp_node.ttype.unwrap(), op, rexp_node.ttype.unwrap())))
                });
            }
            this_type
        },
        Exp::Call(ident, args) => {
            match fenv.get(ident.as_str()) {
                None => {
                    diags.push(diag::Diagnostic{
                        message: format!("unknown function identifier {}", ident),
                        details: Some((exp_node.span, format!("in this expression")))
                    });
                    Some(Type::Invalid)
                },
                Some((ret_type, param_types)) => {
                    let mut arg_types = Vec::new();
                    for exp_node in args {
                        verify_exp(exp_node, fenv, env, diags);
                        arg_types.push(exp_node.ttype);
                    }

                    // if any of arguments has invalid type, it was already reported
                    if let Some(arg_types) = arg_types.into_iter().collect::<Option<Vec<Type>>>() {
                        // all arguments has valid type, check if it matches with the signature
                        if param_types.ne(&arg_types) {
                            diags.push(diag::Diagnostic {
                                message: format!("invalid argument types"),
                                details: Some((exp_node.span, format!("expected {f}({p:?}), got {f}({a:?})", f=ident, p=param_types, a=arg_types)))
                            })
                        }
                    }
                    Some(*ret_type)
                }
            }
        },
        Exp::Var(ident) => {
            match env.get(&ident) {
                None => {
                    diags.push(diag::Diagnostic {
                        message: format!("undeclared variable {}", ident),
                        details: Some((exp_node.span, format!("in this expression")))
                    });
                    Some(Type::Invalid)
                } ,
                Some(ttype) => Some(*ttype),
            }
        },
        Exp::Int(_)  => Some(Type::Int),
        Exp::Bool(_) => Some(Type::Bool),
        Exp::Str(_)  => Some(Type::Str),
    }
}

fn verify_decls(decls: &mut VarDecl, fenv: &FEnv, env: &mut Env, diags: &mut Diags) {
    for var in &mut decls.vars {
        if let Some(init_exp_node) = &mut var.init {
            verify_exp(init_exp_node, fenv, env, diags);
            if decls.type_spec.ttype != init_exp_node.ttype.unwrap() {
                diags.push(diag::gen_invalid_expression_type(decls.type_spec.ttype, init_exp_node.ttype.unwrap(), init_exp_node.span));
            }
        }

        // add variable disregarding init exp type match
        match env.insert(var.ident.clone(), decls.type_spec.ttype) {
            Some(_) => diags.push(diag::gen_multiple_var_decl(&var.ident, var.span)),
            None => ()
        }
    }
}

fn verify_stmt(stmt_node: &mut StmtNode, fn_type: &Type, fenv: &FEnv, env: &mut Env, diags: &mut Diags) {
    stmt_node.ret = match &mut stmt_node.stmt {
        Stmt::BStmt(stmts) => {
            env.push_scope();
            for stmt_node in stmts.iter_mut() {
                verify_stmt(stmt_node, fn_type,fenv, env, diags)
            }
            env.pop_scope();

            match stmts.last() {
                Some(stmt) => stmt.ret,
                None => Some(false)
            }
        },
        Stmt::Decl(decls) => {
            verify_decls(decls, fenv, env, diags);
            Some(false)
        },
        Stmt::Ass(ident, exp_node) => {
            verify_exp(exp_node, fenv, env, diags);
            match env.get(ident) {
                None => {
                    diags.push(diag::gen_undeclared_variable_in_stmt(&ident, stmt_node.span));
                },
                Some(var_type) => {
                    if is_valid(&exp_node.ttype) && var_type != &exp_node.ttype.unwrap() {
                        diags.push(diag::gen_invalid_expression_type(*var_type, exp_node.ttype.unwrap(), exp_node.span));
                    }
                }
            };
            Some(false)
        },
        Stmt::Incr(ident) | Stmt::Decr(ident) => {
            match env.get(ident) {
                None => diags.push(diag::gen_undeclared_variable_in_stmt(ident, stmt_node.span)),
                Some(ttype) => {
                    if *ttype != Type::Int {
                        diags.push(diag::gen_invalid_expression_type(Type::Int, *ttype, stmt_node.span));
                    }
                }
            };
            Some(false)
        },
        Stmt::Ret(exp) => {
            verify_exp(exp, fenv, env, diags);
            let ttype = exp.ttype.unwrap();
            if &ttype != fn_type {
                diags.push(diag::Diagnostic {
                    message: format!("invalid return type"),
                    details: Some((stmt_node.span, format!("expected {}, found {}", fn_type, ttype)))
                })
            };
            Some(true)
        },
        Stmt::VRet => {
            if fn_type != &Type::Void {
                diags.push(diag::Diagnostic {
                    message: format!("invalid return type"),
                    details: Some((stmt_node.span, format!("expected {}, found none", fn_type)))
                })
            };
            Some(true)
        },
        Stmt::Cond(cond, tstmt, fstmt) => {
            verify_exp(cond, fenv, env, diags);
            verify_stmt(tstmt, fn_type, fenv, env, diags);
            if fstmt.is_some() {
                verify_stmt(fstmt.as_mut().unwrap(), fn_type, fenv, env, diags);
            }
            let ttype = cond.ttype.unwrap();
            if ttype != Type::Bool {
                diags.push(diag::gen_invalid_expression_type(Type::Bool, ttype, cond.span));
            }

            Some(match fstmt {
                Some(fstmt) => tstmt.ret.unwrap() && fstmt.ret.unwrap(),
                None => false
            })
        },
        Stmt::While(cond, body) => {
            verify_exp(cond, fenv, env, diags);
            verify_stmt(body, fn_type, fenv, env, diags);
            let ttype = cond.ttype.unwrap();
            if ttype != Type::Bool {
                diags.push(diag::gen_invalid_expression_type(Type::Bool, ttype, cond.span));
            }

            Some(false)
        },
        Stmt::EStmt(exp) => {
            verify_exp(exp, fenv, env, diags);
            Some(false)
        },
    }
}

pub fn verify_program(prog: &mut Program) -> Diags {
    let mut diags = Vec::new();

    // build function env
    let mut fenv = FEnv::new();
    fenv.insert("readInt".to_owned(), (Type::Int, vec![]));
    fenv.insert("readString".to_owned(), (Type::Str, vec![]));
    fenv.insert("printInt".to_owned(), (Type::Void, vec![Type::Int]));
    fenv.insert("printString".to_owned(), (Type::Void, vec![Type::Str]));

    for fdef in &prog.functions {
        // verify function definitions are unique
        match fenv.insert(fdef.ident.clone(), fdef.get_signature()) {
            Some(_) => diags.push(diag::gen_multiple_fn_def(&fdef.ident, fdef.span)),
            None => ()
        }
    }

    // verify main exists and has valid signature
    match fenv.get("main") {
        None => diags.push(diag::gen_no_main()),
        Some((Type::Int, args)) if args.is_empty() => (),
        _ => diags.push(diag::gen_invalid_main()),
    }


    // verify each function code
    for fdef in &mut prog.functions {
        let mut env = Env::new();
        for decls in &mut fdef.params {
            verify_decls(decls, &fenv, &mut env, &mut diags);
        }
        verify_stmt(&mut fdef.body, &fdef.type_spec.ttype, &fenv, &mut env, &mut diags);

        if fdef.type_spec.ttype != Type::Void {
            if !fdef.body.ret.unwrap() {
                diags.push(diag::Diagnostic {
                    message: format!("no return statement in non-void function {}", fdef.ident),
                    details: None
                });
            }
        }
    }

    return diags;
}