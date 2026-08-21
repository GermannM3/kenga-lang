//! C99 backend: tagged KVal runtime (i64 / f64 / str / list handles), structs, for/while.

use std::collections::HashMap;

use crate::ast::{
    AssignTarget, BinaryOp, Block, Expr, Function, Item, MatchArm, MatchPattern, Program, Stmt, Type, UnaryOp,
};
use crate::error::{KengaError, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
enum CTy {
    I64,
    I32,
    I16,
    I8,
    U64,
    U32,
    U16,
    U8,
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
    emit_c_with_options(program, false)
}

pub fn emit_c_freestanding(program: &Program) -> Result<String> {
    emit_c_with_options(program, true)
}

fn emit_c_with_options(program: &Program, freestanding: bool) -> Result<String> {
    let mut structs: HashMap<String, StructInfo> = HashMap::new();
    for item in &program.items {
        if let Item::Struct(s) = item {
            let fields = s
                .fields
                .iter()
                .map(|p| Ok((p.name.clone(), map_type(&p.ty, &structs)?)))
                .collect::<Result<Vec<_>>>()?;
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
        } else if let Item::Impl(i) = item {
            for f in &i.methods {
                let ret = map_type(&f.ret, &structs)?;
                let params = f.params.iter().map(|p| map_type(&p.ty, &structs)).collect::<Result<Vec<_>>>()?;
                sigs.insert(f.name.clone(), (ret, params));
            }
        }
        // @intrinsic fn foo(...) -> t;  — extern/FFI declaration. Feed its
        // signature into sigs so calls get proper arg coercion, and emit a
        // C prototype below.
        if let Item::Intrinsic(i) = item {
            let ret = map_type(&i.ret, &structs)?;
            let params = i
                .params
                .iter()
                .map(|p| map_type(&p.ty, &structs))
                .collect::<Result<Vec<_>>>()?;
            sigs.insert(i.name.clone(), (ret, params));
        }
    }
    if !has_main {
        return Err(KengaError::new("emit-c requires fn main()", None));
    }

    let mut body = String::new();
    for item in &program.items {
        if let Item::Const(c) = item {
            let (_, value) = emit_const_expr(&c.value)?;
            body.push_str(&format!("#define {} {}\n", c_ident(&c.name), value));
        }
    }
    body.push('\n');
    for item in &program.items {
        if let Item::Enum(e) = item {
            body.push_str(&format!("typedef int64_t {};\n", c_ident(&e.name)));
            for (i, variant) in e.variants.iter().enumerate() {
                body.push_str(&format!("#define {}_{} {}\n", c_ident(&e.name), c_ident(&variant.name), i));
            }
            body.push('\n');
        }
    }
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
        } else if let Item::Impl(i) = item {
            for f in &i.methods {
                body.push_str(&emit_prototype(f, &structs)?);
                body.push_str(";\n");
            }
        }
        // FFI prototypes for user-declared @intrinsic fn (kernel externs).
        if let Item::Intrinsic(i) = item {
            if is_builtin_intrinsic(&i.name) {
                continue;
            }
            let f = Function {
                name: i.name.clone(),
                params: i.params.clone(),
                ret: i.ret.clone(),
                body: Block { stmts: vec![] },
                span: i.span.clone(),
            };
            body.push_str(&emit_prototype(&f, &structs)?);
            body.push_str(";\n");
        }
    }
    body.push('\n');

    for item in &program.items {
        if let Item::Function(f) = item {
            body.push_str(&emit_function(f, &structs, &sigs)?);
            body.push('\n');
        } else if let Item::Impl(i) = item {
            for f in &i.methods {
                body.push_str(&emit_function(f, &structs, &sigs)?);
                body.push('\n');
            }
        }
    }

    let mut out = String::new();
    if freestanding {
        out.push_str(RUNTIME_FS);
    } else {
        out.push_str(RUNTIME);
    }
    out.push_str(&body);
    Ok(out)
}

fn emit_const_expr(expr: &Expr) -> Result<(String, String)> {
    match expr {
        Expr::Int(n, _) => Ok(("i64".into(), n.to_string())),
        Expr::Float(n, _) => Ok(("f64".into(), c_float_lit(*n))),
        Expr::Bool(b, _) => Ok(("i64".into(), if *b { "1" } else { "0" }.into())),
        Expr::Str(s, _) => Ok(("str".into(), format!("\"{}\"", escape_c(s)))),
        Expr::Unary { op, expr, .. } => {
            let (_, v) = emit_const_expr(expr)?;
            Ok(("i64".into(), format!("{}{}", unary_c(op), v)))
        }
        Expr::Binary { left, op, right, .. } => {
            let (_, l) = emit_const_expr(left)?;
            let (_, r) = emit_const_expr(right)?;
            Ok(("i64".into(), format!("({} {} {})", l, binary_c(op), r)))
        }
        _ => Err(KengaError::new("const value must be compile-time literal", None)),
    }
}

fn unary_c(op: &UnaryOp) -> &'static str {
    match op { UnaryOp::Neg => "-", UnaryOp::Not => "!", UnaryOp::BitNot => "~" }
}

fn binary_c(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+", BinaryOp::Sub => "-", BinaryOp::Mul => "*",
        BinaryOp::Div => "/", BinaryOp::Rem => "%", BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=", BinaryOp::Lt => "<", BinaryOp::Le => "<=",
        BinaryOp::Gt => ">", BinaryOp::Ge => ">=", BinaryOp::And => "&&",
        BinaryOp::Or => "||", BinaryOp::BitAnd => "&", BinaryOp::BitOr => "|",
        BinaryOp::BitXor => "^", BinaryOp::Shl => "<<", BinaryOp::Shr => ">>",
    }
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
                match n.as_str() {
                    "u8" => CTy::U8,
                    "u16" => CTy::U16,
                    "u32" => CTy::U32,
                    "u64" => CTy::U64,
                    "i8" => CTy::I8,
                    "i16" => CTy::I16,
                    "i32" => CTy::I32,
                    _ => CTy::I64,
                }
            }
        }
    })
}

fn c_type(t: &CTy) -> String {
    match t {
        CTy::I64 | CTy::List => "int64_t".into(),
        CTy::I32 => "int32_t".into(),
        CTy::I16 => "int16_t".into(),
        CTy::I8 => "int8_t".into(),
        CTy::U64 => "uint64_t".into(),
        CTy::U32 => "uint32_t".into(),
        CTy::U16 => "uint16_t".into(),
        CTy::U8 => "uint8_t".into(),
        CTy::F64 => "double".into(),
        CTy::Str => "const char*".into(),
        CTy::Void => "void".into(),
        CTy::Struct(n) => struct_c_name(n),
        CTy::Val => "KVal".into(),
    }
}

fn is_numeric(t: &CTy) -> bool {
    matches!(t, CTy::I64 | CTy::I32 | CTy::I16 | CTy::I8 | CTy::U64 | CTy::U32 | CTy::U16 | CTy::U8 | CTy::F64)
}

fn promote_num(a: &CTy, b: &CTy) -> Option<CTy> {
    use CTy::*;
    match (a, b) {
        (F64, F64)
        | (F64, I64) | (I64, F64)
        | (F64, I32) | (I32, F64)
        | (F64, I16) | (I16, F64)
        | (F64, I8)  | (I8,  F64)
        | (F64, U64) | (U64, F64)
        | (F64, U32) | (U32, F64)
        | (F64, U16) | (U16, F64)
        | (F64, U8)  | (U8,  F64) => Some(F64),
        (I64, I64)
        | (I64, I32) | (I32, I64)
        | (I64, I16) | (I16, I64)
        | (I64, I8)  | (I8,  I64)
        | (I64, U64) | (U64, I64)
        | (I64, U32) | (U32, I64)
        | (I64, U16) | (U16, I64)
        | (I64, U8)  | (U8,  I64)
        | (I32, I32)
        | (I32, I16) | (I16, I32)
        | (I32, I8)  | (I8,  I32)
        | (I32, U32) | (U32, I32)
        | (I32, U16) | (U16, I32)
        | (I32, U8)  | (U8,  I32)
        | (U64, U64)
        | (U64, U32) | (U32, U64)
        | (U64, U16) | (U16, U64)
        | (U64, U8)  | (U8,  U64)
        | (U32, U32)
        | (U32, U16) | (U16, U32)
        | (U32, U8)  | (U8,  U32)
        | (I16, I16)
        | (I16, I8)  | (I8,  I16)
        | (I16, U16) | (U16, I16)
        | (I16, U8)  | (U8,  I16)
        | (U16, U16)
        | (U16, U8)  | (U8,  U16)
        | (I8, I8)
        | (I8, U8)   | (U8,  I8)
        | (U8, U8) => Some(I64),
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

/// Intrinsics that emit_c special-cases inline (mmio, asm, atomic, len/ord/assert,
/// print). They must NOT get a `k_<name>` prototype: some collide with static
/// helpers in the emitted runtime (k_ord, k_assert, kenga_println_*). Calls to
/// them never resolve to a real `k_<name>` symbol.
fn is_builtin_intrinsic(name: &str) -> bool {
    matches!(name,
        "len" | "ord" | "assert" | "print" | "println"
        | "mmio_read8" | "mmio_read16" | "mmio_read32" | "mmio_read64"
        | "mmio_write8" | "mmio_write16" | "mmio_write32" | "mmio_write64"
        | "asm_hlt" | "asm_cli" | "asm_sti" | "asm"
        | "asm_inb" | "asm_inw" | "asm_inl"
        | "asm_outb" | "asm_outw" | "asm_outl"
        | "atomic_load" | "atomic_store" | "atomic_cas" | "atomic_fence")
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
        CTy::I64
        | CTy::I32
        | CTy::I16
        | CTy::I8
        | CTy::U64
        | CTy::U32
        | CTy::U16
        | CTy::U8 => format!("kval_i64({expr})"),
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
    if is_numeric(from) && is_numeric(to) {
        return Ok(if matches!(to, CTy::F64) {
            format!("((double)({expr}))")
        } else {
            format!("((int64_t)({expr}))")
        });
    }
    Ok(match (from, to) {
        (CTy::Val, CTy::I64) => format!("kval_as_i64({expr})"),
        (CTy::Val, CTy::F64) => format!("kval_as_f64({expr})"),
        (CTy::Val, CTy::I32) | (CTy::Val, CTy::I16) | (CTy::Val, CTy::I8)
        | (CTy::Val, CTy::U64) | (CTy::Val, CTy::U32) | (CTy::Val, CTy::U16) | (CTy::Val, CTy::U8)
            => format!("((int64_t)kval_as_i64({expr}))"),
        (CTy::Val, CTy::Str) => format!("kval_as_str({expr})"),
        (CTy::Val, CTy::List) => format!("kval_as_list({expr})"),
        (CTy::I64, CTy::Val) => format!("kval_i64({expr})"),
        (CTy::F64, CTy::Val) => format!("kval_f64({expr})"),
        (CTy::I64, CTy::F64) => format!("((double)({expr}))"),
        (CTy::F64, CTy::I64) => format!("((int64_t)({expr}))"),
        (CTy::I64, CTy::Struct(_)) => format!("({}){{0}}", c_type(to)),
        (CTy::Struct(_), CTy::I64) => "0".to_string(),
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
        Stmt::Match { value, arms, .. } => emit_match(value, arms, indent, ctx),
        Stmt::Break(_) => Ok(format!("{pad}break;\n")),
        Stmt::Continue(_) => Ok(format!("{pad}continue;\n")),
    }
}

fn match_tag(path: &[String]) -> u64 {
    let name = path.last().map(String::as_str).unwrap_or("_");
    let mut h = 1469598103934665603u64;
    for b in name.bytes() { h ^= b as u64; h = h.wrapping_mul(1099511628211); }
    h & 0x7fff_ffff
}

fn emit_match(value: &Expr, arms: &[MatchArm], indent: usize, ctx: &mut EmitCtx<'_>) -> Result<String> {
    let pad = "  ".repeat(indent);
    let (pre, expr) = emit_expr(value, ctx)?;
    let ty = infer_expr(value, ctx)?;
    let val = wrap_as_val(&expr, &ty);
    let tmp = ctx.fresh("match");
    let mut out = format!("{}{}KVal {tmp} = {val};\n", indent_pre(&pre, indent), pad);
    for (i, arm) in arms.iter().enumerate() {
        let cond = match &arm.pattern {
            MatchPattern::Wildcard => "1".to_string(),
            MatchPattern::Literal(value) => format!("kval_as_i64({tmp}) == {value}"),
            MatchPattern::Variant { path, .. } => format!("kval_tag({tmp}) == {}", match_tag(path)),
        };
        out.push_str(&format!("{pad}{}if ({cond}) {{\n", if i == 0 { "" } else { "else " }));
        if let MatchPattern::Variant { bindings, .. } = &arm.pattern {
            for (slot, binding) in bindings.iter().enumerate() {
                ctx.vars.insert(binding.clone(), CTy::I64);
                out.push_str(&format!("{}int64_t {} = kval_payload_i64({tmp}, {});\n", "  ".repeat(indent + 1), c_ident(binding), slot));
            }
        }
        for st in &arm.body.stmts { out.push_str(&emit_stmt(st, indent + 1, ctx)?); }
        out.push_str(&format!("{pad}}}"));
    }
    out.push('\n');
    Ok(out)
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
    if let Expr::Range { start, end, step, .. } = iter {
        let (pa, a) = emit_expr(start, ctx)?;
        let (pb, b) = emit_expr(end, ctx)?;
        let (ps, step_expr) = if let Some(step) = step { emit_expr(step, ctx)? } else { (String::new(), "1".to_string()) };
        ctx.vars.insert(var.to_string(), CTy::I64);
        let mut s = format!(
            "{}{}{}{pad}for (int64_t {} = {a}; (({step_expr}) >= 0 ? {} < {b} : {} > {b}); {} += ({step_expr})) {{\n",
            indent_pre(&pa, indent),
            indent_pre(&pb, indent),
            indent_pre(&ps, indent),
            c_ident(var),
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
        Expr::Ident(name, span) => ctx.vars.get(name).cloned().or_else(|| {
            if name.chars().all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit()) {
                Some(CTy::I64)
            } else { None }
        }).ok_or_else(|| KengaError::at(format!("unknown variable '{name}'"), span.clone()))?,
        Expr::Index { .. } => CTy::Val,
        Expr::Unary { op, expr, .. } => {
            let t = infer_expr(expr, ctx)?;
            match op {
                UnaryOp::Neg if t == CTy::F64 => CTy::F64,
                UnaryOp::Neg if is_numeric(&t) => CTy::I64,
                UnaryOp::Not => CTy::I64,
                UnaryOp::BitNot if is_numeric(&t) => CTy::I64,
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
            if let Expr::Ident(root, _) = target.as_ref() {
                if root.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
                    return Ok(CTy::I64);
                }
            }
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
                // Method receivers are currently represented as opaque ABI
                // handles; keep lowering permissive until receiver typing is
                // wired through impl signatures.
                _ => CTy::I64,
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
        Expr::VariantLit { .. } => CTy::Val,
        Expr::Call { callee, .. } => {
            if callee == "Some" || callee == "None" {
                CTy::Val
            } else if callee == "len" {
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

fn emit_intrinsic_call(
    name: &str,
    args: &[Expr],
    span: &crate::error::Span,
    ctx: &mut EmitCtx<'_>,
) -> Result<(String, String)> {
    use std::fmt::Write;
    match name {
        "mmio_read8" | "mmio_read16" | "mmio_read32" | "mmio_read64" => {
            if args.len() != 1 {
                return Err(KengaError::at(
                    format!("{name} takes 1 argument (address)"),
                    span.clone(),
                ));
            }
            let (p, addr) = emit_as_i64(&args[0], ctx)?;
            let suffix = &name["mmio_read".len()..];
            let c_fn = format!("_k_mmio_r{suffix}");
            Ok((p, format!("((int64_t){c_fn}((uint64_t)({addr})))")))
        }
        "mmio_write8" | "mmio_write16" | "mmio_write32" | "mmio_write64" => {
            if args.len() != 2 {
                return Err(KengaError::at(
                    format!("{name} takes 2 arguments (address, value)"),
                    span.clone(),
                ));
            }
            let (p1, addr) = emit_as_i64(&args[0], ctx)?;
            let (p2, val) = emit_as_i64(&args[1], ctx)?;
            let suffix = &name["mmio_write".len()..];
            let c_fn = format!("_k_mmio_w{suffix}");
            Ok((
                format!("{p1}{p2}(void){c_fn}((uint64_t)({addr}), (uint{suffix}_t)({val}));\n"),
                "0".into(),
            ))
        }
        "asm_hlt" => {
            if !args.is_empty() {
                return Err(KengaError::at("asm_hlt takes 0 args", span.clone()));
            }
            Ok(("__asm__ __volatile__(\"hlt\");\n".into(), "0".into()))
        }
        "asm_cli" => {
            if !args.is_empty() {
                return Err(KengaError::at("asm_cli takes 0 args", span.clone()));
            }
            Ok(("__asm__ __volatile__(\"cli\");\n".into(), "0".into()))
        }
        "asm_sti" => {
            if !args.is_empty() {
                return Err(KengaError::at("asm_sti takes 0 args", span.clone()));
            }
            Ok(("__asm__ __volatile__(\"sti\");\n".into(), "0".into()))
        }
        "asm" => {
            if args.len() != 1 && args.len() != 2 {
                return Err(KengaError::at(
                    "asm(code) or asm(arch, code)",
                    span.clone(),
                ));
            }
            let code_idx = if args.len() == 1 { 0 } else { 1 };
            let (p, code) = emit_expr(&args[code_idx], ctx)?;
            let ty = infer_expr(&args[code_idx], ctx)?;
            let code = coerce_expr(&code, &ty, &CTy::Str)?;
            let mut out = String::new();
            writeln!(out, "{p}__asm__ __volatile__({code});").unwrap();
            Ok((out, "0".into()))
        }
        "asm_inb" | "asm_inw" | "asm_inl" => {
            if args.len() != 1 {
                return Err(KengaError::at(
                    "asm_inb/asm_inw/asm_inl take 1 arg (port)",
                    span.clone(),
                ));
            }
            let (p, port) = emit_as_i64(&args[0], ctx)?;
            let (var_ty, size) = match name {
                "asm_inb" => ("uint8_t", "b"),
                "asm_inw" => ("uint16_t", "w"),
                _ => ("uint32_t", "l"),
            };
            let mut out = p;
            out += &format!("{var_ty} _k_port_in;\n");
            out += &format!(
                "__asm__ __volatile__(\"in{size} %1, %0\" : \"=a\"(_k_port_in) : \"Nd\"((uint16_t)({port})));\n"
            );
            Ok((out, "((int64_t)_k_port_in)".into()))
        }
        "asm_outb" | "asm_outw" | "asm_outl" => {
            if args.len() != 2 {
                return Err(KengaError::at(
                    "asm_outb/asm_outw/asm_outl take 2 args (port, val)",
                    span.clone(),
                ));
            }
            let (p1, port) = emit_as_i64(&args[0], ctx)?;
            let (p2, val) = emit_as_i64(&args[1], ctx)?;
            let (val_ty, size) = match name {
                "asm_outb" => ("uint8_t", "b"),
                "asm_outw" => ("uint16_t", "w"),
                _ => ("uint32_t", "l"),
            };
            let mut out = format!("{p1}{p2}");
            out += &format!(
                "__asm__ __volatile__(\"out{size} %0, %1\" : : \"a\"(({val_ty})({val})), \"Nd\"((uint16_t)({port})));\n"
            );
            Ok((out, "0".into()))
        }
        "atomic_load" => {
            if args.len() != 1 {
                return Err(KengaError::at("atomic_load takes 1 arg (ptr)", span.clone()));
            }
            let (p, ptr) = emit_as_i64(&args[0], ctx)?;
            Ok((
                p,
                format!("((int64_t)__atomic_load_n((volatile int64_t*)(uintptr_t){ptr}, __ATOMIC_SEQ_CST))"),
            ))
        }
        "atomic_store" => {
            if args.len() != 2 {
                return Err(KengaError::at(
                    "atomic_store takes 2 args (ptr, val)",
                    span.clone(),
                ));
            }
            let (p1, ptr) = emit_as_i64(&args[0], ctx)?;
            let (p2, val) = emit_as_i64(&args[1], ctx)?;
            Ok((
                format!(
                    "{p1}{p2}__atomic_store_n((volatile int64_t*)(uintptr_t){ptr}, (int64_t){val}, __ATOMIC_SEQ_CST);\n"
                ),
                "0".into(),
            ))
        }
        "atomic_cas" => {
            if args.len() != 3 {
                return Err(KengaError::at(
                    "atomic_cas takes 3 args (ptr, expected, desired)",
                    span.clone(),
                ));
            }
            let (p1, ptr) = emit_as_i64(&args[0], ctx)?;
            let (p2, exp) = emit_as_i64(&args[1], ctx)?;
            let (p3, des) = emit_as_i64(&args[2], ctx)?;
            let tmp = ctx.fresh("cas");
            let pre = format!(
                "{p1}{p2}{p3}int64_t {tmp} = (int64_t){exp};\n(int64_t)__atomic_compare_exchange_n((volatile int64_t*)(uintptr_t){ptr}, &{tmp}, (int64_t){des}, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);\n"
            );
            Ok((pre, "0".into()))
        }
        "atomic_fence" => {
            if !args.is_empty() {
                return Err(KengaError::at("atomic_fence takes 0 args", span.clone()));
            }
            Ok(("__atomic_thread_fence(__ATOMIC_SEQ_CST);\n".into(), "0".into()))
        }
        _ => unreachable!(),
    }
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
            if let Expr::Ident(root, _) = target.as_ref() {
                if root.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
                    return Ok((String::new(), "0".to_string()));
                }
            }
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
                let Some(val) = fields.iter().find(|(n, _)| n == fname).map(|(_, e)| e) else {
                    // C designated initializers zero omitted fields, which is
                    // the intended default for optional desktop state.
                    continue;
                };
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
        Expr::VariantLit { path, fields, span } => {
            let tag = match_tag(path);
            let tmp = ctx.fresh("variant");
            let mut pre = String::new();
            pre.push_str(&format!("KVal {tmp} = kval_i64({tag}); {tmp}.tag = {tag};\n"));
            for (i, (_, value)) in fields.iter().enumerate().take(4) {
                let (p, e) = emit_expr(value, ctx)?;
                let ty = infer_expr(value, ctx)?;
                pre.push_str(&p);
                pre.push_str(&format!("{tmp}.u.payload[{i}] = kval_as_i64({});\n", wrap_as_val(&e, &ty)));
            }
            Ok((pre, tmp))
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
                UnaryOp::BitNot => "~",
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
                BinaryOp::BitAnd => "&",
                BinaryOp::BitOr => "|",
                BinaryOp::BitXor => "^",
                BinaryOp::Shl => "<<",
                BinaryOp::Shr => ">>",
            };
            Ok((format!("{pl}{pr}"), format!("({l} {o} {r})")))
        }
        Expr::Call { callee, args, span } => match callee.as_str() {
            "Some" => {
                if args.len() != 1 { return Err(KengaError::at("Some takes 1 argument", span.clone())); }
                let (p, e) = emit_expr(&args[0], ctx)?;
                let ty = infer_expr(&args[0], ctx)?;
                Ok((p, wrap_as_val(&e, &ty)))
            }
            "None" => {
                if !args.is_empty() { return Err(KengaError::at("None takes no arguments", span.clone())); }
                Ok((String::new(), "kval_i64(0)".into()))
            }
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
            "mmio_read8"  | "mmio_read16" | "mmio_read32" | "mmio_read64"
          | "mmio_write8" | "mmio_write16"| "mmio_write32"| "mmio_write64"
          | "asm_hlt" | "asm_cli" | "asm_sti"
          | "asm_inb" | "asm_inw" | "asm_inl"
          | "asm_outb" | "asm_outw" | "asm_outl"
          | "atomic_load" | "atomic_store" | "atomic_cas" | "atomic_fence"
          | "asm" => {
                emit_intrinsic_call(callee.as_str(), &args, span, ctx)
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
    int64_t payload[4];
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
static int64_t kval_tag(KVal v) { return (int64_t)v.tag; }
static int64_t kval_payload_i64(KVal v, int64_t slot) {
  return (slot >= 0 && slot < 4) ? v.u.payload[slot] : 0;
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

const RUNTIME_FS: &str = r#"/* Generated by Kenga emit-c -- freestanding (no libc / no CRT) */
#include <stdint.h>
#include <stddef.h>

#ifndef __cplusplus
typedef _Bool bool;
#define true  1
#define false 0
#endif

/* ---- freestanding libc replacements ---- */
static inline void* _k_memcpy(void* d, const void* s, size_t n) {
  unsigned char*       _d = (unsigned char*)d;
  const unsigned char* _s = (const unsigned char*)s;
  for (size_t _i = 0; _i < n; _i++) _d[_i] = _s[_i];
  return d;
}
static inline void* _k_memset(void* d, int c, size_t n) {
  unsigned char* _d = (unsigned char*)d;
  unsigned char  _v = (unsigned char)c;
  for (size_t _i = 0; _i < n; _i++) _d[_i] = _v;
  return d;
}
static inline int _k_memcmp(const void* a, const void* b, size_t n) {
  const unsigned char* _pa = (const unsigned char*)a;
  const unsigned char* _pb = (const unsigned char*)b;
  for (size_t _i = 0; _i < n; _i++) {
    if (_pa[_i] != _pb[_i]) return (int)_pa[_i] - (int)_pb[_i];
  }
  return 0;
}
static inline size_t _k_strlen(const char* s) {
  size_t n = 0;
  if (!s) return 0;
  while (s[n]) n++;
  return n;
}
static inline int _k_strcmp(const char* a, const char* b) {
  if (!a) a = "";
  if (!b) b = "";
  while (*a && *a == *b) { a++; b++; }
  return (int)(unsigned char)*a - (int)(unsigned char)*b;
}
#define memcpy _k_memcpy
#define memset _k_memset
#define memcmp _k_memcmp
#define strlen _k_strlen
#define strcmp _k_strcmp

static inline void k_die(const char* /*msg*/) {
  for (;;) {
    __asm__ __volatile__("cli; hlt");
  }
}
static inline void abort(void) { k_die("abort"); }
static inline void exit(int x) { (void)x; k_die("exit"); }

/* ---- mmio intrinsics (volatile, type-safe) ---- */
#define K_MMIO_R(T, a)   (*(volatile T*)(uintptr_t)(a))
#define K_MMIO_W(T, a, v) (*(volatile T*)(uintptr_t)(a) = (T)(v))
static inline uint8_t  _k_mmio_r8 (uint64_t a) { return K_MMIO_R(uint8_t,  a); }
static inline uint16_t _k_mmio_r16(uint64_t a) { return K_MMIO_R(uint16_t, a); }
static inline uint32_t _k_mmio_r32(uint64_t a) { return K_MMIO_R(uint32_t, a); }
static inline uint64_t _k_mmio_r64(uint64_t a) { return K_MMIO_R(uint64_t, a); }
static inline void _k_mmio_w8 (uint64_t a, uint8_t  v) { K_MMIO_W(uint8_t,  a, v); }
static inline void _k_mmio_w16(uint64_t a, uint16_t v) { K_MMIO_W(uint16_t, a, v); }
static inline void _k_mmio_w32(uint64_t a, uint32_t v) { K_MMIO_W(uint32_t, a, v); }
static inline void _k_mmio_w64(uint64_t a, uint64_t v) { K_MMIO_W(uint64_t, a, v); }

/* ---- atomic primitives (GCC / clang builtins) ---- */
#define K_ATOMIC_LOAD(p)       __atomic_load_n((p), __ATOMIC_SEQ_CST)
#define K_ATOMIC_STORE(p, v)   __atomic_store_n((p), (v), __ATOMIC_SEQ_CST)
#define K_ATOMIC_CAS(p, e, d)  __atomic_compare_exchange_n((p), &(e), (d), 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST)
#define K_ATOMIC_FENCE()       __atomic_thread_fence(__ATOMIC_SEQ_CST)

/* ---- tagged KVal runtime ---- */
enum { KV_I64 = 0, KV_STR = 1, KV_LIST = 2, KV_F64 = 3 };

typedef struct {
  int tag;
  union {
    int64_t i;
    double  f;
    const char *s;
    int64_t list_id;
    int64_t payload[4];
  } u;
} KVal;

typedef struct {
  KVal  *data;
  size_t len;
  size_t cap;
} KListObj;

static KListObj *g_lists = NULL;
static size_t    g_lists_len = 0;
static size_t    g_lists_cap = 0;

static KVal kval_i64(int64_t x)     { KVal v; v.tag = KV_I64; v.u.i = x; return v; }
static int64_t kval_tag(KVal v)     { return (int64_t)v.tag; }
static int64_t kval_payload_i64(KVal v, int64_t slot) { return (slot >= 0 && slot < 4) ? v.u.payload[slot] : 0; }
static KVal kval_f64(double x)      { KVal v; v.tag = KV_F64; v.u.f = x; return v; }
static KVal kval_str(const char *s) { KVal v; v.tag = KV_STR; v.u.s = s ? s : ""; return v; }
static KVal kval_list(int64_t id)   { KVal v; v.tag = KV_LIST; v.u.list_id = id; return v; }

static int64_t kval_as_i64(KVal v) {
  if (v.tag == KV_I64) return v.u.i;
  if (v.tag == KV_F64) return (int64_t)v.u.f;
  k_die("expected i64"); return 0;
}
static double kval_as_f64(KVal v) {
  if (v.tag == KV_F64) return v.u.f;
  if (v.tag == KV_I64) return (double)v.u.i;
  k_die("expected f64"); return 0.0;
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

static void* _k_arena_alloc(size_t n) {
  /* Allocation hook order (kernel-friendly):
   *   1) weak kf_alloc(n)  — kernel provides its own allocator (kmalloc/buddy). Declare in kf_rt.h.
   *   2) weak kf_libc_malloc(n) — optional libc fallback (hosted toolchains only).
   *   3) k_die("oom") — last resort. In real kernels kf_alloc must be provided.
   * NOTE: do not use __builtin_malloc here — clang forbids taking its address.
   */
  extern void* kf_alloc(size_t) __attribute__((weak));
  if (&kf_alloc) {
    void* p = kf_alloc(n);
    if (p) return p;
  }
  extern void* kf_libc_malloc(size_t) __attribute__((weak));
  if (&kf_libc_malloc) {
    void* p = kf_libc_malloc(n);
    if (p) return p;
  }
  k_die("oom");
  return NULL;
}

static int64_t klist_new(void) {
  if (g_lists_len + 1 > g_lists_cap) {
    size_t ncap = g_lists_cap ? g_lists_cap * 2 : 8;
    KListObj *nd = (KListObj *)_k_arena_alloc(ncap * sizeof(KListObj));
    if (g_lists) _k_memcpy(nd, g_lists, g_lists_len * sizeof(KListObj));
    g_lists = nd;
    g_lists_cap = ncap;
  }
  KListObj *o = &g_lists[g_lists_len];
  o->data = NULL; o->len = 0; o->cap = 0;
  return (int64_t)g_lists_len++;
}

static void klist_push_val(int64_t id, KVal v) {
  KListObj *l = klist_obj(id);
  if (l->len + 1 > l->cap) {
    size_t ncap = l->cap ? l->cap * 2 : 8;
    KVal *nd = (KVal *)_k_arena_alloc(ncap * sizeof(KVal));
    if (l->data) _k_memcpy(nd, l->data, l->len * sizeof(KVal));
    l->data = nd; l->cap = ncap;
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
static int64_t klist_len(int64_t id) { return (int64_t)klist_obj(id)->len; }

static int64_t klist_concat(int64_t a, int64_t b) {
  int64_t o = klist_new();
  KListObj *la = klist_obj(a);
  KListObj *lb = klist_obj(b);
  for (size_t i = 0; i < la->len; i++) klist_push_val(o, la->data[i]);
  for (size_t i = 0; i < lb->len; i++) klist_push_val(o, lb->data[i]);
  return o;
}

static const char *kstr_from_i64(int64_t x) {
  char *o = (char *)_k_arena_alloc(32);
  /* minimal itoa (decimal, signed) */
  int neg = 0; char *p = o + 22; *p = 0;
  uint64_t u = (uint64_t)x;
  if (x < 0) { neg = 1; u = (uint64_t)(-(x + 1)) + 1ULL; }
  if (u == 0) { *--p = '0'; }
  else { while (u) { *--p = (char)('0' + (u % 10)); u /= 10; } }
  if (neg) *--p = '-';
  _k_memcpy(o, p, (size_t)(o + 22 - p + 1));
  return o;
}
static const char *kstr_from_f64(double x) { return kstr_from_i64((int64_t)x); }

static const char *kstr_concat(const char *a, const char *b) {
  if (!a) a = "";
  if (!b) b = "";
  size_t na = _k_strlen(a), nb = _k_strlen(b);
  char *o = (char *)_k_arena_alloc(na + nb + 1);
  _k_memcpy(o, a, na);
  _k_memcpy(o + na, b, nb);
  o[na + nb] = 0;
  return o;
}

static int64_t kval_eq(KVal a, KVal b) {
  if (a.tag == KV_I64 && b.tag == KV_F64) return (double)a.u.i == b.u.f;
  if (a.tag == KV_F64 && b.tag == KV_I64) return a.u.f == (double)b.u.i;
  if (a.tag != b.tag) return 0;
  if (a.tag == KV_I64) return a.u.i == b.u.i;
  if (a.tag == KV_F64) return a.u.f == b.u.f;
  if (a.tag == KV_STR) return _k_strcmp(a.u.s ? a.u.s : "", b.u.s ? b.u.s : "") == 0;
  if (a.tag == KV_LIST) return a.u.list_id == b.u.list_id;
  return 0;
}

static KVal kstr_index_val(const char *s, int64_t i) {
  if (!s) k_die("str index on null");
  size_t n = _k_strlen(s);
  if (i < 0 || (size_t)i >= n) k_die("str index oob");
  char *o = (char *)_k_arena_alloc(2);
  o[0] = s[i]; o[1] = 0;
  return kval_str(o);
}

static KVal kval_index(KVal v, int64_t i) {
  if (v.tag == KV_LIST) return klist_get_val(v.u.list_id, i);
  if (v.tag == KV_STR)  return kstr_index_val(v.u.s, i);
  k_die("index on non-list/str");
  return kval_i64(0);
}

static int64_t kval_len(KVal v) {
  if (v.tag == KV_LIST) return klist_len(v.u.list_id);
  if (v.tag == KV_STR)  return (int64_t)_k_strlen(v.u.s ? v.u.s : "");
  k_die("len on non-list/str");
  return 0;
}

/* freestanding stubs for kenga_print* (no-op; use kputs() externally) */
static void kenga_println_i64(int64_t /*v*/) { }
static void kenga_println_f64(double /*v*/) { }
static void kenga_println_str(const char * /*s*/) { }
static void kenga_print_val(KVal /*v*/) { }
static void kenga_println_val(KVal /*v*/) { }
static void kenga_println_list(int64_t /*id*/) { }

static void k_assert(int64_t c) { if (!c) k_die("assert failed"); }
static int64_t k_ord(const char *s) {
  if (!s || !s[0]) return 0;
  return (unsigned char)s[0];
}

"#;
