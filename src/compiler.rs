use crate::ast::*;
use crate::bytecode::{FunctionChunk, Module, Op, Value};
use crate::error::{KengaError, Result};

struct LoopCtx {
    break_patches: Vec<usize>,
    continue_patches: Vec<usize>,
    continue_target: usize,
}

pub fn compile(program: &Program) -> Result<Module> {
    compile_with_options(program, true)
}

pub fn compile_with_options(program: &Program, require_main: bool) -> Result<Module> {
    let mut module = Module::new();

    module.intrinsics.insert("print".into(), 1);
    module.intrinsics.insert("println".into(), 1);
    module.intrinsics.insert("tensor".into(), 0);
    module.intrinsics.insert("now_ms".into(), 0);
    module.intrinsics.insert("sweep".into(), 0);
    module.intrinsics.insert("len".into(), 1);
    module.intrinsics.insert("push".into(), 2);
    module.intrinsics.insert("assert".into(), 1);
    module.intrinsics.insert("typeof".into(), 1);
    module.intrinsics.insert("listen".into(), 2);
    module.intrinsics.insert("emit".into(), 2);
    module.intrinsics.insert("pump".into(), 1);
    module.intrinsics.insert("pending".into(), 0);
    module.intrinsics.insert("sleep_ms".into(), 1);
    module.intrinsics.insert("memory".into(), 0);
    module.intrinsics.insert("memory_config".into(), 3);
    module.intrinsics.insert("remember".into(), 3);
    module.intrinsics.insert("remember_next".into(), 4);
    module.intrinsics.insert("surprise".into(), 2);
    module.intrinsics.insert("foresee".into(), 2);
    module.intrinsics.insert("predict".into(), 2);
    module.intrinsics.insert("learn".into(), 3);
    module.intrinsics.insert("unroll".into(), 3);
    module.intrinsics.insert("foresee_n".into(), 3);
    module.intrinsics.insert("consolidate".into(), 1);
    module.intrinsics.insert("recall".into(), 3);
    module.intrinsics.insert("mem_stats".into(), 1);

    for item in &program.items {
        if let Item::Struct(s) = item {
            let fields: Vec<_> = s.fields.iter().map(|f| f.name.clone()).collect();
            module.structs.insert(s.name.clone(), fields);
        }
    }

    let mut handler_i = 0usize;
    for item in &program.items {
        match item {
            Item::Intrinsic(i) => {
                module.intrinsics.insert(i.name.clone(), i.params.len());
            }
            Item::Struct(_) => {}
            Item::EventHandler(h) => {
                let fname = format!("__on_{}_{}", sanitize(&h.event), handler_i);
                handler_i += 1;
                let f = Function {
                    name: fname.clone(),
                    params: h.params.clone(),
                    ret: Type::Void,
                    body: h.body.clone(),
                    span: h.span.clone(),
                };
                let chunk = compile_function(&f)?;
                module.functions.insert(fname.clone(), chunk);
                module.handlers.push((h.event.clone(), fname));
            }
            Item::Function(f) => {
                let chunk = compile_function(f)?;
                if module.functions.contains_key(&f.name) {
                    return Err(KengaError::at(
                        format!("duplicate function '{}'", f.name),
                        f.span.clone(),
                    ));
                }
                module.functions.insert(f.name.clone(), chunk);
            }
        }
    }

    if require_main && !module.functions.contains_key("main") {
        return Err(KengaError::new("program must define fn main()", None));
    }

    Ok(module)
}

fn compile_function(f: &Function) -> Result<FunctionChunk> {
    let mut ops = Vec::new();
    let mut loops = Vec::new();
    for p in f.params.iter().rev() {
        ops.push(Op::Store {
            name: p.name.clone(),
            ttl_ms: None,
        });
    }
    compile_block(&f.body, &mut ops, &mut loops)?;
    ops.push(Op::Const(Value::Nil));
    ops.push(Op::Return);
    Ok(FunctionChunk {
        name: f.name.clone(),
        arity: f.params.len(),
        ops,
    })
}

fn compile_block(
    block: &Block,
    ops: &mut Vec<Op>,
    loops: &mut Vec<LoopCtx>,
) -> Result<()> {
    for stmt in &block.stmts {
        compile_stmt(stmt, ops, loops)?;
    }
    Ok(())
}

fn compile_stmt(
    stmt: &Stmt,
    ops: &mut Vec<Op>,
    loops: &mut Vec<LoopCtx>,
) -> Result<()> {
    match stmt {
        Stmt::Let {
            name,
            ttl_ms,
            value,
            ..
        } => {
            compile_expr(value, ops)?;
            ops.push(Op::Store {
                name: name.clone(),
                ttl_ms: *ttl_ms,
            });
        }
        Stmt::Assign { target, value, .. } => {
            match target {
                AssignTarget::Name(name) => {
                    compile_expr(value, ops)?;
                    ops.push(Op::Store {
                        name: name.clone(),
                        ttl_ms: None,
                    });
                }
                AssignTarget::Index { target, index } => {
                    compile_expr(target, ops)?;
                    compile_expr(index, ops)?;
                    compile_expr(value, ops)?;
                    ops.push(Op::SetIndex);
                    if let Expr::Ident(name, _) = target {
                        ops.push(Op::Store {
                            name: name.clone(),
                            ttl_ms: None,
                        });
                    } else {
                        ops.push(Op::Pop);
                    }
                }
                AssignTarget::Field { target, field } => {
                    compile_expr(target, ops)?;
                    compile_expr(value, ops)?;
                    ops.push(Op::SetField(field.clone()));
                    if let Expr::Ident(name, _) = target {
                        ops.push(Op::Store {
                            name: name.clone(),
                            ttl_ms: None,
                        });
                    } else {
                        ops.push(Op::Pop);
                    }
                }
            }
        }
        Stmt::Expr { expr, .. } => {
            compile_expr(expr, ops)?;
            ops.push(Op::Pop);
        }
        Stmt::Return { value, .. } => {
            match value {
                Some(e) => compile_expr(e, ops)?,
                None => ops.push(Op::Const(Value::Nil)),
            }
            ops.push(Op::Return);
        }
        Stmt::If {
            cond,
            then_block,
            else_block,
            ..
        } => {
            compile_expr(cond, ops)?;
            let jf = ops.len();
            ops.push(Op::JumpIfFalse(0));
            compile_block(then_block, ops, loops)?;
            if let Some(else_b) = else_block {
                let j = ops.len();
                ops.push(Op::Jump(0));
                let else_start = ops.len();
                if let Op::JumpIfFalse(ref mut t) = ops[jf] {
                    *t = else_start;
                }
                compile_block(else_b, ops, loops)?;
                let after = ops.len();
                if let Op::Jump(ref mut t) = ops[j] {
                    *t = after;
                }
            } else {
                let after = ops.len();
                if let Op::JumpIfFalse(ref mut t) = ops[jf] {
                    *t = after;
                }
            }
        }
        Stmt::While { cond, body, .. } => {
            let loop_start = ops.len();
            loops.push(LoopCtx {
                break_patches: Vec::new(),
                continue_patches: Vec::new(),
                continue_target: loop_start,
            });
            compile_expr(cond, ops)?;
            let jf = ops.len();
            ops.push(Op::JumpIfFalse(0));
            compile_block(body, ops, loops)?;
            ops.push(Op::Jump(loop_start));
            let after = ops.len();
            if let Op::JumpIfFalse(ref mut t) = ops[jf] {
                *t = after;
            }
            patch_loop(ops, loops.pop().unwrap(), after, loop_start);
        }
        Stmt::For {
            var,
            iter,
            body,
            span,
        } => {
            // Desugar:
            // let #it = iter;
            // if range: for i in start..end
            // if list: for x in list via index
            let it = format!("#it_{}", span.line);
            let idx = format!("#idx_{}", span.line);
            compile_expr(iter, ops)?;
            ops.push(Op::Store {
                name: it.clone(),
                ttl_ms: None,
            });

            // Unified: convert range/list to index loop
            // #idx = 0
            ops.push(Op::Const(Value::I64(0)));
            ops.push(Op::Store {
                name: idx.clone(),
                ttl_ms: None,
            });

            let loop_start = ops.len();
            loops.push(LoopCtx {
                break_patches: Vec::new(),
                continue_patches: Vec::new(),
                continue_target: 0, // patched to increment
            });

            // cond: #idx < len(#it)  — for Range, Len returns end-start
            ops.push(Op::Load(idx.clone()));
            ops.push(Op::Load(it.clone()));
            ops.push(Op::Len);
            ops.push(Op::Lt);
            let jf = ops.len();
            ops.push(Op::JumpIfFalse(0));

            // bind var = #it[#idx]  (range yields start+idx)
            ops.push(Op::Load(it.clone()));
            ops.push(Op::Load(idx.clone()));
            ops.push(Op::GetIndex);
            ops.push(Op::Store {
                name: var.clone(),
                ttl_ms: None,
            });

            compile_block(body, ops, loops)?;

            let cont = ops.len();
            // #idx = #idx + 1
            ops.push(Op::Load(idx.clone()));
            ops.push(Op::Const(Value::I64(1)));
            ops.push(Op::Add);
            ops.push(Op::Store {
                name: idx.clone(),
                ttl_ms: None,
            });
            ops.push(Op::Jump(loop_start));

            let after = ops.len();
            if let Op::JumpIfFalse(ref mut t) = ops[jf] {
                *t = after;
            }
            let mut ctx = loops.pop().unwrap();
            ctx.continue_target = cont;
            patch_loop(ops, ctx, after, cont);
        }
        Stmt::Break(span) => {
            if loops.is_empty() {
                return Err(KengaError::at("break outside of loop", span.clone()));
            }
            let i = ops.len();
            ops.push(Op::BreakPlaceholder);
            loops.last_mut().unwrap().break_patches.push(i);
        }
        Stmt::Continue(span) => {
            if loops.is_empty() {
                return Err(KengaError::at("continue outside of loop", span.clone()));
            }
            let i = ops.len();
            ops.push(Op::ContinuePlaceholder);
            loops.last_mut().unwrap().continue_patches.push(i);
        }
    }
    Ok(())
}

fn patch_loop(ops: &mut [Op], ctx: LoopCtx, break_target: usize, continue_target: usize) {
    for i in ctx.break_patches {
        ops[i] = Op::Jump(break_target);
    }
    for i in ctx.continue_patches {
        ops[i] = Op::Jump(continue_target);
    }
}

fn compile_expr(expr: &Expr, ops: &mut Vec<Op>) -> Result<()> {
    match expr {
        Expr::Int(n, _) => ops.push(Op::Const(Value::I64(*n))),
        Expr::Float(n, _) => ops.push(Op::Const(Value::F64(*n))),
        Expr::Bool(b, _) => ops.push(Op::Const(Value::Bool(*b))),
        Expr::Str(s, _) => ops.push(Op::Const(Value::Str(s.clone()))),
        Expr::Ident(name, _) => ops.push(Op::Load(name.clone())),
        Expr::List(elems, _) => {
            for e in elems {
                compile_expr(e, ops)?;
            }
            ops.push(Op::MakeList { n: elems.len() });
        }
        Expr::Range { start, end, .. } => {
            compile_expr(start, ops)?;
            compile_expr(end, ops)?;
            ops.push(Op::MakeRange);
        }
        Expr::StructLit { name, fields, .. } => {
            let mut names = Vec::new();
            for (fname, val) in fields {
                compile_expr(val, ops)?;
                names.push(fname.clone());
            }
            ops.push(Op::MakeStruct {
                name: name.clone(),
                fields: names,
            });
        }
        Expr::Index { target, index, .. } => {
            compile_expr(target, ops)?;
            compile_expr(index, ops)?;
            ops.push(Op::GetIndex);
        }
        Expr::Field { target, field, .. } => {
            compile_expr(target, ops)?;
            ops.push(Op::GetField(field.clone()));
        }
        Expr::Unary { op, expr, .. } => {
            compile_expr(expr, ops)?;
            match op {
                UnaryOp::Neg => ops.push(Op::Neg),
                UnaryOp::Not => ops.push(Op::Not),
                UnaryOp::BitNot => ops.push(Op::BitNot),
            }
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            compile_expr(left, ops)?;
            compile_expr(right, ops)?;
            ops.push(match op {
                BinaryOp::Add => Op::Add,
                BinaryOp::Sub => Op::Sub,
                BinaryOp::Mul => Op::Mul,
                BinaryOp::Div => Op::Div,
                BinaryOp::Rem => Op::Rem,
                BinaryOp::Eq => Op::Eq,
                BinaryOp::Ne => Op::Ne,
                BinaryOp::Lt => Op::Lt,
                BinaryOp::Le => Op::Le,
                BinaryOp::Gt => Op::Gt,
                BinaryOp::Ge => Op::Ge,
                BinaryOp::And => Op::And,
                BinaryOp::Or => Op::Or,
                BinaryOp::BitAnd => Op::BitAnd,
                BinaryOp::BitOr => Op::BitOr,
                BinaryOp::BitXor => Op::BitXor,
                BinaryOp::Shl => Op::Shl,
                BinaryOp::Shr => Op::Shr,
            });
        }
        Expr::Call { callee, args, span } => {
            match callee.as_str() {
                "print" => {
                    expect_argc("print", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::Print);
                    ops.push(Op::Const(Value::Nil));
                }
                "println" => {
                    expect_argc("println", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::Println);
                    ops.push(Op::Const(Value::Nil));
                }
                "tensor" => {
                    for a in args {
                        compile_expr(a, ops)?;
                    }
                    ops.push(Op::MakeTensor { dims: args.len() });
                }
                "sweep" => {
                    expect_argc("sweep", args, 0, span)?;
                    ops.push(Op::SweepMemory);
                    ops.push(Op::Const(Value::Nil));
                }
                "now_ms" => {
                    expect_argc("now_ms", args, 0, span)?;
                    ops.push(Op::Call {
                        name: "now_ms".into(),
                        argc: 0,
                    });
                }
                "len" => {
                    expect_argc("len", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::Len);
                }
                "push" => {
                    expect_argc("push", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::Push);
                }
                "assert" => {
                    expect_argc("assert", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::Assert);
                    ops.push(Op::Const(Value::Nil));
                }
                "typeof" => {
                    expect_argc("typeof", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::TypeOf);
                }
                "round" => {
                    expect_argc("round", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::Round);
                }
                "ord" => {
                    expect_argc("ord", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::Ord);
                }
                "to_str" => {
                    expect_argc("to_str", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::ToStr);
                }
                "input" => {
                    expect_argc("input", args, 0, span)?;
                    ops.push(Op::Input);
                }
                "listen" => {
                    expect_argc("listen", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::Listen);
                    ops.push(Op::Const(Value::Nil));
                }
                "emit" => {
                    expect_argc("emit", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::Emit);
                    ops.push(Op::Const(Value::Nil));
                }
                "pump" => {
                    expect_argc("pump", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::Pump);
                }
                "pending" => {
                    expect_argc("pending", args, 0, span)?;
                    ops.push(Op::Pending);
                }
                "sleep_ms" => {
                    expect_argc("sleep_ms", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::SleepMs);
                    ops.push(Op::Const(Value::Nil));
                }
                "memory" => {
                    expect_argc("memory", args, 0, span)?;
                    ops.push(Op::MakeMemory);
                }
                "memory_config" => {
                    expect_argc("memory_config", args, 3, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    compile_expr(&args[2], ops)?;
                    ops.push(Op::MakeMemoryConfig);
                }
                "remember" => {
                    expect_argc("remember", args, 3, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    compile_expr(&args[2], ops)?;
                    ops.push(Op::Remember);
                }
                "surprise" => {
                    expect_argc("surprise", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::Surprise);
                }
                "foresee" => {
                    expect_argc("foresee", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::Foresee);
                }
                "predict" => {
                    expect_argc("predict", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::Predict);
                }
                "learn" => {
                    expect_argc("learn", args, 3, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    compile_expr(&args[2], ops)?;
                    ops.push(Op::Learn);
                }
                "unroll" => {
                    expect_argc("unroll", args, 3, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    compile_expr(&args[2], ops)?;
                    ops.push(Op::Unroll);
                }
                "foresee_n" => {
                    expect_argc("foresee_n", args, 3, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    compile_expr(&args[2], ops)?;
                    ops.push(Op::ForeseeN);
                }
                "remember_next" => {
                    expect_argc("remember_next", args, 4, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    compile_expr(&args[2], ops)?;
                    compile_expr(&args[3], ops)?;
                    ops.push(Op::RememberNext);
                }
                "consolidate" => {
                    expect_argc("consolidate", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::Consolidate);
                }
                "recall" => {
                    expect_argc("recall", args, 3, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    compile_expr(&args[2], ops)?;
                    ops.push(Op::Recall);
                }
                "mem_stats" => {
                    expect_argc("mem_stats", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::MemStats);
                }
                "save_mind" => {
                    expect_argc("save_mind", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::SaveMind);
                }
                "load_mind" => {
                    expect_argc("load_mind", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::LoadMind);
                }
                "t_fill" => {
                    expect_argc("t_fill", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::TensorFill);
                }
                "t_get" => {
                    expect_argc("t_get", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::TensorGet);
                }
                "t_set" => {
                    expect_argc("t_set", args, 3, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    compile_expr(&args[2], ops)?;
                    ops.push(Op::TensorSet);
                }
                "t_shape" => {
                    expect_argc("t_shape", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::TensorShape);
                }
                "t_add" => {
                    expect_argc("t_add", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::TensorAdd);
                }
                "t_mul" => {
                    expect_argc("t_mul", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::TensorMul);
                }
                "t_matmul" => {
                    expect_argc("t_matmul", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::TensorMatmul);
                }
                "t_from" => {
                    expect_argc("t_from", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::TensorFrom);
                }
                "t_reshape" => {
                    expect_argc("t_reshape", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::TensorReshape);
                }
                "t_transpose" => {
                    expect_argc("t_transpose", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::TensorTranspose);
                }
                "t_exp" => {
                    expect_argc("t_exp", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::TensorExp);
                }
                "t_softmax" => {
                    expect_argc("t_softmax", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::TensorSoftmax);
                }
                "t_dot" => {
                    expect_argc("t_dot", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::TensorDot);
                }
                "t_sub" => {
                    expect_argc("t_sub", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::TensorSub);
                }
                "t_scale" => {
                    expect_argc("t_scale", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::TensorScale);
                }
                "t_sum" => {
                    expect_argc("t_sum", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::TensorSum);
                }
                "t_sgd_step" => {
                    expect_argc("t_sgd_step", args, 4, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    compile_expr(&args[2], ops)?;
                    compile_expr(&args[3], ops)?;
                    ops.push(Op::TensorSgdStep);
                }
                "t_mean" => {
                    expect_argc("t_mean", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::TensorMean);
                }
                "t_linear_grad" => {
                    expect_argc("t_linear_grad", args, 3, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    compile_expr(&args[2], ops)?;
                    ops.push(Op::TensorLinearGrad);
                }
                "t_mse" => {
                    expect_argc("t_mse", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::TensorMse);
                }
                "t_patch_mean" => {
                    expect_argc("t_patch_mean", args, 3, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    compile_expr(&args[2], ops)?;
                    ops.push(Op::TensorPatchMean);
                }
                "ag_clear" => {
                    expect_argc("ag_clear", args, 0, span)?;
                    ops.push(Op::AgClear);
                    ops.push(Op::Const(Value::Nil));
                }
                "ag_param" => {
                    expect_argc("ag_param", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::AgParam);
                }
                "ag_const" => {
                    expect_argc("ag_const", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::AgConst);
                }
                "ag_add" => {
                    expect_argc("ag_add", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::AgAdd);
                }
                "ag_sub" => {
                    expect_argc("ag_sub", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::AgSub);
                }
                "ag_mul" => {
                    expect_argc("ag_mul", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::AgMul);
                }
                "ag_matmul" => {
                    expect_argc("ag_matmul", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::AgMatmul);
                }
                "ag_scale" => {
                    expect_argc("ag_scale", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::AgScale);
                }
                "ag_relu" => {
                    expect_argc("ag_relu", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::AgRelu);
                }
                "ag_neg" => {
                    expect_argc("ag_neg", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::AgNeg);
                }
                "ag_transpose" => {
                    expect_argc("ag_transpose", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::AgTranspose);
                }
                "ag_reshape" => {
                    expect_argc("ag_reshape", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::AgReshape);
                }
                "ag_exp" => {
                    expect_argc("ag_exp", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::AgExp);
                }
                "ag_softmax" => {
                    expect_argc("ag_softmax", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::AgSoftmax);
                }
                "ag_mse" => {
                    expect_argc("ag_mse", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::AgMse);
                }
                "ag_sum" => {
                    expect_argc("ag_sum", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::AgSum);
                }
                "ag_value" => {
                    expect_argc("ag_value", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::AgValue);
                }
                "ag_grad" => {
                    expect_argc("ag_grad", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::AgGrad);
                }
                "ag_backward" => {
                    expect_argc("ag_backward", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::AgBackward);
                    ops.push(Op::Const(Value::Nil));
                }
                "ag_step" => {
                    expect_argc("ag_step", args, 2, span)?;
                    compile_expr(&args[0], ops)?;
                    compile_expr(&args[1], ops)?;
                    ops.push(Op::AgStep);
                }
                "load_ppm" => {
                    expect_argc("load_ppm", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::LoadPpm);
                }
                "load_wav" => {
                    expect_argc("load_wav", args, 1, span)?;
                    compile_expr(&args[0], ops)?;
                    ops.push(Op::LoadWav);
                }
                _ => {
                    for a in args {
                        compile_expr(a, ops)?;
                    }
                    ops.push(Op::Call {
                        name: callee.clone(),
                        argc: args.len(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn sanitize(event: &str) -> String {
    event
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn expect_argc(
    name: &str,
    args: &[Expr],
    n: usize,
    span: &crate::error::Span,
) -> Result<()> {
    if args.len() != n {
        return Err(KengaError::at(
            format!("{name} takes {n} argument(s)"),
            span.clone(),
        ));
    }
    Ok(())
}
