use std::{
    collections::HashMap,
    iter::Peekable,
    str::{from_utf8_unchecked, CharIndices},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    start: u32,
    kind: TokenKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Ident(Id),
    ParenLeft,
    ParenRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Id(pub u32);

pub struct Lexer<'a> {
    line: u32,
    column: u32,
    s: &'a str,
    iter: Peekable<CharIndices<'a>>,
    identifiers: HashMap<&'a str, Id>,
}

impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            iter: s.char_indices().peekable(),
            s,
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
            '_' | 'a'..='z' | 'A'..='Z' => self.word(start),
            c => {
                return Err(Error {
                    line: self.line,
                    column: self.column,
                    c,
                })
            }
        };
        Ok(Some(Token {
            start: c.try_into().unwrap(),
            kind,
        }))
    }

    fn word(&mut self, start: usize) -> TokenKind {
        loop {
            let s = self.s.as_bytes();
            let s = match self.peek() {
                Some((_, '_' | 'a'..='z' | 'A'..='Z')) => {
                    self.take();
                    continue;
                }
                Some((end, _)) => &s[start..end],
                None => &s[start..],
            };
            let s = unsafe { from_utf8_unchecked(s) };
            let ident = Id(self.identifiers.len().try_into().unwrap());
            let ident = *self.identifiers.entry(s).or_insert(ident);
            break TokenKind::Ident(ident);
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

/// Gets the line and column number of the token at the given byte offset.
///
/// SAFETY: token_start must index the start of a UTF-8 char
pub unsafe fn token_line_and_column(s: &str, token_start: usize) -> (u32, u32) {
    let s = &s.as_bytes()[..token_start];
    let s = unsafe { from_utf8_unchecked(s) };
    s.chars().fold((1, 1), |(line, column), c| match c {
        '\n' => (line + 1, 1),
        _ => (line, column + 1),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Unexpected {c} at {line}:{column}")]
pub struct Error {
    line: u32,
    column: u32,
    c: char,
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
    fn idents() {
        assert_tokens_match(" hi hello ", [Ident(Id(0)), Ident(Id(1))])
    }
}
