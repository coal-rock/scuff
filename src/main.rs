mod compiler;
mod error;
mod lexer;
mod makefile;
mod parser;
mod project;
mod token;

use crate::{compiler::Compiler, lexer::Lexer, makefile::Makefile, parser::Parser};
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

    let makefile: Makefile = toml::from_str(
        r#"
            project_name = "Example Project"
            extensions = [
                "Pen", "Music", "VideoSensing", "Text2Speech", "Translate",
                "Makeymakey", "Microbit", "EV3", "Boost", "Wedo2", "Gdxfor"
            ]

            [[stage]]
            name = "Stage 1"
            script = "scripts/stage1.sb3"
            backdrops = [
                { name = "Backdrop 1", path = "assets/backdrops/backdrop1.png" },
                { name = "Backdrop 2", path = "assets/backdrops/backdrop2.png" }
            ]
            sounds = [
                { name = "Sound 1", path = "assets/sounds/stage1/sound1.wav" },
                { name = "Sound 2", path = "assets/sounds/stage1/sound2.wav" }
            ]

            [[sprite]]
            name = "Sprite 1"
            script = "scripts/sprite1.sb3"
            costumes = [
                { name = "Costume 1", path = "assets/costumes/sprite1/costume1.png" },
                { name = "Costume 2", path = "assets/costumes/sprite1/costume2.png" }
            ]
            sounds = [
                { name = "Sound 1", path = "assets/sounds/sprite1/sound1.wav" },
                { name = "Sound 2", path = "assets/sounds/sprite1/sound2.wav" }
            ]

            [[sprite]]
            name = "Sprite 2"
            script = "scripts/sprite2.sb3"
            costumes = [
                { name = "Costume 1", path = "assets/costumes/sprite2/costume1.png" },
                { name = "Costume 2", path = "assets/costumes/sprite2/costume2.png" }
            ]
            sounds = [
                { name = "Sound 1", path = "assets/sounds/sprite2/sound1.wav" },
                { name = "Sound 2", path = "assets/sounds/sprite2/sound2.wav" }
            ]
        "#,
    )
    .unwrap();

    println!("{:#?}", makefile);

    // println!("{}", toml::to_string(&makefile).unwrap());
}
