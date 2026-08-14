use std::collections::{HashMap, VecDeque};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::bytecode::{FunctionChunk, Module, Op, Value};
use crate::error::{KengaError, Result};

#[derive(Clone)]
struct Slot {
    value: Value,
    expires_at_ms: Option<u64>,
}

struct Frame {
    ip: usize,
    locals: HashMap<String, Slot>,
    stack_base: usize,
    ops: Vec<Op>,
}

enum Step {
    /// Keep running
    Continue,
    /// Top-level program finished
    Finished(Value),
}

pub struct Vm {
    module: Module,
    stack: Vec<Value>,
    frames: Vec<Frame>,
    globals: HashMap<String, Slot>,
    listeners: HashMap<String, String>,
    events: VecDeque<(String, Value)>,
}

impl Vm {
    pub fn new(module: Module) -> Self {
        let mut listeners = HashMap::new();
        for (event, handler) in &module.handlers {
            listeners.insert(event.clone(), handler.clone());
        }
        Self {
            module,
            stack: Vec::new(),
            frames: Vec::new(),
            globals: HashMap::new(),
            listeners,
            events: VecDeque::new(),
        }
    }

    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    fn sweep(&mut self) {
        let now = Self::now_ms();
        self.globals.retain(|_, slot| match slot.expires_at_ms {
            Some(t) => t > now,
            None => true,
        });
        if let Some(frame) = self.frames.last_mut() {
            frame.locals.retain(|_, slot| match slot.expires_at_ms {
                Some(t) => t > now,
                None => true,
            });
        }
    }

    fn store(&mut self, name: String, value: Value, ttl_ms: Option<u64>) {
        let expires_at_ms = ttl_ms.map(|ms| Self::now_ms().saturating_add(ms));
        let slot = Slot {
            value,
            expires_at_ms,
        };
        if let Some(frame) = self.frames.last_mut() {
            frame.locals.insert(name, slot);
        } else {
            self.globals.insert(name, slot);
        }
    }

    fn load(&self, name: &str) -> Result<Value> {
        let now = Self::now_ms();
        if let Some(frame) = self.frames.last() {
            if let Some(slot) = frame.locals.get(name) {
                if let Some(t) = slot.expires_at_ms {
                    if t <= now {
                        return Err(KengaError::new(
                            format!("living value '{name}' expired (ttl)"),
                            None,
                        ));
                    }
                }
                return Ok(slot.value.clone());
            }
        }
        if let Some(slot) = self.globals.get(name) {
            if let Some(t) = slot.expires_at_ms {
                if t <= now {
                    return Err(KengaError::new(
                        format!("living value '{name}' expired (ttl)"),
                        None,
                    ));
                }
            }
            return Ok(slot.value.clone());
        }
        Err(KengaError::new(
            format!("undefined variable '{name}'"),
            None,
        ))
    }

    fn pop(&mut self) -> Result<Value> {
        self.stack
            .pop()
            .ok_or_else(|| KengaError::new("stack underflow", None))
    }

    fn push_frame(&mut self, chunk: &FunctionChunk, args: Vec<Value>) {
        let base = self.stack.len();
        for a in args {
            self.stack.push(a);
        }
        self.frames.push(Frame {
            ip: 0,
            locals: HashMap::new(),
            stack_base: base,
            ops: chunk.ops.clone(),
        });
    }

    /// Run callee until it returns to the previous frame depth.
    fn call_sync(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        let callee = self
            .module
            .functions
            .get(name)
            .cloned()
            .ok_or_else(|| KengaError::new(format!("undefined function '{name}'"), None))?;
        if callee.arity != args.len() {
            return Err(KengaError::new(
                format!(
                    "function '{name}' expects {} args, got {}",
                    callee.arity,
                    args.len()
                ),
                None,
            ));
        }
        let depth = self.frames.len();
        self.push_frame(&callee, args);
        while self.frames.len() > depth {
            match self.step()? {
                Step::Continue => {}
                Step::Finished(v) => return Ok(v),
            }
        }
        self.pop()
    }

    pub fn run(&mut self) -> Result<Value> {
        let main = self
            .module
            .functions
            .get("main")
            .ok_or_else(|| KengaError::new("missing main", None))?
            .clone();
        self.push_frame(&main, Vec::new());
        loop {
            match self.step()? {
                Step::Continue => {}
                Step::Finished(v) => return Ok(v),
            }
        }
    }

    fn step(&mut self) -> Result<Step> {
        let (ip, op) = {
            let frame = self
                .frames
                .last()
                .ok_or_else(|| KengaError::new("no frame", None))?;
            if frame.ip >= frame.ops.len() {
                let frame = self.frames.pop().unwrap();
                self.stack.truncate(frame.stack_base);
                if self.frames.is_empty() {
                    return Ok(Step::Finished(Value::Nil));
                }
                self.stack.push(Value::Nil);
                return Ok(Step::Continue);
            }
            let ip = frame.ip;
            let op = frame.ops[ip].clone();
            (ip, op)
        };
        self.frames.last_mut().unwrap().ip = ip + 1;

        match op {
            Op::Const(v) => self.stack.push(v),
            Op::Load(name) => {
                let v = self.load(&name)?;
                self.stack.push(v);
            }
            Op::Store { name, ttl_ms } => {
                let v = self.pop()?;
                self.store(name, v, ttl_ms);
            }
            Op::Pop => {
                self.stack.pop();
            }
            Op::Add => self.bin_num(|a, b| a + b, |a, b| a + b)?,
            Op::Sub => self.bin_num(|a, b| a - b, |a, b| a - b)?,
            Op::Mul => self.bin_num(|a, b| a * b, |a, b| a * b)?,
            Op::Div => self.bin_num(|a, b| a / b, |a, b| a / b)?,
            Op::Rem => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (Value::I64(x), Value::I64(y)) => self.stack.push(Value::I64(x % y)),
                    _ => return Err(KengaError::new("% expects i64", None)),
                }
            }
            Op::Neg => match self.pop()? {
                Value::I64(n) => self.stack.push(Value::I64(-n)),
                Value::F64(n) => self.stack.push(Value::F64(-n)),
                _ => return Err(KengaError::new("unary - on non-number", None)),
            },
            Op::Not => {
                let a = self.pop()?;
                self.stack.push(Value::Bool(!a.as_bool()));
            }
            Op::Eq => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(values_eq(&a, &b)));
            }
            Op::Ne => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(!values_eq(&a, &b)));
            }
            Op::Lt => self.bin_cmp(|a, b| a < b, |a, b| a < b)?,
            Op::Le => self.bin_cmp(|a, b| a <= b, |a, b| a <= b)?,
            Op::Gt => self.bin_cmp(|a, b| a > b, |a, b| a > b)?,
            Op::Ge => self.bin_cmp(|a, b| a >= b, |a, b| a >= b)?,
            Op::And => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack
                    .push(Value::Bool(a.as_bool() && b.as_bool()));
            }
            Op::Or => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack
                    .push(Value::Bool(a.as_bool() || b.as_bool()));
            }
            Op::Jump(target) => {
                self.frames.last_mut().unwrap().ip = target;
            }
            Op::JumpIfFalse(target) => {
                let c = self.pop()?;
                if !c.as_bool() {
                    self.frames.last_mut().unwrap().ip = target;
                }
            }
            Op::Call { name, argc } => {
                if name == "now_ms" {
                    self.stack.push(Value::I64(Self::now_ms() as i64));
                    return Ok(Step::Continue);
                }
                let mut args = Vec::with_capacity(argc);
                for _ in 0..argc {
                    args.push(self.pop()?);
                }
                args.reverse();
                let callee = self
                    .module
                    .functions
                    .get(&name)
                    .cloned()
                    .ok_or_else(|| {
                        KengaError::new(format!("undefined function '{name}'"), None)
                    })?;
                if callee.arity != argc {
                    return Err(KengaError::new(
                        format!(
                            "function '{name}' expects {} args, got {argc}",
                            callee.arity
                        ),
                        None,
                    ));
                }
                self.push_frame(&callee, args);
            }
            Op::Return => {
                let v = self.pop().unwrap_or(Value::Nil);
                let frame = self.frames.pop().unwrap();
                self.stack.truncate(frame.stack_base);
                if self.frames.is_empty() {
                    return Ok(Step::Finished(v));
                }
                self.stack.push(v);
            }
            Op::Print => {
                let v = self.pop()?;
                print!("{}", v.display());
            }
            Op::Println => {
                let v = self.pop()?;
                println!("{}", v.display());
            }
            Op::MakeTensor { dims } => {
                let mut shape = Vec::with_capacity(dims);
                for _ in 0..dims {
                    match self.pop()? {
                        Value::I64(n) if n >= 0 => shape.push(n as usize),
                        _ => {
                            return Err(KengaError::new(
                                "tensor dims must be non-negative i64",
                                None,
                            ));
                        }
                    }
                }
                shape.reverse();
                let len: usize = if shape.is_empty() {
                    0
                } else {
                    shape.iter().product()
                };
                self.stack.push(Value::Tensor {
                    shape,
                    data: vec![0.0; len],
                });
            }
            Op::MakeList { n } => {
                let mut xs = Vec::with_capacity(n);
                for _ in 0..n {
                    xs.push(self.pop()?);
                }
                xs.reverse();
                self.stack.push(Value::List(xs));
            }
            Op::MakeRange => {
                let end = match self.pop()? {
                    Value::I64(n) => n,
                    _ => return Err(KengaError::new("range end must be i64", None)),
                };
                let start = match self.pop()? {
                    Value::I64(n) => n,
                    _ => return Err(KengaError::new("range start must be i64", None)),
                };
                self.stack.push(Value::Range { start, end });
            }
            Op::MakeStruct { name, fields } => {
                let mut map = HashMap::new();
                for f in fields.iter().rev() {
                    map.insert(f.clone(), self.pop()?);
                }
                self.stack.push(Value::Struct { name, fields: map });
            }
            Op::GetIndex => {
                let index = self.pop()?;
                let target = self.pop()?;
                let idx = match index {
                    Value::I64(i) => i,
                    _ => return Err(KengaError::new("index must be i64", None)),
                };
                match target {
                    Value::List(xs) => {
                        if idx < 0 || idx as usize >= xs.len() {
                            return Err(KengaError::new("index out of bounds", None));
                        }
                        self.stack.push(xs[idx as usize].clone());
                    }
                    Value::Range { start, end } => {
                        let len = end - start;
                        if idx < 0 || idx >= len {
                            return Err(KengaError::new("range index out of bounds", None));
                        }
                        self.stack.push(Value::I64(start + idx));
                    }
                    Value::Str(s) => {
                        let chs: Vec<char> = s.chars().collect();
                        if idx < 0 || idx as usize >= chs.len() {
                            return Err(KengaError::new("string index out of bounds", None));
                        }
                        self.stack
                            .push(Value::Str(chs[idx as usize].to_string()));
                    }
                    _ => {
                        return Err(KengaError::new(
                            "indexing only for list, range, str",
                            None,
                        ));
                    }
                }
            }
            Op::SetIndex => {
                let val = self.pop()?;
                let index = self.pop()?;
                let target = self.pop()?;
                let idx = match index {
                    Value::I64(i) => i,
                    _ => return Err(KengaError::new("index must be i64", None)),
                };
                match target {
                    Value::List(mut xs) => {
                        if idx < 0 || idx as usize >= xs.len() {
                            return Err(KengaError::new("index out of bounds", None));
                        }
                        xs[idx as usize] = val;
                        self.stack.push(Value::List(xs));
                    }
                    _ => return Err(KengaError::new("can only set index on list", None)),
                }
            }
            Op::GetField(field) => match self.pop()? {
                Value::Struct { fields, .. } => {
                    let v = fields.get(&field).cloned().ok_or_else(|| {
                        KengaError::new(format!("unknown field '{field}'"), None)
                    })?;
                    self.stack.push(v);
                }
                _ => return Err(KengaError::new("field access on non-struct", None)),
            },
            Op::SetField(field) => {
                let val = self.pop()?;
                match self.pop()? {
                    Value::Struct { name, mut fields } => {
                        fields.insert(field, val);
                        self.stack.push(Value::Struct { name, fields });
                    }
                    _ => return Err(KengaError::new("field set on non-struct", None)),
                }
            }
            Op::Len => {
                let v = self.pop()?;
                let n = match v {
                    Value::List(xs) => xs.len() as i64,
                    Value::Str(s) => s.chars().count() as i64,
                    Value::Range { start, end } => end - start,
                    Value::Tensor { data, .. } => data.len() as i64,
                    _ => return Err(KengaError::new("len() unsupported type", None)),
                };
                self.stack.push(Value::I64(n));
            }
            Op::Push => {
                let val = self.pop()?;
                match self.pop()? {
                    Value::List(mut xs) => {
                        xs.push(val);
                        self.stack.push(Value::List(xs));
                    }
                    _ => return Err(KengaError::new("push expects list", None)),
                }
            }
            Op::Assert => {
                let v = self.pop()?;
                if !v.as_bool() {
                    return Err(KengaError::new("assertion failed", None));
                }
            }
            Op::TypeOf => {
                let v = self.pop()?;
                self.stack.push(Value::Str(v.type_name().into()));
            }
            Op::SweepMemory => self.sweep(),
            Op::Listen => {
                let handler = match self.pop()? {
                    Value::Str(s) => s,
                    _ => return Err(KengaError::new("listen handler must be str", None)),
                };
                let event = match self.pop()? {
                    Value::Str(s) => s,
                    _ => return Err(KengaError::new("listen event must be str", None)),
                };
                self.listeners.insert(event, handler);
            }
            Op::Emit => {
                let value = self.pop()?;
                let event = match self.pop()? {
                    Value::Str(s) => s,
                    _ => return Err(KengaError::new("emit event must be str", None)),
                };
                self.events.push_back((event, value));
            }
            Op::Pump => {
                let max = match self.pop()? {
                    Value::I64(n) if n >= 0 => n as usize,
                    _ => return Err(KengaError::new("pump expects non-negative i64", None)),
                };
                let mut processed = 0i64;
                while processed < max as i64 {
                    let Some((event, value)) = self.events.pop_front() else {
                        break;
                    };
                    let Some(handler) = self.listeners.get(&event).cloned() else {
                        // no listener — drop event
                        continue;
                    };
                    let arity = self
                        .module
                        .functions
                        .get(&handler)
                        .map(|c| c.arity)
                        .unwrap_or(0);
                    let args = match arity {
                        0 => Vec::new(),
                        1 => vec![value],
                        n => {
                            return Err(KengaError::new(
                                format!("event handler '{handler}' must take 0 or 1 arg, got {n}"),
                                None,
                            ));
                        }
                    };
                    let _ = self.call_sync(&handler, args)?;
                    processed += 1;
                }
                self.stack.push(Value::I64(processed));
            }
            Op::Pending => {
                self.stack.push(Value::I64(self.events.len() as i64));
            }
            Op::SleepMs => {
                let ms = match self.pop()? {
                    Value::I64(n) if n >= 0 => n as u64,
                    _ => return Err(KengaError::new("sleep_ms expects non-negative i64", None)),
                };
                thread::sleep(Duration::from_millis(ms));
            }
            Op::BreakPlaceholder | Op::ContinuePlaceholder => {
                return Err(KengaError::new(
                    "internal: unpatched break/continue",
                    None,
                ));
            }
        }
        Ok(Step::Continue)
    }

    fn bin_num(&mut self, i: fn(i64, i64) -> i64, f: fn(f64, f64) -> f64) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        match (a, b) {
            (Value::I64(x), Value::I64(y)) => self.stack.push(Value::I64(i(x, y))),
            (Value::F64(x), Value::F64(y)) => self.stack.push(Value::F64(f(x, y))),
            (Value::I64(x), Value::F64(y)) => self.stack.push(Value::F64(f(x as f64, y))),
            (Value::F64(x), Value::I64(y)) => self.stack.push(Value::F64(f(x, y as f64))),
            (Value::Str(x), Value::Str(y)) => self.stack.push(Value::Str(format!("{x}{y}"))),
            (Value::Str(x), Value::I64(y)) => self.stack.push(Value::Str(format!("{x}{y}"))),
            (Value::List(mut xs), Value::List(ys)) => {
                xs.extend(ys);
                self.stack.push(Value::List(xs));
            }
            _ => return Err(KengaError::new("type error in arithmetic", None)),
        }
        Ok(())
    }

    fn bin_cmp(&mut self, i: fn(i64, i64) -> bool, f: fn(f64, f64) -> bool) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let r = match (a, b) {
            (Value::I64(x), Value::I64(y)) => i(x, y),
            (Value::F64(x), Value::F64(y)) => f(x, y),
            (Value::I64(x), Value::F64(y)) => f(x as f64, y),
            (Value::F64(x), Value::I64(y)) => f(x, y as f64),
            _ => return Err(KengaError::new("type error in comparison", None)),
        };
        self.stack.push(Value::Bool(r));
        Ok(())
    }
}

fn values_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::I64(x), Value::I64(y)) => x == y,
        (Value::F64(x), Value::F64(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Nil, Value::Nil) => true,
        (Value::List(x), Value::List(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| values_eq(a, b))
        }
        _ => false,
    }
}

pub fn interpret(module: Module) -> Result<Value> {
    Vm::new(module).run()
}
