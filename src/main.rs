use syntax::lexer::LexerError;

use crate::syntax::lexer::LexerStream;

mod syntax;

fn main() -> std::result::Result<(), LexerError> {
    let mut v = LexerStream::new("(abcd)(bcda)".chars().enumerate());

    while v.peek(Some('(')).is_ok() {
        v.advance(Some('('))?;
        let mut second_stream = v.eat_until(')')?;

        while let Ok(v) = second_stream.advance(None) {
            println!("value: {:?}", v);
        }
    }

    //println!("Hello, world! {:?}", v.peek(None)?);
    Ok(())
}
