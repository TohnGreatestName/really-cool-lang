use std::{fmt::Display, iter::Enumerate, str::Chars};

use thiserror::Error;

use self::matchers::{AnyChar, SpecificChar};

use super::ast::Span;

pub trait CharMatcher {
    fn dynamic() -> &'static dyn CharMatcher
    where
        Self: Sized;
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
#[derive(Clone, Copy, Debug, Default)]
pub struct CharIndex {
    pub line: usize,
    pub column: usize,
}

impl CharIndex {
    pub fn advance_num(&self, n: usize) -> Self {
        let mut clone = *self;
        clone.column += n;
        clone
    }

    pub fn advance(&self, c: char) -> Self {
        let mut clone = *self;
        if c == '\n' {
            clone.line += 1;
            clone.column = 0;
        } else {
            clone.column += 1;
        }
        clone
    }
}
#[derive(Clone, Copy, Debug)]
pub enum WhitespaceMode {
    Skip,
    Allow,
}

#[derive(Clone, Copy)]
pub enum LexerType {
    UntilEof,
    UntilEnd(&'static dyn CharMatcher),
}
#[derive(Clone)]
pub struct IndexedCharIter<'a> {
    chars: Chars<'a>,
    index: CharIndex,
    whitespace: WhitespaceMode,
}

impl<'a> Iterator for IndexedCharIter<'a> {
    type Item = (CharIndex, char);

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.chars.next()?;
        match self.whitespace {
            WhitespaceMode::Skip => {
                while next.is_whitespace() {
                    next = self.chars.next()?;
                }
            }
            _ => (),
        }
        let og_index = self.index;
        self.index = self.index.advance(next);
        Some((og_index, next))
    }
}

impl<'a> IndexedCharIter<'a> {
    pub fn index(&self) -> CharIndex {
        self.index
    }

    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            chars,
            index: Default::default(),
            whitespace: WhitespaceMode::Skip,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum PeekState {
    Present(CharIndex, char),
    Eof(CharIndex),
}

impl PeekState {
    fn err(&self) -> LexerError {
        match self {
            Self::Present(_, _) => panic!("Not an error value"),
            Self::Eof(v) => LexerError::eof(*v),
        }
    }
}
#[derive(Clone)]
pub struct LexerStream<'a> {
    chars: IndexedCharIter<'a>,
    peek: PeekState,
    ty: LexerType,
    start: CharIndex,
    end: CharIndex,
}

impl<'a> LexerStream<'a> {
    pub fn new(mut chars: IndexedCharIter<'a>) -> Self {
        let peek = match chars.next() {
            Some((idx, char)) => PeekState::Present(idx, char),
            None => PeekState::Eof(chars.index()),
        };
        Self {
            start: chars.index(),
            end: chars.index(),
            chars,
            peek,
            ty: LexerType::UntilEof,
        }
    }

    pub fn span(&self) -> Span {
        Span {
            start: self.start,
            end: self.end,
        }
    }

    pub fn eat_until<C: CharMatcher>(&mut self) -> LexerResult<LexerStream<'a>> {
        let new_lexer = LexerStream {
            chars: self.chars.clone(),
            peek: self.peek.clone(),
            ty: LexerType::UntilEnd(C::dynamic()),
            start: self.start,
            end: self.end,
        };
        while C::dynamic().is_match(self.advance::<AnyChar>()?).is_err() {}
        Ok(new_lexer)
    }

    pub fn peek(&self) -> LexerResult<(CharIndex, char)> {
        let PeekState::Present(idx, char) = self.peek else {
            return Err(self.peek.err());
        };

        Ok((idx, char))
    }

    pub fn eat<const C: char>(&mut self) -> LexerResult<char> {
        self.advance::<SpecificChar<C>>()
    }

    pub fn advance<C: CharMatcher>(&mut self) -> LexerResult<char> {
        let (idx, c) = self.peek()?;

        if let LexerType::UntilEnd(comp) = &self.ty {
            if comp.is_match(c).is_ok() {
                return Err(LexerError::eof(idx));
            }
        }

        if let Err(s) = C::dynamic().is_match(c) {
            return Err(LexerError::incorrect_char(Some(c), idx, s));
        }

        self.peek = match self.chars.next() {
            Some((idx, char)) => PeekState::Present(idx, char),
            None => PeekState::Eof(idx.advance_num(1)),
        };
        self.end = self.chars.index();

        Ok(c)
    }
}

pub type LexerResult<T> = std::result::Result<T, LexerError>;

#[derive(Debug, Error)]
pub struct LexerError {
    err: LexerErrorType,
    position: CharIndex,
}

impl LexerError {
    pub fn is_eof(&self) -> bool {
        matches!(self.err, LexerErrorType::EOF)
    }

    pub fn eof(position: CharIndex) -> Self {
        Self {
            err: LexerErrorType::EOF,
            position,
        }
    }

    pub fn incorrect_char(got: Option<char>, position: CharIndex, expected: String) -> Self {
        Self {
            err: LexerErrorType::IncorrectChar(got, expected),
            position,
        }
    }
}

impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@[{:?}]: {}", self.position, self.err)
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
    use crate::syntax::lexer::{
        matchers::{AnyChar, SpecificChar},
        IndexedCharIter,
    };

    use super::LexerStream;

    #[test]
    fn parenthesis() {
        let mut v = LexerStream::new(IndexedCharIter::new("(abcd)(bcda)".chars()));
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
