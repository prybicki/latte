pub mod ast;
#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(pub latte); // synthesized by LALRPOP

fn main() {
    let parser = latte::GExprRelParser::new();
    loop {
        let mut text = String::new();
        std::io::stdin().read_line(&mut text).unwrap();
        let ast = parser.parse(&text);
        println!("{:?}", ast)
//        let ast = text[1..text.len()-1].replace("\\\"", "\"");
//        println!("{}", ast);
    }
}
