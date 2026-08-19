use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::process::Command;

use kenga::build::{build, BuildOptions};
use kenga::bytecode::dump_ir;
use kenga::codegen::emit_c;
use kenga::compiler::compile;
use kenga::demo::{run_about, run_demo, which_kenga};
use kenga::driver::{compile_file, parse_file};
use kenga::error::{KengaError, Result};
use kenga::lexer::Lexer;
use kenga::parser::{dump_program, Parser};
use kenga::talk::run_talk;
use kenga::vm::interpret;

fn main() {
    if let Err(e) = real_main() {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn real_main() -> Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        print_usage();
        process::exit(2);
    }

    let cmd = args.remove(0);
    match cmd.as_str() {
        "run" | "parse" | "compile" => {
            let use_lite = args.iter().any(|a| a == "--lite");
            let args: Vec<String> = args
                .into_iter()
                .filter(|a| a != "--lite")
                .collect();
            let path = args.first().ok_or_else(|| {
                KengaError::new(
                    format!("usage: kenga {cmd} [--lite] <file.kenga>"),
                    None,
                )
            })?;
            let path = PathBuf::from(path);
            match cmd.as_str() {
                "parse" => {
                    let program = parse_file(&path)?;
                    print!("{}", dump_program(&program));
                }
                "compile" => {
                    let module = compile_file(&path)?;
                    print!("{}", dump_ir(&module));
                }
                "run" => {
                    if use_lite || should_auto_lite(&path) {
                        if let Err(e) = run_lite(&[
                            "run".into(),
                            path.to_string_lossy().into_owned(),
                        ]) {
                            if use_lite {
                                return Err(e);
                            }
                            // auto-lite unavailable → fall back to Rust VM
                            let module = compile_file(&path)?;
                            let result = interpret(module)?;
                            if let kenga::bytecode::Value::I64(code) = result {
                                if code != 0 {
                                    process::exit(code as i32);
                                }
                            }
                        }
                    } else {
                        let module = compile_file(&path)?;
                        let result = interpret(module)?;
                        if let kenga::bytecode::Value::I64(code) = result {
                            if code != 0 {
                                process::exit(code as i32);
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        "emit-c" => {
            let use_freestanding = args.iter().any(|a| a == "--freestanding");
            let args: Vec<String> = args
                .into_iter()
                .filter(|a| a != "--freestanding")
                .collect();
            let path = args.first().ok_or_else(|| {
                KengaError::new(
                    "usage: kenga emit-c <file.kenga> [-o out.c] [--freestanding]",
                    None,
                )
            })?;
            let program = parse_file(&PathBuf::from(path))?;
            let c = if use_freestanding {
                kenga::codegen::emit_c_freestanding(&program)?
            } else {
                emit_c(&program)?
            };
            let out_path = flag_value(&args, "-o").unwrap_or_else(|| {
                PathBuf::from(path)
                    .with_extension("c")
                    .to_string_lossy()
                    .into_owned()
            });
            fs::write(&out_path, &c).map_err(|e| {
                KengaError::new(format!("cannot write {out_path}: {e}"), None)
            })?;
            println!("wrote {out_path}{}", if use_freestanding { " (freestanding)" } else { "" });
        }
        "build" => {
            let path = args.first().ok_or_else(|| {
                KengaError::new(
                    "usage: kenga build <file.kenga> [-o out] [--keep-c]",
                    None,
                )
            })?;
            let out = flag_value(&args, "-o").map(PathBuf::from);
            let keep_c = args.iter().any(|a| a == "--keep-c");
            let bin = build(BuildOptions {
                input: PathBuf::from(path),
                output: out,
                keep_c,
            })?;
            println!("built {}", bin.display());
        }
        "talk" | "chat" => {
            let use_lite = args.iter().any(|a| a == "--lite");
            let mind = args
                .iter()
                .find(|a| !a.starts_with('-') && a.ends_with(".km"))
                .map(PathBuf::from);
            let script_path = flag_value(&args, "--script");
            if use_lite {
                let mut lite_args = vec!["chat".to_string()];
                if let Some(p) = &mind {
                    lite_args.push(p.display().to_string());
                }
                if let Some(p) = &script_path {
                    lite_args.push("--script".into());
                    lite_args.push(p.clone());
                }
                run_lite(&lite_args)?;
            } else {
                let script = if let Some(p) = script_path {
                    Some(
                        fs::read_to_string(&p)
                            .map_err(|e| KengaError::new(format!("cannot read {p}: {e}"), None))?,
                    )
                } else {
                    None
                };
                run_talk(mind, script.as_deref())?;
            }
        }
        "demo" | "tour" => run_demo()?,
        "about" => run_about(),
        "which" => which_kenga()?,
        "lite" => run_lite(&args)?,
        "eval" => {
            let src = args.join(" ");
            if src.is_empty() {
                return Err(KengaError::new("usage: kenga eval <code>", None));
            }
            let wrapped = if src.contains("fn main") {
                src
            } else {
                format!("fn main() {{\n{src}\n}}\n")
            };
            let tokens = Lexer::new(&wrapped).tokenize()?;
            let program = Parser::new(tokens).parse()?;
            let module = compile(&program)?;
            interpret(module)?;
        }
        "version" | "--version" | "-V" => {
            println!("kenga {} (friends-ready)", env!("CARGO_PKG_VERSION"));
        }
        "help" | "--help" | "-h" => print_usage(),
        other => {
            if other.ends_with(".kenga") {
                let module = compile_file(&PathBuf::from(other))?;
                interpret(module)?;
            } else {
                eprintln!("unknown command '{other}'");
                print_usage();
                process::exit(2);
            }
        }
    }
    Ok(())
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
}

/// Run Rust-free C99 bootstrap (`bootstrap/bin/kenga-lite`).
fn run_lite(args: &[String]) -> Result<()> {
    let exe = find_lite_bin()?;
    let status = Command::new(&exe)
        .args(args)
        .status()
        .map_err(|e| KengaError::new(format!("failed to spawn {}: {e}", exe.display()), None))?;
    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

fn find_lite_bin() -> Result<PathBuf> {
    // Windows + Git Bash: accept both kenga-lite.exe and kenga-lite
    let names: &[&str] = if cfg!(windows) {
        &["kenga-lite.exe", "kenga-lite"]
    } else {
        &["kenga-lite"]
    };
    let mut candidates = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        for name in names {
            candidates.push(cwd.join("bootstrap/bin").join(name));
        }
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in names {
                candidates.push(dir.join(name));
                candidates.push(dir.join("../bootstrap/bin").join(name));
            }
        }
    }
    for c in &candidates {
        if c.is_file() {
            return Ok(c.clone());
        }
    }
    Err(KengaError::new(
        "kenga-lite not built. Run: bash bootstrap/build.sh   (Windows: bootstrap\\build.cmd)\nSee docs/UNIX.md / docs/SELFHOST.md",
        None,
    ))
}

/// Heuristic: `*_lite.kenga` or `examples/selfhost/*` → prefer C bootstrap.
fn should_auto_lite(path: &std::path::Path) -> bool {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if name.ends_with("_lite.kenga") {
        return true;
    }
    path.components().any(|c| c.as_os_str() == "selfhost")
}

fn print_usage() {
    eprintln!(
        "Kenga — язык для живого ИИ (friends-ready)

Быстрый тур для знакомых:
  git clone https://github.com/GermannM3/kenga-lang.git
  cd kenga-lang && cargo install --path . --force
  kenga demo

Команды:
  kenga demo|tour                  тур 5–6 минут
  kenga about                       что это / что нет
  kenga run [--lite] <file.kenga>
  kenga lite [run file|eval '..']   Rust-free lite (C99 bootstrap)
  kenga chat [mind.km]              диалог с world-model
  kenga chat --lite [mind.km]       то же на C99 (без Rust)
  kenga eval / compile / emit-c / build
  kenga which | version

Документация: docs/FOR_FRIENDS.md · docs/SELFHOST.md · docs/LEARN.md
https://github.com/GermannM3/kenga-lang"
    );
}
