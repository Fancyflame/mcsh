use anyhow::{anyhow, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::alphanumeric1,
    combinator::{eof, map, map_opt, recognize, verify},
    multi::{many0, many0_count, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, terminated},
};
use std::fmt::Write;

use crate::{
    atoi::{Atoi, Binding},
    format::FormatStyle,
    ir::{FormatArgument, Ir},
    parse::{
        entity_selector::entity_selector,
        lexer::{ident, parse_tokens, specified_punct, string, Lexer, Punct},
        parse_file::{parse_expr, to_anyhow_result},
        MacroCall,
    },
};

use super::read_def::ConstValue;

impl<'a> Atoi<'a> {
    pub(super) fn call_macro(&self, MacroCall { .. }: &MacroCall<'a>) -> Option<Lexer<'a>> {
        None
    }

    pub(super) fn macro_run_concat(&self, insts: &mut Vec<Ir<'a>>, lexer: Lexer<'a>) -> Result<()> {
        let consts = to_anyhow_result(separated_list0(
            specified_punct(Punct::Comma),
            map_opt(parse_expr, |expr| self.read_constant(&expr).ok()),
        )(lexer))?;

        let mut output = String::new();
        for c in consts {
            match c {
                ConstValue::Int(i) => write!(output, "{i}").unwrap(),
                ConstValue::Str(s) => output.push_str(s),
            }
        }
        output = output.replace("\\\"", "\"");

        insts.push(Ir::CmdRaw(output.into()));
        Ok(())
    }

    pub(super) fn macro_run(insts: &mut Vec<Ir<'a>>, lexer: Lexer<'a>) -> Result<()> {
        let (_, cmds) =
            terminated(separated_list0(specified_punct(Punct::Comma), string), eof)(lexer)
                .map_err(|_| anyhow!("syntax error occurred when calling `run` macro"))?;

        for cmd in cmds {
            if cmd.contains('\n') || cmd.contains('\r') {
                return Err(anyhow!("raw command cannot contains new lines (\\r\\n)"));
            }

            insts.push(Ir::CmdRaw(cmd.replace("\\\"", "\"").into()));
        }

        Ok(())
    }

    pub(super) fn macro_print(&self, insts: &mut Vec<Ir<'a>>, lexer: Lexer<'a>) -> Result<()> {
        let (mut prefix, string) = to_anyhow_result(separated_pair(
            entity_selector,
            specified_punct(Punct::Comma),
            string,
        )(lexer))?;

        prefix.insert_str(0, "tellraw ");

        let formatted = self.formatted_args(string)?;
        insts.push(Ir::CmdFmt {
            prefix,
            args: formatted,
        });
        Ok(())
    }

    pub(super) fn macro_title(&self, insts: &mut Vec<Ir<'a>>, lexer: Lexer<'a>) -> Result<()> {
        let (selector, (position, fmt_str)) = to_anyhow_result(separated_pair(
            entity_selector,
            specified_punct(Punct::Comma),
            separated_pair(
                verify(ident, |x| matches!(x, "title" | "subtitle" | "actionbar")),
                specified_punct(Punct::Comma),
                string,
            ),
        )(lexer))?;

        let formatted = self.formatted_args(fmt_str)?;
        insts.push(Ir::CmdFmt {
            prefix: format!("titleraw {selector} {position}"),
            args: formatted,
        });
        Ok(())
    }

    fn formatted_args(&self, input: &'a str) -> Result<Vec<FormatArgument<'a>>> {
        let get_bind = |name: &str| {
            self.bindings.find_newest(name).map(|bind| match bind {
                Binding::Cache(c) => FormatArgument::CacheTag(*c),
                Binding::Constant(i) => FormatArgument::ConstInt(*i),
                Binding::String(s) => FormatArgument::Text(s),
            })
        };

        let selector = map_opt(parse_tokens, |tokens| {
            pair(entity_selector, eof)(Lexer::from(tokens))
                .ok()
                .map(|(_, (s, _))| FormatArgument::Selector(s))
        });

        let parse_ident = || recognize(many0_count(alt((alphanumeric1, tag("_")))));

        let parse_option = alt((
            map_opt(parse_ident(), get_bind),
            map_opt(preceded(tag("#"), parse_ident()), |name| {
                FormatStyle::from_name(name).map(FormatArgument::Style)
            }),
            selector,
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
