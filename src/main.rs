mod compiler;
mod error;
mod lexer;
mod parser;
mod project;
mod token;

use crate::{compiler::Compiler, lexer::Lexer, parser::Parser};
use std::{env, fs::read_to_string};

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() > 1, "Expected file path.");

    let file_contents = read_to_string(args[1].clone()).expect("Could not read file");

    let mut lexer = Lexer::new(file_contents);
    let tokens = lexer.lex();

    let mut parser = Parser::new(tokens.clone());
    let ast = parser.parse();

    println!("{:#?}", &ast);

    let mut compiler = Compiler::new(ast);
    let project = compiler.compile();

    println!("{}", serde_json::to_string_pretty(project).unwrap());
}
