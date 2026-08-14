use kenga::codegen::emit_c;
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
