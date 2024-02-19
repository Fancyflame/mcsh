use anyhow::{anyhow, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::alphanumeric1,
    combinator::{eof, map, map_opt, recognize},
    multi::{many0, many0_count, separated_list0},
    sequence::{delimited, preceded, separated_pair, terminated},
};

use crate::{
    atoi::{Atoi, Binding},
    ir::{format::FormatStyle, FormatArgument, Ir},
    parse::{
        entity_selector::entity_selector,
        lexer::{specified_punct, string, Lexer, Punct},
        parse_file::to_anyhow_result,
        MacroCall,
    },
};

impl<'a> Atoi<'a> {
    pub(super) fn call_macro(&self, MacroCall { .. }: &MacroCall<'a>) -> Option<Lexer<'a>> {
        None
    }

    pub(super) fn macro_run(insts: &mut Vec<Ir<'a>>, lexer: Lexer<'a>) -> Result<()> {
        let (_, cmds) =
            terminated(separated_list0(specified_punct(Punct::Comma), string), eof)(lexer)
                .map_err(|_| anyhow!("syntax error occurred when calling `run` macro"))?;

        for cmd in cmds {
            if cmd.contains("\n") || cmd.contains("\r") {
                return Err(anyhow!("raw command cannot contains new lines (\\r\\n)"));
            }

            insts.push(Ir::CmdRaw(cmd));
        }

        Ok(())
    }

    pub(super) fn macro_print(&self, insts: &mut Vec<Ir<'a>>, lexer: Lexer<'a>) -> Result<()> {
        let (selector, string) = to_anyhow_result(separated_pair(
            entity_selector,
            specified_punct(Punct::Comma),
            string,
        )(lexer))?;
        let formatted = self.formatted_args(string)?;
        insts.push(Ir::PrintFmt {
            target: selector,
            args: formatted,
        });
        Ok(())
    }

    fn formatted_args(&self, input: &'a str) -> Result<Vec<FormatArgument<'a>>> {
        let get_bind = |name: &str| {
            self.bindings.find_newest(name).map(|bind| match bind {
                Binding::Cache(c) => FormatArgument::CacheTag(*c),
                Binding::Constant(i) => FormatArgument::ConstInt(*i),
                Binding::String(s) => FormatArgument::Text(*s),
            })
        };

        let parse_option = alt((
            map_opt(alphanumeric1, get_bind),
            map_opt(preceded(tag("#"), alphanumeric1), |name| {
                println!("{name}");
                FormatStyle::from_name(name).map(FormatArgument::Style)
            }),
        ));

        let r: nom::IResult<_, _> = terminated(
            many0(alt((
                map(tag("{{"), |_| FormatArgument::Text("{")),
                map(tag("}}"), |_| FormatArgument::Text("}")),
                map(is_not("{}"), FormatArgument::Text),
                delimited(tag("{"), parse_option, tag("}")),
            ))),
            eof,
        )(input);

        match r {
            Ok((_, r)) => Ok(r),
            Err(err) => Err(anyhow!("cannot format the string: {err}")),
        }
    }
}

pub(super) fn macro_not_found(name: &str) -> anyhow::Error {
    anyhow!("macro `{name}` not defined or not available on this situation")
}
