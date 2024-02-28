use syntax::lexer::{matchers::{AnyChar, NumericChar, SpecificChar}, LexerError};

use crate::syntax::lexer::LexerStream;

mod syntax;

fn main() -> std::result::Result<(), LexerError> {
    let mut v = LexerStream::new("(1234)(2345)(22)".chars().enumerate());

    while v.advance::<SpecificChar<'('>>().is_ok() {
        let mut second_stream = v.eat_until::<SpecificChar<')'>>().unwrap();


        let mut chars = String::new();
        loop {
            match second_stream.advance::<NumericChar>() {
                Ok(c) => chars.push(c),
                Err(e) if e.is_eof() => break,
                Err(e) => return Err(e)
            }
        }
        println!("{}", chars);

    }

    println!("Hello WOrld");
    //println!("Hello, world! {:?}", v.peek(None)?);
    Ok(())
}
