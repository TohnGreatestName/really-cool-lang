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
struct Number(i64);
impl Parseable for Number {
    fn parse<'a>(
        state: &mut syntax::ast::Parser<'a>,
    ) -> std::result::Result<syntax::ast::Node<Self>, syntax::ast::ParseError> {
        let mut chars = String::new();

        let mut negate = false;
        if matches!(state.lexer().peek(), Ok((_, '-'))) {
            state.lexer().eat::<'-'>()?;
            negate = true;
        }

        loop {
            match state.lexer().advance::<NumericChar>() {
                Ok(c) => chars.push(c),
                Err(e) => {
                    break;
                }
            }
        }
        if chars.is_empty() {
            return Err(state.err(syntax::ast::ParseErrorType::EmptyNumberLiteral));
        }

        let mut val = chars.parse::<i64>().unwrap();
        if negate {
            val = -val;
        }
        Ok(Node::new(Number(val), state.lexer().span()))
    }
}
#[derive(Debug)]
enum Factor {
    Val(Number),
    Parenthesis(Node<Term>),
    Mul(Node<Factor>, Node<Factor>),
    Div(Node<Factor>, Node<Factor>),
}
impl Factor {
    pub fn evaluate(&self) -> i64 {
        match self {
            Factor::Parenthesis(v) => v.evaluate(),
            Factor::Val(v) => v.0,
            Factor::Mul(a, b) => a.evaluate() * b.evaluate(),
            Factor::Div(a, b) => a.evaluate() / b.evaluate(),
        }
    }
}

impl Parseable for Factor {
    fn parse<'a>(
        state: &mut Parser<'a>,
    ) -> std::result::Result<Node<Self>, syntax::ast::ParseError> {
        let num = if let Ok((_, '(')) = state.lexer().peek() {
            state.lexer().eat::<'('>()?;
            let in_parens = state.lexer().eat_until::<SpecificChar<')'>>()?;
            Node::new(
                Self::Parenthesis(state.parse_with_lexer(in_parens)?),
                state.lexer().span(),
            )
        } else {
            let num = state.parse::<Number>()?;
            num.wrap(|v| Factor::Val(v))
        };

        let mut val: Node<Factor> = num;
        match state.lexer().peek() {
            Ok((_, '*')) => {
                state.lexer().eat::<'*'>()?;
                let num_two = state.parse()?;
                val = Node::new(Self::Mul(val, num_two), state.lexer().span())
            }
            Ok((_, '/')) => {
                state.lexer().eat::<'/'>()?;
                let num_two = state.parse()?;
                val = Node::new(Self::Div(val, num_two), state.lexer().span())
            }
            _ => (),
        };
        return Ok(val);
    }
}
#[derive(Debug)]
enum Term {
    Val(Factor),
    Add(Node<Term>, Node<Term>),
    Sub(Node<Term>, Node<Term>),
}
impl Term {
    pub fn evaluate(&self) -> i64 {
        match self {
            Term::Val(v) => v.evaluate(),
            Term::Add(a, b) => a.evaluate() + b.evaluate(),
            Term::Sub(a, b) => a.evaluate() - b.evaluate(),
        }
    }
}

impl Parseable for Term {
    fn parse<'a>(
        state: &mut Parser<'a>,
    ) -> std::result::Result<Node<Self>, syntax::ast::ParseError> {
        let num = state.parse::<Factor>()?;
        let num = num.wrap(|v| Term::Val(v));

        let mut val: Node<Term> = num;
        match state.lexer().peek() {
            Ok((_, '+')) => {
                state.lexer().eat::<'+'>()?;
                let num_two = state.parse()?;
                val = Node::new(Self::Add(val, num_two), state.lexer().span())
            }
            Ok((_, '-')) => {
                state.lexer().eat::<'-'>()?;
                let num_two = state.parse()?;
                val = Node::new(Self::Sub(val, num_two), state.lexer().span())
            }
            _ => (),
        };
        return Ok(val);
    }
}

fn main() -> std::result::Result<(), LexerError> {
    let mut input = String::new();

    stdin().read_line(&mut input).unwrap();

    let mut parser = Parser::new(input.trim());
    let v = parser.parse::<Term>().unwrap();
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
