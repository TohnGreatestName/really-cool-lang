use syntax::{
    ast::{Node, Parseable},
    lexer::{
        matchers::{AnyChar, NumericChar, SpecificChar},
        LexerError,
    },
};

use crate::syntax::{
    ast::Parser,
    lexer::{IndexedCharIter, LexerStream},
};

mod syntax;
#[derive(Debug)]
struct Number(u64);
impl Parseable for Number {
    fn parse<'a>(
        state: &mut syntax::ast::Parser<'a>,
    ) -> std::result::Result<syntax::ast::Node<Self>, syntax::ast::ParseError> {
        let mut chars = String::new();
        loop {
            match state.lexer().advance::<NumericChar>() {
                Ok(c) => chars.push(c),
                Err(e) => {
                    break;
                }
            }
        }
        Ok(Node::new(
            Number(chars.parse::<u64>().unwrap()),
            state.lexer().span(),
        ))
    }
}
#[derive(Debug)]
enum Expr {
    Val(Node<Number>),
    Add(Node<Number>, Node<Expr>),
}
impl Parseable for Expr {
    fn parse<'a>(
        state: &mut Parser<'a>,
    ) -> std::result::Result<Node<Self>, syntax::ast::ParseError> {
        let num = state.parse::<Number>()?;

        if matches!(state.lexer().peek(), Ok((_, '+'))) {
            state.lexer().advance::<SpecificChar<'+'>>()?;
            let num_two = state.parse::<Expr>()?;
            return Ok(Node::new(Self::Add(num, num_two), state.lexer().span()));
        } else {
            return Ok(Node::new(Self::Val(num), state.lexer().span()));
        }
    }
}

fn main() -> std::result::Result<(), LexerError> {
    let mut parser = Parser::new("1234+245");
    let v = parser.parse::<Expr>().unwrap();
    println!("Val: {:#?}", v);

    let mut v = LexerStream::new(IndexedCharIter::new("(1234)(2345)(22)".chars()));

    while v.advance::<SpecificChar<'('>>().is_ok() {
        let mut second_stream = v.eat_until::<SpecificChar<')'>>().unwrap();

        let mut chars = String::new();
        loop {
            match second_stream.advance::<NumericChar>() {
                Ok(c) => chars.push(c),
                Err(e) if e.is_eof() => break,
                Err(e) => return Err(e),
            }
        }
        println!("{}", chars);
    }

    println!("Hello WOrld");
    //println!("Hello, world! {:?}", v.peek(None)?);
    Ok(())
}
