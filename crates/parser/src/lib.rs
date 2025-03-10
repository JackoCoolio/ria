#![allow(dead_code)]

use ria_lexer::{Spanned, Symbol, Token};
use winnow::{
    error::{ContextError, ErrMode, StrContext, StrContextValue},
    stream::Stream,
    ModalResult, Parser,
};

pub mod def;
pub mod expr;
pub mod module;

/// Parses any token.
fn token<'i, S>(input: &mut S) -> ModalResult<Spanned<Token<'i>>>
where
    S: Stream<Token = Spanned<Token<'i>>>,
{
    input
        .next_token()
        .ok_or(ErrMode::Backtrack(ContextError::new()))
}

/// Parses any identifier.
fn ident<'i, S>(input: &mut S) -> ModalResult<Spanned<&'i str>>
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
fn keyword<'i, S>(kw: &'static str) -> impl FnMut(&mut S) -> ModalResult<Spanned<()>>
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
/// Also adds context that the symbol was expected.
fn symbol<'sym, 'i, S>(
    symbol: &'sym Symbol,
) -> impl FnMut(&mut S) -> ModalResult<Spanned<()>> + 'sym
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

fn maybe_newline<'i, S>(input: &mut S) -> Option<Spanned<()>>
where
    S: Stream<Token = Spanned<Token<'i>>>,
{
    match newline.parse_next(input) {
        Ok(x) => Some(x.map(|_| ())),
        _ => None,
    }
}

#[test]
fn test_allow_newline() {
    let mut tokens = [Spanned(Token::NewLine, 0..0)].as_slice();
    maybe_newline(&mut tokens).expect("should eat a newline");

    let mut tokens = [Spanned(Token::Ident("foo"), 0..0)].as_slice();
    assert!(maybe_newline(&mut tokens).is_none());
}

fn newline<'i, S>(input: &mut S) -> ModalResult<Spanned<()>>
where
    S: Stream<Token = Spanned<Token<'i>>>,
{
    token
        .verify_map(|Spanned(tok, span)| match tok {
            Token::NewLine | Token::Semi => Some(Spanned::new((), span)),
            _ => None,
        })
        .context(StrContext::Expected(StrContextValue::Description(
            "a newline",
        )))
        .parse_next(input)
}
