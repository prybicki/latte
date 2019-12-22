use std::fs;
use std::path::Path;
use std::error::Error;
use std::fmt::Write as FmtWrite;
use std::cell::RefCell;
use codespan_reporting::diagnostic::Label;
use codespan_reporting::diagnostic::Diagnostic;
use termcolor;

pub mod latte;
pub mod ast;

type ParseError<'i> = lalrpop_util::ParseError<usize, latte::Token<'i>, &'static str>;

#[derive(Debug)]
enum CompilerError {
    InvalidArgument,
    FileNotFound{path: Box<Path>},
    SyntaxError{msg: String},
    SemanticError{span: (usize, usize), msg: String},
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
//        println!("{} {:?} {:?}", ch, s1, s2);
        output.push(match s1 {
            PrimaryState::InCode => ch,
            _ => if ch == '\n' { '\n' } else { ' ' },
//            PrimaryState::InSingleLineComment => '@',
//            PrimaryState::AfterForwardSlash => '!',
//            PrimaryState::InMultiLineComment => '$',
//            PrimaryState::InMultiLineAfterAsterisk => '%'
        });
    }

    return output;
}

struct DiagCtx {
    out: RefCell<Box<dyn termcolor::WriteColor>>,
    cfg: codespan_reporting::term::Config,
    file_db: RefCell<codespan::Files>,
    file_id: codespan::FileId,
}

impl DiagCtx {
    fn new(file_name: &str, file_content: &str) -> DiagCtx {
        let out = Box::new(termcolor::StandardStream::stderr(termcolor::ColorChoice::Always));
        let cfg = codespan_reporting::term::Config::default();
        let mut file_db = codespan::Files::default();
        let file_id = file_db.add(file_name, file_content);
        DiagCtx{out: RefCell::new(out), cfg, file_db: RefCell::new(file_db), file_id}
    }
}

fn render_syntax_error(ctx: DiagCtx, err: ParseError) -> String
{
    let (tb, te) = match err {
        ParseError::InvalidToken{location: l} => (l, l),
        ParseError::UnrecognizedEOF{location: l, ..} => (l, l),
        ParseError::UnrecognizedToken{token: (b, .., e), ..} => (b, e),
        ParseError::ExtraToken{token: (b, .., e)} => (b, e),
        _ => panic!("unexpected parser error")
    };


    let label = Label::new(ctx.file_id, codespan::Span::new(tb as u32, te as u32), "tomdidomdidom");
    let diag = Diagnostic::new_error("zjebalo sie", label);
    codespan_reporting::term::emit(ctx.out.borrow_mut().as_mut(), &ctx.cfg, &ctx.file_db.borrow(), &diag);

//    let (tb, te) = (tb as i32, te as i32);
//
//    let mut lb: i32 = 0;
//    let mut le: i32 = -1;
//    for (i, line) in source.lines().enumerate() {
//        lb = le + 1; // Assuming LF
//        le = lb + line.bytes().count() as i32;
//        if lb <= tb && te < le {
//            println!("{}: {} ({})", lb, line, le);
//        }
//    }
//
    return "".to_owned();
}

fn run() -> Result<(), CompilerError> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        return Err(CompilerError::InvalidArgument)
    }
    let path = Box::from(Path::new(&args[1]));

    let source = match fs::read_to_string(&path) {
        Err(_) => return Err(CompilerError::FileNotFound{path}),
        Ok(v) => v
    };

    let stripped = remove_comments(&source);
    let diag_ctx = DiagCtx::new(path.to_str().unwrap(), &source);

    let ast = match latte::GProgramParser::new().parse(&stripped) {
        Err(e) => {
            let msg = render_syntax_error(diag_ctx, e);
            return Err(CompilerError::SyntaxError{msg});
        },
        Ok(v) => v
    };

    println!("{:?}", ast);

    Ok(())
}

fn main() {
    match run() {
        Err(e) => {
            println!("ERROR");
            println!();
            println!("{:?}", e); // TODO
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
//      "./lattests/good/core001.lat",
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
        let input = fs::read_to_string(Path::new(item)).expect("test file not found");
        let code = remove_comments(&input);
        for (idx, line) in code.split('\n').enumerate() {
            println!("{}: {}", idx, line);
        }
        let result = latte::GProgramParser::new().parse(&code);
        assert!(result.is_ok(), "{}: {}", item, result.unwrap_err());
    }
}