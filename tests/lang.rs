use kenga::compiler::compile;
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
fn struct_fields() {
    let v = run(
        r#"
        struct P { x: i64, y: i64 }
        fn main() -> i64 {
            let p = P { x: 2, y: 5 };
            return p.x + p.y;
        }
        "#,
    );
    assert!(matches!(v, kenga::bytecode::Value::I64(7)));
}
