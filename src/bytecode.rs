use std::collections::HashMap;
use std::fmt;

use crate::memory::MemoryHandle;

#[derive(Debug, Clone)]
pub enum Value {
    I64(i64),
    F64(f64),
    Bool(bool),
    Str(String),
    List(Vec<Value>),
    Struct {
        name: String,
        fields: HashMap<String, Value>,
    },
    Tensor {
        shape: Vec<usize>,
        data: Vec<f64>,
    },
    /// Inclusive-exclusive range iterator seed: start..end
    Range {
        start: i64,
        end: i64,
    },
    /// Prophet memory (episodic + core + locks)
    Memory(MemoryHandle),
    Nil,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::I64(_) => "i64",
            Value::F64(_) => "f64",
            Value::Bool(_) => "bool",
            Value::Str(_) => "str",
            Value::List(_) => "list",
            Value::Struct { .. } => "struct",
            Value::Tensor { .. } => "Tensor",
            Value::Range { .. } => "range",
            Value::Memory(_) => "Memory",
            Value::Nil => "nil",
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::I64(n) => *n != 0,
            Value::Nil => false,
            Value::Str(s) => !s.is_empty(),
            Value::List(xs) => !xs.is_empty(),
            _ => true,
        }
    }

    pub fn display(&self) -> String {
        match self {
            Value::I64(n) => n.to_string(),
            Value::F64(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => s.clone(),
            Value::List(xs) => {
                let inner = xs
                    .iter()
                    .map(Value::display)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{inner}]")
            }
            Value::Struct { name, fields } => {
                let mut keys: Vec<_> = fields.keys().collect();
                keys.sort();
                let inner = keys
                    .into_iter()
                    .map(|k| format!("{k}: {}", fields[k].display()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{name} {{ {inner} }}")
            }
            Value::Tensor { shape, data } => {
                format!("Tensor(shape={shape:?}, len={})", data.len())
            }
            Value::Range { start, end } => format!("{start}..{end}"),
            Value::Memory(h) => {
                let m = h.borrow();
                format!(
                    "Memory(ep={}, core={}, steps={}, dim={}x{}, thr={:.2})",
                    m.episodic.len(),
                    m.core.len(),
                    m.model.steps,
                    m.model.dim,
                    m.model.hidden,
                    m.threshold
                )
            }
            Value::Nil => "nil".into(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

#[derive(Debug, Clone)]
pub enum Op {
    Const(Value),
    Load(String),
    Store {
        name: String,
        ttl_ms: Option<u64>,
    },
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Neg,
    Not,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Jump(usize),
    JumpIfFalse(usize),
    Call {
        name: String,
        argc: usize,
    },
    Return,
    Print,
    Println,
    MakeTensor {
        dims: usize,
    },
    MakeList {
        n: usize,
    },
    MakeRange,
    MakeStruct {
        name: String,
        fields: Vec<String>,
    },
    GetIndex,
    SetIndex,
    GetField(String),
    SetField(String),
    Len,
    Push,
    Assert,
    TypeOf,
    SweepMemory,
    Listen,
    Emit,
    Pump,
    Pending,
    SleepMs,
    /// Prophet memory
    MakeMemory,
    MakeMemoryConfig,
    Remember,
    Surprise,
    Foresee,
    Predict,
    Learn,
    RememberNext,
    Unroll,
    ForeseeN,
    Consolidate,
    Recall,
    MemStats,
    /// break/continue placeholders patched to jumps
    BreakPlaceholder,
    ContinuePlaceholder,
}

#[derive(Debug, Clone)]
pub struct FunctionChunk {
    pub name: String,
    pub arity: usize,
    pub ops: Vec<Op>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub functions: HashMap<String, FunctionChunk>,
    pub structs: HashMap<String, Vec<String>>,
    pub intrinsics: HashMap<String, usize>,
    /// event name -> handler function name
    pub handlers: Vec<(String, String)>,
}

impl Module {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            structs: HashMap::new(),
            intrinsics: HashMap::new(),
            handlers: Vec::new(),
        }
    }
}

pub fn dump_ir(module: &Module) -> String {
    let mut out = String::new();
    out.push_str("; Kenga IR v0.1 — bootstrap bytecode\n");
    for (name, fields) in &module.structs {
        out.push_str(&format!("struct {name} {{ {} }}\n", fields.join(", ")));
    }
    for (name, arity) in &module.intrinsics {
        out.push_str(&format!("@intrinsic {name}/{arity}\n"));
    }
    out.push('\n');
    let mut names: Vec<_> = module.functions.keys().cloned().collect();
    names.sort();
    for name in names {
        let chunk = &module.functions[&name];
        out.push_str(&format!("fn {}/{}:\n", chunk.name, chunk.arity));
        for (i, op) in chunk.ops.iter().enumerate() {
            out.push_str(&format!("  {i:04}  {}\n", format_op(op)));
        }
        out.push('\n');
    }
    out
}

fn format_op(op: &Op) -> String {
    match op {
        Op::Const(v) => format!("CONST {v}"),
        Op::Load(n) => format!("LOAD {n}"),
        Op::Store { name, ttl_ms } => match ttl_ms {
            Some(ms) => format!("STORE {name} ttl={ms}ms"),
            None => format!("STORE {name}"),
        },
        Op::Pop => "POP".into(),
        Op::Add => "ADD".into(),
        Op::Sub => "SUB".into(),
        Op::Mul => "MUL".into(),
        Op::Div => "DIV".into(),
        Op::Rem => "REM".into(),
        Op::Neg => "NEG".into(),
        Op::Not => "NOT".into(),
        Op::Eq => "EQ".into(),
        Op::Ne => "NE".into(),
        Op::Lt => "LT".into(),
        Op::Le => "LE".into(),
        Op::Gt => "GT".into(),
        Op::Ge => "GE".into(),
        Op::And => "AND".into(),
        Op::Or => "OR".into(),
        Op::Jump(i) => format!("JUMP {i}"),
        Op::JumpIfFalse(i) => format!("JUMP_IF_FALSE {i}"),
        Op::Call { name, argc } => format!("CALL {name}/{argc}"),
        Op::Return => "RETURN".into(),
        Op::Print => "PRINT".into(),
        Op::Println => "PRINTLN".into(),
        Op::MakeTensor { dims } => format!("MAKE_TENSOR dims={dims}"),
        Op::MakeList { n } => format!("MAKE_LIST n={n}"),
        Op::MakeRange => "MAKE_RANGE".into(),
        Op::MakeStruct { name, fields } => {
            format!("MAKE_STRUCT {name} {{{}}}", fields.join(","))
        }
        Op::GetIndex => "GET_INDEX".into(),
        Op::SetIndex => "SET_INDEX".into(),
        Op::GetField(f) => format!("GET_FIELD {f}"),
        Op::SetField(f) => format!("SET_FIELD {f}"),
        Op::Len => "LEN".into(),
        Op::Push => "PUSH".into(),
        Op::Assert => "ASSERT".into(),
        Op::TypeOf => "TYPEOF".into(),
        Op::SweepMemory => "SWEEP_MEMORY".into(),
        Op::Listen => "LISTEN".into(),
        Op::Emit => "EMIT".into(),
        Op::Pump => "PUMP".into(),
        Op::Pending => "PENDING".into(),
        Op::SleepMs => "SLEEP_MS".into(),
        Op::MakeMemory => "MAKE_MEMORY".into(),
        Op::MakeMemoryConfig => "MAKE_MEMORY_CONFIG".into(),
        Op::Remember => "REMEMBER".into(),
        Op::Surprise => "SURPRISE".into(),
        Op::Foresee => "FORESEE".into(),
        Op::Predict => "PREDICT".into(),
        Op::Learn => "LEARN".into(),
        Op::RememberNext => "REMEMBER_NEXT".into(),
        Op::Unroll => "UNROLL".into(),
        Op::ForeseeN => "FORESEE_N".into(),
        Op::Consolidate => "CONSOLIDATE".into(),
        Op::Recall => "RECALL".into(),
        Op::MemStats => "MEM_STATS".into(),
        Op::BreakPlaceholder => "BREAK?".into(),
        Op::ContinuePlaceholder => "CONTINUE?".into(),
    }
}
