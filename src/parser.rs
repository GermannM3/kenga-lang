use crate::ast::*;
use crate::error::{KengaError, Result};
use crate::error::Span;
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
            } else if self.check_ident("export") {
                // Module export directives are link-time metadata; the
                // freestanding C backend has no separate export table yet.
                self.bump();
                self.expect_ident()?;
                if self.check(&TokenKind::Semicolon) { self.bump(); }
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
        /* Portable desktop sources may write `import "x" as Module`.
           Imports are currently flattened by the driver, so retain the
           syntax and validate the alias while keeping the existing AST. */
        if self.check_ident("as") {
            self.bump();
            // Aliases may intentionally use names that are also built-in type
            // words (for example `Memory`), so consume the alias token here.
            if self.check(&TokenKind::Semicolon) || self.check(&TokenKind::Eof) {
                return Err(KengaError::at("expected import alias after 'as'", self.peek().span.clone()));
            }
            self.bump();
        }
        if self.check(&TokenKind::Semicolon) { self.bump(); }
        Ok(Import {
            path,
            span: tok.span,
        })
    }

    fn check_ident(&self, name: &str) -> bool {
        matches!(&self.peek().kind, TokenKind::Ident(s) if s == name)
    }

    fn parse_item(&mut self) -> Result<Item> {
        if self.check(&TokenKind::At) {
            return Ok(Item::Intrinsic(self.parse_intrinsic()?));
        }
        if self.check(&TokenKind::Struct) {
            return Ok(Item::Struct(self.parse_struct()?));
        }
        if self.check_ident("enum") {
            return Ok(Item::Enum(self.parse_enum()?));
        }
        if self.check_ident("union") {
            // Unions use the same field surface as structs for the portable
            // ABI; the native backend lowers them to a tagged payload later.
            self.bump();
            return Ok(Item::Struct(self.parse_struct_body()?));
        }
        if self.check(&TokenKind::Const) {
            return Ok(Item::Const(self.parse_const()?));
        }
        if self.check_ident("impl") {
            return Ok(Item::Impl(self.parse_impl()?));
        }
        if self.check(&TokenKind::On) {
            return Ok(Item::EventHandler(self.parse_on_handler()?));
        }
        Ok(Item::Function(self.parse_function()?))
    }

    fn parse_const(&mut self) -> Result<ConstDef> {
        let tok = self.bump();
        let name = self.expect_ident()?;
        self.expect(TokenKind::Assign, "expected '=' after const name")?;
        let value = self.parse_expr()?;
        /* Newer Kenga source permits declaration-per-line style. */
        if self.check(&TokenKind::Semicolon) { self.bump(); }
        Ok(ConstDef { name, value, span: tok.span })
    }

    fn parse_impl(&mut self) -> Result<ImplDef> {
        let tok = self.bump();
        let target = self.expect_ident()?;
        self.expect(TokenKind::LBrace, "expected '{' after impl target")?;
        let mut methods = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            methods.push(self.parse_function()?);
        }
        self.expect(TokenKind::RBrace, "expected '}' after impl")?;
        Ok(ImplDef { target, methods, span: tok.span })
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
        self.parse_struct_body_named(name, tok.span)
    }

    fn parse_struct_body(&mut self) -> Result<StructDef> {
        let tok = self.peek().span.clone();
        let name = self.expect_ident()?;
        self.parse_struct_body_named(name, tok)
    }

    fn parse_struct_body_named(&mut self, name: String, span: Span) -> Result<StructDef> {
        self.expect(TokenKind::LBrace, "expected '{' after struct name")?;
        let mut fields = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            let fname = self.expect_ident()?;
            self.expect(TokenKind::Colon, "expected ':' in struct field")?;
            let ty = self.parse_type()?;
            fields.push(Param { name: fname, ty });
            if self.check(&TokenKind::Comma) {
                self.bump();
            }
        }
        self.expect(TokenKind::RBrace, "expected '}' after struct fields")?;
        Ok(StructDef {
            name,
            fields,
            span,
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
            // Method receivers use Rust-like syntax in the desktop sources.
            // The current C ABI represents the receiver as an opaque handle;
            // preserving it here lets the native target complete parsing
            // while later backends can lower it to a real object pointer.
            if self.check(&TokenKind::Amp) {
                self.bump();
                if self.check_ident("mut") { self.bump(); }
                let name = self.expect_ident()?;
                if name != "self" {
                    return Err(KengaError::new("expected self receiver", None));
                }
                params.push(Param { name, ty: Type::I64 });
            } else {
            if self.check_ident("self") {
                let tok = self.bump();
                params.push(Param { name: "self".to_string(), ty: Type::I64 });
                if self.check(&TokenKind::Comma) { self.bump(); continue; }
                if self.check(&TokenKind::RParen) { break; }
                return Err(KengaError::at("expected ',' after self parameter", tok.span));
            }
            let name = self.expect_ident()?;
            self.expect(TokenKind::Colon, "expected ':' after param name")?;
            if self.check(&TokenKind::Amp) {
                self.bump();
                if self.check_ident("mut") { self.bump(); }
            }
            let ty = self.parse_type()?;
            params.push(Param { name, ty });
            }
            if self.check(&TokenKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }
        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type> {
        if self.check(&TokenKind::Amp) {
            self.bump();
            if self.check_ident("mut") { self.bump(); }
        }
        let t = self.bump();
        match t.kind {
            TokenKind::TypeI64 => Ok(Type::I64),
            TokenKind::TypeF64 => Ok(Type::F64),
            TokenKind::TypeBool => Ok(Type::Bool),
            TokenKind::TypeStr => Ok(Type::Str),
            TokenKind::TypeTensor => Ok(Type::Tensor),
            TokenKind::TypeList => Ok(Type::List),
            TokenKind::TypeMemory => Ok(Type::Memory),
            TokenKind::Fn => {
                if self.check(&TokenKind::LParen) {
                    self.bump();
                    while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) { self.bump(); }
                    self.expect(TokenKind::RParen, "expected ')' after function type")?;
                }
                Ok(Type::Named("fn".to_string()))
            }
            TokenKind::Ident(mut name) => {
                if (name == "array" || name == "Option" || name == "ptr") && self.check(&TokenKind::Lt) {
                    self.bump();
                    if name == "ptr" && self.check(&TokenKind::Fn) {
                        self.bump();
                        if self.check(&TokenKind::LParen) {
                            self.bump();
                            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) { self.bump(); }
                            self.expect(TokenKind::RParen, "expected ')' after function pointer")?;
                        }
                    } else {
                        let _ = self.parse_type()?;
                    }
                    if self.check(&TokenKind::Comma) {
                        self.bump();
                        let _ = self.bump(); // fixed capacity, represented as a list
                    }
                    self.expect(TokenKind::Gt, "expected '>' after array type")?;
                    return Ok(Type::List);
                }
                /* Accept portable module-qualified types such as
                   Renderer::Framebuffer and Renderer.Framebuffer. */
                loop {
                    if self.check(&TokenKind::Dot) {
                        self.bump();
                        let part = self.expect_ident()?;
                        name.push_str("::");
                        name.push_str(&part);
                    } else if self.check(&TokenKind::Colon)
                        && matches!(self.tokens.get(self.pos + 1).map(|t| &t.kind), Some(TokenKind::Colon)) {
                        self.bump();
                        self.bump();
                        let part = self.expect_ident()?;
                        name.push_str("::");
                        name.push_str(&part);
                    } else {
                        break;
                    }
                }
                Ok(Type::Named(name))
            }
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
        if self.check_ident("loop") {
            let tok = self.bump();
            let body = self.parse_block()?;
            return Ok(Stmt::While {
                cond: Expr::Bool(true, tok.span.clone()),
                body,
                span: tok.span,
            });
        }
        if self.check_ident("asm") {
            let tok = self.bump();
            self.expect(TokenKind::LBrace, "expected '{' after asm")?;
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) { self.bump(); }
            self.expect(TokenKind::RBrace, "expected '}' after asm")?;
            return Ok(Stmt::Expr { expr: Expr::Int(0, tok.span.clone()), span: tok.span });
        }
        match self.peek_kind() {
            TokenKind::Let | TokenKind::Var => self.parse_let(),
            TokenKind::Return => self.parse_return(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Match => self.parse_match(),
            TokenKind::Break => {
                let t = self.bump();
                self.optional_semicolon();
                Ok(Stmt::Break(t.span))
            }
            TokenKind::Continue => {
                let t = self.bump();
                self.optional_semicolon();
                Ok(Stmt::Continue(t.span))
            }
            TokenKind::Ident(_) => self.parse_assign_or_expr_stmt(),
            _ => {
                let expr = self.parse_expr()?;
                let span = expr.span();
                self.optional_semicolon();
                Ok(Stmt::Expr { expr, span })
            }
        }
    }

    fn parse_assign_or_expr_stmt(&mut self) -> Result<Stmt> {
        let save = self.pos;
        let target_expr = self.parse_postfix_primary()?;
        let compound = match self.peek_kind() {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            TokenKind::Star => Some(BinaryOp::Mul),
            TokenKind::Slash => Some(BinaryOp::Div),
            _ => None,
        };
        if self.check(&TokenKind::Assign) || compound.is_some() {
            let span = self.bump().span;
            if let Some(op) = compound {
                self.expect(TokenKind::Assign, "expected '=' after compound assignment")?;
                let value = self.parse_expr()?;
                let target = match target_expr.clone() {
                    Expr::Ident(name, _) => AssignTarget::Name(name),
                    Expr::Index { target, index, .. } => AssignTarget::Index { target: *target, index: *index },
                    Expr::Field { target, field, .. } => AssignTarget::Field { target: *target, field },
                    _ => return Err(KengaError::at("invalid assignment target", span)),
                };
                let lhs = target_expr;
                return Ok(Stmt::Assign { target, value: Expr::Binary { op, left: Box::new(lhs), right: Box::new(value), span: span.clone() }, span });
            }
            let value = self.parse_expr()?;
            self.optional_semicolon();
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
        self.optional_semicolon();
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
        self.optional_semicolon();
        Ok(Stmt::Let {
            name,
            ty,
            ttl_ms,
            value,
            span: let_tok.span,
        })
    }

    fn parse_match(&mut self) -> Result<Stmt> {
        let tok = self.bump();
        let value = if let TokenKind::Ident(name) = self.peek().kind.clone() {
            let t = self.bump();
            Expr::Ident(name, t.span)
        } else { self.parse_expr()? };
        self.expect(TokenKind::LBrace, "expected '{' after match value")?;
        let mut arms = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            let mut path = Vec::new();
            let mut bindings = Vec::new();
            if self.check_ident("_") {
                self.bump();
            } else if let TokenKind::Int(value) = self.peek().kind.clone() {
                self.bump();
                self.expect(TokenKind::FatArrow, "expected '=>' after literal pattern")?;
                let body = if self.check(&TokenKind::LBrace) {
                    self.parse_block()?
                } else {
                    Block { stmts: vec![self.parse_stmt()?] }
                };
                arms.push(MatchArm { pattern: MatchPattern::Literal(value), body });
                if self.check(&TokenKind::Comma) { self.bump(); }
                continue;
            } else {
                path.push(self.expect_ident()?);
                while self.check(&TokenKind::Colon) {
                    self.bump();
                    self.expect(TokenKind::Colon, "expected '::' in match variant")?;
                    path.push(self.expect_ident()?);
                }
                if self.check(&TokenKind::LBrace) {
                    self.bump();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
                        bindings.push(self.expect_ident()?);
                        if self.check(&TokenKind::Comma) { self.bump(); }
                    }
                    self.expect(TokenKind::RBrace, "expected '}' after match bindings")?;
                }
            }
            self.expect(TokenKind::FatArrow, "expected '=>' after match pattern")?;
            let body = if self.check(&TokenKind::LBrace) {
                self.parse_block()?
            } else {
                Block { stmts: vec![self.parse_stmt()?] }
            };
            let pattern = if path.is_empty() { MatchPattern::Wildcard } else { MatchPattern::Variant { path, bindings } };
            arms.push(MatchArm { pattern, body });
            if self.check(&TokenKind::Comma) { self.bump(); }
        }
        self.expect(TokenKind::RBrace, "expected '}' after match")?;
        Ok(Stmt::Match { value, arms, span: tok.span })
    }

    fn parse_enum(&mut self) -> Result<EnumDef> {
        let tok = self.bump();
        let name = self.expect_ident()?;
        self.expect(TokenKind::LBrace, "expected '{' after enum name")?;
        let mut variants = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            let variant_name = self.expect_ident()?;
            let mut fields = Vec::new();
            if self.check(&TokenKind::LBrace) {
                self.bump();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
                    let field_name = self.expect_ident()?;
                    self.expect(TokenKind::Colon, "expected ':' in enum payload")?;
                    let field_ty = self.parse_type()?;
                    fields.push(Param { name: field_name, ty: field_ty });
                    if self.check(&TokenKind::Comma) { self.bump(); }
                }
                self.expect(TokenKind::RBrace, "expected '}' after enum payload")?;
            }
            variants.push(EnumVariant { name: variant_name, fields });
            if self.check(&TokenKind::Comma) { self.bump(); }
        }
        self.expect(TokenKind::RBrace, "expected '}' after enum")?;
        Ok(EnumDef { name, variants, span: tok.span })
    }

    fn optional_semicolon(&mut self) {
        if self.check(&TokenKind::Semicolon) { self.bump(); }
    }

    fn parse_return(&mut self) -> Result<Stmt> {
        let tok = self.bump();
        let value = if self.check(&TokenKind::Semicolon) || self.check(&TokenKind::RBrace) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.optional_semicolon();
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
        let cond = if self.check(&TokenKind::Let) {
            // Optional-pattern form used by the desktop event loop:
            // `while let Some(event) = poll()`. The current ABI represents
            // option-like polling as its scalar condition; consume the
            // pattern and retain the polling expression.
            self.bump();
            let _ = self.expect_ident()?; // Some
            self.expect(TokenKind::LParen, "expected '(' after Some")?;
            let _binding = self.expect_ident()?;
            self.expect(TokenKind::RParen, "expected ')' after Some binding")?;
            self.expect(TokenKind::Assign, "expected '=' in while let")?;
            self.parse_expr()?
        } else { self.parse_expr()? };
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
        let expr = self.parse_range()?;
        // The desktop DSL permits explicit scalar casts (`value as float`).
        // The current tagged C backend already performs the required numeric
        // coercion, so consume the annotation while preserving the expression
        // tree until typed cast nodes are introduced in the native ABI.
        while self.check_ident("as") {
            self.bump();
            let _ = self.expect_ident()?;
        }
        Ok(expr)
    }

    fn parse_range(&mut self) -> Result<Expr> {
        let start = self.parse_or()?;
        if self.check(&TokenKind::DotDot) {
            let span = self.bump().span;
            let end = self.parse_or()?;
            let step = if self.check_ident("step") { self.bump(); Some(Box::new(self.parse_or()?)) } else { None };
            return Ok(Expr::Range {
                start: Box::new(start),
                end: Box::new(end),
                step,
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
        let mut left = self.parse_bitor()?;
        while self.check(&TokenKind::And) {
            let span = self.bump().span;
            let right = self.parse_bitor()?;
            left = Expr::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_bitor(&mut self) -> Result<Expr> {
        let mut left = self.parse_bitxor()?;
        while self.check(&TokenKind::Pipe) {
            let span = self.bump().span;
            let right = self.parse_bitxor()?;
            left = Expr::Binary {
                op: BinaryOp::BitOr,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_bitxor(&mut self) -> Result<Expr> {
        let mut left = self.parse_bitand()?;
        while self.check(&TokenKind::Caret) {
            let span = self.bump().span;
            let right = self.parse_bitand()?;
            left = Expr::Binary {
                op: BinaryOp::BitXor,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_bitand(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality()?;
        while self.check(&TokenKind::Amp) {
            let span = self.bump().span;
            let right = self.parse_equality()?;
            left = Expr::Binary {
                op: BinaryOp::BitAnd,
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
        let mut left = self.parse_shift()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::Le => BinaryOp::Le,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::Ge => BinaryOp::Ge,
                _ => break,
            };
            let span = self.bump().span;
            let right = self.parse_shift()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_shift(&mut self) -> Result<Expr> {
        let mut left = self.parse_term()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Shl => BinaryOp::Shl,
                TokenKind::Shr => BinaryOp::Shr,
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
            TokenKind::Tilde => {
                let span = self.bump().span;
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::BitNot,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::Amp => {
                self.bump();
                if self.check_ident("mut") { self.bump(); }
                // References are ABI-transparent in the current freestanding
                // backend; retain the referenced expression for lowering.
                self.parse_unary()
            }
            _ => {
                let expr = self.parse_postfix_primary()?;
                // Scalar casts bind to the value immediately before them,
                // so `value as float * factor` keeps the multiplication.
                if self.check_ident("as") {
                    self.bump();
                    if self.check(&TokenKind::Fn) {
                        self.bump();
                        if self.check(&TokenKind::LParen) {
                            self.bump();
                            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) { self.bump(); }
                            self.expect(TokenKind::RParen, "expected ')' after function cast")?;
                        }
                    } else {
                        let _ = self.expect_ident()?;
                    }
                    if self.check(&TokenKind::Lt) {
                        while !self.check(&TokenKind::Gt) && !self.check(&TokenKind::Eof) { self.bump(); }
                        self.expect(TokenKind::Gt, "expected '>' after cast type")?;
                    }
                }
                Ok(expr)
            }
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
            } else if self.check(&TokenKind::Colon) &&
                      matches!(self.tokens.get(self.pos + 1).map(|t| &t.kind), Some(TokenKind::Colon)) {
                let span = self.bump().span;
                self.bump();
                let method = self.expect_ident()?;
                // Generic call arguments are compile-time type metadata for
                // the current C ABI, so consume the type list before parsing
                // the runtime argument list (e.g. alloc_array<float>(n)).
                if self.check(&TokenKind::Lt) {
                    let mut depth = 0usize;
                    while !self.check(&TokenKind::Eof) {
                        if self.check(&TokenKind::Lt) { depth += 1; }
                        if self.check(&TokenKind::Gt) {
                            depth -= 1;
                            self.bump();
                            if depth == 0 { break; }
                            continue;
                        }
                        self.bump();
                    }
                }
                if self.check(&TokenKind::LParen) {
                    self.bump();
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if self.check(&TokenKind::Comma) { self.bump(); continue; }
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen, "expected ')' after static call")?;
                    expr = Expr::Call { callee: method, args, span };
                } else if self.check(&TokenKind::LBrace)
                    && matches!(self.tokens.get(self.pos + 2).map(|t| &t.kind), Some(TokenKind::Colon)) {
                    self.bump();
                    let mut fields = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
                        let field = self.expect_ident()?;
                        self.expect(TokenKind::Colon, "expected ':' in variant payload")?;
                        fields.push((field, self.parse_expr()?));
                        if self.check(&TokenKind::Comma) { self.bump(); }
                    }
                    self.expect(TokenKind::RBrace, "expected '}' after variant payload")?;
                    let mut path = Vec::new();
                    if let Expr::Ident(root, _) = expr { path.push(root); }
                    path.push(method);
                    expr = Expr::VariantLit { path, fields, span };
                } else {
                    expr = Expr::Field { target: Box::new(expr), field: method, span };
                }
            } else if self.check(&TokenKind::Dot) {
                let span = self.bump().span;
                let field = self.expect_ident()?;
                if self.check(&TokenKind::LParen) {
                    self.bump();
                    let mut args = Vec::new();
                    let receiver = expr.clone();
                    if !self.check(&TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if self.check(&TokenKind::Comma) { self.bump(); continue; }
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen, "expected ')' after method call")?;
                    if field == "len" { args.insert(0, receiver); }
                    expr = Expr::Call { callee: field, args, span };
                } else {
                    expr = Expr::Field { target: Box::new(expr), field, span };
                }
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
            TokenKind::TypeMemory => Ok(Expr::Ident("Memory".to_string(), tok.span)),
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
            Item::Const(c) => {
                out.push_str(&format!("const {} = ...;\n\n", c.name));
            }
            Item::Enum(e) => {
                out.push_str(&format!("enum {} {{ {} }}\n\n", e.name, e.variants.iter().map(|v| v.name.clone()).collect::<Vec<_>>().join(", ")));
            }
            Item::Impl(i) => {
                out.push_str(&format!("impl {} {{ {} methods }}\n\n", i.target, i.methods.len()));
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
