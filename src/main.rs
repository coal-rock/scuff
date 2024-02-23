mod error;
mod lexer;

use crate::lexer::Lexer;
use std::{env, fs::read_to_string};

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() > 1, "Expected file path.");

    let file_contents = read_to_string(args[1].clone()).expect("Could not read file");
    let mut lexer = Lexer::new(file_contents);

    let tokens = lexer.lex();
    println!("{:#?}", tokens);
}
