use std::{fmt::Display, iter::Enumerate, str::Chars};

use thiserror::Error;

pub enum LexerType {
    UntilEof,
    UntilEnd(char),
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

    pub fn eat_until(&mut self, c: char) -> LexerResult<LexerStream<'a>> {
        let new_lexer = LexerStream {
            chars: self.chars.clone(),
            peek: self.peek.clone(),
            ty: LexerType::UntilEnd(c),
        };
        while self.peek(None)?.1 != c {
            self.advance(None)?;
        }
        Ok(new_lexer)
    }

    pub fn peek(&self, comparison: Option<char>) -> LexerResult<(usize, char)> {
        let PeekState::Present(idx, char) = self.peek else {
            return Err(self.peek.err());
        };

        if let Some(comparison) = comparison {
            if char != comparison {
                return Err(LexerError::incorrect_char(Some((idx, char)), comparison));
            }
        }

        Ok((idx, char))
    }

    pub fn advance(&mut self, comparison: Option<char>) -> LexerResult<char> {
        let (idx, c) = self.peek(comparison)?;
        self.peek = match self.chars.next() {
            Some((idx, char)) => PeekState::Present(idx, char),
            None => PeekState::Eof(idx + 1),
        };
        if let LexerType::UntilEnd(comp) = &self.ty {
            if c == *comp {
                return Err(LexerError::eof(idx));
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
    pub fn eof(position: usize) -> Self {
        Self {
            err: LexerErrorType::EOF,
            position,
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
