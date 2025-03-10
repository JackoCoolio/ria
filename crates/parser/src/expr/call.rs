use ria_lexer::{Spanned, Token};
use winnow::{
    error::{StrContext, StrContextValue},
    stream::Stream,
    ModalResult, Parser,
};

use super::Expr;

#[derive(Debug, PartialEq, Eq)]
pub struct Call<'i> {
    pub func: Box<Expr<'i>>,
    pub arg: Box<Expr<'i>>,
}

impl<'i> Call<'i> {
    pub fn parse<S>(input: &mut S) -> ModalResult<Self>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        let func = Expr::parse
            .context(StrContext::Expected(StrContextValue::Description(
                "a function",
            )))
            .map(Box::from)
            .parse_next(input)?;
        let arg = Expr::parse
            .context(StrContext::Expected(StrContextValue::Description(
                "an argument",
            )))
            .map(Box::from)
            .parse_next(input)?;
        Ok(Call { func, arg })
    }
}
