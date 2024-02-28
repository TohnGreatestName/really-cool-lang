use std::io::stdin;

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
    Add(Node<Expr>, Node<Number>),
    Sub(Node<Expr>, Node<Number>),
}
impl Expr {
    pub fn evaluate(&self) -> u64 {
        match self {
            Expr::Val(v) => v.0,
            Expr::Add(a, b) => a.evaluate() + b.0,
            Expr::Sub(a, b) => a.evaluate() - b.0,
        }
    }
}

impl Parseable for Expr {
    fn parse<'a>(
        state: &mut Parser<'a>,
    ) -> std::result::Result<Node<Self>, syntax::ast::ParseError> {
        let num = state.parse::<Number>()?;
        let num_span = num.span();
        let num = Node::new(Expr::Val(num), num_span);

        let mut val: Node<Expr> = num;
        loop {
            val = match state.lexer().peek() {
                Ok((_, '+')) => {
                    state.lexer().advance::<SpecificChar<'+'>>()?;
                    let num_two = state.parse::<Number>()?;
                    Node::new(Self::Add(val, num_two), state.lexer().span())
                }
                Ok((_, '-')) => {
                    state.lexer().advance::<SpecificChar<'-'>>()?;
                    let num_two = state.parse::<Number>()?;
                    Node::new(Self::Sub(val, num_two), state.lexer().span())
                }
                _ => break,
            };
        }
        return Ok(val);
    }
}

fn main() -> std::result::Result<(), LexerError> {
    let mut input = String::new();

    stdin().read_line(&mut input).unwrap();

    let mut parser = Parser::new(input.trim());
    let v = parser.parse::<Expr>().unwrap();
    println!("Val: {:#?}", v.evaluate());

    // let mut v = LexerStream::new(IndexedCharIter::new("(1234)(2345)(22)".chars()));

    // while v.advance::<SpecificChar<'('>>().is_ok() {
    //     let mut second_stream = v.eat_until::<SpecificChar<')'>>().unwrap();

    //     let mut chars = String::new();
    //     loop {
    //         match second_stream.advance::<NumericChar>() {
    //             Ok(c) => chars.push(c),
    //             Err(e) if e.is_eof() => break,
    //             Err(e) => return Err(e),
    //         }
    //     }
    //     println!("{}", chars);
    // }

    // println!("Hello WOrld");
    //println!("Hello, world! {:?}", v.peek(None)?);
    Ok(())
}
