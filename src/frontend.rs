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

// todo refactor t, typ -> type

// first pass -> expression types
// second pass -> valid expression types in statements

// mismatched types (expressions, variables, returns, parameters)

type Env<'a> = HashMap<Ident, Type>;
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
//
fn verify_exp(exp_node: &mut ExpNode, fenv: &FEnv, env: &Env, diags: &mut Diags) {
    exp_node.ttype = match &mut exp_node.exp {
        Exp::Unary(op, inner) => {
            verify_exp(inner, fenv, env, diags);
            let this_type = get_unary_op_type(op, &inner.ttype);
            if is_valid(&inner.ttype) && is_valid(&this_type) {
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
                    message: format!("invalid expression types for operand {}", op),
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
                                message: format!("invalid argument types in function call"),
                                details: Some((exp_node.span, format!("expected {f}({p:?}), got {f}({a:?})", f=ident, p=param_types, a=arg_types)))
                            })
                        }
                    }
                    Some(*ret_type)
                }
            }
        },
        Exp::Var(ident) => {
            match env.get(ident.as_str()) {
                None => {
                    diags.push(diag::Diagnostic {
                        message: format!("unknown variable identifier {}", ident),
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
//
//fn verify_decls(decls: &mut Vec<ast::VarDecl>, fenv: &FEnv, env: &mut Env, diags: &mut Diags) {
//    for ast::VarDecl(vtype, ident, init) in decls {
//        if let Some(texp) = init {
//            verify_exp(texp, fenv, env, diags);
//            if *vtype != texp.etype {
//                diags.push(diag::gen_invalid_assignment(ident, &texp.exp));
//                continue;
//            }
//        }
//        env.insert(ident.clone(), *vtype);
//    }
//}
//
//fn verify_stmt(stmt: &mut ast::Stmt, fenv: &FEnv, env: &mut Env, diags: &mut Diags) {
//    match stmt.stmt {
//        ast::Stmt::BStmt(block) => verify_block(block, fenv, env, diags),
//        ast::Stmt::Decl(decls) => verify_decls(decls, fenv, env, diags),
//        ast::Stmt::Ass(var, texp) => {
//            verify_exp(texp, fenv, env, diags);
//            match env.get(var) {
//                None => diags.push(diag::gen_unknown_variable(var)),
//                Some(vtype) => {
//                    if vtype != &texp.etype {
//                        diags.push(diag::Diagnostic {
//                            message: format!("expression {} has invalid type ({}) in assignment to {} ({})", texp.exp, texp.etype, var, vtype),
//                            details: None,
//                        });
//                    }
//                }
//            }
//        },
//        ast::Stmt::Incr(var) | ast::Stmt::Decr(var) => {
//            match env.get(var) {
//                None => diags.push(diag::gen_unknown_variable(var)),
//                Some(vtype) => {
//                    if vtype != &ast::Type::Int {
//                        diags.push(diag::Diagnostic {
//                            message: format!("cannot inc/dec non-int variable: {}", var),
//                            details: None,
//                        });
//                    }
//                }
//            }
//        },
//        ast::Stmt::Ret(_) => {},
//        ast::Stmt::VRet => {},
//        ast::Stmt::Cond(_, _, _) => {},
//        ast::Stmt::While(_, _) => {},
//        ast::Stmt::EStmt(_) => {},
//    }
//
//}
//
//fn verify_block(ast::Block(stmts): &mut ast::Block, fenv: &FEnv, env: &Env, diags: &mut Diags) {
//    let mut block_env = env.clone();
//    for stmt in stmts {
//        verify_stmt(stmt, fenv, &mut block_env, diags)
//    }
//}
//
pub fn verify_program(ast: &mut Program) -> Diags {
    Vec::new()
//    let mut diags = Vec::new();
//    let ast::Program(functions) = ast;
//
//    // build function env
//    let mut fenv = FEnv::new();
//    for fdef in functions.iter() {
//        let ast::FnDef(_, fname, _, _) = fdef;
//
//        // verify function definitions are unique
//        match fenv.insert(fname.clone(), fdef.get_signature()) {
//            Some(_) => diags.push(diag::gen_multiple_fn_def(&fname)),
//            None => ()
//        }
//    }
//
//    // verify main exists and has valid signature
//    match fenv.get("main") {
//        None => diags.push(diag::gen_no_main()),
//        Some((ast::Type::Int, args)) if args.is_empty() => (),
//        _ => diags.push(diag::gen_invalid_main()),
//    }
//
//
//    // verify each function code
//    for ast::FnDef(_, _, args, block) in functions {
//        let mut env = Env::new();
//        for ast::VarDecl(var_type, var_name, _) in args {
//            // verify function arguments are unique
//            match env.insert(var_name.clone(), *var_type) {
//                Some(_) => diags.push(diag::gen_multiple_arg_def(var_name)),
//                None => ()
//            }
//        }
//        verify_block(block, &fenv, &env, &mut diags);
//    }
//
//
//    return diags;
}