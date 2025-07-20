#[derive(Debug, Clone)]
pub enum Token {
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

    //RQuote,
    //LQuote,
    Type,
    Return,

    If,
    ElseIf,
    Else,
    Match,
    Arm, // |

    Identifier,

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

    pub fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    pub fn peek_next(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
    }

    pub fn advance(&mut self) -> Option<char> {
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

    pub fn is_end(&self) -> bool {
        self.pos >= self.input.len()
    }
}
