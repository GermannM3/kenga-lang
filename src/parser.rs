use crate::ast::*;
use crate::error::{KengaError, Result};
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(mut self) -> Result<Program> {
        let mut imports = Vec::new();
        let mut items = Vec::new();
        while !self.check(&TokenKind::Eof) {
            if self.check(&TokenKind::Import) {
                imports.push(self.parse_import()?);
            } else {
                items.push(self.parse_item()?);
            }
        }
        Ok(Program { imports, items })
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind)
    }

    fn bump(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, kind: TokenKind, msg: &str) -> Result<Token> {
        if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(&kind) {
            Ok(self.bump())
        } else {
            Err(KengaError::at(msg, self.peek().span.clone()))
        }
    }

    fn parse_import(&mut self) -> Result<Import> {
        let tok = self.bump();
        let path_tok = self.bump();
        let path = match path_tok.kind {
            TokenKind::Str(s) => s,
            _ => {
                return Err(KengaError::at(
                    "expected string path after import",
                    path_tok.span,
                ))
            }
        };
        self.expect(TokenKind::Semicolon, "expected ';' after import")?;
        Ok(Import {
            path,
            span: tok.span,
        })
    }

    fn parse_item(&mut self) -> Result<Item> {
        if self.check(&TokenKind::At) {
            return Ok(Item::Intrinsic(self.parse_intrinsic()?));
        }
        if self.check(&TokenKind::Struct) {
            return Ok(Item::Struct(self.parse_struct()?));
        }
        if self.check(&TokenKind::On) {
            return Ok(Item::EventHandler(self.parse_on_handler()?));
        }
        Ok(Item::Function(self.parse_function()?))
    }

    fn parse_on_handler(&mut self) -> Result<EventHandler> {
        let tok = self.bump(); // on
        let ev = self.bump();
        let event = match ev.kind {
            TokenKind::Str(s) => s,
            _ => {
                return Err(KengaError::at(
                    "expected event name string after on",
                    ev.span,
                ))
            }
        };
        self.expect(TokenKind::LParen, "expected '(' after event name")?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen, "expected ')'")?;
        let body = self.parse_block()?;
        Ok(EventHandler {
            event,
            params,
            body,
            span: tok.span,
        })
    }

    fn parse_struct(&mut self) -> Result<StructDef> {
        let tok = self.bump();
        let name = self.expect_ident()?;
        self.expect(TokenKind::LBrace, "expected '{' after struct name")?;
        let mut fields = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            let fname = self.expect_ident()?;
            self.expect(TokenKind::Colon, "expected ':' in struct field")?;
            let ty = self.parse_type()?;
            fields.push(Param { name: fname, ty });
            if self.check(&TokenKind::Comma) {
                self.bump();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RBrace, "expected '}' after struct fields")?;
        Ok(StructDef {
            name,
            fields,
            span: tok.span,
        })
    }

    fn parse_intrinsic(&mut self) -> Result<IntrinsicDecl> {
        let at = self.expect(TokenKind::At, "expected '@'")?;
        self.expect(TokenKind::Intrinsic, "expected 'intrinsic'")?;
        self.expect(TokenKind::Fn, "expected 'fn' after @intrinsic")?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::LParen, "expected '('")?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen, "expected ')'")?;
        let ret = if self.check(&TokenKind::Arrow) {
            self.bump();
            self.parse_type()?
        } else {
            Type::Void
        };
        self.expect(TokenKind::Semicolon, "expected ';' after intrinsic")?;
        Ok(IntrinsicDecl {
            name,
            params,
            ret,
            span: at.span,
        })
    }

    fn parse_function(&mut self) -> Result<Function> {
        let fn_tok = self.expect(TokenKind::Fn, "expected 'fn'")?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::LParen, "expected '('")?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen, "expected ')'")?;
        let ret = if self.check(&TokenKind::Arrow) {
            self.bump();
            self.parse_type()?
        } else {
            Type::Void
        };
        let body = self.parse_block()?;
        Ok(Function {
            name,
            params,
            ret,
            body,
            span: fn_tok.span,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();
        if self.check(&TokenKind::RParen) {
            return Ok(params);
        }
        loop {
            let name = self.expect_ident()?;
            self.expect(TokenKind::Colon, "expected ':' after param name")?;
            let ty = self.parse_type()?;
            params.push(Param { name, ty });
            if self.check(&TokenKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }
        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type> {
        let t = self.bump();
        match t.kind {
            TokenKind::TypeI64 => Ok(Type::I64),
            TokenKind::TypeF64 => Ok(Type::F64),
            TokenKind::TypeBool => Ok(Type::Bool),
            TokenKind::TypeStr => Ok(Type::Str),
            TokenKind::TypeTensor => Ok(Type::Tensor),
            TokenKind::TypeList => Ok(Type::List),
            TokenKind::TypeMemory => Ok(Type::Memory),
            TokenKind::Ident(name) => Ok(Type::Named(name)),
            _ => Err(KengaError::at("expected type name", t.span)),
        }
    }

    fn parse_block(&mut self) -> Result<Block> {
        self.expect(TokenKind::LBrace, "expected '{'")?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RBrace, "expected '}'")?;
        Ok(Block { stmts })
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        match self.peek_kind() {
            TokenKind::Let => self.parse_let(),
            TokenKind::Return => self.parse_return(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Break => {
                let t = self.bump();
                self.expect(TokenKind::Semicolon, "expected ';' after break")?;
                Ok(Stmt::Break(t.span))
            }
            TokenKind::Continue => {
                let t = self.bump();
                self.expect(TokenKind::Semicolon, "expected ';' after continue")?;
                Ok(Stmt::Continue(t.span))
            }
            TokenKind::Ident(_) => self.parse_assign_or_expr_stmt(),
            _ => {
                let expr = self.parse_expr()?;
                let span = expr.span();
                self.expect(TokenKind::Semicolon, "expected ';'")?;
                Ok(Stmt::Expr { expr, span })
            }
        }
    }

    fn parse_assign_or_expr_stmt(&mut self) -> Result<Stmt> {
        let save = self.pos;
        let target_expr = self.parse_postfix_primary()?;
        if self.check(&TokenKind::Assign) {
            let span = self.bump().span;
            let value = self.parse_expr()?;
            self.expect(TokenKind::Semicolon, "expected ';'")?;
            let target = match target_expr {
                Expr::Ident(name, _) => AssignTarget::Name(name),
                Expr::Index { target, index, .. } => AssignTarget::Index {
                    target: *target,
                    index: *index,
                },
                Expr::Field { target, field, .. } => AssignTarget::Field {
                    target: *target,
                    field,
                },
                _ => {
                    return Err(KengaError::at(
                        "invalid assignment target",
                        span,
                    ));
                }
            };
            return Ok(Stmt::Assign {
                target,
                value,
                span,
            });
        }
        self.pos = save;
        let expr = self.parse_expr()?;
        let span = expr.span();
        self.expect(TokenKind::Semicolon, "expected ';'")?;
        Ok(Stmt::Expr { expr, span })
    }

    fn parse_let(&mut self) -> Result<Stmt> {
        let let_tok = self.bump();
        let name = self.expect_ident()?;
        let mut ty = None;
        let mut ttl_ms = None;
        if self.check(&TokenKind::Colon) {
            self.bump();
            ty = Some(self.parse_type()?);
            if self.check(&TokenKind::Ttl) {
                self.bump();
                let d = self.bump();
                match d.kind {
                    TokenKind::DurationMs(ms) => ttl_ms = Some(ms),
                    _ => {
                        return Err(KengaError::at(
                            "expected duration after ttl (e.g. 5s, 100ms)",
                            d.span,
                        ));
                    }
                }
            }
        }
        self.expect(TokenKind::Assign, "expected '=' in let")?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Semicolon, "expected ';'")?;
        Ok(Stmt::Let {
            name,
            ty,
            ttl_ms,
            value,
            span: let_tok.span,
        })
    }

    fn parse_return(&mut self) -> Result<Stmt> {
        let tok = self.bump();
        let value = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect(TokenKind::Semicolon, "expected ';'")?;
        Ok(Stmt::Return {
            value,
            span: tok.span,
        })
    }

    fn parse_if(&mut self) -> Result<Stmt> {
        let tok = self.bump();
        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let else_block = if self.check(&TokenKind::Else) {
            self.bump();
            if self.check(&TokenKind::If) {
                // else if … → nested If as a one-statement block
                let nested = self.parse_if()?;
                Some(Block {
                    stmts: vec![nested],
                })
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        Ok(Stmt::If {
            cond,
            then_block,
            else_block,
            span: tok.span,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt> {
        let tok = self.bump();
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While {
            cond,
            body,
            span: tok.span,
        })
    }

    fn parse_for(&mut self) -> Result<Stmt> {
        let tok = self.bump();
        let var = self.expect_ident()?;
        self.expect(TokenKind::In, "expected 'in' after for variable")?;
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::For {
            var,
            iter,
            body,
            span: tok.span,
        })
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_range()
    }

    fn parse_range(&mut self) -> Result<Expr> {
        let start = self.parse_or()?;
        if self.check(&TokenKind::DotDot) {
            let span = self.bump().span;
            let end = self.parse_or()?;
            return Ok(Expr::Range {
                start: Box::new(start),
                end: Box::new(end),
                span,
            });
        }
        Ok(start)
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        while self.check(&TokenKind::Or) {
            let span = self.bump().span;
            let right = self.parse_and()?;
            left = Expr::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality()?;
        while self.check(&TokenKind::And) {
            let span = self.bump().span;
            let right = self.parse_equality()?;
            left = Expr::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Eq => BinaryOp::Eq,
                TokenKind::Ne => BinaryOp::Ne,
                _ => break,
            };
            let span = self.bump().span;
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_term()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::Le => BinaryOp::Le,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::Ge => BinaryOp::Ge,
                _ => break,
            };
            let span = self.bump().span;
            let right = self.parse_term()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr> {
        let mut left = self.parse_factor()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };
            let span = self.bump().span;
            let right = self.parse_factor()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Rem,
                _ => break,
            };
            let span = self.bump().span;
            let right = self.parse_unary()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        match self.peek_kind() {
            TokenKind::Minus => {
                let span = self.bump().span;
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::Not => {
                let span = self.bump().span;
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                    span,
                })
            }
            _ => self.parse_postfix_primary(),
        }
    }

    fn parse_postfix_primary(&mut self) -> Result<Expr> {
        let mut expr = self.parse_atom()?;
        loop {
            if self.check(&TokenKind::LBracket) {
                let span = self.bump().span;
                let index = self.parse_expr()?;
                self.expect(TokenKind::RBracket, "expected ']'")?;
                expr = Expr::Index {
                    target: Box::new(expr),
                    index: Box::new(index),
                    span,
                };
            } else if self.check(&TokenKind::Dot) {
                let span = self.bump().span;
                let field = self.expect_ident()?;
                expr = Expr::Field {
                    target: Box::new(expr),
                    field,
                    span,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_atom(&mut self) -> Result<Expr> {
        let tok = self.bump();
        match tok.kind {
            TokenKind::Int(n) => Ok(Expr::Int(n, tok.span)),
            TokenKind::Float(n) => Ok(Expr::Float(n, tok.span)),
            TokenKind::True => Ok(Expr::Bool(true, tok.span)),
            TokenKind::False => Ok(Expr::Bool(false, tok.span)),
            TokenKind::Str(s) => Ok(Expr::Str(s, tok.span)),
            TokenKind::LBracket => {
                let mut elems = Vec::new();
                if !self.check(&TokenKind::RBracket) {
                    loop {
                        elems.push(self.parse_expr()?);
                        if self.check(&TokenKind::Comma) {
                            self.bump();
                            continue;
                        }
                        break;
                    }
                }
                self.expect(TokenKind::RBracket, "expected ']'")?;
                Ok(Expr::List(elems, tok.span))
            }
            TokenKind::Ident(name) => {
                if self.check(&TokenKind::LParen) {
                    self.bump();
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if self.check(&TokenKind::Comma) {
                                self.bump();
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    Ok(Expr::Call {
                        callee: name,
                        args,
                        span: tok.span,
                    })
                } else if self.looks_like_struct_lit() {
                    self.bump(); // {
                    let mut fields = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
                        let fname = self.expect_ident()?;
                        self.expect(TokenKind::Colon, "expected ':' in struct literal")?;
                        let val = self.parse_expr()?;
                        fields.push((fname, val));
                        if self.check(&TokenKind::Comma) {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    self.expect(TokenKind::RBrace, "expected '}' in struct literal")?;
                    Ok(Expr::StructLit {
                        name,
                        fields,
                        span: tok.span,
                    })
                } else {
                    Ok(Expr::Ident(name, tok.span))
                }
            }
            TokenKind::LParen => {
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen, "expected ')'")?;
                Ok(expr)
            }
            _ => Err(KengaError::at("expected expression", tok.span)),
        }
    }

    fn expect_ident(&mut self) -> Result<String> {
        let t = self.bump();
        match t.kind {
            TokenKind::Ident(s) => Ok(s),
            _ => Err(KengaError::at("expected identifier", t.span)),
        }
    }

    /// Distinguish `Point { x: 1 }` from `for x in xs {`
    fn looks_like_struct_lit(&self) -> bool {
        if !self.check(&TokenKind::LBrace) {
            return false;
        }
        let a = self.tokens.get(self.pos + 1).map(|t| &t.kind);
        match a {
            Some(TokenKind::RBrace) => true,
            Some(TokenKind::Ident(_)) => {
                matches!(
                    self.tokens.get(self.pos + 2).map(|t| &t.kind),
                    Some(TokenKind::Colon)
                )
            }
            _ => false,
        }
    }
}

pub fn dump_program(program: &Program) -> String {
    let mut out = String::new();
    for imp in &program.imports {
        out.push_str(&format!("import \"{}\";\n", imp.path));
    }
    if !program.imports.is_empty() {
        out.push('\n');
    }
    for item in &program.items {
        match item {
            Item::Intrinsic(i) => {
                out.push_str(&format!(
                    "@intrinsic fn {}({}) -> {}\n",
                    i.name,
                    i.params
                        .iter()
                        .map(|p| format!("{}: {}", p.name, p.ty.name()))
                        .collect::<Vec<_>>()
                        .join(", "),
                    i.ret.name()
                ));
            }
            Item::Struct(s) => {
                out.push_str(&format!("struct {} {{ ... }}\n\n", s.name));
            }
            Item::EventHandler(h) => {
                out.push_str(&format!(
                    "on \"{}\"(...) {{ {} stmts }}\n\n",
                    h.event,
                    h.body.stmts.len()
                ));
            }
            Item::Function(f) => {
                out.push_str(&format!(
                    "fn {}({}) -> {} {{ {} stmts }}\n\n",
                    f.name,
                    f.params
                        .iter()
                        .map(|p| format!("{}: {}", p.name, p.ty.name()))
                        .collect::<Vec<_>>()
                        .join(", "),
                    f.ret.name(),
                    f.body.stmts.len()
                ));
            }
        }
    }
    out
}
