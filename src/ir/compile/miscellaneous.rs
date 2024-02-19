use crate::ir::{
    compile::{compile_load_func, compile_store_func, REG_MEM_PTR},
    to_display, BoolOperator, BoolOprRhs, FormatArgument, Operator,
};

use super::{CacheTag, Ir, Label, PREFIX};
use std::fmt::{Display, Formatter, Result as FmtResult, Write};

pub(super) fn compile_ir<'a>(ir: &'a Ir) -> impl Display + 'a {
    to_display(move |output| match ir {
        Ir::Assign { dst, value } => {
            let dst = compile_cache_tag(*dst);
            writeln!(output, "scoreboard players set MCSH {dst} {value}",)
        }

        Ir::BoolOperation {
            dst,
            lhs,
            opr,
            rhs: BoolOprRhs::CacheTag(rhs),
        } => {
            let (dst, lhs, rhs) = (
                compile_cache_tag(*dst),
                compile_cache_tag(*lhs),
                compile_cache_tag(*rhs),
            );

            let mut use_builtin = |opr| {
                writeln!(
                    output,
                    "scoreboard players set MCSH {dst} 0\n\
                    execute if score MCSH {lhs} {opr} MCSH {rhs} run \
                        scoreboard players set MCSH {dst} 1",
                )
            };

            match opr {
                BoolOperator::Equal => use_builtin("="),
                BoolOperator::Gt => use_builtin(">"),
                BoolOperator::Lt => use_builtin("<"),
                BoolOperator::Ge => use_builtin(">="),
                BoolOperator::Le => use_builtin("<="),
                BoolOperator::NotEqual => writeln!(
                    output,
                    "scoreboard players set MCSH {dst} 1\n\
                    execute if score MCSH {lhs} = MCSH {rhs} run \
                        scoreboard players set MCSH {dst} 0",
                ),
                BoolOperator::And => writeln!(
                    output,
                    "scoreboard players set MCSH {dst} 0\n\
                    execute unless score MCSH {lhs} matches 0 \
                        unless score MCSH {rhs} matches 0 run \
                        scoreboard players set MCSH {dst} 1",
                ),
                BoolOperator::Or => writeln!(
                    output,
                    "scoreboard players set MCSH {dst} 1\n\
                    execute if score MCSH {lhs} matches 0 \
                        if score MCSH {rhs} matches 0 run \
                        scoreboard players set MCSH {dst} 0",
                ),
            }
        }

        Ir::BoolOperation {
            dst,
            lhs,
            opr,
            rhs: BoolOprRhs::Constant(rhs),
        } => {
            let (dst, lhs) = (compile_cache_tag(*dst), compile_cache_tag(*lhs));

            let mut use_builtin = |range: &dyn Display| {
                writeln!(
                    output,
                    "scoreboard players set MCSH {dst} 0\n\
                    execute if score MCSH {lhs} matches {range} run \
                        scoreboard players set MCSH {dst} 1",
                )
            };

            let write_false = display_write!("scoreboard players set MCSH {dst} 0\n");

            match opr {
                BoolOperator::Equal => use_builtin(&display_write!("{rhs}")),
                BoolOperator::NotEqual => use_builtin(&display_write!("!{rhs}")),

                BoolOperator::Gt => match rhs.checked_add(1) {
                    Some(bound) => use_builtin(&display_write!("{bound}..")),
                    None => write_false.fmt(output),
                },

                BoolOperator::Lt => match rhs.checked_sub(1) {
                    Some(bound) => use_builtin(&display_write!("..{bound}")),
                    None => write_false.fmt(output),
                },

                BoolOperator::Ge => use_builtin(&display_write!("{rhs}..")),
                BoolOperator::Le => use_builtin(&display_write!("..{rhs}")),

                BoolOperator::And if *rhs == 0 => write_false.fmt(output),

                BoolOperator::Or if *rhs != 0 => {
                    writeln!(output, "scoreboard players set MCSH {dst} 1")
                }

                BoolOperator::And | BoolOperator::Or => {
                    writeln!(
                        output,
                        "scoreboard players set MCSH {dst} 0\n\
                        execute unless score MCSH {lhs} matches 0 run \
                            scoreboard players set MCSH {dst} 1",
                    )
                }
            }
        }

        Ir::Call { label } => {
            let label = compile_label(label, true);
            writeln!(output, "function {label}")
        }

        Ir::CmdRaw(name) => {
            writeln!(output, "{name}")
        }

        Ir::Cond {
            positive,
            cond,
            then,
        } => {
            let cond = compile_cache_tag(*cond);
            let then = compile_label(then, true);
            let if_tag = if *positive { "unless" } else { "if" };

            writeln!(
                output,
                "execute {if_tag} score MCSH {cond} matches 0 run function {then}",
            )
        }

        Ir::Increase { dst, value } => {
            let dst = compile_cache_tag(*dst);
            writeln!(output, "scoreboard players add MCSH {dst} {value}")
        }

        Ir::Operation { dst, opr, src } => {
            let dst = compile_cache_tag(*dst);
            let src = compile_cache_tag(*src);

            let opr = match opr {
                Operator::Set => "=",
                Operator::Add => "+=",
                Operator::Sub => "-=",
                Operator::Mul => "*=",
                Operator::Div => "/=",
                Operator::Rem => "%=",
                Operator::Max => ">",
                Operator::Min => "<",
                Operator::Swp => "><",
            };

            writeln!(
                output,
                "scoreboard players operation MCSH {dst} {opr} MCSH {src}",
            )
        }

        Ir::Store { mem_offset, size } => {
            let mem_offset = compile_cache_tag(*mem_offset);
            let store = compile_store_func(*size);

            writeln!(
                output,
                "scoreboard players operation MCSH {REG_MEM_PTR} = MCSH {mem_offset}\n\
                function MCSH/{store}"
            )
        }

        Ir::Load { mem_offset, size } => {
            let mem_offset = compile_cache_tag(*mem_offset);
            let load = compile_load_func(*size);

            writeln!(
                output,
                "scoreboard players operation MCSH {REG_MEM_PTR} = MCSH {mem_offset}\n\
                function MCSH/{load}"
            )
        }

        Ir::Not { dst } => {
            let dst = compile_cache_tag(*dst);
            writeln!(
                output,
                "scoreboard players set MCSH {dst} 0\n\
                execute if score MCSH {dst} matches 0 run \
                    scoreboard players set MCSH {dst} 1"
            )
        }

        Ir::Random { dst, max, min } => {
            let dst = compile_cache_tag(*dst);
            writeln!(output, "scoreboard players random MCSH {dst} {min} {max}")
        }

        Ir::SimulationAbort => Ok(()),

        Ir::PrintFmt { args, target } => {
            let mut printer = Printer::new(output, target)?;

            for arg in args {
                match arg {
                    FormatArgument::CacheTag(ct) => {
                        printer.flush()?;
                        printer.push_comma()?;
                        write!(
                            printer.output,
                            "{{ \
                                \"score\": {{ \
                                    \"name\": \"MCSH\", \
                                    \"objective\": \"{}\" \
                                }} \
                            }}",
                            compile_cache_tag(*ct)
                        )?;
                    }
                    FormatArgument::ConstInt(int) => {
                        write!(printer.buffer, "{int}")?;
                    }
                    FormatArgument::Selector(sel) => {
                        printer.flush()?;
                        printer.push_comma()?;
                        write!(printer.output, r#"{{ "selector": "{sel}" }}"#)?;
                    }
                    FormatArgument::Style(style) => {
                        write!(printer.buffer, "§{}", style.code())?;
                    }
                    FormatArgument::Text(t) => {
                        write!(printer.buffer, "{t}")?;
                    }
                }
            }

            printer.end()
        }
    })
}

pub(super) fn compile_cache_tag(ct: CacheTag) -> impl Display + '_ {
    to_display(move |f| match ct {
        CacheTag::Regular(id) => write!(f, "{PREFIX}_CacheTag_{id}"),
        CacheTag::Static(id) => write!(f, "{PREFIX}_StaticCacheTag_{id}"),
        CacheTag::StaticExport(name) => name.fmt(f),
        CacheTag::StaticBuiltin(name) => write!(f, "{PREFIX}_StaticBuiltin_{name}"),
    })
}

pub(super) fn compile_label<'a>(label: &'a Label, with_dir: bool) -> impl Display + 'a {
    let dir = if with_dir { "MCSH/" } else { "" };

    to_display(move |f| match label {
        Label::Anonymous(id) => write!(f, "{dir}{PREFIX}_AnonymousLabel_{id}"),
        Label::Named { name, export } => {
            if *export {
                name.fmt(f)
            } else {
                write!(f, "{dir}{PREFIX}_Label_{name}")
            }
        }
    })
}

struct Printer<'a, 'f> {
    is_first: bool,
    buffer: String,
    output: &'a mut Formatter<'f>,
}

impl<'a, 'f> Printer<'a, 'f> {
    fn new(output: &'a mut Formatter<'f>, target: &str) -> Result<Self, std::fmt::Error> {
        write!(output, r#"tellraw {target} {{ "rawtext":[ "#)?;
        Ok(Printer {
            is_first: true,
            buffer: String::new(),
            output,
        })
    }

    fn flush(&mut self) -> FmtResult {
        if self.buffer.is_empty() {
            return Ok(());
        }
        self.push_comma()?;
        write!(self.output, r#"{{ "text": "{}" }}"#, self.buffer)?;
        self.buffer.clear();
        Ok(())
    }

    fn push_comma(&mut self) -> FmtResult {
        if self.is_first {
            self.is_first = false;
        } else {
            write!(self.output, ", ")?;
        }
        Ok(())
    }

    fn end(mut self) -> FmtResult {
        self.flush()?;
        write!(self.output, " ] }}\n")?;
        Ok(())
    }
}
