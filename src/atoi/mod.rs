use anyhow::{anyhow, Result};

use crate::{
    ir::{CacheTag, Label, LabelInfo, LabelMap},
    parse::{Definition, Expr, ItemConstant, ItemFn, ItemStatic},
};

use self::{
    core::{CONST_MINUS_ONE, REG_CURRENT_MEM_OFFSET},
    stack::UnsizedStack,
};

mod core;
mod stack;

#[derive(Clone, Copy)]
enum Binding<'a> {
    Constant(i32),
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
        label_map.insert_static(REG_CURRENT_MEM_OFFSET, 0).unwrap();
        label_map.insert_static(CONST_MINUS_ONE, -1).unwrap();

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
            match def {
                Definition::Constant(ItemConstant { name, expr, .. }) => {
                    self.bindings
                        .push(name, Binding::Constant(constant_calculate(expr)?));
                }

                Definition::Static(ItemStatic { name, expr, export }) => {
                    let value = constant_calculate(expr)?;

                    let cache_tag = if *export {
                        CacheTag::StaticExport(name)
                    } else {
                        CacheTag::Static(get_anonymous_id(&mut self.anonymous_static_pool))
                    };

                    self.label_map.insert_static(cache_tag, value)?;
                    self.bindings.push(name, Binding::Cache(cache_tag));
                }

                Definition::Function(item_fn) => self.functions.push(
                    item_fn.name,
                    FuncDef {
                        label: get_fn_label(item_fn),
                        arg_count: item_fn.args.len() as _,
                    },
                ),
            }
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
    anyhow!("variable `{var}` not found")
}

fn constant_calculate(expr: &Expr) -> Result<i32> {
    if let Expr::Integer(value) = expr {
        Ok(*value)
    } else {
        Err(anyhow!(
            "given expression cannot be considered as constant value"
        ))
    }
}
