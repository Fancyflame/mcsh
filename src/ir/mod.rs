use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
};

use anyhow::{anyhow, Result};

const PREFIX: &str = "__MCSH_Private";

pub mod compile;
pub mod simulate;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum CacheTag<'a> {
    Regular(u32),
    Static(u32),
    StaticBuiltin(&'a str),
    StaticExport(&'a str),
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Label<'a> {
    Named { name: &'a str, export: bool },
    Anonymous(u32),
}

pub struct LabelMap<'a> {
    label_map: HashMap<Label<'a>, LabelInfo<'a>>,
    static_map: HashMap<CacheTag<'a>, i32>,
    mem_size: u32,
    word_width: u32,
}

#[derive(Clone)]
pub struct LabelInfo<'a> {
    pub label: Label<'a>,
    pub insts: Vec<Ir<'a>>,
}

impl<'a> LabelInfo<'a> {
    pub fn new(label: Label<'a>) -> Self {
        LabelInfo {
            label,
            insts: Vec::new(),
        }
    }
}

impl<'a> LabelMap<'a> {
    pub fn new(mem_size: u32, word_width: u32) -> Self {
        Self {
            label_map: Default::default(),
            static_map: Default::default(),
            mem_size,
            word_width,
        }
    }

    pub fn insert_label(&mut self, label_info: LabelInfo<'a>) -> Result<()> {
        if self
            .label_map
            .insert(label_info.label, label_info)
            .is_some()
        {
            Err(anyhow!(
                "cannot define label because the label has already exists"
            ))
        } else {
            Ok(())
        }
    }

    pub fn insert_static(&mut self, cache_tag: CacheTag<'a>, value: i32) -> Result<()> {
        assert!(
            !matches!(cache_tag, CacheTag::Regular(_)),
            "cache tag must be static cache"
        );

        if self.static_map.insert(cache_tag, value).is_some() {
            Err(anyhow!(
                "cannot insert static because the static has already exists"
            ))
        } else {
            Ok(())
        }
    }

    pub fn word_width(&self) -> u32 {
        self.word_width
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Ir<'a> {
    Assign {
        dst: CacheTag<'a>,
        value: i32,
    },
    Call {
        label: Label<'a>,
    },
    CallExtern {
        name: &'a str,
    },
    Increase {
        dst: CacheTag<'a>,
        value: i32,
    },
    Operation {
        dst: CacheTag<'a>,
        opr: Operator,
        src: CacheTag<'a>,
    },
    BoolOperation {
        dst: CacheTag<'a>,
        lhs: CacheTag<'a>,
        opr: BoolOperator,
        rhs: BoolOprRhs<'a>,
    },
    Not {
        dst: CacheTag<'a>,
    },
    Cond {
        positive: bool,
        cond: CacheTag<'a>,
        then: Label<'a>,
    },
    Load {
        mem_offset: CacheTag<'a>,
        size: u32,
    },
    Store {
        mem_offset: CacheTag<'a>,
        size: u32,
    },
    Random {
        dst: CacheTag<'a>,
        max: i32,
        min: i32,
    },
    SimulationAbort,
}

#[derive(Clone, Copy, Debug)]
pub enum BoolOprRhs<'a> {
    CacheTag(CacheTag<'a>),
    Constant(i32),
}

#[derive(Clone, Copy, Debug)]
pub enum Operator {
    Set,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Max,
    Min,
    Swp,
}

#[derive(Clone, Copy, Debug)]
pub enum BoolOperator {
    Equal,
    NotEqual,
    And,
    Or,
    Gt,
    Lt,
    Ge,
    Le,
}

pub enum OperatorAsDisplay {
    BinaryOp(&'static str),
    Function(&'static str),
}

impl Operator {
    fn as_display(&self) -> OperatorAsDisplay {
        match self {
            Self::Add => OperatorAsDisplay::BinaryOp("+="),
            Self::Div => OperatorAsDisplay::BinaryOp("/="),
            Self::Mul => OperatorAsDisplay::BinaryOp("*="),
            Self::Rem => OperatorAsDisplay::BinaryOp("%="),
            Self::Set => OperatorAsDisplay::BinaryOp("="),
            Self::Sub => OperatorAsDisplay::BinaryOp("-="),
            Self::Swp => OperatorAsDisplay::Function("swap"),
            Self::Max => OperatorAsDisplay::Function("max"),
            Self::Min => OperatorAsDisplay::Function("min"),
        }
    }
}

impl Display for BoolOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Equal => "==",
                Self::NotEqual => "!=",
                Self::And => "&&",
                Self::Or => "||",
                Self::Gt => ">",
                Self::Lt => "<",
                Self::Ge => ">=",
                Self::Le => "<=",
            }
        )
    }
}

fn to_display<F>(f: F) -> impl Display
where
    F: Fn(&mut Formatter) -> Result<(), std::fmt::Error>,
{
    struct FormatFn<F>(F);
    impl<F> Display for FormatFn<F>
    where
        F: Fn(&mut Formatter) -> Result<(), std::fmt::Error>,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            (self.0)(f)
        }
    }

    FormatFn(f)
}
