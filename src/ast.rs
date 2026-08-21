#![allow(dead_code)]

use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    I64,
    F64,
    Bool,
    Str,
    Tensor,
    List,
    Memory,
    Named(String),
    Void,
}

impl Type {
    pub fn name(&self) -> String {
        match self {
            Type::I64 => "i64".into(),
            Type::F64 => "f64".into(),
            Type::Bool => "bool".into(),
            Type::Str => "str".into(),
            Type::Tensor => "Tensor".into(),
            Type::List => "list".into(),
            Type::Memory => "Memory".into(),
            Type::Named(n) => n.clone(),
            Type::Void => "void".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub imports: Vec<Import>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Intrinsic(IntrinsicDecl),
    Struct(StructDef),
    Const(ConstDef),
    Impl(ImplDef),
    /// `on "event"(args) { ... }` — native agent handler
    EventHandler(EventHandler),
}

#[derive(Debug, Clone)]
pub struct ConstDef {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImplDef {
    pub target: String,
    pub methods: Vec<Function>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EventHandler {
    pub event: String,
    pub params: Vec<Param>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Param>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IntrinsicDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Type,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum AssignTarget {
    Name(String),
    Index { target: Expr, index: Expr },
    Field { target: Expr, field: String },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        ty: Option<Type>,
        ttl_ms: Option<u64>,
        value: Expr,
        span: Span,
    },
    Assign {
        target: AssignTarget,
        value: Expr,
        span: Span,
    },
    Expr {
        expr: Expr,
        span: Span,
    },
    Return {
        value: Option<Expr>,
        span: Span,
    },
    If {
        cond: Expr,
        then_block: Block,
        else_block: Option<Block>,
        span: Span,
    },
    While {
        cond: Expr,
        body: Block,
        span: Span,
    },
    For {
        var: String,
        iter: Expr,
        body: Block,
        span: Span,
    },
    Break(Span),
    Continue(Span),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64, Span),
    Float(f64, Span),
    Bool(bool, Span),
    Str(String, Span),
    Ident(String, Span),
    List(Vec<Expr>, Span),
    StructLit {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        span: Span,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    Field {
        target: Box<Expr>,
        field: String,
        span: Span,
    },
    Call {
        callee: String,
        args: Vec<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Int(_, s)
            | Expr::Float(_, s)
            | Expr::Bool(_, s)
            | Expr::Str(_, s)
            | Expr::Ident(_, s)
            | Expr::List(_, s) => s.clone(),
            Expr::StructLit { span, .. }
            | Expr::Range { span, .. }
            | Expr::Index { span, .. }
            | Expr::Field { span, .. }
            | Expr::Call { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Binary { span, .. } => span.clone(),
        }
    }
}
