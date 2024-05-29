use crate::{
    error::parse_error,
    extract,
    token::{Token, TokenType},
};

#[derive(Debug)]
pub enum Expr {
    Number(f64),
    String(String),
    Identifier(String),
    Bool(bool),
    Binary(Box<Expr>, char, Box<Expr>),
}

#[derive(Debug)]
pub enum VarType {
    Number,
    String,
    Table,
    Bool,
}

impl VarType {
    fn from(token_type: TokenType) -> VarType {
        match token_type {
            TokenType::NumberType => VarType::Number,
            TokenType::StringType => VarType::String,
            TokenType::TableType => VarType::Table,
            TokenType::BoolType => VarType::Bool,
            _ => panic!("Expected a type, recieved: {:?}", token_type),
        }
    }
}

#[derive(Debug)]
pub enum Stmt {
    Expression(Expr),
    VariableDeclaration(String, VarType, Expr),
    FunctionDeclaration(String, Vec<String>, Vec<Stmt>),
}

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

    // returns the type associated with the current token
    fn current_token(&self) -> TokenType {
        self.tokens.get(self.position).unwrap().token_type.clone()
    }

    // returns the token including it's metadata such as line number and position
    fn current_token_full(&self) -> &Token {
        self.tokens.get(self.position).unwrap()
    }

    fn advance(&mut self) -> TokenType {
        self.position += 1;
        self.current_token()
    }

    fn expect(&mut self, expected: TokenType) -> TokenType {
        self.position += 1;

        if std::mem::discriminant(&self.current_token()) != std::mem::discriminant(&expected) {
            parse_error(
                self.current_token_full().clone(),
                format!("Expected: {:?}, got: {:?}", expected, self.current_token()),
            )
        }

        self.current_token()
    }

    fn expect_multiple(&mut self, expected: Vec<TokenType>) -> TokenType {
        self.position += 1;

        if !expected.contains(&self.current_token()) {
            parse_error(
                self.current_token_full().clone(),
                format!(
                    "Expected one of: {:?}, got: {:?}",
                    expected,
                    self.current_token()
                ),
            )
        }

        self.current_token()
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();

        while self.current_token() != TokenType::EOF {
            let statement = self.parse_statement();
            println!("{:#?}", statement);
            statements.push(statement);
        }

        statements
    }

    fn parse_statement(&mut self) -> Stmt {
        match self.current_token() {
            // let var_name: var_type = expression;
            TokenType::Let => {
                let ident = extract!(
                    self.expect(TokenType::Ident(String::new())),
                    TokenType::Ident
                );

                self.expect(TokenType::Colon);

                let var_type = self.expect_multiple(vec![
                    TokenType::BoolType,
                    TokenType::StringType,
                    TokenType::NumberType,
                    TokenType::TableType,
                ]);

                self.expect(TokenType::Equal);
                let expr = self.parse_expression();
                self.expect(TokenType::Semicolon);
                return Stmt::VariableDeclaration(ident, VarType::from(var_type), expr);
            }
            _ => panic!(),
        }
    }

    fn parse_expression(&mut self) -> Expr {
        match self.advance() {
            TokenType::Number(value) => Expr::Number(value),
            TokenType::String(value) => Expr::String(value),
            TokenType::Ident(value) => Expr::Identifier(value),
            TokenType::Bool(value) => Expr::Bool(value),
            _ => panic!(
                "token type: {:?} not expected in expression",
                self.current_token()
            ),
        }
    }
}
