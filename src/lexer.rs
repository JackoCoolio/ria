use std::{borrow::Borrow, ops::Range};

use winnow::{
    ascii::space0,
    combinator::alt,
    stream::{AsChar, Compare, Location, Stream, StreamIsPartial},
    token::take_while,
    Located, PResult, Parser,
};

#[derive(Debug, Clone)]
pub struct Lexer<'i> {
    remaining: Located<&'i str>,
    position: usize,
}

impl<'i> Lexer<'i> {
    pub fn new(mut input: &'i str) -> Self {
        // parse any leading spaces
        let _ = space0::<_, ()>.parse_next(&mut input);

        Self {
            remaining: Located::new(input),
            position: 0,
        }
    }
}

impl<'i> Iterator for Lexer<'i> {
    type Item = Spanned<Token<'i>>;

    fn next(&mut self) -> Option<Self::Item> {
        (Token::parse.with_span(), space0)
            // map to Spanned
            .map(|(spanned_tok, _)| Spanned::from(spanned_tok))
            // parse it
            .parse_next(&mut self.remaining)
            // return as Option
            .ok()
    }
}

#[test]
fn test_lexer() {
    let input = "identity = \\x -> x";
    let mut lexer = Lexer::new(input);

    assert_eq!(lexer.next().unwrap().inner(), Token::Ident("identity"));
    assert_eq!(lexer.next().unwrap().inner(), Token::Symbol(Symbol::Define));
    assert_eq!(lexer.next().unwrap().inner(), Token::Symbol(Symbol::Lambda));
    assert_eq!(lexer.next().unwrap().inner(), Token::Ident("x"));
    assert_eq!(lexer.next().unwrap().inner(), Token::Symbol(Symbol::Arrow));
    assert_eq!(lexer.next().unwrap().inner(), Token::Ident("x"));
    assert!(lexer.next().is_none()); // empty now
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spanned<T>(pub T, pub Range<usize>);

impl<T> Location for Spanned<T> {
    fn location(&self) -> usize {
        self.1.start
    }
}

impl<T> Spanned<T> {
    /// Creates a new `Spanned` token with the given `token` and `range`.
    pub fn new(token: T, range: Range<usize>) -> Self {
        Self(token, range)
    }

    /// Returns the length of the Spanned token.
    pub fn len(&self) -> usize {
        let Spanned(_, range) = self;
        range.len()
    }

    /// Unwraps the Spanned token to return the inner token.
    pub fn inner(self) -> T {
        self.0
    }

    /// Maps the inner token with `func`.
    pub fn map<U>(self, func: impl FnOnce(T) -> U) -> Spanned<U> {
        let Spanned(token, range) = self;
        Spanned(func(token), range)
    }
}

impl<T> Borrow<T> for Spanned<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T> From<(T, Range<usize>)> for Spanned<T> {
    fn from((token, range): (T, Range<usize>)) -> Self {
        Self::new(token, range)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'i> {
    Symbol(Symbol),
    Ident(&'i str),
}

impl<'i> Token<'i> {
    fn parse_kw<S>(input: &mut S) -> PResult<Self>
    where
        S: Stream + StreamIsPartial + Compare<&'static str>,
    {
        Symbol::parse.map(Token::Symbol).parse_next(input)
    }

    fn parse_ident<S>(input: &mut S) -> PResult<Self>
    where
        S: Stream<Token = char, Slice = &'i str> + StreamIsPartial + Compare<&'static str>,
    {
        parse_ident.map(Token::Ident).parse_next(input)
    }

    /// Parse a token.
    pub fn parse<S>(input: &mut S) -> PResult<Self>
    where
        S: Stream<Token = char, Slice = &'i str> + StreamIsPartial + Compare<&'static str>,
    {
        alt((Self::parse_kw, Self::parse_ident)).parse_next(input)
    }
}

fn is_ident_first(c: char) -> bool {
    c.is_alpha() || c == '_'
}

fn is_ident_rest(c: char) -> bool {
    is_ident_first(c) || c.is_dec_digit()
}

fn take_indent<S>() -> impl FnMut(&mut S) -> PResult<()>
where
    S: Stream<Token = char> + StreamIsPartial,
{
    move |input| {
        let _ = take_while(1, is_ident_first).parse_next(input)?;
        let _ = take_while(0.., is_ident_rest).parse_next(input)?;
        Ok(())
    }
}

fn parse_ident<'i, S>(input: &mut S) -> PResult<&'i str>
where
    S: Stream<Token = char, Slice = &'i str> + StreamIsPartial,
{
    take_indent().recognize().parse_next(input)
}

#[cfg(test)]
mod test {
    use std::assert_matches::assert_matches;

    use winnow::Parser;

    use crate::lexer::{parse_ident, Symbol};

    #[test]
    fn test_parse_ident() {
        fn fails(s: &str) {
            assert!(parse_ident.parse_peek(s).is_err());
        }

        // empty fails
        fails("");

        // digit fails
        fails("1");

        // starts with digit fails
        fails("1a");

        fn works(s: &str) {
            assert_eq!(parse_ident.parse_peek(s), Ok(("", s)));
        }

        // just underscore is Ok
        works("_");

        // starts with underscore is Ok
        works("_a");
        works("_1");

        // starts with alpha is Ok
        works("foo");

        // contains underscore is Ok
        works("foo_bar");
    }

    #[test]
    fn test_parse_symbol() {
        // lambda
        assert_matches!(Symbol::parse.parse_peek("\\"), Ok((_, Symbol::Lambda)));

        // arrow
        assert_matches!(Symbol::parse.parse_peek("->"), Ok((_, Symbol::Arrow)));

        // define
        assert_matches!(Symbol::parse.parse_peek("="), Ok((_, Symbol::Define)));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    Lambda,
    Arrow,
    Define,
}

impl Symbol {
    /// Parses a [Symbol].
    pub fn parse<S>(input: &mut S) -> PResult<Self>
    where
        S: Stream + Compare<&'static str> + StreamIsPartial,
    {
        use Symbol::*;
        alt(("\\".value(Lambda), "->".value(Arrow), "=".value(Define))).parse_next(input)
    }
}
