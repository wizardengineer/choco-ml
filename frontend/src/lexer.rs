use log::error;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum TokenType {
    Def, // def func()...

    RParam,
    LParam,
    RBracket,
    LBracket,
    Comma,
    Colon,

    Plus,   // +
    Star,   // *
    Minus,  // -
    Divide, // /

    // Operator tokens
    // TODO: Add more
    Eq,     // =
    EqEq,   // ==
    Lt,     // <
    Gt,     // >
    LtEq,   // <=
    GtEq,   // >=
    Neq,    // ~=
    Modulo, // %
    Exp,    // ^

    Not,
    //RQuote,
    //LQuote,
    TypeRecord, // for records
    Return,

    If,
    ElseIf,
    Else,
    Match,
    With,
    Arm, // |
    Let,
    From,
    MethodFuncsBody, // |>

    Identifier(String),
    String(String),

    Undefined,
    Integer(i64),
    Eof,
}

#[derive(Debug, Clone, Default)]
pub struct Span {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone)]
pub struct Lexer {
    input: Vec<char>,
    span: Span,
    pos: usize,
    line: usize,
    col: usize,
}

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Lexing failed due to one or more errors.")]
    LexerFailed,
}

type Result<T> = std::result::Result<T, LexerError>;

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            col: 0,
            pos: 0,
            line: 1,
            span: Span::default(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let chr = self.peek();

        if chr.is_some() {
            if chr == Some('\n') {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }
        }
        chr
    }

    fn is_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(ch) if ch.is_whitespace() => {
                    self.advance();
                }
                Some('#') => {
                    self.skip_comments();
                }
                _ => break,
            };
        }
    }

    fn skip_comments(&mut self) {
        while self.peek() == Some('\n') {
            // consume the #
            self.advance();
            while let Some(ch) = self.peek() {
                self.advance();
                if ch == '\n' {
                    break;
                }
            }
        }
    }

    fn handle_number(&mut self) -> TokenType {
        let base = self.pos;
        while let Some(ch) = self.peek() {
            if !ch.is_ascii_digit() {
                break;
            }

            // consume the next number
            self.advance();
        }

        let number: String = self.input[base..self.pos].iter().collect();
        if let Ok(number) = number.parse::<i64>() {
            return TokenType::Integer(number);
        }

        error!(
            "Invalid Integer ({}), at Line: {}, Position: {}",
            number, self.line, self.pos
        );

        TokenType::Undefined
    }

    fn handle_identifier(&mut self) -> Result<TokenType> {
        let keyword = HashMap::from([
            ("def", TokenType::Def),
            ("if", TokenType::If),
            ("elseif", TokenType::ElseIf),
            ("else", TokenType::Else),
            ("let", TokenType::Let),
            ("return", TokenType::Return),
            ("type", TokenType::TypeRecord),
            ("not", TokenType::Not),
            ("with", TokenType::With),
            ("match", TokenType::Match),
            ("from", TokenType::From),
        ]);

        let base = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() || ch == '_' {
                self.advance();
            }
        }

        let id = self.input[base..=self.pos].iter().collect::<String>();

        if let Some(token_type) = keyword.get(&id.as_str()) {
            return Ok(token_type.to_owned());
        }

        Ok(TokenType::Identifier(id))
    }
}
