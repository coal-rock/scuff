#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Bang,

    EqualEqual,
    BangEqual,

    Greater,
    Less,

    GreaterEqual,
    LessEqual,

    Minus,
    Plus,

    Ampersand,

    Slash,
    Star,
    Caret,

    // hack?
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    String,
    Bool,
    Table,
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Semicolon,
    Colon,
    Equal,
    Arrow,

    Operator(Operator),
    Type(Type),

    Ident(String),
    String(String),
    Number(f64),
    Bool(bool),

    If,
    Else,
    And,
    Or,
    Break,
    Continue,
    Return,
    Let,

    While,
    For,

    Function,
    Event,

    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize) -> Token {
        Token { token_type, line }
    }

    pub fn to_string(&self) -> String {
        format!("{:?}", self.token_type)
    }
}

// hacky macro for extracting the value of an enum of known type
#[macro_export]
macro_rules! extract {
    ($token:expr, $expected:path) => {
        if let $expected(value) = $token {
            value
        } else {
            panic!()
        }
    };
}
