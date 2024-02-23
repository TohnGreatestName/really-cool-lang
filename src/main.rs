use syntax::lexer::LexerError;

use crate::syntax::lexer::LexerStream;

mod syntax;

fn main() -> std::result::Result<(), LexerError> {
    let mut v = LexerStream::new("abed".chars().enumerate());

    v.advance(Some('a'))?;
    v.advance(Some('b'))?;
    v.advance(Some('c'))?;
    v.advance(Some('d'))?;
    println!("Hello, world!");
    Ok(())
}
