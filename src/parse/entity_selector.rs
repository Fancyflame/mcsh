use std::fmt::Write;

use super::{
    lexer::{group, ident, specified_punct, Delimiter, Lexer, Punct},
    IResult,
};
use nom::{
    combinator::{map, opt},
    sequence::tuple,
};

pub fn entity_selector(lexer: Lexer) -> IResult<String> {
    map(
        tuple((
            specified_punct(Punct::At),
            ident,
            opt(group(Delimiter::Bracket)),
        )),
        |(at, id, gr)| {
            let mut s = format!("{at}{id}");
            if let Some(l) = gr {
                write!(s, "[{l}]").unwrap();
            }
            s
        },
    )(lexer)
}
