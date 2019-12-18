
pub mod ast;
#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(pub latte); // synthesized by LALRPOP

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
            _ => ' ',
//            PrimaryState::InSingleLineComment => '@',
//            PrimaryState::AfterForwardSlash => '!',
//            PrimaryState::InMultiLineComment => '$',
//            PrimaryState::InMultiLineAfterAsterisk => '%'
        });
    }

    return output;
}

use std::fs;
fn main() {
//    let text = String::from(r#"/* Multiline with tricky " \"*/\" fake ending */"#);
    let text = fs::read_to_string("/home/prybicki/code/latte/lattests/comments.lat").unwrap();
    println!("{}", remove_comments(&text));

//    loop {
//        let mut input = String::new();
//        std::io::stdin().read_line(&mut input).unwrap();
//        let pattern = &input[0..input.len()-1];
//        let re = Regex::new(pattern).unwrap();
//        println!("{:?}", re);
//        println!("{}", if re.is_match(text) {"Matched"} else {"???"});
//        let out = re.replace_all(text, "@");
//        println!("{}", out);
//    }

}


//# Comment
//
//int main() {
//print("Hello world /*");
//}
//
//;;;
///* print("Hello world */");
//    */
//    // Another comment
//    let parser = latte::GProgramParser::new();
//    loop {
//        let mut text = String::new();
//        std::io::stdin().read_line(&mut text).unwrap();
//        let ast = parser.parse(&text);
//        println!("{:?}", ast.unwrap())
//        let ast = text[1..text.len()-1].replace("\\\"", "\"");
//        println!("{}", ast);
//    }
//fn remove_comments(text: &str) -> String {
////    enum State {
////        NoComment,
////        MultiLineComment
////
//    };
////    let state = State::NoComment;
//    let mut out = String::new();
//    let pos = 0;
//    loop {
////        let  = text[pos..len(text)].find("//");
//        let next_comment = text[pos..len(text)].find("#|//|/*|");
//        if let Some(n) = next_comment {
//            pos = n;
//        }
//    }
//
//    return out;
//}

//fn parse(text: &str) {
//    for ch in text.chars() {
//
//    }
//    let parser = latte::GProgramParser::new();
//}

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

    println!("kotek");
    let parser = latte::GProgramParser::new();
    for item in list.iter() {
        let text = fs::read_to_string(item).unwrap();
        let ast = parser.parse(&text);
        match ast {
            Ok(_) => println!("{} OK", item),
            Err(e) => println!("{} Error: {}", item, e),
        }
    }
}