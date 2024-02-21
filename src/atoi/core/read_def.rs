use anyhow::{anyhow, Result};

use crate::{
    atoi::{
        calculate_arithmetical_bin_expr, calculate_bool_bin_expr, get_anonymous_id, get_fn_label,
        no_string_error, variable_not_found, Atoi, Binding, FuncDef,
    },
    ir::CacheTag,
    parse::{
        lexer::Punct,
        parse_file::{parse_expr, to_anyhow_result},
        Definition, Expr, ExprBinary, ExprBlock, ExprUnary, ItemConstant, ItemStatic,
    },
};

use super::{convert_bool_opr, convert_opr, macros::macro_not_found};

#[derive(Clone, Copy)]
pub enum ConstValue<'a> {
    Int(i32),
    Str(&'a str),
}

impl<'a> Atoi<'a> {
    pub fn read_def(&mut self, def: &Definition<'a>) -> Result<()> {
        match def {
            Definition::Constant(ItemConstant { name, expr }) => {
                if self.bindings.has_sibling_namesake(name) {
                    return Err(anyhow!("constant or static `{name}` has been defined"));
                }

                let value = match self.read_constant(expr)? {
                    ConstValue::Int(int) => Binding::Constant(int),
                    ConstValue::Str(s) => Binding::String(s),
                };

                self.bindings.push(name, value);
            }

            Definition::Static(ItemStatic { name, expr, export }) => {
                if self.bindings.has_sibling_namesake(name) {
                    return Err(anyhow!("constant or static `{name}` has been defined"));
                }

                let ConstValue::Int(value) = self.read_constant(expr)? else {
                    return Err(no_string_error());
                };

                let cache_tag = if *export {
                    CacheTag::StaticExport(name)
                } else {
                    CacheTag::Static(get_anonymous_id(&mut self.anonymous_static_pool))
                };

                self.label_map.insert_static(cache_tag, value)?;
                self.bindings.push(name, Binding::Cache(cache_tag));
            }

            Definition::Function(item_fn) => {
                if self.functions.has_sibling_namesake(item_fn.name) {
                    return Err(anyhow!(
                        "function or macro `{}` has been defined",
                        item_fn.name
                    ));
                }

                self.functions.push(
                    item_fn.name,
                    FuncDef {
                        label: get_fn_label(item_fn),
                        arg_count: item_fn.args.len() as _,
                    },
                )
            }
        }
        Ok(())
    }

    fn read_constant(&self, expr: &Expr<'a>) -> Result<ConstValue<'a>> {
        match expr {
            Expr::Integer(int) => Ok(ConstValue::Int(*int)),
            Expr::Binary(ExprBinary { bin_op, lhs, rhs }) => {
                let (ConstValue::Int(lhs), ConstValue::Int(rhs)) =
                    (self.read_constant(lhs)?, self.read_constant(rhs)?)
                else {
                    return Err(anyhow!("string cannot do binary operation"));
                };

                let r = if let Some(op) = convert_opr(bin_op) {
                    calculate_arithmetical_bin_expr(lhs, rhs, op)
                } else if let Some(op) = convert_bool_opr(bin_op) {
                    calculate_bool_bin_expr(lhs, rhs, op)
                } else {
                    return Err(anyhow!("unrecognized binary operator `{bin_op}`"));
                };
                Ok(ConstValue::Int(r))
            }
            Expr::Unary(ExprUnary { op, expr }) => {
                let ConstValue::Int(val) = self.read_constant(expr)? else {
                    return Err(anyhow!("string cannot do unary operation"));
                };

                let r = match op {
                    Punct::Bang => {
                        if val != 0 {
                            0
                        } else {
                            1
                        }
                    }
                    Punct::Minus => -val,
                    _ => return Err(anyhow!("unrecognized unary operator `{op}`")),
                };

                Ok(ConstValue::Int(r))
            }
            Expr::Var(id) => {
                let Some(bind) = self.bindings.find_newest(id) else {
                    return Err(variable_not_found(id));
                };

                match bind {
                    Binding::Cache(_) => Err(anyhow!("identifier `{id}` is not a constant")),
                    Binding::Constant(val) => Ok(ConstValue::Int(*val)),
                    Binding::String(val) => Ok(ConstValue::Str(val)),
                }
            }
            Expr::Str(s) => Ok(ConstValue::Str(s)),
            Expr::Block(ExprBlock { stmts, ret }) => {
                if stmts.is_empty() {
                    self.read_constant(ret)
                } else {
                    Err(anyhow!(
                        "a constant expression block cannot contains any statement yet"
                    ))
                }
            }
            Expr::Call(_) => Err(anyhow!(
                "calling a function is cannot be a constant operation yet"
            )),
            Expr::MacroCall(m) => {
                let Some(lexer) = self.call_macro(m) else {
                    return Err(macro_not_found(m.name));
                };

                let e = to_anyhow_result(parse_expr(lexer))?;
                self.read_constant(&e)
            }
        }
    }
}

//fn read_constant(expr: &ConstExpr) -> Result<Const>
