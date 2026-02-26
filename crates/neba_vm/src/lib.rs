pub mod chunk;
pub mod compiler;
pub mod error;
pub mod opcode;
pub mod stdlib;
pub mod value;
pub mod vm;

pub use compiler::Compiler;
pub use error::{VmError, VmResult};
pub use value::Value;
pub use vm::Vm;

/// Compila ed esegue sorgente Neba tramite la bytecode VM.
pub fn run(source: &str) -> VmResult<Value> {
    let (program, lex_errors, parse_errors) = neba_parser::parse(source);
    if let Some(e) = lex_errors.into_iter().next() {
        return Err(VmError::CompileError(e.to_string()));
    }
    if let Some(e) = parse_errors.into_iter().next() {
        return Err(VmError::CompileError(e.to_string()));
    }
    let chunk = Compiler::compile(&program)?;
    let mut vm = Vm::new();
    vm.run_chunk(chunk)
}

// ── Test ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn r(src: &str) -> Value {
        match run(src) {
            Ok(v)  => v,
            Err(e) => panic!("VmError: {}", e),
        }
    }
    fn r_err(src: &str) -> VmError {
        match run(src) {
            Err(e) => e,
            Ok(v)  => panic!("Expected error, got {:?}", v),
        }
    }

    // ── Aritmetica ────────────────────────────────────────────────────────
    #[test] fn t_int_add()  { assert_eq!(r("1 + 2"),    Value::Int(3)); }
    #[test] fn t_int_sub()  { assert_eq!(r("10 - 3"),   Value::Int(7)); }
    #[test] fn t_int_mul()  { assert_eq!(r("4 * 5"),    Value::Int(20)); }
    #[test] fn t_int_div()  { assert_eq!(r("10 / 4"),   Value::Float(2.5)); }
    #[test] fn t_intdiv()   { assert_eq!(r("10 // 3"),  Value::Int(3)); }
    #[test] fn t_mod()      { assert_eq!(r("10 % 3"),   Value::Int(1)); }
    #[test] fn t_pow()      { assert_eq!(r("2 ** 10"),  Value::Int(1024)); }
    #[test] fn t_neg()      { assert_eq!(r("-5"),        Value::Int(-5)); }
    #[test] fn t_mixed()    { assert_eq!(r("1 + 2.5"),  Value::Float(3.5)); }
    #[test] fn t_strcat()   { assert_eq!(r(r#""ab" + "cd""#), Value::str("abcd")); }
    #[test] fn t_repeat()   { assert_eq!(r(r#""ha" * 3"#),    Value::str("hahaha")); }
    #[test] fn t_dbz()      { assert!(matches!(r_err("1 / 0"), VmError::DivisionByZero)); }

    // ── Confronto / logica ────────────────────────────────────────────────
    #[test] fn t_eq()  { assert_eq!(r("1 == 1"),         Value::Bool(true)); }
    #[test] fn t_ne()  { assert_eq!(r("1 != 2"),         Value::Bool(true)); }
    #[test] fn t_lt()  { assert_eq!(r("1 < 2"),          Value::Bool(true)); }
    #[test] fn t_gt()  { assert_eq!(r("2 > 1"),          Value::Bool(true)); }
    #[test] fn t_and() { assert_eq!(r("true and false"),  Value::Bool(false)); }
    #[test] fn t_or()  { assert_eq!(r("false or true"),   Value::Bool(true)); }
    #[test] fn t_not() { assert_eq!(r("not true"),        Value::Bool(false)); }

    // ── Variabili ─────────────────────────────────────────────────────────
    #[test] fn t_let()   { assert_eq!(r("let x = 42\nx"),    Value::Int(42)); }
    #[test] fn t_var()   { assert_eq!(r("var x = 1\nx = 2\nx"), Value::Int(2)); }
    #[test] fn t_compound() { assert_eq!(r("var x = 10\nx += 5\nx"), Value::Int(15)); }
    #[test] fn t_undef() { assert!(matches!(r_err("foo"), VmError::UndefinedVariable(_))); }

    // ── Controllo di flusso ───────────────────────────────────────────────
    #[test] fn t_if_true()  { assert_eq!(r("var x = 0\nif true\n    x = 1\nx"), Value::Int(1)); }
    #[test] fn t_if_false() { assert_eq!(r("var x = 0\nif false\n    x = 1\nx"), Value::Int(0)); }
    #[test] fn t_while() {
        assert_eq!(r("var i = 0\nvar s = 0\nwhile i < 5\n    s += i\n    i += 1\ns"), Value::Int(10));
    }
    #[test] fn t_for_range() {
        assert_eq!(r("var s = 0\nfor i in 0..5\n    s += i\ns"), Value::Int(10));
    }
    #[test] fn t_for_incl() {
        assert_eq!(r("var s = 0\nfor i in 0..=5\n    s += i\ns"), Value::Int(15));
    }
    #[test] fn t_break() {
        assert_eq!(r("var i = 0\nwhile true\n    if i == 3\n        break\n    i += 1\ni"), Value::Int(3));
    }
    #[test] fn t_continue() {
        assert_eq!(r("var s = 0\nfor i in 0..6\n    if i % 2 == 0\n        continue\n    s += i\ns"), Value::Int(9));
    }

    // ── Funzioni ──────────────────────────────────────────────────────────
    #[test] fn t_fn_basic() {
        assert_eq!(r("fn add(a: Int, b: Int) -> Int\n    return a + b\nadd(3, 4)"), Value::Int(7));
    }
    #[test] fn t_fn_recursion() {
        let src = "fn fact(n: Int) -> Int\n    if n <= 1\n        return 1\n    return n * fact(n - 1)\nfact(5)";
        assert_eq!(r(src), Value::Int(120));
    }
    #[test] fn t_fn_default() {
        assert_eq!(r("fn greet(name: Str = \"world\")\n    return name\ngreet()"), Value::str("world"));
    }
    #[test] fn t_stackoverflow() {
        assert!(matches!(r_err("fn inf()\n    return inf()\ninf()"), VmError::StackOverflow));
    }

    // ── Array ─────────────────────────────────────────────────────────────
    #[test] fn t_arr_idx()  { assert_eq!(r("let a = [10,20,30]\na[1]"),        Value::Int(20)); }
    #[test] fn t_arr_neg()  { assert_eq!(r("let a = [1,2,3]\na[-1]"),          Value::Int(3)); }
    #[test] fn t_arr_set()  { assert_eq!(r("var a = [1,2,3]\na[0] = 99\na[0]"), Value::Int(99)); }
    #[test] fn t_arr_in()   { assert_eq!(r("2 in [1,2,3]"),                    Value::Bool(true)); }
    #[test] fn t_arr_notin(){ assert_eq!(r("5 not in [1,2,3]"),                Value::Bool(true)); }

    // ── Stringhe ──────────────────────────────────────────────────────────
    #[test] fn t_str_in()  { assert_eq!(r(r#""ell" in "hello""#), Value::Bool(true)); }
    #[test] fn t_str_idx() { assert_eq!(r(r#""hello"[1]"#),       Value::str("e")); }

    // ── F-string ──────────────────────────────────────────────────────────
    #[test] fn t_fstr_var() {
        assert_eq!(r("let name = \"Neba\"\nf\"Hello, {name}!\""), Value::str("Hello, Neba!"));
    }
    #[test] fn t_fstr_expr() {
        assert_eq!(r("f\"{1 + 2}\""), Value::str("3"));
    }

    // ── Option / Result ───────────────────────────────────────────────────
    #[test] fn t_some()  { assert_eq!(r("Some(42)"),  Value::Some_(Box::new(Value::Int(42)))); }
    #[test] fn t_ok()    { assert_eq!(r("Ok(1)"),     Value::Ok_(Box::new(Value::Int(1)))); }
    #[test] fn t_err()   { assert_eq!(r("Err(0)"),    Value::Err_(Box::new(Value::Int(0)))); }
    #[test] fn t_some_truthy() { assert!(r("Some(0)").is_truthy()); }
    #[test] fn t_none_falsy()  { assert!(!r("None").is_truthy()); }

    // ── Built-in ──────────────────────────────────────────────────────────
    #[test] fn t_len_arr()  { assert_eq!(r("len([1,2,3])"),    Value::Int(3)); }
    #[test] fn t_len_str()  { assert_eq!(r("len(\"hello\")"),  Value::Int(5)); }
    #[test] fn t_str_conv() { assert_eq!(r("str(42)"),         Value::str("42")); }
    #[test] fn t_int_conv() { assert_eq!(r("int(\"42\")"),     Value::Int(42)); }
    #[test] fn t_float_conv(){ assert_eq!(r("float(3)"),        Value::Float(3.0)); }
    #[test] fn t_abs()      { assert_eq!(r("abs(-42)"),         Value::Int(42)); }
    #[test] fn t_min()      { assert_eq!(r("min(3,1,2)"),       Value::Int(1)); }
    #[test] fn t_max()      { assert_eq!(r("max(3,1,2)"),       Value::Int(3)); }
    #[test] fn t_range_fn() { assert!(matches!(r("range(5)"),   Value::Array(_))); }
    #[test] fn t_assert_ok(){ assert_eq!(r("assert(true)"),     Value::None); }
    #[test] fn t_typeof()   { assert_eq!(r("typeof(42)"),       Value::str("Int")); }

    // ── Fibonacci ─────────────────────────────────────────────────────────
    #[test] fn t_fibonacci() {
        let src = "fn fib(n: Int) -> Int\n    if n <= 1\n        return n\n    return fib(n-1) + fib(n-2)\nfib(10)";
        assert_eq!(r(src), Value::Int(55));
    }
}

#[test]
fn debug_var_assign() {
    let src = "var x = 1\nx = 2\nx";
    let (program, _, _) = neba_parser::parse(src);
    println!("Stmts: {}", program.stmts.len());
    for (i, s) in program.stmts.iter().enumerate() {
        println!("  stmt[{}] = {:?}", i, std::mem::discriminant(&s.inner));
    }
    let chunk = crate::compiler::Compiler::compile(&program).unwrap();
    println!("{}", chunk.disassemble("<script>"));
}
