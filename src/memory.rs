//! Prophet memory: episodic + core + MLP world model + multi-step unroll.
//!
//! World model (residual): y = x + W2·tanh(W1·x + b1) + b2, EWC-lite locks.
//! Linear residual head so dynamics like x→x+1 actually learn (no output tanh).
//! `unroll` rolls the model forward N steps (разворот будущего).

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

/// Two-layer MLP world model with EWC locks.
#[derive(Debug, Clone)]
pub struct WorldModel {
    pub dim: usize,
    pub hidden: usize,
    /// W1: hidden x dim (row-major)
    pub w1: Vec<f64>,
    pub b1: Vec<f64>,
    /// W2: dim x hidden
    pub w2: Vec<f64>,
    pub b2: Vec<f64>,
    pub w1_lock: Vec<f64>,
    pub b1_lock: Vec<f64>,
    pub w2_lock: Vec<f64>,
    pub b2_lock: Vec<f64>,
    pub steps: u64,
}

fn hidden_for(dim: usize) -> usize {
    (dim * 3).max(8).min(64)
}

fn xavier(fan_in: usize, fan_out: usize, i: usize) -> f64 {
    let s = (6.0 / (fan_in + fan_out) as f64).sqrt();
    // deterministic pseudo-noise from index
    let u = ((i * 1103515245 + 12345) % 10000) as f64 / 10000.0;
    (u * 2.0 - 1.0) * s
}

struct Fwd {
    h: Vec<f64>,
    y: Vec<f64>,
}

impl WorldModel {
    pub fn new(dim: usize) -> Self {
        let dim = dim.max(1);
        let h = hidden_for(dim);
        let mut m = Self {
            dim,
            hidden: h,
            w1: vec![0.0; h * dim],
            b1: vec![0.0; h],
            w2: vec![0.0; dim * h],
            b2: vec![0.0; dim],
            w1_lock: vec![0.0; h * dim],
            b1_lock: vec![0.0; h],
            w2_lock: vec![0.0; dim * h],
            b2_lock: vec![0.0; dim],
            steps: 0,
        };
        m.init_weights();
        m
    }

    fn init_weights(&mut self) {
        let (d, h) = (self.dim, self.hidden);
        for i in 0..(h * d) {
            self.w1[i] = xavier(d, h, i);
        }
        // small delta head — residual starts near identity
        for i in 0..(d * h) {
            self.w2[i] = xavier(h, d, i + 17) * 0.25;
        }
    }

    pub fn ensure_dim(&mut self, dim: usize) {
        let dim = dim.max(1);
        if dim <= self.dim {
            return;
        }
        // Grow: rebuild with larger dims, copy overlapping block
        let old = self.clone();
        *self = WorldModel::new(dim);
        let od = old.dim;
        let oh = old.hidden;
        let nh = self.hidden;
        for r in 0..oh.min(nh) {
            for c in 0..od {
                if r * self.dim + c < self.w1.len() && r * od + c < old.w1.len() {
                    self.w1[r * self.dim + c] = old.w1[r * od + c];
                    self.w1_lock[r * self.dim + c] = old.w1_lock[r * od + c];
                }
            }
            self.b1[r] = old.b1[r];
            self.b1_lock[r] = old.b1_lock[r];
        }
        for r in 0..od {
            for c in 0..oh.min(nh) {
                self.w2[r * nh + c] = old.w2[r * oh + c];
                self.w2_lock[r * nh + c] = old.w2_lock[r * oh + c];
            }
            self.b2[r] = old.b2[r];
            self.b2_lock[r] = old.b2_lock[r];
        }
        self.steps = old.steps;
    }

    fn forward_act(&self, x: &[f64]) -> Fwd {
        let (d, h) = (self.dim, self.hidden);
        let mut hh = vec![0.0; h];
        for r in 0..h {
            let mut s = self.b1[r];
            for c in 0..d {
                s += self.w1[r * d + c] * x.get(c).copied().unwrap_or(0.0);
            }
            hh[r] = s.tanh();
        }
        // residual: prediction = input + learned delta
        let mut y = vec![0.0; d];
        for r in 0..d {
            let mut delta = self.b2[r];
            for c in 0..h {
                delta += self.w2[r * h + c] * hh[c];
            }
            y[r] = x.get(r).copied().unwrap_or(0.0) + delta;
        }
        Fwd { h: hh, y }
    }

    pub fn forward(&self, x: &[f64]) -> Vec<f64> {
        self.forward_act(x).y
    }

    pub fn train_step(&mut self, x: &[f64], target: &[f64], lr: f64) -> f64 {
        let d = x.len().max(target.len()).max(1);
        self.ensure_dim(d);
        let act = self.forward_act(x);
        let (d, h) = (self.dim, self.hidden);

        let mut loss = 0.0;
        let mut dy = vec![0.0; d];
        for i in 0..d {
            let t = target.get(i).copied().unwrap_or(0.0);
            let err = t - act.y[i];
            loss += err * err;
            // linear residual head: dL/ddelta = -(t - y)
            dy[i] = err;
        }
        loss /= d as f64;

        let mut dh = vec![0.0; h];
        for c in 0..h {
            let mut s = 0.0;
            for r in 0..d {
                s += self.w2[r * h + c] * dy[r];
            }
            dh[c] = s * (1.0 - act.h[c] * act.h[c]);
        }

        for r in 0..d {
            for c in 0..h {
                let idx = r * h + c;
                let g = dy[r] * act.h[c];
                let eff = lr / (1.0 + self.w2_lock[idx]);
                self.w2[idx] += eff * g;
                self.w2_lock[idx] += 0.02 * g.abs();
            }
            let eff = lr / (1.0 + self.b2_lock[r]);
            self.b2[r] += eff * dy[r];
            self.b2_lock[r] += 0.02 * dy[r].abs();
        }

        for r in 0..h {
            for c in 0..d {
                let idx = r * d + c;
                let xv = x.get(c).copied().unwrap_or(0.0);
                let g = dh[r] * xv;
                let eff = lr / (1.0 + self.w1_lock[idx]);
                self.w1[idx] += eff * g;
                self.w1_lock[idx] += 0.02 * g.abs();
            }
            let eff = lr / (1.0 + self.b1_lock[r]);
            self.b1[r] += eff * dh[r];
            self.b1_lock[r] += 0.02 * dh[r].abs();
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
            model: WorldModel::new(1),
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
        model: WorldModel::new(1),
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
            (i, (1.0 / (0.05 + d)) * (1.0 + c.importance))
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

pub fn predict(mem: &ProphetMemory, obs: &[f64]) -> Vec<f64> {
    let mut tmp = mem.model.clone();
    tmp.ensure_dim(obs.len().max(1));
    // Use ensured model without mutating original when dims match after clone
    if mem.model.dim >= obs.len().max(1) {
        mem.model.forward(obs)
    } else {
        tmp.forward(obs)
    }
}

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
            let nw = if mem.model.steps > 0 { 0.6 } else { 0.15 };
            let cw = 1.0 - nw;
            for i in 0..dim {
                let n = neural.get(i).copied().unwrap_or(0.0);
                let c = core.get(i).copied().unwrap_or(0.0);
                let o = obs.get(i).copied().unwrap_or(0.0);
                out[i] = nw * n + cw * c * 0.8 + o * 0.2;
            }
            out
        }
    }
}

/// Multi-step future unroll (neural dynamics).
/// Returns list of states for t+1 .. t+steps.
pub fn unroll(mem: &ProphetMemory, obs: &[f64], steps: usize) -> Vec<Vec<f64>> {
    let steps = steps.clamp(1, 64);
    let mut cur = obs.to_vec();
    let mut out = Vec::with_capacity(steps);
    for _ in 0..steps {
        cur = predict(mem, &cur);
        out.push(cur.clone());
    }
    out
}

/// Hybrid multi-step (foresee each step).
pub fn foresee_n(mem: &ProphetMemory, obs: &[f64], steps: usize) -> Vec<Vec<f64>> {
    let steps = steps.clamp(1, 64);
    let mut cur = obs.to_vec();
    let mut out = Vec::with_capacity(steps);
    for _ in 0..steps {
        cur = foresee(mem, &cur);
        out.push(cur.clone());
    }
    out
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
    mem.model
        .ensure_dim(pattern.len().max(target.as_ref().map(|t| t.len()).unwrap_or(0)));
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

pub fn learn(mem: &mut ProphetMemory, x: &[f64], y: &[f64]) -> f64 {
    mem.model.ensure_dim(x.len().max(y.len()).max(1));
    mem.model.train_step(x, y, mem.lr)
}

pub fn consolidate(mem: &mut ProphetMemory) -> i64 {
    let mut folded = 0i64;
    let episodes = std::mem::take(&mut mem.episodic);
    for ep in &episodes {
        folded += 1;
        let target = ep.target.as_ref().unwrap_or(&ep.pattern);
        mem.model.ensure_dim(ep.pattern.len().max(target.len()));
        // primary transition + one extra replay scaled by surprise
        mem.model.train_step(&ep.pattern, target, mem.lr);
        if ep.surprise > 0.4 {
            mem.model
                .train_step(&ep.pattern, target, mem.lr * 0.5);
        }

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
        Value::I64(mem.model.hidden as i64),
    ])
}

fn write_f64s(out: &mut String, xs: &[f64]) {
    for (i, x) in xs.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&format!("{x}"));
    }
    out.push('\n');
}

fn parse_f64s(line: &str) -> Result<Vec<f64>> {
    line.split_whitespace()
        .map(|t| {
            t.parse::<f64>()
                .map_err(|_| KengaError::new(format!("bad f64 '{t}' in mind file"), None))
        })
        .collect()
}

fn parse_u64_field(line: &str, key: &str) -> Result<u64> {
    let parts: Vec<_> = line.split_whitespace().collect();
    if parts.len() != 2 || parts[0] != key {
        return Err(KengaError::new(
            format!("expected '{key} <n>', got '{line}'"),
            None,
        ));
    }
    parts[1]
        .parse()
        .map_err(|_| KengaError::new(format!("bad u64 for {key}"), None))
}

fn parse_f64_field(line: &str, key: &str) -> Result<f64> {
    let parts: Vec<_> = line.split_whitespace().collect();
    if parts.len() != 2 || parts[0] != key {
        return Err(KengaError::new(
            format!("expected '{key} <n>', got '{line}'"),
            None,
        ));
    }
    parts[1]
        .parse()
        .map_err(|_| KengaError::new(format!("bad f64 for {key}"), None))
}

fn parse_usize_field(line: &str, key: &str) -> Result<usize> {
    Ok(parse_u64_field(line, key)? as usize)
}

/// Serialize Prophet memory to a portable text mind file.
pub fn save_mind(mem: &ProphetMemory, path: &std::path::Path) -> Result<()> {
    let mut out = String::new();
    out.push_str("KENGA_MIND 1\n");
    out.push_str(&format!("threshold {}\n", mem.threshold));
    out.push_str(&format!("ep_cap {}\n", mem.ep_cap));
    out.push_str(&format!("core_cap {}\n", mem.core_cap));
    out.push_str(&format!("lr {}\n", mem.lr));
    out.push_str(&format!(
        "model {} {} {}\n",
        mem.model.dim, mem.model.hidden, mem.model.steps
    ));
    write_f64s(&mut out, &mem.model.w1);
    write_f64s(&mut out, &mem.model.b1);
    write_f64s(&mut out, &mem.model.w2);
    write_f64s(&mut out, &mem.model.b2);
    write_f64s(&mut out, &mem.model.w1_lock);
    write_f64s(&mut out, &mem.model.b1_lock);
    write_f64s(&mut out, &mem.model.w2_lock);
    write_f64s(&mut out, &mem.model.b2_lock);

    out.push_str(&format!("core {}\n", mem.core.len()));
    for c in &mem.core {
        out.push_str(&format!("{} {} {}\n", c.importance, c.replays, c.pattern.len()));
        write_f64s(&mut out, &c.pattern);
    }

    out.push_str(&format!("episodic {}\n", mem.episodic.len()));
    for e in &mem.episodic {
        let has_t = if e.target.is_some() { 1 } else { 0 };
        out.push_str(&format!(
            "{} {} {} {} {}\n",
            e.surprise,
            e.ts_ms,
            e.pattern.len(),
            has_t,
            e.target.as_ref().map(|t| t.len()).unwrap_or(0)
        ));
        write_f64s(&mut out, &e.pattern);
        if let Some(t) = &e.target {
            write_f64s(&mut out, t);
        }
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                KengaError::new(format!("cannot create {}: {e}", parent.display()), None)
            })?;
        }
    }
    std::fs::write(path, out).map_err(|e| {
        KengaError::new(format!("cannot write {}: {e}", path.display()), None)
    })?;
    Ok(())
}

/// Load Prophet memory from a mind file.
pub fn load_mind(path: &std::path::Path) -> Result<MemoryHandle> {
    let raw = std::fs::read_to_string(path).map_err(|e| {
        KengaError::new(format!("cannot read {}: {e}", path.display()), None)
    })?;
    let mut lines = raw.lines().filter(|l| !l.trim().is_empty());
    let magic = lines
        .next()
        .ok_or_else(|| KengaError::new("empty mind file", None))?;
    if magic != "KENGA_MIND 1" {
        return Err(KengaError::new(
            format!("unsupported mind format: {magic}"),
            None,
        ));
    }

    let threshold = parse_f64_field(
        lines
            .next()
            .ok_or_else(|| KengaError::new("missing threshold", None))?,
        "threshold",
    )?;
    let ep_cap = parse_usize_field(
        lines
            .next()
            .ok_or_else(|| KengaError::new("missing ep_cap", None))?,
        "ep_cap",
    )?;
    let core_cap = parse_usize_field(
        lines
            .next()
            .ok_or_else(|| KengaError::new("missing core_cap", None))?,
        "core_cap",
    )?;
    let lr = parse_f64_field(
        lines
            .next()
            .ok_or_else(|| KengaError::new("missing lr", None))?,
        "lr",
    )?;

    let model_line = lines
        .next()
        .ok_or_else(|| KengaError::new("missing model header", None))?;
    let mp: Vec<_> = model_line.split_whitespace().collect();
    if mp.len() != 4 || mp[0] != "model" {
        return Err(KengaError::new(
            format!("bad model header: {model_line}"),
            None,
        ));
    }
    let dim: usize = mp[1]
        .parse()
        .map_err(|_| KengaError::new("bad model dim", None))?;
    let hidden: usize = mp[2]
        .parse()
        .map_err(|_| KengaError::new("bad model hidden", None))?;
    let steps: u64 = mp[3]
        .parse()
        .map_err(|_| KengaError::new("bad model steps", None))?;

    let rest: Vec<&str> = lines.collect();
    let mut i = 0usize;
    let take = |i: &mut usize, n: usize, what: &str| -> Result<Vec<f64>> {
        let line = rest
            .get(*i)
            .ok_or_else(|| KengaError::new(format!("missing {what}"), None))?;
        *i += 1;
        let xs = parse_f64s(line)?;
        if xs.len() != n {
            return Err(KengaError::new(
                format!("{what}: expected {n} floats, got {}", xs.len()),
                None,
            ));
        }
        Ok(xs)
    };

    let w1 = take(&mut i, hidden * dim, "w1")?;
    let b1 = take(&mut i, hidden, "b1")?;
    let w2 = take(&mut i, dim * hidden, "w2")?;
    let b2 = take(&mut i, dim, "b2")?;
    let w1_lock = take(&mut i, hidden * dim, "w1_lock")?;
    let b1_lock = take(&mut i, hidden, "b1_lock")?;
    let w2_lock = take(&mut i, dim * hidden, "w2_lock")?;
    let b2_lock = take(&mut i, dim, "b2_lock")?;

    let core_hdr = rest
        .get(i)
        .ok_or_else(|| KengaError::new("missing core header", None))?;
    i += 1;
    let core_n = parse_usize_field(core_hdr, "core")?;
    let mut core = Vec::with_capacity(core_n);
    for _ in 0..core_n {
        let meta = rest
            .get(i)
            .ok_or_else(|| KengaError::new("missing core meta", None))?;
        i += 1;
        let p: Vec<_> = meta.split_whitespace().collect();
        if p.len() != 3 {
            return Err(KengaError::new(format!("bad core meta: {meta}"), None));
        }
        let importance: f64 = p[0].parse().map_err(|_| KengaError::new("bad importance", None))?;
        let replays: u64 = p[1].parse().map_err(|_| KengaError::new("bad replays", None))?;
        let plen: usize = p[2].parse().map_err(|_| KengaError::new("bad pattern len", None))?;
        let pattern = take(&mut i, plen, "core pattern")?;
        core.push(CoreTrace {
            pattern,
            importance,
            replays,
        });
    }

    let ep_hdr = rest
        .get(i)
        .ok_or_else(|| KengaError::new("missing episodic header", None))?;
    i += 1;
    let ep_n = parse_usize_field(ep_hdr, "episodic")?;
    let mut episodic = Vec::with_capacity(ep_n);
    for _ in 0..ep_n {
        let meta = rest
            .get(i)
            .ok_or_else(|| KengaError::new("missing episode meta", None))?;
        i += 1;
        let p: Vec<_> = meta.split_whitespace().collect();
        if p.len() != 5 {
            return Err(KengaError::new(format!("bad episode meta: {meta}"), None));
        }
        let surprise: f64 = p[0].parse().map_err(|_| KengaError::new("bad surprise", None))?;
        let ts_ms: u64 = p[1].parse().map_err(|_| KengaError::new("bad ts", None))?;
        let plen: usize = p[2].parse().map_err(|_| KengaError::new("bad plen", None))?;
        let has_t: u64 = p[3].parse().map_err(|_| KengaError::new("bad has_t", None))?;
        let tlen: usize = p[4].parse().map_err(|_| KengaError::new("bad tlen", None))?;
        let pattern = take(&mut i, plen, "episode pattern")?;
        let target = if has_t == 1 {
            Some(take(&mut i, tlen, "episode target")?)
        } else {
            None
        };
        episodic.push(Episode {
            pattern,
            target,
            surprise,
            ts_ms,
        });
    }

    let mem = ProphetMemory {
        episodic,
        core,
        model: WorldModel {
            dim,
            hidden,
            w1,
            b1,
            w2,
            b2,
            w1_lock,
            b1_lock,
            w2_lock,
            b2_lock,
            steps,
        },
        threshold,
        ep_cap,
        core_cap,
        lr,
    };
    Ok(Rc::new(RefCell::new(mem)))
}

/// Helper used by talk REPL / tests.
pub fn train_physics_epoch(mem: &mut ProphetMemory) -> f64 {
    let mut loss = 0.0f64;
    let mut n = 0.0f64;
    for pos in 0..14 {
        for vel in 0..3i64 {
            let v = vel - 1;
            let fuel = 9i64;
            let x = vec![pos as f64, v as f64, fuel as f64];
            let y = vec![(pos + v) as f64, v as f64, (fuel - 1) as f64];
            remember_pair(mem, x.clone(), Some(y.clone()), 0.5, 0);
            loss += learn(mem, &x, &y);
            n += 1.0;
        }
    }
    loss / n.max(1.0)
}
