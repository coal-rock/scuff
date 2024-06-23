#![recursion_limit = "256"]
mod compilation_test;
mod compiler;
mod error;
mod lexer;
mod makefile;
mod packager;
mod parser;
mod project;
mod token;
mod validate;

use project::Project;

use crate::{
    compiler::Compiler,
    lexer::Lexer,
    makefile::{MakefileData, TargetData},
    packager::package_project,
    parser::{Parser, Stmt},
    validate::validate_project,
};

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() > 1, "Expected file path.");
    let (project, targets) = compile_project(args[1].clone());

    println!("{}", serde_json::to_string_pretty(&project).unwrap());
    validate_project(&project);
    package_project(&project, targets, "project.sb3".into());
    println!("project written to: project.sb3");
}

pub fn compile_project(makefile_path: String) -> (Project, Vec<(TargetData, Vec<Stmt>)>) {
    let makefile = MakefileData::parse(makefile_path.clone().into());
    // println!("{:#?}", makefile);

    let mut targets: Vec<(TargetData, Vec<Stmt>)> = vec![];

    for target in makefile.targets {
        let mut lexer = Lexer::new(&target.script);
        let tokens = lexer.lex();

        // println!("------------------------------------------------------------------");
        // println!("{:#?}", tokens);
        // println!("------------------------------------------------------------------");

        let mut parser = Parser::new(tokens);
        targets.push((target, parser.parse()));
    }

    // println!("{:#?}", targets);

    let mut compiler = Compiler::new(targets.clone());
    (compiler.compile().clone(), targets)
}
