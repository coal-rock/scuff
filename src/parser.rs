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
pub enum DataType {
    Number,
    String,
    Table,
    Bool,
    Void,
}

impl DataType {
    fn from(token_type: TokenType) -> DataType {
        match token_type {
            TokenType::NumberType => DataType::Number,
            TokenType::StringType => DataType::String,
            TokenType::TableType => DataType::Table,
            TokenType::BoolType => DataType::Bool,
            TokenType::VoidType => DataType::Void,
            _ => panic!("Expected a type, recieved: {:?}", token_type),
        }
    }
}

#[derive(Debug)]
pub enum Stmt {
    Expression(Expr),
    VariableDeclaration(String, DataType, Expr),
    FunctionDeclaration(String, Vec<(String, DataType)>, Vec<Stmt>, DataType),
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
        let token = self.tokens.get(self.position).unwrap().token_type.clone();
        println!("{:#?}", token);
        token
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

    fn expect_type(&mut self) -> TokenType {
        self.expect_multiple(vec![
            TokenType::BoolType,
            TokenType::StringType,
            TokenType::NumberType,
            TokenType::TableType,
            TokenType::VoidType,
        ])
    }

    fn peek_next(&self) -> TokenType {
        self.tokens
            .get(self.position + 1)
            .unwrap()
            .token_type
            .clone()
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
                let var_type = self.expect_type();
                self.expect(TokenType::Equal);
                let expr = self.parse_expression();
                self.expect(TokenType::Semicolon);
                self.advance();
                return Stmt::VariableDeclaration(ident, DataType::from(var_type), expr);
            }
            // function function_name(arg_name: arg_type) -> return_type { body }
            TokenType::Function => {
                let function_name = extract!(
                    self.expect(TokenType::Ident(String::new())),
                    TokenType::Ident
                );

                let mut arguments: Vec<(String, DataType)> = Vec::new();

                self.expect(TokenType::LeftParen);

                while self.peek_next() != TokenType::RightParen {
                    let arg_name = extract!(
                        self.expect(TokenType::Ident(String::new())),
                        TokenType::Ident
                    );

                    self.expect(TokenType::Colon);

                    let arg_type = self.expect_type();

                    if self.peek_next() == TokenType::Comma {
                        self.advance();
                    }

                    arguments.push((arg_name, DataType::from(arg_type)));
                }

                self.advance();
                self.expect(TokenType::Arrow);
                let return_type = DataType::from(self.expect_type());

                self.expect(TokenType::LeftBrace);
                self.advance();

                let mut body_statements: Vec<Stmt> = Vec::new();

                while self.current_token() != TokenType::RightBrace {
                    body_statements.push(self.parse_statement());
                }

                self.advance();

                Stmt::FunctionDeclaration(function_name, arguments, body_statements, return_type)
            }
            _ => panic!("Unhandled token in statement: {:?}", self.current_token()),
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
