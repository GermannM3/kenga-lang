use std::path::PathBuf;

use kenga::codegen::emit_c;
use kenga::driver::parse_file;
use kenga::lexer::Lexer;
use kenga::parser::Parser;

#[test]
fn emit_c_hello() {
    let src = r#"
        fn main() -> i64 {
            println("hi");
            return 42;
        }
    "#;
    let tokens = Lexer::new(src).tokenize().unwrap();
    let program = Parser::new(tokens).parse().unwrap();
    let c = emit_c(&program).unwrap();
    assert!(c.contains("kenga_println_str"));
    assert!(c.contains("return 42"));
}

#[test]
fn emit_c_lists_for() {
    let src = r#"
        fn main() -> i64 {
            let xs: list = [1, 2, 3];
            let s: i64 = 0;
            for v in xs {
                s = s + v;
            }
            for i in 0..3 {
                s = s + i;
            }
            println(s);
            return s;
        }
    "#;
    let tokens = Lexer::new(src).tokenize().unwrap();
    let program = Parser::new(tokens).parse().unwrap();
    let c = emit_c(&program).unwrap();
    assert!(c.contains("KList"));
    assert!(c.contains("klist_push"));
    assert!(c.contains("for ("));
}

#[test]
fn emit_c_struct() {
    let src = r#"
        struct Point { x: i64, y: i64 }
        fn main() -> i64 {
            let p: Point = Point { x: 3, y: 4 };
            return p.x * p.x + p.y * p.y;
        }
    "#;
    let tokens = Lexer::new(src).tokenize().unwrap();
    let program = Parser::new(tokens).parse().unwrap();
    let c = emit_c(&program).unwrap();
    assert!(c.contains("typedef struct"));
    assert!(c.contains("K_Point"));
    assert!(c.contains("k_x"));
}

#[test]
fn emit_c_from_imported_example() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let program = parse_file(&root.join("examples/native_struct.kenga")).unwrap();
    let c = emit_c(&program).unwrap();
    assert!(c.contains("K_Point"));
    assert!(c.contains("k_sum"));
    assert!(c.contains("k_dist2"));
}
