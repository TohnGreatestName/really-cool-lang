use std::{fmt::Display, iter::Enumerate, str::Chars};

use thiserror::Error;

pub enum LexerType {
    UntilEof,
    UntilEnd(char),
}

type IndexedCharIter<'a> = Enumerate<Chars<'a>>;

pub struct LexerStream<'a> {
    chars: IndexedCharIter<'a>,
    peek: Option<(usize, char)>,
    ty: LexerType,
}

impl<'a> LexerStream<'a> {
    pub fn new(mut chars: IndexedCharIter<'a>) -> Self {
        let peek = chars.next();
        Self {
            chars,
            peek,
            ty: LexerType::UntilEof,
        }
    }

    pub fn eat_until(&mut self, c: char) -> LexerResult<LexerStream<'a>> {
        let new_lexer = LexerStream {
            chars: self.chars.clone(),
            peek: self.peek.clone(),
            ty: LexerType::UntilEnd(c),
        };
        while self.peek(None)? != c {
            self.advance(None)?;
        }
        Ok(new_lexer)
    }

    pub fn peek(&self, comparison: Option<char>) -> LexerResult<char> {
        if self.peek.is_none() {
            return Err(LexerError::eof());
        }
        if let Some(comparison) = comparison {
            if self.peek.map(|v| v.1) != Some(comparison) {
                return Err(LexerError::incorrect_char(self.peek, comparison));
            }
        }

        if let Some(current) = self.peek {
            Ok(current.1)
        } else {
            Err(LexerError::eof())
        }
    }

    pub fn advance(&mut self, comparison: Option<char>) -> LexerResult<char> {
        let c = self.peek(comparison)?;
        self.peek = self.chars.next();
        if let LexerType::UntilEnd(comp) = &self.ty {
            if c == *comp {
                return Err(LexerError::eof());
            }
        }
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
    pub fn eof() -> Self {
        Self {
            err: LexerErrorType::EOF,
            position: 0, // TODO - "Enumerate" iterator helper does not let us access the current position without
                         // advancing the iterator. Will fix this later
        }
    }

    pub fn incorrect_char(got: Option<(usize, char)>, expected: char) -> Self {
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
    IncorrectChar(Option<char>, char),
    #[error("encountered EOF")]
    EOF,
}
