use std::{
    cell::Cell,
    fmt::{Display, Write},
    rc::Rc,
};

use anyhow::anyhow;
use nom::{
    combinator::{eof, value, verify},
    sequence::terminated,
    InputLength,
};

pub use self::parse::*;

use super::{IResult, McshError};

mod parse;

#[derive(Clone, Debug)]
pub struct Lexer<'a> {
    tokens: Rc<[Token<'a>]>,
    cursor: Cell<usize>,
}

impl InputLength for Lexer<'_> {
    fn input_len(&self) -> usize {
        self.tokens.len() - self.cursor.get()
    }
}

impl Display for Lexer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for token in self.tokens.get(self.cursor.get()..).unwrap_or_default() {
            if first {
                token.fmt(f)?;
                first = false;
            } else {
                write!(f, " {token}")?;
            }
        }
        Ok(())
    }
}

impl<'a> Lexer<'a> {
    pub fn parse(input: &'a str) -> anyhow::Result<Self> {
        let result = nom::combinator::map(terminated(parse_tokens, eof), |tokens| Self {
            tokens,
            cursor: Default::default(),
        })(input);

        match result {
            Ok((_, lexer)) => Ok(lexer),
            Err(err) => Err(anyhow!("{err}")),
        }
    }

    pub fn peek(&self) -> &Token<'a> {
        self.tokens.get(self.cursor.get()).unwrap_or(&Token::Eof)
    }

    pub fn step(&self, length: usize) {
        self.cursor
            .set(self.tokens.len().min(self.cursor.get() + length))
    }

    pub fn print_err(&self) -> anyhow::Error {
        let index = self.cursor.get();
        let len = self.tokens.len();
        let mut output = String::new();

        let before = &self.tokens[index.checked_sub(10).unwrap_or(0)..index];
        let after = &self.tokens[(index + 1).min(len)..(index + 11).min(len)];

        for t in before {
            write!(output, "{t} ").unwrap();
        }

        output.push_str(">>>>>> ");
        if let Some(t) = self.tokens.get(index) {
            write!(output, "{t}").unwrap();
        }
        output.push_str(" <<<<<<");

        for t in after {
            write!(output, " {t}").unwrap();
        }

        anyhow!("{output}")
    }
}

impl<'a> From<Rc<[Token<'a>]>> for Lexer<'a> {
    fn from(value: Rc<[Token<'a>]>) -> Self {
        Lexer {
            tokens: value,
            cursor: Default::default(),
        }
    }
}

pub fn keyword<'a>(kw: &'a str) -> impl Fn(Lexer<'a>) -> IResult<'a, ()> {
    move |input| {
        let p = input.peek();
        if let Token::Ident(ident) = p {
            if *ident == kw {
                input.step(1);
                return Ok((input, ()));
            }
        }
        error(kw, p)
    }
}

pub fn group(delimiter: Delimiter) -> impl Fn(Lexer) -> IResult<Lexer> {
    move |input| {
        let p = input.peek();
        match p {
            Token::Group(group) if group.delimiter == delimiter => {
                input.step(1);
                let inside = Lexer {
                    tokens: group.content.clone(),
                    cursor: Cell::new(0),
                };
                Ok((input, inside))
            }
            _ => error(delimiter.display(), p),
        }
    }
}

pub fn ident(input: Lexer) -> IResult<&str> {
    let p = input.peek();
    if let &Token::Ident(ident) = p {
        input.step(1);
        Ok((input, ident))
    } else {
        error("identifier", p)
    }
}

pub fn punct(input: Lexer) -> IResult<Punct> {
    let p = input.peek();
    if let &Token::Punct(punct) = p {
        input.step(1);
        Ok((input, punct))
    } else {
        error("punctuation", p)
    }
}

pub fn specified_punct<'a>(expect: Punct) -> impl FnMut(Lexer<'a>) -> IResult<'a, Punct> {
    value(expect, verify(punct, move |p| *p == expect))
}

pub fn integer(input: Lexer) -> IResult<i32> {
    let p = input.peek();
    if let &Token::Literal(Literal::Int(int)) = p {
        input.step(1);
        Ok((input, int))
    } else {
        error("integer", p)
    }
}

pub fn string(input: Lexer) -> IResult<&str> {
    let p = input.peek();
    if let &Token::Literal(Literal::Str(s)) = p {
        input.step(1);
        Ok((input, s))
    } else {
        error("string", p)
    }
}

fn error<'a, O>(expected: &'a str, found: &Token<'a>) -> IResult<'a, O> {
    Err(nom::Err::Error(McshError::token(
        expected,
        (*found).clone(),
    )))
}
