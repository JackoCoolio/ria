use ria_lexer::{Spanned, Symbol, Token};
use winnow::{
    combinator::cut_err,
    error::{StrContext, StrContextValue},
    stream::Stream,
    PResult, Parser,
};

use crate::{ident, maybe_newline, symbol};

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
        maybe_newline(input);
        let _ = symbol(&Symbol::Arrow).parse_next(input)?;
        maybe_newline(input);
        let body = Expr::parse
            .context(StrContext::Expected(StrContextValue::Description(
                "an expression",
            )))
            .parse_next(input)?;
        Ok(Self {
            param,
            body: Box::new(body),
        })
    }
}
