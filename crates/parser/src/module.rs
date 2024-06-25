use ria_lexer::{Spanned, Token};
use winnow::{stream::Stream, PResult, Parser};

use crate::{def::DefList, maybe_newline};

#[derive(Debug)]
pub struct Module<'i> {
    defs: DefList<'i>,
}

impl<'i> Module<'i> {
    pub fn parse<S>(input: &mut S) -> PResult<Self>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        let defs = DefList::parse.parse_next(input)?;
        maybe_newline(input);
        Ok(Self { defs })
    }
}
