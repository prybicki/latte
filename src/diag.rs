use crate::File;
use crate::latte;
use crate::ParseError;
use crate::ast;
use codespan_reporting::term::{emit, DisplayStyle};
use codespan_reporting::diagnostic::Label;
use codespan_reporting::term::Config;
use codespan_reporting::diagnostic::Diagnostic as Diag;

pub struct Diagnostic {
    pub message: String,
    pub details: Option<(ast::Span, String)>
}

pub fn gen_no_main() -> Diagnostic {
    Diagnostic {message: "missing main function".to_owned(), details: None}
}

pub fn gen_invalid_main() -> Diagnostic {
    Diagnostic {message: "invalid main function".to_owned(), details: None}
}

pub fn gen_multiple_fn_def(ident: &ast::Ident, span: ast::Span) -> Diagnostic {
    Diagnostic {
        message: format!("multiple declaration of function {}", ident),
        details: Some((span, "defined second time here".to_owned()))
    }
}

pub fn gen_multiple_var_decl(ident: &ast::Ident, span: ast::Span) -> Diagnostic {
    Diagnostic {
        message: format!("variable already declared in current scope {}", ident),
        details: Some((span, "second definition here".to_owned()))
    }
}

pub fn gen_undeclared_variable_in_stmt(ident: &ast::Ident, span: ast::Span) -> Diagnostic {
    Diagnostic {
        message: format!("undeclared variable {}", ident),
        details: Some((span, format!("in this statement")))
    }
}

pub fn gen_invalid_expression_type(expected: &ast::Type, actual: &ast::ExpTypeVal, span: ast::Span) -> Diagnostic {
    Diagnostic {
        message: format!("invalid expression type"),
        details: Some((span, format!("expected {}, got {}", expected, actual)))
    }
}

//pub fn gen_invalid_unary(exp: &ast::Exp) -> Diagnostic {
//    Diagnostic {message: format!("invalid unary exp: {}", exp), details: None }
//}
//
//pub fn gen_invalid_binary(exp: &ast::Exp) -> Diagnostic {
//    Diagnostic {message: format!("invalid binary exp: {}", exp), details: None }
//}
//
//pub fn gen_unknown_function(ident: &ast::Ident, exp: &ast::Exp) -> Diagnostic {
//    Diagnostic {message: format!("unknown function: {} (in expr: {})", ident, exp), details: None}
//}
//
//pub fn gen_invalid_arguments(exp: &ast::Exp) -> Diagnostic {
//    Diagnostic {message: format!("invalid arguments in call: {}", exp), details: None}
//}
//
//pub fn gen_invalid_assignment(ident: &ast::Ident, exp: &ast::Exp) -> Diagnostic {
//    Diagnostic {message: format!("cannot assign {} to {}", exp, ident), details: None}
//}
//
//pub fn gen_unknown_variable(ident: &ast::Ident) -> Diagnostic {
//    Diagnostic {
//        message: format!("unknown variable: {}", ident),
//        details: None,
//    }
//}

pub fn gen_from_parse_error(err: ParseError) -> Diagnostic {
    let ((b, e), comment) = match err {
        ParseError::InvalidToken{location: l} => {
            ((l, l), "invalid token".to_owned())
        },
        ParseError::UnrecognizedEOF{location: l, ..} => {
            ((l, l), "unexpected eof".to_owned())
        },
        ParseError::UnrecognizedToken{token: (b, latte::Token(_, token_str), e), expected: exp_vec} => {
            ((b, e), format!("unrecognized token: {:?}, expected one of: {:?}", token_str, exp_vec))
        },
        ParseError::ExtraToken{token: (b, latte::Token(_, token_str), e)} => {
            ((b, e), format!("unexpected additional token: {}", token_str))
        },
        ParseError::User{error} => ((0, 0), format!("{}", error)),
    };
    if (b, e) == (0, 0) {
        return Diagnostic {message: comment, details: None}
    }
    Diagnostic {message: "syntax error".to_owned(), details: Some((ast::Span(b, e) , comment))}
}

pub fn print_all(diagnostics: &[Diagnostic], file: &File) {
    let mut stream = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
    let long_cfg = Config::default();
    let short_cfg = {let mut cfg = Config::default(); cfg.display_style = DisplayStyle::Short; cfg};

    for diagnostic in diagnostics {
        let (diag, config) =
            if let Some((span, comment)) = &diagnostic.details {
                let span = codespan::Span::new(span.0 as u32, span.1 as u32);
                let label = Label::new(file.file_id, span, comment);
                let diag = codespan_reporting::diagnostic::Diagnostic::new_error(diagnostic.message.clone(), label);
                let config = &long_cfg;
                (diag, config)
            }
            else {
                let span = codespan::Span::new(0, 0);
                let label = Label::new(file.file_id, span, "");
                let diag = Diag::new_error(diagnostic.message.clone(), label);
                let config = &short_cfg;
                (diag, config)
            };
        emit(&mut stream, config, &file.file_db, &diag).unwrap()
    }
}
