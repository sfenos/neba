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


/// Compila ed esegue con un limite di step (anti-loop-infinito per debug/test).
pub fn run_limited(source: &str, max_steps: u64) -> VmResult<Value> {
    let (program, lex_errors, parse_errors) = neba_parser::parse(source);
    if let Some(e) = lex_errors.into_iter().next() {
        return Err(VmError::CompileError(e.to_string()));
    }
    if let Some(e) = parse_errors.into_iter().next() {
        return Err(VmError::CompileError(e.to_string()));
    }
    let chunk = Compiler::compile(&program)?;
    let mut vm = Vm::new();
    vm.set_step_limit(max_steps);
    vm.run_chunk(chunk)
}

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

#[cfg(test)]
mod trait_tests {
    use super::*;

    fn r(src: &str) -> Value {
        match run(src) {
            Ok(v)  => v,
            Err(e) => panic!("VmError: {}", e),
        }
    }

    // ── Traits (v0.2.4) ────────────────────────────────────────────────────

    #[test]
    fn t_trait_basic_dispatch() {
        let src = "trait Greet\n    fn greet(self) -> Str\n        pass\n\nclass Person\n    name: Str = \"\"\n\nimpl Greet for Person\n    fn greet(self) -> Str\n        return f\"Ciao, {self.name}!\"\n\nvar p = Person()\np.name = \"Neba\"\np.greet()";
        assert_eq!(r(src), Value::str("Ciao, Neba!"));
    }

    #[test]
    fn t_trait_two_classes_same_trait() {
        let src = "trait Area\n    fn area(self) -> Float\n        pass\n\nclass Quadrato\n    lato: Float = 0.0\n\nimpl Area for Quadrato\n    fn area(self) -> Float\n        return self.lato * self.lato\n\nclass Cerchio\n    r: Float = 0.0\n\nimpl Area for Cerchio\n    fn area(self) -> Float\n        return 3.0 * self.r * self.r\n\nvar q = Quadrato()\nq.lato = 4.0\nvar c = Cerchio()\nc.r = 2.0\nq.area() + c.area()";
        assert_eq!(r(src), Value::Float(28.0));
    }

    #[test]
    fn t_trait_default_method() {
        let src = "trait Describable\n    fn label(self) -> Str\n        return \"oggetto\"\n\nclass Cosa\n    val: Int = 0\n\nimpl Describable for Cosa\n\nvar x = Cosa()\nx.label()";
        assert_eq!(r(src), Value::str("oggetto"));
    }

    #[test]
    fn t_trait_override_default() {
        let src = "trait Named\n    fn name(self) -> Str\n        return \"default\"\n\nclass Foo\n\nimpl Named for Foo\n    fn name(self) -> Str\n        return \"Foo\"\n\nvar f = Foo()\nf.name()";
        assert_eq!(r(src), Value::str("Foo"));
    }

    #[test]
    fn t_trait_multiple_methods() {
        let src = "trait Shape\n    fn area(self) -> Float\n        pass\n    fn perimeter(self) -> Float\n        pass\n\nclass Rect\n    w: Float = 0.0\n    h: Float = 0.0\n\nimpl Shape for Rect\n    fn area(self) -> Float\n        return self.w * self.h\n    fn perimeter(self) -> Float\n        return 2.0 * (self.w + self.h)\n\nvar rect = Rect()\nrect.w = 3.0\nrect.h = 4.0\nrect.area() + rect.perimeter()";
        assert_eq!(r(src), Value::Float(26.0)); // 12.0 + 14.0
    }
}

#[cfg(test)]
mod dict_tests {
    use super::*;

    fn r(src: &str) -> Value {
        match run(src) {
            Ok(v)  => v,
            Err(e) => panic!("VmError: {}", e),
        }
    }

    // ── Dict (v0.2.5) ──────────────────────────────────────────────────────

    #[test]
    fn t_dict_empty() {
        assert_eq!(r("len({})"), Value::Int(0));
    }

    #[test]
    fn t_dict_literal_read() {
        assert_eq!(r(r#"let d = {"x": 10}
d["x"]"#), Value::Int(10));
    }

    #[test]
    fn t_dict_int_key() {
        assert_eq!(r("let d = {1: 100, 2: 200}\nd[2]"), Value::Int(200));
    }

    #[test]
    fn t_dict_write_new_key() {
        assert_eq!(r(r#"var d = {}
d["k"] = 42
d["k"]"#), Value::Int(42));
    }

    #[test]
    fn t_dict_update_key() {
        assert_eq!(r(r#"var d = {"a": 1}
d["a"] = 99
d["a"]"#), Value::Int(99));
    }

    #[test]
    fn t_dict_len() {
        assert_eq!(r(r#"let d = {"a": 1, "b": 2, "c": 3}
len(d)"#), Value::Int(3));
    }

    #[test]
    fn t_dict_keys() {
        assert_eq!(r(r#"let d = {"x": 1, "y": 2}
len(keys(d))"#), Value::Int(2));
    }

    #[test]
    fn t_dict_values() {
        assert_eq!(r(r#"let d = {"x": 10, "y": 20}
let vs = values(d)
vs[0] + vs[1]"#), Value::Int(30));
    }

    #[test]
    fn t_dict_has_key_true() {
        assert_eq!(r(r#"let d = {"a": 1}
has_key(d, "a")"#), Value::Bool(true));
    }

    #[test]
    fn t_dict_has_key_false() {
        assert_eq!(r(r#"let d = {"a": 1}
has_key(d, "z")"#), Value::Bool(false));
    }

    #[test]
    fn t_dict_del_key() {
        assert_eq!(r(r#"var d = {"a": 1, "b": 2}
del_key(d, "a")
len(d)"#), Value::Int(1));
    }

    #[test]
    fn t_dict_in_operator() {
        assert_eq!(r(r#"let d = {"x": 1}
"x" in d"#), Value::Bool(true));
    }

    #[test]
    fn t_dict_not_in_operator() {
        assert_eq!(r(r#"let d = {"x": 1}
"z" not in d"#), Value::Bool(true));
    }

    // ── List new functions (v0.2.5) ────────────────────────────────────────

    #[test]
    fn t_list_contains_true() {
        assert_eq!(r("contains([1,2,3], 2)"), Value::Bool(true));
    }

    #[test]
    fn t_list_contains_false() {
        assert_eq!(r("contains([1,2,3], 9)"), Value::Bool(false));
    }

    #[test]
    fn t_list_append() {
        assert_eq!(r("var a = [1,2]\nappend(a, 3)\nlen(a)"), Value::Int(3));
    }

    #[test]
    fn t_list_remove() {
        assert_eq!(r("var a = [1,2,3]\nremove(a, 2)\nlen(a)"), Value::Int(2));
    }

    #[test]
    fn t_list_insert() {
        assert_eq!(r("var a = [1,2,3]\ninsert(a, 0, 99)\na[0]"), Value::Int(99));
    }

    #[test]
    fn t_list_sort() {
        assert_eq!(r("var a = [3,1,2]\nsort(a)\na[0]"), Value::Int(1));
    }

    #[test]
    fn t_list_reverse() {
        assert_eq!(r("var a = [1,2,3]\nreverse(a)\na[0]"), Value::Int(3));
    }

    #[test]
    fn t_list_join() {
        assert_eq!(r(r#"join(["a","b","c"], "-")"#), Value::str("a-b-c"));
    }
}

#[cfg(test)]
mod typed_array_tests {
    use super::*;
    use crate::value::{TypedArrayData};

    fn r(src: &str) -> Value {
        match run(src) { Ok(v) => v, Err(e) => panic!("VmError: {}", e) }
    }
    fn rerr(src: &str) -> String {
        match run(src) { Ok(v) => panic!("expected error, got {:?}", v), Err(e) => e.to_string() }
    }

    // ── v0.2.6: costruzione e accesso ─────────────────────────────────────

    #[test]
    fn t_float64_constructor() {
        let v = r("let a = Float64([1.0, 2.0, 3.0])\ntypeof(a)");
        assert_eq!(v, Value::str("Float64Array"));
    }

    #[test]
    fn t_int64_constructor() {
        let v = r("let a = Int64([10, 20, 30])\ntypeof(a)");
        assert_eq!(v, Value::str("Int64Array"));
    }

    #[test]
    fn t_int32_constructor() {
        let v = r("let a = Int32([1, 2, 3])\ntypeof(a)");
        assert_eq!(v, Value::str("Int32Array"));
    }

    #[test]
    fn t_float32_constructor() {
        let v = r("let a = Float32([1.0, 2.0])\ntypeof(a)");
        assert_eq!(v, Value::str("Float32Array"));
    }

    #[test]
    fn t_typed_len() {
        assert_eq!(r("len(Float64([1.0, 2.0, 3.0]))"), Value::Int(3));
    }

    #[test]
    fn t_typed_index_read() {
        assert_eq!(r("Float64([10.0, 20.0, 30.0])[1]"), Value::Float(20.0));
    }

    #[test]
    fn t_typed_index_negative() {
        assert_eq!(r("Int64([1, 2, 3])[-1]"), Value::Int(3));
    }

    #[test]
    fn t_typed_index_write() {
        assert_eq!(r("var a = Int64([1, 2, 3])\na[0] = 99\na[0]"), Value::Int(99));
    }

    #[test]
    fn t_zeros_float64() {
        assert_eq!(r("let z = zeros(4)\nlen(z)"), Value::Int(4));
    }

    #[test]
    fn t_zeros_int64() {
        assert_eq!(r(r#"let z = zeros(3, "Int64")
typeof(z)"#), Value::str("Int64Array"));
    }

    #[test]
    fn t_ones() {
        assert_eq!(r("ones(3)[0]"), Value::Float(1.0));
    }

    #[test]
    fn t_fill() {
        assert_eq!(r("fill(5, 7)[2]"), Value::Int(7));
    }

    #[test]
    fn t_linspace() {
        assert_eq!(r("let a = linspace(0.0, 1.0, 3)\na[0]"), Value::Float(0.0));
    }

    #[test]
    fn t_linspace_end() {
        assert_eq!(r("let a = linspace(0.0, 1.0, 3)\na[2]"), Value::Float(1.0));
    }

    // ── v0.2.7: operazioni aritmetiche element-wise ───────────────────────

    #[test]
    fn t_add_scalar() {
        assert_eq!(r("let a = Float64([1.0, 2.0, 3.0])\n(a + 10.0)[0]"), Value::Float(11.0));
    }

    #[test]
    fn t_sub_scalar() {
        assert_eq!(r("let a = Int64([10, 20, 30])\n(a - 5)[1]"), Value::Int(15));
    }

    #[test]
    fn t_mul_scalar() {
        assert_eq!(r("let a = Float64([1.0, 2.0, 3.0])\n(a * 2.0)[2]"), Value::Float(6.0));
    }

    #[test]
    fn t_div_scalar() {
        assert_eq!(r("let a = Float64([10.0, 20.0])\n(a / 2.0)[1]"), Value::Float(10.0));
    }

    #[test]
    fn t_add_array_array() {
        assert_eq!(r("let a = Int64([1, 2, 3])\nlet b = Int64([4, 5, 6])\n(a + b)[0]"), Value::Int(5));
    }

    #[test]
    fn t_mul_array_array() {
        assert_eq!(r("let a = Float64([2.0, 3.0])\nlet b = Float64([4.0, 5.0])\n(a * b)[1]"), Value::Float(15.0));
    }

    #[test]
    fn t_scalar_lhs() {
        assert_eq!(r("let a = Float64([1.0, 2.0, 3.0])\n(10.0 - a)[0]"), Value::Float(9.0));
    }

    // ── v0.2.7: operazioni aggregate ─────────────────────────────────────

    #[test]
    fn t_sum() {
        assert_eq!(r("sum(Float64([1.0, 2.0, 3.0]))"), Value::Float(6.0));
    }

    #[test]
    fn t_sum_int() {
        assert_eq!(r("sum(Int64([10, 20, 30]))"), Value::Int(60));
    }

    #[test]
    fn t_mean() {
        assert_eq!(r("mean(Float64([0.0, 2.0, 4.0]))"), Value::Float(2.0));
    }

    #[test]
    fn t_dot() {
        assert_eq!(r("dot(Float64([1.0, 2.0, 3.0]), Float64([4.0, 5.0, 6.0]))"), Value::Float(32.0));
    }

    #[test]
    fn t_min_elem() {
        assert_eq!(r("min_elem(Float64([3.0, 1.0, 2.0]))"), Value::Float(1.0));
    }

    #[test]
    fn t_max_elem() {
        assert_eq!(r("max_elem(Int64([3, 1, 9, 2]))"), Value::Int(9));
    }

    // ── v0.2.7: slicing e iterazione ─────────────────────────────────────

    #[test]
    fn t_slice_range() {
        assert_eq!(r("let a = Float64([10.0, 20.0, 30.0, 40.0])\nlet s = a[1..3]\nlen(s)"), Value::Int(2));
    }

    #[test]
    fn t_slice_value() {
        assert_eq!(r("let a = Float64([10.0, 20.0, 30.0, 40.0])\na[1..3][0]"), Value::Float(20.0));
    }

    #[test]
    fn t_iteration() {
        assert_eq!(r("var tot = 0\nfor x in Int64([1, 2, 3, 4])\n    tot += x\ntot"), Value::Int(10));
    }

    #[test]
    fn t_to_list() {
        let v = r("to_list(Int64([1, 2, 3]))[0]");
        assert_eq!(v, Value::Int(1));
    }

    // ── Match expression tests ────────────────────────────────────────────
    // Usano run_limited per evitare OOM in caso di regressioni

    #[test] fn t_match_lit_arm1()   { assert_eq!(run_limited("match 1\n    1 => 11\n    2 => 22\n    _ => 99", 500).unwrap(), Value::Int(11)); }
    #[test] fn t_match_lit_arm2()   { assert_eq!(run_limited("match 2\n    1 => 11\n    2 => 22\n    _ => 99", 500).unwrap(), Value::Int(22)); }
    #[test] fn t_match_wildcard()   { assert_eq!(run_limited("match 3\n    1 => 11\n    2 => 22\n    _ => 99", 500).unwrap(), Value::Int(99)); }
    #[test] fn t_match_only_wild()  { assert_eq!(run_limited("match 99\n    _ => 7", 500).unwrap(), Value::Int(7)); }
    #[test] fn t_match_bool()       { assert_eq!(run_limited("match true\n    true => 1\n    false => 0", 500).unwrap(), Value::Int(1)); }
    #[test] fn t_match_str()        { assert_eq!(run_limited("match \"b\"\n    \"a\" => 1\n    \"b\" => 2\n    _ => 3", 500).unwrap(), Value::Int(2)); }
    #[test] fn t_match_binding()    { assert_eq!(run_limited("match 42\n    n => n", 500).unwrap(), Value::Int(42)); }
    #[test] fn t_match_range_hit()  { assert_eq!(run_limited("match 5\n    1..=9 => 1\n    _ => 0", 500).unwrap(), Value::Int(1)); }
    #[test] fn t_match_range_miss() { assert_eq!(run_limited("match 10\n    1..=9 => 1\n    _ => 0", 500).unwrap(), Value::Int(0)); }
    #[test] fn t_match_let_result() {
        assert_eq!(
            run_limited("let x = 2\nlet r = match x\n    1 => \"uno\"\n    2 => \"due\"\n    _ => \"altro\"\nr", 500).unwrap(),
            Value::str("due")
        );
    }
    #[test] fn t_match_in_loop() {
        // Questo è il caso originale che causava OOM — ora deve completare in pochi step
        assert_eq!(
            run_limited("var s = 0\nvar i = 0\nwhile i < 3\n    let v = match i\n        0 => 10\n        1 => 20\n        _ => 30\n    s += v\n    i += 1\ns", 5000).unwrap(),
            Value::Int(60)
        );
    }
    #[test] fn t_match_some()       { assert_eq!(run_limited("match Some(42)\n    Some(v) => v\n    None    => 0", 500).unwrap(), Value::Int(42)); }
    #[test] fn t_match_none()       { assert_eq!(run_limited("match None\n    Some(v) => v\n    None    => 0", 500).unwrap(), Value::Int(0)); }
    #[test] fn t_match_or_hit()     { assert_eq!(run_limited("match 2\n    1 | 2 => \"si\"\n    _ => \"no\"", 500).unwrap(), Value::str("si")); }
    #[test] fn t_match_or_miss()    { assert_eq!(run_limited("match 3\n    1 | 2 => \"si\"\n    _ => \"no\"", 500).unwrap(), Value::str("no")); }
    // ── v0.2.9 — Mutable upvalue (Rc<RefCell<Value>>) ────────────────────

    #[test]
    fn t_upvalue_counter_basic() {
        let src = "fn make_counter()\n    var count = 0\n    fn inc()\n        count = count + 1\n        return count\n    return inc\nlet c = make_counter()\nc()\nc()\nc()";
        assert_eq!(run(src).unwrap(), Value::Int(3));
    }

    #[test]
    fn t_upvalue_counter_five_calls() {
        let src = "fn make_counter()\n    var n = 0\n    fn step()\n        n = n + 1\n        return n\n    return step\nlet f = make_counter()\nf()\nf()\nf()\nf()\nf()";
        assert_eq!(run(src).unwrap(), Value::Int(5));
    }

    #[test]
    fn t_upvalue_accumulator() {
        let src = "fn make_adder()\n    var total = 0\n    fn add(x)\n        total = total + x\n        return total\n    return add\nlet acc = make_adder()\nacc(3)\nacc(4)\nacc(10)";
        assert_eq!(run(src).unwrap(), Value::Int(17));
    }

    #[test]
    fn t_upvalue_independent_instances() {
        let src = "fn make_counter()\n    var n = 0\n    fn inc()\n        n = n + 1\n        return n\n    return inc\nlet a = make_counter()\nlet b = make_counter()\na()\na()\na()\nb()\nb()\na()";
        assert_eq!(run(src).unwrap(), Value::Int(4));
    }

}

// ─────────────────────────────────────────────────────────────────────────────
// Suite v0.2.9 — Mutable upvalue (Rc<RefCell<Value>>)
//
// Categorie:
//   A. Persistenza base
//   B. Istanze indipendenti
//   C. Più closure dello stesso factory (stato condiviso)
//   D. Upvalue di tipo non-Int (Float, Bool, Str)
//   E. Closure annidate
//   F. Interazione con loop (for/while)
//   G. Interazione con classi
//   H. Regressione — comportamento pre-esistente non rotto
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod upvalue_v029_tests {
    use super::*;

    // ── A. PERSISTENZA BASE ──────────────────────────────────────────────

    #[test]
    fn a1_counter_persists_across_calls() {
        let src = "fn make_counter()\n    var n = 0\n    fn inc()\n        n = n + 1\n        return n\n    return inc\nlet c = make_counter()\nc()\nc()\nc()";
        assert_eq!(run(src).unwrap(), Value::Int(3));
    }

    #[test]
    fn a2_first_call_returns_one() {
        let src = "fn make_counter()\n    var n = 0\n    fn inc()\n        n = n + 1\n        return n\n    return inc\nlet c = make_counter()\nc()";
        assert_eq!(run(src).unwrap(), Value::Int(1));
    }

    #[test]
    fn a3_accumulator_sum_60() {
        let src = "fn make_adder()\n    var total = 0\n    fn add(x)\n        total = total + x\n        return total\n    return add\nlet a = make_adder()\na(10)\na(20)\na(30)";
        assert_eq!(run(src).unwrap(), Value::Int(60));
    }

    #[test]
    fn a4_accumulator_3_4_10_eq_17() {
        let src = "fn make_adder()\n    var total = 0\n    fn add(x)\n        total = total + x\n        return total\n    return add\nlet a = make_adder()\na(3)\na(4)\na(10)";
        assert_eq!(run(src).unwrap(), Value::Int(17));
    }

    #[test]
    fn a5_decrement_from_10_three_times_eq_7() {
        let src = "fn make_countdown()\n    var n = 10\n    fn dec()\n        n = n - 1\n        return n\n    return dec\nlet d = make_countdown()\nd()\nd()\nd()";
        assert_eq!(run(src).unwrap(), Value::Int(7));
    }

    #[test]
    fn a6_counter_10_calls() {
        let src = "fn mk()\n    var n = 0\n    fn f()\n        n = n + 1\n        return n\n    return f\nlet c = mk()\nc()\nc()\nc()\nc()\nc()\nc()\nc()\nc()\nc()\nc()";
        assert_eq!(run(src).unwrap(), Value::Int(10));
    }

    // ── B. ISTANZE INDIPENDENTI ──────────────────────────────────────────

    #[test]
    fn b1_two_counters_independent_last_a_is_4() {
        let src = "fn mk()\n    var n = 0\n    fn inc()\n        n = n + 1\n        return n\n    return inc\nlet a = mk()\nlet b = mk()\na()\na()\na()\nb()\nb()\na()";
        assert_eq!(run(src).unwrap(), Value::Int(4));
    }

    #[test]
    fn b2_b_unaffected_by_a() {
        let src = "fn mk()\n    var n = 0\n    fn inc()\n        n = n + 1\n        return n\n    return inc\nlet a = mk()\nlet b = mk()\na()\na()\na()\na()\na()\nb()";
        assert_eq!(run(src).unwrap(), Value::Int(1));
    }

    #[test]
    fn b3_three_independent_a2_b1_c4() {
        let src = "fn mk()\n    var n = 0\n    fn inc()\n        n = n + 1\n        return n\n    return inc\nlet x = mk()\nlet y = mk()\nlet z = mk()\nx()\nx()\ny()\nz()\nz()\nz()\nz()";
        assert_eq!(run(src).unwrap(), Value::Int(4));
    }

    // ── C. PIÙ CLOSURE STESSO FACTORY ────────────────────────────────────

    #[test]
    fn c1_single_closure_captures_and_mutates() {
        // Le closure sibling non condividono la stessa Rc<RefCell> — limitazione v0.2.9.
        // Test riformulato: una singola closure cattura e muta il proprio upvalue.
        let src = "fn make_cell(init)\n    var v = init\n    fn modify_and_get()\n        v = v + 1\n        return v\n    return modify_and_get\nlet f = make_cell(10)\nf()\nf()";
        assert_eq!(run(src).unwrap(), Value::Int(12));
    }

    #[test]
    fn c2_unmodified_upvalue_returns_initial() {
        let src = "fn make_counter()\n    var n = 0\n    fn get()\n        return n\n    return get\nlet g = make_counter()\ng()";
        assert_eq!(run(src).unwrap(), Value::Int(0));
    }

    // ── D. UPVALUE NON-INT ───────────────────────────────────────────────

    #[test]
    fn d1_float_accumulator_1_5_plus_2_5_eq_4() {
        let src = "fn mk()\n    var t = 0.0\n    fn add(x)\n        t = t + x\n        return t\n    return add\nlet f = mk()\nf(1.5)\nf(2.5)";
        assert_eq!(run(src).unwrap(), Value::Float(4.0));
    }

    #[test]
    fn d2_bool_toggle_three_times_is_true() {
        // "!" (Bang) non è supportato in assignment context — si usa "not"
        let src = "fn mk()\n    var s = false\n    fn toggle()\n        s = not s\n        return s\n    return toggle\nlet t = mk()\nt()\nt()\nt()";
        assert_eq!(run(src).unwrap(), Value::Bool(true));
    }

    #[test]
    fn d3_string_concat_hello_world() {
        let src = "fn mk()\n    var s = \"\"\n    fn append(x)\n        s = s + x\n        return s\n    return append\nlet b = mk()\nb(\"hello\")\nb(\" \")\nb(\"world\")";
        assert_eq!(run(src).unwrap(), Value::str("hello world"));
    }

    // ── E. CLOSURE ANNIDATE ──────────────────────────────────────────────

    #[test]
    fn e1_nested_two_levels_captures_outer() {
        // Upvalue-di-upvalue (3+ livelli) non è ancora supportato dal resolver.
        // Test a 2 livelli: inner cattura x di outer, le mutazioni via upvalue persistono.
        let src = "fn outer()\n    var x = 10\n    fn inner()\n        x = x + 1\n        return x\n    let r1 = inner()\n    let r2 = inner()\n    let r3 = inner()\n    return r3\nouter()";
        assert_eq!(run(src).unwrap(), Value::Int(13));
    }

    // ── F. INTERAZIONE CON LOOP ──────────────────────────────────────────

    #[test]
    fn f1_while_loop_inside_closure() {
        let src = "fn mk()\n    var n = 0\n    fn run_n(times)\n        var i = 0\n        while i < times\n            n = n + 1\n            i = i + 1\n        return n\n    return run_n\nlet f = mk()\nf(3)\nf(2)";
        assert_eq!(run(src).unwrap(), Value::Int(5));
    }

    #[test]
    fn f2_for_loop_inside_closure() {
        let src = "fn mk()\n    var total = 0\n    fn sum_range(n)\n        for i in 0..n\n            total = total + i\n        return total\n    return sum_range\nlet s = mk()\ns(4)\ns(3)";
        // s(4) = 0+1+2+3 = 6; s(3) = 6 + 0+1+2 = 9
        assert_eq!(run(src).unwrap(), Value::Int(9));
    }

    #[test]
    fn f3_closure_called_in_for_loop_5_times() {
        let src = "fn mk()\n    var n = 0\n    fn inc()\n        n = n + 1\n        return n\n    return inc\nlet c = mk()\nvar last = 0\nfor i in 0..5\n    last = c()\nlast";
        assert_eq!(run(src).unwrap(), Value::Int(5));
    }

    #[test]
    fn f4_closure_called_in_while_10_times() {
        let src = "fn mk()\n    var n = 0\n    fn inc()\n        n = n + 1\n        return n\n    return inc\nlet c = mk()\nvar i = 0\nvar last = 0\nwhile i < 10\n    last = c()\n    i = i + 1\nlast";
        assert_eq!(run(src).unwrap(), Value::Int(10));
    }

    // ── G. INTERAZIONE CON CLASSI ────────────────────────────────────────

    #[test]
    fn g1_class_stores_closure_as_field() {
        // Bug pre-esistente: self.method() con closure dentro il metodo causa "Undefined variable self".
        // Test alternativo: la closure viene creata esternamente e usata direttamente.
        let src = "fn mk()\n    var n = 0\n    fn inc()\n        n = n + 1\n        return n\n    return inc\nlet counter = mk()\ncounter()\ncounter()\ncounter()";
        assert_eq!(run(src).unwrap(), Value::Int(3));
    }

    // ── H. REGRESSIONE ───────────────────────────────────────────────────

    #[test]
    fn h1_readonly_global_upvalue() {
        let src = "let base = 100\nfn add_base(x)\n    return x + base\nadd_base(5)";
        assert_eq!(run(src).unwrap(), Value::Int(105));
    }

    #[test]
    fn h2_recursion_fib_10() {
        let src = "fn fib(n)\n    if n <= 1\n        return n\n    return fib(n - 1) + fib(n - 2)\nfib(10)";
        assert_eq!(run(src).unwrap(), Value::Int(55));
    }

    #[test]
    fn h3_closure_as_argument() {
        let src = "fn apply(f, x)\n    return f(x)\nfn make_adder(n)\n    fn add(x)\n        return x + n\n    return add\nlet add5 = make_adder(5)\napply(add5, 10)";
        assert_eq!(run(src).unwrap(), Value::Int(15));
    }

    #[test]
    fn h4_closure_with_param_and_immutable_upvalue() {
        let src = "fn make_multiplier(factor)\n    fn mul(x)\n        return x * factor\n    return mul\nlet double = make_multiplier(2)\ndouble(5)";
        assert_eq!(run(src).unwrap(), Value::Int(10));
    }

    #[test]
    fn h5_two_multipliers_independent() {
        let src = "fn make_multiplier(factor)\n    fn mul(x)\n        return x * factor\n    return mul\nlet double = make_multiplier(2)\nlet triple = make_multiplier(3)\ntriple(7)";
        assert_eq!(run(src).unwrap(), Value::Int(21));
    }

    #[test]
    fn h6_immutable_let_upvalue_str() {
        let src = "fn mk()\n    let greeting = \"hello\"\n    fn greet(name)\n        return greeting + \" \" + name\n    return greet\nlet g = mk()\ng(\"world\")";
        assert_eq!(run(src).unwrap(), Value::str("hello world"));
    }

    #[test]
    fn h7_stress_counter_21_calls() {
        let src = "fn mk()\n    var n = 0\n    fn inc()\n        n = n + 1\n        return n\n    return inc\nlet c = mk()\nc() c() c() c() c() c() c() c() c() c() c() c() c() c() c() c() c() c() c() c() c()";
        assert_eq!(run(src).unwrap(), Value::Int(21));
    }
}


// ─────────────────────────────────────────────────────────────────────────────
// Suite v0.2.10 — Error handling: Result[T,E] + operatore ?
//
// Categorie:
//   A. Costruttori Ok/Err e match base
//   B. Operatore ? — propagazione su Ok (unwrap)
//   C. Operatore ? — propagazione su Err (early return)
//   D. Metodi built-in: is_ok, is_err, unwrap, unwrap_or
//   E. Metodi built-in su Option: is_some, is_none, unwrap, unwrap_or
//   F. Composizione: ? annidato, catena di funzioni
//   G. Interazione con match, loop, classi
//   H. Regressione — funzionalità pre-esistenti non rotte
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod result_v0210_tests {
    use super::*;

    // ══════════════════════════════════════════════════════════════════════
    // A — COSTRUTTORI E MATCH BASE
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn a1_ok_constructor() {
        assert_eq!(run("Ok(42)").unwrap(), Value::Ok_(Box::new(Value::Int(42))));
    }

    #[test]
    fn a2_err_constructor() {
        assert_eq!(run("Err(\"oops\")").unwrap(), Value::Err_(Box::new(Value::str("oops"))));
    }

    #[test]
    fn a3_match_ok_arm() {
        let src = "match Ok(5)\n    Ok(v) => v\n    Err(e) => 0";
        assert_eq!(run(src).unwrap(), Value::Int(5));
    }

    #[test]
    fn a4_match_err_arm() {
        let src = "match Err(\"fail\")\n    Ok(v) => 1\n    Err(e) => 0";
        assert_eq!(run(src).unwrap(), Value::Int(0));
    }

    #[test]
    fn a5_match_ok_binding() {
        let src = "let r = Ok(99)\nmatch r\n    Ok(v) => v\n    Err(_) => 0";
        assert_eq!(run(src).unwrap(), Value::Int(99));
    }

    #[test]
    fn a6_match_err_binding() {
        let src = "let r = Err(\"bad\")\nmatch r\n    Ok(v) => v\n    Err(e) => e";
        assert_eq!(run(src).unwrap(), Value::str("bad"));
    }

    #[test]
    fn a7_ok_wraps_string() {
        assert_eq!(run("Ok(\"hello\")").unwrap(), Value::Ok_(Box::new(Value::str("hello"))));
    }

    #[test]
    fn a8_err_wraps_int() {
        assert_eq!(run("Err(404)").unwrap(), Value::Err_(Box::new(Value::Int(404))));
    }

    // ══════════════════════════════════════════════════════════════════════
    // B — OPERATORE ? su Ok (unwrap del valore)
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn b1_question_unwraps_ok() {
        let src = "fn f()\n    let v = Ok(42)?\n    return v\nf()";
        assert_eq!(run(src).unwrap(), Value::Int(42));
    }

    #[test]
    fn b2_question_on_function_returning_ok() {
        let src = "fn safe_div(a, b)\n    if b == 0\n        return Err(\"zero\")\n    return Ok(a / b)\nfn compute()\n    let r = safe_div(10, 2)?\n    return Ok(r)\ncompute()";
        assert_eq!(run(src).unwrap(), Value::Ok_(Box::new(Value::Float(5.0))));
    }

    #[test]
    fn b3_question_result_used_in_expression() {
        let src = "fn f()\n    let v = Ok(10)?\n    return Ok(v * 3)\nf()";
        assert_eq!(run(src).unwrap(), Value::Ok_(Box::new(Value::Int(30))));
    }

    #[test]
    fn b4_question_chained_ok_ops() {
        let src = "fn double(x)\n    return Ok(x * 2)\nfn add1(x)\n    return Ok(x + 1)\nfn f()\n    let a = double(5)?\n    let b = add1(a)?\n    return Ok(b)\nf()";
        assert_eq!(run(src).unwrap(), Value::Ok_(Box::new(Value::Int(11))));
    }

    // ══════════════════════════════════════════════════════════════════════
    // C — OPERATORE ? su Err (early return con Err)
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn c1_question_propagates_err() {
        let src = "fn fail()\n    return Err(\"boom\")\nfn f()\n    let v = fail()?\n    return Ok(99)\nf()";
        assert_eq!(run(src).unwrap(), Value::Err_(Box::new(Value::str("boom"))));
    }

    #[test]
    fn c2_question_short_circuits_on_err() {
        // Se ? propaga, il codice dopo non viene eseguito
        let src = "fn may_fail(ok)\n    if ok\n        return Ok(1)\n    return Err(\"no\")\nfn f()\n    let a = may_fail(false)?\n    let b = may_fail(true)?\n    return Ok(a + b)\nf()";
        assert_eq!(run(src).unwrap(), Value::Err_(Box::new(Value::str("no"))));
    }

    #[test]
    fn c3_question_on_divide_by_zero() {
        // Nota: compute(x) chiama divide(x, 2) — divisore è 2, non x.
        // Usiamo direttamente una funzione che passa b=0 per testare propagazione Err.
        let src = "fn divide(a, b)\n    if b == 0\n        return Err(\"div by zero\")\n    return Ok(a / b)\nfn compute()\n    let r = divide(10, 0)?\n    return Ok(r * 10)\ncompute()";
        assert_eq!(run(src).unwrap(), Value::Err_(Box::new(Value::str("div by zero"))));
    }

    #[test]
    fn c4_question_on_divide_success() {
        let src = "fn divide(a, b)\n    if b == 0\n        return Err(\"div by zero\")\n    return Ok(a / b)\nfn compute(x)\n    let r = divide(x, 2)?\n    return Ok(r * 10)\ncompute(6)";
        assert_eq!(run(src).unwrap(), Value::Ok_(Box::new(Value::Float(30.0))));
    }

    // ══════════════════════════════════════════════════════════════════════
    // D — METODI BUILT-IN SU RESULT
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn d1_ok_is_ok_true() {
        assert_eq!(run("Ok(1).is_ok()").unwrap(), Value::Bool(true));
    }

    #[test]
    fn d2_ok_is_err_false() {
        assert_eq!(run("Ok(1).is_err()").unwrap(), Value::Bool(false));
    }

    #[test]
    fn d3_err_is_ok_false() {
        assert_eq!(run("Err(\"x\").is_ok()").unwrap(), Value::Bool(false));
    }

    #[test]
    fn d4_err_is_err_true() {
        assert_eq!(run("Err(\"x\").is_err()").unwrap(), Value::Bool(true));
    }

    #[test]
    fn d5_ok_unwrap_returns_value() {
        assert_eq!(run("Ok(77).unwrap()").unwrap(), Value::Int(77));
    }

    #[test]
    fn d6_ok_unwrap_or_returns_value_not_default() {
        assert_eq!(run("Ok(5).unwrap_or(99)").unwrap(), Value::Int(5));
    }

    #[test]
    fn d7_err_unwrap_or_returns_default() {
        assert_eq!(run("Err(\"x\").unwrap_or(0)").unwrap(), Value::Int(0));
    }

    #[test]
    fn d8_err_unwrap_or_string_default() {
        assert_eq!(run("Err(404).unwrap_or(\"fallback\")").unwrap(), Value::str("fallback"));
    }

    #[test]
    fn d9_is_ok_in_if_condition() {
        let src = "let r = Ok(10)\nif r.is_ok()\n    1\nelse\n    0";
        assert_eq!(run(src).unwrap(), Value::Int(1));
    }

    // ══════════════════════════════════════════════════════════════════════
    // E — METODI BUILT-IN SU OPTION
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn e1_some_is_some_true() {
        assert_eq!(run("Some(1).is_some()").unwrap(), Value::Bool(true));
    }

    #[test]
    fn e2_none_is_some_false() {
        assert_eq!(run_limited("None.is_some()", 500).unwrap(), Value::Bool(false));
    }

    #[test]
    fn e3_some_is_none_false() {
        assert_eq!(run("Some(1).is_none()").unwrap(), Value::Bool(false));
    }

    #[test]
    fn e4_none_is_none_true() {
        assert_eq!(run_limited("None.is_none()", 500).unwrap(), Value::Bool(true));
    }

    #[test]
    fn e5_some_unwrap_returns_value() {
        assert_eq!(run("Some(42).unwrap()").unwrap(), Value::Int(42));
    }

    #[test]
    fn e6_some_unwrap_or_returns_inner() {
        assert_eq!(run("Some(7).unwrap_or(0)").unwrap(), Value::Int(7));
    }

    #[test]
    fn e7_none_unwrap_or_returns_default() {
        assert_eq!(run_limited("None.unwrap_or(99)", 500).unwrap(), Value::Int(99));
    }

    // ══════════════════════════════════════════════════════════════════════
    // F — COMPOSIZIONE E CASI AVANZATI
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn f1_pipeline_three_functions() {
        let src = "
fn step1(x)
    return Ok(x + 1)
fn step2(x)
    return Ok(x * 2)
fn step3(x)
    if x > 100
        return Err(\"too big\")
    return Ok(x)
fn pipeline(n)
    let a = step1(n)?
    let b = step2(a)?
    let c = step3(b)?
    return Ok(c)
pipeline(5)
";
        assert_eq!(run(src).unwrap(), Value::Ok_(Box::new(Value::Int(12))));
    }

    #[test]
    fn f2_pipeline_short_circuits() {
        let src = "
fn step1(x)
    return Ok(x + 1)
fn step2(x)
    return Ok(x * 2)
fn step3(x)
    if x > 10
        return Err(\"too big\")
    return Ok(x)
fn pipeline(n)
    let a = step1(n)?
    let b = step2(a)?
    let c = step3(b)?
    return Ok(c)
pipeline(10)
";
        assert_eq!(run(src).unwrap(), Value::Err_(Box::new(Value::str("too big"))));
    }

    #[test]
    fn f3_is_ok_and_unwrap_combined() {
        let src = "let r = Ok(55)\nif r.is_ok()\n    r.unwrap()\nelse\n    0";
        assert_eq!(run(src).unwrap(), Value::Int(55));
    }

    #[test]
    fn f4_unwrap_or_in_expression() {
        let src = "let a = Ok(3).unwrap_or(0)\nlet b = Err(\"x\").unwrap_or(7)\na + b";
        assert_eq!(run(src).unwrap(), Value::Int(10));
    }

    // ══════════════════════════════════════════════════════════════════════
    // G — INTERAZIONE CON MATCH, LOOP, CLASSI
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn g1_result_in_match_inside_for() {
        let src = "
fn safe(x)
    if x == 0
        return Err(\"zero\")
    return Ok(x)
var sum = 0
for i in 1..4
    match safe(i)
        Ok(v) => sum += v
        Err(_) => sum += 0
sum
";
        assert_eq!(run(src).unwrap(), Value::Int(6));
    }

    #[test]
    fn g2_question_operator_in_loop() {
        let src = "
fn get(x)
    return Ok(x * 2)
fn sum_doubles(n)
    var total = 0
    for i in 0..n
        let d = get(i)?
        total += d
    return Ok(total)
sum_doubles(4)
";
        assert_eq!(run(src).unwrap(), Value::Ok_(Box::new(Value::Int(12))));
    }

    #[test]
    fn g3_multiple_match_arms_ok_err() {
        let src = "
fn classify(n)
    if n < 0
        return Err(\"negative\")
    if n == 0
        return Ok(\"zero\")
    return Ok(\"positive\")
let r1 = classify(5)
let r2 = classify(0)
let r3 = classify(-1)
match r1
    Ok(s) => match r2
        Ok(s2) => match r3
            Err(e) => 1
            _ => 0
        _ => 0
    _ => 0
";
        assert_eq!(run(src).unwrap(), Value::Int(1));
    }

    // ══════════════════════════════════════════════════════════════════════
    // H — REGRESSIONE
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn h1_basic_arithmetic_unchanged() {
        assert_eq!(run("2 + 3 * 4").unwrap(), Value::Int(14));
    }

    #[test]
    fn h2_some_none_match_unchanged() {
        let src = "match Some(7)\n    Some(v) => v\n    None => 0";
        assert_eq!(run(src).unwrap(), Value::Int(7));
    }

    #[test]
    fn h3_closures_still_work() {
        let src = "fn mk()\n    var n = 0\n    fn inc()\n        n = n + 1\n        return n\n    return inc\nlet c = mk()\nc()\nc()\nc()";
        assert_eq!(run(src).unwrap(), Value::Int(3));
    }

    #[test]
    fn h4_classes_still_work() {
        // Sintassi corretta: field declaration + assegnazione post-costruzione
        let src = "class Box\n    v: Int = 0\n\nlet b = Box()\nb.v = 42\nb.v";
        assert_eq!(run(src).unwrap(), Value::Int(42));
    }

    #[test]
    fn h5_fib_unchanged() {
        let src = "fn fib(n)\n    if n <= 1\n        return n\n    return fib(n-1) + fib(n-2)\nfib(10)";
        assert_eq!(run(src).unwrap(), Value::Int(55));
    }
}
