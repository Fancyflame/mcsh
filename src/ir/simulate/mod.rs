use std::{
    collections::HashMap,
    fmt::{Display, Write},
    ops::Range,
};

use anyhow::{anyhow, Result};
use rand::{rngs::ThreadRng, Rng};

use crate::ir::OperatorAsDisplay;

use super::{to_display, BoolOperator, BoolOprRhs, CacheTag, Ir, Label, LabelMap, Operator};

#[must_use]
pub struct SimulateResult {
    pub result: Result<i32>,
    pub log: String,
}

struct SimulateMachine<'a> {
    label_map: &'a LabelMap<'a>,
    memory: Vec<Option<i32>>,
    registers: HashMap<CacheTag<'a>, i32>,
    rest_ir: Vec<Ir<'a>>,
    log: String,
    rng: ThreadRng,
}

impl<'a> SimulateMachine<'a> {
    fn initialize(label_map: &'a LabelMap<'a>) -> Self {
        SimulateMachine {
            label_map,
            memory: vec![None; (label_map.mem_size * label_map.word_width) as _],
            registers: HashMap::new(),
            rest_ir: Vec::new(),
            log: String::new(),
            rng: rand::thread_rng(),
        }
    }

    fn run(&mut self, label: &Label) -> Result<()> {
        self.memory.fill(None);
        self.registers.clear();
        self.rest_ir.clear();
        self.log.clear();

        for (cache_tag, value) in self.label_map.static_map.iter() {
            self.registers.insert(*cache_tag, *value);
        }

        self.call(label)?;

        while let Some(inst) = self.rest_ir.pop() {
            let r = self.eval(inst);
            if let Err(err) = &r {
                write!(
                    self.log,
                    "\n\
                    SIMULATION FAILED\n\
                    - error message: {err}\n\
                    - when executing: {inst:?}\n"
                )
                .unwrap();
                return r;
            }
        }
        self.log += "SIMULATION FINISHED";
        Ok(())
    }

    fn display_value(&self, ct: &CacheTag) -> impl Display {
        let val = self.registers.get(ct).copied();
        to_display(move |f| match val {
            Some(val) => val.fmt(f),
            None => "none".fmt(f),
        })
    }

    fn read_value(&self, ct: &CacheTag) -> Result<i32> {
        match self.registers.get(ct) {
            Some(v) => Ok(*v),
            None => Err(anyhow!("trying to read `{ct:?}` before initialize")),
        }
    }

    fn get_value_mut(&mut self, ct: &CacheTag<'a>) -> Result<&mut i32> {
        match self.registers.get_mut(ct) {
            Some(v) => Ok(v),
            None => Err(anyhow!("trying to operate `{ct:?}` before initialize")),
        }
    }

    fn call(&mut self, label: &Label) -> Result<()> {
        let Some(info) = self.label_map.label_map.get(label) else {
            return Err(anyhow!("cannot call `{label:?}` as it is not defined"));
        };

        self.rest_ir.extend(info.insts.iter().rev().copied());
        Ok(())
    }

    fn get_mem_slice(&self, mem_offset: CacheTag, size: u32) -> Result<Range<usize>> {
        let pointer = self.read_value(&mem_offset)?;
        if pointer < 0 {
            return Err(anyhow!(
                "attempt to read an invalid pointer with the value 0, \
                but the pointer must be non-negative"
            ));
        }

        let word_width = self.label_map.word_width as usize;
        let start = self.read_value(&mem_offset)? as usize * word_width;
        let end = start + size as usize * word_width;

        if self.memory.get(start..end).is_none() {
            return Err(anyhow!(
                "memory overflow: attempt to read memory from {start} to {end}, \
                but the memory size is {}",
                self.label_map.mem_size
            ));
        };

        Ok(start..end)
    }

    fn eval(&mut self, ir: Ir<'a>) -> Result<()> {
        macro_rules! log {
            ($($tt:tt)*) => {
                writeln!(self.log, $($tt)*).unwrap()
            };
        }

        match ir {
            Ir::Assign { dst, value } => {
                let lhs_old = self.display_value(&dst);
                self.registers.insert(dst, value);
                log!("{dst:?} = {value} ({lhs_old} -> {value})");
            }

            Ir::BoolOperation { dst, lhs, opr, rhs } => {
                let rhs_val = match rhs {
                    BoolOprRhs::CacheTag(ct) => self.read_value(&ct)?,
                    BoolOprRhs::Constant(val) => val,
                };
                let lhs_val = self.read_value(&lhs)?;

                let result = match opr {
                    BoolOperator::And => (lhs_val != 0) && (rhs_val != 0),
                    BoolOperator::Equal => lhs_val == rhs_val,
                    BoolOperator::Ge => lhs_val >= rhs_val,
                    BoolOperator::Gt => lhs_val > rhs_val,
                    BoolOperator::Le => lhs_val <= rhs_val,
                    BoolOperator::Lt => lhs_val < rhs_val,
                    BoolOperator::NotEqual => lhs_val != rhs_val,
                    BoolOperator::Or => (lhs_val != 0) || (rhs_val != 0),
                };
                self.registers.insert(dst, if result { 1 } else { 0 });

                log!("{dst:?} = {lhs:?} {opr} {rhs:?} (lhs = {lhs_val}, rhs = {rhs_val})");
            }

            Ir::Call { label } => {
                self.call(&label)?;
                log!("call {label:?}");
            }

            Ir::CallExtern { name } => {
                log!("call external function `{name}`");
            }

            Ir::Cond {
                positive,
                cond,
                then,
            } => {
                let mut cond_val = self.read_value(&cond)? != 0;

                if !positive {
                    cond_val = !cond_val;
                }

                if cond_val {
                    self.call(&then)?;
                }

                log!(
                    "if{} {cond:?} then {then:?} (cond = {cond_val})",
                    if positive { "" } else { " not" }
                );
            }

            Ir::Increase { dst, value } => {
                *self.get_value_mut(&dst)? += value;
                log!("{dst:?} += {value}");
            }

            Ir::Load { mem_offset, size } => {
                let range = self.get_mem_slice(mem_offset, size)?;
                let mem = &self.memory[range.clone()];

                for (index, src) in mem.into_iter().enumerate() {
                    let ct = CacheTag::Regular(index as _);
                    match *src {
                        Some(val) => self.registers.insert(ct, val),
                        None => self.registers.remove(&ct),
                    };
                }

                log!(
                    "load {size} chunks from pointer {mem_offset:?} ({}..{})",
                    range.start,
                    range.end
                );
            }

            Ir::Not { dst } => {
                let val = self.get_value_mut(&dst)?;
                let old = *val;
                *val = if old == 0 { 1 } else { 0 };
                let val = *val;

                log!("not {dst:?} ({old} -> {val})");
            }

            Ir::Operation {
                dst,
                opr: Operator::Set,
                src,
            } => {
                let rhs = self.read_value(&src)?;
                let lhs_old = self.display_value(&dst);
                log!("{dst:?} = {src:?} ({lhs_old} -> {rhs})");
                self.registers.insert(dst, rhs);
            }

            Ir::Operation { dst, opr, src } => {
                let rhs = self.read_value(&src)?;
                let lhs = self.get_value_mut(&dst)?;
                let lhs_value = *lhs;

                match opr {
                    Operator::Add => *lhs += rhs,
                    Operator::Div => *lhs /= rhs,
                    Operator::Max => *lhs = (*lhs).max(rhs),
                    Operator::Min => *lhs = (*lhs).min(rhs),
                    Operator::Mul => *lhs *= rhs,
                    Operator::Rem => *lhs %= rhs,
                    Operator::Set => unreachable!(),
                    Operator::Sub => *lhs -= rhs,
                    Operator::Swp => {
                        *lhs = rhs;
                        *self.get_value_mut(&src).unwrap() = lhs_value;
                    }
                }

                match opr.as_display() {
                    OperatorAsDisplay::BinaryOp(binop) => {
                        log!("{dst:?} {binop} {src:?} (lhs = {lhs_value}, rhs = {rhs})")
                    }
                    OperatorAsDisplay::Function(func) => {
                        log!("{func} {dst:?} {src:?} (lhs = {lhs_value}, rhs = {rhs})")
                    }
                }
            }

            Ir::Random { dst, max, min } => {
                let value = self.rng.gen_range(min..=max);
                let lhs_old = self.display_value(&dst);
                self.registers.insert(dst, value);
                log!("{dst:?} = random {min}..{max} ({lhs_old} -> {value})");
            }

            Ir::Store { mem_offset, size } => {
                let range = self.get_mem_slice(mem_offset, size)?;
                let mem = &mut self.memory[range.clone()];

                for (index, dst) in mem.into_iter().enumerate() {
                    let ct = CacheTag::Regular(index as _);
                    *dst = self.registers.get(&ct).copied();
                }

                log!(
                    "store {size} chunks from pointer {mem_offset:?} ({}..{})",
                    range.start,
                    range.end
                );
            }

            Ir::SimulationAbort => {
                log!("simulation abort");
                return Err(anyhow!("simulation was aborted by pause command"));
            }
        }
        Ok(())
    }
}

impl<'a> LabelMap<'a> {
    pub fn simulate_pub(&self, fn_name: &str) -> SimulateResult {
        self.simulate(&Label::Named {
            name: fn_name,
            export: true,
        })
    }

    pub fn simulate(&self, entry_fn: &Label) -> SimulateResult {
        let mut machine = SimulateMachine::initialize(self);
        let r = machine.run(entry_fn).map(|()| {
            machine
                .registers
                .get(&CacheTag::StaticBuiltin("ReturnedValue"))
                .copied()
                .unwrap()
        });

        SimulateResult {
            result: r,
            log: machine.log,
        }
    }
}
