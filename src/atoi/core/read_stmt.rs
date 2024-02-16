use anyhow::{anyhow, Result};

use crate::{
    atoi::{get_anonymous_id, variable_not_found, Atoi, Binding},
    ir::{BoolOperator, BoolOprRhs, CacheTag, Ir, Label, LabelInfo, Operator},
    parse::{Expr, Stmt, StmtAssign, StmtIf, StmtReturn, StmtSwap, StmtWhile},
};

use super::{REG_COND_ENABLE, REG_CURRENT_MEM_OFFSET, REG_PARENT_MEM_OFFSET, REG_RETURNED_VALUE};

pub(super) struct ReadStmtWorkflow<'a> {
    pub label: Option<LabelInfo<'a>>,
    pub continue_break_points: Option<(Label<'a>, Label<'a>)>,
    pub cache_offset: u32,
}

impl<'a> ReadStmtWorkflow<'a> {
    fn insts(&mut self) -> &mut Vec<Ir<'a>> {
        &mut self.label.as_mut().unwrap().insts
    }
}

impl<'a> ReadStmtWorkflow<'a> {
    fn read_expr(&mut self, atoi: &mut Atoi<'a>, expr: &Expr<'a>) -> Result<CacheTag<'a>> {
        atoi.read_expr_at_next_reg(
            expr,
            &mut self.label.as_mut().unwrap().insts,
            &mut self.cache_offset,
        )
    }
}

impl<'a> Atoi<'a> {
    fn read_block(&mut self, stmts: &Vec<Stmt<'a>>, wf: &mut ReadStmtWorkflow<'a>) -> Result<()> {
        self.bindings.delimite();
        for stmt in stmts {
            if wf.label.is_none() {
                break;
            }
            self.read_stmt(stmt, wf)?;
        }
        self.bindings.pop_block();
        Ok(())
    }

    fn read_arm(
        &mut self,
        stmts: &Vec<Stmt<'a>>,
        branch_end: Label<'a>,
        wf: &mut ReadStmtWorkflow<'a>,
    ) -> Result<Label<'a>> {
        let mut new_info = self.new_label();
        let new_label = new_info.label;

        new_info.insts.push(Ir::Assign {
            dst: REG_COND_ENABLE,
            value: 0,
        });

        let mut wf2 = ReadStmtWorkflow {
            label: Some(new_info),
            continue_break_points: wf.continue_break_points,
            cache_offset: wf.cache_offset,
        };

        self.read_block(stmts, &mut wf2)?;

        if let Some(mut new_label) = wf2.label.take() {
            new_label.insts.push(Ir::Call { label: branch_end });
            self.label_map.insert_label(new_label)?;
        }

        Ok(new_label)
    }

    fn find_variable(&self, name: &str) -> Result<CacheTag<'a>> {
        let Some(bind) = self.bindings.find_newest(name) else {
            return Err(variable_not_found(name));
        };

        match bind {
            Binding::Constant(_) => Err(anyhow!(
                "cannot assign value to a constant identifier `{name}`"
            )),
            Binding::Cache(cache_tag) => Ok(*cache_tag),
        }
    }

    pub(super) fn read_stmt(
        &mut self,
        stmt: &Stmt<'a>,
        wf: &mut ReadStmtWorkflow<'a>,
    ) -> Result<()> {
        match stmt {
            Stmt::Assign(StmtAssign {
                is_bind,
                name,
                expr,
            }) => {
                if *is_bind {
                    let result = wf.read_expr(self, expr)?;
                    self.bindings.push(name, Binding::Cache(result));
                } else {
                    let dst = self.find_variable(name)?;
                    let cache_offset = wf.cache_offset;
                    self.read_expr(expr, wf.insts(), dst, cache_offset)?;
                }
            }

            Stmt::Block(block) => {
                self.read_block(block, wf)?;
            }

            Stmt::Expr(expr) => {
                wf.read_expr(self, expr)?;
                wf.cache_offset -= 1;
            }

            Stmt::Yield => return Err(anyhow!("yielding is not support yet")),

            Stmt::Break => match wf.continue_break_points {
                Some((_, break_point)) => {
                    wf.insts().push(Ir::Call { label: break_point });
                    self.label_map.insert_label(wf.label.take().unwrap())?;
                }
                None => return Err(anyhow!("keyword `break` can only be used in loop")),
            },

            Stmt::Continue => match wf.continue_break_points {
                Some((continue_point, _)) => {
                    wf.insts().push(Ir::Call {
                        label: continue_point,
                    });
                    self.label_map.insert_label(wf.label.take().unwrap())?;
                }
                None => return Err(anyhow!("keyword `continue` can only be used in loop")),
            },

            Stmt::Return(StmtReturn { expr }) => {
                let mut info = wf.label.take().unwrap();
                if let Some(expr) = expr {
                    self.read_expr(expr, &mut info.insts, REG_RETURNED_VALUE, wf.cache_offset)?;
                }

                info.insts.push(Ir::Operation {
                    dst: REG_CURRENT_MEM_OFFSET,
                    opr: Operator::Set,
                    src: REG_PARENT_MEM_OFFSET,
                });
                self.label_map.insert_label(info)?;
            }

            Stmt::Swap(StmtSwap { lhs, rhs }) => {
                let lhs = self.find_variable(lhs)?;
                let rhs = self.find_variable(rhs)?;
                wf.insts().push(Ir::Operation {
                    dst: lhs,
                    opr: Operator::Swp,
                    src: rhs,
                });
            }

            Stmt::If(StmtIf { arms, default }) => {
                wf.insts().push(Ir::Assign {
                    dst: REG_COND_ENABLE,
                    value: 1,
                });

                let branch_end = self.new_label();

                let mut is_first = true;
                for (cond, stmts) in arms {
                    let mut cond = wf.read_expr(self, cond)?;

                    // 如果不是第一个，则设置判决成功
                    if is_first {
                        is_first = false;
                    } else {
                        let cond2 = CacheTag::Regular(get_anonymous_id(&mut wf.cache_offset));
                        wf.insts().push(Ir::BoolOperation {
                            dst: cond2,
                            lhs: cond,
                            opr: BoolOperator::And,
                            rhs: BoolOprRhs::CacheTag(REG_COND_ENABLE),
                        });
                        cond = cond2;
                    }

                    let arm_label = self.read_arm(stmts, branch_end.label, wf)?;
                    wf.insts().push(Ir::Cond {
                        positive: true,
                        cond,
                        then: arm_label,
                    });
                }

                static EMPTY_BLOCK: Vec<Stmt<'static>> = Vec::new();
                let default_block = default.as_ref().unwrap_or(&EMPTY_BLOCK);
                let default_label = self.read_arm(default_block, branch_end.label, wf)?;
                wf.insts().push(Ir::Cond {
                    positive: true,
                    cond: REG_COND_ENABLE,
                    then: default_label,
                });

                self.label_map
                    .insert_label(wf.label.replace(branch_end).unwrap())?;
            }

            Stmt::While(StmtWhile { expr, body }) => {
                let mut loop_end = self.new_label();
                loop_end.insts.push(Ir::Assign {
                    dst: REG_COND_ENABLE,
                    value: 0,
                });
                let loop_end_label = loop_end.label;
                self.label_map.insert_label(loop_end)?;

                let mut cond_info = self.new_label();
                let body_info = self.new_label();
                wf.insts().push(Ir::Call {
                    label: cond_info.label,
                });

                let expr_result = self.read_expr_at_next_reg(
                    expr,
                    &mut cond_info.insts,
                    &mut wf.cache_offset.clone(),
                )?;
                cond_info.insts.push(Ir::Cond {
                    positive: true,
                    cond: expr_result,
                    then: body_info.label,
                });

                let mut body_wf = ReadStmtWorkflow {
                    label: Some(body_info),
                    continue_break_points: Some((cond_info.label, loop_end_label)),
                    cache_offset: wf.cache_offset,
                };
                self.read_block(body, &mut body_wf)?;
                if let Some(mut body_info) = body_wf.label.take() {
                    body_info.insts.push(Ir::Call {
                        label: cond_info.label,
                    });
                    self.label_map.insert_label(body_info)?;
                }

                self.label_map.insert_label(cond_info)?;
            }

            Stmt::Debugger => {
                wf.insts().push(Ir::SimulationAbort);
            }
        }

        Ok(())
    }
}
