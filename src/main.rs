use std::env;
use std::path::PathBuf;
use std::process;

use kenga::bytecode::dump_ir;
use kenga::compiler::compile;
use kenga::driver::{compile_file, parse_file};
use kenga::error::{KengaError, Result};
use kenga::lexer::Lexer;
use kenga::parser::{dump_program, Parser};
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
            println!("kenga {} (bootstrap)", env!("CARGO_PKG_VERSION"));
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

fn print_usage() {
    eprintln!(
        "\x20Kenga — язык для живого ИИ

Установка и запуск:
  kenga run <file.kenga>
  kenga parse <file.kenga>
  kenga compile <file.kenga>
  kenga eval 'println(1+1);'
  kenga <file.kenga>

Rust-бинарник — временный bootstrap. Цель: self-host на самой Kenga.
Сайт установки: https://github.com/GermannM3/kenga-lang"
    );
}
