use ria_lexer::{Spanned, Symbol, Token};
use winnow::{
    combinator::{delimited, opt, preceded},
    stream::Stream,
    PResult, Parser,
};

use crate::{def::DefList, newline, symbol};

use super::Expr;

#[derive(Debug, PartialEq, Eq)]
pub struct Block<'i> {
    pub defs: DefList<'i>,
    pub expr: Option<Box<Expr<'i>>>,
}

impl<'i> Block<'i> {
    pub fn parse<S>(input: &mut S) -> PResult<Self>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        delimited(
            symbol(&Symbol::OpenParen),
            |input: &mut S| {
                let defs = DefList::parse.parse_next(input)?;
                let expr = opt(preceded(newline, Expr::parse))
                    .map(|expr| expr.map(Box::from))
                    .parse_next(input)?;

                Ok(Block { defs, expr })
            },
            symbol(&Symbol::CloseParen),
        )
        .parse_next(input)
    }
}
