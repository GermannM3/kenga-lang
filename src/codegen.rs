//! C99 backend for a practical Kenga subset.
//! Supports: i64, lists, ranges, for/while/break/continue, if, fn calls, println.

use std::collections::HashMap;

use crate::ast::{
    AssignTarget, BinaryOp, Expr, Function, Item, Program, Stmt, Type, UnaryOp,
};
use crate::error::{KengaError, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CTy {
    I64,
    List,
    Str,
    Void,
}

struct EmitCtx {
    vars: HashMap<String, CTy>,
    tmp: usize,
}

impl EmitCtx {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
            tmp: 0,
        }
    }

    fn fresh(&mut self, prefix: &str) -> String {
        let n = self.tmp;
        self.tmp += 1;
        format!("__{prefix}_{n}")
    }
}

pub fn emit_c(program: &Program) -> Result<String> {
    let mut fns = String::new();
    let mut has_main = false;
    let mut fn_sigs: HashMap<String, (CTy, Vec<CTy>)> = HashMap::new();

    for item in &program.items {
        if let Item::Function(f) = item {
            let ret = map_ret(&f.ret);
            let params: Vec<CTy> = f
                .params
                .iter()
                .map(|p| map_type(&p.ty))
                .collect();
            fn_sigs.insert(f.name.clone(), (ret, params));
            if f.name == "main" {
                has_main = true;
            }
        }
    }

    if !has_main {
        return Err(KengaError::new("emit-c requires fn main()", None));
    }

    for item in &program.items {
        match item {
            Item::Function(f) => {
                fns.push_str(&emit_function(f, &fn_sigs)?);
                fns.push('\n');
            }
            Item::Struct(_) | Item::Intrinsic(_) | Item::EventHandler(_) => {}
        }
    }

    let mut out = String::new();
    out.push_str(RUNTIME);
    out.push_str(&fns);
    Ok(out)
}

fn map_ret(t: &Type) -> CTy {
    match t {
        Type::Void => CTy::Void,
        Type::List => CTy::List,
        Type::Str => CTy::Str,
        _ => CTy::I64,
    }
}

fn map_type(t: &Type) -> CTy {
    match t {
        Type::List => CTy::List,
        Type::Str => CTy::Str,
        Type::Void => CTy::Void,
        _ => CTy::I64,
    }
}

fn c_type(t: CTy) -> &'static str {
    match t {
        CTy::I64 => "int64_t",
        CTy::List => "KList",
        CTy::Str => "const char*",
        CTy::Void => "void",
    }
}

fn emit_function(f: &Function, sigs: &HashMap<String, (CTy, Vec<CTy>)>) -> Result<String> {
    let (ret, _) = sigs
        .get(&f.name)
        .cloned()
        .unwrap_or((map_ret(&f.ret), Vec::new()));
    let mut ctx = EmitCtx::new();
    let mut s = format!("{} {}(", c_type(ret), c_ident(&f.name));
    if f.params.is_empty() {
        s.push_str("void");
    } else {
        for (i, p) in f.params.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            let ty = map_type(&p.ty);
            ctx.vars.insert(p.name.clone(), ty);
            s.push_str(&format!("{} {}", c_type(ty), c_ident(&p.name)));
        }
    }
    s.push_str(") {\n");
    for stmt in &f.body.stmts {
        s.push_str(&emit_stmt(stmt, 1, &mut ctx, sigs)?);
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

fn emit_stmt(
    stmt: &Stmt,
    indent: usize,
    ctx: &mut EmitCtx,
    sigs: &HashMap<String, (CTy, Vec<CTy>)>,
) -> Result<String> {
    let pad = "  ".repeat(indent);
    match stmt {
        Stmt::Let { name, value, ty, .. } => {
            let inferred = infer_expr(value, ctx, sigs)?;
            let cty = ty.as_ref().map(map_type).unwrap_or(inferred);
            ctx.vars.insert(name.clone(), cty);
            let (pre, expr) = emit_expr(value, ctx, sigs)?;
            Ok(format!(
                "{pre}{pad}{} {} = {};\n",
                c_type(cty),
                c_ident(name),
                expr
            ))
        }
        Stmt::Assign { target, value, .. } => match target {
            AssignTarget::Name(name) => {
                let (pre, expr) = emit_expr(value, ctx, sigs)?;
                Ok(format!("{pre}{pad}{} = {};\n", c_ident(name), expr))
            }
            AssignTarget::Index { target, index } => {
                let (pre_t, t_expr) = emit_expr(target, ctx, sigs)?;
                let (pre_i, i_expr) = emit_expr(index, ctx, sigs)?;
                let (pre_v, v_expr) = emit_expr(value, ctx, sigs)?;
                // list set returns new list conceptually; mutate in place in C
                let tmp = ctx.fresh("lst");
                Ok(format!(
                    "{pre_t}{pre_i}{pre_v}{pad}KList {tmp} = {t_expr};\n{pad}klist_set(&{tmp}, {i_expr}, {v_expr});\n{pad}{assign}\n",
                    assign = if let Expr::Ident(n, _) = target {
                        format!("{} = {};", c_ident(n), tmp)
                    } else {
                        format!("(void){tmp};")
                    }
                ))
            }
            AssignTarget::Field { .. } => Err(KengaError::new(
                "emit-c: struct field assign not supported yet",
                None,
            )),
        },
        Stmt::Expr { expr, .. } => {
            if let Expr::Call { callee, args, .. } = expr {
                if callee == "println" && args.len() == 1 {
                    return emit_println(&args[0], indent, ctx, sigs);
                }
            }
            let (pre, e) = emit_expr(expr, ctx, sigs)?;
            Ok(format!("{pre}{pad}(void)({e});\n"))
        }
        Stmt::Return { value, .. } => match value {
            Some(e) => {
                let (pre, expr) = emit_expr(e, ctx, sigs)?;
                Ok(format!("{pre}{pad}return {expr};\n"))
            }
            None => Ok(format!("{pad}return;\n")),
        },
        Stmt::If {
            cond,
            then_block,
            else_block,
            ..
        } => {
            let (pre, c) = emit_expr(cond, ctx, sigs)?;
            let mut s = format!("{pre}{pad}if ({c}) {{\n");
            for st in &then_block.stmts {
                s.push_str(&emit_stmt(st, indent + 1, ctx, sigs)?);
            }
            s.push_str(&format!("{pad}}}"));
            if let Some(eb) = else_block {
                s.push_str(" else {\n");
                for st in &eb.stmts {
                    s.push_str(&emit_stmt(st, indent + 1, ctx, sigs)?);
                }
                s.push_str(&format!("{pad}}}\n"));
            } else {
                s.push('\n');
            }
            Ok(s)
        }
        Stmt::While { cond, body, .. } => {
            let (pre, c) = emit_expr(cond, ctx, sigs)?;
            // hoist precondition temps once; recompute cond in loop header via while(1)+if break
            let mut s = format!("{pre}{pad}while (1) {{\n");
            let (pre2, c2) = emit_expr(cond, ctx, sigs)?;
            s.push_str(&pre2);
            s.push_str(&format!(
                "{}if (!({c2})) break;\n",
                "  ".repeat(indent + 1)
            ));
            for st in &body.stmts {
                s.push_str(&emit_stmt(st, indent + 1, ctx, sigs)?);
            }
            s.push_str(&format!("{pad}}}\n"));
            let _ = c; // initial pre already emitted
            Ok(s)
        }
        Stmt::For {
            var,
            iter,
            body,
            ..
        } => emit_for(var, iter, body, indent, ctx, sigs),
        Stmt::Break(_) => Ok(format!("{pad}break;\n")),
        Stmt::Continue(_) => Ok(format!("{pad}continue;\n")),
    }
}

fn emit_for(
    var: &str,
    iter: &Expr,
    body: &crate::ast::Block,
    indent: usize,
    ctx: &mut EmitCtx,
    sigs: &HashMap<String, (CTy, Vec<CTy>)>,
) -> Result<String> {
    let pad = "  ".repeat(indent);
    let ip = "  ".repeat(indent + 1);

    if let Expr::Range { start, end, .. } = iter {
        let (pre_a, a) = emit_expr(start, ctx, sigs)?;
        let (pre_b, b) = emit_expr(end, ctx, sigs)?;
        ctx.vars.insert(var.to_string(), CTy::I64);
        let mut s = format!("{pre_a}{pre_b}{pad}for (int64_t {} = {a}; {} < {b}; {}++) {{\n", c_ident(var), c_ident(var), c_ident(var));
        for st in &body.stmts {
            s.push_str(&emit_stmt(st, indent + 1, ctx, sigs)?);
        }
        s.push_str(&format!("{pad}}}\n"));
        return Ok(s);
    }

    // for x in list
    let (pre, list_e) = emit_expr(iter, ctx, sigs)?;
    let lst = ctx.fresh("iter");
    let idx = ctx.fresh("i");
    ctx.vars.insert(var.to_string(), CTy::I64);
    let mut s = format!("{pre}{pad}KList {lst} = {list_e};\n");
    s.push_str(&format!(
        "{pad}for (size_t {idx} = 0; {idx} < {lst}.len; {idx}++) {{\n"
    ));
    s.push_str(&format!(
        "{ip}int64_t {} = klist_get({lst}, (int64_t){idx});\n",
        c_ident(var)
    ));
    for st in &body.stmts {
        s.push_str(&emit_stmt(st, indent + 1, ctx, sigs)?);
    }
    s.push_str(&format!("{pad}}}\n"));
    Ok(s)
}

fn emit_println(
    arg: &Expr,
    indent: usize,
    ctx: &mut EmitCtx,
    sigs: &HashMap<String, (CTy, Vec<CTy>)>,
) -> Result<String> {
    let pad = "  ".repeat(indent);
    match arg {
        Expr::Str(s, _) => Ok(format!(
            "{pad}kenga_println_str(\"{}\");\n",
            escape_c(s)
        )),
        other => {
            let ty = infer_expr(other, ctx, sigs)?;
            let (pre, e) = emit_expr(other, ctx, sigs)?;
            match ty {
                CTy::List => Ok(format!("{pre}{pad}kenga_println_list({e});\n")),
                CTy::Str => Ok(format!("{pre}{pad}kenga_println_str({e});\n")),
                _ => Ok(format!("{pre}{pad}kenga_println_i64({e});\n")),
            }
        }
    }
}

fn infer_expr(
    expr: &Expr,
    ctx: &EmitCtx,
    sigs: &HashMap<String, (CTy, Vec<CTy>)>,
) -> Result<CTy> {
    Ok(match expr {
        Expr::Int(_, _) | Expr::Bool(_, _) | Expr::Float(_, _) => CTy::I64,
        Expr::Str(_, _) => CTy::Str,
        Expr::List(_, _) | Expr::Range { .. } => CTy::List,
        Expr::Ident(name, _) => *ctx.vars.get(name).unwrap_or(&CTy::I64),
        Expr::Index { .. } => CTy::I64,
        Expr::Unary { .. } | Expr::Binary { .. } => CTy::I64,
        Expr::Call { callee, .. } => {
            if callee == "len" || callee == "push" {
                if callee == "push" {
                    CTy::List
                } else {
                    CTy::I64
                }
            } else if let Some((ret, _)) = sigs.get(callee) {
                *ret
            } else {
                CTy::I64
            }
        }
        Expr::Field { .. } | Expr::StructLit { .. } => {
            return Err(KengaError::new(
                "emit-c: structs not supported yet",
                None,
            ));
        }
    })
}

/// Returns (prelude_statements, expression_code)
fn emit_expr(
    expr: &Expr,
    ctx: &mut EmitCtx,
    sigs: &HashMap<String, (CTy, Vec<CTy>)>,
) -> Result<(String, String)> {
    match expr {
        Expr::Int(n, _) => Ok((String::new(), n.to_string())),
        Expr::Float(n, _) => Ok((String::new(), format!("{}", *n as i64))),
        Expr::Bool(true, _) => Ok((String::new(), "1".into())),
        Expr::Bool(false, _) => Ok((String::new(), "0".into())),
        Expr::Str(s, _) => Ok((String::new(), format!("\"{}\"", escape_c(s)))),
        Expr::Ident(name, _) => Ok((String::new(), c_ident(name))),
        Expr::List(elems, _) => {
            let tmp = ctx.fresh("list");
            let mut pre = format!("KList {tmp} = klist_new();\n");
            // need indent? callers prefix pad only on final lines — prelude without pad is ok if we add spaces at use sites
            // Better: emit prelude with no pad; stmt emitter will place it before pad lines.
            for e in elems {
                let (p, v) = emit_expr(e, ctx, sigs)?;
                pre.push_str(&p);
                pre.push_str(&format!("klist_push(&{tmp}, {v});\n"));
            }
            Ok((pre, tmp))
        }
        Expr::Range { start, end, .. } => {
            // Materialize range as list for generality
            let tmp = ctx.fresh("range");
            let (pa, a) = emit_expr(start, ctx, sigs)?;
            let (pb, b) = emit_expr(end, ctx, sigs)?;
            let i = ctx.fresh("r");
            let mut pre = format!("{pa}{pb}KList {tmp} = klist_new();\n");
            pre.push_str(&format!(
                "for (int64_t {i} = {a}; {i} < {b}; {i}++) klist_push(&{tmp}, {i});\n"
            ));
            Ok((pre, tmp))
        }
        Expr::Index { target, index, .. } => {
            let (pt, t) = emit_expr(target, ctx, sigs)?;
            let (pi, i) = emit_expr(index, ctx, sigs)?;
            Ok((format!("{pt}{pi}"), format!("klist_get({t}, {i})")))
        }
        Expr::Unary { op, expr, .. } => {
            let (p, e) = emit_expr(expr, ctx, sigs)?;
            let o = match op {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "!",
            };
            Ok((p, format!("({o}{e})")))
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            let (pl, l) = emit_expr(left, ctx, sigs)?;
            let (pr, r) = emit_expr(right, ctx, sigs)?;
            // string concat not supported; list concat via klist_concat
            if *op == BinaryOp::Add {
                let lt = infer_expr(left, ctx, sigs)?;
                let rt = infer_expr(right, ctx, sigs)?;
                if lt == CTy::List && rt == CTy::List {
                    let tmp = ctx.fresh("cat");
                    let pre = format!("{pl}{pr}KList {tmp} = klist_concat({l}, {r});\n");
                    return Ok((pre, tmp));
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
        Expr::Call { callee, args, .. } => match callee.as_str() {
            "println" => Err(KengaError::new(
                "println must be used as a statement",
                None,
            )),
            "len" => {
                if args.len() != 1 {
                    return Err(KengaError::new("len takes 1 arg", None));
                }
                let (p, e) = emit_expr(&args[0], ctx, sigs)?;
                Ok((p, format!("((int64_t){e}.len)")))
            }
            "push" => {
                if args.len() != 2 {
                    return Err(KengaError::new("push takes 2 args", None));
                }
                let (pl, l) = emit_expr(&args[0], ctx, sigs)?;
                let (pv, v) = emit_expr(&args[1], ctx, sigs)?;
                let tmp = ctx.fresh("push");
                let pre = format!("{pl}{pv}KList {tmp} = {l};\nklist_push(&{tmp}, {v});\n");
                Ok((pre, tmp))
            }
            other => {
                let mut pre = String::new();
                let mut parts = Vec::new();
                for a in args {
                    let (p, e) = emit_expr(a, ctx, sigs)?;
                    pre.push_str(&p);
                    parts.push(e);
                }
                Ok((
                    pre,
                    format!("{}({})", c_ident(other), parts.join(", ")),
                ))
            }
        },
        Expr::Field { .. } | Expr::StructLit { .. } => Err(KengaError::new(
            "emit-c: structs not supported yet",
            None,
        )),
    }
}

fn c_ident(name: &str) -> String {
    if name == "main" {
        return "main".into();
    }
    format!("k_{name}")
}

fn escape_c(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

const RUNTIME: &str = r#"/* Generated by Kenga emit-c - bootstrap native backend */
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
  KList l;
  l.data = NULL;
  l.len = 0;
  l.cap = 0;
  return l;
}

static void klist_push(KList *l, int64_t v) {
  if (l->len + 1 > l->cap) {
    size_t ncap = l->cap ? l->cap * 2 : 8;
    int64_t *nd = (int64_t *)realloc(l->data, ncap * sizeof(int64_t));
    if (!nd) { fprintf(stderr, "oom\n"); exit(1); }
    l->data = nd;
    l->cap = ncap;
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
