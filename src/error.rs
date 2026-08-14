use std::fmt;

#[derive(Debug, Clone)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

#[derive(Debug)]
pub struct KengaError {
    pub message: String,
    pub span: Option<Span>,
}

impl KengaError {
    pub fn new(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }

    pub fn at(message: impl Into<String>, span: Span) -> Self {
        Self::new(message, Some(span))
    }
}

impl fmt::Display for KengaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.span {
            Some(s) => write!(f, "error at {}: {}", s, self.message),
            None => write!(f, "error: {}", self.message),
        }
    }
}

impl std::error::Error for KengaError {}

pub type Result<T> = std::result::Result<T, KengaError>;
