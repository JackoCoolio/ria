#![allow(dead_code)]

use ria_lexer::{Spanned, Symbol, Token};
use winnow::{
    error::{ContextError, ErrMode, StrContext, StrContextValue},
    stream::Stream,
    PResult, Parser,
};

pub mod def;
pub mod expr;

/// Parses any token.
fn token<'i, S>(input: &mut S) -> PResult<Spanned<Token<'i>>>
where
    S: Stream<Token = Spanned<Token<'i>>>,
{
    input
        .next_token()
        .ok_or(ErrMode::Backtrack(ContextError::new()))
}

/// Parses any identifier.
fn ident<'i, S>(input: &mut S) -> PResult<Spanned<&'i str>>
where
    S: Stream<Token = Spanned<Token<'i>>>,
{
    token
        .verify_map(|Spanned(tok, span)| match tok {
            Token::Ident(ident) => Some(Spanned::new(ident, span)),
            _ => None,
        })
        .context(StrContext::Expected(StrContextValue::Description(
            "an identifier",
        )))
        .parse_next(input)
}

/// Parse the given keyword.
fn keyword<'i, S>(kw: &'static str) -> impl FnMut(&mut S) -> PResult<Spanned<()>>
where
    S: Stream<Token = Spanned<Token<'i>>>,
{
    move |input: &mut S| {
        ident
            .verify_map(|Spanned(str, span)| {
                if str == kw {
                    Some(Spanned::new((), span))
                } else {
                    None
                }
            })
            .context(StrContext::Expected(StrContextValue::StringLiteral(kw)))
            .parse_next(input)
    }
}

/// Parses the given symbol.
fn symbol<'sym, 'i, S>(symbol: &'sym Symbol) -> impl FnMut(&mut S) -> PResult<Spanned<()>> + 'sym
where
    S: Stream<Token = Spanned<Token<'i>>>,
{
    move |input: &mut S| {
        token
            .verify_map(|Spanned(tok, span)| match tok {
                Token::Symbol(sym) if sym == *symbol => Some(Spanned::new((), span)),
                _ => None,
            })
            .context(StrContext::Expected(symbol.str_context_value()))
            .parse_next(input)
    }
}

/// Parses a newline.
/// This can be either '\n' or ';'.
fn newline<'i, S>(input: &mut S) -> PResult<Spanned<()>>
where
    S: Stream<Token = Spanned<Token<'i>>>,
{
    token
        .verify_map(|Spanned(tok, span)| match tok {
            Token::NewLine => Some(Spanned::new((), span)),
            _ => None,
        })
        .parse_next(input)
}
