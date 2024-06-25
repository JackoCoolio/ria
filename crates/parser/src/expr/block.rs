use ria_lexer::{Spanned, Symbol, Token};
use winnow::{combinator::opt, stream::Stream, PResult, Parser};

use crate::{def::DefList, maybe_newline, newline, symbol};

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
        symbol(&Symbol::OpenParen).parse_next(input)?;
        maybe_newline(input);

        let defs = DefList::parse.parse_next(input)?;

        newline(input)?;

        let expr = opt(Expr::parse)
            .map(|expr| expr.map(Box::from))
            .parse_next(input)?;
        maybe_newline(input);

        symbol(&Symbol::CloseParen).parse_next(input)?;

        Ok(Block { defs, expr })
    }
}
