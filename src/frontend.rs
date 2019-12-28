use crate::ast;
use crate::diag;
use std::collections::HashMap;
use std::collections::HashSet;
use crate::ast::{ExpRaw, UnaryOp, Stmt};

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

// first pass -> expression types
// second pass -> valid expression types in statements

// mismatched types (expressions, variables, returns, parameters)

type Env<'a> = HashMap<&'a str, ast::Type>;
type FEnv<'a> = HashMap<&'a str, &'a ast::FnDef>;
type Diags = Vec<diag::Diagnostic>;

fn get_unary_op_type(op: ast::UnaryOp, etype: ast::Type) -> ast::Type {
    use ast::BinaryOp::*;
    use ast::Type::*;
    return match (op, etype) {
        (Neg, Int) => Int,
        (Not, Bool) => Bool,
        _ => Invalid
    };
}

fn get_binary_op_type(op: ast::BinaryOp, ltype: ast::Type, rtype: ast::Type) -> ast::Type {
    use ast::BinaryOp::*;
    use ast::Type::*;
    return match (op, ltype, rtype) {
        (Or, Bool, Bool) => Bool,
        (And, Bool, Bool) => Bool,
        (Eq, l, r) => if l == r { Bool } else { Invalid },
        (Neq, l, r) => if l == r { Bool } else { Invalid },
        (Gt, Int, Int) => Bool,
        (Gte, Int, Int) => Bool,
        (Lt, Int, Int) => Bool,
        (Lte, Int, Int) => Bool,
        (Add, Int, Int) => Int,
        (Sub, Int, Int) => Int,
        (Mul, Int, Int) => Int,
        (Div, Int, Int) => Int,
        (Mod, Int, Int) => Int,
        _ => Invalid
    };
}

fn verify_exp(exp: &mut ast::Exp, fenv: &FEnv, env: &Env, diag: &mut Diags) {
    let ast::Exp{raw: raw, typ: typ} = exp;
    match raw {
        ExpRaw::Unary(op, inner) => {
            let ast::Exp{raw: inner_raw, typ: inner_type} = inner;
            verify_exp(inner, fenv, env, diag);
            *typ = get_unary_op_type(*op, *inner_type);
        },
        ExpRaw::Binary(_, _, _) => {},
        ExpRaw::Call(_, _) => {},
        ExpRaw::Int(_) => {},
        ExpRaw::Bool(_) => {},
        ExpRaw::Str(_) => {},
        ExpRaw::Var(_) => {},
    }
}

fn verify_decls(declarations: &Vec<ast::VarDecl>, fenv: &FEnv, env: &mut Env, diags: &mut Diags) {
//    for d in decls {
//        verify_exp()
//    }
}

fn verify_stmt(stmt: &ast::Stmt, fenv: &FEnv, env: &mut Env, diags: &mut Diags) {
    match stmt {
        Stmt::BStmt(block) => verify_block(block, fenv, env, diags),
        Stmt::Decl(decls) => verify_decls(decls, fenv, env, diags),
        Stmt::Ass(_, _) => {},
        Stmt::Incr(_) => {},
        Stmt::Decr(_) => {},
        Stmt::Ret(_) => {},
        Stmt::VRet => {},
        Stmt::Cond(_, _, _) => {},
        Stmt::While(_, _) => {},
        Stmt::EStmt(_) => {},
    }

}

fn verify_block(ast::Block(stmts): &ast::Block, fenv: &FEnv, env: &Env, diags: &mut Diags) {
    let mut block_env = env.clone();
    for stmt in stmts {
        verify_stmt(stmt, fenv, &mut block_env, diags)
    }
}

pub fn verify_program(ast: &ast::Program) -> Vec<diag::Diagnostic> {
    let mut diags = Vec::new();
    let ast::Program(functions) = ast;

    let mut fenv = FEnv::new();
    for fdef in functions {
        let ast::FnDef(t, fname, args, block) = fdef;

        // verify function definitions are unique
        match fenv.insert(fname, fdef) {
            Some(_) => diags.push(diag::gen_multiple_fn_def(fname)),
            None => ()
        }

        // verify each function code
        let mut env = HashMap::<&str, ast::Type>::new();
        for ast::VarDecl(t, n, _) in args {
            // verify function arguments are unique
            match env.insert(n, *t) {
                Some(_) => diags.push(diag::gen_multiple_arg_def(n)),
                None => ()
            }
        }
        verify_block(block, &fenv, &env, &mut diags);
    }

    // verify main exists and has valid signature
    match fenv.get("main") {
        None => diags.push(diag::gen_no_main()),
        Some(ast::FnDef(t, n, args, _)) => {
            let main_ok = (*t == ast::Type::Int && args.is_empty());
            if !main_ok {
                diags.push(diag::gen_invalid_main())
            }
        }
    }

    return diags;
}