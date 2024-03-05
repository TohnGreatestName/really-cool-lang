use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use thiserror::Error;

use super::lexer::{CharIndex, IndexedCharIter, LexerError, LexerStream};
/// Represents a region of a file.
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: CharIndex,
    pub end: CharIndex,
}

impl Span {
    pub fn new(start: CharIndex, end: CharIndex) -> Self {
        Self { start, end }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Span({} to {})", self.start, self.end)
    }
}

/// Represents a node of the syntax tree. All nodes are
/// heap-allocated, might change this in the future.
pub struct Node<T> {
    value: Box<T>,
    span: Span,
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("value", &self.value)
            .field("span", &self.span)
            .finish()
    }
}

impl<T> Node<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self {
            value: Box::new(value),
            span,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn wrap<U>(self, f: fn(T) -> U) -> Node<U> {
        let span = self.span;
        let value = (f)(*self.value);
        Node::new(value, span)
    }
}

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.value
    }
}

pub trait Parseable: Sized {
    fn parse<'a>(state: &mut Parser<'a>) -> std::result::Result<Node<Self>, ParseError>;
}

/// Represents the state of the parser. Primarily a wrapper for a
/// changing `LexerStream` to allow nested parsing.
pub struct Parser<'a> {
    stream: LexerStream<'a>,
}
impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            stream: LexerStream::new(IndexedCharIter::new(s.chars())),
        }
    }

    pub fn parse_with_lexer<T: Parseable>(
        &mut self,
        lexer: LexerStream<'a>,
    ) -> std::result::Result<Node<T>, ParseError> {
        let current_lexer = std::mem::replace(&mut self.stream, lexer);
        let v = self.parse();
        let _ = std::mem::replace(&mut self.stream, current_lexer);
        v
    }

    pub fn parse<T: Parseable>(&mut self) -> std::result::Result<Node<T>, ParseError> {
        let stream = self.stream.clone();
        match T::parse(self) {
            Ok(v) => Ok(v),
            Err(e) => {
                self.stream = stream;
                Err(e)
            }
        }
    }

    pub fn lexer(&mut self) -> &mut LexerStream<'a> {
        &mut self.stream
    }

    pub fn err(&self, e: ParseErrorType) -> ParseError {
        ParseError {
            span: self.stream.span(),
            ty: e,
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseErrorType {
    #[error("lexer: {0}")]
    LexerError(#[from] LexerError),
    #[error("given empty number literal")]
    EmptyNumberLiteral,
    #[error("extra dot in number literal")]
    ExtraDotInNumberLiteral,
}

#[derive(Debug, Error)]
pub struct ParseError {
    pub span: Span,
    pub ty: ParseErrorType,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}", self.ty, self.span)
    }
}

impl From<LexerError> for ParseError {
    fn from(value: LexerError) -> Self {
        Self {
            span: Span::new(value.position(), value.position()),
            ty: value.into(),
        }
    }
}
