use lexer::{Lexer, Token};

mod lexer;
mod parser;

pub fn parse(s: &str) -> Result<(), Error> {
    let tokens = {
        let mut tokens = vec![];
        let mut lexer = Lexer::new(s);
        while let Some(token) = lexer.token()? {
            tokens.push(token);
        }
        tokens.push(Token {
            start: s.len().try_into().unwrap(),
            kind: lexer::TokenKind::Eof,
        });
        tokens
    };
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Lexer(#[from] lexer::Error),
}
