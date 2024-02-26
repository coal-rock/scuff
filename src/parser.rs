use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            position: 0,
        }
    }

    // pub fn parse(&mut self) -> Vec<Statement> {}
}

type Identifier = String;
type Body = Vec<Statement>;

#[derive(Debug, Clone)]
pub enum ReturnType {
    Number,
    String,
    Bool,
    Table,
    Void,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub identifier: String,
    pub param_type: ReturnType,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Function(Identifier, Vec<Param>, Body, ReturnType),
    Let(Identifier, Expression, ReturnType),
    If(Box<Expression>, Box<Statement>, Option<Box<Expression>>),
    While(Box<Expression>, Box<Statement>),
    Expression(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Number(f64),
    Boolean(bool),
    String(String),
    Variable(Identifier),
    Binary(Box<Expression>, BinaryOperator, Box<Expression>),
    Assign(Identifier, Box<Expression>),
    Error,
    Call(Box<Expression>, Vec<Expression>),
}

#[derive(PartialEq, PartialOrd, Copy, Clone)]
pub enum Precedence {
    None,
    Assign, // =
    Or,
    And,
    Equality,   // == !=
    Comparison, // < <= > >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // ()
    List,       // []
    Primary,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BinaryOperator {
    Slash,
    Star,
    Plus,
    Minus,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    BangEqual,
    EqualEqual,
}
