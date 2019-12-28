use crate::File;
use crate::latte;
use crate::ParseError;
use codespan::Span;
use codespan_reporting::term::{emit, DisplayStyle};
use codespan_reporting::diagnostic::Label;
use codespan_reporting::term::Config;
use codespan_reporting::diagnostic::Diagnostic as Diag;

pub struct Diagnostic {
    message: &'static str,
    details: Option<(u32, u32, String)>
}

pub fn gen_invalid_main() -> Diagnostic {
    Diagnostic {message: "no valid main function", details: None}
}

pub fn gen_from_parse_error(err: ParseError) -> Diagnostic {
    let ((b, e), comment) = match err {
        ParseError::InvalidToken{location: l} => {
            ((l, l), "invalid token".to_owned())
        },
        ParseError::UnrecognizedEOF{location: l, ..} => {
            ((l, l), "unexpected eof".to_owned())
        },
        ParseError::UnrecognizedToken{token: (b, latte::Token(_, token_str), e), expected: exp_vec} => {
            // TODO pretty print of exp_vec
            ((b, e), format!("unrecognized token: {:?}, expected one of: {:?}", token_str, exp_vec))
        },
        ParseError::ExtraToken{token: (b, latte::Token(_, token_str), e)} => {
            ((b, e), format!("unexpected additional token: {}", token_str))
        },
        _ => panic!("undefined parser error")
    };

    Diagnostic {message: "syntax error", details: Some((b as u32, e as u32 , comment))}
}

pub fn print_all(diagnostics: &[Diagnostic], file: &File) {
    let mut stream = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
    let long_cfg = Config::default();
    let short_cfg = {let mut cfg = Config::default(); cfg.display_style = DisplayStyle::Short; cfg};

    for diagnostic in diagnostics {
        let (diag, config) =
            if let Some((b, e, ref comment)) = diagnostic.details {
                let span = Span::new(b, e);
                let label = Label::new(file.file_id, span, comment);
                let diag = codespan_reporting::diagnostic::Diagnostic::new_error(diagnostic.message, label);
                let config = &long_cfg;
                (diag, config)
            }
            else {
                let span = Span::new(0, 0);
                let label = Label::new(file.file_id, span, "");
                let diag = Diag::new_error(diagnostic.message, label);
                let config = &short_cfg;
                (diag, config)
            };
        emit(&mut stream, config, &file.file_db, &diag).unwrap() // TODO
    }
}
