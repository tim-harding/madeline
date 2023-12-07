use std::{collections::HashMap, iter::Peekable, str::CharIndices};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    start: u32,
    kind: TokenKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Ident,
    Fn,
    ParenLeft,
    ParenRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentifierId(pub u32);

pub struct Lexer<'a> {
    s: &'a str,
    iter: Peekable<CharIndices<'a>>,
    identifiers: HashMap<String, IdentifierId>,
}

impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            iter: s.char_indices().peekable(),
            s,
            identifiers: HashMap::new(),
        }
    }

    pub fn token(&mut self) -> Result<Option<Token>, Error> {
        self.take_whitespace();
        let Some(next) = self.take() else {
            return Ok(None);
        };
        let kind = match next.1 {
            '(' => Ok(TokenKind::ParenLeft),
            ')' => Ok(TokenKind::ParenRight),
            c => Err(Error::Unexpected(c)),
        }?;
        Ok(Some(Token {
            start: next.0.try_into().unwrap(),
            kind,
        }))
    }

    fn take(&mut self) -> Option<(usize, char)> {
        self.iter.next()
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        self.iter.peek().cloned()
    }

    fn take_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some((_, c)) if c.is_whitespace() => {
                    self.take();
                }
                _ => break,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Unexpected {0} while lexing")]
    Unexpected(char),
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
    fn left_paren() {
        assert_tokens_match(" ( ) ", [ParenLeft, ParenRight])
    }
}
