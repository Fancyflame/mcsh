use nom::error::{Error as NomError, ErrorKind, ParseError};

use super::lexer::{Lexer, Token};

#[derive(Debug, thiserror::Error)]
pub enum McshError<'a> {
    #[error("expected {expected}, found {found}")]
    Token { expected: &'a str, found: Token<'a> },

    #[error("{0}")]
    Nom(NomError<Lexer<'a>>),
}

impl<'a> ParseError<Lexer<'a>> for McshError<'a> {
    fn from_error_kind(input: Lexer<'a>, kind: ErrorKind) -> Self {
        McshError::Nom(NomError::new(input, kind))
    }

    fn append(input: Lexer<'a>, kind: ErrorKind, other: Self) -> Self {
        match other {
            Self::Nom(nerr) => Self::Nom(NomError::append(input, kind, nerr)),
            Self::Token { .. } => other,
        }
    }
}

impl<'a> McshError<'a> {
    pub fn token(expected: &'a str, found: Token<'a>) -> Self {
        Self::Token { expected, found }
    }
}
