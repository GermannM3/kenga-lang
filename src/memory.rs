//! Prophet memory: episodic buffer + core knowledge + trainable world weights.
//!
//! Surprise gates episodic storage. `consolidate` (sleep) replays into core
//! traces with EWC-lite locks AND updates a linear world model `W, b` so
//! `foresee` / `predict` improve over time without wiping old knowledge.

use std::cell::RefCell;
use std::rc::Rc;

use crate::bytecode::Value;
use crate::error::{KengaError, Result};

const DEFAULT_THRESHOLD: f64 = 0.15;
const DEFAULT_EP_CAP: usize = 64;
const DEFAULT_CORE_CAP: usize = 32;
const DEFAULT_LR: f64 = 0.08;

#[derive(Debug, Clone)]
pub struct Episode {
    pub pattern: Vec<f64>,
    /// Optional next-state target for world-model training
    pub target: Option<Vec<f64>>,
    pub surprise: f64,
    pub ts_ms: u64,
}

#[derive(Debug, Clone)]
pub struct CoreTrace {
    pub pattern: Vec<f64>,
    pub importance: f64,
    pub replays: u64,
}

/// Linear world model: y ≈ tanh(W x + b)
#[derive(Debug, Clone)]
pub struct WorldModel {
    pub dim: usize,
    /// row-major dim*dim
    pub w: Vec<f64>,
    pub b: Vec<f64>,
    /// Fisher-ish importance per weight (EWC)
    pub w_lock: Vec<f64>,
    pub b_lock: Vec<f64>,
    pub steps: u64,
}

impl WorldModel {
    pub fn new(dim: usize) -> Self {
        let n = dim * dim;
        // Small identity-ish init
        let mut w = vec![0.0; n];
        for i in 0..dim {
            w[i * dim + i] = 0.2;
        }
        Self {
            dim,
            w,
            b: vec![0.0; dim],
            w_lock: vec![0.0; n],
            b_lock: vec![0.0; dim],
            steps: 0,
        }
    }

    pub fn ensure_dim(&mut self, dim: usize) {
        if dim <= self.dim {
            return;
        }
        let old = self.dim;
        let mut nw = vec![0.0; dim * dim];
        let mut nw_lock = vec![0.0; dim * dim];
        for r in 0..old {
            for c in 0..old {
                nw[r * dim + c] = self.w[r * old + c];
                nw_lock[r * dim + c] = self.w_lock[r * old + c];
            }
            if r < dim {
                nw[r * dim + r] = nw[r * dim + r].max(0.2);
            }
        }
        for i in old..dim {
            nw[i * dim + i] = 0.2;
        }
        let mut nb = vec![0.0; dim];
        let mut nb_lock = vec![0.0; dim];
        nb[..old].copy_from_slice(&self.b);
        nb_lock[..old].copy_from_slice(&self.b_lock);
        self.dim = dim;
        self.w = nw;
        self.w_lock = nw_lock;
        self.b = nb;
        self.b_lock = nb_lock;
    }

    fn forward(&self, x: &[f64]) -> Vec<f64> {
        let d = self.dim;
        let mut y = vec![0.0; d];
        for r in 0..d {
            let mut s = self.b[r];
            for c in 0..d {
                let xv = x.get(c).copied().unwrap_or(0.0);
                s += self.w[r * d + c] * xv;
            }
            y[r] = s.tanh();
        }
        y
    }

    /// Delta rule with EWC penalty on locked weights.
    pub fn train_step(&mut self, x: &[f64], target: &[f64], lr: f64) -> f64 {
        let d = x.len().max(target.len()).max(1);
        self.ensure_dim(d);
        let pred = self.forward(x);
        let mut loss = 0.0;
        let mut delta = vec![0.0; self.dim];
        for i in 0..self.dim {
            let t = target.get(i).copied().unwrap_or(0.0);
            // Map target into tanh range softly
            let t = t.tanh();
            let y = pred[i];
            let err = t - y;
            loss += err * err;
            // d(tanh)/dz = 1 - y^2
            delta[i] = err * (1.0 - y * y);
        }
        loss /= self.dim as f64;

        for r in 0..self.dim {
            for c in 0..self.dim {
                let xv = x.get(c).copied().unwrap_or(0.0);
                let idx = r * self.dim + c;
                let lock = self.w_lock[idx];
                let eff_lr = lr / (1.0 + lock);
                self.w[idx] += eff_lr * delta[r] * xv;
                // accumulate importance ~ |grad|
                self.w_lock[idx] += 0.05 * (delta[r] * xv).abs();
            }
            let lock = self.b_lock[r];
            let eff_lr = lr / (1.0 + lock);
            self.b[r] += eff_lr * delta[r];
            self.b_lock[r] += 0.05 * delta[r].abs();
        }
        self.steps += 1;
        loss
    }
}

#[derive(Debug, Clone)]
pub struct ProphetMemory {
    pub episodic: Vec<Episode>,
    pub core: Vec<CoreTrace>,
    pub model: WorldModel,
    pub threshold: f64,
    pub ep_cap: usize,
    pub core_cap: usize,
    pub lr: f64,
}

impl Default for ProphetMemory {
    fn default() -> Self {
        Self {
            episodic: Vec::new(),
            core: Vec::new(),
            model: WorldModel::new(4),
            threshold: DEFAULT_THRESHOLD,
            ep_cap: DEFAULT_EP_CAP,
            core_cap: DEFAULT_CORE_CAP,
            lr: DEFAULT_LR,
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
        model: WorldModel::new(4),
        threshold,
        ep_cap: ep_cap as usize,
        core_cap: core_cap as usize,
        lr: DEFAULT_LR,
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

fn cmp_f64(a: f64, b: f64) -> std::cmp::Ordering {
    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
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

fn blend_core(mem: &ProphetMemory, obs: &[f64]) -> Option<Vec<f64>> {
    if mem.core.is_empty() {
        return None;
    }
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
    weights.sort_by(|a, b| cmp_f64(b.1, a.1));
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
    Some(out)
}

/// Neural prediction from world model weights.
pub fn predict(mem: &ProphetMemory, obs: &[f64]) -> Vec<f64> {
    let mut m = mem.model.clone();
    m.ensure_dim(obs.len().max(1));
    // Use forward on a temp ensure — but model is behind & — so reimplement read-only:
    let d = mem.model.dim.max(obs.len()).max(1);
    // If dims mismatch, pad locally
    if d == mem.model.dim {
        return mem.model.forward(obs);
    }
    let mut tmp = mem.model.clone();
    tmp.ensure_dim(d);
    tmp.forward(obs)
}

/// Hybrid foresee: neural predict blended with core traces.
pub fn foresee(mem: &ProphetMemory, obs: &[f64]) -> Vec<f64> {
    let neural = predict(mem, obs);
    match blend_core(mem, obs) {
        None => {
            if mem.model.steps == 0 {
                obs.to_vec()
            } else {
                neural
            }
        }
        Some(core) => {
            let dim = neural.len().max(core.len()).max(obs.len());
            let mut out = vec![0.0; dim];
            let nw = if mem.model.steps > 0 { 0.55 } else { 0.15 };
            let cw = 1.0 - nw;
            for i in 0..dim {
                let n = neural.get(i).copied().unwrap_or(0.0);
                let c = core.get(i).copied().unwrap_or(0.0);
                let o = obs.get(i).copied().unwrap_or(0.0);
                out[i] = nw * n + cw * c * 0.85 + o * 0.15;
            }
            out
        }
    }
}

pub fn remember(
    mem: &mut ProphetMemory,
    pattern: Vec<f64>,
    surprise: f64,
    ts_ms: u64,
) -> bool {
    remember_pair(mem, pattern, None, surprise, ts_ms)
}

pub fn remember_pair(
    mem: &mut ProphetMemory,
    pattern: Vec<f64>,
    target: Option<Vec<f64>>,
    surprise: f64,
    ts_ms: u64,
) -> bool {
    if surprise < mem.threshold {
        return false;
    }
    mem.model.ensure_dim(pattern.len().max(target.as_ref().map(|t| t.len()).unwrap_or(0)));
    mem.episodic.push(Episode {
        pattern,
        target,
        surprise,
        ts_ms,
    });
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

/// Explicit supervised step on world weights.
pub fn learn(mem: &mut ProphetMemory, x: &[f64], y: &[f64]) -> f64 {
    mem.model.ensure_dim(x.len().max(y.len()).max(1));
    mem.model.train_step(x, y, mem.lr)
}

/// Sleep / consolidate: core fold + world-model replay training.
pub fn consolidate(mem: &mut ProphetMemory) -> i64 {
    let mut folded = 0i64;
    let episodes = std::mem::take(&mut mem.episodic);
    for ep in &episodes {
        folded += 1;
        // Train world model: target = next state or self-pattern
        let target = ep.target.as_ref().unwrap_or(&ep.pattern);
        mem.model.ensure_dim(ep.pattern.len().max(target.len()));
        mem.model.train_step(&ep.pattern, target, mem.lr);

        if let Some((idx, dist)) = nearest_core(mem, &ep.pattern) {
            if dist < 0.35 {
                let lock = mem.core[idx].importance.max(0.0);
                let lr = 0.35 / (1.0 + lock);
                let dim = mem.core[idx].pattern.len().max(ep.pattern.len());
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
        mem.core.push(CoreTrace {
            pattern: ep.pattern.clone(),
            importance: ep.surprise,
            replays: 1,
        });
    }

    // Extra dream replays on core (stabilize locks)
    let core_snap: Vec<Vec<f64>> = mem.core.iter().map(|c| c.pattern.clone()).collect();
    for p in core_snap {
        mem.model.train_step(&p, &p, mem.lr * 0.5);
    }

    if mem.core.len() > mem.core_cap {
        mem.core
            .sort_by(|a, b| cmp_f64(b.importance, a.importance));
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
    scored.sort_by(|a, b| cmp_f64(a.0, b.0));
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
        Value::I64(mem.model.steps as i64),
        Value::I64(mem.model.dim as i64),
    ])
}
