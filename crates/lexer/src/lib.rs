#![feature(assert_matches)]
#![allow(dead_code)]

use std::{borrow::Borrow, ops::Range};

use winnow::{
    ascii::{newline, space0},
    combinator::{alt, opt},
    error::StrContextValue,
    stream::{AsChar, Compare, Location, Stream, StreamIsPartial},
    token::{one_of, take_while},
    Located, PResult, Parser,
};

#[derive(Debug, Clone)]
pub struct Lexer<'i> {
    remaining: Located<&'i str>,
    last_was_newline: bool,
}

impl<'i> Lexer<'i> {
    pub fn new(mut input: &'i str) -> Self {
        // parse any leading spaces
        let _ = space0::<_, ()>.parse_next(&mut input);

        Self {
            remaining: Located::new(input),
            // there might have been a leading newline, but we don't really care
            last_was_newline: false,
        }
    }
}

impl<'i> Iterator for Lexer<'i> {
    type Item = Spanned<Token<'i>>;

    fn next(&mut self) -> Option<Self::Item> {
        let ((token, span), _) = (Token::parse.with_span(), opt(one_of((' ', '\t', '\r'))))
            .parse_next(&mut self.remaining)
            .ok()?;

        // we return the first occurrence of a newline, then subsequent
        // occurrences are skipped
        if let Token::NewLine = token {
            if self.last_was_newline {
                return self.next();
            } else {
                self.last_was_newline = true;
            }
        } else {
            self.last_was_newline = false;
        }

        Some(Spanned::from((token, span)))
    }
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
    #[allow(clippy::len_without_is_empty)] // never empty
    pub fn len(&self) -> usize {
        let Spanned(_, range) = self;
        range.len()
    }

    /// Returns a reference to the inner token.
    pub fn inner(&self) -> &T {
        &self.0
    }

    /// Maps the inner token with `func`.
    pub fn map<U>(self, func: impl FnOnce(T) -> U) -> Spanned<U> {
        let Spanned(token, range) = self;
        Spanned(func(token), range)
    }

    pub fn start(&self) -> usize {
        self.1.start
    }

    pub fn end(&self) -> usize {
        self.1.end
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
    NewLine,
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

    fn parse_newline<S>(input: &mut S) -> PResult<Self>
    where
        S: Stream<Token = char, Slice = &'i str> + StreamIsPartial + Compare<char>,
    {
        // lines can be separated with '\n' or ';'
        alt((newline, ';')).value(Self::NewLine).parse_next(input)
    }

    /// Parse a token.
    pub fn parse<S>(input: &mut S) -> PResult<Self>
    where
        S: Stream<Token = char, Slice = &'i str>
            + StreamIsPartial
            + Compare<&'static str>
            + Compare<char>,
    {
        alt((Self::parse_kw, Self::parse_ident, Self::parse_newline)).parse_next(input)
    }
}

fn is_ident_first(c: char) -> bool {
    c.is_alpha() || c == '_'
}

fn is_ident_rest(c: char) -> bool {
    is_ident_first(c) || c.is_dec_digit()
}

fn take_ident<S>() -> impl FnMut(&mut S) -> PResult<()>
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
    take_ident().recognize().parse_next(input)
}

macro_rules! symbols {
    ($str:literal => $sym:ident $(, $strs:literal => $syms:ident)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub enum Symbol {
            $sym
            $(, $syms)*
        }

        impl Symbol {
            pub fn parse<S>(input: &mut S) -> PResult<Self>
            where
                S: Stream + Compare<&'static str> + StreamIsPartial,
            {
                use Symbol::*;

                winnow::combinator::alt((
                    $str.value($sym)
                    $(, $strs.value($syms))*
                )).parse_next(input)
            }

            /// Returns the StrContextValue for the Symbol.
            pub const fn str_context_value(&self) -> winnow::error::StrContextValue {
                use Symbol::*;

                StrContextValue::StringLiteral(match (self) {
                    $sym => $str
                    $(, $syms => $strs)*
                })
            }
        }
    };
}

symbols! {
    "\\" => Lambda,
    "->" => Arrow,
    "="  => Define,
    "("  => OpenParen,
    ")"  => CloseParen
}

#[cfg(test)]
mod test {
    use std::assert_matches::assert_matches;

    use winnow::Parser;

    use super::{parse_ident, Lexer, Symbol, Token};

    #[test]
    fn lex_tokens() {
        let input = "identity = \\x -> x";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next().unwrap().inner(), &Token::Ident("identity"));
        assert_eq!(
            lexer.next().unwrap().inner(),
            &Token::Symbol(Symbol::Define)
        );
        assert_eq!(
            lexer.next().unwrap().inner(),
            &Token::Symbol(Symbol::Lambda)
        );
        assert_eq!(lexer.next().unwrap().inner(), &Token::Ident("x"));
        assert_eq!(lexer.next().unwrap().inner(), &Token::Symbol(Symbol::Arrow));
        assert_eq!(lexer.next().unwrap().inner(), &Token::Ident("x"));
        assert!(lexer.next().is_none()); // empty now
    }

    #[test]
    fn parse_idents() {
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
    fn parse_symbols() {
        // lambda
        assert_matches!(Symbol::parse.parse_peek("\\"), Ok((_, Symbol::Lambda)));

        // arrow
        assert_matches!(Symbol::parse.parse_peek("->"), Ok((_, Symbol::Arrow)));

        // define
        assert_matches!(Symbol::parse.parse_peek("="), Ok((_, Symbol::Define)));
    }
}
