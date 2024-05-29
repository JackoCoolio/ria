use winnow::{stream::Stream, PResult, Parser};

use crate::lexer::{Lexer, Spanned, Symbol, Token};

use super::{expr::Expr, ident, symbol};

struct Module<'i> {
    defs: Box<[Def<'i>]>,
}

pub struct Def<'i> {
    ident: Spanned<&'i str>,
    expr: Expr<'i>,
}

impl<'i> Def<'i> {
    pub fn parse<S>(input: &mut S) -> PResult<Def<'i>>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        let ident = ident.parse_next(input)?;
        let _ = symbol(&Symbol::Define).parse_next(input)?;
        let expr = Expr::parse.parse_next(input)?;

        Ok(Self { ident, expr })
    }
}

#[test]
fn parse_simple_def() {
    let input = "x = y";
    let lexer = Lexer::new(input);
    let tokens: Box<[Spanned<Token>]> = lexer.collect();

    let def = Def::parse.parse(tokens.as_ref()).unwrap();

    assert_eq!(def.ident.inner(), "x");
    assert!(matches!(def.expr, Expr::Variable(Spanned("y", _))));
}