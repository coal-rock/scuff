use std::collections::HashMap;
use std::io::Write;

use md5::{Digest, Md5};
use serde_json::{json, Value};

use crate::makefile::TargetData;
use crate::parser::{Event, Stmt};
use crate::project::{Block, Costume, Mutation, Project, Target};
use crate::token::Type;

pub struct Compiler {
    targets: Vec<(TargetData, Vec<Stmt>)>,
    current_target: (TargetData, Vec<Stmt>),
    project: Project,
    block_id: usize,
    arg_id: usize,
    /// ```
    /// let function_table = arg_table.get(function_name)?;
    /// let argument_id = function_table.get(argument_name)?;
    arg_table: HashMap<String, HashMap<String, String>>,
    target_index: usize,
    parent: Option<usize>,
}

// TODO: fix excessive cloning
impl Compiler {
    pub fn new(targets: Vec<(TargetData, Vec<Stmt>)>) -> Compiler {
        Compiler {
            targets: targets.clone(),
            current_target: targets[0].clone(),
            project: Project::new(),
            block_id: 0,
            arg_id: 0,
            arg_table: HashMap::new(),
            target_index: 0,
            parent: None,
        }
    }

    pub fn compile(&mut self) -> &Project {
        for target in self.targets.clone() {
            self.current_target = target.clone();
            self.block_id = 0;
            self.parent = None;

            self.project.targets.push(Target {
                is_stage: self.current_target.0.is_stage,
                name: self.current_target.0.name.clone(),
                ..Target::default()
            });

            for costume in &self.current_target.0.costumes {
                let mut hasher = Md5::new();
                hasher.update(&costume.content);
                let hash = format!("{:x}", hasher.finalize());

                // FIXME: what the fuck, rust moment
                let extension = costume
                    .path
                    .extension()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                self.project.targets[self.target_index]
                    .costumes
                    .push(Costume {
                        name: costume.name.clone(),
                        data_format: extension.clone(),
                        asset_id: hash.clone().into(),
                        md5ext: format!("{}.{}", hash, extension),
                        rotation_center_x: None,
                        rotation_center_y: None,
                    })
            }

            let ast = &self.current_target.1.clone();

            for statement in ast {
                self.compile_statement(statement, None, None, None);
            }

            self.target_index += 1;
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
                let current_id = self.gen_block_id();

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
                            let id = self.gen_block_id();

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
            Stmt::FunctionDeclaration(func_name, args, body, return_type) => {
                let prototype_id = self.gen_block_id(); // a
                let definition_id = self.gen_block_id(); // b

                let mut prototype_inputs: HashMap<String, Value> = HashMap::new();
                let mut definition_inputs: HashMap<String, Value> = HashMap::new();

                let mut arg_blocks: Vec<(usize, Block)> = Vec::new();

                let mut proc_code = func_name.clone();
                let mut argument_ids = String::from("[");
                let mut argument_names = String::from("[");
                let mut argument_defaults = String::from("[");

                for (index, (arg_name, arg_type)) in args.iter().enumerate() {
                    let proc_code_frag = match arg_type {
                        Type::Number | Type::String => " %s",
                        Type::Bool => " %b",
                        Type::Table => todo!(),
                        Type::Void => todo!(),
                    };

                    proc_code.push_str(proc_code_frag);

                    if index != 0 {
                        argument_ids.push_str(",");
                        argument_names.push_str(",");
                        argument_defaults.push_str(",");
                    }

                    let arg_id = self.gen_arg_id().to_string();

                    let arg_default = match arg_type {
                        Type::Number | Type::String => "",
                        Type::Bool => "false",
                        Type::Table => todo!(),
                        Type::Void => todo!(),
                    };

                    let arg_block_id = self.gen_block_id();
                    prototype_inputs.insert(arg_id.clone(), json!([1, arg_block_id.to_string()]));

                    let opcode = match arg_type {
                        Type::Number | Type::String => "argument_reporter_string_number",
                        Type::Bool => "argument_reporter_boolean",
                        Type::Table => todo!(),
                        Type::Void => todo!(),
                    };

                    let arg_block = Block {
                        opcode: opcode.to_string(),
                        parent: Some(prototype_id.to_string()),
                        fields: Some(json!({"VALUE": [arg_name, Value::Null]})),
                        shadow: Some(true),
                        top_level: Some(false),
                        ..Default::default()
                    };

                    arg_blocks.push((arg_block_id, arg_block));

                    argument_ids.push_str(&format!("\"{}\"", arg_id));
                    argument_names.push_str(&format!("\"{}\"", arg_name));
                    argument_defaults.push_str(&format!("\"{}\"", arg_default))
                }

                argument_ids.push_str("]");
                argument_names.push_str("]");
                argument_defaults.push_str("]");

                definition_inputs.insert(
                    "custom_block".to_string(),
                    json!([1, prototype_id.to_string()]),
                );

                let proc_definition = Block {
                    opcode: "procedures_definition".to_string(),
                    next: Some((self.block_id + 1).to_string()),
                    inputs: Some(definition_inputs),
                    top_level: Some(true),
                    ..Default::default()
                };

                let proc_prototype = Block {
                    opcode: "procedures_prototype".to_string(),
                    parent: Some(definition_id.to_string()),
                    inputs: Some(prototype_inputs),
                    shadow: Some(true),
                    top_level: Some(false),
                    mutation: Some(Mutation {
                        tag_name: "mutation".to_string(),
                        children: vec![],
                        proccode: proc_code,
                        argumentids: argument_ids,
                        argumentnames: Some(argument_names),
                        argumentdefaults: Some(argument_defaults),
                        warp: "false".to_string(),
                    }),
                    ..Default::default()
                };

                self.push_block(&proc_definition, definition_id);
                self.push_block(&proc_prototype, prototype_id);

                for (id, block) in arg_blocks {
                    self.push_block(&block, id);
                }

                for statement in body {
                    let current_id = self.gen_block_id();
                    self.compile_statement(statement, Some(definition_id), Some(current_id), None);
                }

                // self.push_block(
                //     &Block {
                //         opcode: "procedures_definition".to_string(),
                //     },
                //     current_id,
                // )
            }
            _ => panic!(),
        }
    }

    fn compile_body(&mut self, body: &Vec<Stmt>, current_id: usize, parent_id: usize) {
        // for
    }

    fn push_block(&mut self, block: &Block, id: usize) {
        self.project.targets[self.target_index]
            .blocks
            .insert(id.to_string(), block.clone());
    }

    fn gen_block_id(&mut self) -> usize {
        self.block_id += 1;
        self.block_id
    }

    fn gen_arg_id(&mut self) -> usize {
        self.arg_id += 1;
        self.arg_id
    }
}
