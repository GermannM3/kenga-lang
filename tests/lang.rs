use std::path::PathBuf;

use kenga::compiler::compile;
use kenga::driver::{compile_file, parse_file};
use kenga::lexer::Lexer;
use kenga::parser::Parser;
use kenga::vm::interpret;

fn run(src: &str) -> kenga::bytecode::Value {
    let tokens = Lexer::new(src).tokenize().expect("lex");
    let program = Parser::new(tokens).parse().expect("parse");
    let module = compile(&program).expect("compile");
    interpret(module).expect("run")
}

#[test]
fn hello_math() {
    let v = run(
        r#"
        fn main() -> i64 {
            return 21 * 2;
        }
        "#,
    );
    assert!(matches!(v, kenga::bytecode::Value::I64(42)));
}

#[test]
fn lists_and_for() {
    let v = run(
        r#"
        fn main() -> i64 {
            let xs = [1, 2, 3];
            let s = 0;
            for x in xs {
                s = s + x;
            }
            assert(s == 6);
            return len(xs);
        }
        "#,
    );
    assert!(matches!(v, kenga::bytecode::Value::I64(3)));
}

#[test]
fn range_break() {
    let v = run(
        r#"
        fn main() -> i64 {
            let n = 0;
            for i in 0..10 {
                if i == 4 {
                    break;
                }
                n = n + 1;
            }
            return n;
        }
        "#,
    );
    assert!(matches!(v, kenga::bytecode::Value::I64(4)));
}

#[test]
fn event_loop_on_emit_pump() {
    let v = run(
        r#"
        on "tick"(n: i64) {
            if n < 2 {
                emit("tick", n + 1);
            }
        }
        fn main() -> i64 {
            emit("tick", 0);
            return pump(10);
        }
        "#,
    );
    // 0,1,2 processed = 3
    assert!(matches!(v, kenga::bytecode::Value::I64(3)));
}

#[test]
fn prophet_memory_consolidate() {
    let v = run(
        r#"
        fn main() -> i64 {
            let m = memory_config(10, 16, 8);
            assert(remember(m, [1, 2, 3], 5) == false);
            assert(remember(m, [9, 0, 9], 80) == true);
            assert(remember(m, [8, 1, 8], 70) == true);
            let n = consolidate(m);
            assert(n == 2);
            let st = mem_stats(m);
            assert(st[0] == 0);
            assert(st[1] == 2);
            assert(st[3] > 0);
            let hits = recall(m, [9, 0, 9], 1);
            assert(len(hits) == 1);
            return st[1];
        }
        "#,
    );
    assert!(matches!(v, kenga::bytecode::Value::I64(2)));
}

#[test]
fn prophet_learn_predict() {
    let v = run(
        r#"
        fn main() -> i64 {
            let m = memory();
            learn(m, [1, 0, 0], [0, 1, 0]);
            learn(m, [0, 1, 0], [0, 0, 1]);
            let p = predict(m, [1, 0, 0]);
            assert(typeof(p) == "list");
            let st = mem_stats(m);
            assert(st[3] >= 2);
            assert(st[5] >= 8);
            return st[3];
        }
        "#,
    );
    match v {
        kenga::bytecode::Value::I64(n) => assert!(n >= 2),
        _ => panic!("expected i64"),
    }
}

#[test]
fn prophet_unroll() {
    let v = run(
        r#"
        fn main() -> i64 {
            let m = memory();
            for i in 0..6 {
                let x = [i, 1];
                let y = [i + 1, 1];
                learn(m, x, y);
            }
            let traj = unroll(m, [0, 1], 4);
            assert(len(traj) == 4);
            return len(traj);
        }
        "#,
    );
    assert!(matches!(v, kenga::bytecode::Value::I64(4)));
}

#[test]
fn neuromodel_learns_dynamics() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let module = compile_file(&root.join("examples/neuromodel.kenga")).expect("compile");
    let v = interpret(module).expect("run neuromodel");
    match v {
        kenga::bytecode::Value::I64(hits) => assert!(hits >= 24, "hits={hits}"),
        other => panic!("expected hits i64, got {other:?}"),
    }
}

#[test]
fn selfhost_iffy() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let module = compile_file(&root.join("examples/selfhost/iffy.kenga")).expect("compile");
    let v = interpret(module).expect("run iffy");
    assert!(matches!(v, kenga::bytecode::Value::I64(0)));
}


#[test]
fn str_builtin() {
    let v = run(
        r#"
        fn main() -> i64 {
            assert(to_str(42) == "42");
            assert(to_str(7) + "!" == "7!");
            return 0;
        }
        "#,
    );
    assert!(matches!(v, kenga::bytecode::Value::I64(0)));
}


#[test]
fn save_load_mind_roundtrip() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("examples/persist_mind.kenga");
    let module = compile_file(&path).expect("compile");
    let v = interpret(module).expect("run persist");
    assert!(matches!(v, kenga::bytecode::Value::I64(0)));
    let mind_path = root.join("minds/agent.km");
    assert!(mind_path.exists(), "mind file should exist after persist");
}

#[test]
fn ord_builtin() {
    let v = run(
        r#"
        fn main() -> i64 {
            assert(ord("0") == 48);
            assert(ord("9") == 57);
            return ord("A");
        }
        "#,
    );
    assert!(matches!(v, kenga::bytecode::Value::I64(65)));
}


#[test]
fn import_stdlib_and_struct() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("examples/native_struct.kenga");
    let program = parse_file(&path).expect("parse import");
    assert!(program
        .items
        .iter()
        .any(|i| matches!(i, kenga::ast::Item::Function(f) if f.name == "sum")));
    let module = compile_file(&path).expect("compile");
    let v = interpret(module).expect("run");
    assert!(matches!(v, kenga::bytecode::Value::I64(0)));
}
