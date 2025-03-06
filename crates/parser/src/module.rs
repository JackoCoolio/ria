use ria_lexer::{Spanned, Token};
use winnow::{stream::Stream, ModalResult, Parser};

use crate::{def::DefList, maybe_newline};

/// A module - a file.
#[derive(Debug)]
pub struct Module<'i> {
    /// The top-level definitions in the file.
    defs: DefList<'i>,
}

impl<'i> Module<'i> {
    /// Parses a `Module`.
    pub fn parse<S>(input: &mut S) -> ModalResult<Self>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        let defs = DefList::parse.parse_next(input)?;
        maybe_newline(input);
        Ok(Self { defs })
    }
}
