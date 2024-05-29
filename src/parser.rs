use winnow::{
    combinator::{alt, cut_err},
    error::{AddContext, ContextError, ErrMode, InputError},
    stream::{Location, Stream},
    PResult, Parser,
};

use crate::lexer::{Lexer, Spanned, Symbol, Token};

struct Module<'i> {
    defs: Box<[Def<'i>]>,
}

struct Def<'i> {
    ident: Spanned<&'i str>,
    expr: Expr<'i>,
}

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

impl<'i> Def<'i> {
    pub fn parse<S>(input: &mut S) -> PResult<Def<'i>>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        let ident = ident.parse_next(input)?;
        let _ = symbol(&Symbol::Define).parse_next(input)?;
        let expr = Expr::parse.parse_next(input)?;

        Ok(Self { ident, expr })
    }
}

#[test]
fn parse_simple_def() {
    let input = "x = y";
    let lexer = Lexer::new(input);
    let tokens: Box<[Spanned<Token>]> = lexer.collect();

    let def = Def::parse.parse(tokens.as_ref()).unwrap();

    assert_eq!(def.ident.inner(), "x");
    assert!(matches!(def.expr, Expr::Variable(Spanned("y", _))));
}

#[derive(Debug, PartialEq, Eq)]
enum Expr<'i> {
    Variable(Spanned<&'i str>),
    Lambda(Lambda<'i>),
}

#[derive(Debug, PartialEq, Eq)]
struct Lambda<'i> {
    param: Spanned<&'i str>,
    body: Box<Expr<'i>>,
}

impl<'i> Expr<'i> {
    pub fn parse<S>(input: &mut S) -> PResult<Expr<'i>>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        alt((
            ident.map(Expr::Variable),
            Expr::parse_lambda.map(Expr::Lambda),
        ))
        .parse_next(input)
    }

    /// Parses a [Lambda].
    fn parse_lambda<S>(input: &mut S) -> PResult<Lambda<'i>>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        let _ = symbol(&Symbol::Lambda).parse_next(input)?;
        let param = cut_err(ident).parse_next(input)?;
        let _ = symbol(&Symbol::Arrow).parse_next(input)?;
        let body = Expr::parse.parse_next(input)?;
        Ok(Lambda {
            param,
            body: Box::new(body),
        })
    }
}

#[test]
fn parse_arity_1_lambda() {
    let tokens: Box<[Spanned<Token>]> = Lexer::new("\\x -> x").collect();
    let expr = Expr::parse
        .parse(tokens.as_ref())
        .expect("simple lambda expr should parse");
    assert_eq!(
        expr,
        Expr::Lambda(Lambda {
            param: Spanned::new("x", 1..2),
            body: Box::new(Expr::Variable(Spanned::new("x", 6..7)))
        })
    );
}

#[test]
fn parse_arity_2_lambda() {
    let tokens: Box<_> = Lexer::new("\\x -> \\y -> z").collect();
    let expr = Expr::parse
        .parse(tokens.as_ref())
        .expect("arity 2 lambda expr should parse");

    assert_eq!(
        expr,
        Expr::Lambda(Lambda {
            param: Spanned::new("x", 1..2),
            body: Box::new(Expr::Lambda(Lambda {
                param: Spanned::new("y", 7..8),
                body: Box::new(Expr::Variable(Spanned::new("z", 12..13))),
            })),
        }),
    );
}
