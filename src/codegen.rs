//! C99 backend: i64, lists, structs, for/while, imports (via merged Program).

use std::collections::HashMap;

use crate::ast::{
    AssignTarget, BinaryOp, Expr, Function, Item, Program, Stmt, Type, UnaryOp,
};
use crate::error::{KengaError, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
enum CTy {
    I64,
    List,
    Str,
    Void,
    Struct(String),
}

struct StructInfo {
    fields: Vec<(String, CTy)>,
}

struct EmitCtx<'a> {
    vars: HashMap<String, CTy>,
    structs: &'a HashMap<String, StructInfo>,
    sigs: &'a HashMap<String, (CTy, Vec<CTy>)>,
    tmp: usize,
}

impl<'a> EmitCtx<'a> {
    fn fresh(&mut self, prefix: &str) -> String {
        let n = self.tmp;
        self.tmp += 1;
        format!("__{prefix}_{n}")
    }
}

pub fn emit_c(program: &Program) -> Result<String> {
    let mut structs: HashMap<String, StructInfo> = HashMap::new();
    for item in &program.items {
        if let Item::Struct(s) = item {
            let fields = s
                .fields
                .iter()
                .map(|p| Ok((p.name.clone(), map_type(&p.ty, &structs)?)))
                .collect::<Result<Vec<_>>>()?;
            for (_, ty) in &fields {
                if matches!(ty, CTy::Struct(_)) {
                    return Err(KengaError::at(
                        "emit-c: nested structs not supported yet",
                        s.span.clone(),
                    ));
                }
            }
            structs.insert(s.name.clone(), StructInfo { fields });
        }
    }

    let mut sigs: HashMap<String, (CTy, Vec<CTy>)> = HashMap::new();
    let mut has_main = false;
    for item in &program.items {
        if let Item::Function(f) = item {
            let ret = map_type(&f.ret, &structs)?;
            let params = f
                .params
                .iter()
                .map(|p| map_type(&p.ty, &structs))
                .collect::<Result<Vec<_>>>()?;
            sigs.insert(f.name.clone(), (ret, params));
            if f.name == "main" {
                has_main = true;
            }
        }
    }
    if !has_main {
        return Err(KengaError::new("emit-c requires fn main()", None));
    }

    let mut body = String::new();
    let mut names: Vec<_> = structs.keys().cloned().collect();
    names.sort();
    for name in names {
        let info = &structs[&name];
        body.push_str("typedef struct {\n");
        for (fname, fty) in &info.fields {
            body.push_str(&format!("  {} {};\n", c_type(fty), c_ident(fname)));
        }
        body.push_str(&format!("}} {};\n\n", struct_c_name(&name)));
    }

    for item in &program.items {
        if let Item::Function(f) = item {
            body.push_str(&emit_function(f, &structs, &sigs)?);
            body.push('\n');
        }
    }

    let mut out = String::new();
    out.push_str(RUNTIME);
    out.push_str(&body);
    Ok(out)
}

fn map_type(t: &Type, structs: &HashMap<String, StructInfo>) -> Result<CTy> {
    Ok(match t {
        Type::Void => CTy::Void,
        Type::List => CTy::List,
        Type::Str => CTy::Str,
        Type::I64 | Type::F64 | Type::Bool | Type::Tensor | Type::Memory => CTy::I64,
        Type::Named(n) => {
            if structs.contains_key(n) {
                CTy::Struct(n.clone())
            } else {
                CTy::I64
            }
        }
    })
}

fn c_type(t: &CTy) -> String {
    match t {
        CTy::I64 => "int64_t".into(),
        CTy::List => "KList".into(),
        CTy::Str => "const char*".into(),
        CTy::Void => "void".into(),
        CTy::Struct(n) => struct_c_name(n),
    }
}

fn struct_c_name(name: &str) -> String {
    format!("K_{name}")
}

fn c_ident(name: &str) -> String {
    if name == "main" {
        "main".into()
    } else {
        format!("k_{name}")
    }
}

fn escape_c(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn indent_pre(pre: &str, indent: usize) -> String {
    if pre.is_empty() {
        return String::new();
    }
    let pad = "  ".repeat(indent);
    pre.lines().map(|l| format!("{pad}{l}\n")).collect()
}

fn emit_function(
    f: &Function,
    structs: &HashMap<String, StructInfo>,
    sigs: &HashMap<String, (CTy, Vec<CTy>)>,
) -> Result<String> {
    let (ret, _) = sigs
        .get(&f.name)
        .cloned()
        .ok_or_else(|| KengaError::at("internal: missing sig", f.span.clone()))?;
    let mut ctx = EmitCtx {
        vars: HashMap::new(),
        structs,
        sigs,
        tmp: 0,
    };
    let mut s = format!("{} {}(", c_type(&ret), c_ident(&f.name));
    if f.params.is_empty() {
        s.push_str("void");
    } else {
        for (i, p) in f.params.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            let ty = map_type(&p.ty, structs)?;
            ctx.vars.insert(p.name.clone(), ty.clone());
            s.push_str(&format!("{} {}", c_type(&ty), c_ident(&p.name)));
        }
    }
    s.push_str(") {\n");
    for stmt in &f.body.stmts {
        s.push_str(&emit_stmt(stmt, 1, &mut ctx)?);
    }
    if ret == CTy::I64
        && !f
            .body
            .stmts
            .iter()
            .any(|st| matches!(st, Stmt::Return { .. }))
    {
        s.push_str("  return 0;\n");
    }
    s.push_str("}\n");
    Ok(s)
}

fn emit_stmt(stmt: &Stmt, indent: usize, ctx: &mut EmitCtx<'_>) -> Result<String> {
    let pad = "  ".repeat(indent);
    match stmt {
        Stmt::Let {
            name, value, ty, ..
        } => {
            let inferred = infer_expr(value, ctx)?;
            let cty = if let Some(t) = ty {
                map_type(t, ctx.structs)?
            } else {
                inferred
            };
            ctx.vars.insert(name.clone(), cty.clone());
            let (pre, expr) = emit_expr(value, ctx)?;
            Ok(format!(
                "{}{pad}{} {} = {};\n",
                indent_pre(&pre, indent),
                c_type(&cty),
                c_ident(name),
                expr
            ))
        }
        Stmt::Assign { target, value, span } => match target {
            AssignTarget::Name(name) => {
                let (pre, expr) = emit_expr(value, ctx)?;
                Ok(format!(
                    "{}{pad}{} = {};\n",
                    indent_pre(&pre, indent),
                    c_ident(name),
                    expr
                ))
            }
            AssignTarget::Index { target, index } => {
                let (pt, te) = emit_expr(target, ctx)?;
                let (pi, ie) = emit_expr(index, ctx)?;
                let (pv, ve) = emit_expr(value, ctx)?;
                let tmp = ctx.fresh("lst");
                let assign = if let Expr::Ident(n, _) = target {
                    format!("{pad}{} = {tmp};\n", c_ident(n))
                } else {
                    String::new()
                };
                Ok(format!(
                    "{}{}{}{pad}KList {tmp} = {te};\n{pad}klist_set(&{tmp}, {ie}, {ve});\n{assign}",
                    indent_pre(&pt, indent),
                    indent_pre(&pi, indent),
                    indent_pre(&pv, indent),
                ))
            }
            AssignTarget::Field { target, field } => {
                let (pv, ve) = emit_expr(value, ctx)?;
                if let Expr::Ident(n, _) = target {
                    Ok(format!(
                        "{}{pad}{}.{} = {};\n",
                        indent_pre(&pv, indent),
                        c_ident(n),
                        c_ident(field),
                        ve
                    ))
                } else {
                    Err(KengaError::at(
                        "emit-c: field assign only on named vars",
                        span.clone(),
                    ))
                }
            }
        },
        Stmt::Expr { expr, .. } => {
            if let Expr::Call { callee, args, .. } = expr {
                if callee == "println" && args.len() == 1 {
                    return emit_println(&args[0], indent, ctx);
                }
            }
            let (pre, e) = emit_expr(expr, ctx)?;
            Ok(format!("{}{pad}(void)({e});\n", indent_pre(&pre, indent)))
        }
        Stmt::Return { value, .. } => match value {
            Some(e) => {
                let (pre, expr) = emit_expr(e, ctx)?;
                Ok(format!(
                    "{}{pad}return {expr};\n",
                    indent_pre(&pre, indent)
                ))
            }
            None => Ok(format!("{pad}return;\n")),
        },
        Stmt::If {
            cond,
            then_block,
            else_block,
            ..
        } => {
            let (pre, c) = emit_expr(cond, ctx)?;
            let mut s = format!("{}{pad}if ({c}) {{\n", indent_pre(&pre, indent));
            for st in &then_block.stmts {
                s.push_str(&emit_stmt(st, indent + 1, ctx)?);
            }
            s.push_str(&format!("{pad}}}"));
            if let Some(eb) = else_block {
                s.push_str(" else {\n");
                for st in &eb.stmts {
                    s.push_str(&emit_stmt(st, indent + 1, ctx)?);
                }
                s.push_str(&format!("{pad}}}\n"));
            } else {
                s.push('\n');
            }
            Ok(s)
        }
        Stmt::While { cond, body, .. } => {
            let mut s = format!("{pad}while (1) {{\n");
            let (pre, c) = emit_expr(cond, ctx)?;
            s.push_str(&indent_pre(&pre, indent + 1));
            s.push_str(&format!("{}if (!({c})) break;\n", "  ".repeat(indent + 1)));
            for st in &body.stmts {
                s.push_str(&emit_stmt(st, indent + 1, ctx)?);
            }
            s.push_str(&format!("{pad}}}\n"));
            Ok(s)
        }
        Stmt::For {
            var, iter, body, ..
        } => emit_for(var, iter, body, indent, ctx),
        Stmt::Break(_) => Ok(format!("{pad}break;\n")),
        Stmt::Continue(_) => Ok(format!("{pad}continue;\n")),
    }
}

fn emit_for(
    var: &str,
    iter: &Expr,
    body: &crate::ast::Block,
    indent: usize,
    ctx: &mut EmitCtx<'_>,
) -> Result<String> {
    let pad = "  ".repeat(indent);
    let ip = "  ".repeat(indent + 1);
    if let Expr::Range { start, end, .. } = iter {
        let (pa, a) = emit_expr(start, ctx)?;
        let (pb, b) = emit_expr(end, ctx)?;
        ctx.vars.insert(var.to_string(), CTy::I64);
        let mut s = format!(
            "{}{}{pad}for (int64_t {} = {a}; {} < {b}; {}++) {{\n",
            indent_pre(&pa, indent),
            indent_pre(&pb, indent),
            c_ident(var),
            c_ident(var),
            c_ident(var)
        );
        for st in &body.stmts {
            s.push_str(&emit_stmt(st, indent + 1, ctx)?);
        }
        s.push_str(&format!("{pad}}}\n"));
        return Ok(s);
    }
    let (pre, list_e) = emit_expr(iter, ctx)?;
    let lst = ctx.fresh("iter");
    let idx = ctx.fresh("i");
    ctx.vars.insert(var.to_string(), CTy::I64);
    let mut s = format!("{}{pad}KList {lst} = {list_e};\n", indent_pre(&pre, indent));
    s.push_str(&format!(
        "{pad}for (size_t {idx} = 0; {idx} < {lst}.len; {idx}++) {{\n"
    ));
    s.push_str(&format!(
        "{ip}int64_t {} = klist_get({lst}, (int64_t){idx});\n",
        c_ident(var)
    ));
    for st in &body.stmts {
        s.push_str(&emit_stmt(st, indent + 1, ctx)?);
    }
    s.push_str(&format!("{pad}}}\n"));
    Ok(s)
}

fn emit_println(arg: &Expr, indent: usize, ctx: &mut EmitCtx<'_>) -> Result<String> {
    let pad = "  ".repeat(indent);
    if let Expr::Str(s, _) = arg {
        return Ok(format!(
            "{pad}kenga_println_str(\"{}\");\n",
            escape_c(s)
        ));
    }
    let ty = infer_expr(arg, ctx)?;
    let (pre, e) = emit_expr(arg, ctx)?;
    let call = match &ty {
        CTy::List => format!("kenga_println_list({e})"),
        CTy::Str => format!("kenga_println_str({e})"),
        CTy::Struct(n) => format!("kenga_println_str(\"<struct {n}>\")"),
        _ => format!("kenga_println_i64({e})"),
    };
    Ok(format!("{}{pad}{call};\n", indent_pre(&pre, indent)))
}

fn infer_expr(expr: &Expr, ctx: &EmitCtx<'_>) -> Result<CTy> {
    Ok(match expr {
        Expr::Int(_, _) | Expr::Bool(_, _) | Expr::Float(_, _) => CTy::I64,
        Expr::Str(_, _) => CTy::Str,
        Expr::List(_, _) | Expr::Range { .. } => CTy::List,
        Expr::Ident(name, span) => ctx.vars.get(name).cloned().ok_or_else(|| {
            KengaError::at(format!("unknown variable '{name}'"), span.clone())
        })?,
        Expr::Index { .. } => CTy::I64,
        Expr::Unary { .. } | Expr::Binary { .. } => CTy::I64,
        Expr::Field {
            target,
            field,
            span,
        } => {
            let t = infer_expr(target, ctx)?;
            match t {
                CTy::Struct(n) => {
                    let info = ctx.structs.get(&n).ok_or_else(|| {
                        KengaError::at(format!("unknown struct '{n}'"), span.clone())
                    })?;
                    info.fields
                        .iter()
                        .find(|(f, _)| f == field)
                        .map(|(_, ty)| ty.clone())
                        .ok_or_else(|| {
                            KengaError::at(format!("unknown field '{field}'"), span.clone())
                        })?
                }
                _ => {
                    return Err(KengaError::at(
                        "field access on non-struct",
                        span.clone(),
                    ))
                }
            }
        }
        Expr::StructLit { name, span, .. } => {
            if ctx.structs.contains_key(name) {
                CTy::Struct(name.clone())
            } else {
                return Err(KengaError::at(
                    format!("unknown struct '{name}'"),
                    span.clone(),
                ));
            }
        }
        Expr::Call { callee, .. } => {
            if callee == "len" {
                CTy::I64
            } else if callee == "push" {
                CTy::List
            } else if let Some((ret, _)) = ctx.sigs.get(callee) {
                ret.clone()
            } else {
                CTy::I64
            }
        }
    })
}

fn emit_expr(expr: &Expr, ctx: &mut EmitCtx<'_>) -> Result<(String, String)> {
    match expr {
        Expr::Int(n, _) => Ok((String::new(), n.to_string())),
        Expr::Float(n, _) => Ok((String::new(), format!("{}", *n as i64))),
        Expr::Bool(b, _) => Ok((String::new(), if *b { "1" } else { "0" }.into())),
        Expr::Str(s, _) => Ok((String::new(), format!("\"{}\"", escape_c(s)))),
        Expr::Ident(name, _) => Ok((String::new(), c_ident(name))),
        Expr::List(elems, _) => {
            let tmp = ctx.fresh("list");
            let mut pre = format!("KList {tmp} = klist_new();\n");
            for e in elems {
                let (p, v) = emit_expr(e, ctx)?;
                pre.push_str(&p);
                pre.push_str(&format!("klist_push(&{tmp}, {v});\n"));
            }
            Ok((pre, tmp))
        }
        Expr::Range { start, end, .. } => {
            let tmp = ctx.fresh("range");
            let (pa, a) = emit_expr(start, ctx)?;
            let (pb, b) = emit_expr(end, ctx)?;
            let i = ctx.fresh("r");
            let mut pre = format!("{pa}{pb}KList {tmp} = klist_new();\n");
            pre.push_str(&format!(
                "for (int64_t {i} = {a}; {i} < {b}; {i}++) klist_push(&{tmp}, {i});\n"
            ));
            Ok((pre, tmp))
        }
        Expr::Index { target, index, .. } => {
            let (pt, t) = emit_expr(target, ctx)?;
            let (pi, i) = emit_expr(index, ctx)?;
            Ok((format!("{pt}{pi}"), format!("klist_get({t}, {i})")))
        }
        Expr::Field { target, field, .. } => {
            let (p, t) = emit_expr(target, ctx)?;
            Ok((p, format!("{}.{}", t, c_ident(field))))
        }
        Expr::StructLit { name, fields, span } => {
            let info = ctx.structs.get(name).ok_or_else(|| {
                KengaError::at(format!("unknown struct '{name}'"), span.clone())
            })?;
            let field_defs = info.fields.clone();
            let mut pre = String::new();
            let mut parts = Vec::new();
            for (fname, _) in &field_defs {
                let val = fields
                    .iter()
                    .find(|(n, _)| n == fname)
                    .map(|(_, e)| e)
                    .ok_or_else(|| {
                        KengaError::at(
                            format!("missing field '{fname}' in {name} literal"),
                            span.clone(),
                        )
                    })?;
                let (p, e) = emit_expr(val, ctx)?;
                pre.push_str(&p);
                parts.push(format!(".{} = {e}", c_ident(fname)));
            }
            Ok((
                pre,
                format!("({}){{ {} }}", struct_c_name(name), parts.join(", ")),
            ))
        }
        Expr::Unary { op, expr, .. } => {
            let (p, e) = emit_expr(expr, ctx)?;
            let o = match op {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "!",
            };
            Ok((p, format!("({o}{e})")))
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            let (pl, l) = emit_expr(left, ctx)?;
            let (pr, r) = emit_expr(right, ctx)?;
            if *op == BinaryOp::Add {
                let lt = infer_expr(left, ctx)?;
                let rt = infer_expr(right, ctx)?;
                if lt == CTy::List && rt == CTy::List {
                    let tmp = ctx.fresh("cat");
                    return Ok((
                        format!("{pl}{pr}KList {tmp} = klist_concat({l}, {r});\n"),
                        tmp,
                    ));
                }
            }
            let o = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Rem => "%",
                BinaryOp::Eq => "==",
                BinaryOp::Ne => "!=",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                BinaryOp::And => "&&",
                BinaryOp::Or => "||",
            };
            Ok((format!("{pl}{pr}"), format!("({l} {o} {r})")))
        }
        Expr::Call { callee, args, span } => match callee.as_str() {
            "println" => Err(KengaError::at(
                "println must be a statement",
                span.clone(),
            )),
            "len" => {
                if args.len() != 1 {
                    return Err(KengaError::at("len takes 1 arg", span.clone()));
                }
                let (p, e) = emit_expr(&args[0], ctx)?;
                Ok((p, format!("((int64_t){e}.len)")))
            }
            "push" => {
                if args.len() != 2 {
                    return Err(KengaError::at("push takes 2 args", span.clone()));
                }
                let (pl, l) = emit_expr(&args[0], ctx)?;
                let (pv, v) = emit_expr(&args[1], ctx)?;
                let tmp = ctx.fresh("push");
                Ok((
                    format!("{pl}{pv}KList {tmp} = {l};\nklist_push(&{tmp}, {v});\n"),
                    tmp,
                ))
            }
            other => {
                let mut pre = String::new();
                let mut parts = Vec::new();
                for a in args {
                    let (p, e) = emit_expr(a, ctx)?;
                    pre.push_str(&p);
                    parts.push(e);
                }
                Ok((
                    pre,
                    format!("{}({})", c_ident(other), parts.join(", ")),
                ))
            }
        },
    }
}

const RUNTIME: &str = r#"/* Generated by Kenga emit-c */
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

typedef struct {
  int64_t *data;
  size_t len;
  size_t cap;
} KList;

static KList klist_new(void) {
  KList l; l.data = NULL; l.len = 0; l.cap = 0; return l;
}
static void klist_push(KList *l, int64_t v) {
  if (l->len + 1 > l->cap) {
    size_t ncap = l->cap ? l->cap * 2 : 8;
    int64_t *nd = (int64_t *)realloc(l->data, ncap * sizeof(int64_t));
    if (!nd) { fprintf(stderr, "oom\n"); exit(1); }
    l->data = nd; l->cap = ncap;
  }
  l->data[l->len++] = v;
}
static int64_t klist_get(KList l, int64_t i) {
  if (i < 0 || (size_t)i >= l.len) { fprintf(stderr, "index oob\n"); exit(1); }
  return l.data[i];
}
static void klist_set(KList *l, int64_t i, int64_t v) {
  if (i < 0 || (size_t)i >= l->len) { fprintf(stderr, "index oob\n"); exit(1); }
  l->data[i] = v;
}
static KList klist_concat(KList a, KList b) {
  KList o = klist_new();
  for (size_t i = 0; i < a.len; i++) klist_push(&o, a.data[i]);
  for (size_t i = 0; i < b.len; i++) klist_push(&o, b.data[i]);
  return o;
}
static void kenga_println_i64(int64_t v) { printf("%lld\n", (long long)v); }
static void kenga_println_str(const char *s) { puts(s); }
static void kenga_println_list(KList l) {
  putchar('[');
  for (size_t i = 0; i < l.len; i++) {
    if (i) printf(", ");
    printf("%lld", (long long)l.data[i]);
  }
  puts("]");
}

"#;
