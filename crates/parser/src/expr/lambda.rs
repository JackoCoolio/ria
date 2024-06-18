use ria_lexer::{Spanned, Symbol, Token};
use winnow::{combinator::cut_err, stream::Stream, PResult, Parser};

use crate::{ident, symbol};

use super::Expr;

#[derive(Debug, PartialEq, Eq)]
pub struct Lambda<'i> {
    pub param: Spanned<&'i str>,
    pub body: Box<Expr<'i>>,
}

impl<'i> Lambda<'i> {
    pub fn parse<S>(input: &mut S) -> PResult<Self>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        let _ = symbol(&Symbol::Lambda).parse_next(input)?;
        let param = cut_err(ident).parse_next(input)?;
        let _ = symbol(&Symbol::Arrow).parse_next(input)?;
        let body = Expr::parse.parse_next(input)?;
        Ok(Self {
            param,
            body: Box::new(body),
        })
    }
}
