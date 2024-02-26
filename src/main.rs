use syntax::lexer::LexerError;

use crate::syntax::lexer::LexerStream;

mod syntax;

fn main() -> std::result::Result<(), LexerError> {
    let mut v = LexerStream::new("abed".chars().enumerate());

    let mut second_stream = v.eat_until('e')?;
    second_stream.advance(Some('a'))?;
    second_stream.advance(Some('b'))?;
    v.advance(Some('e'))?;
    v.advance(Some('d'))?;
    println!("Hello, world!");
    Ok(())
}
