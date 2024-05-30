use crate::{
    error::parse_error,
    extract,
    token::{Operator, Token, TokenType, Type},
};

#[derive(Debug)]
pub enum Expr {
    Number(f64),
    String(String),
    Identifier(String),
    Bool(bool),
    Binary(Box<Expr>, Operator, Box<Expr>),
}

#[derive(Debug)]
pub enum Stmt {
    Expression(Expr),
    VariableDeclaration(String, Type, Expr),
    FunctionDeclaration(String, Vec<(String, Type)>, Vec<Stmt>, Type),
    FunctionCall(String, Vec<Expr>),
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

    fn expect_type(&mut self) -> Type {
        extract!(self.expect(TokenType::Type(Type::Void)), TokenType::Type)
    }

    fn expect_operator(&mut self) -> Operator {
        extract!(
            self.expect(TokenType::Operator(Operator::None)),
            TokenType::Operator
        )
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
                return Stmt::VariableDeclaration(ident, var_type, expr);
            }
            // function function_name(arg_name: arg_type) -> return_type { body }
            TokenType::Function => {
                let function_name = extract!(
                    self.expect(TokenType::Ident(String::new())),
                    TokenType::Ident
                );

                let mut args: Vec<(String, Type)> = Vec::new();

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

                    args.push((arg_name, arg_type));
                }

                self.advance();
                self.expect(TokenType::Arrow);
                let return_type = self.expect_type();

                self.expect(TokenType::LeftBrace);
                self.advance();

                let mut body_statements: Vec<Stmt> = Vec::new();

                while self.current_token() != TokenType::RightBrace {
                    body_statements.push(self.parse_statement());
                }

                self.advance();

                Stmt::FunctionDeclaration(function_name, args, body_statements, return_type)
            }
            TokenType::Ident(ident) => match self.peek_next() {
                // function call
                TokenType::LeftParen => {
                    self.expect(TokenType::LeftParen);

                    let mut args: Vec<Expr> = Vec::new();

                    while self.peek_next() != TokenType::RightParen {
                        let arg = self.parse_expression();

                        if self.peek_next() == TokenType::Comma {
                            self.advance();
                        }

                        args.push(arg);
                    }

                    self.expect(TokenType::RightParen);
                    self.expect(TokenType::Semicolon);

                    self.advance();
                    Stmt::FunctionCall(ident, args)
                }
                _ => panic!(
                    "unexpected token after parsing ident: {:?}",
                    self.peek_next()
                ),
            },
            _ => panic!("Unhandled token in statement: {:?}", self.current_token()),
        }
    }

    fn parse_expression(&mut self) -> Expr {
        match self.advance() {
            TokenType::Number(value) => Expr::Number(value),
            TokenType::String(value) => Expr::String(value),
            TokenType::Ident(value) => Expr::Identifier(value),
            TokenType::Bool(value) => Expr::Bool(value),
            TokenType::LeftParen => {
                let left = self.parse_expression();
                let op = self.expect_operator();
                let right = self.parse_expression();
                self.expect(TokenType::RightParen);
                return Expr::Binary(Box::new(left), op, Box::new(right));
            }
            _ => panic!(
                "token type: {:?} not expected in expression",
                self.current_token()
            ),
        }
    }
}
