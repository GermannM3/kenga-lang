//! Parser/lexer laxity regression tests.
//!
//! The self-hosted compiler kenga-lite (`bootstrap/kenga_lite.c`) silently
//! compiles programs with juxtaposed tokens, e.g. `return a b + 9;`. The
//! optional-semicolon change made statement boundaries ambiguous, so such
//! fragments may parse as two separate statements instead of an error.
//!
//! Every test below asserts that the program MUST be rejected (by lexing,
//! parsing or compilation). They are ignored until boundary checking lands,
//! because several of them are currently accepted by both kenga-lite and
//! the Rust parser (e.g. `return 5 5;` parses as `return 5;` + bare `5`).

use kenga::compiler::compile;
use kenga::lexer::Lexer;
use kenga::parser::Parser;

/// Rejected iff any stage fails: lex -> parse -> compile.
fn rejects(src: &str) -> bool {
    let tokens = match Lexer::new(src).tokenize() {
        Ok(t) => t,
        Err(_) => return true,
    };
    let program = match Parser::new(tokens).parse() {
        Ok(p) => p,
        Err(_) => return true,
    };
    compile(&program).is_err()
}

#[test]
#[ignore] // TODO(lexer): currently accepted by kenga-lite, must reject
fn reject_juxtaposed_idents_after_return() {
    // The originally observed bug: two identifiers with no operator.
    assert!(rejects(
        r#"
        fn main(a: i64, b: i64) -> i64 {
            return a b + 9;
        }
        "#,
    ));
}

#[test]
#[ignore] // TODO(lexer): currently accepted by kenga-lite, must reject
fn reject_juxtaposed_ints_after_return() {
    assert!(rejects(
        r#"
        fn main() -> i64 {
            return 5 5;
        }
        "#,
    ));
}

#[test]
#[ignore] // TODO(lexer): currently accepted by kenga-lite, must reject
fn reject_juxtaposed_idents_in_let() {
    assert!(rejects(
        r#"
        fn main(a: i64, b: i64) -> i64 {
            let x = a b;
            return x;
        }
        "#,
    ));
}

#[test]
#[ignore] // TODO(lexer): currently accepted by kenga-lite, must reject
fn reject_juxtaposed_ints_in_let() {
    assert!(rejects(
        r#"
        fn main() -> i64 {
            let x = 3 4;
            return x;
        }
        "#,
    ));
}

#[test]
#[ignore] // TODO(lexer): currently accepted by kenga-lite, must reject
fn reject_missing_operator_between_calls() {
    assert!(rejects(
        r#"
        fn f(x: i64) -> i64 {
            return x + 1;
        }
        fn g(x: i64) -> i64 {
            return x * 2;
        }
        fn main() -> i64 {
            let y = f(1) g(2);
            return y;
        }
        "#,
    ));
}

#[test]
#[ignore] // TODO(lexer): currently accepted by kenga-lite, must reject
fn reject_juxtaposed_idents_in_assign() {
    assert!(rejects(
        r#"
        fn main(s: i64, b: i64) -> i64 {
            s = s b;
            return s;
        }
        "#,
    ));
}

#[test]
#[ignore] // TODO(lexer): currently accepted by kenga-lite, must reject
fn reject_juxtaposed_idents_in_if_cond() {
    assert!(rejects(
        r#"
        fn main(a: i64, b: i64) -> i64 {
            if a b {
                return 1;
            }
            return 0;
        }
        "#,
    ));
}

#[test]
#[ignore] // TODO(lexer): currently accepted by kenga-lite, must reject
fn reject_trailing_operand_after_expr_stmt() {
    // Statement split by optional semicolons: `n 9` becomes two statements.
    assert!(rejects(
        r#"
        fn main() -> i64 {
            let n = 7;
            n 9;
            return n;
        }
        "#,
    ));
}
