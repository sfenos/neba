pub mod check;
pub mod env;
pub mod error;
pub mod infer;
pub mod types;

pub use check::check_program;
pub use env::TypeEnv;
pub use error::{Severity, TypeError};
pub use types::Type;

/// Analizza il sorgente e restituisce la lista di diagnostici.
/// Non blocca a errori: restituisce sempre tutti i problemi trovati.
pub fn analyse(source: &str) -> Vec<TypeError> {
    let (program, lex_errors, parse_errors) = neba_parser::parse(source);

    let mut errors = Vec::new();

    // Converti errori di lex/parse in TypeError con span dummy
    let dummy_span = neba_lexer::Span::new(0, 0, 0, 0);
    for e in lex_errors {
        errors.push(TypeError::error(format!("lex error: {}", e), dummy_span.clone()));
    }
    for e in parse_errors {
        errors.push(TypeError::error(format!("parse error: {}", e), dummy_span.clone()));
    }

    let mut env = TypeEnv::new();
    check_program(&program, &mut env, &mut errors);
    errors
}

/// Analizza e restituisce solo gli errori (non i warning).
pub fn check(source: &str) -> Vec<TypeError> {
    analyse(source)
        .into_iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .collect()
}

// ── Test ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(src: &str) {
        let errs = check(src);
        assert!(errs.is_empty(), "expected no errors, got: {:?}", errs);
    }

    fn err(src: &str) {
        let errs = check(src);
        assert!(!errs.is_empty(), "expected errors, got none for: {}", src);
    }

    fn err_contains(src: &str, fragment: &str) {
        let errs = check(src);
        let found = errs.iter().any(|e| e.message.contains(fragment));
        assert!(found, "expected error containing '{}', got: {:?}", fragment, errs);
    }

    // ── Letterali ─────────────────────────────────────────────────────────
    #[test] fn t_int_lit()   { ok("let x = 42"); }
    #[test] fn t_float_lit() { ok("let x = 3.14"); }
    #[test] fn t_str_lit()   { ok("let x = \"hello\""); }
    #[test] fn t_bool_lit()  { ok("let x = true"); }
    #[test] fn t_none_lit()  { ok("let x = None"); }

    // ── Variabili ─────────────────────────────────────────────────────────
    #[test] fn t_let_infer()  { ok("let x = 1\nlet y = x"); }
    #[test] fn t_var_infer()  { ok("var x = 1\nx += 1"); }
    #[test] fn t_undef()      { err("let x = undefined_var"); }
    #[test] fn t_immut()      { err("let x = 1\nx = 2"); }
    #[test] fn t_mut_ok()     { ok("var x = 1\nx = 2"); }

    // ── Annotazioni di tipo ───────────────────────────────────────────────
    #[test] fn t_annot_ok()   { ok("let x: Int = 42"); }
    #[test] fn t_annot_err()  { err_contains("let x: Int = \"hello\"", "type mismatch"); }
    #[test] fn t_annot_float(){ ok("let x: Float = 1"); } // Int → Float ok
    #[test] fn t_annot_str()  { ok("let s: Str = \"world\""); }

    // ── Operatori ─────────────────────────────────────────────────────────
    #[test] fn t_add_int()   { ok("let x = 1 + 2"); }
    #[test] fn t_add_float() { ok("let x = 1.0 + 2.0"); }
    #[test] fn t_add_str()   { ok("let s = \"a\" + \"b\""); }
    #[test] fn t_add_mixed() { err_contains("let x = \"a\" + 1", "operator '+'"); }
    #[test] fn t_cmp()       { ok("let b = 1 < 2"); }
    #[test] fn t_neg()       { ok("let x = -5"); }
    #[test] fn t_not()       { ok("let b = not true"); }
    #[test] fn t_bitwise()   { ok("let x = 1 & 2"); }

    // ── Array ─────────────────────────────────────────────────────────────
    #[test] fn t_arr_ok()    { ok("let a = [1, 2, 3]"); }
    #[test] fn t_arr_empty() { ok("let a: Array[Int] = []"); }
    #[test] fn t_arr_idx()   { ok("let a = [1, 2]\nlet x = a[0]"); }
    #[test] fn t_arr_bad_idx(){ err_contains("let a = [1, 2]\nlet x = a[\"k\"]", "index must be Int"); }

    // ── Funzioni ──────────────────────────────────────────────────────────
    #[test] fn t_fn_ok() {
        ok("fn add(a: Int, b: Int) -> Int\n    return a + b\nadd(1, 2)");
    }
    #[test] fn t_fn_arity() {
        err_contains("fn f(a: Int) -> Int\n    return a\nf(1, 2)", "expects 1");
    }
    #[test] fn t_fn_return_ok() {
        ok("fn double(x: Int) -> Int\n    return x * 2");
    }
    #[test] fn t_fn_return_mismatch() {
        err_contains(
            "fn f() -> Int\n    return \"hello\"",
            "return type mismatch",
        );
    }
    #[test] fn t_fn_no_annot() {
        ok("fn f(x)\n    return x\nf(42)");
    }
    #[test] fn t_fn_recursive() {
        ok("fn fact(n: Int) -> Int\n    if n <= 1\n        return 1\n    return n * fact(n - 1)");
    }

    // ── Controllo di flusso ───────────────────────────────────────────────
    #[test] fn t_if_bool()   { ok("if true\n    let x = 1"); }
    #[test] fn t_if_int()    { err_contains("if 1\n    let x = 1", "type mismatch"); }
    #[test] fn t_while_bool(){ ok("var i = 0\nwhile i < 10\n    i += 1"); }
    #[test] fn t_for_range() { ok("var s = 0\nfor i in 0..5\n    s += i"); }
    #[test] fn t_for_arr()   { ok("let a = [1, 2, 3]\nfor x in a\n    let y = x"); }
    #[test] fn t_for_str()   { ok("for c in \"hello\"\n    let x = c"); }

    // ── Option / Result ───────────────────────────────────────────────────
    #[test] fn t_some()      { ok("let x = Some(42)"); }
    #[test] fn t_none()      { ok("let x = None"); }
    #[test] fn t_ok()        { ok("let x = Ok(1)"); }
    #[test] fn t_err_val()   { ok("let x = Err(\"fail\")"); }

    // ── Classi ────────────────────────────────────────────────────────────
    #[test] fn t_class_basic() {
        ok("class Point\n    x: Int\n    y: Int");
    }
    #[test] fn t_class_method() {
        ok("class Counter\n    n: Int\n    fn inc(self) -> Int\n        return self.n");
    }

    // ── Built-in ─────────────────────────────────────────────────────────
    #[test] fn t_builtin_len()   { ok("let x = len(\"hello\")"); }
    #[test] fn t_builtin_str()   { ok("let s = str(42)"); }
    #[test] fn t_builtin_range() { ok("let r = range(0, 10)"); }
    #[test] fn t_builtin_print() { ok("print(\"hello\")"); }

    // ── Forward reference ─────────────────────────────────────────────────
    #[test] fn t_forward_ref() {
        ok("let x = add(1, 2)\nfn add(a: Int, b: Int) -> Int\n    return a + b");
    }

    // ── F-string ─────────────────────────────────────────────────────────
    #[test] fn t_fstr() { ok("let name = \"world\"\nlet s = f\"hello {name}\""); }
}
