use std::collections::HashMap;

use self::{
    error::McshError,
    lexer::{Lexer, Punct},
};

pub use parse_file::parse_file;

pub mod entity_selector;
pub mod error;
pub mod lexer;
pub mod parse_file;

pub type IResult<'a, O> = nom::IResult<Lexer<'a>, O, McshError<'a>>;
pub type Block<'a> = Vec<Stmt<'a>>;

#[derive(Clone, Debug)]
pub enum Definition<'a> {
    Function(ItemFn<'a>),
    Constant(ItemConstant<'a>),
    Static(ItemStatic<'a>),
}

#[derive(Clone, Debug)]
pub struct ItemConstant<'a> {
    pub name: &'a str,
    pub expr: Expr<'a>,
}

#[derive(Clone, Debug)]
pub struct ItemStatic<'a> {
    pub export: bool,
    pub name: &'a str,
    pub expr: Expr<'a>,
}

#[derive(Clone, Debug)]
pub enum Stmt<'a> {
    Block(Vec<Stmt<'a>>),
    Assign(StmtAssign<'a>),
    While(StmtWhile<'a>),
    If(StmtIf<'a>),
    Yield,
    Return(StmtReturn<'a>),
    Break,
    Continue,
    Expr(Expr<'a>),
    Swap(StmtSwap<'a>),
    Debugger,
    MacroCall(MacroCall<'a>),
}

#[derive(Clone, Debug)]
pub enum Expr<'a> {
    Var(&'a str),
    Integer(i32),
    Binary(ExprBinary<'a>),
    Unary(ExprUnary<'a>),
    Call(ExprFnCall<'a>),
    Str(&'a str),
    Block(ExprBlock<'a>),
    MacroCall(MacroCall<'a>),
}

#[derive(Clone, Debug)]
pub struct ExprUnary<'a> {
    pub op: Punct,
    pub expr: Box<Expr<'a>>,
}

#[derive(Clone, Copy, Debug)]
pub struct StmtSwap<'a> {
    pub lhs: &'a str,
    pub rhs: &'a str,
}

#[derive(Clone, Debug)]
pub struct StmtAssign<'a> {
    pub is_bind: bool,
    pub name: &'a str,
    pub expr: Expr<'a>,
}

#[derive(Clone, Debug)]
pub struct StmtWhile<'a> {
    pub expr: Expr<'a>,
    pub body: Block<'a>,
}

#[derive(Clone, Debug)]
pub struct StmtIf<'a> {
    pub arms: Vec<(Expr<'a>, Block<'a>)>,
    pub default: Option<Block<'a>>,
}

#[derive(Clone, Debug)]
pub struct StmtMatch<'a> {
    pub map: HashMap<i32, Block<'a>>,
}

#[derive(Clone, Debug)]
pub struct StmtReturn<'a> {
    pub expr: Option<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct ExprFnCall<'a> {
    pub name: &'a str,
    pub args: Vec<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct ExprBinary<'a> {
    pub bin_op: Punct,
    pub lhs: Box<Expr<'a>>,
    pub rhs: Box<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct ExprBlock<'a> {
    pub stmts: Vec<Stmt<'a>>,
    pub ret: Box<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct MacroCall<'a> {
    pub name: &'a str,
    pub tokens: Lexer<'a>,
}

#[derive(Clone, Debug)]
pub struct ItemFn<'a> {
    pub export: bool,
    pub name: &'a str,
    pub args: Vec<&'a str>,
    pub body: Block<'a>,
}
