use log::error;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
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
    Excl,   // !
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
    And,    // &
    AndAnd, // &&
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
    Arrow,
    MethodScope, // |>

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

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone)]
pub struct Lexer<'src> {
    input: &'src str,
    span: Span,
    pos: usize,
    line: usize,
    col: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    span: Span,
}

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Lexing failed due to one or more errors.")]
    LexerFailed,
    #[error("Passed in an invalid token")]
    LexerInvalid,
}

type Result<T> = std::result::Result<T, LexerError>;

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            input: source,
            col: 0,
            pos: 0,
            line: 1,
            span: Span::default(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        self.input[self.pos..].chars().nth(1)
    }

    fn advance(&mut self) -> Option<char> {
        let chr = self.peek();

        self.pos += 1;

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

        let number: String = self.input[base..self.pos].to_string();
        if let Ok(number) = number.parse::<i64>() {
            return TokenType::Integer(number);
        }

        error!(
            "Invalid Integer ({}), at Line: {}, Column: {}",
            number, self.line, self.col
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
            } else {
                break;
            }
        }

        let id = &self.input[base..self.pos];

        if let Some(token_type) = keyword.get(&id) {
            return Ok(token_type.to_owned());
        }

        Ok(TokenType::Identifier(id.to_string()))
    }

    fn accept(&mut self, strs: &str) -> bool {
        let end = self.pos + strs.len();

        if end <= self.input.len() && self.input[self.pos..].starts_with(strs) {
            self.pos += strs.len();
            return true;
        }

        false
    }

    fn accept_multichar(&mut self, strs: &str, token_type: TokenType) -> Option<Token> {
        if self.accept(strs) {
            return Some(Token {
                token_type,
                span: Span::new(self.pos, self.pos + strs.len()),
            });
        }
        None
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let base = self.pos;

        if self.is_end() {
            return Token {
                token_type: TokenType::Eof,
                span: Span::new(base, self.pos),
            };
        }

        // Handle numbers
        if self.peek().unwrap().is_numeric() {
            let number = self.handle_number();

            return Token {
                span: Span::new(base, self.pos),
                token_type: number,
            };
        }
        // Handle Identifiers
        if self.peek().unwrap().is_alphabetic() || self.peek().unwrap() == '_' {
            let id = self.handle_identifier();

            return Token {
                span: Span::new(base, self.pos),
                token_type: id.unwrap(),
            };
        }

        // TODO: Handle strings
        // if curr_char ...

        if let Some(tok) = self.accept_multichar("<>", TokenType::Neq) {
            return tok;
        }

        if let Some(tok) = self.accept_multichar("->", TokenType::Arrow) {
            return tok;
        }

        if let Some(tok) = self.accept_multichar("&&", TokenType::AndAnd) {
            return tok;
        }

        if let Some(tok) = self.accept_multichar("==", TokenType::EqEq) {
            return tok;
        }

        if let Some(tok) = self.accept_multichar(">=", TokenType::EqEq) {
            return tok;
        }

        if let Some(tok) = self.accept_multichar("<=", TokenType::EqEq) {
            return tok;
        }

        let kind = match self.peek().unwrap() {
            // arithmetic
            '+' => {
                self.advance();
                TokenType::Plus
            }
            '-' => {
                self.advance();
                TokenType::Minus
            }
            '*' => {
                self.advance();
                TokenType::Star
            }
            '/' => {
                self.advance();
                TokenType::Divide
            }
            '%' => {
                self.advance();
                TokenType::Modulo
            }
            '^' => {
                self.advance();
                TokenType::Exp
            }

            // bang / not
            '!' => {
                self.advance();
                TokenType::Excl
            }

            // equals
            '=' => {
                self.advance();
                TokenType::Eq
            }

            // comparisons
            '<' => {
                self.advance();
                TokenType::Lt
            }
            '>' => {
                self.advance();
                TokenType::Gt
            }

            // boolean‐and (single &)
            '&' => {
                self.advance();
                TokenType::And
            }

            // match‐arm (single |)
            '|' => {
                self.advance();
                TokenType::Arm
            }

            // grouping & punctuation
            '(' => {
                self.advance();
                TokenType::LParam
            }
            ')' => {
                self.advance();
                TokenType::RParam
            }
            '[' => {
                self.advance();
                TokenType::LBracket
            }
            ']' => {
                self.advance();
                TokenType::RBracket
            }
            ',' => {
                self.advance();
                TokenType::Comma
            }
            ':' => {
                self.advance();
                TokenType::Colon
            }

            // anything else → undefined token
            _ => {
                self.advance();
                TokenType::Undefined
            }
        };
        // handle single chars
        Token {
            token_type: kind,
            span: Span::new(base, self.pos),
        }
    }

    pub fn scan_all(&mut self) -> Result<Vec<Token>> {
        let mut token: Vec<Token> = Vec::new();

        while self.pos < self.input.len() {
            let tok = self.next_token();
            token.push(tok.clone());
            if tok.token_type == TokenType::Eof {
                break;
            }
        }
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let mut lex = Lexer::new("def aa() -> = 1 + s_s");
        let def = lex.next_token();
        assert_eq!(def.token_type, TokenType::Def, "Expected: `Def`");

        let aa_id = lex.next_token();
        assert_eq!(
            aa_id.token_type,
            TokenType::Identifier("aa".into()),
            "Expected: `Identifier`"
        );
        let l_param = lex.next_token();
        assert_eq!(l_param.token_type, TokenType::LParam, "Expected: `(`");

        let r_param = lex.next_token();
        assert_eq!(r_param.token_type, TokenType::RParam, "Expected: `)`");

        let arrow = lex.next_token();
        assert_eq!(arrow.token_type, TokenType::Arrow, "Expected: `->`");

        let num = lex.next_token();
        assert_eq!(num.token_type, TokenType::Integer(1), "Expected: `Integer`");

        let plus = lex.next_token();
        assert_eq!(plus.token_type, TokenType::Plus, "Expected: `Plus`");

        let var_id = lex.next_token();
        assert_eq!(
            var_id.token_type,
            TokenType::Identifier("s_s".into()),
            "Expected: `Identifier`"
        );
    }
}
