use crate::ast::*;
use crate::diag;
use crate::scoped_map::ScopedMap;
use std::collections::HashMap;
use std::convert::TryInto;

type Env<'a> = ScopedMap<Ident, Type>;
type FEnv<'a> = HashMap<Ident, (Type, Vec<Type>)>;
type Diags = Vec<diag::Diagnostic>;

fn get_unary_op_typeval(op: &UnaryOp, typeval: &ExpTypeVal) -> ExpTypeVal {
    match (op, typeval) {
        (UnaryOp::Neg, ExpTypeVal::Int(Some(v))) => ExpTypeVal::Int(Some(-*v)),
        (UnaryOp::Not, ExpTypeVal::Bool(Some(v))) => ExpTypeVal::Bool(Some(!*v)),
        (UnaryOp::Neg, ExpTypeVal::Int(_)) =>  ExpTypeVal::Int(None),
        (UnaryOp::Not, ExpTypeVal::Bool(_)) => ExpTypeVal::Bool(None),
        _ => ExpTypeVal::Invalid
    }
}

fn get_binary_op_typeval(op: &BinaryOp, ltypeval: &ExpTypeVal, rtypeval: &ExpTypeVal) -> ExpTypeVal {
    match (op, ltypeval, rtypeval) {
        (BinaryOp::Eq,  ExpTypeVal::Bool(Some(l)), ExpTypeVal::Bool(Some(r))) => ExpTypeVal::Bool(Some(*l == *r)),
        (BinaryOp::Eq,  ExpTypeVal::Int(Some(l)),   ExpTypeVal::Int(Some(r))) => ExpTypeVal::Bool(Some(*l == *r)),
        (BinaryOp::Neq,  ExpTypeVal::Bool(Some(l)), ExpTypeVal::Bool(Some(r))) => ExpTypeVal::Bool(Some(*l != *r)),
        (BinaryOp::Neq,  ExpTypeVal::Int(Some(l)),   ExpTypeVal::Int(Some(r))) => ExpTypeVal::Bool(Some(*l != *r)),
        (BinaryOp::Or,  ExpTypeVal::Bool(Some(l)), ExpTypeVal::Bool(Some(r))) => ExpTypeVal::Bool(Some(*l || *r)),
        (BinaryOp::And, ExpTypeVal::Bool(Some(l)), ExpTypeVal::Bool(Some(r))) => ExpTypeVal::Bool(Some(*l && *r)),

        (BinaryOp::Gt,  ExpTypeVal::Int(Some(l)),  ExpTypeVal::Int(Some(r)))  => ExpTypeVal::Bool(Some(*l > *r)),
        (BinaryOp::Gte, ExpTypeVal::Int(Some(l)),  ExpTypeVal::Int(Some(r)))  => ExpTypeVal::Bool(Some(*l >= *r)),
        (BinaryOp::Lt,  ExpTypeVal::Int(Some(l)),  ExpTypeVal::Int(Some(r)))  => ExpTypeVal::Bool(Some(*l < *r)),
        (BinaryOp::Lte, ExpTypeVal::Int(Some(l)),  ExpTypeVal::Int(Some(r)))  => ExpTypeVal::Bool(Some(*l <= *r)),

        (BinaryOp::Add, ExpTypeVal::Int(Some(l)),  ExpTypeVal::Int(Some(r)))  => ExpTypeVal::Int(Some(*l + *r)),
        (BinaryOp::Sub, ExpTypeVal::Int(Some(l)),  ExpTypeVal::Int(Some(r)))  => ExpTypeVal::Int(Some(*l - *r)),
        (BinaryOp::Mul, ExpTypeVal::Int(Some(l)),  ExpTypeVal::Int(Some(r)))  => ExpTypeVal::Int(Some(*l * *r)),
        (BinaryOp::Mod, ExpTypeVal::Int(Some(l)),  ExpTypeVal::Int(Some(r))) if *r != 0 => ExpTypeVal::Int(Some(*l % *r)),
        (BinaryOp::Div, ExpTypeVal::Int(Some(l)),  ExpTypeVal::Int(Some(r))) if *r != 0 => ExpTypeVal::Int(Some(*l / *r)),

        (BinaryOp::Eq,  ExpTypeVal::Bool(_),  ExpTypeVal::Bool(_)) => ExpTypeVal::Bool(None),
        (BinaryOp::Eq,  ExpTypeVal::Int(_),   ExpTypeVal::Int(_))  => ExpTypeVal::Bool(None),
        (BinaryOp::Eq,  ExpTypeVal::Str(_),   ExpTypeVal::Str(_))  => ExpTypeVal::Bool(None),
        (BinaryOp::Neq,  ExpTypeVal::Bool(_), ExpTypeVal::Bool(_)) => ExpTypeVal::Bool(None),
        (BinaryOp::Neq,  ExpTypeVal::Int(_),  ExpTypeVal::Int(_))  => ExpTypeVal::Bool(None),
        (BinaryOp::Neq,  ExpTypeVal::Str(_),  ExpTypeVal::Str(_))  => ExpTypeVal::Bool(None),
        (BinaryOp::Or,  ExpTypeVal::Bool(_),  ExpTypeVal::Bool(_)) => ExpTypeVal::Bool(None),
        (BinaryOp::And, ExpTypeVal::Bool(_),  ExpTypeVal::Bool(_)) => ExpTypeVal::Bool(None),
        (BinaryOp::Gt,  ExpTypeVal::Int(_),   ExpTypeVal::Int(_))  => ExpTypeVal::Bool(None),
        (BinaryOp::Gte, ExpTypeVal::Int(_),   ExpTypeVal::Int(_))  => ExpTypeVal::Bool(None),
        (BinaryOp::Lt,  ExpTypeVal::Int(_),   ExpTypeVal::Int(_))  => ExpTypeVal::Bool(None),
        (BinaryOp::Lte, ExpTypeVal::Int(_),   ExpTypeVal::Int(_))  => ExpTypeVal::Bool(None),
        (BinaryOp::Add, ExpTypeVal::Int(_),   ExpTypeVal::Int(_))  => ExpTypeVal::Int(None),
        (BinaryOp::Add, ExpTypeVal::Str(_),   ExpTypeVal::Str(_))  => ExpTypeVal::Str(None),
        (BinaryOp::Sub, ExpTypeVal::Int(_),   ExpTypeVal::Int(_))  => ExpTypeVal::Int(None),
        (BinaryOp::Mul, ExpTypeVal::Int(_),   ExpTypeVal::Int(_))  => ExpTypeVal::Int(None),
        (BinaryOp::Div, ExpTypeVal::Int(_),   ExpTypeVal::Int(_))  => ExpTypeVal::Int(None),
        (BinaryOp::Mod, ExpTypeVal::Int(_),   ExpTypeVal::Int(_))  => ExpTypeVal::Int(None),
    _ => ExpTypeVal::Invalid
    }
}

fn verify_exp(exp_node: &mut ExpNode, fenv: &FEnv, env: &Env, diags: &mut Diags) {
    exp_node.typeval = match &mut exp_node.exp {
        Exp::Unary(op, inner) => {
            verify_exp(inner, fenv, env, diags);
            let inner_tv = inner.typeval.as_ref().unwrap();
            let typeval = get_unary_op_typeval(op, inner_tv);
            if inner_tv.has_valid_type() && !typeval.has_valid_type() {
                diags.push(diag::Diagnostic{
                    message: format!("invalid use of operand {}", op),
                    details: Some((inner.span, format!("type mismatch: {} {}", op, &inner_tv)))
                });
            }
            Some(typeval)
        },
        Exp::Binary(lexp_node, op, rexp_node) => {
            verify_exp(lexp_node, fenv, env, diags);
            verify_exp(rexp_node, fenv, env, diags);
            let ltv = lexp_node.typeval.as_ref().unwrap();
            let rtv = rexp_node.typeval.as_ref().unwrap();
            let typeval = get_binary_op_typeval(op, ltv, rtv);
            if ltv.has_valid_type() && rtv.has_valid_type() && !typeval.has_valid_type() {
                diags.push(diag::Diagnostic{
                    message: format!("invalid use of operand {}", op),
                    details: Some((exp_node.span, format!("type mismatch: {} {} {}", &ltv, op, &rtv)))
                });
            }
            Some(typeval)
        },
        Exp::Call(ident, args) => {
            match fenv.get(ident.as_str()) {
                None => {
                    diags.push(diag::Diagnostic{
                        message: format!("unknown function identifier {}", ident),
                        details: Some((exp_node.span, format!("in this expression")))
                    });
                    Some(ExpTypeVal::Invalid)
                },
                Some((fn_type, param_types)) => {
                    let mut arg_types = Vec::new();
                    for exp_node in args {
                        verify_exp(exp_node, fenv, env, diags);
                        arg_types.push(exp_node.typeval.as_ref().unwrap());
                    }

                    let arg_types: Vec<Result<Type, ()>> = arg_types.into_iter().map(<&ExpTypeVal as TryInto<Type>>::try_into).collect();
                    let arg_types: Result<Vec<Type>, ()> = arg_types.into_iter().collect();

                    // if any of arguments has invalid type, it was already reported
                    if let Ok(arg_types) = arg_types {
                        // all arguments has valid type, check if it matches with the signature
                        if param_types.ne(&arg_types) {
                            diags.push(diag::Diagnostic {
                                message: format!("invalid argument types"),
                                details: Some((exp_node.span, format!("expected {f}({p:?}), got {f}({a:?})", f=ident, p=param_types, a=arg_types)))
                            })
                        }
                    }
                    Some(ExpTypeVal::from_type(fn_type))
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
                    Some(ExpTypeVal::Invalid)
                } ,
                Some(vtype) => Some(ExpTypeVal::from_type(vtype)),
            }
        },
        Exp::Int(v) => Some(ExpTypeVal::Int(Some(*v))),
        Exp::Bool(v)=> Some(ExpTypeVal::Bool(Some(*v))),
        Exp::Str(v)=>  Some(ExpTypeVal::Str(Some(v.clone()))),
        Exp::Field(_, _) => {unimplemented!()}
        Exp::New(_) => {unimplemented!()}
    }
}

fn verify_decls(decls: &mut VarDecl, fenv: &FEnv, env: &mut Env, diags: &mut Diags) {
    for var in &mut decls.vars {
        if let Some(init_exp_node) = &mut var.init {
            verify_exp(init_exp_node, fenv, env, diags);
            let etv = init_exp_node.typeval.as_ref().unwrap();
            if !etv.has_type(&decls.type_spec.ttype) {
                diags.push(diag::gen_invalid_expression_type(&decls.type_spec.ttype, &etv, init_exp_node.span));
            }
        }

        // add variable disregarding init exp type mismatch
        match env.insert_into_top_scope(var.ident.clone(), decls.type_spec.ttype.clone()) {
            Some(_) => diags.push(diag::gen_multiple_var_decl(&var.ident, var.span)),
            None => ()
        }
    }
}

fn verify_stmt(stmt_node: &mut StmtNode, fn_type: &Type, fenv: &FEnv, env: &mut Env, diags: &mut Diags) {
    stmt_node.will_return = match &mut stmt_node.stmt {
        Stmt::BStmt(stmts) => {
            let mut block_returns = false;
            let mut reachable_stmts = vec![];
            env.push_scope();
            for mut stmt_node in stmts.drain(..) {
                verify_stmt(&mut *stmt_node, fn_type,fenv, env, diags);
                let stmt_returned = stmt_node.will_return.unwrap();
                reachable_stmts.push(stmt_node);
                if stmt_returned {
                    block_returns = true;
                    break;
                }
            }
            env.pop_scope();
            *stmts = reachable_stmts;
            Some(block_returns)
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
                    let etv = exp_node.typeval.as_ref().unwrap();
                    if etv.has_valid_type() && !etv.has_type(var_type) {
                        diags.push(diag::gen_invalid_expression_type(var_type, &etv, exp_node.span));
                    }
                }
            };
            Some(false)
        },
        Stmt::Incr(ident) | Stmt::Decr(ident) => {
            match env.get(ident) {
                None => diags.push(diag::gen_undeclared_variable_in_stmt(ident, stmt_node.span)),
                Some(vtype) => {
                    if *vtype != Type::Int {
                        diags.push(diag::gen_invalid_expression_type(&Type::Int, &ExpTypeVal::from_type(vtype), stmt_node.span));
                    }
                }
            };
            Some(false)
        },
        Stmt::Ret(exp) => {
            verify_exp(exp, fenv, env, diags);
            let etv = exp.typeval.as_ref().unwrap();
            if !etv.has_type(fn_type) {
                diags.push(diag::Diagnostic {
                    message: format!("invalid return type"),
                    details: Some((stmt_node.span, format!("expected {}, found {}", fn_type, &etv)))
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
            if let Some(fstmt) = fstmt {
                verify_stmt(fstmt, fn_type, fenv, env, diags);
            }

            let ctv = cond.typeval.as_ref().unwrap();
            match ctv {
                ExpTypeVal::Bool(condval) => {
                    match (&condval, &fstmt) {
                        (Some(true), _) => tstmt.will_return,
                        (Some(false), Some(_)) => fstmt.as_ref().unwrap().will_return,
                        (None, Some(_)) => Some(tstmt.will_return.unwrap() && fstmt.as_ref().unwrap().will_return.unwrap()),
                        _ => Some(false)
                    }
                }
                _ => {
                    diags.push(diag::gen_invalid_expression_type(&Type::Bool, &ctv, cond.span));
                    Some(false)
                }
            }
        },
        Stmt::While(cond, body) => {
            verify_exp(cond, fenv, env, diags);
            verify_stmt(body, fn_type, fenv, env, diags);

            let ctv = cond.typeval.as_ref().unwrap();
            match &ctv {
                ExpTypeVal::Bool(condval) => {
                    match &condval {
                        // if while(true), this will either loop infinitely or return
                        Some(true) => Some(true),
                        _ => Some(false)
                    }
                }
                _ => {
                    diags.push(diag::gen_invalid_expression_type(&Type::Bool, &ctv, cond.span));
                    Some(false)
                }
            }
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
            if !fdef.body.will_return.unwrap() {
                diags.push(diag::Diagnostic {
                    message: format!("no return statement in non-void function {}", fdef.ident),
                    details: None
                });
            }
        }
        else { // void, push implicit vret if needed
            if let Stmt::BStmt(vec) = &mut fdef.body.stmt {
                match vec.last() {
                    None => {
                        vec.push(Box::new(StmtNode{span: Span(0, 0), will_return: Some(true), stmt: Stmt::VRet}));
                    }
                    Some(lstmt) if !lstmt.will_return.unwrap() => {
                        vec.push(Box::new(StmtNode{span: Span(0, 0), will_return: Some(true), stmt: Stmt::VRet}));
                    }
                    _ => ()
                }
            }
        }
    }

    return diags;
}
