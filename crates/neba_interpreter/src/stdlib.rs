use crate::environment::Env;
use crate::value::Value;

pub fn register(env: &mut Env) {
    env.define("print",   Value::NativeFunction("print".into(), std::rc::Rc::new(neba_print)),   false);
    env.define("println", Value::NativeFunction("println".into(), std::rc::Rc::new(neba_println)), false);
    env.define("input",   Value::NativeFunction("input".into(), std::rc::Rc::new(neba_input)),   false);
    env.define("len",     Value::NativeFunction("len".into(), std::rc::Rc::new(neba_len)),     false);
    env.define("str",     Value::NativeFunction("str".into(), std::rc::Rc::new(neba_str)),     false);
    env.define("int",     Value::NativeFunction("int".into(), std::rc::Rc::new(neba_int)),     false);
    env.define("float",   Value::NativeFunction("float".into(), std::rc::Rc::new(neba_float)),   false);
    env.define("bool",    Value::NativeFunction("bool".into(), std::rc::Rc::new(neba_bool)),    false);
    env.define("typeof",    Value::NativeFunction("typeof".into(), std::rc::Rc::new(neba_type)),    false);
    env.define("abs",     Value::NativeFunction("abs".into(), std::rc::Rc::new(neba_abs)),     false);
    env.define("min",     Value::NativeFunction("min".into(), std::rc::Rc::new(neba_min)),     false);
    env.define("max",     Value::NativeFunction("max".into(), std::rc::Rc::new(neba_max)),     false);
    env.define("range",   Value::NativeFunction("range".into(), std::rc::Rc::new(neba_range)),   false);
    env.define("push",    Value::NativeFunction("push".into(), std::rc::Rc::new(neba_push)),    false);
    env.define("pop",     Value::NativeFunction("pop".into(), std::rc::Rc::new(neba_pop)),     false);
    env.define("assert",  Value::NativeFunction("assert".into(), std::rc::Rc::new(neba_assert)),  false);
}

fn neba_print(args: Vec<Value>) -> Result<Value, String> {
    print!("{}", args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" "));
    Ok(Value::None)
}
fn neba_println(args: Vec<Value>) -> Result<Value, String> {
    println!("{}", args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" "));
    Ok(Value::None)
}
fn neba_input(args: Vec<Value>) -> Result<Value, String> {
    use std::io::{self, Write};
    if let Some(p) = args.first() { print!("{}", p); io::stdout().flush().ok(); }
    let mut line = String::new();
    io::stdin().read_line(&mut line).map_err(|e| e.to_string())?;
    Ok(Value::Str(line.trim_end_matches('\n').to_string()))
}
fn neba_len(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => Ok(Value::Int(a.borrow().len() as i64)),
        Some(Value::Str(s))   => Ok(Value::Int(s.chars().count() as i64)),
        Some(v) => Err(format!("len() not supported for {}", v.type_name())),
        None    => Err("len() requires 1 argument".into()),
    }
}
fn neba_str(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Str(args.first().map_or("None".to_string(), |v| v.to_string())))
}
fn neba_int(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Int(n))   => Ok(Value::Int(*n)),
        Some(Value::Float(f)) => Ok(Value::Int(*f as i64)),
        Some(Value::Bool(b))  => Ok(Value::Int(*b as i64)),
        Some(Value::Str(s))   => s.trim().parse::<i64>().map(Value::Int).map_err(|_| format!("cannot convert '{}' to Int", s)),
        Some(v) => Err(format!("cannot convert {} to Int", v.type_name())),
        None    => Err("int() requires 1 argument".into()),
    }
}
fn neba_float(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Float(f)) => Ok(Value::Float(*f)),
        Some(Value::Int(n))   => Ok(Value::Float(*n as f64)),
        Some(Value::Bool(b))  => Ok(Value::Float(*b as i64 as f64)),
        Some(Value::Str(s))   => s.trim().parse::<f64>().map(Value::Float).map_err(|_| format!("cannot convert '{}' to Float", s)),
        Some(v) => Err(format!("cannot convert {} to Float", v.type_name())),
        None    => Err("float() requires 1 argument".into()),
    }
}
fn neba_bool(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Bool(args.first().map_or(false, |v| v.is_truthy())))
}
fn neba_type(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Str(args.first().map_or("None", |v| v.type_name()).to_string()))
}
fn neba_abs(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Int(n))   => Ok(Value::Int(n.abs())),
        Some(Value::Float(f)) => Ok(Value::Float(f.abs())),
        Some(v) => Err(format!("abs() not supported for {}", v.type_name())),
        None    => Err("abs() requires 1 argument".into()),
    }
}
fn neba_min(args: Vec<Value>) -> Result<Value, String> {
    let items: Vec<Value> = if args.len() == 1 {
        if let Some(Value::Array(a)) = args.first() {
            let b = a.borrow();
            if b.is_empty() { return Err("min() of empty array".into()); }
            b.clone()
        } else { args }
    } else { args };
    items.into_iter().reduce(|a, b| if a <= b { a } else { b })
        .ok_or_else(|| "min() requires at least 1 argument".into())
}
fn neba_max(args: Vec<Value>) -> Result<Value, String> {
    let items: Vec<Value> = if args.len() == 1 {
        if let Some(Value::Array(a)) = args.first() {
            let b = a.borrow();
            if b.is_empty() { return Err("max() of empty array".into()); }
            b.clone()
        } else { args }
    } else { args };
    items.into_iter().reduce(|a, b| if a >= b { a } else { b })
        .ok_or_else(|| "max() requires at least 1 argument".into())
}
fn neba_range(args: Vec<Value>) -> Result<Value, String> {
    use std::rc::Rc;
    use std::cell::RefCell;
    let (start, end, step) = match args.as_slice() {
        [Value::Int(e)]                                      => (0, *e, 1),
        [Value::Int(s), Value::Int(e)]                       => (*s, *e, 1),
        [Value::Int(s), Value::Int(e), Value::Int(st)]       => (*s, *e, *st),
        _ => return Err("range() expects 1–3 Int arguments".into()),
    };
    if step == 0 { return Err("range() step cannot be zero".into()); }
    let mut v = Vec::new();
    let mut i = start;
    while (step > 0 && i < end) || (step < 0 && i > end) { v.push(Value::Int(i)); i += step; }
    Ok(Value::Array(Rc::new(RefCell::new(v))))
}
fn neba_push(args: Vec<Value>) -> Result<Value, String> {
    match args.as_slice() {
        [Value::Array(arr), val] => { arr.borrow_mut().push(val.clone()); Ok(Value::None) }
        _ => Err("push(array, value) requires Array and value".into()),
    }
}
fn neba_pop(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(arr)) => arr.borrow_mut().pop().ok_or_else(|| "pop() on empty array".into()),
        _ => Err("pop(array) requires an Array".into()),
    }
}
fn neba_assert(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(v) if v.is_truthy() => Ok(Value::None),
        Some(_) => {
            let msg = args.get(1).map_or("assertion failed".to_string(), |m| m.to_string());
            Err(msg)
        }
        None => Err("assert() requires 1 argument".into()),
    }
}
