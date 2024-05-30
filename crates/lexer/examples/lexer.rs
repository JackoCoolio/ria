use std::{env::args, io::Read};

use atty::Stream;
use ria_lexer::{Lexer, Spanned};

fn read_input() -> std::io::Result<String> {
    if atty::is(Stream::Stdin) {
        let mut args = args();
        let filepath = args
            .nth(1)
            .expect("usage: cargo run --example lexer <filepath>");
        std::fs::read_to_string(filepath)
    } else {
        let mut string = String::new();
        std::io::stdin().read_to_string(&mut string)?;
        Ok(string)
    }
}

fn main() {
    let input = read_input().expect("unable to read input");
    let lexer = Lexer::new(&input);
    let tokens: Vec<_> = lexer.collect();

    for Spanned(tok, span) in tokens.into_iter() {
        let input_span = &input[span.clone()];
        println!("{span:?}:\n\t'{input_span}'\n->\t{tok:?}");
    }
}
