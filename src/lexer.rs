use crate::error::{KengaError, Result, Span};
use crate::token::{Token, TokenKind};

pub struct Lexer<'a> {
    src: &'a str,
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            chars: src.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = matches!(tok.kind, TokenKind::Eof);
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    fn span(&self) -> Span {
        Span::new(self.line, self.col)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek2(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn skip_ws_and_comments(&mut self) -> Result<()> {
        loop {
            while matches!(self.peek(), Some(c) if c.is_whitespace()) {
                self.bump();
            }
            if self.peek() == Some('/') && self.peek2() == Some('/') {
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.bump();
                }
                continue;
            }
            if self.peek() == Some('/') && self.peek2() == Some('*') {
                let start = self.span();
                self.bump();
                self.bump();
                loop {
                    match self.peek() {
                        None => {
                            return Err(KengaError::at("unclosed block comment", start));
                        }
                        Some('*') if self.peek2() == Some('/') => {
                            self.bump();
                            self.bump();
                            break;
                        }
                        _ => {
                            self.bump();
                        }
                    }
                }
                continue;
            }
            break;
        }
        Ok(())
    }

    fn next_token(&mut self) -> Result<Token> {
        self.skip_ws_and_comments()?;
        let start = self.span();
        let Some(c) = self.peek() else {
            return Ok(Token::new(TokenKind::Eof, start));
        };

        if c.is_ascii_alphabetic() || c == '_' {
            return Ok(self.ident_or_keyword(start));
        }
        if c.is_ascii_digit() {
            return self.number(start);
        }
        if c == '"' {
            return self.string(start);
        }

        self.bump();
        let kind = match c {
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            ':' => TokenKind::Colon,
            '.' => {
                if self.peek() == Some('.') {
                    self.bump();
                    TokenKind::DotDot
                } else {
                    TokenKind::Dot
                }
            }
            '@' => TokenKind::At,
            '+' => TokenKind::Plus,
            '-' => {
                if self.peek() == Some('>') {
                    self.bump();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::Eq
                } else {
                    TokenKind::Assign
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::Ne
                } else {
                    TokenKind::Not
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::Le
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::Ge
                } else {
                    TokenKind::Gt
                }
            }
            '&' if self.peek() == Some('&') => {
                self.bump();
                TokenKind::And
            }
            '|' if self.peek() == Some('|') => {
                self.bump();
                TokenKind::Or
            }
            other => {
                return Err(KengaError::at(
                    format!("unexpected character '{other}'"),
                    start,
                ));
            }
        };
        Ok(Token::new(kind, start))
    }

    fn ident_or_keyword(&mut self, start: Span) -> Token {
        let mut s = String::new();
        while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '_') {
            s.push(self.bump().unwrap());
        }
        let kind = match s.as_str() {
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "struct" => TokenKind::Struct,
            "import" => TokenKind::Import,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "ttl" => TokenKind::Ttl,
            "intrinsic" => TokenKind::Intrinsic,
            "i64" => TokenKind::TypeI64,
            "f64" => TokenKind::TypeF64,
            "bool" => TokenKind::TypeBool,
            "str" => TokenKind::TypeStr,
            "Tensor" => TokenKind::TypeTensor,
            "list" => TokenKind::TypeList,
            _ => TokenKind::Ident(s),
        };
        Token::new(kind, start)
    }

    fn number(&mut self, start: Span) -> Result<Token> {
        let mut raw = String::new();
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            raw.push(self.bump().unwrap());
        }

        // duration: 5s / 10ms / 2m
        if matches!(self.peek(), Some(c) if c.is_ascii_alphabetic()) {
            let mut unit = String::new();
            while matches!(self.peek(), Some(c) if c.is_ascii_alphabetic()) {
                unit.push(self.bump().unwrap());
            }
            let n: u64 = raw.parse().map_err(|_| {
                KengaError::at(format!("invalid duration number '{raw}'"), start.clone())
            })?;
            let ms = match unit.as_str() {
                "ms" => n,
                "s" => n.saturating_mul(1000),
                "m" => n.saturating_mul(60_000),
                other => {
                    return Err(KengaError::at(
                        format!("unknown duration unit '{other}' (use ms, s, m)"),
                        start,
                    ));
                }
            };
            return Ok(Token::new(TokenKind::DurationMs(ms), start));
        }

        if self.peek() == Some('.') && matches!(self.peek2(), Some(c) if c.is_ascii_digit()) {
            raw.push(self.bump().unwrap());
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                raw.push(self.bump().unwrap());
            }
            let v: f64 = raw
                .parse()
                .map_err(|_| KengaError::at(format!("invalid float '{raw}'"), start.clone()))?;
            return Ok(Token::new(TokenKind::Float(v), start));
        }

        let v: i64 = raw
            .parse()
            .map_err(|_| KengaError::at(format!("invalid integer '{raw}'"), start.clone()))?;
        Ok(Token::new(TokenKind::Int(v), start))
    }

    fn string(&mut self, start: Span) -> Result<Token> {
        self.bump(); // "
        let mut s = String::new();
        loop {
            match self.bump() {
                None => return Err(KengaError::at("unclosed string", start)),
                Some('"') => break,
                Some('\\') => match self.bump() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('\\') => s.push('\\'),
                    Some('"') => s.push('"'),
                    Some(c) => s.push(c),
                    None => return Err(KengaError::at("unclosed string escape", start)),
                },
                Some(c) => s.push(c),
            }
        }
        let _ = self.src; // keep field used for future source maps
        Ok(Token::new(TokenKind::Str(s), start))
    }
}
