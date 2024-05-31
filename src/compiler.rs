use crate::parser::Stmt;
use crate::project::{Block, Project};

pub struct Compiler {
    ast: Vec<Stmt>,
    project: Project,
    block_id: usize,
    parent: Option<usize>,
}
