use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use kenga::build::{build, BuildOptions};
use kenga::bytecode::dump_ir;
use kenga::codegen::emit_c;
use kenga::compiler::compile;
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
            let path = args.first().ok_or_else(|| {
                KengaError::new(format!("usage: kenga {cmd} <file.kenga>"), None)
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
                    let module = compile_file(&path)?;
                    let result = interpret(module)?;
                    if let kenga::bytecode::Value::I64(code) = result {
                        if code != 0 {
                            process::exit(code as i32);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        "emit-c" => {
            let path = args.first().ok_or_else(|| {
                KengaError::new("usage: kenga emit-c <file.kenga> [-o out.c]", None)
            })?;
            let program = parse_file(&PathBuf::from(path))?;
            let c = emit_c(&program)?;
            let out_path = flag_value(&args, "-o").unwrap_or_else(|| {
                PathBuf::from(path)
                    .with_extension("c")
                    .to_string_lossy()
                    .into_owned()
            });
            fs::write(&out_path, &c).map_err(|e| {
                KengaError::new(format!("cannot write {out_path}: {e}"), None)
            })?;
            println!("wrote {out_path}");
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
            let mind = args
                .iter()
                .find(|a| !a.starts_with('-') && a.ends_with(".km"))
                .map(PathBuf::from);
            let script_path = flag_value(&args, "--script");
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
            println!("kenga {}", env!("CARGO_PKG_VERSION"));
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

fn print_usage() {
    eprintln!(
        "Kenga — язык для живого ИИ

Команды:
  kenga run <file.kenga>              VM
  kenga talk|chat [mind.km] [--script f]  диалог с world-model
  kenga parse|compile <file.kenga>
  kenga emit-c / build
  kenga eval '<code>'
  kenga version

Важно: компилятор пока bootstrap на Rust. Программы .kenga и mind — уже твои.
Self-host seed: examples/selfhost/arith.kenga
Python не нужен.

https://github.com/GermannM3/kenga-lang"
    );
}
