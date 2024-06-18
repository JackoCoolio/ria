use ria_lexer::{Spanned, Token};
use winnow::{stream::Stream, PResult, Parser};

use super::Expr;

#[derive(Debug, PartialEq, Eq)]
pub struct Call<'i> {
    pub func: Box<Expr<'i>>,
    pub arg: Box<Expr<'i>>,
}

impl<'i> Call<'i> {
    pub fn parse<S>(input: &mut S) -> PResult<Self>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        let func = Expr::parse.map(Box::from).parse_next(input)?;
        let arg = Expr::parse.map(Box::from).parse_next(input)?;
        Ok(Call { func, arg })
    }
}
