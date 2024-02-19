use std::{
    fmt::{Display, Formatter},
    rc::Rc,
};

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag, take_until},
    character::complete::{self, alpha1, alphanumeric1, multispace0, one_of},
    combinator::{fail, map, opt, recognize, value},
    multi::{many0, many0_count},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

#[derive(Debug, Clone)]
pub enum Token<'a> {
    Ident(&'a str),
    Punct(Punct),
    Group(Group<'a>),
    Literal(Literal<'a>),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Group<'a> {
    pub delimiter: Delimiter,
    pub content: Rc<[Token<'a>]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delimiter {
    Paren,
    Bracket,
    Brace,
}

macro_rules! punct {
    {$($Punct:ident $display:literal,)*} => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Punct {
            $(
                #[doc = concat!("`", $display, "`")]
                $Punct,
            )*
        }

        impl Punct {
            pub fn display(&self) -> &'static str {
                match self {
                    $(
                        Self::$Punct => $display,
                    )*
                }
            }
        }

        fn parse_punct(input: &str) -> IResult<&str, Punct> {
            $(if let r @ Ok(_) = value(Punct::$Punct, tag($display))(input) {
                r
            } else)* {
                fail(input)
            }
        }
    };
}

punct! {
    Equal2 "==",
    NotEq "!=",
    LessEq "<=",
    GreaterEq ">=",
    And2 "&&",
    Or2 "||",
    Swap "><",
    Dot2 "..",
    Equal "=",
    Plus "+",
    Minus "-",
    Star "*",
    Slash "/",
    Percent "%",
    Semi ";",
    Dot ".",
    Comma ",",
    LessThan "<",
    GreaterThan ">",
    Bang "!",
    At "@",
    Pound "#",
}

#[derive(Debug, Clone, Copy)]
pub enum Literal<'a> {
    Int(i32),
    Str(&'a str),
}

pub fn parse_tokens(input: &str) -> IResult<&str, Rc<[Token]>> {
    let parse_token = alt((
        map(parse_ident, Token::Ident),
        map(parse_group, Token::Group),
        map(parse_str, |s| Token::Literal(Literal::Str(s))),
        map(parse_punct, Token::Punct), // punct必须在int前，因为它需要解析数字前符号
        map(complete::i32, |num| Token::Literal(Literal::Int(num))),
    ));

    map(
        preceded(parse_sep, many0(terminated(parse_token, parse_sep))),
        |content| content.into_boxed_slice().into(),
    )(input)
}

fn parse_ident(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn parse_str(input: &str) -> IResult<&str, &str> {
    delimited(
        tag("\""),
        escaped(is_not("\\\"\r\n"), '\\', one_of(r#""\"#)),
        tag("\""),
    )(input)
}

fn parse_sep(input: &str) -> IResult<&str, ()> {
    value(
        (),
        pair(
            multispace0,
            many0_count(pair(
                alt((
                    value((), pair(tag("//"), opt(is_not("\r\n")))),
                    value((), tuple((tag("/*"), take_until("*/"), tag("*/")))),
                )),
                multispace0,
            )),
        ),
    )(input)
}

fn parse_group(input: &str) -> IResult<&str, Group> {
    let (input, delimiter) = alt((
        value(Delimiter::Paren, tag("(")),
        value(Delimiter::Bracket, tag("[")),
        value(Delimiter::Brace, tag("{")),
    ))(input)?;

    let ret = map(
        terminated(
            |input| {
                let r = parse_tokens(input);
                //dbg!(&r);
                r
            },
            match delimiter {
                Delimiter::Paren => tag(")"),
                Delimiter::Bracket => tag("]"),
                Delimiter::Brace => tag("}"),
            },
        ),
        |content| Group { delimiter, content },
    )(input);
    ret
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eof => write!(f, "end of input"),
            Self::Group(group) => write!(f, "{group}"),
            Self::Ident(id) => write!(f, "{id}"),
            Self::Literal(lit) => write!(f, "{lit}"),
            Self::Punct(punct) => write!(f, "{punct}"),
        }
    }
}

impl Delimiter {
    pub fn display(&self) -> &'static str {
        match self {
            Delimiter::Paren => "(...)",
            Delimiter::Bracket => "[...]",
            Delimiter::Brace => "{...}",
        }
    }
}

impl Display for Group<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (start, end) = match self.delimiter {
            Delimiter::Paren => ("(", ")"),
            Delimiter::Bracket => ("[", "]"),
            Delimiter::Brace => ("{", "}"),
        };

        write!(f, "{start} ")?;
        for t in self.content.iter() {
            write!(f, "{t}")?;
        }
        write!(f, "{end}")
    }
}

impl Display for Punct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.display().fmt(f)
    }
}

impl Display for Literal<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(int) => int.fmt(f),
            Self::Str(s) => write!(f, "\"{s}\""),
        }
    }
}
