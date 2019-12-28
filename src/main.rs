
pub mod ast;
pub mod diag;
pub mod frontend;

#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(pub latte);
pub type ParseError<'i> = lalrpop_util::ParseError<usize, latte::Token<'i>, &'static str>;

use std::fs;
use std::io;
use std::path::Path;

pub struct File {
    file_db: codespan::Files, // works only for single file
    file_id: codespan::FileId, // codespan is broken
}

impl File {
    pub fn new(name: &str) -> Result<File, io::Error> {
        let content = fs::read_to_string(Path::new(name))?;
        let mut file_db = codespan::Files::default();
        let file_id = file_db.add(name, content);
        Ok(File {file_db, file_id})
    }

    pub fn get_content(&self) -> &str { self.file_db.source(self.file_id) }
}

fn compile(file: &File) -> Result<(), Vec<diag::Diagnostic>> {
    let stripped = frontend::remove_comments(file.get_content());

    let mut ast = match latte::GProgramParser::new().parse(&stripped) {
        Err(e) => return Err(vec![diag::gen_from_parse_error(e)]),
        Ok(v) => v
    };

    let diags = frontend::do_semantic_check(&mut ast);
    if !diags.is_empty() {
        return Err(diags);
    }

    return Ok(())
}

fn main() {
    fn die(msg: &str) -> ! {
        eprintln!("ERROR\n");
        eprintln!("{}", msg);
        std::process::exit(1);
    }
    let args: Vec<String> = std::env::args().collect();

    let path = args.get(1)
        .unwrap_or_else(|| die(&format!("expected 1 argument, got {}", args.len()-1)));

    let file = File::new(path)
        .unwrap_or_else(|e| die(&format!("error while reading file: {}", e)));

    match compile(&file) {
        Err(diags) => {
            eprintln!("ERROR\n");
            diag::print_all(&diags, &file);
            std::process::exit(1);
        },
        Ok(_) => {
            eprintln!("OK");
            std::process::exit(0);
        }
    }
}


fn test_case(path: &str, expect_success: bool) -> bool {
    eprint!("{} => ", path);
    let success: bool;
    let file = File::new(path).unwrap();
    let result = compile(&file);
    match result {
        Err(_) => success = !expect_success,
        Ok(_) => success = expect_success,
    }
    eprintln!("{}", if success { "OK" } else { "ERR" });
    if let Err(diags) = result {
        diag::print_all(&diags, &file);
    }
    return success;
}

#[test]
fn good() {
    let mut success = true;
    for i in 1..=22 {
        let path = format!("./lattests/good/core{:03}.lat", i);
        success &= test_case(&path, true);
    }
    assert!(success);
}

#[test]
fn bad() {
    let mut success = true;
    for i in 1..=27 {
        if i == 14 {continue}
        let path = format!("./lattests/bad/bad{:03}.lat", i);
        success &= test_case(&path, false);
    }
    assert!(success);
}



//struct Diagnostic {
//    message: &'static str,
//    highlight: Option<(u32, u32, &'static str)>,
//}
//
//enum TranslationUnitError {
//    InvalidArgs,
//    InvalidFile(io::Error),
//    InvalidSource(Vec<Diagnostic>),
//}
//
//fn pretty_print_error(err: TranslationUnitError) {
//    match err {
//        TranslationUnitError::InvalidArgs => eprintln!("invalid number of arguments"),
//        TranslationUnitError::InvalidFile(e) => eprintln!("invalid file: {}", e),
//        TranslationUnitError::InvalidSource(errs) => {
//            let stream =  termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
//
//        }
//    }
//}


//fn do_semantic_analysis(ctx: &Context, ast: &mut ast::Program) -> Vec<Diagnostic> {

//}
