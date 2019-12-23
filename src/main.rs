use std::fs;
use std::path::Path;
use std::cell::RefCell;
use codespan_reporting::diagnostic::Diagnostic;
use termcolor;
use std::io;

#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(pub latte);

pub mod ast;

type ParseError<'i> = lalrpop_util::ParseError<usize, latte::Token<'i>, &'static str>;

struct Context {
    out: RefCell<Box<dyn termcolor::WriteColor>>,
    cfg: codespan_reporting::term::Config,
    file_db: RefCell<codespan::Files>,
    file_id: codespan::FileId,
}

impl Context {
    fn new(file_name: &str) -> Result<Context, io::Error> {
        let mut file_db = codespan::Files::default();
        let file_content = fs::read_to_string(Path::new(file_name))?;
        let file_id = file_db.add(file_name, file_content);
        let out = Box::new(termcolor::StandardStream::stderr(termcolor::ColorChoice::Always));
        let cfg = codespan_reporting::term::Config::default();
        Ok(Context{out: RefCell::new(out), cfg, file_db: RefCell::new(file_db), file_id})
    }

    fn print(&self, diagnostic: &Diagnostic) {
        codespan_reporting::term::emit(
            self.out.borrow_mut().as_mut(),
            &self.cfg,
            &self.file_db.borrow(),
            &diagnostic
        ).unwrap() // TODO
    }
}


// cannot be a trait due to https://github.com/rust-lang/rfcs/blob/master/text/1023-rebalancing-coherence.md
fn parse_error_to_diagnostic(err: ParseError, file_id: codespan::FileId) -> Diagnostic {
    let (comment, (b, e)) = match err {
        ParseError::InvalidToken{location: l} => {
            ("invalid token".to_owned(), (l, l))
        },
        ParseError::UnrecognizedEOF{location: l, ..} => {
            ("unexpected eof".to_owned(), (l, l))
        },
        ParseError::UnrecognizedToken{token: (b, latte::Token(_, token_str), e), expected: exp_vec} => {
            // TODO pretty print of exp_vec
            (format!("unrecognized token: {:?}, expected one of: {:?}", token_str, exp_vec), (b, e))
        },
        ParseError::ExtraToken{token: (b, latte::Token(_, token_str), e)} => {
            (format!("unexpected additional token: {}", token_str), (b, e))
        },
        _ => panic!("undefined parser error")
    };
    let (b, e) = (b as u32, e as u32);
    let label = codespan_reporting::diagnostic::Label::new(
        file_id, codespan::Span::new(b, e),&comment);
    return Diagnostic::new_error("syntax error", label);
}

fn remove_comments(text: &str) -> String {
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

fn compile(ctx: &Context) -> Result<(), Vec<Diagnostic>> {
    let stripped = remove_comments(ctx.file_db.borrow().source(ctx.file_id));
    let ast = match latte::GProgramParser::new().parse(&stripped) {
        Err(err) => return Err(vec![parse_error_to_diagnostic(err, ctx.file_id)]),
        Ok(v) => v
    };
    Ok(())
}

fn die(msg: &str) -> ! {
    println!("ERROR\n");
    println!("{}", msg);
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 { die(&format!("expected 1 argument, got {}", args.len()-1)) }

    let ctx = match Context::new(&args[1]) {
        Err(err) => die(&format!("error while reading file: {}", err)),
        Ok(v) => v,
    };

    match compile(&ctx) {
        Err(diags) => {
            println!("ERROR\n");
            for diag in diags.iter() {
                ctx.print(diag);
            }
            std::process::exit(1);
        },
        Ok(_) => {
            println!("OK");
            std::process::exit(0);
        }
    }
}

#[test]
fn good() {
    let list = [
        "./lattests/good/core001.lat",
        "./lattests/good/core002.lat",
        "./lattests/good/core003.lat",
        "./lattests/good/core004.lat",
        "./lattests/good/core005.lat",
        "./lattests/good/core006.lat",
        "./lattests/good/core007.lat",
        "./lattests/good/core008.lat",
        "./lattests/good/core009.lat",
        "./lattests/good/core010.lat",
        "./lattests/good/core011.lat",
        "./lattests/good/core012.lat",
        "./lattests/good/core013.lat",
        "./lattests/good/core014.lat",
        "./lattests/good/core015.lat",
        "./lattests/good/core016.lat",
        "./lattests/good/core017.lat",
        "./lattests/good/core018.lat",
        "./lattests/good/core019.lat",
        "./lattests/good/core020.lat",
        "./lattests/good/core021.lat",
        "./lattests/good/core022.lat",
    ];

    for &item in list.iter() {
        let ctx = Context::new(item).unwrap();
        let result = compile(&ctx);
        assert!(result.is_ok(), "{}: {:?}", item, result.unwrap_err());
        println!("OK {}", item);
    }
}