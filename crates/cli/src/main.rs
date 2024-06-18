use std::path::PathBuf;

use clap::Parser;
use ria_lexer::Lexer;
use ria_parser::expr::Expr;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The source filepath
    #[arg(name = "file")]
    source_file: PathBuf,
}

fn main() {
    let args = Args::parse();

    let source = std::fs::read_to_string(args.source_file).unwrap();

    let tokens = Lexer::new(&source).collect::<Box<_>>();

    println!("lexed:\n{:?}", tokens);

    let ast = Expr::parse(&mut tokens.as_ref()).unwrap();

    println!("parsed:\n{:?}", ast);
}
