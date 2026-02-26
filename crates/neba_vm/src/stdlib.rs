use std::collections::HashMap;
use crate::value::Value;

pub fn register_globals(globals: &mut HashMap<String, (Value, bool)>) {
    macro_rules! reg {
        ($name:expr, $fn:expr) => {
            globals.insert($name.to_string(), (Value::NativeFn($name.to_string(), $fn), false));
        };
    }
    reg!("print",   neba_print);
    reg!("println", neba_println);
    reg!("input",   neba_input);
    reg!("len",     neba_len);
    reg!("str",     neba_str);
    reg!("int",     neba_int);
    reg!("float",   neba_float);
    reg!("bool",    neba_bool);
    reg!("typeof",  neba_type);
    reg!("abs",     neba_abs);
    reg!("min",     neba_min);
    reg!("max",     neba_max);
    reg!("range",   neba_range);
    reg!("push",    neba_push);
    reg!("pop",     neba_pop);
    reg!("assert",  neba_assert);
}

fn neba_print(args: &[Value]) -> Result<Value, String> {
    print!("{}", args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" "));
    Ok(Value::None)
}
fn neba_println(args: &[Value]) -> Result<Value, String> {
    println!("{}", args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" "));
    Ok(Value::None)
}
fn neba_input(args: &[Value]) -> Result<Value, String> {
    use std::io::{self, Write};
    if let Some(p) = args.first() { print!("{}", p); io::stdout().flush().ok(); }
    let mut line = String::new();
    io::stdin().read_line(&mut line).map_err(|e| e.to_string())?;
    Ok(Value::str(line.trim_end_matches('\n')))
}
fn neba_len(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => Ok(Value::Int(a.borrow().len() as i64)),
        Some(Value::Str(s))   => Ok(Value::Int(s.chars().count() as i64)),
        Some(v) => Err(format!("len() not supported for {}", v.type_name())),
        None    => Err("len() requires 1 argument".into()),
    }
}
fn neba_str(args: &[Value]) -> Result<Value, String> {
    Ok(Value::str(args.first().map_or("None".to_string(), |v| v.to_string())))
}
fn neba_int(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Int(n))   => Ok(Value::Int(*n)),
        Some(Value::Float(f)) => Ok(Value::Int(*f as i64)),
        Some(Value::Bool(b))  => Ok(Value::Int(*b as i64)),
        Some(Value::Str(s))   => s.trim().parse::<i64>().map(Value::Int).map_err(|_| format!("cannot convert '{}' to Int", s)),
        Some(v) => Err(format!("cannot convert {} to Int", v.type_name())),
        None    => Err("int() requires 1 argument".into()),
    }
}
fn neba_float(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Float(f)) => Ok(Value::Float(*f)),
        Some(Value::Int(n))   => Ok(Value::Float(*n as f64)),
        Some(Value::Bool(b))  => Ok(Value::Float(*b as i64 as f64)),
        Some(Value::Str(s))   => s.trim().parse::<f64>().map(Value::Float).map_err(|_| format!("cannot convert '{}' to Float", s)),
        Some(v) => Err(format!("cannot convert {} to Float", v.type_name())),
        None    => Err("float() requires 1 argument".into()),
    }
}
fn neba_bool(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Bool(args.first().map_or(false, |v| v.is_truthy())))
}
fn neba_type(args: &[Value]) -> Result<Value, String> {
    Ok(Value::str(args.first().map_or("None", |v| v.type_name())))
}
fn neba_abs(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Int(n))   => Ok(Value::Int(n.abs())),
        Some(Value::Float(f)) => Ok(Value::Float(f.abs())),
        Some(v) => Err(format!("abs() not supported for {}", v.type_name())),
        None    => Err("abs() requires 1 argument".into()),
    }
}
fn neba_min(args: &[Value]) -> Result<Value, String> {
    let items: Vec<Value> = if args.len() == 1 {
        if let Some(Value::Array(a)) = args.first() {
            let b = a.borrow(); if b.is_empty() { return Err("min() of empty array".into()); }
            b.clone()
        } else { args.to_vec() }
    } else { args.to_vec() };
    items.into_iter().reduce(|a, b| if a <= b { a } else { b })
        .ok_or_else(|| "min() requires at least 1 argument".into())
}
fn neba_max(args: &[Value]) -> Result<Value, String> {
    let items: Vec<Value> = if args.len() == 1 {
        if let Some(Value::Array(a)) = args.first() {
            let b = a.borrow(); if b.is_empty() { return Err("max() of empty array".into()); }
            b.clone()
        } else { args.to_vec() }
    } else { args.to_vec() };
    items.into_iter().reduce(|a, b| if a >= b { a } else { b })
        .ok_or_else(|| "max() requires at least 1 argument".into())
}
fn neba_range(args: &[Value]) -> Result<Value, String> {
    let (start, end, step) = match args {
        [Value::Int(e)]                                => (0, *e, 1),
        [Value::Int(s), Value::Int(e)]                 => (*s, *e, 1),
        [Value::Int(s), Value::Int(e), Value::Int(st)] => (*s, *e, *st),
        _ => return Err("range() expects 1–3 Int arguments".into()),
    };
    if step == 0 { return Err("range() step cannot be zero".into()); }
    let mut v = Vec::new();
    let mut i = start;
    while (step > 0 && i < end) || (step < 0 && i > end) { v.push(Value::Int(i)); i += step; }
    Ok(Value::array(v))
}
fn neba_push(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(arr), val] => { arr.borrow_mut().push(val.clone()); Ok(Value::None) }
        _ => Err("push(array, value) requires Array and value".into()),
    }
}
fn neba_pop(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(arr)) => arr.borrow_mut().pop().ok_or_else(|| "pop() on empty array".into()),
        _ => Err("pop(array) requires an Array".into()),
    }
}
fn neba_assert(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(v) if v.is_truthy() => Ok(Value::None),
        Some(_) => {
            let msg = args.get(1).map_or("assertion failed".to_string(), |m| m.to_string());
            Err(msg)
        }
        None => Err("assert() requires 1 argument".into()),
    }
}
