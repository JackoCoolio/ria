#![allow(dead_code)]

use ria_lexer::{Spanned, Symbol, Token};
use winnow::{
    error::{ContextError, ErrMode},
    stream::Stream,
    PResult, Parser,
};

pub mod def;
pub mod expr;

fn ident<'i, S>(input: &mut S) -> PResult<Spanned<&'i str>>
where
    S: Stream<Token = Spanned<Token<'i>>>,
{
    match input.next_token() {
        Some(Spanned(Token::Ident(ident_str), span)) => Ok(Spanned::new(ident_str, span)),
        _ => Err(ErrMode::Backtrack(ContextError::new())),
    }
}

/// Parse a keyword.
fn keyword<'i, 'kw, S>(kw: &'kw str) -> impl FnMut(&mut S) -> PResult<Spanned<&'i str>> + 'kw
where
    S: Stream<Token = Spanned<Token<'i>>>,
{
    move |input: &mut S| {
        ident
            .verify(|ident: &Spanned<&str>| ident.0 == kw)
            .parse_next(input)
    }
}

/// Parses the given symbol.
fn symbol<'sym, 'i, S>(symbol: &'sym Symbol) -> impl FnMut(&mut S) -> PResult<Spanned<()>> + 'sym
where
    S: Stream<Token = Spanned<Token<'i>>>,
{
    move |input: &mut S| match input.next_token() {
        Some(Spanned(Token::Symbol(ref the_symbol), span)) if the_symbol == symbol => {
            Ok(Spanned::new((), span))
        }
        _ => Err(ErrMode::Backtrack(ContextError::new())),
    }
}

fn newline<'i, S>(input: &mut S) -> PResult<Spanned<()>>
where
    S: Stream<Token = Spanned<Token<'i>>>,
{
    match input.next_token() {
        Some(Spanned(Token::NewLine, span)) => Ok(Spanned::new((), span)),
        _ => Err(ErrMode::Backtrack(ContextError::new())),
    }
}
