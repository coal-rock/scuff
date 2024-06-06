mod compiler;
mod error;
mod lexer;
mod makefile;
mod parser;
mod project;
mod token;

use crate::{
    compiler::Compiler,
    lexer::Lexer,
    makefile::{MakefileData, TargetData},
    parser::{Parser, Stmt},
};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() > 1, "Expected file path.");

    let makefile = MakefileData::parse(args[1].clone().into());
    println!("{:#?}", makefile);

    let mut targets: Vec<(TargetData, Vec<Stmt>)> = vec![];

    for target in makefile.targets {
        let mut lexer = Lexer::new(&target.script);
        let tokens = lexer.lex();

        let mut parser = Parser::new(tokens);
        targets.push((target, parser.parse()));
    }

    println!("{:#?}", targets);

    let mut compiler = Compiler::new(targets);
    let project = compiler.compile();

    println!("{}", serde_json::to_string_pretty(project).unwrap());
}
