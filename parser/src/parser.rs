use crate::lexer::{Token, TokenKind};
use std::{
    iter::{Enumerate, Peekable},
    slice::Iter,
};

pub enum NodeKind {
    Ident,
    Float,
    Sum,
    Difference,
    Product,
    Negation,
    Not,
    Quotient,
}

// TODO: Better to separately store leaves, unaries, and binaries?
// TODO: Is node kind needed or can we reuse the token?
// TODO: SOA
pub struct Node {
    kind: NodeKind,
    token_index: u32,
    children: (u32, u32),
}

pub struct Parser<'a> {
    tokens: Peekable<Enumerate<Iter<'a, Token>>>,
    nodes: Vec<Node>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            nodes: vec![],
            tokens: tokens.into_iter().enumerate().peekable(),
        }
    }

    fn unary(&mut self) -> Result<u32, Error> {
        match self.peek_token() {
            TokenKind::Minus => {
                let lhs = self.unary()?;
                Ok(self.accept_unary(NodeKind::Difference, lhs))
            }
            TokenKind::Exclamation => {
                let child = self.unary()?;
                Ok(self.accept_unary(NodeKind::Not, child))
            }
            // TODO: Add context to errors
            _ => self.primary(),
        }
    }

    /// float | ident | "(" expression ")"
    fn primary(&mut self) -> Result<u32, Error> {
        match self.take_token() {
            TokenKind::Float => Ok(self.accept_leaf(NodeKind::Float)),
            TokenKind::Ident => Ok(self.accept_leaf(NodeKind::Ident)),
            _ => Err(self.error(ErrorKind::Primary)),
        }
    }

    fn take_token(&mut self) -> TokenKind {
        self.tokens
            .next()
            .map(|(_, token)| token.kind)
            .unwrap_or(TokenKind::Eof)
    }

    fn peek_token(&mut self) -> TokenKind {
        self.tokens
            .peek()
            .map(|(_, token)| token.kind)
            .unwrap_or(TokenKind::Eof)
    }

    fn token_index(&mut self) -> u32 {
        let &(i, _) = self.tokens.peek().unwrap();
        i.try_into().unwrap()
    }

    fn push(&mut self, node: Node) -> u32 {
        let i = self.nodes.len();
        self.nodes.push(node);
        i.try_into().unwrap()
    }

    fn accept_binary(&mut self, kind: NodeKind, lhs: u32, rhs: u32) -> u32 {
        let token_index = self.token_index();
        self.take_token();
        self.push(Node {
            kind,
            token_index,
            children: (lhs, rhs),
        })
    }

    fn accept_unary(&mut self, kind: NodeKind, child: u32) -> u32 {
        self.accept_binary(kind, child, 0)
    }

    fn accept_leaf(&mut self, kind: NodeKind) -> u32 {
        self.accept_unary(kind, 0)
    }

    fn error(&mut self, kind: ErrorKind) -> Error {
        Error {
            token_index: self.token_index(),
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Primary,
    Unary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub token_index: u32,
    pub kind: ErrorKind,
}
