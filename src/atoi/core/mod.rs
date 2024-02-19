use anyhow::{anyhow, Result};

use crate::{
    ir::{BoolOperator, CacheTag, Ir, Label, LabelInfo, Operator},
    parse::{lexer::Punct, ItemFn},
};

use self::read_stmt::ReadStmtWorkflow;

use super::{get_fn_label, Atoi, Binding, FuncDef};

mod macros;
mod read_def;
mod read_expr;
mod read_stmt;

const FRAME_HEAD_LENGTH: u32 = 1;

pub(super) const REG_CURRENT_MEM_OFFSET: CacheTag = CacheTag::StaticBuiltin("CurrentMemoryOffset");
pub(super) const REG_RETURNED_VALUE: CacheTag = CacheTag::StaticBuiltin("ReturnedValue");
pub(super) const REG_COND_ENABLE: CacheTag = CacheTag::StaticBuiltin("CondEnable");
pub(super) const CONST_MINUS_ONE: CacheTag = CacheTag::StaticBuiltin("MinusOne");
const REG_PARENT_MEM_OFFSET: CacheTag = CacheTag::Regular(0);

impl<'a> Atoi<'a> {
    pub fn insert_fn(
        &mut self,
        item_fn @ ItemFn {
            export,
            name,
            args,
            body,
        }: &ItemFn<'a>,
    ) -> Result<()> {
        self.bindings.delimite();
        let mut info = self.new_label();

        if *export {
            if args.len() != 0 {
                return Err(anyhow!(
                    "cannot export function `{name}` because \
                    it must takes no arguments",
                ));
            }
            self.insert_entry_fn(get_fn_label(item_fn), info.label)?;
        }

        self.functions.push(
            name,
            FuncDef {
                label: info.label,
                arg_count: args.len() as _,
            },
        );

        let mut cache_offset = FRAME_HEAD_LENGTH;

        for arg in args.iter().copied() {
            self.bindings
                .push(arg, Binding::Cache(CacheTag::Regular(cache_offset)));
            cache_offset += 1;
        }

        let mut wf = ReadStmtWorkflow {
            label: Some(info),
            continue_break_points: None,
            cache_offset,
        };

        for stmt in body {
            self.read_stmt(stmt, &mut wf)?;
        }
        self.bindings.pop_block();

        if let Some(mut info) = wf.label.take() {
            // 返回值默认为0
            info.insts.push(Ir::Assign {
                dst: REG_RETURNED_VALUE,
                value: 0,
            });
            info.insts.push(Ir::Operation {
                dst: REG_CURRENT_MEM_OFFSET,
                opr: Operator::Set,
                src: REG_PARENT_MEM_OFFSET,
            });
            self.label_map.insert_label(info)?;
        }

        Ok(())
    }

    fn insert_entry_fn(&mut self, label: Label<'a>, turn_to: Label<'a>) -> Result<()> {
        let mut info = LabelInfo::new(label);
        info.insts = vec![
            Ir::Assign {
                dst: REG_CURRENT_MEM_OFFSET,
                value: 0,
            },
            Ir::Assign {
                dst: REG_PARENT_MEM_OFFSET,
                value: 0,
            },
            Ir::Call { label: turn_to },
        ];
        self.label_map.insert_label(info)
    }
}

pub(super) fn convert_opr(p: &Punct) -> Option<Operator> {
    let opr = match p {
        Punct::Plus => Operator::Add,
        Punct::Minus => Operator::Sub,
        Punct::Star => Operator::Mul,
        Punct::Slash => Operator::Div,
        Punct::Percent => Operator::Rem,
        _ => return None,
    };
    Some(opr)
}

pub(super) fn convert_bool_opr(p: &Punct) -> Option<BoolOperator> {
    let opr = match p {
        Punct::GreaterThan => BoolOperator::Gt,
        Punct::LessThan => BoolOperator::Lt,
        Punct::GreaterEq => BoolOperator::Ge,
        Punct::LessEq => BoolOperator::Le,
        Punct::Equal2 => BoolOperator::Equal,
        Punct::NotEq => BoolOperator::NotEqual,
        Punct::And2 => BoolOperator::And,
        Punct::Or2 => BoolOperator::Or,
        _ => return None,
    };
    Some(opr)
}
