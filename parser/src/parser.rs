use crate::lexer::{self, Lexer, Token, TokenKind};

enum NodeKind {
    Sum,
    Float,
}

struct Node {
    kind: NodeKind,
    children: (u32, u32),
}

pub struct Ast {
    // TODO: SOA
    nodes: Vec<Node>,
}

impl Ast {
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    pub fn push(&mut self, node: Node) -> u32 {
        let i = self.nodes.len();
        self.nodes.push(node);
        i.try_into().unwrap()
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    ast: Ast,
}

impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            lexer: Lexer::new(s),
            ast: Ast::new(),
        }
    }

    pub fn parse(mut self) -> Result<Ast, Error> {
        self.literal()?;
        Ok(self.ast)
    }

    fn literal(&mut self) -> Result<u32, Error> {
        let token = self.token()?;
        match token.kind {
            _ => todo!(),
        }
    }

    fn token(&mut self) -> Result<Token, Error> {
        match self.lexer.token()? {
            Some(token) => Ok(token),
            None => Err(self.error(ErrorKind::Eof)),
        }
    }

    fn error(&self, kind: ErrorKind) -> Error {
        Error {
            line: self.lexer.line,
            column: self.lexer.column,
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErrorKind {
    #[error("Lexer error: {0}")]
    Lexer(#[from] lexer::ErrorKind),
    #[error("Unexpected EOF")]
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Lexer error at {line}:{column}:\n{kind}")]
pub struct Error {
    pub line: u32,
    pub column: u32,
    pub kind: ErrorKind,
}

impl From<lexer::Error> for Error {
    fn from(value: lexer::Error) -> Self {
        let lexer::Error { line, column, kind } = value;
        Self {
            line,
            column,
            kind: kind.into(),
        }
    }
}
