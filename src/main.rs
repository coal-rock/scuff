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
    makefile::{MakefileData, SpriteStageData},
    parser::{Parser, Stmt},
    token::Token,
};
use std::{collections::HashMap, env, fs::read_to_string, path::PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() > 1, "Expected file path.");

    let makefile = MakefileData::parse(args[1].clone().into());
    println!("{:#?}", makefile);

    let mut sprites_stages: HashMap<SpriteStageData, Vec<Stmt>> = HashMap::new();

    for sprite_stage in makefile.sprites_stages {
        let mut lexer = Lexer::new(&sprite_stage.script);
        let tokens = lexer.lex();

        let mut parser = Parser::new(tokens);
        sprites_stages.insert(sprite_stage, parser.parse());
    }

    println!("{:#?}", sprites_stages);

    // let mut lexer = Lexer::new(file_contents);
    // let tokens = lexer.lex();
    //
    // let mut parser = Parser::new(tokens.clone());
    // let ast = parser.parse();
    //
    // println!("{:#?}", &ast);
    //
    // let mut compiler = Compiler::new(ast);
    // let project = compiler.compile();
    //
    // println!("{}", serde_json::to_string_pretty(project).unwrap());
}
