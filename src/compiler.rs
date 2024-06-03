use std::collections::HashMap;

use serde_json::json;

use crate::parser::{Event, Stmt};
use crate::project::{Block, Project};

pub struct Compiler {
    ast: Vec<Stmt>,
    project: Project,
    block_id: usize,
    parent: Option<usize>,
}

impl Compiler {
    pub fn new(ast: Vec<Stmt>) -> Compiler {
        Compiler {
            ast,
            project: Project::new(),
            block_id: 0,
            parent: None,
        }
    }

    pub fn compile(&mut self) -> &Project {
        let ast = self.ast.clone();

        for statement in ast {
            self.compile_statement(&statement, None, None, None);
        }
        //
        &self.project
    }

    fn compile_statement(
        &mut self,
        statement: &Stmt,
        parent_id: Option<usize>,
        current_id: Option<usize>,
        next_id: Option<usize>,
    ) {
        match statement {
            Stmt::EventHandler(event, body) => {
                let current_id = self.gen_id();

                match event {
                    Event::FlagClicked => {
                        self.push_block(
                            &Block {
                                opcode: "event_whenflagclicked".to_string(),
                                next: Some((current_id + 1).to_string()),
                                top_level: Some(true),
                                ..Block::default()
                            },
                            current_id,
                        );

                        for (idx, stmt) in body.iter().enumerate() {
                            let id = self.gen_id();

                            let next_id = if (idx + 1) == body.len() {
                                None
                            } else {
                                Some(id + 1)
                            };

                            self.compile_statement(stmt, Some(current_id), Some(id), next_id);
                        }
                    }
                    Event::KeyPressed(_) => todo!(),
                }
            }
            Stmt::FunctionCall(func_name, args) => {
                let opcode = match func_name.as_str() {
                    "say" => "looks_say",
                    _ => "procedures_call",
                };

                match opcode {
                    "looks_say" => match &args[0] {
                        crate::parser::Expr::Number(_) => todo!(),
                        crate::parser::Expr::String(string) => {
                            let mut inputs = HashMap::new();
                            let value = json!([1, [10, string,]]);

                            inputs.insert("MESSAGE".to_string(), value);

                            self.push_block(
                                &Block {
                                    opcode: "looks_say".to_string(),
                                    parent: Some(parent_id.unwrap().to_string()),
                                    inputs: Some(inputs),
                                    next: next_id.map(|id| id.to_string()),
                                    ..Block::default()
                                },
                                current_id.unwrap(),
                            );
                        }
                        crate::parser::Expr::Identifier(_) => todo!(),
                        crate::parser::Expr::Bool(_) => todo!(),
                        crate::parser::Expr::Binary(_, _, _) => todo!(),
                    },
                    _ => panic!(),
                }

                // self.push_block(block, id)
            }
            _ => panic!(),
        }
    }

    fn compile_body(&mut self, body: &Vec<Stmt>, current_id: usize, parent_id: usize) {
        // for
    }

    fn push_block(&mut self, block: &Block, id: usize) {
        self.project.targets[0]
            .blocks
            .insert(id.to_string(), block.clone());
    }

    fn gen_id(&mut self) -> usize {
        self.block_id += 1;
        self.block_id
    }
}
