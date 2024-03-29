use std::io::{stdin, stdout, BufRead, Write};

use syntax::{
    ast::{Node, ParseErrorType, Parseable},
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
struct Number(f64);
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

        let mut seen_dot = false;
        loop {
            if matches!(state.lexer().peek(), Ok((_, '.'))) {
                if !seen_dot {
                    state.lexer().eat::<'.'>()?;
                    chars.push('.');
                    seen_dot = true;
                } else {
                    return Err(state.err(ParseErrorType::ExtraDotInNumberLiteral));
                }
            }

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

        let mut val = chars.parse::<f64>().unwrap();
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
    pub fn evaluate(&self) -> f64 {
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

        match state.lexer().peek() {
            Ok((_, '*')) => {
                state.lexer().eat::<'*'>()?;
                let num_two = state.parse()?;
                Ok(state.node(Self::Mul(num, num_two)))
            }
            Ok((_, '/')) => {
                state.lexer().eat::<'/'>()?;
                let num_two = state.parse()?;
                Ok(state.node(Self::Div(num, num_two)))
            }
            _ => Ok(num),
        }
    }
}
#[derive(Debug)]
enum Term {
    Val(Factor),
    Add(Node<Term>, Node<Term>),
    Sub(Node<Term>, Node<Term>),
}
impl Term {
    pub fn evaluate(&self) -> f64 {
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

        match state.lexer().peek() {
            Ok((_, '+')) => {
                state.lexer().eat::<'+'>()?;
                let num_two = state.parse()?;
                Ok(state.node(Self::Add(num, num_two)))
            }
            Ok((_, '-')) => {
                state.lexer().eat::<'-'>()?;
                let num_two = state.parse()?;
                Ok(state.node(Self::Sub(num, num_two)))
            }
            _ => Ok(num),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{syntax::ast::Parser, Term};

    #[test]
    fn precedence_test() {
        let mut parser = Parser::new("1+2*5");
        assert_eq!(parser.parse::<Term>().unwrap().evaluate(), 11.0);
    }

    #[test]
    fn long_expr() {
        let mut parser = Parser::new("2/3*9");
        assert_eq!(parser.parse::<Term>().unwrap().evaluate(), 6.0);
    }
}

fn main() -> std::result::Result<(), LexerError> {
    let mut input = String::new();

    let mut stdin = stdin().lock();
    let mut stdout = stdout().lock();
    loop {
        stdout.write(b"> ").unwrap();
        stdout.flush().unwrap();
        stdin.read_line(&mut input).unwrap();

        if input.is_empty() {
            println!();
            break;
        }

        let mut parser = Parser::new(input.trim());
        match parser.parse::<Term>() {
            Ok(v) => {
                if !parser.lexer().is_finished() {
                    eprintln!("Trailing data @ {}", parser.lexer().position());
                } else {
                    println!("{}", v.evaluate());
                }
            }
            Err(e) => eprintln!("Err: {}", e),
        }
        input.clear();
    }

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
