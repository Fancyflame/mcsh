use anyhow::{anyhow, Result};

use crate::{
    atoi::{get_anonymous_id, no_string_error, variable_not_found, Atoi, Binding},
    ir::{BoolOprRhs, CacheTag, Ir, Operator},
    parse::{
        lexer::Punct,
        parse_file::{parse_expr, to_anyhow_result},
        Expr, ExprBinary, ExprBlock, ExprFnCall, ExprUnary,
    },
};

use super::{
    convert_bool_opr, convert_opr, macros::macro_not_found, CONST_MINUS_ONE, FRAME_HEAD_LENGTH,
    REG_CURRENT_MEM_OFFSET, REG_PARENT_MEM_OFFSET, REG_RETURNED_VALUE,
};

fn new_reg(cache_offset: &mut u32) -> CacheTag<'static> {
    CacheTag::Regular(get_anonymous_id(cache_offset))
}

impl<'a> Atoi<'a> {
    pub fn read_expr_at_next_reg(
        &mut self,
        expr: &Expr<'a>,
        insts: &mut Vec<Ir<'a>>,
        cache_offset: &mut u32,
    ) -> Result<CacheTag<'a>> {
        let reg = new_reg(cache_offset);
        self.read_expr(expr, insts, reg, *cache_offset)?;
        Ok(reg)
    }

    pub(super) fn read_expr(
        &mut self,
        expr: &Expr<'a>,
        insts: &mut Vec<Ir<'a>>,
        dst: CacheTag<'a>,
        mut cache_offset: u32,
    ) -> Result<()> {
        match expr {
            Expr::Integer(value) => {
                insts.push(Ir::Assign { dst, value: *value });
            }

            Expr::Str(_) => {
                return Err(anyhow!("string can only be assigned to constant"));
            }

            Expr::Block(ExprBlock { .. }) => {
                todo!()
            }

            Expr::MacroCall(m) => {
                let Some(lexer) = self.call_macro(m) else {
                    return Err(macro_not_found(m.name));
                };

                self.read_expr(
                    &to_anyhow_result(parse_expr(lexer))?,
                    insts,
                    dst,
                    cache_offset,
                )?;
            }

            Expr::Var(var) => {
                let Some(tag) = self.bindings.find_newest(var) else {
                    return Err(variable_not_found(var));
                };

                match tag {
                    Binding::Cache(src) => insts.push(Ir::Operation {
                        dst,
                        opr: Operator::Set,
                        src: *src,
                    }),
                    Binding::Constant(val) => insts.push(Ir::Assign { dst, value: *val }),
                    Binding::String(_) => return Err(no_string_error()),
                }
            }

            Expr::Binary(ExprBinary {
                bin_op,
                lhs: lhs_expr,
                rhs: rhs_expr,
            }) => {
                let ir = if let Some(opr) = convert_opr(bin_op) {
                    self.read_expr(lhs_expr, insts, dst, cache_offset)?;
                    let rhs = self.read_expr_at_next_reg(rhs_expr, insts, &mut cache_offset)?;

                    Ir::Operation { dst, opr, src: rhs }
                } else if let Some(opr) = convert_bool_opr(bin_op) {
                    let lhs = self.read_expr_at_next_reg(&lhs_expr, insts, &mut cache_offset)?;

                    if let Expr::Integer(val) = **rhs_expr {
                        Ir::BoolOperation {
                            dst,
                            lhs,
                            opr,
                            rhs: BoolOprRhs::Constant(val),
                        }
                    } else {
                        let rhs =
                            self.read_expr_at_next_reg(&rhs_expr, insts, &mut cache_offset)?;
                        Ir::BoolOperation {
                            dst,
                            lhs,
                            opr,
                            rhs: BoolOprRhs::CacheTag(rhs),
                        }
                    }
                } else {
                    return Err(anyhow!("unrecognized binary operator `{bin_op}`"));
                };

                insts.push(ir);
            }

            Expr::Unary(ExprUnary { op, expr }) => {
                self.read_expr(expr, insts, dst, cache_offset)?;
                let ir = match op {
                    Punct::Bang => Ir::Not { dst },
                    Punct::Minus => Ir::Operation {
                        dst,
                        opr: Operator::Mul,
                        src: CONST_MINUS_ONE,
                    },
                    _ => return Err(anyhow!("unrecognized unary operator `{op}`")),
                };
                insts.push(ir);
            }

            Expr::Call(expr_fn_call @ ExprFnCall { name, args }) => {
                let Some(def) = self.functions.find_newest(name).copied() else {
                    if self.call_builtin_function(expr_fn_call, insts, dst, cache_offset)? {
                        return Ok(());
                    } else {
                        return Err(anyhow!("function `{name}` not found"));
                    }
                };

                if args.len() as u32 != def.arg_count {
                    return Err(anyhow!(
                        "function `{name}` requires {} arguments, but {} was provided",
                        def.arg_count,
                        args.len()
                    ));
                }

                // 求得需要交换的字数
                let chunks_to_swap = cache_offset.div_ceil(self.label_map.word_width());

                // 参数求值
                let mut temp_cache_offset = cache_offset;
                for arg in args {
                    let nth_arg = new_reg(&mut temp_cache_offset);
                    self.read_expr(arg, insts, nth_arg, temp_cache_offset)?;
                }

                // 把缓存换进内存
                insts.push(Ir::Store {
                    mem_offset: REG_CURRENT_MEM_OFFSET,
                    size: chunks_to_swap,
                });

                // 记录父函数内存位移
                insts.push(Ir::Operation {
                    dst: REG_PARENT_MEM_OFFSET,
                    opr: Operator::Set,
                    src: REG_CURRENT_MEM_OFFSET,
                });
                insts.push(Ir::Increase {
                    dst: REG_CURRENT_MEM_OFFSET,
                    value: chunks_to_swap as _,
                });

                // 参数搬移
                for offset in 0..(args.len() as u32) {
                    insts.push(Ir::Operation {
                        dst: CacheTag::Regular(FRAME_HEAD_LENGTH + offset),
                        opr: Operator::Set,
                        src: CacheTag::Regular(cache_offset + offset),
                    });
                }

                // 调用函数
                insts.push(Ir::Call { label: def.label });

                // 把缓存换出来
                insts.push(Ir::Load {
                    mem_offset: REG_CURRENT_MEM_OFFSET,
                    size: chunks_to_swap,
                });

                // 写入返回值
                insts.push(Ir::Operation {
                    dst,
                    opr: Operator::Set,
                    src: REG_RETURNED_VALUE,
                });
            }
        };

        Ok(())
    }

    fn require_constant(&self, expr: &Expr) -> Result<i32> {
        match expr {
            Expr::Integer(v) => Ok(*v),
            Expr::Var(name) => {
                let Some(var) = self.bindings.find_newest(name) else {
                    return Err(variable_not_found(name));
                };

                match var {
                    Binding::Cache(_) => Err(anyhow!(
                        "only constant value is allowed, but variable `{name}` was found"
                    )),
                    Binding::Constant(v) => Ok(*v),
                    Binding::String(_) => Err(no_string_error()),
                }
            }
            _ => Err(anyhow!("only constant value is allowed")),
        }
    }

    fn call_builtin_function(
        &mut self,
        ExprFnCall { name, args }: &ExprFnCall<'a>,
        insts: &mut Vec<Ir<'a>>,
        dst: CacheTag<'a>,
        mut cache_offset: u32,
    ) -> Result<bool> {
        let get_args = || match &**args {
            [lhs_expr, rhs_expr] => Ok((lhs_expr, rhs_expr)),
            _ => Err(anyhow!(
                "builtin function `{name}` requires 2 arguments, but {} was provided",
                args.len()
            )),
        };

        let mut to_opr_expr = |this: &mut Self, insts: &mut _, opr| {
            let (lhs_expr, rhs_expr) = get_args()?;
            let rhs = new_reg(&mut cache_offset);
            this.read_expr(lhs_expr, insts, dst, cache_offset)?;
            this.read_expr(rhs_expr, insts, rhs, cache_offset)?;

            insts.push(Ir::Operation { dst, opr, src: rhs });
            Ok(true)
        };

        match *name {
            "min" => to_opr_expr(self, insts, Operator::Min),
            "max" => to_opr_expr(self, insts, Operator::Max),
            "random" => {
                let (lhs_expr, rhs_expr) = get_args()?;
                let min = self.require_constant(lhs_expr)?;
                let max = self.require_constant(rhs_expr)?;
                insts.push(Ir::Random { dst, max, min });
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
