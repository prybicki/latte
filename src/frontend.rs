use crate::ast::*;
use crate::diag;
use crate::scoped_map::ScopedMap;
use std::collections::HashMap;
use std::convert::TryInto;
use std::ops::DerefMut;

type Env = ScopedMap<Ident, Type>;
type FEnv = HashMap<Ident, (Type, Vec<Type>)>;
type Diags = Vec<diag::Diagnostic>;

type ClassDesc = HashMap<Ident, Type>;

#[derive(Debug)]
struct CEnv {
    classes: HashMap<Ident, ClassDesc>
}

impl CEnv {
    fn new(class_list: &Vec<ClassDef>) -> Self {
        let mut classes = HashMap::new();
        for class in class_list.iter() {
            let mut class_desc = HashMap::new();
            for field in class.fields.iter() {
                class_desc.insert(field.vars.first().unwrap().ident.clone(), field.type_spec.ttype.clone());
            }
            classes.insert(class.ident.clone(), class_desc);
        }
        CEnv{classes}
    }

    fn get_type_of_field(&self, class_name: &Ident, field_name: &Ident) -> Option<Type> {
        self.classes.get(class_name).and_then(|class| class.get(field_name)).cloned()
    }

    fn has_type(&self, typename: &Ident) -> bool {
        let result = self.classes.get(typename).is_some();
        result
    }
}

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
        (BinaryOp::Eq,  ExpTypeVal::Class(lhs),   ExpTypeVal::Class(rhs))  => {
            if lhs == rhs { ExpTypeVal::Bool(None) } else { ExpTypeVal::Invalid }
        },
        (BinaryOp::Neq,  ExpTypeVal::Class(lhs),   ExpTypeVal::Class(rhs))  => {
            if lhs == rhs { ExpTypeVal::Bool(None) } else { ExpTypeVal::Invalid }
        },
    _ => ExpTypeVal::Invalid
    }
}

fn verify_class_field(ttype: &Type, field_name: &Ident, cenv: &CEnv, diags: &mut Diags, span: Span) -> ExpTypeVal {
    if let Type::Class(class_name) = ttype {
        if cenv.has_type(class_name) {
            if let Some(field_type) = cenv.get_type_of_field(class_name, field_name) {
                ExpTypeVal::from_type(&field_type)
            } else {
                diags.push(diag::Diagnostic{
                    message: format!("class \"{}\" does not have \"{}\" field", class_name, field_name),
                    details: Some((span, format!("no such field")))
                });
                ExpTypeVal::Invalid
            }
        }
        else {
            diags.push(diag::Diagnostic{
                message: format!("undeclared class \"{}\"", class_name),
                details: Some((span, format!("in this expression")))
            });
            ExpTypeVal::Invalid
        }
    }
    else {
        diags.push(diag::Diagnostic{
            message: format!("invalid use of . (dot) operator"),
            details: Some((span, format!("this is not a class")))
        });
        ExpTypeVal::Invalid
    }
}

fn verify_object_field(field: &mut FieldNode, cenv: &CEnv, env: &Env, diags: &mut Diags) {
    field.typeval = Some(match &mut field.field {
        Field::Direct(obj_name, field_name) => {
            if let Some(ttype) = env.get(obj_name) {
                // left side is in environment
                verify_class_field(ttype, field_name, cenv, diags, field.span)
            } else {
                diags.push(diag::Diagnostic{
                    message: format!("object not found in current scope: {}", obj_name),
                    details: Some((field.span, format!("in this place")))
                });
                ExpTypeVal::Invalid
            }
        }
        Field::Indirect(obj_field, fld) => {
            verify_object_field(obj_field, cenv, env, diags);
            let typeval = obj_field.typeval.as_mut().unwrap();
            if let ExpTypeVal::Class(typename) = typeval {
                // left side evaluates to class typeval
                verify_class_field(&Type::Class(typename.clone()), &fld, cenv, diags, field.span)
            }
            else {
                // error already emitted by verify_object_field
                ExpTypeVal::Invalid
            }
        }
    });
}

fn verify_exp(exp_node: &mut ExpNode, fenv: &FEnv, cenv: &CEnv, env: &Env, diags: &mut Diags) {
    exp_node.typeval = match &mut exp_node.exp {
        Exp::Unary(op, inner) => {
            verify_exp(inner, fenv, cenv, env, diags);
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
            verify_exp(lexp_node, fenv, cenv, env, diags);
            verify_exp(rexp_node, fenv, cenv, env, diags);
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
                        verify_exp(exp_node, fenv, cenv, env, diags);
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
                                details: Some((exp_node.span, format!("expected {f}({p:?}), found {f}({a:?})", f=ident, p=param_types, a=arg_types)))
                            })
                        }
                    }
                    Some(ExpTypeVal::from_type(fn_type))
                }
            }
        },
        Exp::Obj(MemLoc::Var(ident)) => {
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
        Exp::Obj(MemLoc::Field(field)) => {
            verify_object_field(field, cenv, env, diags);
            field.typeval.clone()
        },
        Exp::Int(v) => Some(ExpTypeVal::Int(Some(*v))),
        Exp::Bool(v)=> Some(ExpTypeVal::Bool(Some(*v))),
        Exp::Str(v)=>  Some(ExpTypeVal::Str(Some(v.clone()))),
        Exp::New(typename) | Exp::Null(typename) => {
            if cenv.has_type(typename) {
                Some(ExpTypeVal::Class(typename.clone()))
            }
            else {
                diags.push(diag::Diagnostic{
                    message: format!("unknown class name"),
                    details: Some((exp_node.span, format!("in this expression"))),
                });
                Some(ExpTypeVal::Invalid)
            }
        }
    }
}

fn verify_decls(decls: &mut VarDecl, fenv: &FEnv, cenv: &CEnv, env: &mut Env, diags: &mut Diags) {
    for var in &mut decls.vars {
        if let Some(init_exp_node) = &mut var.init {
            verify_exp(init_exp_node, fenv, cenv, env, diags);
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

fn verify_stmt(stmt_node: &mut StmtNode, fn_type: &Type, fenv: &FEnv, cenv: &CEnv, env: &mut Env, diags: &mut Diags) {
    stmt_node.will_return = match &mut stmt_node.stmt {
        Stmt::BStmt(stmts) => {
            let mut block_returns = false;
            let mut reachable_stmts = vec![];
            env.push_scope();
            for mut stmt_node in stmts.drain(..) {
                verify_stmt(&mut *stmt_node, fn_type, fenv, cenv, env, diags);
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
            verify_decls(decls, fenv, cenv, env, diags);
            Some(false)
        },
        Stmt::Ass(MemLoc::Var(ident), exp_node) => {
            verify_exp(exp_node, fenv, cenv, env, diags);
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
        Stmt::Incr(MemLoc::Var(ident)) | Stmt::Decr(MemLoc::Var(ident)) => {
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
        Stmt::Incr(MemLoc::Field(_)) | Stmt::Decr(MemLoc::Field(_)) | Stmt::Ass(MemLoc::Field(_), _) => {
            println!("Ignoring field in inc/dec/ass");
            Some(false)
        }
        Stmt::Ret(exp) => {
            verify_exp(exp, fenv, cenv, env, diags);
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
            verify_exp(cond, fenv, cenv, env, diags);

            verify_stmt(tstmt, fn_type, fenv, cenv, env, diags);
            if let Some(fstmt) = fstmt {
                verify_stmt(fstmt, fn_type, fenv, cenv, env, diags);
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
            verify_exp(cond, fenv, cenv, env, diags);
            verify_stmt(body, fn_type, fenv, cenv, env, diags);

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
            verify_exp(exp, fenv, cenv, env, diags);
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

    // process struct definitions
    let cenv = CEnv::new(&prog.classes);


    // verify each function code
    for fdef in &mut prog.functions {
        let mut env = Env::new();
        for decls in &mut fdef.params {
            verify_decls(decls, &fenv, &cenv, &mut env, &mut diags);
        }
        verify_stmt(&mut fdef.body, &fdef.type_spec.ttype, &fenv, &cenv, &mut env, &mut diags);

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
