use ria_lexer::{Spanned, Symbol, Token};
use winnow::{combinator::repeat, stream::Stream, PResult, Parser};

use super::{expr::Expr, ident, symbol};

struct DefList<'i> {
    defs: Box<[Def<'i>]>,
}

impl<'i> DefList<'i> {
    pub fn parse<S>(input: &mut S) -> PResult<DefList<'i>>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        let defs: Vec<_> = repeat(0.., Def::parse).parse_next(input)?;
        Ok(DefList {
            defs: defs.into_boxed_slice(),
        })
    }
}

pub struct Def<'i> {
    ident: Spanned<&'i str>,
    expr: Expr<'i>,
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

#[cfg(test)]
mod test {
    use ria_lexer::{Lexer, Spanned, Token};
    use winnow::Parser;

    use crate::{def::Def, expr::Expr};

    #[test]
    fn parse_simple_def() {
        let input = "x = y";
        let lexer = Lexer::new(input);
        let tokens: Box<[Spanned<Token>]> = lexer.collect();

        let def = Def::parse.parse(tokens.as_ref()).unwrap();

        assert_eq!(def.ident.inner(), "x");
        assert!(matches!(def.expr, Expr::Variable(Spanned("y", _))));
    }
}
