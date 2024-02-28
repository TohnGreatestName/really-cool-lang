use std::{fmt::Display, iter::Enumerate, str::Chars};

use thiserror::Error;

use self::matchers::AnyChar;

pub trait CharMatcher {
    fn dynamic() -> &'static dyn CharMatcher where Self: Sized;
    fn is_match(&self, c: char) -> std::result::Result<(), String>;
}


pub mod matchers {
    use super::CharMatcher;

    #[derive(Clone, Copy)]
    pub struct NumericChar;
    impl CharMatcher for NumericChar {

        fn is_match(&self, c: char) -> std::result::Result<(), String> {
            if c.is_numeric() {
                Ok(())
            } else {
                Err(format!("got non-numeric character {c}"))
            }
        }


        fn dynamic() -> &'static dyn CharMatcher {
            Self::VALUE
        }
    }
    impl NumericChar {
        pub const VALUE: &'static dyn CharMatcher = &Self;
    }


    #[derive(Clone, Copy)]
    pub struct AnyChar;
    impl CharMatcher for AnyChar {

        fn is_match(&self, _: char) -> std::result::Result<(), String> {
            Ok(())
        }


        fn dynamic() -> &'static dyn CharMatcher {
            Self::VALUE
        }
    }
    impl AnyChar {
        pub const VALUE: &'static dyn CharMatcher = &Self;
    }

    #[derive(Clone, Copy)]
    pub struct SpecificChar<const C: char>;
    impl<const C: char> CharMatcher for SpecificChar<C> {
        fn is_match(&self, c: char) -> std::result::Result<(), String> {
            if c == C {
                Ok(())
            } else {
                Err(format!("got {} but expected {}", c, C))
            }
        }

        fn dynamic() -> &'static dyn CharMatcher {
            Self::VALUE
        }
    }

    impl<const C: char> SpecificChar<C> {
        pub const VALUE: &'static dyn CharMatcher = &Self;
    }
}


pub enum LexerType {
    UntilEof,
    UntilEnd(&'static dyn CharMatcher),
}

type IndexedCharIter<'a> = Enumerate<Chars<'a>>;
#[derive(Debug, Clone, Copy)]
enum PeekState {
    Present(usize, char),
    Eof(usize),
}

impl PeekState {
    fn err(&self) -> LexerError {
        match self {
            Self::Present(_, _) => panic!("Not an error value"),
            Self::Eof(v) => LexerError::eof(*v),
        }
    }
}

pub struct LexerStream<'a> {
    chars: IndexedCharIter<'a>,
    peek: PeekState,
    ty: LexerType,
}

impl<'a> LexerStream<'a> {
    pub fn new(mut chars: IndexedCharIter<'a>) -> Self {
        let peek = match chars.next() {
            Some((idx, char)) => PeekState::Present(idx, char),
            None => PeekState::Eof(0),
        };
        Self {
            chars,
            peek,
            ty: LexerType::UntilEof,
        }
    }

    pub fn eat_until<C: CharMatcher>(&mut self) -> LexerResult<LexerStream<'a>> {
        let new_lexer = LexerStream {
            chars: self.chars.clone(),
            peek: self.peek.clone(),
            ty: LexerType::UntilEnd(C::dynamic()),
        };
        while C::dynamic().is_match(self.advance::<AnyChar>()?).is_err() {}
        Ok(new_lexer)
    }

    pub fn peek(&self) -> LexerResult<(usize, char)> {
        let PeekState::Present(idx, char) = self.peek else {
            return Err(self.peek.err());
        };


        Ok((idx, char))
    }

    pub fn advance<C: CharMatcher>(&mut self) -> LexerResult<char> {
        let (idx, c) = self.peek()?;

        if let LexerType::UntilEnd(comp) = &self.ty {
            if comp.is_match(c).is_ok() {
                return Err(LexerError::eof(idx));
            }
        }

        if let Err(s) = C::dynamic().is_match(c) {
            return Err(LexerError::incorrect_char(Some((idx, c)), s))
        }


        self.peek = match self.chars.next() {
            Some((idx, char)) => PeekState::Present(idx, char),
            None => PeekState::Eof(idx + 1),
        };

        Ok(c)
    }
}

pub type LexerResult<T> = std::result::Result<T, LexerError>;

#[derive(Debug, Error)]
pub struct LexerError {
    err: LexerErrorType,
    position: usize,
}

impl LexerError {
    pub fn is_eof(&self) -> bool {
        matches!(self.err, LexerErrorType::EOF)
    }
    
    pub fn eof(position: usize) -> Self {
        Self {
            err: LexerErrorType::EOF,
            position,
        }
    }

    pub fn incorrect_char(got: Option<(usize, char)>, expected: String) -> Self {
        let position = if let Some((idx, _)) = got {
            idx
        } else {
            0 // TODO - position tracking in the stream if we encounter EOF
        };
        Self {
            err: LexerErrorType::IncorrectChar(got.map(|v| v.1), expected),
            position,
        }
    }
}

impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@[{}]: {}", self.position, self.err)
    }
}

#[derive(Debug, Error)]
pub enum LexerErrorType {
    #[error("got {0:?} but expected {1}")]
    IncorrectChar(Option<char>, String),
    #[error("encountered EOF")]
    EOF,
}

#[cfg(test)]
mod tests {
    use crate::syntax::lexer::matchers::{AnyChar, SpecificChar};

    use super::LexerStream;

    #[test]
    fn parenthesis() {
        let mut v = LexerStream::new("(abcd)(bcda)".chars().enumerate());
        let expected = ['a', 'b', 'c', 'd', 'b', 'c', 'd', 'a'];
        let mut received = vec![];

        while v.advance::<SpecificChar<'('>>().is_ok() {
            let mut second_stream = v.eat_until::<SpecificChar<')'>>().unwrap();

            while let Ok(v) = second_stream.advance::<AnyChar>() {
                received.push(v);
            }
        }

        assert_eq!(received, expected)
    }
}
