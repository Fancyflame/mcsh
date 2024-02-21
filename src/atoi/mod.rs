use anyhow::{anyhow, Ok, Result};

use crate::{
    ir::{BoolOperator, CacheTag, Label, LabelInfo, LabelMap, Operator},
    parse::{Definition, ItemFn},
};

use self::{
    core::{CONST_MINUS_ONE, REG_COND_ENABLE, REG_CURRENT_MEM_OFFSET, REG_RETURNED_VALUE},
    stack::UnsizedStack,
};

mod core;
mod stack;

#[derive(Clone, Copy, Debug)]
enum Binding<'a> {
    Constant(i32),
    String(&'a str),
    Cache(CacheTag<'a>),
}

#[derive(Clone, Copy)]
struct FuncDef<'a> {
    label: Label<'a>,
    arg_count: u32,
}

pub struct Atoi<'a> {
    functions: UnsizedStack<'a, FuncDef<'a>>,
    bindings: UnsizedStack<'a, Binding<'a>>,
    label_map: LabelMap<'a>,
    anonymous_label_pool: u32,
    anonymous_static_pool: u32,
}

impl<'a> Atoi<'a> {
    pub fn new() -> Self {
        let mut label_map = LabelMap::new(64, 4);
        for (key, val) in [
            (REG_COND_ENABLE, 0),
            (REG_CURRENT_MEM_OFFSET, 0),
            (CONST_MINUS_ONE, -1),
            (REG_RETURNED_VALUE, 0),
        ] {
            label_map.insert_static(key, val).unwrap();
        }

        Self {
            functions: UnsizedStack::new(),
            bindings: UnsizedStack::new(),
            label_map,
            anonymous_label_pool: 0,
            anonymous_static_pool: 0,
        }
    }

    fn new_label(&mut self) -> LabelInfo<'a> {
        LabelInfo::new(Label::Anonymous(get_anonymous_id(
            &mut self.anonymous_label_pool,
        )))
    }

    pub fn parse(&mut self, defs: &[Definition<'a>]) -> Result<()> {
        for def in defs {
            self.read_def(def)?;
        }

        for def in defs {
            let Definition::Function(item_fn) = def else {
                continue;
            };

            self.insert_fn(item_fn)?;
        }

        Ok(())
    }

    pub fn finish(self) -> LabelMap<'a> {
        self.label_map
    }
}

fn get_fn_label<'a>(ast: &ItemFn<'a>) -> Label<'a> {
    Label::Named {
        name: ast.name,
        export: ast.export,
    }
}

fn get_anonymous_id(pool: &mut u32) -> u32 {
    let r = *pool;
    *pool = pool.checked_add(1).expect("anonymous pool overflow");
    r
}

fn variable_not_found(var: &str) -> anyhow::Error {
    anyhow!("identifier `{var}` is not defined")
}

fn no_string_error() -> anyhow::Error {
    anyhow!("string can only be used in constant and macro definition")
}

pub fn calculate_arithmetical_bin_expr(lhs: i32, rhs: i32, opr: Operator) -> i32 {
    match opr {
        Operator::Add => lhs + rhs,
        Operator::Sub => lhs - rhs,
        Operator::Mul => lhs * rhs,
        Operator::Div => lhs / rhs,
        Operator::Rem => lhs % rhs,
        Operator::Max => lhs.max(rhs),
        Operator::Min => lhs.min(rhs),
        Operator::Set | Operator::Swp => panic!("set or swap operation is invalid"),
    }
}

pub fn calculate_bool_bin_expr(lhs: i32, rhs: i32, opr: BoolOperator) -> i32 {
    let r = match opr {
        BoolOperator::And => (lhs != 0) && (rhs != 0),
        BoolOperator::Or => (lhs != 0) || (rhs != 0),
        BoolOperator::Equal => lhs == rhs,
        BoolOperator::Ge => lhs >= rhs,
        BoolOperator::Gt => lhs > rhs,
        BoolOperator::Le => lhs <= rhs,
        BoolOperator::Lt => lhs < rhs,
        BoolOperator::NotEqual => lhs != rhs,
    };
    if r {
        1
    } else {
        0
    }
}
