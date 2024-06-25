use ria_lexer::{Spanned, Symbol, Token};
use winnow::{
    combinator::separated,
    error::{StrContext, StrContextValue},
    stream::Stream,
    PResult, Parser,
};

use crate::newline;

use super::{expr::Expr, ident, symbol};

#[derive(Debug, PartialEq, Eq)]
pub struct DefList<'i> {
    pub defs: Box<[Def<'i>]>,
}

impl<'i> DefList<'i> {
    pub fn parse<S>(input: &mut S) -> PResult<DefList<'i>>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        let defs: Vec<_> = separated(0.., Def::parse, newline)
            .context(StrContext::Label("def list"))
            .parse_next(input)?;
        Ok(DefList {
            defs: defs.into_boxed_slice(),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Def<'i> {
    pub ident: Spanned<&'i str>,
    pub expr: Expr<'i>,
}

impl<'i> Def<'i> {
    pub fn parse<S>(input: &mut S) -> PResult<Def<'i>>
    where
        S: Stream<Token = Spanned<Token<'i>>>,
    {
        let ident = ident
            .context(StrContext::Label("identifier"))
            .parse_next(input)?;
        symbol(&Symbol::Define).parse_next(input)?;
        let expr = Expr::parse
            .context(StrContext::Expected(StrContextValue::Description(
                "an expression",
            )))
            .parse_next(input)?;

        Ok(Self { ident, expr })
    }
}

#[cfg(test)]
mod test {
    use ria_lexer::{Lexer, Spanned, Token};
    use winnow::Parser;

    use crate::{def::Def, expr::Expr};

    use super::DefList;

    macro_rules! assert_def_eq {
        ($def:expr, $ident:ident = $pat:pat) => {{
            assert_eq!(*$def.ident.inner(), stringify!($ident));
            assert!(matches!($def.expr, $pat));
        }};
    }

    #[test]
    fn parse_simple_def() {
        let input = "x = y";
        let lexer = Lexer::new(input);
        let tokens: Box<[Spanned<Token>]> = lexer.collect();

        let def = Def::parse.parse(tokens.as_ref()).unwrap();

        assert_def_eq!(def, x = Expr::Variable(Spanned("y", _)));
    }

    #[test]
    fn parse_def_list() {
        let input = "x = y\ny = z";
        let lexer = Lexer::new(input);
        let tokens: Box<[Spanned<Token>]> = lexer.collect();

        let def_list = DefList::parse.parse(tokens.as_ref()).unwrap();

        assert_def_eq!(&def_list.defs[0], x = Expr::Variable(Spanned("y", _)));
        assert_def_eq!(&def_list.defs[1], y = Expr::Variable(Spanned("z", _)));
    }
}
