//! `kenga demo` — one-command tour for friends / ML engineers.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::driver::compile_file;
use crate::error::{KengaError, Result};
use crate::talk::run_talk;
use crate::vm::interpret;

fn find_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let candidates = [
        cwd.clone(),
        cwd.join(".."),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    ];
    for c in candidates {
        if c.join("examples/hello.kenga").is_file() {
            return Ok(c);
        }
    }
    Err(KengaError::new(
        "не вижу examples/ — склонируй репо и запускай из корня kenga-lang:\n  git clone https://github.com/GermannM3/kenga-lang.git\n  cd kenga-lang\n  cargo install --path . --force\n  kenga demo",
        None,
    ))
}

fn run_example(root: &Path, rel: &str) -> Result<()> {
    let path = root.join(rel);
    println!("── {rel}");
    let module = compile_file(&path)?;
    let v = interpret(module)?;
    if let crate::bytecode::Value::I64(code) = v {
        if code != 0 {
            return Err(KengaError::new(
                format!("{rel} вернул код {code}"),
                None,
            ));
        }
    }
    println!();
    Ok(())
}

pub fn run_demo() -> Result<()> {
    let root = find_root()?;
    println!("Kenga {} — demo tour\n", env!("CARGO_PKG_VERSION"));
    println!("корень: {}\n", root.display());

    println!("[1/6] hello");
    run_example(&root, "examples/hello.kenga")?;

    println!("[2/6] living memory + events");
    run_example(&root, "examples/showcase.kenga")?;

    println!("[3/6] Prophet world-model train");
    run_example(&root, "examples/train.kenga")?;

    println!("[4/6] neuromodel (pure Kenga)");
    run_example(&root, "examples/neuromodel.kenga")?;

    println!("[5/6] chat with mind");
    ensure_mind(&root)?;
    let mind = root.join("minds/agent.km");
    let script = root.join("examples/chat_fix.txt");
    let text = std::fs::read_to_string(&script).map_err(|e| {
        KengaError::new(format!("cannot read chat script: {e}"), None)
    })?;
    run_talk(Some(mind), Some(&text))?;
    println!();

    println!("[6/6] self-host Kenga-lite → bytecode");
    run_example(&root, "examples/selfhost/kenga_lite.kenga")?;

    println!("═══ demo ok ═══");
    println!("дальше:");
    println!("  kenga chat minds/agent.km");
    println!("  kenga run examples/ml/world_model.kenga");
    println!("  docs/FOR_FRIENDS.md");
    Ok(())
}

pub fn run_about() {
    println!(
        "kenga {} — язык для живого ИИ (friends-ready)

Что это:
  • свой язык: .kenga → bytecode VM (хост компилятора пока Rust)
  • Prophet: episodic memory + residual MLP world-model
  • агенты: on/emit/pump, ttl/sweep
  • self-host ladder: Kenga-lite компилируется в bytecode на чистой Kenga
  • Python не нужен

Что это НЕ:
  • не ChatGPT / не LLM из коробки
  • не полный self-host (Rust ещё хост)
  • не CUDA/PyTorch replacement

Для знакомых ML:
  git clone https://github.com/GermannM3/kenga-lang.git
  cd kenga-lang && cargo install --path . --force
  kenga demo

https://github.com/GermannM3/kenga-lang
",
        env!("CARGO_PKG_VERSION")
    );
}

/// Ensure deep-trained mind exists for chat demos.
pub fn ensure_mind(root: &Path) -> Result<()> {
    let mind = root.join("minds/agent.km");
    if mind.is_file() {
        return Ok(());
    }
    println!("нет minds/agent.km — учу…");
    run_example(root, "examples/deep_train.kenga")
}

pub fn which_kenga() -> Result<()> {
    let exe = std::env::current_exe().unwrap_or_default();
    println!("kenga {}", env!("CARGO_PKG_VERSION"));
    println!("binary: {}", exe.display());
    if let Ok(root) = find_root() {
        println!("repo:   {}", root.display());
    } else {
        println!("repo:   (examples не найдены — клонируй репо)");
    }
    // show if another kenga shadows
    if let Ok(out) = Command::new("where").arg("kenga").output() {
        let s = String::from_utf8_lossy(&out.stdout);
        if !s.trim().is_empty() {
            println!("PATH:\n{}", s.trim());
        }
    }
    Ok(())
}
