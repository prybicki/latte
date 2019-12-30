use crate::ast;
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

type Env<'a> = HashMap<ast::Ident, ast::Type>;
type FEnv<'a> = HashMap<ast::Ident, (ast::Type, Vec<ast::Type>)>;
type Diags = Vec<diag::Diagnostic>;

fn get_unary_op_type(op: &ast::UnaryOp, etype: &ast::Type) -> ast::Type {
    use ast::UnaryOp;
    use ast::Type;
    return match (op, etype) {
        (UnaryOp::Neg, Type::Int) =>  Type::Int,
        (UnaryOp::Not, Type::Bool) => Type::Bool,
        _ => Type::Invalid
    };
}

fn get_binary_op_type(op: &ast::BinaryOp, ltype: &ast::Type, rtype: &ast::Type) -> ast::Type {
    use ast::BinaryOp::*;
    use ast::Type;
    return match (op, ltype, rtype) {
        (Eq,  l, r) => if l == r { Type::Bool } else { Type::Invalid },
        (Neq, l, r) => if l == r { Type::Bool } else { Type::Invalid },
        (Or, Type::Bool, Type::Bool) => Type::Bool,
        (And, Type::Bool, Type::Bool) => Type::Bool,
        (Gt, Type::Int, Type::Int) => Type::Bool,
        (Gte, Type::Int, Type::Int) => Type::Bool,
        (Lt, Type::Int, Type::Int) => Type::Bool,
        (Lte, Type::Int, Type::Int) => Type::Bool,
        (Add, Type::Int, Type::Int) => Type::Int,
        (Sub, Type::Int, Type::Int) => Type::Int,
        (Mul, Type::Int, Type::Int) => Type::Int,
        (Div, Type::Int, Type::Int) => Type::Int,
        (Mod, Type::Int, Type::Int) => Type::Int,
        _ => Type::Invalid
    };
}

fn verify_exp(texp: &mut ast::TypedExp, fenv: &FEnv, env: &Env, diags: &mut Diags) {
    texp.etype = match &mut texp.exp {
        ast::Exp::Unary(ref op, ref mut inner) => {
            verify_exp(inner, fenv, env, diags);
            let etype = get_unary_op_type(op, &inner.etype);
            if inner.etype.is_valid() && etype.is_valid() {
                diags.push(diag::gen_invalid_unary(&texp.exp));
            }
            etype
        },
        _ => ast::Type::Invalid,
        ast::Exp::Binary(lexp, op, rexp) => {
            verify_exp(lexp, fenv, env, diags);
            verify_exp(rexp, fenv, env, diags);
            let etype = get_binary_op_type(op, &lexp.etype, &rexp.etype);
            if lexp.etype.is_valid() && rexp.etype.is_valid() && !etype.is_valid() {
                diags.push(diag::gen_invalid_binary(&texp.exp));
            }
            etype
        },
        ast::Exp::Call(ident, args) => {
            match fenv.get(ident.as_str()) {
                None => {
                    // TODO improve error handling
//                    diags.push(diag::gen_unknown_function(&ident, &texp.exp));
                    diags.push(diag::Diagnostic{
                        message: format!("unknown function: {}", ident),
                        details: None
                    });
                    ast::Type::Invalid
                },
                Some((ret_type, param_types)) => {
                    let mut arg_types = Vec::new();
                    for exp in args {
                        verify_exp(exp, fenv, env, diags);
                        arg_types.push(exp.etype);
                    }
                    // if any of arguments has invalid type, it will be reported earlier
                    if arg_types.iter().all(ast::Type::is_valid) && param_types.ne(&arg_types) {
                        diags.push(diag::gen_invalid_arguments(&texp.exp));
                    }
                    *ret_type
                }
            }
        },
        ast::Exp::Var(ident) => {
            match env.get(ident.as_str()) {
                None => ast::Type::Invalid,
                Some(var_type) => *var_type,
            }
        },
        ast::Exp::Int(_) => ast::Type::Int,
        ast::Exp::Bool(_) => ast::Type::Bool,
        ast::Exp::Str(_) => ast::Type::Str,
    }
}

fn verify_decls(decls: &mut Vec<ast::VarDecl>, fenv: &FEnv, env: &mut Env, diags: &mut Diags) {
    for ast::VarDecl(vtype, ident, init) in decls {
        if let Some(texp) = init {
            verify_exp(texp, fenv, env, diags);
            if *vtype != texp.etype {
                diags.push(diag::gen_invalid_assignment(ident, &texp.exp));
                continue;
            }
        }
        env.insert(ident.clone(), *vtype);
    }
}

fn verify_stmt(stmt: &mut ast::Stmt, fenv: &FEnv, env: &mut Env, diags: &mut Diags) {
    match stmt {
        ast::Stmt::BStmt(block) => verify_block(block, fenv, env, diags),
        ast::Stmt::Decl(decls) => verify_decls(decls, fenv, env, diags),
        ast::Stmt::Ass(var, texp) => {
            verify_exp(texp, fenv, env, diags);
            match env.get(var) {
                None => diags.push(diag::gen_unknown_variable(var)),
                Some(vtype) => {
                    if vtype != &texp.etype {
                        diags.push(diag::Diagnostic {
                            message: format!("expression {} has invalid type ({}) in assignment to {} ({})", texp.exp, texp.etype, var, vtype),
                            details: None,
                        });
                    }
                }
            }
        },
        ast::Stmt::Incr(var) | ast::Stmt::Decr(var) => {
            match env.get(var) {
                None => diags.push(diag::gen_unknown_variable(var)),
                Some(vtype) => {
                    if vtype != &ast::Type::Int {
                        diags.push(diag::Diagnostic {
                            message: format!("cannot inc/dec non-int variable: {}", var),
                            details: None,
                        });
                    }
                }
            }
        },
        ast::Stmt::Ret(_) => {},
        ast::Stmt::VRet => {},
        ast::Stmt::Cond(_, _, _) => {},
        ast::Stmt::While(_, _) => {},
        ast::Stmt::EStmt(_) => {},
    }

}

fn verify_block(ast::Block(stmts): &mut ast::Block, fenv: &FEnv, env: &Env, diags: &mut Diags) {
    let mut block_env = env.clone();
    for stmt in stmts {
        verify_stmt(stmt, fenv, &mut block_env, diags)
    }
}

pub fn verify_program(ast: &mut ast::Program) -> Vec<diag::Diagnostic> {
    let mut diags = Vec::new();
    let ast::Program(functions) = ast;

    // build function env
    let mut fenv = FEnv::new();
    for fdef in functions.iter() {
        let ast::FnDef(_, fname, _, _) = fdef;

        // verify function definitions are unique
        match fenv.insert(fname.clone(), fdef.get_signature()) {
            Some(_) => diags.push(diag::gen_multiple_fn_def(&fname)),
            None => ()
        }
    }

    // verify main exists and has valid signature
    match fenv.get("main") {
        None => diags.push(diag::gen_no_main()),
        Some((ast::Type::Int, args)) if args.is_empty() => (),
        _ => diags.push(diag::gen_invalid_main()),
    }


    // verify each function code
    for ast::FnDef(_, _, args, block) in functions {
        let mut env = Env::new();
        for ast::VarDecl(var_type, var_name, _) in args {
            // verify function arguments are unique
            match env.insert(var_name.clone(), *var_type) {
                Some(_) => diags.push(diag::gen_multiple_arg_def(var_name)),
                None => ()
            }
        }
        verify_block(block, &fenv, &env, &mut diags);
    }


    return diags;
}