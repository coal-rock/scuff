#![recursion_limit = "256"]
mod compiler;
mod error;
mod lexer;
mod makefile;
mod packager;
mod parser;
mod project;
mod token;
mod validate;

use crate::{
    compiler::Compiler,
    lexer::Lexer,
    makefile::{MakefileData, TargetData},
    parser::{Parser, Stmt},
    validate::validate_project,
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
    validate_project(project);
}
