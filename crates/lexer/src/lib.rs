use std::{borrow::Borrow, ops::Range};

use winnow::{
    ascii::newline,
    combinator::{alt, repeat, trace},
    error::StrContextValue,
    stream::{AsChar, Compare, Location, Stream, StreamIsPartial},
    token::{one_of, take_while},
    LocatingSlice, ModalResult, Parser,
};

#[derive(Debug, Clone)]
pub struct Lexer<'i> {
    remaining: LocatingSlice<&'i str>,
}

impl<'i> Lexer<'i> {
    pub fn new(input: &'i str) -> Self {
        Self {
            remaining: LocatingSlice::new(input),
        }
    }
}

impl<'i> Lexer<'i> {
    fn eat_whitespace(&mut self) -> Option<Spanned<Token<'i>>> {
        let mut first_newline = None;

        loop {
            let _ = trace(
                "ria::whitespace",
                repeat::<_, _, Vec<_>, (), _>(0.., one_of((' ', '\t', '\r'))),
            )
            .parse_next(&mut self.remaining);

            let checkpoint = self.remaining;
            match trace("ria::Token", Token::parse)
                .with_span()
                .parse_next(&mut self.remaining)
                .map(Spanned::from)
            {
                Ok(spanned_tok @ Spanned(Token::NewLine | Token::Semi, _)) => {
                    if first_newline.is_none() {
                        first_newline = Some(spanned_tok);
                    }
                }
                _ => {
                    self.remaining = checkpoint;
                    return first_newline;
                }
            }
        }
    }
}

impl<'i> Iterator for Lexer<'i> {
    type Item = Spanned<Token<'i>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(newline) = self.eat_whitespace() {
            return Some(newline);
        };

        Token::parse
            .with_span()
            .map(Spanned::from)
            .parse_next(&mut self.remaining)
            .ok()
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
    Semi,
    Symbol(Symbol),
    Ident(&'i str),
}

impl<'i> Token<'i> {
    fn parse_kw<S>(input: &mut S) -> ModalResult<Self>
    where
        S: Stream + StreamIsPartial + Compare<&'static str>,
    {
        trace("ria::parse_kw", Symbol::parse)
            .map(Token::Symbol)
            .parse_next(input)
    }

    fn parse_ident<S>(input: &mut S) -> ModalResult<Self>
    where
        S: Stream<Token = char, Slice = &'i str> + StreamIsPartial + Compare<&'static str>,
    {
        trace("ria::parse_ident", parse_ident)
            .map(Token::Ident)
            .parse_next(input)
    }

    fn parse_semi<S>(input: &mut S) -> ModalResult<Self>
    where
        S: Stream + StreamIsPartial + Compare<char>,
    {
        trace("ria::parse_semi", ';')
            .value(Self::Semi)
            .parse_next(input)
    }

    fn parse_newline<S>(input: &mut S) -> ModalResult<Self>
    where
        S: Stream<Token = char, Slice = &'i str> + StreamIsPartial + Compare<char>,
    {
        // lines can be separated with '\n' or ';'
        trace("ria::parse_newline", newline)
            .value(Self::NewLine)
            .parse_next(input)
    }

    /// Parse a token.
    pub fn parse<S>(input: &mut S) -> ModalResult<Self>
    where
        S: Stream<Token = char, Slice = &'i str>
            + StreamIsPartial
            + Compare<&'static str>
            + Compare<char>,
    {
        trace(
            "ria::Token",
            alt((
                Self::parse_kw,
                Self::parse_ident,
                Self::parse_newline,
                Self::parse_semi,
            )),
        )
        .parse_next(input)
    }
}

fn is_ident_first(c: char) -> bool {
    c.is_alpha() || c == '_'
}

fn is_ident_rest(c: char) -> bool {
    is_ident_first(c) || c.is_dec_digit()
}

fn take_ident<S>() -> impl FnMut(&mut S) -> ModalResult<()>
where
    S: Stream<Token = char> + StreamIsPartial,
{
    move |input| {
        let _ = take_while(1, is_ident_first).parse_next(input)?;
        let _ = take_while(0.., is_ident_rest).parse_next(input)?;
        Ok(())
    }
}

fn parse_ident<'i, S>(input: &mut S) -> ModalResult<&'i str>
where
    S: Stream<Token = char, Slice = &'i str> + StreamIsPartial,
{
    take_ident().take().parse_next(input)
}

macro_rules! symbols {
    ($str:literal => $sym:ident $(, $strs:literal => $syms:ident)*,) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub enum Symbol {
            $sym
            $(, $syms)*
        }

        impl Symbol {
            pub fn parse<S>(input: &mut S) -> ModalResult<Self>
            where
                S: Stream + Compare<&'static str> + StreamIsPartial,
            {
                use Symbol::*;

                winnow::combinator::alt((
                    winnow::combinator::trace(concat!("ria::Symbol::", stringify!($sym)), $str.value($sym))
                    $(, winnow::combinator::trace(concat!("ria::Symbol::", stringify!($syms)), $strs.value($syms)))*
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
    ")"  => CloseParen,
}

#[cfg(test)]
mod test {
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

        // starts/ends with underscore is Ok
        works("_a");
        works("_1");
        works("a_");
        works("__");
        works("_a_");

        // starts with alpha is Ok
        works("foo");

        // contains underscore is Ok
        works("foo_bar");
    }

    #[test]
    fn parse_symbols() {
        // lambda
        assert!(Symbol::parse
            .parse_peek("\\")
            .is_ok_and(|(_, x)| x == Symbol::Lambda));

        // arrow
        assert!(Symbol::parse
            .parse_peek("->")
            .is_ok_and(|(_, x)| x == Symbol::Arrow));

        // define
        assert!(Symbol::parse
            .parse_peek("=")
            .is_ok_and(|(_, x)| x == Symbol::Define));
    }

    #[test]
    fn parse_newlines() {
        let tokens: Box<_> = Lexer::new("x\n  y").collect();
        assert_eq!(tokens.len(), 3);
    }
}
