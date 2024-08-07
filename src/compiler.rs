use std::collections::HashMap;
use std::default;

use md5::{Digest, Md5};
use serde_json::{json, Value};

use crate::makefile::TargetData;
use crate::parser::{Event, Expr, Stmt};
use crate::project::{Block, Costume, Mutation, Project, Target};
use crate::token::{Operator, Type};

pub struct Compiler {
    project: Project,
    targets: Vec<(TargetData, Vec<Stmt>)>,
    current_target: (TargetData, Vec<Stmt>),
    next_block_id: Option<String>,
    block_id: usize,
    /// maps variables to their scope
    /// ```
    /// HashMap<ScopePath, HashMap<VarName, (VarId, VarType)>>
    var_table: HashMap<Vec<String>, HashMap<String, (String, Type)>>,
    var_id: usize,
    /// ```
    /// let function_table = arg_table.get(function_name)?;
    /// let (arg_id, arg_name) = function_table[arg_position]?;
    /// HashMap<FunctionName, Vec<(ArgId, ArgName)>>
    arg_table: HashMap<String, Vec<(String, String)>>,
    arg_id: usize,
    target_index: usize,
    parent: Option<usize>,
    /// Vec<block_id>
    scope_path: Vec<String>,
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
            var_id: 0,
            target_index: 0,
            parent: None,
            scope_path: Vec::new(),
            var_table: HashMap::new(),
            next_block_id: None,
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

    fn compile_condition(&mut self, condition: &Expr, parent_id: String, current_id: String) {
        // never nest!
        let (left, op, right) = if let Expr::Binary(left, op, right) = condition {
            (left, op, right)
        } else {
            panic!(
                "expected binary expression in condition, recieved: {:?}",
                condition
            );
        };

        match op {
            Operator::Bang => {
                todo!("this will be removed, i just can't be fucked to bother with the lexer")
            }
            Operator::EqualEqual => {
                self.compile_binary_expr(condition, parent_id, current_id);
            },
            Operator::And => {
                self.compile_binary_expr(condition, parent_id, current_id);
            }
            Operator::Or => {
                self.compile_binary_expr(condition, parent_id, current_id);
            }
            Operator::BangEqual => {
                self.compile_binary_expr(condition, parent_id, current_id);
            },
            Operator::Greater => todo!(),
            Operator::Less => todo!(),
            Operator::GreaterEqual => todo!(),
            Operator::LessEqual => todo!(),
            _ => panic!("a comparison operator is required as the root operator in a condition, found: {:#?}", op)
        }
    }

    fn compile_binary_expr(&mut self, expression: &Expr, parent_id: String, current_id: String) {
        if parent_id == "a" {
            panic!()
        }

        match expression {
            Expr::Binary(left, op, right) => match op {
                // string concat lol
                Operator::Ampersand => {
                    self.compile_simple_operator(
                        "STRING1",
                        "STRING2",
                        left,
                        right,
                        "operator_join",
                        current_id,
                        parent_id,
                    );
                }
                Operator::Bang => todo!(),
                Operator::EqualEqual => {
                    let inputs = HashMap::from([
                        (
                            "OPERAND1".to_string(),
                            self.value_from_expr(left, current_id.clone(), None),
                        ),
                        (
                            "OPERAND2".to_string(),
                            self.value_from_expr(right, current_id.clone(), None),
                        ),
                    ]);

                    self.push_block(
                        &Block {
                            opcode: "operator_equals".to_string(),
                            parent: Some(parent_id),
                            inputs: Some(inputs),
                            ..Default::default()
                        },
                        current_id,
                    );
                }
                Operator::BangEqual => {
                    let equals_id = self.gen_block_id();

                    self.push_block(
                        &Block {
                            opcode: "operator_not".to_string(),
                            parent: Some(parent_id.clone()),
                            inputs: Some(HashMap::from([(
                                "OPERAND".to_string(),
                                json!([2, equals_id]),
                            )])),
                            shadow: Some(false),
                            top_level: Some(false),
                            ..Default::default()
                        },
                        current_id.clone(),
                    );

                    // FIXME: expensive cloning(?)
                    let expression =
                        Expr::Binary(left.clone(), Operator::EqualEqual, right.clone());
                    self.compile_binary_expr(&expression, current_id, equals_id);
                }
                Operator::Greater => todo!(),
                Operator::Less => todo!(),
                Operator::GreaterEqual => todo!(),
                Operator::LessEqual => todo!(),
                Operator::Minus => {
                    self.compile_simple_operator(
                        "NUM1",
                        "NUM2",
                        left,
                        right,
                        "operator_subtract",
                        current_id,
                        parent_id,
                    );
                }
                Operator::Plus => {
                    self.compile_simple_operator(
                        "NUM1",
                        "NUM2",
                        left,
                        right,
                        "operator_add",
                        current_id,
                        parent_id,
                    );
                }
                Operator::Slash => {
                    self.compile_simple_operator(
                        "NUM1",
                        "NUM2",
                        left,
                        right,
                        "operator_divide",
                        current_id,
                        parent_id,
                    );
                }
                Operator::Star => {
                    self.compile_simple_operator(
                        "NUM1",
                        "NUM2",
                        left,
                        right,
                        "operator_multiply",
                        current_id,
                        parent_id,
                    );
                }
                Operator::Caret => todo!(),
                Operator::None => panic!("we should never be here."),
                Operator::And => {
                    let inputs = HashMap::from([
                        (
                            "OPERAND1".to_string(),
                            self.value_from_expr(left, current_id.clone(), None),
                        ),
                        (
                            "OPERAND2".to_string(),
                            self.value_from_expr(right, current_id.clone(), None),
                        ),
                    ]);

                    self.push_block(
                        &Block {
                            opcode: "operator_and".to_string(),
                            parent: Some(parent_id),
                            inputs: Some(inputs),
                            ..Default::default()
                        },
                        current_id,
                    );
                }
                Operator::Or => {
                    let inputs = HashMap::from([
                        (
                            "OPERAND1".to_string(),
                            self.value_from_expr(left, current_id.clone(), None),
                        ),
                        (
                            "OPERAND2".to_string(),
                            self.value_from_expr(right, current_id.clone(), None),
                        ),
                    ]);

                    self.push_block(
                        &Block {
                            opcode: "operator_or".to_string(),
                            parent: Some(parent_id),
                            inputs: Some(inputs),
                            ..Default::default()
                        },
                        current_id,
                    );
                }
                Operator::PlusEqual => todo!(),
                Operator::MinusEqual => todo!(),
                Operator::StarEqual => todo!(),
                Operator::SlashEqual => todo!(),
            },
            _ => todo!(),
        }
    }

    fn compile_simple_operator(
        &mut self,
        key1: &str,
        key2: &str,
        val1: &Expr,
        val2: &Expr,
        opcode: &str,
        current_id: String,
        parent_id: String,
    ) {
        let val1 = self.value_from_expr(val1, current_id.clone(), None);
        let val2 = self.value_from_expr(val2, current_id.clone(), None);

        self.push_block(
            &Block {
                opcode: opcode.to_string(),
                parent: Some(parent_id.to_string()),
                inputs: Some(HashMap::from([
                    (key1.to_string(), val1),
                    (key2.to_string(), val2),
                ])),
                shadow: Some(false),
                top_level: Some(false),
                ..Default::default()
            },
            current_id,
        )
    }

    fn compile_conditional_control(
        &mut self,
        opcode: &str,
        cond: &Expr,
        substacks: (Option<&Vec<Stmt>>, Option<&Vec<Stmt>>),
        current_id: String,
        parent_id: String,
        index: usize,
        body_len: usize,
    ) {
        let condition_id = self.gen_block_id();

        self.compile_condition(cond, current_id.clone(), condition_id.clone());

        let mut inputs = HashMap::new();
        inputs.insert("CONDITION".to_string(), json!([2, condition_id]));

        // TODO: remove this code-duplication
        if let Some(substack) = substacks.0 {
            let substack_id = self.peek_next_block_id();
            self.compile_body_statements(substack, current_id.clone(), None);
            inputs.insert("SUBSTACK".to_string(), json!([2, substack_id]));
        }

        if let Some(substack) = substacks.1 {
            let substack_id = self.peek_next_block_id();
            self.compile_body_statements(substack, current_id.clone(), None);
            inputs.insert("SUBSTACK2".to_string(), json!([2, substack_id]));
        }

        self.next_block_id = if (index + 1) >= body_len {
            None
        } else {
            Some(self.peek_next_block_id())
        };

        self.push_block(
            &Block {
                opcode: opcode.to_string(),
                next: self.next_block_id.clone(),
                parent: Some(parent_id.clone()),
                inputs: Some(inputs),
                shadow: Some(false),
                top_level: Some(false),
                ..Default::default()
            },
            current_id,
        );
    }

    fn value_from_expr(
        &mut self,
        expr: &Expr,
        parent_id: String,
        current_id: Option<String>,
    ) -> Value {
        match expr {
            Expr::String(value) => json!([1, [10, value.to_string()]]),
            Expr::Number(value) => json!([1, [10, value.to_string()]]),
            Expr::Bool(value) => json!([1, [10, value.to_string()]]),
            Expr::Identifier(ident) => {
                if self.var_exists(self.scope_path.clone(), ident.clone()) {
                    json!([
                        3,
                        [
                            12,
                            ident.clone(),
                            self.get_var_id(self.scope_path.clone(), ident.clone())
                        ],
                        [10, ""]
                    ])
                } else {
                    let child_id = self.gen_block_id();
                    self.push_block(
                        &Block {
                            opcode: "argument_reporter_string_number".to_string(),
                            parent: Some(parent_id.to_string()),
                            fields: Some(json!({"VALUE": [ident, serde_json::Value::Null]})),
                            shadow: Some(false),
                            top_level: Some(false),
                            ..Default::default()
                        },
                        child_id.clone(),
                    );

                    json!([3, child_id.to_string(), [10, ""]])
                }
            }
            Expr::Binary(_, op, _) => {
                // TODO: type checking here, some operators can't be used as input for other operators
                let id = self.gen_block_id();
                self.compile_binary_expr(expr, current_id.unwrap(), id.clone());
                // TODO: the "3" here shouldn't be static, see comments above
                // Project::Inputs
                //
                // let input_shadow_num = match op {
                //     Operator::EqualEqual => 1,
                //     _ => 1,
                // };

                json!([3, id.to_string(), [10, ""]])
            }
            Expr::FunctionCall(func_name, args) => {
                self.compile_function_call(
                    func_name.clone(),
                    args.clone(),
                    current_id.unwrap(),
                    parent_id,
                    1,
                    3,
                );

                let return_var_name = format!("!func_var_{}", func_name);

                json!([
                    3,
                    [
                        12,
                        return_var_name,
                        self.get_var_id(self.scope_path.clone(), return_var_name.clone())
                    ],
                    [10, ""]
                ])
            }
        }
    }

    fn compile_function_call(
        &mut self,
        func_name: String,
        args: Vec<Expr>,
        current_id: String,
        parent_id: String,
        index: usize,
        body_len: usize,
    ) {
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

                    self.next_block_id = if (index + 1) >= body_len {
                        None
                    } else {
                        Some(self.peek_next_block_id())
                    };

                    self.push_block(
                        &Block {
                            opcode: "looks_say".to_string(),
                            parent: Some(parent_id.clone()),
                            inputs: Some(inputs),
                            next: self.next_block_id.clone(),
                            ..Block::default()
                        },
                        current_id,
                    );
                }
                Expr::Identifier(ident) => {
                    let looks_say_id = current_id.clone();

                    let value = if self.var_exists(self.scope_path.clone(), ident.clone()) {
                        json!([
                            3,
                            [
                                12,
                                ident.clone(),
                                self.get_var_id(self.scope_path.clone(), ident.clone())
                            ],
                            [10, ""]
                        ])
                    } else {
                        let arg_reporter_id = self.gen_block_id();

                        self.push_block(
                            &Block {
                                opcode: "argument_reporter_string_number".to_string(),
                                parent: Some(looks_say_id.clone()),
                                fields: Some(json!({"VALUE": [ident, Value::Null]})),
                                shadow: Some(false),
                                top_level: Some(false),
                                ..Block::default()
                            },
                            arg_reporter_id.clone(),
                        );

                        json!([3, arg_reporter_id, [10, ""]])
                    };

                    let mut inputs = HashMap::new();
                    inputs.insert("MESSAGE".to_string(), value);

                    self.next_block_id = if (index + 1) >= body_len {
                        None
                    } else {
                        Some(self.peek_next_block_id())
                    };

                    self.push_block(
                        &Block {
                            opcode: "looks_say".to_string(),
                            parent: Some(parent_id.clone()),
                            inputs: Some(inputs),
                            next: self.next_block_id.clone(),
                            ..Block::default()
                        },
                        looks_say_id.clone(),
                    );
                }
                Expr::Bool(_) => todo!(),
                Expr::Binary(_, _, _) => {
                    let new_id = self.gen_block_id();

                    self.compile_binary_expr(&args[0], current_id.clone(), new_id.clone());

                    let mut inputs = HashMap::new();
                    inputs.insert("MESSAGE".to_string(), json!([3, new_id, [10, ""]]));

                    self.next_block_id = if (index + 1) >= body_len {
                        None
                    } else {
                        Some(self.peek_next_block_id())
                    };

                    self.push_block(
                        &Block {
                            opcode: "looks_say".to_string(),
                            parent: Some(parent_id.to_string()),
                            inputs: Some(inputs),
                            next: self.next_block_id.clone(),
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
                        Expr::Number(_) => " %s",
                        Expr::String(_) => " %s",
                        Expr::Identifier(_) => " %s",
                        // FIXME:: accept more than just string type
                        Expr::Bool(_) => todo!(),
                        Expr::Binary(_, _, _) => {
                            expr_block_id = Some(self.gen_block_id());
                            " %s"
                        }
                        _ => todo!(),
                    };

                    proc_codes.push_str(proc_code);

                    if index != 0 {
                        argument_ids.push_str(", ");
                    }

                    // TODO: just add error checking instead of crashing please
                    let arg_id = &self.arg_table.get(&func_name).unwrap()[index].0;
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
                                        self.get_var_id(self.scope_path.clone(), ident.clone())
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
                                &arg,
                                current_id.clone(),
                                expr_block_id.unwrap(),
                            );
                        }
                        _ => todo!(),
                    }
                }

                argument_ids.push_str("]");

                self.next_block_id = if (index + 1) >= body_len {
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
                            proccode: Some(proc_codes),
                            argumentids: Some(argument_ids),
                            argumentnames: None,
                            argumentdefaults: None,
                            warp: Some("false".to_string()),
                            ..Default::default()
                        }),
                        next: self.next_block_id.clone(),
                        ..Block::default()
                    },
                    current_id,
                );
            }
        }
    }

    fn compile_body_statements(
        &mut self,
        body: &Vec<Stmt>,
        parent_id: String,
        // Option<(VarName, VarId)>
        return_var: Option<(String, String)>,
    ) {
        self.scope_path.push(parent_id.clone());
        for (index, stmt) in body.into_iter().enumerate() {
            let current_id = self.gen_block_id();

            match stmt {
                Stmt::FunctionCall(func_name, args) => self.compile_function_call(
                    func_name.clone(),
                    args.clone(),
                    current_id,
                    parent_id.clone(),
                    index,
                    body.len(),
                ),
                Stmt::VariableDeclaration(var_name, var_type, expr) => {
                    println!(
                        "FORTNITE (DECLARATION) ->\n current: {} - parent: {} - next: {:?}",
                        current_id.clone(),
                        parent_id.clone(),
                        self.next_block_id,
                    );

                    let var_id =
                        self.push_var(self.scope_path.clone(), var_name.clone(), var_type.clone());

                    self.compile_variable_assignment(
                        var_name.clone(),
                        var_id,
                        var_type.clone(),
                        expr,
                        current_id,
                        parent_id.clone(),
                        index,
                        body.len(),
                    );
                }
                Stmt::While(cond, body_true) => {
                    self.compile_conditional_control(
                        "control_while",
                        cond,
                        (Some(body_true), None),
                        current_id,
                        parent_id.clone(),
                        index,
                        body.len(),
                    );
                }
                Stmt::If(cond, body_true, body_false) => {
                    // if-else
                    if let Some(body_false) = body_false {
                        self.compile_conditional_control(
                            "control_if_else",
                            cond,
                            (Some(body_true), Some(body_false)),
                            current_id,
                            parent_id.clone(),
                            index,
                            body.len(),
                        );
                    }
                    // if
                    else {
                        self.compile_conditional_control(
                            "control_if",
                            cond,
                            (Some(body_true), None),
                            current_id,
                            parent_id.clone(),
                            index,
                            body.len(),
                        );
                    }
                }
                Stmt::VariableAssignment(var_name, expr) => {
                    let (var_id, var_type) =
                        self.get_var(self.scope_path.clone(), var_name.to_string());

                    println!(
                        "FORTNITE (ASSIGNMENT) ->\n current: {} - parent: {} - next: {:?}",
                        current_id.clone(),
                        parent_id.clone(),
                        self.next_block_id,
                    );

                    self.compile_variable_assignment(
                        var_name.clone(),
                        var_id,
                        var_type.clone(),
                        expr,
                        current_id,
                        parent_id.clone(),
                        index,
                        body.len(),
                    );
                }
                Stmt::VariableMutation(var_name, op, mutation_value) => {
                    let var_id = self.get_var_id(self.scope_path.clone(), var_name.to_string());

                    // hack?
                    let op = match op {
                        crate::parser::MutationOperator::AddEqual => Operator::Plus,
                        crate::parser::MutationOperator::SubEqual => Operator::Minus,
                        crate::parser::MutationOperator::MultEqual => Operator::Star,
                        crate::parser::MutationOperator::DivEqual => Operator::Slash,
                    };

                    let expr = Expr::Binary(
                        Box::new(Expr::Identifier(var_name.to_string())),
                        op,
                        Box::new(mutation_value.clone()),
                    );

                    let value = self.value_from_expr(&expr, current_id.clone(), None);

                    let mut inputs = HashMap::new();
                    inputs.insert("VALUE".to_string(), value);

                    self.next_block_id = if (index + 1) >= body.len() {
                        None
                    } else {
                        Some(self.peek_next_block_id())
                    };

                    self.push_block(
                        &Block {
                            opcode: "data_setvariableto".to_string(),
                            parent: Some(parent_id.clone()),
                            inputs: Some(inputs),
                            fields: Some(json!({"VARIABLE": [var_name, var_id]})),
                            next: self.next_block_id.clone(),
                            ..Block::default()
                        },
                        current_id,
                    );
                }
                Stmt::Return(expr) => {
                    if !((index + 1) >= body.len()) {
                        panic!("return must be final statement in body/branch");
                    }

                    let return_var = return_var.clone().expect(
                        "cannot return in function that isn't declared as having a return type",
                    );

                    let value = self.value_from_expr(&expr, current_id.clone(), None);

                    let mut inputs = HashMap::new();
                    inputs.insert("VALUE".to_string(), value);

                    let stop_block_id = self.gen_block_id();

                    self.push_block(
                        &Block {
                            opcode: "data_setvariableto".to_string(),
                            parent: Some(parent_id.clone()),
                            inputs: Some(inputs),
                            fields: Some(json!({"VARIABLE": [return_var.0, return_var.1]})),
                            next: Some(stop_block_id.clone()),
                            ..Block::default()
                        },
                        current_id,
                    );

                    self.push_block(
                        &Block {
                            opcode: "control_stop".to_string(),
                            parent: Some(parent_id.clone()),
                            fields: Some(json!({"STOP_OPTION": ["this script", Value::Null]})),
                            shadow: Some(false),
                            top_level: Some(false),
                            mutation: Some(Mutation {
                                tag_name: "mutation".to_string(),
                                children: vec![],
                                hasnext: Some("false".to_string()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                        stop_block_id,
                    );
                }
                _ => panic!("statment: {:#?} not valid in body", stmt),
            }
        }
        self.scope_path.pop();
    }

    fn compile_variable_assignment(
        &mut self,
        var_name: String,
        var_id: String,
        var_type: Type,
        expr: &Expr,
        current_id: String,
        parent_id: String,
        index: usize,
        body_len: usize,
    ) {
        let value = self.value_from_expr(expr, parent_id.clone(), Some(current_id.clone()));

        println!(
            "FORTNITE (AFTER) ->\n current: {} - parent: {} - next: {:?}",
            current_id.clone(),
            parent_id.clone(),
            self.next_block_id,
        );

        let mut inputs = HashMap::new();
        inputs.insert("VALUE".to_string(), value);

        // let current_id = match expr {
        //     Expr::FunctionCall(_, _) => self.gen_block_id(),
        //     _ => current_id.clone(),
        // };

        self.next_block_id = if (index + 1) >= body_len {
            None
        } else {
            Some(self.peek_next_block_id())
        };

        self.push_block(
            &Block {
                opcode: "data_setvariableto".to_string(),
                parent: Some(parent_id.clone()),
                inputs: Some(inputs),
                fields: Some(json!({"VARIABLE": [var_name, var_id]})),
                next: self.next_block_id.clone(),
                ..Block::default()
            },
            current_id,
        );
    }

    fn compile_top_level_statement(&mut self, statement: &Stmt) {
        match statement {
            // TODO: scope for event handler
            Stmt::EventHandler(event, body) => match event {
                Event::FlagClicked => {
                    let flag_id = self.gen_block_id();

                    self.next_block_id = if body.len() > 0 {
                        Some(self.peek_next_block_id())
                    } else {
                        None
                    };

                    self.push_block(
                        &Block {
                            opcode: "event_whenflagclicked".to_string(),
                            next: self.next_block_id.clone(),
                            top_level: Some(true),
                            ..Block::default()
                        },
                        flag_id.clone(),
                    );

                    self.compile_body_statements(body, flag_id, None);
                }
                _ => todo!(),
            },
            Stmt::FunctionDeclaration(func_name, args, body, return_type) => {
                let return_var_name = format!("!func_var_{}", func_name);
                let mut parent_scope = self.scope_path.clone();
                parent_scope.pop();

                let return_var = match return_type {
                    Type::Number => Some((
                        return_var_name.clone(),
                        self.push_var(parent_scope, return_var_name.clone(), Type::Number),
                    )),
                    Type::String => Some((
                        return_var_name.clone(),
                        self.push_var(parent_scope, return_var_name.clone(), Type::String),
                    )),
                    Type::Bool => todo!(),
                    Type::Table => todo!(),
                    Type::Void => None,
                };

                self.arg_table.insert(func_name.clone(), vec![]);

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
                        proccode: Some(proc_code),
                        argumentids: Some(argument_ids),
                        argumentnames: Some(argument_names),
                        argumentdefaults: Some(argument_defaults),
                        warp: Some("false".to_string()),
                        ..Mutation::default()
                    }),
                    ..Default::default()
                };

                self.push_block(&proc_definition, definition_id.clone());
                self.push_block(&proc_prototype, prototype_id);

                for (id, block) in arg_blocks {
                    self.push_block(&block, id);
                }

                self.compile_body_statements(body, definition_id.clone(), return_var);
            }
            _ => panic!("statement type: {:#?} cannot be top-level", statement),
        }
    }

    fn push_block(&mut self, block: &Block, id: String) {
        println!(
            "BLOCK PUSHED [{}] -> [{:?}]: {}",
            id, self.next_block_id, block.opcode
        );

        self.project.targets[self.target_index]
            .blocks
            .insert(id, block.clone());
    }

    /// adds a variable to both the internal table used by the compiler,
    /// and to the Project struct that's eventually serialized
    /// returns the ID of the variable
    fn push_var(&mut self, scope_path: Vec<String>, var_name: String, var_type: Type) -> String {
        // add an empty hashmap for the scope if it doesn't exist within the var table
        // TODO: add heiarchy to scope
        // FIXME: this is completely broken, trusting the user to not redeclare vars for now lol
        // if !self
        //     .var_table
        //     .clone()
        //     .into_keys()
        //     .collect::<Vec<Scope>>()
        //     .contains(&scope)
        // {
        //     self.var_table.insert(scope.clone(), HashMap::new());
        // } else {
        //     // panic!("variable {:?} already exists in scope", var_name);
        // }

        let keys = self
            .var_table
            .clone()
            .into_keys()
            .collect::<Vec<Vec<String>>>();

        // check if scope exists
        // if it doesn't, create it
        // TODO: error checking
        if !keys.contains(&scope_path) {
            self.var_table.insert(scope_path.clone(), HashMap::new());
        }

        let var_id = self.gen_var_id().to_string();

        self.var_table
            .get_mut(&scope_path)
            .unwrap()
            .insert(var_name.clone(), (var_id.clone(), var_type.clone()));

        self.project.targets[self.target_index]
            .variables
            .insert(var_id.clone(), json!([var_name, 0]));

        println!("{:#?} - {:#?}", scope_path, var_name);
        var_id
    }

    fn get_var_id(&self, scope_path: Vec<String>, var_name: String) -> String {
        self.get_var(scope_path, var_name).0
    }

    fn get_var(&self, scope_path: Vec<String>, var_name: String) -> (String, Type) {
        let mut scope_path = scope_path.clone();

        // this is really bad
        loop {
            let scope = self.var_table.get(&scope_path);

            if scope.is_none() {
                match scope_path.pop() {
                    Some(_) => continue,
                    None => panic!("variable {:?} not found in scope", var_name),
                }
            }

            let scope = scope.unwrap();

            let var = scope.get(&var_name);

            if var.is_none() {
                match scope_path.pop() {
                    Some(_) => continue,
                    None => panic!("variable {:?} not found in scope", var_name),
                }
            }

            return var.unwrap().clone();
        }
    }

    fn var_exists(&self, scope_path: Vec<String>, var_name: String) -> bool {
        let scope_table = self.var_table.get(&scope_path);
        let scoped_table = if let Some(table) = scope_table {
            table
        } else {
            return false;
        };

        match scoped_table.get(&var_name) {
            Some(_) => true,
            None => false,
        }
    }

    // DO NOT TOUCH LMFAO
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
