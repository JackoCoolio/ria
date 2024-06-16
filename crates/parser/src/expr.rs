use ria_lexer::{Spanned, Symbol, Token};
use winnow::{
    combinator::{alt, cut_err, delimited, opt, preceded},
    stream::Stream,
    PResult, Parser,
};

use crate::{def::DefList, newline};

use super::{ident, symbol};

#[derive(Debug, PartialEq, Eq)]
pub enum Expr<'i> {
    Variable(Spanned<&'i str>),
    Lambda(Lambda<'i>),
    Block(Block<'i>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Lambda<'i> {
    param: Spanned<&'i str>,
    body: Box<Expr<'i>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Block<'i> {
    defs: DefList<'i>,
    expr: Option<Box<Expr<'i>>>,
}

impl<'i> Block<'i> {
    pub fn parse<S>(input: &mut S) -> PResult<Self>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        delimited(
            symbol(&Symbol::OpenParen),
            |input: &mut S| {
                let defs = DefList::parse.parse_next(input)?;
                let expr = opt(preceded(newline, Expr::parse))
                    .map(|expr| expr.map(Box::from))
                    .parse_next(input)?;

                Ok(Block { defs, expr })
            },
            symbol(&Symbol::CloseParen),
        )
        .parse_next(input)
    }
}

impl<'i> Expr<'i> {
    pub fn parse<S>(input: &mut S) -> PResult<Expr<'i>>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        alt((
            ident.map(Expr::Variable),
            Expr::parse_lambda.map(Expr::Lambda),
            Block::parse.map(Expr::Block),
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
    use ria_lexer::{Lexer, Spanned, Token};
    use winnow::Parser;

    use crate::{
        def::{Def, DefList},
        expr::{Block, Expr, Lambda},
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

    #[test]
    fn parse_block() {
        let tokens: Box<_> = Lexer::new("(x = y; x)").collect();
        let expr = Expr::parse
            .parse(tokens.as_ref())
            .expect("block should parse");

        assert_eq!(
            expr,
            Expr::Block(Block {
                defs: DefList {
                    defs: [Def {
                        ident: Spanned("x", 1..2),
                        expr: Expr::Variable(Spanned("y", 5..6)),
                    }]
                    .into(),
                },
                expr: Some(Expr::Variable(Spanned("x", 8..9)).into()),
            }),
        );
    }
}
