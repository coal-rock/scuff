use crate::error::error;
use crate::token::{Operator, Token, TokenType, Type};

pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: &String) -> Lexer {
        Lexer {
            source: source.to_string(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    // rust has these in-built, but we define them slightly
    // differently
    fn is_digit(&self, character: char) -> bool {
        match character {
            '0'..='9' => true,
            _ => false,
        }
    }

    fn is_alpha(&self, character: char) -> bool {
        match character {
            'a'..='z' | 'A'..='Z' | '_' => true,
            _ => false,
        }
    }

    fn is_alpha_numeric(&self, character: char) -> bool {
        self.is_digit(character) || self.is_alpha(character)
    }

    fn advance(&mut self) -> char {
        let char = self
            .source
            .chars()
            .collect::<Vec<char>>()
            .get(self.current)
            .unwrap_or(&'\0')
            .clone();

        self.current += 1;

        char
    }

    fn peek(&self) -> char {
        let chars = self.source.chars().collect::<Vec<char>>();
        let char = chars.get(self.current);

        if self.is_at_end() {
            return '\0';
        }

        return char.unwrap().clone();
    }

    fn peek_next(&self) -> char {
        let chars = self.source.chars().collect::<Vec<char>>();
        let char = chars.get(self.current + 1);

        if self.is_at_end() {
            return '\0';
        }

        return char.unwrap().clone();
    }

    fn compare(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.peek() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    // Basically ternary
    fn add_token_cond(&mut self, expected: char, matches: TokenType, fails: TokenType) {
        if self.compare(expected) {
            self.add_token(matches);
        } else {
            self.add_token(fails);
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        let string: String = self.source[self.start..self.current].to_string();

        self.tokens.push(Token::new(token_type, self.line));
    }

    fn string_to_keyword(&self, string: String) -> Option<TokenType> {
        match string.as_str() {
            "if" => Some(TokenType::If),
            "else" => Some(TokenType::Else),
            "and" => Some(TokenType::And),
            "or" => Some(TokenType::Or),
            "break" => Some(TokenType::Break),
            "continue" => Some(TokenType::Continue),
            "return" => Some(TokenType::Return),
            "while" => Some(TokenType::While),
            "for" => Some(TokenType::For),
            "function" => Some(TokenType::Function),
            "event" => Some(TokenType::Event),
            "true" => Some(TokenType::Bool(true)),
            "false" => Some(TokenType::Bool(false)),
            "let" => Some(TokenType::Let),
            "number" => Some(TokenType::Type(Type::Number)),
            "string" => Some(TokenType::Type(Type::String)),
            "bool" => Some(TokenType::Type(Type::Bool)),
            "table" => Some(TokenType::Type(Type::Table)),
            "void" => Some(TokenType::Type(Type::Void)),
            _ => None,
        }
    }

    fn lex_string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1
            };
            self.advance();
        }

        if self.is_at_end() {
            error(self.line, "Unterminted string.".to_string());
        }

        self.advance();

        let string: String = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token(TokenType::String(string));
    }

    // this might be hacky, maybe in the future
    // we should handle number as ints instead of floats
    fn lex_number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let number = self.source[self.start..self.current].to_string();
        let number: f64 = number.parse().unwrap();

        self.add_token(TokenType::Number(number));
    }

    fn lex_identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let string = self.source[self.start..self.current].to_string();
        let token = self.string_to_keyword(string.clone());

        if token.is_none() {
            self.add_token(TokenType::Ident(string));
        } else {
            self.add_token(token.unwrap());
        }
    }

    pub fn lex_token(&mut self) {
        let char = self.advance();

        match char {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '[' => self.add_token(TokenType::LeftBracket),
            ']' => self.add_token(TokenType::RightBracket),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            ';' => self.add_token(TokenType::Semicolon),
            ':' => self.add_token(TokenType::Colon),
            '+' => self.add_token(TokenType::Operator(Operator::Plus)),
            '&' => self.add_token(TokenType::Operator(Operator::Ampersand)),
            '*' => self.add_token(TokenType::Operator(Operator::Star)),
            '^' => self.add_token(TokenType::Operator(Operator::Caret)),
            '!' => self.add_token_cond(
                '=',
                TokenType::Operator(Operator::BangEqual),
                TokenType::Operator(Operator::Bang),
            ),
            '=' => self.add_token_cond(
                '=',
                TokenType::Operator(Operator::EqualEqual),
                TokenType::Equal,
            ),
            '<' => self.add_token_cond(
                '=',
                TokenType::Operator(Operator::LessEqual),
                TokenType::Operator(Operator::Less),
            ),
            '>' => self.add_token_cond(
                '=',
                TokenType::Operator(Operator::GreaterEqual),
                TokenType::Operator(Operator::Greater),
            ),
            '-' => self.add_token_cond('>', TokenType::Arrow, TokenType::Operator(Operator::Minus)),
            '/' => {
                if self.compare('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Operator(Operator::Slash));
                }
            }
            '"' => self.lex_string(),
            '\t' | ' ' => {}
            '\n' => self.line += 1,
            '0'..='9' => self.lex_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier(),
            _ => {
                if !self.peek().is_ascii_whitespace() {
                    error(self.line, format!("Unexpected character: {:?}", char))
                }
            }
        }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.lex_token();
        }

        self.add_token(TokenType::EOF);
        return self.tokens.clone();
    }
}
