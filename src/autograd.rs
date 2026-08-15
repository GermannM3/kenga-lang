//! Minimal reverse-mode tape for dense f64 tensors.
//!
//! Handles are opaque `i64` node ids exposed to Kenga:
//!   ag_param / ag_const / ag_add / ag_mul / ag_matmul / ag_scale /
//!   ag_relu / ag_neg / ag_transpose / ag_reshape / ag_exp / ag_softmax /
//!   ag_mse / ag_backward / ag_grad / ag_value / ag_step / ag_clear

use std::cell::RefCell;

use crate::bytecode::Value;
use crate::error::{KengaError, Result};
use crate::tensor::{self, as_f64};

#[derive(Clone)]
enum OpKind {
    Leaf,
    Add(usize, usize),
    Sub(usize, usize),
    Mul(usize, usize),
    Matmul(usize, usize),
    Scale(usize, f64),
    Relu(usize),
    Neg(usize),
    Transpose(usize),
    Reshape(usize, Vec<usize>),
    Exp(usize),
    Softmax(usize),
    Mse(usize, usize),
    Sum(usize),
}

struct Node {
    value: Value,
    grad: Option<Value>,
    op: OpKind,
    requires_grad: bool,
}

#[derive(Default)]
struct Tape {
    nodes: Vec<Node>,
}

thread_local! {
    static TAPE: RefCell<Tape> = RefCell::new(Tape::default());
}

fn with_tape_mut<R>(f: impl FnOnce(&mut Tape) -> Result<R>) -> Result<R> {
    TAPE.with(|t| f(&mut t.borrow_mut()))
}

fn with_tape<R>(f: impl FnOnce(&Tape) -> Result<R>) -> Result<R> {
    TAPE.with(|t| f(&t.borrow()))
}

fn push_node(tape: &mut Tape, value: Value, op: OpKind, requires_grad: bool) -> i64 {
    let id = tape.nodes.len();
    tape.nodes.push(Node {
        value,
        grad: None,
        op,
        requires_grad,
    });
    id as i64
}

fn get_node<'a>(tape: &'a Tape, id: i64) -> Result<&'a Node> {
    tape.nodes
        .get(id as usize)
        .ok_or_else(|| KengaError::new(format!("ag: bad node id {id}"), None))
}

fn tensor_zeros_like(t: &Value) -> Result<Value> {
    match t {
        Value::Tensor { shape, data } => Ok(Value::Tensor {
            shape: shape.clone(),
            data: vec![0.0; data.len()],
        }),
        Value::F64(_) => Ok(Value::F64(0.0)),
        _ => Err(KengaError::new("ag: expected tensor", None)),
    }
}

fn accum_grad(dst: &mut Option<Value>, add: Value) -> Result<()> {
    match dst {
        None => *dst = Some(add),
        Some(prev) => {
            let left = std::mem::replace(prev, Value::Nil);
            *prev = match (left, add) {
                (Value::F64(a), Value::F64(b)) => Value::F64(a + b),
                (
                    Value::Tensor {
                        shape,
                        data: d1,
                    },
                    Value::Tensor { data: d2, .. },
                ) => {
                    if d1.len() != d2.len() {
                        return Err(KengaError::new("ag: grad len mismatch", None));
                    }
                    Value::Tensor {
                        shape,
                        data: d1
                            .into_iter()
                            .zip(d2.into_iter())
                            .map(|(x, y)| x + y)
                            .collect(),
                    }
                }
                _ => return Err(KengaError::new("ag: grad shape/type mismatch", None)),
            };
        }
    }
    Ok(())
}

pub fn ag_clear() {
    TAPE.with(|t| {
        t.borrow_mut().nodes.clear();
    });
}

pub fn ag_param(t: Value) -> Result<i64> {
    match &t {
        Value::Tensor { .. } => {}
        _ => return Err(KengaError::new("ag_param expects Tensor", None)),
    }
    with_tape_mut(|tape| Ok(push_node(tape, t, OpKind::Leaf, true)))
}

pub fn ag_const(t: Value) -> Result<i64> {
    match &t {
        Value::Tensor { .. } | Value::F64(_) => {}
        Value::I64(n) => {
            return with_tape_mut(|tape| {
                Ok(push_node(
                    tape,
                    Value::F64(*n as f64),
                    OpKind::Leaf,
                    false,
                ))
            });
        }
        _ => return Err(KengaError::new("ag_const expects Tensor|number", None)),
    }
    with_tape_mut(|tape| Ok(push_node(tape, t, OpKind::Leaf, false)))
}

fn bin(
    a: i64,
    b: i64,
    kind: impl Fn(usize, usize) -> OpKind,
    f: impl Fn(Value, Value) -> Result<Value>,
) -> Result<i64> {
    with_tape_mut(|tape| {
        let na = get_node(tape, a)?;
        let nb = get_node(tape, b)?;
        let requires = na.requires_grad || nb.requires_grad;
        let value = f(na.value.clone(), nb.value.clone())?;
        Ok(push_node(
            tape,
            value,
            kind(a as usize, b as usize),
            requires,
        ))
    })
}

pub fn ag_add(a: i64, b: i64) -> Result<i64> {
    bin(a, b, OpKind::Add, |x, y| tensor::tensor_ew(x, y, |u, v| u + v))
}

pub fn ag_sub(a: i64, b: i64) -> Result<i64> {
    bin(a, b, OpKind::Sub, |x, y| tensor::tensor_ew(x, y, |u, v| u - v))
}

pub fn ag_mul(a: i64, b: i64) -> Result<i64> {
    bin(a, b, OpKind::Mul, |x, y| tensor::tensor_ew(x, y, |u, v| u * v))
}

pub fn ag_matmul(a: i64, b: i64) -> Result<i64> {
    bin(a, b, OpKind::Matmul, tensor::tensor_matmul)
}

pub fn ag_scale(a: i64, s: f64) -> Result<i64> {
    with_tape_mut(|tape| {
        let na = get_node(tape, a)?;
        let requires = na.requires_grad;
        let value = tensor::tensor_scale(na.value.clone(), s)?;
        Ok(push_node(tape, value, OpKind::Scale(a as usize, s), requires))
    })
}

pub fn ag_relu(a: i64) -> Result<i64> {
    with_tape_mut(|tape| {
        let na = get_node(tape, a)?;
        let requires = na.requires_grad;
        let value = match &na.value {
            Value::Tensor { shape, data } => Value::Tensor {
                shape: shape.clone(),
                data: data.iter().map(|x| x.max(0.0)).collect(),
            },
            Value::F64(x) => Value::F64(x.max(0.0)),
            _ => return Err(KengaError::new("ag_relu expects tensor", None)),
        };
        Ok(push_node(tape, value, OpKind::Relu(a as usize), requires))
    })
}

pub fn ag_neg(a: i64) -> Result<i64> {
    with_tape_mut(|tape| {
        let na = get_node(tape, a)?;
        let requires = na.requires_grad;
        let value = tensor::tensor_scale(na.value.clone(), -1.0)?;
        Ok(push_node(tape, value, OpKind::Neg(a as usize), requires))
    })
}

pub fn ag_transpose(a: i64) -> Result<i64> {
    with_tape_mut(|tape| {
        let na = get_node(tape, a)?;
        let requires = na.requires_grad;
        let value = tensor::tensor_transpose(na.value.clone())?;
        Ok(push_node(
            tape,
            value,
            OpKind::Transpose(a as usize),
            requires,
        ))
    })
}

pub fn ag_reshape(a: i64, shape: Value) -> Result<i64> {
    with_tape_mut(|tape| {
        let na = get_node(tape, a)?;
        let requires = na.requires_grad;
        let old_shape = match &na.value {
            Value::Tensor { shape, .. } => shape.clone(),
            _ => return Err(KengaError::new("ag_reshape expects tensor", None)),
        };
        let value = tensor::tensor_reshape(na.value.clone(), shape)?;
        Ok(push_node(
            tape,
            value,
            OpKind::Reshape(a as usize, old_shape),
            requires,
        ))
    })
}

pub fn ag_exp(a: i64) -> Result<i64> {
    with_tape_mut(|tape| {
        let na = get_node(tape, a)?;
        let requires = na.requires_grad;
        let value = tensor::tensor_exp(na.value.clone())?;
        Ok(push_node(tape, value, OpKind::Exp(a as usize), requires))
    })
}

pub fn ag_softmax(a: i64) -> Result<i64> {
    with_tape_mut(|tape| {
        let na = get_node(tape, a)?;
        let requires = na.requires_grad;
        let value = tensor::tensor_softmax(na.value.clone())?;
        Ok(push_node(
            tape,
            value,
            OpKind::Softmax(a as usize),
            requires,
        ))
    })
}

pub fn ag_mse(pred: i64, target: i64) -> Result<i64> {
    with_tape_mut(|tape| {
        let np = get_node(tape, pred)?;
        let nt = get_node(tape, target)?;
        let requires = np.requires_grad || nt.requires_grad;
        let loss = tensor::tensor_mse(np.value.clone(), nt.value.clone())?;
        Ok(push_node(
            tape,
            Value::F64(loss),
            OpKind::Mse(pred as usize, target as usize),
            requires,
        ))
    })
}

pub fn ag_sum(a: i64) -> Result<i64> {
    with_tape_mut(|tape| {
        let na = get_node(tape, a)?;
        let requires = na.requires_grad;
        let s = tensor::tensor_sum(&na.value)?;
        Ok(push_node(
            tape,
            Value::F64(s),
            OpKind::Sum(a as usize),
            requires,
        ))
    })
}

pub fn ag_value(id: i64) -> Result<Value> {
    with_tape(|tape| Ok(get_node(tape, id)?.value.clone()))
}

pub fn ag_grad(id: i64) -> Result<Value> {
    with_tape(|tape| {
        let n = get_node(tape, id)?;
        n.grad
            .clone()
            .ok_or_else(|| KengaError::new("ag_grad: no grad (call ag_backward first)", None))
    })
}

pub fn ag_backward(loss_id: i64) -> Result<()> {
    with_tape_mut(|tape| {
        let n = tape.nodes.len();
        if loss_id < 0 || loss_id as usize >= n {
            return Err(KengaError::new("ag_backward: bad loss id", None));
        }
        for node in &mut tape.nodes {
            node.grad = None;
        }
        // seed
        match &tape.nodes[loss_id as usize].value {
            Value::F64(_) => {
                tape.nodes[loss_id as usize].grad = Some(Value::F64(1.0));
            }
            Value::Tensor { .. } => {
                let z = tensor_zeros_like(&tape.nodes[loss_id as usize].value)?;
                // ones
                let ones = match z {
                    Value::Tensor { shape, data } => Value::Tensor {
                        shape,
                        data: vec![1.0; data.len()],
                    },
                    other => other,
                };
                tape.nodes[loss_id as usize].grad = Some(ones);
            }
            _ => return Err(KengaError::new("ag_backward: loss must be scalar/tensor", None)),
        }

        for i in (0..=loss_id as usize).rev() {
            let grad = match tape.nodes[i].grad.clone() {
                Some(g) => g,
                None => continue,
            };
            let op = tape.nodes[i].op.clone();
            match op {
                OpKind::Leaf => {}
                OpKind::Add(a, b) => {
                    accum_grad(&mut tape.nodes[a].grad, grad.clone())?;
                    accum_grad(&mut tape.nodes[b].grad, grad)?;
                }
                OpKind::Sub(a, b) => {
                    accum_grad(&mut tape.nodes[a].grad, grad.clone())?;
                    let neg = tensor::tensor_scale(grad, -1.0)?;
                    accum_grad(&mut tape.nodes[b].grad, neg)?;
                }
                OpKind::Mul(a, b) => {
                    let va = tape.nodes[a].value.clone();
                    let vb = tape.nodes[b].value.clone();
                    let ga = tensor::tensor_ew(grad.clone(), vb, |g, x| g * x)?;
                    let gb = tensor::tensor_ew(grad, va, |g, x| g * x)?;
                    accum_grad(&mut tape.nodes[a].grad, ga)?;
                    accum_grad(&mut tape.nodes[b].grad, gb)?;
                }
                OpKind::Matmul(a, b) => {
                    // dA = G @ B^T, dB = A^T @ G
                    let va = tape.nodes[a].value.clone();
                    let vb = tape.nodes[b].value.clone();
                    let bt = tensor::tensor_transpose(vb)?;
                    let at = tensor::tensor_transpose(va)?;
                    let ga = tensor::tensor_matmul(grad.clone(), bt)?;
                    let gb = tensor::tensor_matmul(at, grad)?;
                    accum_grad(&mut tape.nodes[a].grad, ga)?;
                    accum_grad(&mut tape.nodes[b].grad, gb)?;
                }
                OpKind::Scale(a, s) => {
                    let ga = tensor::tensor_scale(grad, s)?;
                    accum_grad(&mut tape.nodes[a].grad, ga)?;
                }
                OpKind::Relu(a) => {
                    let va = tape.nodes[a].value.clone();
                    let ga = match (grad, va) {
                        (
                            Value::Tensor {
                                shape: gs,
                                data: gd,
                            },
                            Value::Tensor { data: vd, .. },
                        ) => Value::Tensor {
                            shape: gs,
                            data: gd
                                .into_iter()
                                .zip(vd.into_iter())
                                .map(|(g, v)| if v > 0.0 { g } else { 0.0 })
                                .collect(),
                        },
                        (Value::F64(g), Value::F64(v)) => Value::F64(if v > 0.0 { g } else { 0.0 }),
                        _ => return Err(KengaError::new("ag relu grad mismatch", None)),
                    };
                    accum_grad(&mut tape.nodes[a].grad, ga)?;
                }
                OpKind::Neg(a) => {
                    let ga = tensor::tensor_scale(grad, -1.0)?;
                    accum_grad(&mut tape.nodes[a].grad, ga)?;
                }
                OpKind::Transpose(a) => {
                    let ga = tensor::tensor_transpose(grad)?;
                    accum_grad(&mut tape.nodes[a].grad, ga)?;
                }
                OpKind::Reshape(a, old_shape) => {
                    let shape_v = Value::List(
                        old_shape
                            .into_iter()
                            .map(|d| Value::I64(d as i64))
                            .collect(),
                    );
                    let ga = tensor::tensor_reshape(grad, shape_v)?;
                    accum_grad(&mut tape.nodes[a].grad, ga)?;
                }
                OpKind::Exp(a) => {
                    // d exp(x) = exp(x) * g = y * g
                    let y = tape.nodes[i].value.clone();
                    let ga = tensor::tensor_ew(grad, y, |g, y| g * y)?;
                    accum_grad(&mut tape.nodes[a].grad, ga)?;
                }
                OpKind::Softmax(a) => {
                    // y = softmax(x); dx = y * (gy - sum(gy * y))
                    let y = tape.nodes[i].value.clone();
                    let ga = match (grad, y) {
                        (
                            Value::Tensor {
                                shape,
                                data: gy,
                            },
                            Value::Tensor { data: yv, .. },
                        ) => {
                            if gy.len() != yv.len() {
                                return Err(KengaError::new("ag softmax grad len", None));
                            }
                            let dot: f64 = gy.iter().zip(yv.iter()).map(|(g, y)| g * y).sum();
                            Value::Tensor {
                                shape,
                                data: gy
                                    .into_iter()
                                    .zip(yv.into_iter())
                                    .map(|(g, y)| y * (g - dot))
                                    .collect(),
                            }
                        }
                        _ => return Err(KengaError::new("ag softmax grad mismatch", None)),
                    };
                    accum_grad(&mut tape.nodes[a].grad, ga)?;
                }
                OpKind::Mse(p, t) => {
                    // d/dpred (mean (pred-target)^2) = 2/n * (pred-target)
                    // d/dtarget = -that
                    let g = as_f64(&grad)?; // scalar loss grad
                    let pred = tape.nodes[p].value.clone();
                    let target = tape.nodes[t].value.clone();
                    let err = tensor::tensor_sub(pred, target)?;
                    let n = match &err {
                        Value::Tensor { data, .. } => data.len().max(1) as f64,
                        _ => 1.0,
                    };
                    let scale = 2.0 * g / n;
                    let gp = tensor::tensor_scale(err.clone(), scale)?;
                    let gt = tensor::tensor_scale(err, -scale)?;
                    accum_grad(&mut tape.nodes[p].grad, gp)?;
                    accum_grad(&mut tape.nodes[t].grad, gt)?;
                }
                OpKind::Sum(a) => {
                    let g = as_f64(&grad)?;
                    let ones = tensor_zeros_like(&tape.nodes[a].value)?;
                    let ga = match ones {
                        Value::Tensor { shape, data } => Value::Tensor {
                            shape,
                            data: vec![g; data.len()],
                        },
                        Value::F64(_) => Value::F64(g),
                        other => other,
                    };
                    accum_grad(&mut tape.nodes[a].grad, ga)?;
                }
            }
        }
        Ok(())
    })
}

/// Parameter update: param_value - lr * grad
pub fn ag_step(param_id: i64, lr: f64) -> Result<Value> {
    with_tape(|tape| {
        let n = get_node(tape, param_id)?;
        let g = n
            .grad
            .clone()
            .ok_or_else(|| KengaError::new("ag_step: missing grad", None))?;
        let step = tensor::tensor_scale(g, lr)?;
        tensor::tensor_sub(n.value.clone(), step)
    })
}
