use ria_lexer::{Spanned, Symbol, Token};
use winnow::{combinator::separated, stream::Stream, PResult, Parser};

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
        let defs: Vec<_> = separated(1.., Def::parse, newline).parse_next(input)?;
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
