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
    Minus,
    Plus,
    Semicolon,
    Colon,
    Slash,
    Star,
    Caret,

    Bang,
    Equal,

    EqualEqual,
    BangEqual,

    Greater,
    Less,

    GreaterEqual,
    LessEqual,

    Arrow,

    Ident(String),
    String(String),
    Number(f64),

    True,
    False,

    If,
    Else,
    And,
    Or,
    Break,
    Continue,
    Return,
    Let,

    NumberType,
    StringType,
    BoolType,
    TableType,
    VoidType,

    While,
    For,

    Function,

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
