use winnow::{
    combinator::{alt, cut_err},
    stream::Stream,
    PResult, Parser,
};

use crate::lexer::{Spanned, Symbol, Token};

use super::{ident, symbol};

#[derive(Debug, PartialEq, Eq)]
pub enum Expr<'i> {
    Variable(Spanned<&'i str>),
    Lambda(Lambda<'i>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Lambda<'i> {
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

#[cfg(test)]
mod test {
    use winnow::Parser;

    use crate::{
        lexer::{Lexer, Spanned, Token},
        parser::expr::{Expr, Lambda},
    };

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
}
