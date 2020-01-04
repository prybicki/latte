pub mod ast;
pub mod diag;
pub mod frontend;
pub mod backend;

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

fn process(file: &File) -> Result<(), Vec<diag::Diagnostic>> {
    let stripped = frontend::remove_comments(file.get_content());

    let mut ast = match latte::GProgramParser::new().parse(&stripped) {
        Err(e) => return Err(vec![diag::gen_from_parse_error(e)]),
        Ok(v) => v
    };

    println!("{:?}", ast);

    let diags = frontend::verify_program(&mut ast);
    if !diags.is_empty() {
        return Err(diags);
    }

    let llvm_ir = backend::compile(&ast);

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

    match process(&file) {
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
    let result = process(&file);
    match result {
        Err(_) => success = !expect_success,
        Ok(_) => success = expect_success,
    }
    eprintln!("{}", if success { "OK" } else { "ERR" });
    if let Err(_) = result {
//        diag::print_all(&diags, &file);
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

// TODO open