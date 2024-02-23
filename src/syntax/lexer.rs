use std::str::Chars;

pub enum LexerType {
    UntilEof,
    UntilEnd(char),
}

pub struct LexerStream<'a> {
    chars: Chars<'a>,
}

