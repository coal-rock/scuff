use crate::parser::Stmt;
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
        // for statement in self.ast {
        //     self.compile_statement(statement);
        // }
        //
        &self.project
    }

    pub fn compile_statement(&mut self, statement: Stmt) {
        match statement {
            _ => {}
        }
    }

    fn gen_id(&mut self) -> usize {
        self.block_id += 1;
        self.block_id
    }
}
