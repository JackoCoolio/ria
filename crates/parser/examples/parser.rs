use std::{env::args, error::Error};

use ria_lexer::{Lexer, Spanned, Token};
use ria_parser::expr::Expr;
use winnow::{
    stream::{Stream, StreamIsPartial},
    Parser,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = args().skip(1);
    let filepath = args.next().unwrap();
    let parser = args.next().unwrap();

    let input = std::fs::read_to_string(filepath).unwrap();
    let tokens = Lexer::new(&input).collect::<Box<_>>();

    match do_parse(&parser, tokens.as_ref()) {
        Ok(ast) => println!("success:\n{}", ast),
        Err(err) => eprintln!("error:\n{}", err),
    }

    Ok(())
}

fn do_parse<'i, S>(parser: &str, tokens: S) -> Result<String, String>
where
    S: Stream<Token = Spanned<Token<'i>>> + Clone + StreamIsPartial + Sized,
{
    match parser.to_lowercase().as_str() {
        "expr" => Expr::parse
            .parse(tokens)
            .map(|expr| format!("{expr:?}"))
            .map_err(|e| format!("{e:?}")),
        _ => unimplemented!(),
    }
}
