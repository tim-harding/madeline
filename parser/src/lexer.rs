use std::{
    collections::HashMap,
    iter::Peekable,
    str::{from_utf8_unchecked, CharIndices},
};

// TODO: SOA
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub start: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Eof,
    Ident,
    Float,
    ParenLeft,
    ParenRight,
    Fn,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Exclamation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Id(pub u32);

pub struct Lexer<'a> {
    pub line: u32,
    pub column: u32,
    input: &'a [u8],
    iter: Peekable<CharIndices<'a>>,
    identifiers: HashMap<&'a str, Id>,
}

impl<'a> Lexer<'a> {
    // TODO: Take byte slice instead to avoid a second pass
    pub fn new(s: &'a str) -> Self {
        Self {
            iter: s.char_indices().peekable(),
            input: s.as_bytes(),
            identifiers: HashMap::new(),
            line: 1,
            column: 1,
        }
    }

    pub fn token(&mut self) -> Result<Option<Token>, Error> {
        self.take_whitespace();
        let Some((start, c)) = self.take() else {
            return Ok(None);
        };
        let kind = match c {
            '(' => TokenKind::ParenLeft,
            ')' => TokenKind::ParenRight,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Asterisk,
            '/' => TokenKind::Slash,
            '!' => TokenKind::Exclamation,
            '_' | 'a'..='z' | 'A'..='Z' => self.ident_or_keyword(start),
            c => {
                return Err(self.error(ErrorKind::TokenStart(c)));
            }
        };
        Ok(Some(Token {
            start: c.try_into().unwrap(),
            kind,
        }))
    }

    fn float(&mut self) -> Result<TokenKind, Error> {
        // TODO: Support underscores
        // TODO: Support signed ints
        // TODO: Support floats
        loop {
            match self.peek() {
                Some((_, '0'..='9')) => {
                    self.take();
                    continue;
                }
                _ => break,
            };
        }
        Ok(TokenKind::Float)
    }

    fn ident_or_keyword(&mut self, start: usize) -> TokenKind {
        let s = loop {
            match self.peek() {
                Some((_, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9')) => {
                    self.take();
                    continue;
                }
                Some((end, _)) => break &self.input[start..end],
                None => break &self.input[start..],
            }
        };
        match unsafe { from_utf8_unchecked(s) } {
            "fn" => TokenKind::Fn,
            _ => TokenKind::Ident,
        }
    }

    fn error(&self, kind: ErrorKind) -> Error {
        Error {
            line: self.line,
            column: self.column,
            kind,
        }
    }

    fn take(&mut self) -> Option<(usize, char)> {
        self.column += 1;
        self.iter.next()
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        self.iter.peek().cloned()
    }

    fn take_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some((_, '\n')) => {
                    self.take();
                    self.line += 1;
                    self.column = 1;
                }
                Some((_, c)) if c.is_whitespace() => {
                    self.take();
                }
                _ => break,
            }
        }
    }
}

// TODO: Store line starts to accelerate file position lookups

/// Gets the line and column number of the token at the given byte offset.
///
/// SAFETY: token_start must index the start of a UTF-8 char
pub unsafe fn token_line_and_column(s: &str, token_start: usize) -> (u32, u32) {
    let s = &s.as_bytes()[..token_start];
    let s = unsafe { from_utf8_unchecked(s) };
    last_line_and_column(s)
}

pub fn last_line_and_column(s: &str) -> (u32, u32) {
    s.chars().fold((1, 1), |(line, column), c| match c {
        '\n' => (line + 1, 1),
        _ => (line, column + 1),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Lexer error at {line}:{column}:\n{kind}")]
pub struct Error {
    pub line: u32,
    pub column: u32,
    pub kind: ErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Unexpected {c} at {line}:{column}")]
pub enum ErrorKind {
    #[error("Unexpected {0} where a token was expected to start")]
    TokenStart(char),
}

#[cfg(test)]
mod tests {
    use super::TokenKind::*;
    use super::*;

    fn token_kinds(s: &str) -> Box<[TokenKind]> {
        let mut v = Vec::new();
        let mut lexer = Lexer::new(s);
        while let Ok(Some(token)) = lexer.token() {
            v.push(token.kind);
        }
        v.into_boxed_slice()
    }

    fn assert_tokens_match<const N: usize>(actual: &str, expected: [TokenKind; N]) {
        assert_eq!(token_kinds(actual).as_ref(), expected.as_slice(),)
    }

    #[test]
    fn parens() {
        assert_tokens_match(" ( ) ", [ParenLeft, ParenRight])
    }

    #[test]
    fn words() {
        assert_tokens_match(" hi hello fn ", [Ident, Ident, Fn])
    }
}
