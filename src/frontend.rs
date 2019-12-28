use crate::ast;
use crate::diag;

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

pub fn do_semantic_check(ast: &ast::Program) -> Vec<diag::Diagnostic> {
    let mut diags = Vec::new();
    let ast::Program(functions) = ast;

    // Check if main exists
    let main = functions.iter().find(|ast::TopDef(t, n, a, _)| *t == ast::Type::Int && n == "main" && a.len() == 0);
    if let None = main {
        diags.push(diag::gen_invalid_main());
    }

    return diags;
}