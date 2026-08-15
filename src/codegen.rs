//! C99 backend: tagged KVal runtime (i64 / f64 / str / list handles), structs, for/while.

use std::collections::HashMap;

use crate::ast::{
    AssignTarget, BinaryOp, Expr, Function, Item, Program, Stmt, Type, UnaryOp,
};
use crate::error::{KengaError, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
enum CTy {
    I64,
    F64,
    List,
    Str,
    Void,
    Struct(String),
    /// Dynamic tagged value (list index, for-loop elem over hetero list).
    Val,
}

struct StructInfo {
    fields: Vec<(String, CTy)>,
}

struct EmitCtx<'a> {
    vars: HashMap<String, CTy>,
    structs: &'a HashMap<String, StructInfo>,
    sigs: &'a HashMap<String, (CTy, Vec<CTy>)>,
    ret: CTy,
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

    // Forward declarations (mutual recursion in selfhost etc.)
    for item in &program.items {
        if let Item::Function(f) = item {
            if f.name == "main" {
                continue;
            }
            body.push_str(&emit_prototype(f, &structs)?);
            body.push_str(";\n");
        }
    }
    body.push('\n');

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
        Type::F64 => CTy::F64,
        Type::I64 | Type::Bool | Type::Tensor | Type::Memory => CTy::I64,
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
        CTy::I64 | CTy::List => "int64_t".into(),
        CTy::F64 => "double".into(),
        CTy::Str => "const char*".into(),
        CTy::Void => "void".into(),
        CTy::Struct(n) => struct_c_name(n),
        CTy::Val => "KVal".into(),
    }
}

fn is_numeric(t: &CTy) -> bool {
    matches!(t, CTy::I64 | CTy::F64)
}

fn promote_num(a: &CTy, b: &CTy) -> Option<CTy> {
    match (a, b) {
        (CTy::F64, CTy::F64)
        | (CTy::F64, CTy::I64)
        | (CTy::I64, CTy::F64) => Some(CTy::F64),
        (CTy::I64, CTy::I64) => Some(CTy::I64),
        _ => None,
    }
}

fn c_float_lit(n: f64) -> String {
    if n.is_finite() && n.fract() == 0.0 {
        format!("{n:.1}")
    } else {
        format!("{n}")
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
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn indent_pre(pre: &str, indent: usize) -> String {
    if pre.is_empty() {
        return String::new();
    }
    let pad = "  ".repeat(indent);
    pre.lines().map(|l| format!("{pad}{l}\n")).collect()
}

fn wrap_as_val(expr: &str, from: &CTy) -> String {
    match from {
        CTy::Val => expr.to_string(),
        CTy::I64 => format!("kval_i64({expr})"),
        CTy::F64 => format!("kval_f64({expr})"),
        CTy::Str => format!("kval_str({expr})"),
        CTy::List => format!("kval_list({expr})"),
        CTy::Void | CTy::Struct(_) => format!("kval_i64(0)"),
    }
}

fn coerce_expr(expr: &str, from: &CTy, to: &CTy) -> Result<String> {
    if from == to {
        return Ok(expr.to_string());
    }
    Ok(match (from, to) {
        (CTy::Val, CTy::I64) => format!("kval_as_i64({expr})"),
        (CTy::Val, CTy::F64) => format!("kval_as_f64({expr})"),
        (CTy::Val, CTy::Str) => format!("kval_as_str({expr})"),
        (CTy::Val, CTy::List) => format!("kval_as_list({expr})"),
        (CTy::I64, CTy::Val) => format!("kval_i64({expr})"),
        (CTy::F64, CTy::Val) => format!("kval_f64({expr})"),
        (CTy::I64, CTy::F64) => format!("((double)({expr}))"),
        (CTy::F64, CTy::I64) => format!("((int64_t)({expr}))"),
        (CTy::I64, CTy::Str) => format!("kstr_from_i64({expr})"),
        (CTy::F64, CTy::Str) => format!("kstr_from_f64({expr})"),
        (CTy::Str, CTy::Val) => format!("kval_str({expr})"),
        (CTy::List, CTy::Val) => format!("kval_list({expr})"),
        // List is an int64_t handle; Val→List already covered. I64↔List is a handle misuse —
        // allow only if both are int64_t at C level without conversion (should not happen).
        _ if c_type(from) == c_type(to) => expr.to_string(),
        _ => {
            return Err(KengaError::new(
                format!("emit-c: cannot coerce {:?} to {:?}", from, to),
                None,
            ))
        }
    })
}

fn emit_prototype(f: &Function, structs: &HashMap<String, StructInfo>) -> Result<String> {
    let ret = map_type(&f.ret, structs)?;
    let mut s = format!("{} {}(", c_type(&ret), c_ident(&f.name));
    if f.params.is_empty() {
        s.push_str("void");
    } else {
        for (i, p) in f.params.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            let ty = map_type(&p.ty, structs)?;
            s.push_str(&format!("{} {}", c_type(&ty), c_ident(&p.name)));
        }
    }
    s.push(')');
    Ok(s)
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
        ret: ret.clone(),
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
                inferred.clone()
            };
            ctx.vars.insert(name.clone(), cty.clone());
            let (pre, expr) = emit_expr(value, ctx)?;
            let expr = coerce_expr(&expr, &inferred, &cty)?;
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
                let value_ty = infer_expr(value, ctx)?;
                let target_ty = ctx.vars.get(name).cloned().unwrap_or(value_ty.clone());
                let expr = coerce_expr(&expr, &value_ty, &target_ty)?;
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
                let list_ty = infer_expr(target, ctx)?;
                let idx_ty = infer_expr(index, ctx)?;
                let val_ty = infer_expr(value, ctx)?;
                let list_e = coerce_expr(&te, &list_ty, &CTy::List)?;
                let idx_e = coerce_expr(&ie, &idx_ty, &CTy::I64)?;
                let val_e = wrap_as_val(&ve, &val_ty);
                Ok(format!(
                    "{}{}{}{pad}klist_set_val({list_e}, {idx_e}, {val_e});\n",
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
                let ety = infer_expr(e, ctx)?;
                let expr = coerce_expr(&expr, &ety, &ctx.ret)?;
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
            let cty = infer_expr(cond, ctx)?;
            let c = coerce_expr(&c, &cty, &CTy::I64)?;
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
            let cty = infer_expr(cond, ctx)?;
            let c = coerce_expr(&c, &cty, &CTy::I64)?;
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
    let iter_ty = infer_expr(iter, ctx)?;
    let list_e = coerce_expr(&list_e, &iter_ty, &CTy::List)?;
    let lst = ctx.fresh("iter");
    let idx = ctx.fresh("i");
    ctx.vars.insert(var.to_string(), CTy::Val);
    let mut s = format!(
        "{}{pad}int64_t {lst} = {list_e};\n",
        indent_pre(&pre, indent)
    );
    s.push_str(&format!(
        "{pad}for (int64_t {idx} = 0; {idx} < klist_len({lst}); {idx}++) {{\n"
    ));
    s.push_str(&format!(
        "{ip}KVal {} = klist_get_val({lst}, {idx});\n",
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
        CTy::Val => format!("kenga_println_val({e})"),
        CTy::F64 => format!("kenga_println_f64({e})"),
        CTy::Struct(n) => format!("kenga_println_str(\"<struct {n}>\")"),
        _ => format!("kenga_println_i64({e})"),
    };
    Ok(format!("{}{pad}{call};\n", indent_pre(&pre, indent)))
}

fn infer_expr(expr: &Expr, ctx: &EmitCtx<'_>) -> Result<CTy> {
    Ok(match expr {
        Expr::Int(_, _) | Expr::Bool(_, _) => CTy::I64,
        Expr::Float(_, _) => CTy::F64,
        Expr::Str(_, _) => CTy::Str,
        Expr::List(_, _) => CTy::List,
        Expr::Range { .. } => CTy::List,
        Expr::Ident(name, span) => ctx.vars.get(name).cloned().ok_or_else(|| {
            KengaError::at(format!("unknown variable '{name}'"), span.clone())
        })?,
        Expr::Index { .. } => CTy::Val,
        Expr::Unary { op, expr, .. } => {
            let t = infer_expr(expr, ctx)?;
            match op {
                UnaryOp::Neg if t == CTy::F64 => CTy::F64,
                UnaryOp::Neg if is_numeric(&t) => CTy::I64,
                UnaryOp::Not => CTy::I64,
                _ => CTy::I64,
            }
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            let lt = infer_expr(left, ctx)?;
            let rt = infer_expr(right, ctx)?;
            if *op == BinaryOp::Add {
                if lt == CTy::List && rt == CTy::List {
                    return Ok(CTy::List);
                }
                if lt == CTy::Str || rt == CTy::Str {
                    return Ok(CTy::Str);
                }
            }
            if matches!(
                *op,
                BinaryOp::Eq
                    | BinaryOp::Ne
                    | BinaryOp::Lt
                    | BinaryOp::Le
                    | BinaryOp::Gt
                    | BinaryOp::Ge
                    | BinaryOp::And
                    | BinaryOp::Or
            ) {
                return Ok(CTy::I64);
            }
            if let Some(p) = promote_num(&lt, &rt) {
                return Ok(p);
            }
            CTy::I64
        }
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
            } else if callee == "round" {
                CTy::I64
            } else if let Some((ret, _)) = ctx.sigs.get(callee) {
                ret.clone()
            } else {
                CTy::I64
            }
        }
    })
}

fn emit_as_i64(expr: &Expr, ctx: &mut EmitCtx<'_>) -> Result<(String, String)> {
    let (pre, e) = emit_expr(expr, ctx)?;
    let ty = infer_expr(expr, ctx)?;
    let e = coerce_expr(&e, &ty, &CTy::I64)?;
    Ok((pre, e))
}

fn emit_as_num(expr: &Expr, ctx: &mut EmitCtx<'_>, want: &CTy) -> Result<(String, String)> {
    let (pre, e) = emit_expr(expr, ctx)?;
    let ty = infer_expr(expr, ctx)?;
    let e = coerce_expr(&e, &ty, want)?;
    Ok((pre, e))
}

fn emit_expr(expr: &Expr, ctx: &mut EmitCtx<'_>) -> Result<(String, String)> {
    match expr {
        Expr::Int(n, _) => Ok((String::new(), n.to_string())),
        Expr::Float(n, _) => Ok((String::new(), c_float_lit(*n))),
        Expr::Bool(b, _) => Ok((String::new(), if *b { "1" } else { "0" }.into())),
        Expr::Str(s, _) => Ok((String::new(), format!("\"{}\"", escape_c(s)))),
        Expr::Ident(name, _) => Ok((String::new(), c_ident(name))),
        Expr::List(elems, _) => {
            let tmp = ctx.fresh("list");
            let mut pre = format!("int64_t {tmp} = klist_new();\n");
            for e in elems {
                let (p, v) = emit_expr(e, ctx)?;
                let ty = infer_expr(e, ctx)?;
                pre.push_str(&p);
                pre.push_str(&format!(
                    "klist_push_val({tmp}, {});\n",
                    wrap_as_val(&v, &ty)
                ));
            }
            Ok((pre, tmp))
        }
        Expr::Range { start, end, .. } => {
            let tmp = ctx.fresh("range");
            let (pa, a) = emit_expr(start, ctx)?;
            let (pb, b) = emit_expr(end, ctx)?;
            let i = ctx.fresh("r");
            let mut pre = format!("{pa}{pb}int64_t {tmp} = klist_new();\n");
            pre.push_str(&format!(
                "for (int64_t {i} = {a}; {i} < {b}; {i}++) klist_push_val({tmp}, kval_i64({i}));\n"
            ));
            Ok((pre, tmp))
        }
        Expr::Index { target, index, .. } => {
            let (pt, t) = emit_expr(target, ctx)?;
            let (pi, i) = emit_expr(index, ctx)?;
            let tty = infer_expr(target, ctx)?;
            let ity = infer_expr(index, ctx)?;
            let i = coerce_expr(&i, &ity, &CTy::I64)?;
            if tty == CTy::Str {
                Ok((
                    format!("{pt}{pi}"),
                    format!("kstr_index_val({t}, {i})"),
                ))
            } else if tty == CTy::Val {
                // dynamic: try list first via helper
                Ok((
                    format!("{pt}{pi}"),
                    format!("kval_index({t}, {i})"),
                ))
            } else {
                let t = coerce_expr(&t, &tty, &CTy::List)?;
                Ok((format!("{pt}{pi}"), format!("klist_get_val({t}, {i})")))
            }
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
            for (fname, fty) in &field_defs {
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
                let ety = infer_expr(val, ctx)?;
                let e = coerce_expr(&e, &ety, fty)?;
                pre.push_str(&p);
                parts.push(format!(".{} = {e}", c_ident(fname)));
            }
            Ok((
                pre,
                format!("({}){{ {} }}", struct_c_name(name), parts.join(", ")),
            ))
        }
        Expr::Unary { op, expr, .. } => {
            let ty = infer_expr(expr, ctx)?;
            let want = if *op == UnaryOp::Neg && ty == CTy::F64 {
                CTy::F64
            } else {
                CTy::I64
            };
            let (p, e) = emit_as_num(expr, ctx, &want)?;
            let o = match op {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "!",
            };
            Ok((p, format!("({o}{e})")))
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            let lt = infer_expr(left, ctx)?;
            let rt = infer_expr(right, ctx)?;
            if *op == BinaryOp::Add {
                if lt == CTy::List && rt == CTy::List {
                    let (pl, l) = emit_expr(left, ctx)?;
                    let (pr, r) = emit_expr(right, ctx)?;
                    let tmp = ctx.fresh("cat");
                    return Ok((
                        format!("{pl}{pr}int64_t {tmp} = klist_concat({l}, {r});\n"),
                        tmp,
                    ));
                }
                // String concat only when at least one side is known Str
                if lt == CTy::Str || rt == CTy::Str {
                    let (pl, l) = emit_expr(left, ctx)?;
                    let (pr, r) = emit_expr(right, ctx)?;
                    let l = coerce_expr(&l, &lt, &CTy::Str)?;
                    let r = coerce_expr(&r, &rt, &CTy::Str)?;
                    let tmp = ctx.fresh("scat");
                    return Ok((
                        format!("{pl}{pr}const char *{tmp} = kstr_concat({l}, {r});\n"),
                        tmp,
                    ));
                }
            }
            if matches!(*op, BinaryOp::Eq | BinaryOp::Ne)
                && (lt == CTy::Str || rt == CTy::Str)
            {
                let (pl, l) = emit_expr(left, ctx)?;
                let (pr, r) = emit_expr(right, ctx)?;
                let l = coerce_expr(&l, &lt, &CTy::Str)?;
                let r = coerce_expr(&r, &rt, &CTy::Str)?;
                let cmp = if *op == BinaryOp::Eq {
                    format!("(strcmp({l}, {r}) == 0)")
                } else {
                    format!("(strcmp({l}, {r}) != 0)")
                };
                return Ok((format!("{pl}{pr}"), cmp));
            }
            // Val == Val: strcmp if both strings at runtime, else i64
            if matches!(*op, BinaryOp::Eq | BinaryOp::Ne) && lt == CTy::Val && rt == CTy::Val {
                let (pl, l) = emit_expr(left, ctx)?;
                let (pr, r) = emit_expr(right, ctx)?;
                let cmp = if *op == BinaryOp::Eq {
                    format!("kval_eq({l}, {r})")
                } else {
                    format!("(!kval_eq({l}, {r}))")
                };
                return Ok((format!("{pl}{pr}"), cmp));
            }
            if *op == BinaryOp::Rem {
                let (pl, l) = emit_as_i64(left, ctx)?;
                let (pr, r) = emit_as_i64(right, ctx)?;
                return Ok((format!("{pl}{pr}"), format!("({l} % {r})")));
            }
            let want = promote_num(&lt, &rt).unwrap_or(CTy::I64);
            let (pl, l) = emit_as_num(left, ctx, &want)?;
            let (pr, r) = emit_as_num(right, ctx, &want)?;
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
            "assert" => {
                if args.len() != 1 {
                    return Err(KengaError::at("assert takes 1 arg", span.clone()));
                }
                let (p, e) = emit_as_i64(&args[0], ctx)?;
                Ok((format!("{p}k_assert({e});\n"), "0".into()))
            }
            "ord" => {
                if args.len() != 1 {
                    return Err(KengaError::at("ord takes 1 arg", span.clone()));
                }
                let (p, e) = emit_expr(&args[0], ctx)?;
                let ty = infer_expr(&args[0], ctx)?;
                let e = coerce_expr(&e, &ty, &CTy::Str)?;
                Ok((p, format!("k_ord({e})")))
            }
            "len" => {
                if args.len() != 1 {
                    return Err(KengaError::at("len takes 1 arg", span.clone()));
                }
                let (p, e) = emit_expr(&args[0], ctx)?;
                let ty = infer_expr(&args[0], ctx)?;
                match ty {
                    CTy::Str => Ok((p, format!("((int64_t)strlen({e}))"))),
                    CTy::Val => Ok((p, format!("kval_len({e})"))),
                    _ => {
                        let e = coerce_expr(&e, &ty, &CTy::List)?;
                        Ok((p, format!("klist_len({e})")))
                    }
                }
            }
            "round" => {
                if args.len() != 1 {
                    return Err(KengaError::at("round takes 1 arg", span.clone()));
                }
                let (p, e) = emit_as_num(&args[0], ctx, &CTy::F64)?;
                Ok((p, format!("((int64_t)llround({e}))")))
            }
            "push" => {
                if args.len() != 2 {
                    return Err(KengaError::at("push takes 2 args", span.clone()));
                }
                let (pl, l) = emit_expr(&args[0], ctx)?;
                let (pv, v) = emit_expr(&args[1], ctx)?;
                let lty = infer_expr(&args[0], ctx)?;
                let vty = infer_expr(&args[1], ctx)?;
                let l = coerce_expr(&l, &lty, &CTy::List)?;
                let v = wrap_as_val(&v, &vty);
                let tmp = ctx.fresh("push");
                Ok((
                    format!("{pl}{pv}int64_t {tmp} = {l};\nklist_push_val({tmp}, {v});\n"),
                    tmp,
                ))
            }
            other => {
                let mut pre = String::new();
                let mut parts = Vec::new();
                let param_tys = ctx.sigs.get(other).map(|(_, ps)| ps.clone());
                for (i, a) in args.iter().enumerate() {
                    let (p, e) = emit_expr(a, ctx)?;
                    let aty = infer_expr(a, ctx)?;
                    pre.push_str(&p);
                    let e = if let Some(ref ps) = param_tys {
                        if let Some(pty) = ps.get(i) {
                            coerce_expr(&e, &aty, pty)?
                        } else {
                            e
                        }
                    } else {
                        e
                    };
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

const RUNTIME: &str = r#"/* Generated by Kenga emit-c — tagged KVal runtime */
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <math.h>

enum { KV_I64 = 0, KV_STR = 1, KV_LIST = 2, KV_F64 = 3 };

typedef struct {
  int tag;
  union {
    int64_t i;
    double f;
    const char *s;
    int64_t list_id;
  } u;
} KVal;

typedef struct {
  KVal *data;
  size_t len;
  size_t cap;
} KListObj;

static KListObj *g_lists = NULL;
static size_t g_lists_len = 0;
static size_t g_lists_cap = 0;

static void k_die(const char *msg) {
  fprintf(stderr, "kenga: %s\n", msg);
  exit(1);
}

static KVal kval_i64(int64_t x) {
  KVal v; v.tag = KV_I64; v.u.i = x; return v;
}
static KVal kval_f64(double x) {
  KVal v; v.tag = KV_F64; v.u.f = x; return v;
}
static KVal kval_str(const char *s) {
  KVal v; v.tag = KV_STR; v.u.s = s ? s : ""; return v;
}
static KVal kval_list(int64_t id) {
  KVal v; v.tag = KV_LIST; v.u.list_id = id; return v;
}

static int64_t kval_as_i64(KVal v) {
  if (v.tag == KV_I64) return v.u.i;
  if (v.tag == KV_F64) return (int64_t)v.u.f;
  k_die("expected i64");
  return 0;
}
static double kval_as_f64(KVal v) {
  if (v.tag == KV_F64) return v.u.f;
  if (v.tag == KV_I64) return (double)v.u.i;
  k_die("expected f64");
  return 0.0;
}
static const char *kval_as_str(KVal v) {
  if (v.tag != KV_STR) k_die("expected str");
  return v.u.s;
}
static int64_t kval_as_list(KVal v) {
  if (v.tag != KV_LIST) k_die("expected list");
  return v.u.list_id;
}

static KListObj *klist_obj(int64_t id) {
  if (id < 0 || (size_t)id >= g_lists_len) k_die("bad list handle");
  return &g_lists[id];
}

static int64_t klist_new(void) {
  if (g_lists_len + 1 > g_lists_cap) {
    size_t ncap = g_lists_cap ? g_lists_cap * 2 : 8;
    KListObj *nd = (KListObj *)realloc(g_lists, ncap * sizeof(KListObj));
    if (!nd) k_die("oom");
    g_lists = nd;
    g_lists_cap = ncap;
  }
  KListObj *o = &g_lists[g_lists_len];
  o->data = NULL;
  o->len = 0;
  o->cap = 0;
  return (int64_t)g_lists_len++;
}

static void klist_push_val(int64_t id, KVal v) {
  KListObj *l = klist_obj(id);
  if (l->len + 1 > l->cap) {
    size_t ncap = l->cap ? l->cap * 2 : 8;
    KVal *nd = (KVal *)realloc(l->data, ncap * sizeof(KVal));
    if (!nd) k_die("oom");
    l->data = nd;
    l->cap = ncap;
  }
  l->data[l->len++] = v;
}

static KVal klist_get_val(int64_t id, int64_t i) {
  KListObj *l = klist_obj(id);
  if (i < 0 || (size_t)i >= l->len) k_die("index oob");
  return l->data[i];
}

static void klist_set_val(int64_t id, int64_t i, KVal v) {
  KListObj *l = klist_obj(id);
  if (i < 0 || (size_t)i >= l->len) k_die("index oob");
  l->data[i] = v;
}

static int64_t klist_len(int64_t id) {
  return (int64_t)klist_obj(id)->len;
}

static int64_t klist_concat(int64_t a, int64_t b) {
  int64_t o = klist_new();
  KListObj *la = klist_obj(a);
  KListObj *lb = klist_obj(b);
  for (size_t i = 0; i < la->len; i++) klist_push_val(o, la->data[i]);
  for (size_t i = 0; i < lb->len; i++) klist_push_val(o, lb->data[i]);
  return o;
}

static const char *kstr_from_i64(int64_t x) {
  char *o = (char *)malloc(32);
  if (!o) k_die("oom");
  snprintf(o, 32, "%lld", (long long)x);
  return o;
}

static const char *kstr_from_f64(double x) {
  char *o = (char *)malloc(64);
  if (!o) k_die("oom");
  snprintf(o, 64, "%g", x);
  return o;
}

static const char *kstr_concat(const char *a, const char *b) {
  if (!a) a = "";
  if (!b) b = "";
  size_t na = strlen(a), nb = strlen(b);
  char *o = (char *)malloc(na + nb + 1);
  if (!o) k_die("oom");
  memcpy(o, a, na);
  memcpy(o + na, b, nb);
  o[na + nb] = 0;
  return o;
}

static int64_t kval_eq(KVal a, KVal b) {
  if (a.tag == KV_I64 && b.tag == KV_F64) return (double)a.u.i == b.u.f;
  if (a.tag == KV_F64 && b.tag == KV_I64) return a.u.f == (double)b.u.i;
  if (a.tag != b.tag) return 0;
  if (a.tag == KV_I64) return a.u.i == b.u.i;
  if (a.tag == KV_F64) return a.u.f == b.u.f;
  if (a.tag == KV_STR) return strcmp(a.u.s ? a.u.s : "", b.u.s ? b.u.s : "") == 0;
  if (a.tag == KV_LIST) return a.u.list_id == b.u.list_id;
  return 0;
}

static KVal kstr_index_val(const char *s, int64_t i) {
  if (!s) k_die("str index on null");
  size_t n = strlen(s);
  if (i < 0 || (size_t)i >= n) k_die("str index oob");
  char *o = (char *)malloc(2);
  if (!o) k_die("oom");
  o[0] = s[i];
  o[1] = 0;
  return kval_str(o);
}

static KVal kval_index(KVal v, int64_t i) {
  if (v.tag == KV_LIST) return klist_get_val(v.u.list_id, i);
  if (v.tag == KV_STR) return kstr_index_val(v.u.s, i);
  k_die("index on non-list/str");
  return kval_i64(0);
}

static int64_t kval_len(KVal v) {
  if (v.tag == KV_LIST) return klist_len(v.u.list_id);
  if (v.tag == KV_STR) return (int64_t)strlen(v.u.s ? v.u.s : "");
  k_die("len on non-list/str");
  return 0;
}

static void kenga_println_i64(int64_t v) { printf("%lld\n", (long long)v); }
static void kenga_println_f64(double v) { printf("%g\n", v); }
static void kenga_println_str(const char *s) { puts(s ? s : ""); }

static void kenga_print_val(KVal v);

static void kenga_println_list(int64_t id) {
  KListObj *l = klist_obj(id);
  putchar('[');
  for (size_t i = 0; i < l->len; i++) {
    if (i) printf(", ");
    kenga_print_val(l->data[i]);
  }
  puts("]");
}

static void kenga_print_val(KVal v) {
  if (v.tag == KV_I64) {
    printf("%lld", (long long)v.u.i);
  } else if (v.tag == KV_F64) {
    printf("%g", v.u.f);
  } else if (v.tag == KV_STR) {
    printf("%s", v.u.s ? v.u.s : "");
  } else if (v.tag == KV_LIST) {
    KListObj *l = klist_obj(v.u.list_id);
    putchar('[');
    for (size_t i = 0; i < l->len; i++) {
      if (i) printf(", ");
      kenga_print_val(l->data[i]);
    }
    putchar(']');
  } else {
    k_die("bad value tag");
  }
}

static void kenga_println_val(KVal v) {
  kenga_print_val(v);
  putchar('\n');
}

static void k_assert(int64_t c) {
  if (!c) k_die("assert failed");
}
static int64_t k_ord(const char *s) {
  if (!s || !s[0]) return 0;
  return (unsigned char)s[0];
}

"#;
