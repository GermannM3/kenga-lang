use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // keywords
    Fn,
    Let,
    Var,
    Return,
    If,
    Else,
    While,
    For,
    In,
    Break,
    Continue,
    Struct,
    Import,
    On,
    True,
    False,
    Ttl,
    Intrinsic,
    Const,

    // types
    TypeI64,
    TypeF64,
    TypeBool,
    TypeStr,
    TypeTensor,
    TypeList,
    TypeMemory,

    // literals / idents
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),

    // symbols
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,
    Arrow,
    FatArrow,
    Assign,
    Dot,
    DotDot,
    At,

    // operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    // bitwise (single & / | / ^ / ~ — disambiguate from logical &&/|| by lexer)
    Amp,
    Pipe,
    Caret,
    Tilde,
    Shl,
    Shr,

    DurationMs(u64),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
