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
    assert!(matches!(v, kenga::bytecode::Value::I64(0)));
}

#[test]
fn ml_world_model() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let v = interpret(compile_file(&root.join("examples/ml/world_model.kenga")).unwrap()).unwrap();
    assert!(matches!(v, kenga::bytecode::Value::I64(0)));
}

#[test]
fn selfhost_arith_seed() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let module = compile_file(&root.join("examples/selfhost/arith.kenga")).expect("compile");
    let v = interpret(module).expect("run arith");
    assert!(matches!(v, kenga::bytecode::Value::I64(0)));
}

#[test]
fn selfhost_mini_vars() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let module = compile_file(&root.join("examples/selfhost/mini.kenga")).expect("compile");
    let v = interpret(module).expect("run mini");
    assert!(matches!(v, kenga::bytecode::Value::I64(0)));
}

#[test]
fn selfhost_loopfn() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let v = interpret(compile_file(&root.join("examples/selfhost/loopfn.kenga")).unwrap()).unwrap();
    assert!(matches!(v, kenga::bytecode::Value::I64(0)));
}

#[test]
fn selfhost_bytecode() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let v = interpret(compile_file(&root.join("examples/selfhost/bytecode.kenga")).unwrap()).unwrap();
    assert!(matches!(v, kenga::bytecode::Value::I64(0)));
}

#[test]
fn selfhost_bc_while() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let v = interpret(compile_file(&root.join("examples/selfhost/bc_while.kenga")).unwrap()).unwrap();
    assert!(matches!(v, kenga::bytecode::Value::I64(0)));
}

#[test]
fn selfhost_bc_fn() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let v = interpret(compile_file(&root.join("examples/selfhost/bc_fn.kenga")).unwrap()).unwrap();
    assert!(matches!(v, kenga::bytecode::Value::I64(0)));
}

#[test]
fn selfhost_kenga_lite() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let v = interpret(compile_file(&root.join("examples/selfhost/kenga_lite.kenga")).unwrap()).unwrap();
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

#[test]
fn bitwise_ops() {
    let v = run(
        r#"
        fn main() -> i64 {
            assert((5 & 3) == 1);
            assert((5 | 3) == 7);
            assert((5 ^ 3) == 6);
            assert((~0) == -1);
            assert((1 << 4) == 16);
            assert((16 >> 2) == 4);
            assert((1 + 2 << 3) == 24);
            assert((2 << 2 & 8 | 1) == 9);
            assert((0x20 & 0xF0) == 32);
            assert((0b1010 >> 1) == 5);
            return 0;
        }
        "#,
    );
    assert!(matches!(v, kenga::bytecode::Value::I64(0)));
}

#[test]
fn intrinsic_ffi_emit_c() {
    let tokens = Lexer::new(
        r#"
        @intrinsic fn kf_get_boot_info() -> i64;
        @intrinsic fn kf_str(addr: i64) -> str;

        fn main() -> i64 {
            let bi: i64 = kf_get_boot_info();
            let a: i64 = kf_str(bi) != "";
            return a;
        }
        "#,
    )
    .tokenize()
    .expect("lex");
    let program = Parser::new(tokens).parse().expect("parse");
    let c = kenga::codegen::emit_c_freestanding(&program).expect("emit-c");
    assert!(c.contains("int64_t k_kf_get_boot_info(void);"), "ffi prototype");
    assert!(c.contains("const char* k_kf_str(int64_t k_addr);"), "ffi prototype 2");
    // builtin len/ord must not get an additional extern prototype (only the
    // runtime's own definitions may reference them)
    assert_eq!(c.matches("k_len(").count(), 0, "builtin len must not be referenced");
    assert_eq!(c.matches("k_ord(").count(), 1, "only the static runtime k_ord def");
}

// ==== docs/KENGA_RESOURCE_SPEC.md · Фаза A: parse-only скелеты ====
// TODO(parser): снять #[ignore], когда lexer/parser получат токены
// pool/place/budget/adapt/checkpoint/grow/pin/evict/rest/every/by + Size-литералы.
// До тех пор эти программы обязаны падать на parse — поэтому ignore, а не assert.

#[allow(dead_code)]
fn parses(src: &str) {
    let tokens = Lexer::new(src).tokenize().expect("lex");
    Parser::new(tokens).parse().expect("parse");
}

#[test]
#[ignore = "TODO(parser): pool"] // docs/KENGA_RESOURCE_SPEC.md §2.1
fn resource_pool_decl() {
    parses(
        r#"
        pool desk {
            cpu: auto,
            ram: 24GiB,
            gpu: none
        }
        pool peer {
            cpu: 8,
            ram: 16GiB,
            gpu: auto
        }
        fn main() -> i64 {
            return 0;
        }
        "#,
    );
}

#[test]
#[ignore = "TODO(parser): place/pin/evict"] // docs/KENGA_RESOURCE_SPEC.md §2.2
fn resource_place_pin_evict() {
    parses(
        r#"
        pool home { cpu: auto, ram: 24GiB, gpu: none }
        pool lab { cpu: 16, ram: 64GiB, gpu: auto }
        let w = t_from([8, 4096, 4096], weights);
        place w on home(4GiB) evict;
        fn migrate_to_lab() -> i64 {
            place w on lab(12GiB) pin;
            return 0;
        }
        fn main() -> i64 {
            return migrate_to_lab();
        }
        "#,
    );
}

#[test]
#[ignore = "TODO(parser): budget/rest"] // docs/KENGA_RESOURCE_SPEC.md §2.3
fn resource_budget_slots() {
    parses(
        r#"
        budget moe {
            hot_experts: 10GiB,
            kv: 6GiB,
            other: rest
        }
        fn main() -> i64 {
            return 0;
        }
        "#,
    );
}

#[test]
#[ignore = "TODO(parser): budget slot assign"] // docs/KENGA_RESOURCE_SPEC.md §2.3
fn resource_budget_rebalance() {
    parses(
        r#"
        budget moe {
            hot_experts: 10GiB,
            kv: 6GiB,
            other: rest
        }
        pool desk { cpu: auto, ram: 24GiB, gpu: none }
        place experts on desk(moe.hot_experts) evict;
        place kv on desk(moe.kv) pin;
        fn main() -> i64 {
            moe.kv = 7GiB;
            return 0;
        }
        "#,
    );
}

#[test]
#[ignore = "TODO(parser): adapt"] // docs/KENGA_RESOURCE_SPEC.md §2.4
fn resource_adapt_link_policy() {
    parses(
        r#"
        adapt laptop_to_rack {
            min_bw: 80MiBps,
            prefer: remote
        }
        fn main() -> i64 {
            return 0;
        }
        "#,
    );
}

#[test]
#[ignore = "TODO(parser): checkpoint/load checkpoint"] // docs/KENGA_RESOURCE_SPEC.md §2.5
fn resource_checkpoint_roundtrip() {
    parses(
        r#"
        checkpoint semantic "sessions/chat_a" every 200;
        fn main() -> i64 {
            load checkpoint "sessions/chat_a";
            return 0;
        }
        "#,
    );
}

#[test]
#[ignore = "TODO(parser): grow ... by"] // docs/KENGA_RESOURCE_SPEC.md §2.6, reserved
fn resource_grow_zspace_reserved() {
    parses(
        r#"
        fn main() -> i64 {
            grow zspace by 2;
            return 0;
        }
        "#,
    );
}
