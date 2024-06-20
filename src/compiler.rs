use std::collections::HashMap;
use std::io::Write;

use md5::{Digest, Md5};
use serde_json::{json, Value};

use crate::makefile::TargetData;
use crate::parser::{Event, Expr, Stmt};
use crate::project::{Block, Costume, Mutation, Project, Target};
use crate::token::Type;

#[derive(Clone, Hash)]
enum Scope {
    Local,
    Function(String),
}

// kanye west
impl PartialEq for Scope {
    fn eq(&self, other: &Scope) -> bool {
        match (other, self) {
            (Scope::Local, Scope::Local) => true,
            (Scope::Local, Scope::Function(_)) => false,
            (Scope::Function(_), Scope::Local) => false,
            (Scope::Function(func), Scope::Function(other_func)) => func == other_func,
        }
    }
}
impl Eq for Scope {}

pub struct Compiler {
    project: Project,
    targets: Vec<(TargetData, Vec<Stmt>)>,
    current_target: (TargetData, Vec<Stmt>),
    scope: Scope,
    block_id: usize,
    /// ```
    /// HashMap<Scope, HashMap<VarName, (VarId, VarType)>>
    var_table: HashMap<Scope, HashMap<String, (String, Type)>>,
    var_id: usize,
    /// ```
    /// let function_table = arg_table.get(function_name)?;
    /// let (arg_id, arg_name) = function_table[arg_position]?;
    /// HashMap<FunctionName, Vec<(ArgId, ArgName)>>
    arg_table: HashMap<String, Vec<(String, String)>>,
    arg_id: usize,
    target_index: usize,
    parent: Option<usize>,
}

// TODO: fix excessive cloning
impl Compiler {
    pub fn new(targets: Vec<(TargetData, Vec<Stmt>)>) -> Compiler {
        Compiler {
            scope: Scope::Local,
            targets: targets.clone(),
            current_target: targets[0].clone(),
            project: Project::new(),
            block_id: 0,
            arg_id: 0,
            arg_table: HashMap::new(),
            var_id: 0,
            target_index: 0,
            parent: None,
            var_table: HashMap::new(),
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
                self.compile_top_level_statement(statement);
            }

            self.target_index += 1;
        }

        //
        &self.project
    }

    // TODO: actual type checking
    fn assert_type(&self, expected_types: Vec<Type>, actual_type: Type) {
        if !expected_types.contains(&actual_type) {
            panic!(
                "expected type: {:?}, recieved type: {:?}",
                expected_types, actual_type
            );
        }
    }

    fn compile_binary_expr(&mut self, expression: &Expr, parent_id: String, current_id: String) {
        match expression {
            Expr::Number(_) => todo!(),
            Expr::String(_) => todo!(),
            Expr::Identifier(_) => todo!(),
            Expr::Bool(_) => todo!(),
            Expr::Binary(left, op, right) => match op {
                // string concat lol
                crate::token::Operator::Ampersand => {
                    let string_1 = match left.as_ref() {
                        Expr::String(value) => json!([1, [10, value.to_string()]]),
                        Expr::Number(value) => json!([1, [10, value.to_string()]]),
                        Expr::Bool(value) => json!([1, [10, value.to_string()]]),
                        Expr::Identifier(ident) => {
                            if self.var_exists(self.scope.clone(), ident.clone()) {
                                json!([
                                    3,
                                    [
                                        12,
                                        ident.clone(),
                                        self.get_var_id(self.scope.clone(), ident.clone())
                                    ],
                                    [10, ""]
                                ])
                            } else {
                                let child_id = self.gen_block_id();
                                self.push_block(
                                    &Block {
                                        opcode: "argument_reporter_string_number".to_string(),
                                        parent: Some(current_id.to_string()),
                                        fields: Some(
                                            json!({"VALUE": [ident, serde_json::Value::Null]}),
                                        ),
                                        shadow: Some(false),
                                        top_level: Some(false),
                                        ..Default::default()
                                    },
                                    child_id.clone(),
                                );

                                json!([3, child_id.to_string(), [10, ""]])
                            }
                        }
                        Expr::Binary(_, _, _) => {
                            // TODO: type checking here, some operators can't be used as input for other operators
                            let id = self.gen_block_id();
                            self.compile_binary_expr(left, current_id.clone(), id.clone());
                            json!([3, id.to_string(), [10, ""]])
                        }
                    };

                    let string_2 = match right.as_ref() {
                        Expr::String(value) => json!([1, [10, value.to_string()]]),
                        Expr::Number(value) => json!([1, [10, value.to_string()]]),
                        Expr::Bool(value) => json!([1, [10, value.to_string()]]),
                        Expr::Identifier(ident) => {
                            if self.var_exists(self.scope.clone(), ident.clone()) {
                                json!([
                                    3,
                                    [
                                        12,
                                        ident.clone(),
                                        self.get_var_id(self.scope.clone(), ident.clone())
                                    ],
                                    [10, ""]
                                ])
                            } else {
                                let child_id = self.gen_block_id();
                                self.push_block(
                                    &Block {
                                        opcode: "argument_reporter_string_number".to_string(),
                                        parent: Some(current_id.to_string()),
                                        fields: Some(
                                            json!({"VALUE": [ident, serde_json::Value::Null]}),
                                        ),
                                        shadow: Some(false),
                                        top_level: Some(false),
                                        ..Default::default()
                                    },
                                    child_id.clone(),
                                );

                                json!([3, child_id.to_string(), [10, ""]])
                            }
                        }
                        Expr::Binary(_, _, _) => {
                            // TODO: type checking here, some operators can't be used as input for other operators
                            let id = self.gen_block_id();
                            self.compile_binary_expr(right, current_id.clone(), id.clone());
                            json!([3, id.to_string(), [10, ""]])
                        }
                    };

                    self.push_block(
                        &Block {
                            opcode: "operator_join".to_string(),
                            parent: Some(parent_id.to_string()),
                            inputs: Some(HashMap::from([
                                ("STRING1".to_string(), string_1),
                                ("STRING2".to_string(), string_2),
                            ])),
                            shadow: Some(false),
                            top_level: Some(false),
                            ..Default::default()
                        },
                        current_id,
                    )
                }
                crate::token::Operator::Bang => todo!(),
                crate::token::Operator::EqualEqual => todo!(),
                crate::token::Operator::BangEqual => todo!(),
                crate::token::Operator::Greater => todo!(),
                crate::token::Operator::Less => todo!(),
                crate::token::Operator::GreaterEqual => todo!(),
                crate::token::Operator::LessEqual => todo!(),
                crate::token::Operator::Minus => todo!(),
                crate::token::Operator::Plus => todo!(),
                crate::token::Operator::Slash => todo!(),
                crate::token::Operator::Star => todo!(),
                crate::token::Operator::Caret => todo!(),
                crate::token::Operator::None => panic!("we should never be here."),
            },
        }
    }

    fn compile_body_statements(&mut self, body: &Vec<Stmt>, parent_id: String) {
        for (index, stmt) in body.into_iter().enumerate() {
            let current_id = self.gen_block_id();

            match stmt {
                Stmt::FunctionCall(func_name, args) => {
                    let opcode = match func_name.as_str() {
                        "say" => "looks_say",
                        _ => "procedures_call",
                    };

                    match opcode {
                        // FIXME: we only care about the first expression, lel
                        "looks_say" => match &args[0] {
                            Expr::Number(_) => todo!(),
                            Expr::String(string) => {
                                let mut inputs = HashMap::new();
                                let value = json!([1, [10, string,]]);
                                inputs.insert("MESSAGE".to_string(), value);

                                let next_id = if (index + 1) >= body.len() {
                                    None
                                } else {
                                    Some(self.peek_next_block_id())
                                };

                                self.push_block(
                                    &Block {
                                        opcode: "looks_say".to_string(),
                                        parent: Some(parent_id.clone()),
                                        inputs: Some(inputs),
                                        next: next_id,
                                        ..Block::default()
                                    },
                                    current_id,
                                );
                            }
                            Expr::Identifier(ident) => {
                                let arg_reporter_id = self.gen_block_id();
                                let looks_say_id = current_id;

                                let mut inputs = HashMap::new();
                                let value = json!([3, arg_reporter_id, [10, ""]]);
                                inputs.insert("MESSAGE".to_string(), value);

                                let next_id = if (index + 1) >= body.len() {
                                    None
                                } else {
                                    Some(self.peek_next_block_id())
                                };

                                self.push_block(
                                    &Block {
                                        opcode: "looks_say".to_string(),
                                        parent: Some(parent_id.clone()),
                                        inputs: Some(inputs),
                                        next: next_id,
                                        ..Block::default()
                                    },
                                    looks_say_id.clone(),
                                );

                                self.push_block(
                                    &Block {
                                        opcode: "argument_reporter_string_number".to_string(),
                                        parent: Some(looks_say_id),
                                        fields: Some(json!({"VALUE": [ident, Value::Null]})),
                                        shadow: Some(false),
                                        top_level: Some(false),
                                        ..Block::default()
                                    },
                                    arg_reporter_id,
                                );
                            }
                            Expr::Bool(_) => todo!(),
                            Expr::Binary(_, _, _) => {
                                let new_id = self.gen_block_id();

                                self.compile_binary_expr(
                                    &args[0],
                                    current_id.clone(),
                                    new_id.clone(),
                                );

                                let mut inputs = HashMap::new();
                                inputs.insert("MESSAGE".to_string(), json!([3, new_id, [10, ""]]));

                                self.push_block(
                                    &Block {
                                        opcode: "looks_say".to_string(),
                                        parent: Some(parent_id.to_string()),
                                        inputs: Some(inputs),
                                        next: Some((self.block_id + 1).to_string()),
                                        ..Block::default()
                                    },
                                    current_id,
                                );
                            }
                            _ => panic!(),
                        },
                        _ => {
                            let mut inputs: HashMap<String, Value> = HashMap::new();
                            let mut proc_codes = func_name.clone();
                            let mut argument_ids = String::from("[");

                            let mut expr_block_id = None;

                            for (index, arg) in args.into_iter().enumerate() {
                                let proc_code = match arg {
                                    Expr::Number(value) => " %s",
                                    Expr::String(value) => " %s",
                                    Expr::Identifier(_) => " %s",
                                    // FIXME:: accept more than just string type
                                    Expr::Bool(_) => todo!(),
                                    Expr::Binary(_, _, _) => {
                                        expr_block_id = Some(self.gen_block_id());
                                        " %s"
                                    }
                                };

                                proc_codes.push_str(proc_code);

                                if index != 0 {
                                    argument_ids.push_str(", ");
                                }

                                // TODO: just add error checking instead of crashing please
                                let arg_id = &self.arg_table.get(func_name).unwrap()[index].0;
                                argument_ids.push_str(&format!("\"{}\"", arg_id.to_string()));

                                match arg {
                                    Expr::Number(value) => {
                                        inputs.insert(arg_id.to_string(), json!([1, [10, value]]));
                                    }
                                    Expr::String(value) => {
                                        inputs.insert(arg_id.to_string(), json!([1, [10, value]]));
                                    }
                                    Expr::Identifier(ident) => {
                                        inputs.insert(
                                            arg_id.to_string(),
                                            json!([
                                                3,
                                                [
                                                    12,
                                                    ident,
                                                    self.get_var_id(
                                                        self.scope.clone(),
                                                        ident.clone()
                                                    )
                                                ]
                                            ]),
                                        );
                                    }
                                    Expr::Bool(_) => todo!(),
                                    Expr::Binary(_, _, _) => {
                                        let expr_block_id = expr_block_id.clone();

                                        inputs.insert(
                                            arg_id.clone().to_string(),
                                            json!([3, expr_block_id.clone().unwrap(), [10, ""]]),
                                        );

                                        self.compile_binary_expr(
                                            arg,
                                            current_id.clone(),
                                            expr_block_id.unwrap(),
                                        );
                                    }
                                }
                            }

                            argument_ids.push_str("]");

                            let next_id = if (index + 1) >= body.len() {
                                None
                            } else {
                                Some(self.peek_next_block_id())
                            };

                            self.push_block(
                                &Block {
                                    opcode: "procedures_call".to_string(),
                                    parent: Some(parent_id.clone()),
                                    inputs: Some(inputs),
                                    mutation: Some(Mutation {
                                        tag_name: "mutation".to_string(),
                                        children: vec![],
                                        proccode: proc_codes,
                                        argumentids: argument_ids,
                                        argumentnames: None,
                                        argumentdefaults: None,
                                        warp: "false".to_string(),
                                    }),
                                    next: next_id,
                                    ..Block::default()
                                },
                                current_id,
                            );
                        }
                    }

                    // self.push_block(block, id)
                }
                _ => panic!("statment: {:#?} not valid in body", stmt),
            }
        }
    }

    fn compile_top_level_statement(&mut self, statement: &Stmt) {
        match statement {
            // TODO: scope for event handler
            Stmt::EventHandler(event, body) => match event {
                Event::FlagClicked => {
                    let flag_id = self.gen_block_id();

                    let next_id = if body.len() > 0 {
                        Some(self.peek_next_block_id())
                    } else {
                        None
                    };

                    self.push_block(
                        &Block {
                            opcode: "event_whenflagclicked".to_string(),
                            next: next_id,
                            top_level: Some(true),
                            ..Block::default()
                        },
                        flag_id.clone(),
                    );

                    self.compile_body_statements(body, flag_id);
                }
                _ => todo!(),
            },
            Stmt::FunctionDeclaration(func_name, args, body, return_type) => {
                self.arg_table.insert(func_name.clone(), vec![]);
                self.scope = Scope::Function(func_name.clone());

                let prototype_id = self.gen_block_id(); // a
                let definition_id = self.gen_block_id(); // b

                let mut prototype_inputs: HashMap<String, Value> = HashMap::new();
                let mut definition_inputs: HashMap<String, Value> = HashMap::new();

                let mut arg_blocks: Vec<(String, Block)> = Vec::new();

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
                    argument_defaults.push_str(&format!("\"{}\"", arg_default));

                    // TODO: error handling please???
                    let function_table = self.arg_table.get_mut(func_name).unwrap();
                    function_table.push((arg_id, arg_name.to_string()));
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
                    next: Some(self.peek_next_block_id()),
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

                self.push_block(&proc_definition, definition_id.clone());
                self.push_block(&proc_prototype, prototype_id);

                for (id, block) in arg_blocks {
                    self.push_block(&block, id);
                }

                self.compile_body_statements(body, definition_id.clone());

                self.scope = Scope::Local;
            }
            _ => panic!("statement type: {:#?} cannot be top-level", statement),
        }
    }

    fn compile_statement(
        &mut self,
        statement: &Stmt,
        parent_id: Option<String>,
        current_id: Option<String>,
        next_id: Option<String>,
    ) {
        match statement {
            Stmt::EventHandler(event, body) => match event {
                Event::FlagClicked => {
                    let event_id = self.gen_block_id();

                    self.push_block(
                        &Block {
                            opcode: "event_whenflagclicked".to_string(),
                            next: Some(self.peek_next_block_id()),
                            top_level: Some(true),
                            ..Block::default()
                        },
                        event_id.clone(),
                    );

                    for (idx, stmt) in body.iter().enumerate() {
                        let stmt_id = self.gen_block_id();

                        let next_stmt_id = if (idx + 1) == body.len() {
                            None
                        } else {
                            Some(self.peek_next_block_id())
                        };

                        self.compile_statement(
                            stmt,
                            Some(event_id.clone()),
                            Some(stmt_id),
                            next_stmt_id,
                        );
                    }
                }
                Event::KeyPressed(_) => todo!(),
            },
            Stmt::FunctionCall(func_name, args) => {
                let opcode = match func_name.as_str() {
                    "say" => "looks_say",
                    _ => "procedures_call",
                };

                match opcode {
                    // FIXME: we only care about the first expression, lel
                    "looks_say" => match &args[0] {
                        Expr::Number(_) => todo!(),
                        Expr::String(string) => {
                            let mut inputs = HashMap::new();
                            let value = json!([1, [10, string,]]);

                            inputs.insert("MESSAGE".to_string(), value);

                            self.push_block(
                                &Block {
                                    opcode: "looks_say".to_string(),
                                    parent: Some(parent_id.unwrap().to_string()),
                                    inputs: Some(inputs),
                                    next: Some(self.peek_next_block_id()),
                                    ..Block::default()
                                },
                                current_id.unwrap(),
                            );
                        }
                        Expr::Identifier(ident) => {
                            let arg_reporter_id = self.gen_block_id();

                            let mut inputs = HashMap::new();
                            let value = json!([3, arg_reporter_id.to_string(), [10, ""]]);

                            inputs.insert("MESSAGE".to_string(), value);

                            self.push_block(
                                &Block {
                                    opcode: "looks_say".to_string(),
                                    parent: Some(parent_id.unwrap().to_string()),
                                    inputs: Some(inputs),
                                    next: next_id.map(|id| id.to_string()),
                                    ..Block::default()
                                },
                                current_id.clone().unwrap(),
                            );

                            self.push_block(
                                &Block {
                                    opcode: "argument_reporter_string_number".to_string(),
                                    parent: current_id.map(|id| id.to_string()),
                                    fields: Some(json!({"VALUE": [ident, Value::Null]})),
                                    shadow: Some(false),
                                    top_level: Some(false),
                                    ..Block::default()
                                },
                                arg_reporter_id,
                            );
                        }
                        Expr::Bool(_) => todo!(),
                        Expr::Binary(_, _, _) => {
                            let new_id = self.gen_block_id();

                            self.compile_binary_expr(
                                &args[0],
                                current_id.clone().unwrap(),
                                new_id.clone(),
                            );

                            let mut inputs = HashMap::new();
                            inputs.insert("MESSAGE".to_string(), json!([3, new_id, [10, ""]]));

                            self.push_block(
                                &Block {
                                    opcode: "looks_say".to_string(),
                                    parent: Some(parent_id.unwrap().to_string()),
                                    inputs: Some(inputs),
                                    next: Some((self.block_id + 1).to_string()),
                                    ..Block::default()
                                },
                                current_id.unwrap(),
                            );
                        }
                    },
                    _ => {
                        let mut inputs: HashMap<String, Value> = HashMap::new();
                        let mut proc_codes = func_name.clone();
                        let mut argument_ids = String::from("[");

                        for (index, arg) in args.into_iter().enumerate() {
                            let proc_code = match arg {
                                Expr::Number(value) => " %s",
                                Expr::String(value) => " %s",
                                Expr::Identifier(_) => " %s",
                                // FIXME:: accept more than just string type
                                Expr::Bool(_) => todo!(),
                                Expr::Binary(_, _, _) => todo!(),
                            };

                            proc_codes.push_str(proc_code);

                            if index != 0 {
                                argument_ids.push_str(", ");
                            }

                            // TODO: just add error checking instead of crashing please
                            let arg_id = &self.arg_table.get(func_name).unwrap()[index].0;
                            argument_ids.push_str(&format!("\"{}\"", arg_id.to_string()));

                            match arg {
                                Expr::Number(value) => {
                                    inputs.insert(arg_id.to_string(), json!([1, [10, value]]));
                                }
                                Expr::String(value) => {
                                    inputs.insert(arg_id.to_string(), json!([1, [10, value]]));
                                }
                                Expr::Identifier(ident) => {
                                    inputs.insert(
                                        arg_id.to_string(),
                                        json!([
                                            3,
                                            [
                                                12,
                                                ident,
                                                self.get_var_id(self.scope.clone(), ident.clone())
                                            ]
                                        ]),
                                    );
                                }
                                Expr::Bool(_) => todo!(),
                                Expr::Binary(_, _, _) => todo!(),
                            }
                        }

                        argument_ids.push_str("]");

                        self.push_block(
                            &Block {
                                opcode: "procedures_call".to_string(),
                                parent: Some(parent_id.unwrap().to_string()),
                                inputs: Some(inputs),
                                mutation: Some(Mutation {
                                    tag_name: "mutation".to_string(),
                                    children: vec![],
                                    proccode: proc_codes,
                                    argumentids: argument_ids,
                                    argumentnames: None,
                                    argumentdefaults: None,
                                    warp: "false".to_string(),
                                }),
                                next: Some(self.peek_next_block_id()),
                                ..Block::default()
                            },
                            current_id.unwrap(),
                        );
                    }
                }

                // self.push_block(block, id)
            }
            Stmt::FunctionDeclaration(func_name, args, body, return_type) => {
                self.arg_table.insert(func_name.clone(), vec![]);
                self.scope = Scope::Function(func_name.clone());

                let prototype_id = self.gen_block_id(); // a
                let definition_id = self.gen_block_id(); // b

                let mut prototype_inputs: HashMap<String, Value> = HashMap::new();
                let mut definition_inputs: HashMap<String, Value> = HashMap::new();

                let mut arg_blocks: Vec<(String, Block)> = Vec::new();

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
                    argument_defaults.push_str(&format!("\"{}\"", arg_default));

                    // TODO: error handling please???
                    let function_table = self.arg_table.get_mut(func_name).unwrap();
                    function_table.push((arg_id, arg_name.to_string()));
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

                self.push_block(&proc_definition, definition_id.clone());
                self.push_block(&proc_prototype, prototype_id);

                for (id, block) in arg_blocks {
                    self.push_block(&block, id);
                }

                for statement in body {
                    let current_id = self.gen_block_id();
                    self.compile_statement(
                        statement,
                        Some(definition_id.clone()),
                        Some(current_id),
                        None,
                    );
                }

                self.scope = Scope::Local;
            }
            // TODO: actually handle type checking?
            Stmt::VariableDeclaration(var_name, var_type, expr) => {
                // TODO: handle expressions, idents
                let var_id = self.push_var(self.scope.clone(), var_name.clone(), var_type.clone());

                let value = match expr {
                    Expr::Number(value) => json!([1, [10, value.to_string()]]),
                    Expr::String(value) => json!([1, [10, value.to_string()]]),
                    Expr::Identifier(ident) => {
                        let ident_id = self.get_var_id(self.scope.clone(), ident.clone());
                        json!([1, [12, ident, ident_id]])
                    }
                    Expr::Bool(_) => todo!(),
                    Expr::Binary(_, _, _) => todo!(),
                };

                let mut inputs = HashMap::new();
                inputs.insert("VALUE".to_string(), value);

                self.push_block(
                    &Block {
                        opcode: "data_setvariableto".to_string(),
                        parent: Some(parent_id.unwrap().to_string()),
                        inputs: Some(inputs),
                        fields: Some(json!({"VARIABLE": [var_name, var_id]})),
                        next: next_id.map(|id| id.to_string()),
                        ..Block::default()
                    },
                    current_id.unwrap().to_string(),
                );
            }
            _ => panic!(),
        }
    }

    fn push_block(&mut self, block: &Block, id: String) {
        self.project.targets[self.target_index]
            .blocks
            .insert(id, block.clone());
    }

    /// adds a variable to both the internal table used by the compiler,
    /// and to the Project struct that's eventually serialized
    /// returns the ID of the variable
    fn push_var(&mut self, scope: Scope, var_name: String, var_type: Type) -> String {
        // add an empty hashmap for the scope if it doesn't exist within the var table
        if !self
            .var_table
            .clone()
            .into_keys()
            .collect::<Vec<Scope>>()
            .contains(&scope)
        {
            self.var_table.insert(scope.clone(), HashMap::new());
        }

        let var_id = self.gen_var_id().to_string();

        self.var_table
            .get_mut(&scope)
            .unwrap()
            .insert(var_name.clone(), (var_id.clone(), var_type.clone()));

        self.project.targets[self.target_index]
            .variables
            .insert(var_id.clone(), json!([var_name, 0]));

        var_id
    }

    fn get_var_id(&self, scope: Scope, var_name: String) -> String {
        self.var_table
            .get(&scope)
            .unwrap()
            .get(&var_name)
            .unwrap()
            .0
            .to_string()
    }

    fn var_exists(&self, scope: Scope, var_name: String) -> bool {
        let scoped_table = self.var_table.get(&scope);
        let scoped_table = if let Some(table) = scoped_table {
            table
        } else {
            return false;
        };

        match scoped_table.get(&var_name) {
            Some(_) => true,
            None => false,
        }
    }

    fn gen_block_id(&mut self) -> String {
        self.block_id += 1;
        ('a'..='z').into_iter().collect::<Vec<char>>()[self.block_id - 1].to_string()
    }

    fn peek_next_block_id(&self) -> String {
        ('a'..='z').into_iter().collect::<Vec<char>>()[self.block_id].to_string()
    }

    fn gen_arg_id(&mut self) -> String {
        self.arg_id += 1;
        format!("arg_{}", self.arg_id)
    }

    fn gen_var_id(&mut self) -> String {
        self.var_id += 1;
        format!("var_{}", self.var_id)
    }
}
