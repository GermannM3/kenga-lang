//! Prophet memory: episodic buffer + core knowledge + consolidate (sleep).
//!
//! Surprise gates what enters episodic memory. During `consolidate` the runtime
//! replays episodes into core traces and raises importance locks (EWC-lite),
//! so new learning does not wipe old laws of the world.

use std::cell::RefCell;
use std::rc::Rc;

use crate::bytecode::Value;
use crate::error::{KengaError, Result};

const DEFAULT_THRESHOLD: f64 = 0.15;
const DEFAULT_EP_CAP: usize = 64;
const DEFAULT_CORE_CAP: usize = 32;

#[derive(Debug, Clone)]
pub struct Episode {
    pub pattern: Vec<f64>,
    pub surprise: f64,
    pub ts_ms: u64,
}

#[derive(Debug, Clone)]
pub struct CoreTrace {
    pub pattern: Vec<f64>,
    /// EWC-like lock: higher => harder to overwrite
    pub importance: f64,
    pub replays: u64,
}

#[derive(Debug, Clone)]
pub struct ProphetMemory {
    pub episodic: Vec<Episode>,
    pub core: Vec<CoreTrace>,
    pub threshold: f64,
    pub ep_cap: usize,
    pub core_cap: usize,
}

impl Default for ProphetMemory {
    fn default() -> Self {
        Self {
            episodic: Vec::new(),
            core: Vec::new(),
            threshold: DEFAULT_THRESHOLD,
            ep_cap: DEFAULT_EP_CAP,
            core_cap: DEFAULT_CORE_CAP,
        }
    }
}

pub type MemoryHandle = Rc<RefCell<ProphetMemory>>;

pub fn new_memory() -> MemoryHandle {
    Rc::new(RefCell::new(ProphetMemory::default()))
}

pub fn new_memory_config(threshold: f64, ep_cap: i64, core_cap: i64) -> Result<MemoryHandle> {
    if !(0.0..=10.0).contains(&threshold) {
        return Err(KengaError::new("memory threshold out of range", None));
    }
    if ep_cap < 1 || core_cap < 1 {
        return Err(KengaError::new("memory capacities must be >= 1", None));
    }
    Ok(Rc::new(RefCell::new(ProphetMemory {
        episodic: Vec::new(),
        core: Vec::new(),
        threshold,
        ep_cap: ep_cap as usize,
        core_cap: core_cap as usize,
    })))
}

pub fn value_to_pattern(v: &Value) -> Result<Vec<f64>> {
    match v {
        Value::List(xs) => {
            let mut out = Vec::with_capacity(xs.len());
            for x in xs {
                out.push(scalar(x)?);
            }
            Ok(out)
        }
        Value::Tensor { data, .. } => Ok(data.clone()),
        Value::I64(n) => Ok(vec![*n as f64]),
        Value::F64(n) => Ok(vec![*n]),
        _ => Err(KengaError::new(
            "memory pattern expects list, tensor, or number",
            None,
        )),
    }
}

fn scalar(v: &Value) -> Result<f64> {
    match v {
        Value::I64(n) => Ok(*n as f64),
        Value::F64(n) => Ok(*n),
        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        _ => Err(KengaError::new("pattern elements must be numeric", None)),
    }
}

pub fn pattern_to_list(p: &[f64]) -> Value {
    Value::List(p.iter().map(|x| Value::F64(*x)).collect())
}

/// Euclidean distance, length-normalized.
pub fn surprise_score(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().max(b.len()).max(1) as f64;
    let mut sum = 0.0;
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).copied().unwrap_or(0.0);
        let y = b.get(i).copied().unwrap_or(0.0);
        let d = x - y;
        sum += d * d;
    }
    (sum / n).sqrt()
}

fn nearest_core(mem: &ProphetMemory, pattern: &[f64]) -> Option<(usize, f64)> {
    let mut best: Option<(usize, f64)> = None;
    for (i, c) in mem.core.iter().enumerate() {
        let d = surprise_score(&c.pattern, pattern);
        if best.map(|(_, bd)| d < bd).unwrap_or(true) {
            best = Some((i, d));
        }
    }
    best
}

/// Predict latent continuation / expected state from core knowledge.
pub fn foresee(mem: &ProphetMemory, obs: &[f64]) -> Vec<f64> {
    if mem.core.is_empty() {
        return obs.to_vec();
    }
    // Softmax-ish blend of nearest cores
    let mut weights: Vec<(usize, f64)> = mem
        .core
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let d = surprise_score(&c.pattern, obs);
            let w = 1.0 / (0.05 + d);
            (i, w * (1.0 + c.importance))
        })
        .collect();
    weights.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let top = &weights[..weights.len().min(3)];
    let sum_w: f64 = top.iter().map(|(_, w)| *w).sum::<f64>().max(1e-9);
    let dim = top
        .iter()
        .map(|(i, _)| mem.core[*i].pattern.len())
        .max()
        .unwrap_or(obs.len())
        .max(obs.len());
    let mut out = vec![0.0; dim];
    for (i, w) in top {
        let p = &mem.core[*i].pattern;
        for d in 0..dim {
            out[d] += w * p.get(d).copied().unwrap_or(0.0);
        }
    }
    for x in &mut out {
        *x /= sum_w;
    }
    // Mild residual toward observation (online grounding)
    for (i, o) in obs.iter().enumerate() {
        if i < out.len() {
            out[i] = 0.7 * out[i] + 0.3 * *o;
        }
    }
    out
}

pub fn remember(mem: &mut ProphetMemory, pattern: Vec<f64>, surprise: f64, ts_ms: u64) -> bool {
    if surprise < mem.threshold {
        return false; // not surprising — like ignoring expected noise
    }
    mem.episodic.push(Episode {
        pattern,
        surprise,
        ts_ms,
    });
    // Cap: drop lowest surprise first
    while mem.episodic.len() > mem.ep_cap {
        let mut min_i = 0;
        let mut min_s = f64::MAX;
        for (i, e) in mem.episodic.iter().enumerate() {
            if e.surprise < min_s {
                min_s = e.surprise;
                min_i = i;
            }
        }
        mem.episodic.remove(min_i);
    }
    true
}

/// Sleep / consolidate: replay episodes into core with importance locks.
/// Returns how many episodes were folded into core.
pub fn consolidate(mem: &mut ProphetMemory) -> i64 {
    let mut folded = 0i64;
    let episodes = std::mem::take(&mut mem.episodic);
    for ep in episodes {
        folded += 1;
        if let Some((idx, dist)) = nearest_core(mem, &ep.pattern) {
            if dist < 0.35 {
                // Blend into existing core; learning rate shrinks with importance (EWC-lite)
                let lock = mem.core[idx].importance.max(0.0);
                let lr = 0.35 / (1.0 + lock);
                let dim = mem.core[idx]
                    .pattern
                    .len()
                    .max(ep.pattern.len());
                mem.core[idx].pattern.resize(dim, 0.0);
                for i in 0..dim {
                    let old = mem.core[idx].pattern[i];
                    let new = ep.pattern.get(i).copied().unwrap_or(old);
                    mem.core[idx].pattern[i] = old * (1.0 - lr) + new * lr;
                }
                mem.core[idx].importance += ep.surprise * 0.5;
                mem.core[idx].replays += 1;
                continue;
            }
        }
        // New law of the world
        mem.core.push(CoreTrace {
            pattern: ep.pattern,
            importance: ep.surprise,
            replays: 1,
        });
    }

    // Cap core: keep highest importance
    if mem.core.len() > mem.core_cap {
        mem.core
            .sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        mem.core.truncate(mem.core_cap);
    }
    folded
}

pub fn recall(mem: &ProphetMemory, query: &[f64], k: usize) -> Vec<Vec<f64>> {
    let mut scored: Vec<(f64, Vec<f64>)> = Vec::new();
    for c in &mem.core {
        scored.push((surprise_score(&c.pattern, query), c.pattern.clone()));
    }
    for e in &mem.episodic {
        scored.push((surprise_score(&e.pattern, query), e.pattern.clone()));
    }
    scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    scored
        .into_iter()
        .take(k.max(1))
        .map(|(_, p)| p)
        .collect()
}

pub fn stats(mem: &ProphetMemory) -> Value {
    let locked = mem.core.iter().filter(|c| c.importance >= 1.0).count() as i64;
    Value::List(vec![
        Value::I64(mem.episodic.len() as i64),
        Value::I64(mem.core.len() as i64),
        Value::I64(locked),
    ])
}
