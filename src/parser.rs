use std::i8;

use crate::{
    error::parse_error,
    extract,
    token::{Operator, Token, TokenType, Type},
};

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
    Identifier(String),
    Bool(bool),
    Binary(Box<Expr>, Operator, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Event {
    FlagClicked,
    KeyPressed(Key),
}

#[derive(Debug, Clone)]
pub enum Key {
    Any,
    Space,
    Up,
    Down,
    Left,
    Right,
    Char(char), // a..=z, 0..=9
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    VariableDeclaration(String, Type, Expr), // name, type, value
    VariableAssignment(String, Expr),
    FunctionDeclaration(String, Vec<(String, Type)>, Vec<Stmt>, Type), // name, arguments, body, return type
    EventHandler(Event, Vec<Stmt>),                                    // event, body
    FunctionCall(String, Vec<Expr>),                                   // name, arguments
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>), // condition, block if true, block if false
    While(Expr, Vec<Stmt>),                 // condition, block if true
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
            statements.push(statement);
        }

        statements
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        self.expect(TokenType::LeftBrace);
        self.advance();

        let mut body_statements: Vec<Stmt> = Vec::new();

        while self.current_token() != TokenType::RightBrace {
            body_statements.push(self.parse_statement());
        }

        self.advance();

        body_statements
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
                let body_statements = self.parse_block();

                Stmt::FunctionDeclaration(function_name, args, body_statements, return_type)
            }
            TokenType::Event => {
                let event_name = extract!(
                    self.expect(TokenType::Ident(String::new())),
                    TokenType::Ident
                );

                let event = match event_name.as_str() {
                    "flag_clicked" => Event::FlagClicked,
                    "key_pressed" => {
                        self.expect(TokenType::LeftParen);

                        let key = if let TokenType::Number(num) = self.peek_next() {
                            self.advance();

                            if num.fract() != 0.0 {
                                panic!("key press must be whole number");
                            }

                            let num = num as u32;

                            match num {
                                0..=9 => Key::Char(char::from_digit(num, 10).unwrap()),
                                _ => panic!(),
                            }
                        } else {
                            let key = extract!(
                                self.expect(TokenType::Ident(String::new())),
                                TokenType::Ident
                            );

                            match key.as_str() {
                                "any" => Key::Any,
                                "space" => Key::Space,
                                "up_arrow" => Key::Up,
                                "down_arrow" => Key::Down,
                                "left_arrow" => Key::Left,
                                "right_arrow" => Key::Right,
                                key_string if key.len() == 1 => {
                                    let char = key_string.chars().collect::<Vec<char>>()[0];

                                    match char {
                                        'a'..='z' => Key::Char(char),
                                        _ => panic!(),
                                    }
                                }
                                _ => panic!(),
                            }
                        };

                        self.expect(TokenType::RightParen);
                        Event::KeyPressed(key)
                    }
                    _ => panic!(),
                };

                let body = self.parse_block();

                Stmt::EventHandler(event, body)
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
                // variable assignment
                TokenType::Equal => {
                    self.expect(TokenType::Equal);
                    let value = self.parse_expression();
                    self.expect(TokenType::Semicolon);
                    self.advance();

                    Stmt::VariableAssignment(ident, value)
                }
                _ => panic!(
                    "unexpected token after parsing ident: {:?}",
                    self.peek_next()
                ),
            },
            TokenType::If => {
                let condition = self.parse_expression();

                let if_true = self.parse_block();

                let if_false = if self.peek_next() == TokenType::LeftBrace {
                    Some(self.parse_block())
                } else {
                    None
                };

                Stmt::If(condition, if_true, if_false)
            }
            TokenType::While => {
                let condition = self.parse_expression();
                let body = self.parse_block();

                Stmt::While(condition, body)
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
