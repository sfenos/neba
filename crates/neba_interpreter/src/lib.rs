pub mod environment;
pub mod error;
pub mod interpreter;
pub mod stdlib;
pub mod value;

pub use environment::Env;
pub use error::{InterpResult, RuntimeError};
pub use interpreter::{ClassMeta, Interpreter};
pub use value::Value;

/// Convenience: parsa + interpreta in un colpo solo.
pub fn eval(source: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let (program, lex_errors, parse_errors) = neba_parser::parse(source);
    if let Some(e) = lex_errors.into_iter().next()   { return Err(Box::new(e)); }
    if let Some(e) = parse_errors.into_iter().next() { return Err(Box::new(e)); }
    let mut interp = Interpreter::new();
    interp.run(&program)?;
    Ok(Value::None)
}

// ── Test ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Value {
        let (program, lex_errors, parse_errors) = neba_parser::parse(src);
        assert!(lex_errors.is_empty(),   "Lex: {:?}", lex_errors);
        assert!(parse_errors.is_empty(), "Parse: {:?}", parse_errors);
        let mut interp = Interpreter::new();
        let mut last = Value::None;
        for stmt in &program.stmts {
            last = interp.exec_stmt(stmt)
                .unwrap_or_else(|e| panic!("RuntimeError: {}", e));
        }
        last
    }

    fn run_err(src: &str) -> RuntimeError {
        let (program, _, _) = neba_parser::parse(src);
        let mut interp = Interpreter::new();
        for stmt in &program.stmts {
            if let Err(e) = interp.exec_stmt(stmt) { return e; }
        }
        panic!("Expected RuntimeError but got none");
    }

    // ── Aritmetica ────────────────────────────────────────────────────────
    #[test] fn t_add()    { assert_eq!(run("1 + 2"),       Value::Int(3)); }
    #[test] fn t_sub()    { assert_eq!(run("10 - 3"),      Value::Int(7)); }
    #[test] fn t_mul()    { assert_eq!(run("4 * 5"),       Value::Int(20)); }
    #[test] fn t_div()    { assert_eq!(run("10 / 4"),      Value::Float(2.5)); }
    #[test] fn t_intdiv() { assert_eq!(run("10 // 3"),     Value::Int(3)); }
    #[test] fn t_mod()    { assert_eq!(run("10 % 3"),      Value::Int(1)); }
    #[test] fn t_pow()    { assert_eq!(run("2 ** 10"),     Value::Int(1024)); }
    #[test] fn t_mixed()  { assert_eq!(run("1 + 2.5"),     Value::Float(3.5)); }
    #[test] fn t_strcat() { assert_eq!(run(r#""ab" + "cd""#), Value::Str("abcd".into())); }
    #[test] fn t_repeat() { assert_eq!(run(r#""ha" * 3"#), Value::Str("hahaha".into())); }
    #[test] fn t_dbz()    { assert!(matches!(run_err("1 / 0"), RuntimeError::DivisionByZero)); }

    // ── Confronto ─────────────────────────────────────────────────────────
    #[test] fn t_eq()  { assert_eq!(run("1 == 1"),        Value::Bool(true)); }
    #[test] fn t_ne()  { assert_eq!(run("1 != 2"),        Value::Bool(true)); }
    #[test] fn t_lt()  { assert_eq!(run("1 < 2"),         Value::Bool(true)); }
    #[test] fn t_gt()  { assert_eq!(run("2 > 1"),         Value::Bool(true)); }
    #[test] fn t_and() { assert_eq!(run("true and false"), Value::Bool(false)); }
    #[test] fn t_or()  { assert_eq!(run("false or true"),  Value::Bool(true)); }
    #[test] fn t_not() { assert_eq!(run("not true"),       Value::Bool(false)); }

    // ── Variabili ─────────────────────────────────────────────────────────
    #[test] fn t_var()      { assert_eq!(run("var x = 1\nx = 2\nx"), Value::Int(2)); }
    #[test] fn t_let_immut() {
        let e = run_err("let x = 1\nx = 2");
        assert!(matches!(e, RuntimeError::AssignError { .. }));
    }
    #[test] fn t_compound() { assert_eq!(run("var x = 10\nx += 5\nx"), Value::Int(15)); }
    #[test] fn t_undef()    { assert!(matches!(run_err("foo"), RuntimeError::UndefinedVariable { .. })); }

    // ── Controllo di flusso ───────────────────────────────────────────────
    #[test] fn t_if_true()  { assert_eq!(run("if true\n    42\nelse\n    0\n"), Value::Int(42)); }
    #[test] fn t_if_false() { assert_eq!(run("if false\n    42\nelse\n    0\n"), Value::Int(0)); }
    #[test] fn t_elif() {
        assert_eq!(run("var x = 2\nif x == 1\n    10\nelif x == 2\n    20\nelse\n    30\n"), Value::Int(20));
    }
    #[test] fn t_while() {
        assert_eq!(run("var i = 0\nvar s = 0\nwhile i < 5\n    s += i\n    i += 1\ns"), Value::Int(10));
    }
    #[test] fn t_for_range() {
        assert_eq!(run("var s = 0\nfor i in 0..5\n    s += i\ns"), Value::Int(10));
    }
    #[test] fn t_for_incl() {
        assert_eq!(run("var s = 0\nfor i in 0..=5\n    s += i\ns"), Value::Int(15));
    }
    #[test] fn t_break() {
        assert_eq!(run("var i = 0\nwhile true\n    if i == 3\n        break\n    i += 1\ni"), Value::Int(3));
    }
    #[test] fn t_continue() {
        assert_eq!(run("var s = 0\nfor i in 0..6\n    if i % 2 == 0\n        continue\n    s += i\ns"), Value::Int(9));
    }

    // ── Funzioni ──────────────────────────────────────────────────────────
    #[test] fn t_fn_basic() {
        assert_eq!(run("fn add(a: Int, b: Int) -> Int\n    return a + b\nadd(3, 4)"), Value::Int(7));
    }
    #[test] fn t_fn_recursion() {
        let src = "fn fact(n: Int) -> Int\n    if n <= 1\n        return 1\n    return n * fact(n - 1)\nfact(5)";
        assert_eq!(run(src), Value::Int(120));
    }
    #[test] fn t_fn_default() {
        assert_eq!(run("fn greet(name: Str = \"world\")\n    return name\ngreet()"), Value::Str("world".into()));
    }
    #[test] fn t_fn_closure() {
        let src = "fn make_adder(n: Int)\n    fn add(x: Int)\n        return x + n\n    return add\nlet add5 = make_adder(5)\nadd5(3)";
        assert_eq!(run(src), Value::Int(8));
    }
    #[test] fn t_arity_err() {
        assert!(matches!(run_err("fn f(a: Int)\n    return a\nf()"), RuntimeError::ArityMismatch { .. }));
    }
    #[test] fn t_stackoverflow() {
        assert!(matches!(run_err("fn inf()\n    return inf()\ninf()"), RuntimeError::StackOverflow));
    }

    // ── Match ─────────────────────────────────────────────────────────────
    #[test] fn t_match_literal() {
        assert_eq!(run("match 2\n    case 1 => 10\n    case 2 => 20\n    case _ => 0\n"), Value::Int(20));
    }
    #[test] fn t_match_wildcard() {
        assert_eq!(run("match 99\n    case 1 => 1\n    case _ => 42\n"), Value::Int(42));
    }
    #[test] fn t_match_some() {
        assert_eq!(run("let x = Some(42)\nmatch x\n    case Some(v) => v\n    case None => 0\n"), Value::Int(42));
    }
    #[test] fn t_match_none() {
        assert_eq!(run("let x: Option[Int] = None\nmatch x\n    case Some(v) => v\n    case None => 0\n"), Value::Int(0));
    }
    #[test] fn t_match_range() {
        assert_eq!(run("match 75\n    case 0..=59 => 0\n    case 60..=100 => 1\n    case _ => 2\n"), Value::Int(1));
    }
    #[test] fn t_match_or() {
        assert_eq!(run("match 3\n    case 1 | 2 | 3 => 99\n    case _ => 0\n"), Value::Int(99));
    }
    #[test] fn t_match_ok() {
        assert_eq!(run("let r = Ok(7)\nmatch r\n    case Ok(v) => v\n    case Err(e) => 0\n"), Value::Int(7));
    }

    // ── Array ─────────────────────────────────────────────────────────────
    #[test] fn t_arr_index()  { assert_eq!(run("let a = [10,20,30]\na[1]"), Value::Int(20)); }
    #[test] fn t_arr_neg()    { assert_eq!(run("let a = [1,2,3]\na[-1]"), Value::Int(3)); }
    #[test] fn t_arr_set()    { assert_eq!(run("var a = [1,2,3]\na[0] = 99\na[0]"), Value::Int(99)); }
    #[test] fn t_arr_len()    { assert_eq!(run("let a = [1,2,3,4]\na.len"), Value::Int(4)); }
    #[test] fn t_arr_in()     { assert_eq!(run("2 in [1,2,3]"), Value::Bool(true)); }
    #[test] fn t_arr_notin()  { assert_eq!(run("5 not in [1,2,3]"), Value::Bool(true)); }
    #[test] fn t_push_pop()   { assert_eq!(run("var a = [1,2]\npush(a,3)\npop(a)"), Value::Int(3)); }

    // ── String ────────────────────────────────────────────────────────────
    #[test] fn t_str_in()    { assert_eq!(run(r#""ell" in "hello""#), Value::Bool(true)); }
    #[test] fn t_str_idx()   { assert_eq!(run(r#""hello"[1]"#), Value::Str("e".into())); }

    // ── f-string ──────────────────────────────────────────────────────────
    #[test] fn t_fstr_var() {
        assert_eq!(run("let name = \"Neba\"\nf\"Hello, {name}!\""), Value::Str("Hello, Neba!".into()));
    }
    #[test] fn t_fstr_expr() {
        assert_eq!(run("f\"{1 + 2}\""), Value::Str("3".into()));
    }

    // ── Classe ────────────────────────────────────────────────────────────
    #[test] fn t_class_new() {
        let src = "class Point\n    x: Int = 0\n    y: Int = 0\nlet p = Point()\np.x";
        assert_eq!(run(src), Value::Int(0));
    }
    #[test] fn t_class_set() {
        let src = "class Point\n    x: Int = 0\nvar p = Point()\np.x = 42\np.x";
        assert_eq!(run(src), Value::Int(42));
    }
    #[test] fn t_class_method() {
        let src = "class Counter\n    count: Int = 0\n    fn increment(self)\n        self.count += 1\nvar c = Counter()\nc.increment()\nc.count";
        assert_eq!(run(src), Value::Int(1));
    }
    #[test] fn t_class_method_return() {
        let src = "class Calc\n    val: Int = 0\n    fn double(self) -> Int\n        return self.val * 2\nvar c = Calc()\nc.val = 5\nc.double()";
        assert_eq!(run(src), Value::Int(10));
    }

    // ── Built-in ──────────────────────────────────────────────────────────
    #[test] fn t_len_arr()  { assert_eq!(run("len([1,2,3])"),       Value::Int(3)); }
    #[test] fn t_len_str()  { assert_eq!(run("len(\"hello\")"),     Value::Int(5)); }
    #[test] fn t_str_conv() { assert_eq!(run("str(42)"),            Value::Str("42".into())); }
    #[test] fn t_int_conv() { assert_eq!(run("int(\"42\")"),        Value::Int(42)); }
    #[test] fn t_flt_conv() { assert_eq!(run("float(3)"),           Value::Float(3.0)); }
    #[test] fn t_abs()      { assert_eq!(run("abs(-42)"),           Value::Int(42)); }
    #[test] fn t_min()      { assert_eq!(run("min(3,1,2)"),         Value::Int(1)); }
    #[test] fn t_max()      { assert_eq!(run("max(3,1,2)"),         Value::Int(3)); }
    #[test] fn t_range_fn() { assert!(matches!(run("range(5)"),     Value::Array(_))); }
    #[test] fn t_assert_ok() { assert_eq!(run("assert(true)"),      Value::None); }
    #[test] fn t_assert_fail() { assert!(matches!(run_err("assert(false)"), RuntimeError::Generic { .. })); }
    #[test] fn t_type_fn()  { assert_eq!(run("typeof(42)"),           Value::Str("Int".into())); }

    // ── Option / Result ────────────────────────────────────────────────────
    #[test] fn t_some_truthy() { assert!(run("Some(0)").is_truthy()); }
    #[test] fn t_none_falsy()  { assert!(!run("None").is_truthy()); }
    #[test] fn t_ok()          { assert_eq!(run("Ok(1)"),  Value::Ok(Box::new(Value::Int(1)))); }
    #[test] fn t_err()         { assert_eq!(run("Err(0)"), Value::Err(Box::new(Value::Int(0)))); }

    // ── Algoritmi completi ────────────────────────────────────────────────
    #[test] fn t_fibonacci() {
        let src = "fn fib(n: Int) -> Int\n    if n <= 1\n        return n\n    return fib(n-1) + fib(n-2)\nfib(10)";
        assert_eq!(run(src), Value::Int(55));
    }
    #[test] fn t_bubble_sort() {
        let src = r#"
var arr = [5, 3, 1, 4, 2]
var n = len(arr)
var i = 0
while i < n
    var j = 0
    while j < n - i - 1
        if arr[j] > arr[j+1]
            var tmp = arr[j]
            arr[j] = arr[j+1]
            arr[j+1] = tmp
        j += 1
    i += 1
arr[0]
"#;
        assert_eq!(run(src), Value::Int(1));
    }
}
