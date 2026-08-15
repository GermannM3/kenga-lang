//! Dense f64 tensor helpers for the VM.

use crate::bytecode::Value;
use crate::error::{KengaError, Result};

pub fn as_f64(v: &Value) -> Result<f64> {
    match v {
        Value::F64(n) => Ok(*n),
        Value::I64(n) => Ok(*n as f64),
        _ => Err(KengaError::new("expected number (i64|f64)", None)),
    }
}

pub fn tensor_fill(t: Value, v: f64) -> Result<Value> {
    match t {
        Value::Tensor { shape, mut data } => {
            for x in &mut data {
                *x = v;
            }
            Ok(Value::Tensor { shape, data })
        }
        _ => Err(KengaError::new("t_fill expects Tensor", None)),
    }
}

pub fn tensor_get(t: &Value, index: i64) -> Result<f64> {
    match t {
        Value::Tensor { data, .. } => {
            if index < 0 || index as usize >= data.len() {
                return Err(KengaError::new("t_get index out of range", None));
            }
            Ok(data[index as usize])
        }
        _ => Err(KengaError::new("t_get expects Tensor", None)),
    }
}

pub fn tensor_set(t: Value, index: i64, v: f64) -> Result<Value> {
    match t {
        Value::Tensor { shape, mut data } => {
            if index < 0 || index as usize >= data.len() {
                return Err(KengaError::new("t_set index out of range", None));
            }
            data[index as usize] = v;
            Ok(Value::Tensor { shape, data })
        }
        _ => Err(KengaError::new("t_set expects Tensor", None)),
    }
}

pub fn tensor_shape(t: &Value) -> Result<Value> {
    match t {
        Value::Tensor { shape, .. } => Ok(Value::List(
            shape.iter().map(|d| Value::I64(*d as i64)).collect(),
        )),
        _ => Err(KengaError::new("t_shape expects Tensor", None)),
    }
}

fn same_shape(a: &[usize], b: &[usize]) -> bool {
    a == b
}

pub fn tensor_ew(a: Value, b: Value, op: fn(f64, f64) -> f64) -> Result<Value> {
    match (a, b) {
        (
            Value::Tensor {
                shape: sa,
                data: da,
            },
            Value::Tensor {
                shape: sb,
                data: db,
            },
        ) => {
            if !same_shape(&sa, &sb) {
                return Err(KengaError::new("tensor shape mismatch", None));
            }
            let data = da
                .into_iter()
                .zip(db.into_iter())
                .map(|(x, y)| op(x, y))
                .collect();
            Ok(Value::Tensor { shape: sa, data })
        }
        _ => Err(KengaError::new("elementwise ops expect two Tensors", None)),
    }
}

/// Matmul for rank-2 tensors: (m,k) @ (k,n) -> (m,n)
pub fn tensor_matmul(a: Value, b: Value) -> Result<Value> {
    match (a, b) {
        (
            Value::Tensor {
                shape: sa,
                data: da,
            },
            Value::Tensor {
                shape: sb,
                data: db,
            },
        ) => {
            if sa.len() != 2 || sb.len() != 2 {
                return Err(KengaError::new("t_matmul expects rank-2 tensors", None));
            }
            let (m, k) = (sa[0], sa[1]);
            let (k2, n) = (sb[0], sb[1]);
            if k != k2 {
                return Err(KengaError::new(
                    format!("t_matmul inner dim mismatch {k} vs {k2}"),
                    None,
                ));
            }
            let mut out = vec![0.0; m * n];
            for i in 0..m {
                for j in 0..n {
                    let mut s = 0.0;
                    for t in 0..k {
                        s += da[i * k + t] * db[t * n + j];
                    }
                    out[i * n + j] = s;
                }
            }
            Ok(Value::Tensor {
                shape: vec![m, n],
                data: out,
            })
        }
        _ => Err(KengaError::new("t_matmul expects two Tensors", None)),
    }
}

/// Build tensor from shape list + flat data list (numbers).
pub fn tensor_from(shape_v: Value, data_v: Value) -> Result<Value> {
    let shape = match shape_v {
        Value::List(xs) => xs
            .into_iter()
            .map(|v| match v {
                Value::I64(n) if n >= 0 => Ok(n as usize),
                _ => Err(KengaError::new("t_from shape must be list of i64", None)),
            })
            .collect::<Result<Vec<_>>>()?,
        _ => return Err(KengaError::new("t_from shape must be list", None)),
    };
    let data = match data_v {
        Value::List(xs) => xs
            .into_iter()
            .map(|v| as_f64(&v))
            .collect::<Result<Vec<_>>>()?,
        _ => return Err(KengaError::new("t_from data must be list", None)),
    };
    let need: usize = if shape.is_empty() {
        0
    } else {
        shape.iter().product()
    };
    if data.len() != need {
        return Err(KengaError::new(
            format!("t_from expected {need} values, got {}", data.len()),
            None,
        ));
    }
    Ok(Value::Tensor { shape, data })
}

/// Change shape without moving data; product(shape) must match len(data).
pub fn tensor_reshape(t: Value, shape_v: Value) -> Result<Value> {
    let shape = match shape_v {
        Value::List(xs) => xs
            .into_iter()
            .map(|v| match v {
                Value::I64(n) if n >= 0 => Ok(n as usize),
                _ => Err(KengaError::new("t_reshape shape must be list of i64", None)),
            })
            .collect::<Result<Vec<_>>>()?,
        _ => return Err(KengaError::new("t_reshape shape must be list", None)),
    };
    match t {
        Value::Tensor { data, .. } => {
            let need: usize = if shape.is_empty() {
                0
            } else {
                shape.iter().product()
            };
            if data.len() != need {
                return Err(KengaError::new(
                    format!(
                        "t_reshape expected {need} elems, tensor has {}",
                        data.len()
                    ),
                    None,
                ));
            }
            Ok(Value::Tensor { shape, data })
        }
        _ => Err(KengaError::new("t_reshape expects Tensor", None)),
    }
}

/// Transpose a rank-2 tensor (rows ↔ cols).
pub fn tensor_transpose(t: Value) -> Result<Value> {
    match t {
        Value::Tensor { shape, data } if shape.len() == 2 => {
            let (r, c) = (shape[0], shape[1]);
            let mut out = vec![0.0; r * c];
            for i in 0..r {
                for j in 0..c {
                    out[j * r + i] = data[i * c + j];
                }
            }
            Ok(Value::Tensor {
                shape: vec![c, r],
                data: out,
            })
        }
        Value::Tensor { shape, .. } => Err(KengaError::new(
            format!("t_transpose expects rank-2, got {:?}", shape),
            None,
        )),
        _ => Err(KengaError::new("t_transpose expects Tensor", None)),
    }
}

/// Element-wise exp.
pub fn tensor_exp(t: Value) -> Result<Value> {
    match t {
        Value::Tensor { shape, data } => Ok(Value::Tensor {
            shape,
            data: data.into_iter().map(f64::exp).collect(),
        }),
        Value::F64(x) => Ok(Value::F64(x.exp())),
        _ => Err(KengaError::new("t_exp expects Tensor", None)),
    }
}

/// Softmax over all elements (flattened). Numerically stable.
pub fn tensor_softmax(t: Value) -> Result<Value> {
    match t {
        Value::Tensor { shape, data } => {
            if data.is_empty() {
                return Ok(Value::Tensor { shape, data });
            }
            let m = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let exps: Vec<f64> = data.iter().map(|x| (x - m).exp()).collect();
            let s: f64 = exps.iter().sum();
            let inv = if s == 0.0 { 0.0 } else { 1.0 / s };
            Ok(Value::Tensor {
                shape,
                data: exps.into_iter().map(|e| e * inv).collect(),
            })
        }
        Value::F64(_) => Ok(Value::F64(1.0)),
        _ => Err(KengaError::new("t_softmax expects Tensor", None)),
    }
}

/// Dot product of two 1-D tensors (same length) → f64.
pub fn tensor_dot(a: Value, b: Value) -> Result<Value> {
    match (a, b) {
        (
            Value::Tensor {
                shape: sa,
                data: da,
            },
            Value::Tensor {
                shape: sb,
                data: db,
            },
        ) => {
            if sa.len() != 1 || sb.len() != 1 {
                return Err(KengaError::new("t_dot expects rank-1 tensors", None));
            }
            if da.len() != db.len() {
                return Err(KengaError::new("t_dot length mismatch", None));
            }
            let s: f64 = da.iter().zip(db.iter()).map(|(x, y)| x * y).sum();
            Ok(Value::F64(s))
        }
        _ => Err(KengaError::new("t_dot expects two Tensors", None)),
    }
}

pub fn tensor_sub(a: Value, b: Value) -> Result<Value> {
    tensor_ew(a, b, |x, y| x - y)
}

pub fn tensor_scale(t: Value, s: f64) -> Result<Value> {
    match t {
        Value::Tensor { shape, data } => Ok(Value::Tensor {
            shape,
            data: data.into_iter().map(|x| x * s).collect(),
        }),
        _ => Err(KengaError::new("t_scale expects Tensor", None)),
    }
}

pub fn tensor_sum(t: &Value) -> Result<f64> {
    match t {
        Value::Tensor { data, .. } => Ok(data.iter().sum()),
        _ => Err(KengaError::new("t_sum expects Tensor", None)),
    }
}

/// One SGD step for linear map `y ≈ W @ x` (MSE).
/// W is (out, in), x is (in, 1) or (in,), y is (out, 1) or (out,).
/// Returns updated W.
pub fn tensor_sgd_step(w: Value, x: Value, y: Value, lr: f64) -> Result<Value> {
    let pred = tensor_matmul(w.clone(), ensure_col(x.clone())?)?;
    let ycol = ensure_col(y)?;
    let err = tensor_sub(pred, ycol)?; // (out, 1)
    // dW = err @ x^T
    let xt = transpose_col_to_row(ensure_col(x)?)?; // (1, in)
    let grad = tensor_matmul(err, xt)?; // (out, in)
    let step = tensor_scale(grad, lr)?;
    tensor_sub(w, step)
}

/// Return `(loss, grad_w)` for linear `W @ x ≈ y` (MSE). Explicit autograd slice.
pub fn tensor_linear_backward(w: Value, x: Value, y: Value) -> Result<(f64, Value)> {
    let xcol = ensure_col(x)?;
    let ycol = ensure_col(y)?;
    let pred = tensor_matmul(w, xcol.clone())?;
    let err = tensor_sub(pred, ycol)?;
    let loss = match &err {
        Value::Tensor { data, .. } => {
            data.iter().map(|e| e * e).sum::<f64>() / data.len().max(1) as f64
        }
        _ => 0.0,
    };
    let xt = transpose_col_to_row(xcol)?;
    let grad = tensor_matmul(err, xt)?;
    Ok((loss, grad))
}

/// Gradient-only helper for language binding (loss via separate mse if needed).
pub fn tensor_linear_grad(w: Value, x: Value, y: Value) -> Result<Value> {
    Ok(tensor_linear_backward(w, x, y)?.1)
}

pub fn tensor_mse(a: Value, b: Value) -> Result<f64> {
    let err = tensor_sub(a, b)?;
    match err {
        Value::Tensor { data, .. } => {
            Ok(data.iter().map(|e| e * e).sum::<f64>() / data.len().max(1) as f64)
        }
        _ => Err(KengaError::new("t_mse expects tensors", None)),
    }
}

/// Tiny patch encoder: non-overlapping mean-pool of [h,w,c] → [gh, gw, c].
pub fn tensor_patch_mean(t: Value, gh: i64, gw: i64) -> Result<Value> {
    if gh <= 0 || gw <= 0 {
        return Err(KengaError::new("t_patch_mean grid must be > 0", None));
    }
    match t {
        Value::Tensor { shape, data } if shape.len() == 3 => {
            let (h, w, c) = (shape[0], shape[1], shape[2]);
            let gh = gh as usize;
            let gw = gw as usize;
            let mut out = vec![0.0; gh * gw * c];
            let ph = h / gh;
            let pw = w / gw;
            if ph == 0 || pw == 0 {
                return Err(KengaError::new("t_patch_mean: image smaller than grid", None));
            }
            for gy in 0..gh {
                for gx in 0..gw {
                    for ch in 0..c {
                        let mut s = 0.0;
                        let mut n = 0.0;
                        for y in gy * ph..(gy + 1) * ph {
                            for x in gx * pw..(gx + 1) * pw {
                                s += data[(y * w + x) * c + ch];
                                n += 1.0;
                            }
                        }
                        out[(gy * gw + gx) * c + ch] = s / n;
                    }
                }
            }
            Ok(Value::Tensor {
                shape: vec![gh, gw, c],
                data: out,
            })
        }
        _ => Err(KengaError::new("t_patch_mean expects Tensor [h,w,c]", None)),
    }
}

fn ensure_col(t: Value) -> Result<Value> {
    match t {
        Value::Tensor { shape, data } if shape.len() == 1 => Ok(Value::Tensor {
            shape: vec![shape[0], 1],
            data,
        }),
        Value::Tensor { shape, data } if shape.len() == 2 && shape[1] == 1 => {
            Ok(Value::Tensor { shape, data })
        }
        Value::Tensor { .. } => Err(KengaError::new(
            "expected vector or column matrix (n,) / (n,1)",
            None,
        )),
        _ => Err(KengaError::new("expected Tensor", None)),
    }
}

fn transpose_col_to_row(t: Value) -> Result<Value> {
    match t {
        Value::Tensor { shape, data } if shape.len() == 2 && shape[1] == 1 => Ok(Value::Tensor {
            shape: vec![1, shape[0]],
            data,
        }),
        _ => Err(KengaError::new("transpose expects (n,1)", None)),
    }
}

/// Load binary PPM P6 (RGB) → Tensor shape [h, w, 3] with values in 0..1.
pub fn load_ppm(path: &str) -> Result<Value> {
    let bytes = std::fs::read(path)
        .map_err(|e| KengaError::new(format!("load_ppm: cannot read {path}: {e}"), None))?;
    parse_ppm_p6(&bytes)
}

fn parse_ppm_p6(bytes: &[u8]) -> Result<Value> {
    let mut i = 0;
    fn skip_ws_comments(bytes: &[u8], i: &mut usize) {
        loop {
            while *i < bytes.len() && bytes[*i].is_ascii_whitespace() {
                *i += 1;
            }
            if *i < bytes.len() && bytes[*i] == b'#' {
                while *i < bytes.len() && bytes[*i] != b'\n' {
                    *i += 1;
                }
                continue;
            }
            break;
        }
    }
    fn read_token<'a>(bytes: &'a [u8], i: &mut usize) -> Result<&'a [u8]> {
        skip_ws_comments(bytes, i);
        let start = *i;
        while *i < bytes.len() && !bytes[*i].is_ascii_whitespace() && bytes[*i] != b'#' {
            *i += 1;
        }
        if start == *i {
            return Err(KengaError::new("load_ppm: truncated header", None));
        }
        Ok(&bytes[start..*i])
    }
    let magic = read_token(bytes, &mut i)?;
    if magic != b"P6" {
        return Err(KengaError::new("load_ppm: only P6 binary PPM supported", None));
    }
    let w: usize = std::str::from_utf8(read_token(bytes, &mut i)?)
        .ok()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| KengaError::new("load_ppm: bad width", None))?;
    let h: usize = std::str::from_utf8(read_token(bytes, &mut i)?)
        .ok()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| KengaError::new("load_ppm: bad height", None))?;
    let maxv: f64 = std::str::from_utf8(read_token(bytes, &mut i)?)
        .ok()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| KengaError::new("load_ppm: bad maxval", None))?;
    if maxv <= 0.0 {
        return Err(KengaError::new("load_ppm: maxval must be > 0", None));
    }
    // single whitespace after maxval
    if i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let need = w * h * 3;
    if bytes.len() < i + need {
        return Err(KengaError::new("load_ppm: truncated pixel data", None));
    }
    let mut data = Vec::with_capacity(need);
    for b in &bytes[i..i + need] {
        data.push(*b as f64 / maxv);
    }
    Ok(Value::Tensor {
        shape: vec![h, w, 3],
        data,
    })
}

/// Mean over spatial dims of [h,w,c] or [h,w] → vector length c (or 1).
pub fn tensor_mean(t: &Value) -> Result<Value> {
    match t {
        Value::Tensor { shape, data } if shape.len() == 3 => {
            let (h, w, c) = (shape[0], shape[1], shape[2]);
            let n = (h * w) as f64;
            let mut out = vec![0.0; c];
            for y in 0..h {
                for x in 0..w {
                    for ch in 0..c {
                        out[ch] += data[(y * w + x) * c + ch];
                    }
                }
            }
            for v in &mut out {
                *v /= n;
            }
            Ok(Value::Tensor {
                shape: vec![c],
                data: out,
            })
        }
        Value::Tensor { shape, data } if shape.len() == 2 => {
            let n = data.len() as f64;
            let s: f64 = data.iter().sum::<f64>() / n;
            Ok(Value::Tensor {
                shape: vec![1],
                data: vec![s],
            })
        }
        Value::Tensor { data, .. } => {
            let n = data.len().max(1) as f64;
            Ok(Value::F64(data.iter().sum::<f64>() / n))
        }
        _ => Err(KengaError::new("t_mean expects Tensor", None)),
    }
}

/// Load PCM WAV (16-bit mono or first channel) → Tensor shape [n] in -1..1.
pub fn load_wav(path: &str) -> Result<Value> {
    let bytes = std::fs::read(path)
        .map_err(|e| KengaError::new(format!("load_wav: cannot read {path}: {e}"), None))?;
    parse_wav_pcm16(&bytes)
}

fn parse_wav_pcm16(bytes: &[u8]) -> Result<Value> {
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(KengaError::new("load_wav: not a RIFF/WAVE file", None));
    }
    let mut pos = 12usize;
    let mut fmt_channels = 1u16;
    let mut bits = 16u16;
    let mut data_off = 0usize;
    let mut data_len = 0usize;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let chunk = pos + 8;
        if id == b"fmt " {
            if size < 16 || chunk + 16 > bytes.len() {
                return Err(KengaError::new("load_wav: bad fmt chunk", None));
            }
            let format = u16::from_le_bytes(bytes[chunk..chunk + 2].try_into().unwrap());
            if format != 1 {
                return Err(KengaError::new("load_wav: only PCM format=1", None));
            }
            fmt_channels = u16::from_le_bytes(bytes[chunk + 2..chunk + 4].try_into().unwrap());
            bits = u16::from_le_bytes(bytes[chunk + 14..chunk + 16].try_into().unwrap());
        } else if id == b"data" {
            data_off = chunk;
            data_len = size;
            break;
        }
        pos = chunk + size + (size % 2); // word align
    }
    if data_off == 0 || bits != 16 {
        return Err(KengaError::new(
            "load_wav: need 16-bit PCM data chunk",
            None,
        ));
    }
    if data_off + data_len > bytes.len() {
        return Err(KengaError::new("load_wav: truncated data", None));
    }
    let step = fmt_channels as usize;
    let samples = data_len / 2 / step.max(1);
    let mut data = Vec::with_capacity(samples);
    for i in 0..samples {
        let off = data_off + i * step * 2;
        let s = i16::from_le_bytes(bytes[off..off + 2].try_into().unwrap());
        data.push(s as f64 / 32768.0);
    }
    Ok(Value::Tensor {
        shape: vec![data.len()],
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ppm_roundtrip_mean() {
        let bytes = b"P6\n2 2\n255\n\xFF\x00\x00\x00\xFF\x00\x00\x00\xFF\xFF\xFF\xFF";
        let t = parse_ppm_p6(bytes).unwrap();
        match &t {
            Value::Tensor { shape, .. } => assert_eq!(shape, &[2, 2, 3]),
            _ => panic!(),
        }
        let m = tensor_mean(&t).unwrap();
        match m {
            Value::Tensor { data, .. } => {
                assert!((data[0] - 0.5).abs() < 1e-9);
                assert!((data[1] - 0.5).abs() < 1e-9);
                assert!((data[2] - 0.5).abs() < 1e-9);
            }
            _ => panic!(),
        }
    }
}
