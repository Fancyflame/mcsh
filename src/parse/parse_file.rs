use anyhow::anyhow;
use nom::{
    branch::alt,
    combinator::{eof, map, opt, value, verify},
    multi::{many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Parser,
};

use super::{
    lexer::{group, ident, integer, keyword, punct, specified_punct, Delimiter, Lexer, Punct},
    Block, Definition, Expr, ExprFnCall, ExprUnary, IResult, ItemConstant, ItemFn, ItemStatic,
    Stmt, StmtAssign, StmtIf, StmtReturn, StmtSwap, StmtWhile,
};

#[cfg(debug_assertions)]
#[allow(dead_code)]
fn debug<'a, O, E>(mut parser: impl Parser<Lexer<'a>, O, E>) -> impl Parser<Lexer<'a>, O, E> {
    move |input| {
        println!("{input}");
        let r = parser.parse(input);
        println!("<<<<parse success: {}>>>>", r.is_ok());
        r
    }
}

pub fn parse_file(file: &str) -> anyhow::Result<Vec<Definition>> {
    let lexer = Lexer::parse(file)?;
    let (_, vec) =
        terminated(many0(parse_definition), eof)(lexer).map_err(|err| anyhow!("{err}"))?;
    Ok(vec)
}

pub fn parse_definition(input: Lexer) -> IResult<Definition> {
    let parse_const_bind = |kw| {
        tuple((
            map(opt(keyword("pub")), |o| o.is_some()),
            keyword(kw),
            ident,
            specified_punct(Punct::Equal),
            parse_expr,
            specified_punct(Punct::Semi),
        ))
    };

    let parse_const = map(
        parse_const_bind("const"),
        |(export, _, name, _, expr, _)| Definition::Constant(ItemConstant { export, name, expr }),
    );

    let parse_static = map(
        parse_const_bind("static"),
        |(export, _, name, _, expr, _)| Definition::Static(ItemStatic { export, name, expr }),
    );

    alt((
        parse_const,
        parse_static,
        map(parse_item_fn, Definition::Function),
    ))(input)
}

pub fn parse_item_fn(input: Lexer) -> IResult<ItemFn> {
    map(
        pair(
            map(opt(keyword("pub")), |o| o.is_some()),
            preceded(
                keyword("fn"),
                tuple((
                    ident,
                    group(Delimiter::Paren).and_then(terminated(
                        separated_list0(specified_punct(Punct::Comma), ident),
                        eof,
                    )),
                    parse_stmts,
                )),
            ),
        ),
        |(export, (name, args, body))| ItemFn {
            export,
            name,
            args,
            body,
        },
    )(input)
}

pub fn parse_stmts(input: Lexer) -> IResult<Block> {
    group(Delimiter::Brace)
        .and_then(terminated(many0(parse_stmt), eof))
        .parse(input)
}

fn one_kw_parser<'a>(
    val: Stmt<'a>,
    kw: &'static str,
) -> impl FnMut(Lexer<'a>) -> IResult<'a, Stmt<'a>> {
    value(val, pair(keyword(kw), specified_punct(Punct::Semi)))
}

pub fn parse_stmt(input: Lexer) -> IResult<Stmt> {
    let parse_block = map(group(Delimiter::Brace).and_then(parse_stmts), Stmt::Block);

    let parse_let = map(
        terminated(
            pair(
                opt(keyword("let")),
                separated_pair(ident, specified_punct(Punct::Equal), parse_expr),
            ),
            specified_punct(Punct::Semi),
        ),
        |(bind, (name, expr))| {
            Stmt::Assign(StmtAssign {
                is_bind: bind.is_some(),
                name,
                expr,
            })
        },
    );

    let parse_while = map(
        preceded(keyword("while"), pair(parse_expr, parse_stmts)),
        |(expr, body)| Stmt::While(StmtWhile { expr, body }),
    );

    let parse_if = {
        map(
            pair(
                separated_list1(
                    keyword("else"),
                    preceded(keyword("if"), pair(parse_expr, parse_stmts)),
                ),
                opt(preceded(keyword("else"), parse_stmts)),
            ),
            |(arms, default)| Stmt::If(StmtIf { arms, default }),
        )
    };

    let parse_yield = one_kw_parser(Stmt::Yield, "yield");
    let parse_break = one_kw_parser(Stmt::Break, "break");
    let parse_continue = one_kw_parser(Stmt::Continue, "continue");
    let parse_debugger = one_kw_parser(Stmt::Debugger, "debugger");

    let parse_swap = map(
        terminated(
            separated_pair(ident, specified_punct(Punct::Swap), ident),
            specified_punct(Punct::Semi),
        ),
        |(lhs, rhs)| Stmt::Swap(StmtSwap { lhs, rhs }),
    );

    let parse_return = map(
        delimited(
            keyword("return"),
            opt(parse_expr),
            specified_punct(Punct::Semi),
        ),
        |expr| Stmt::Return(StmtReturn { expr }),
    );

    let parse_stmt_expr = map(
        terminated(parse_expr, specified_punct(Punct::Semi)),
        Stmt::Expr,
    );

    alt((
        parse_block,
        parse_let,
        parse_while,
        parse_if,
        parse_yield,
        parse_break,
        parse_continue,
        parse_debugger,
        parse_return,
        parse_swap,
        parse_stmt_expr,
    ))(input)
}

pub fn parse_expr(input: Lexer) -> IResult<Expr> {
    let binop = verify(punct, |p| infix_binding_power(*p).is_some());

    map(
        pair(parse_atomic_expr, many0(pair(binop, parse_atomic_expr))),
        |(first, rest)| parse_binary_expr(first, &rest),
    )(input)
}

fn parse_atomic_expr(input: Lexer) -> IResult<Expr> {
    let atomic_expr = alt((
        group(Delimiter::Paren).and_then(parse_expr),
        map(integer, Expr::Integer),
        map(
            pair(
                ident,
                group(Delimiter::Paren).and_then(terminated(
                    separated_list0(specified_punct(Punct::Comma), parse_expr),
                    eof,
                )),
            ),
            |(name, args)| Expr::Call(ExprFnCall { name, args }),
        ),
        map(ident, Expr::Var),
    ));

    alt((
        map(
            alt((
                pair(specified_punct(Punct::Bang), parse_atomic_expr),
                pair(specified_punct(Punct::Minus), parse_atomic_expr),
            )),
            |(op, expr)| {
                Expr::Unary(ExprUnary {
                    op,
                    expr: Box::new(expr),
                })
            },
        ),
        atomic_expr,
    ))(input)
}

fn parse_binary_expr<'a>(mut lhs: Expr<'a>, mut rest: &[(Punct, Expr<'a>)]) -> Expr<'a> {
    while let Some(((bin_op, right_first), rest2)) = rest.split_first() {
        rest = rest2;
        let (_, min_bp) = infix_binding_power(*bin_op).unwrap();

        let mut upper_bound = rest.len();
        for (index, (binop, _)) in rest.iter().enumerate() {
            let (l_bp, _) = infix_binding_power(*binop).unwrap();
            if l_bp < min_bp {
                upper_bound = index;
                break;
            }
        }

        let rhs = parse_binary_expr(right_first.clone(), &rest[..upper_bound]);
        lhs = Expr::Binary(super::ExprBinary {
            bin_op: *bin_op,
            lhs: lhs.into(),
            rhs: rhs.into(),
        });
        rest = &rest[upper_bound..];
    }
    lhs
}

fn infix_binding_power(punct: Punct) -> Option<(u8, u8)> {
    let level = match punct {
        Punct::Or2 => 1,
        Punct::And2 => 2,
        Punct::Equal2 | Punct::NotEq => 3,
        Punct::LessThan | Punct::LessEq | Punct::GreaterThan | Punct::GreaterEq => 4,
        Punct::Plus | Punct::Minus => 5,
        Punct::Star | Punct::Slash | Punct::Percent => 6,
        _ => return None,
    };

    Some((level * 2 - 1, level * 2))
}
