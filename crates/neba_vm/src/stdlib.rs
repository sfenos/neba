use std::collections::HashMap;
use crate::value::{Value, RcDict};

pub fn register_globals(globals: &mut HashMap<String, (Value, bool)>) {
    macro_rules! reg {
        ($name:expr, $fn:expr) => {
            globals.insert($name.to_string(), (Value::NativeFn($name.to_string(), $fn), false));
        };
    }
    reg!("print",    neba_print);
    reg!("println",  neba_println);
    reg!("input",    neba_input);
    reg!("len",      neba_len);
    reg!("str",      neba_str);
    reg!("int",      neba_int);
    reg!("float",    neba_float);
    reg!("bool",     neba_bool);
    reg!("typeof",   neba_type);
    reg!("abs",      neba_abs);
    reg!("min",      neba_min);
    reg!("max",      neba_max);
    reg!("range",    neba_range);
    reg!("push",     neba_push);
    reg!("pop",      neba_pop);
    reg!("assert",   neba_assert);
    reg!("clock",    neba_clock);
    // ── Dict ──────────────────────────────────────────────────────────────
    reg!("keys",     neba_keys);
    reg!("values",   neba_values);
    reg!("items",    neba_items);
    reg!("has_key",  neba_has_key);
    reg!("del_key",  neba_del_key);
    // ── List (Array) ──────────────────────────────────────────────────────
    reg!("append",   neba_append);
    reg!("remove",   neba_remove);
    reg!("contains", neba_contains);
    reg!("insert",   neba_insert);
    reg!("sort",     neba_sort);
    reg!("reverse",  neba_reverse);
    reg!("join",     neba_join);
    // ── TypedArray (v0.2.6) ───────────────────────────────────────────────
    register_typed_array_globals(globals);
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
        Some(Value::Array(a))     => Ok(Value::Int(a.borrow().len() as i64)),
        Some(Value::Str(s))       => Ok(Value::Int(s.chars().count() as i64)),
        Some(Value::Dict(d))      => Ok(Value::Int(d.borrow().len() as i64)),
        Some(Value::TypedArray(t))=> Ok(Value::Int(t.borrow().len() as i64)),
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
fn neba_clock(_args: &[Value]) -> Result<Value, String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs_f64();
    Ok(Value::Float(secs))
}

// ── Dict functions ────────────────────────────────────────────────────────

/// keys(dict) → Array di chiavi in ordine di inserimento
fn neba_keys(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Dict(d)) => Ok(Value::array(d.borrow().iter().map(|(k, _)| k.clone()).collect())),
        Some(v) => Err(format!("keys() requires Dict, got {}", v.type_name())),
        None    => Err("keys() requires 1 argument".into()),
    }
}

/// values(dict) → Array di valori in ordine di inserimento
fn neba_values(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Dict(d)) => Ok(Value::array(d.borrow().iter().map(|(_, v)| v.clone()).collect())),
        Some(v) => Err(format!("values() requires Dict, got {}", v.type_name())),
        None    => Err("values() requires 1 argument".into()),
    }
}

/// items(dict) → Array di [chiave, valore] in ordine di inserimento
fn neba_items(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Dict(d)) => Ok(Value::array(
            d.borrow().iter()
                .map(|(k, v)| Value::array(vec![k.clone(), v.clone()]))
                .collect()
        )),
        Some(v) => Err(format!("items() requires Dict, got {}", v.type_name())),
        None    => Err("items() requires 1 argument".into()),
    }
}

/// has_key(dict, key) → Bool
fn neba_has_key(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Dict(d), key] => Ok(Value::Bool(d.borrow().iter().any(|(k, _)| k == key))),
        _ => Err("has_key(dict, key) requires Dict and a key".into()),
    }
}

/// del_key(dict, key) → None (rimuove la chiave se presente)
fn neba_del_key(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Dict(d), key] => {
            let mut d = d.borrow_mut();
            if let Some(pos) = d.iter().position(|(k, _)| k == key) {
                d.remove(pos);
            }
            Ok(Value::None)
        }
        _ => Err("del_key(dict, key) requires Dict and a key".into()),
    }
}

// ── List (Array) functions ────────────────────────────────────────────────

/// append(array, value) → None  (alias di push, nome più comune)
fn neba_append(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(arr), val] => { arr.borrow_mut().push(val.clone()); Ok(Value::None) }
        _ => Err("append(array, value) requires Array and value".into()),
    }
}

/// remove(array, value) → Bool  (rimuove la prima occorrenza, restituisce true se trovata)
fn neba_remove(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(arr), val] => {
            let mut a = arr.borrow_mut();
            if let Some(pos) = a.iter().position(|v| v == val) {
                a.remove(pos);
                Ok(Value::Bool(true))
            } else {
                Ok(Value::Bool(false))
            }
        }
        _ => Err("remove(array, value) requires Array and value".into()),
    }
}

/// contains(array, value) → Bool
fn neba_contains(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(arr), val] => Ok(Value::Bool(arr.borrow().contains(val))),
        [Value::Str(s), Value::Str(sub)] => Ok(Value::Bool(s.contains(sub.as_str()))),
        _ => Err("contains(array, value) requires Array and value".into()),
    }
}

/// insert(array, index, value) → None  (inserisce alla posizione)
fn neba_insert(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(arr), Value::Int(idx), val] => {
            let mut a = arr.borrow_mut();
            let len = a.len() as i64;
            let i = if *idx < 0 { (len + idx).max(0) as usize } else { (*idx as usize).min(a.len()) };
            a.insert(i, val.clone());
            Ok(Value::None)
        }
        _ => Err("insert(array, index, value) requires Array, Int, and value".into()),
    }
}

/// sort(array) → None  (ordina in-place, valori omogenei)
fn neba_sort(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(arr)) => {
            let mut a = arr.borrow_mut();
            a.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
            Ok(Value::None)
        }
        _ => Err("sort(array) requires an Array".into()),
    }
}

/// reverse(array) → None  (inverte in-place)
fn neba_reverse(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(arr)) => { arr.borrow_mut().reverse(); Ok(Value::None) }
        _ => Err("reverse(array) requires an Array".into()),
    }
}

/// join(array, separator) → Str
fn neba_join(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(arr), Value::Str(sep)] => {
            let parts: Vec<String> = arr.borrow().iter().map(|v| v.to_string()).collect();
            Ok(Value::str(parts.join(sep.as_str())))
        }
        [Value::Array(arr)] => {
            let parts: Vec<String> = arr.borrow().iter().map(|v| v.to_string()).collect();
            Ok(Value::str(parts.join("")))
        }
        _ => Err("join(array, sep?) requires Array and optional Str separator".into()),
    }
}

// ── TypedArray functions (v0.2.6 / v0.2.7) ───────────────────────────────

use crate::value::{TypedArrayData, Dtype};

/// Registra le funzioni TypedArray nei globals (chiamata da register_globals)
pub fn register_typed_array_globals(globals: &mut std::collections::HashMap<String, (Value, bool)>) {
    macro_rules! reg {
        ($name:expr, $fn:expr) => {
            globals.insert($name.to_string(), (Value::NativeFn($name.to_string(), $fn), false));
        };
    }
    // Costruttori
    reg!("Float64",   ta_float64);
    reg!("Float32",   ta_float32);
    reg!("Int64",     ta_int64);
    reg!("Int32",     ta_int32);
    // Costruttori di utilità
    reg!("zeros",     ta_zeros);
    reg!("ones",      ta_ones);
    reg!("fill",      ta_fill);
    reg!("linspace",  ta_linspace);
    // Operazioni
    reg!("sum",       ta_sum);
    reg!("mean",      ta_mean);
    reg!("dot",       ta_dot);
    reg!("min_elem",  ta_min_elem);
    reg!("max_elem",  ta_max_elem);
    reg!("to_list",   ta_to_list);
}

// ── Costruttori ──────────────────────────────────────────────────────────

/// Float64([1.0, 2.0, 3.0]) → Float64Array
fn ta_float64(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => {
            let v: Result<Vec<f64>, _> = a.borrow().iter()
                .map(|x| x.as_float().ok_or_else(|| format!("Float64: cannot convert {} to Float64", x.type_name())))
                .collect();
            Ok(Value::typed_array(TypedArrayData::Float64(v?)))
        }
        Some(v) => Err(format!("Float64() expects Array, got {}", v.type_name())),
        None => Err("Float64() requires 1 argument".into()),
    }
}

/// Float32([1.0, 2.0]) → Float32Array
fn ta_float32(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => {
            let v: Result<Vec<f32>, _> = a.borrow().iter()
                .map(|x| x.as_float().map(|f| f as f32).ok_or_else(|| format!("Float32: cannot convert {}", x.type_name())))
                .collect();
            Ok(Value::typed_array(TypedArrayData::Float32(v?)))
        }
        Some(v) => Err(format!("Float32() expects Array, got {}", v.type_name())),
        None => Err("Float32() requires 1 argument".into()),
    }
}

/// Int64([1, 2, 3]) → Int64Array
fn ta_int64(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => {
            let v: Result<Vec<i64>, _> = a.borrow().iter()
                .map(|x| match x {
                    Value::Int(n) => Ok(*n),
                    Value::Float(f) => Ok(*f as i64),
                    _ => Err(format!("Int64: cannot convert {}", x.type_name())),
                })
                .collect();
            Ok(Value::typed_array(TypedArrayData::Int64(v?)))
        }
        Some(v) => Err(format!("Int64() expects Array, got {}", v.type_name())),
        None => Err("Int64() requires 1 argument".into()),
    }
}

/// Int32([1, 2, 3]) → Int32Array
fn ta_int32(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => {
            let v: Result<Vec<i32>, _> = a.borrow().iter()
                .map(|x| match x {
                    Value::Int(n) => Ok(*n as i32),
                    Value::Float(f) => Ok(*f as i32),
                    _ => Err(format!("Int32: cannot convert {}", x.type_name())),
                })
                .collect();
            Ok(Value::typed_array(TypedArrayData::Int32(v?)))
        }
        Some(v) => Err(format!("Int32() expects Array, got {}", v.type_name())),
        None => Err("Int32() requires 1 argument".into()),
    }
}

// ── Costruttori di utilità ────────────────────────────────────────────────

/// zeros(n) → Float64Array di zeri; zeros(n, "Int64") → Int64Array
fn ta_zeros(args: &[Value]) -> Result<Value, String> {
    let (n, dtype) = parse_size_dtype(args, "Float64")?;
    Ok(match dtype.as_str() {
        "Float64" => Value::typed_array(TypedArrayData::Float64(vec![0.0f64; n])),
        "Float32" => Value::typed_array(TypedArrayData::Float32(vec![0.0f32; n])),
        "Int64"   => Value::typed_array(TypedArrayData::Int64(vec![0i64; n])),
        "Int32"   => Value::typed_array(TypedArrayData::Int32(vec![0i32; n])),
        d => return Err(format!("zeros: unknown dtype '{}'", d)),
    })
}

/// ones(n) → Float64Array di uni; ones(n, "Int64") → Int64Array
fn ta_ones(args: &[Value]) -> Result<Value, String> {
    let (n, dtype) = parse_size_dtype(args, "Float64")?;
    Ok(match dtype.as_str() {
        "Float64" => Value::typed_array(TypedArrayData::Float64(vec![1.0f64; n])),
        "Float32" => Value::typed_array(TypedArrayData::Float32(vec![1.0f32; n])),
        "Int64"   => Value::typed_array(TypedArrayData::Int64(vec![1i64; n])),
        "Int32"   => Value::typed_array(TypedArrayData::Int32(vec![1i32; n])),
        d => return Err(format!("ones: unknown dtype '{}'", d)),
    })
}

/// fill(n, value) → Float64Array riempito con value (o IntArray se value è Int)
fn ta_fill(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Int(n), Value::Float(v)] =>
            Ok(Value::typed_array(TypedArrayData::Float64(vec![*v; *n as usize]))),
        [Value::Int(n), Value::Int(v)] =>
            Ok(Value::typed_array(TypedArrayData::Int64(vec![*v; *n as usize]))),
        _ => Err("fill(n: Int, value) requires Int size and numeric fill value".into()),
    }
}

/// linspace(start, stop, n) → Float64Array di n valori equispaziati
fn ta_linspace(args: &[Value]) -> Result<Value, String> {
    let (start, stop, n) = match args {
        [s, e, Value::Int(n)] => {
            let s = s.as_float().ok_or("linspace: start must be numeric")?;
            let e = e.as_float().ok_or("linspace: stop must be numeric")?;
            (s, e, *n as usize)
        }
        _ => return Err("linspace(start, stop, n) requires 3 arguments".into()),
    };
    if n == 0 { return Ok(Value::typed_array(TypedArrayData::Float64(vec![]))); }
    if n == 1 { return Ok(Value::typed_array(TypedArrayData::Float64(vec![start]))); }
    let step = (stop - start) / (n - 1) as f64;
    let v: Vec<f64> = (0..n).map(|i| start + i as f64 * step).collect();
    Ok(Value::typed_array(TypedArrayData::Float64(v)))
}

// ── Operazioni ────────────────────────────────────────────────────────────

/// sum(ta) → valore scalare (somma di tutti gli elementi)
fn ta_sum(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::TypedArray(t)) => {
            let d = t.borrow();
            Ok(match &*d {
                TypedArrayData::Float64(v) => Value::Float(v.iter().sum()),
                TypedArrayData::Float32(v) => Value::Float(v.iter().map(|&x| x as f64).sum()),
                TypedArrayData::Int64(v)   => Value::Int(v.iter().sum()),
                TypedArrayData::Int32(v)   => Value::Int(v.iter().map(|&x| x as i64).sum()),
            })
        }
        Some(Value::Array(a)) => {
            // Fallback per Array normale
            let mut total = 0.0f64;
            for v in a.borrow().iter() {
                total += v.as_float().ok_or_else(|| format!("sum: non-numeric element {}", v.type_name()))?;
            }
            Ok(Value::Float(total))
        }
        _ => Err("sum() requires TypedArray or Array".into()),
    }
}

/// mean(ta) → Float64 (media)
fn ta_mean(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::TypedArray(t)) => {
            let d = t.borrow();
            let n = d.len();
            if n == 0 { return Err("mean() of empty array".into()); }
            let s: f64 = (0..n).map(|i| d.get(i).unwrap().as_float().unwrap()).sum();
            Ok(Value::Float(s / n as f64))
        }
        _ => Err("mean() requires TypedArray".into()),
    }
}

/// dot(a, b) → prodotto scalare (sum of a[i]*b[i])
fn ta_dot(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::TypedArray(a), Value::TypedArray(b)] => {
            let da = a.borrow();
            let db = b.borrow();
            if da.len() != db.len() {
                return Err(format!("dot: size mismatch {} vs {}", da.len(), db.len()));
            }
            let s: f64 = (0..da.len())
                .map(|i| da.get(i).unwrap().as_float().unwrap() * db.get(i).unwrap().as_float().unwrap())
                .sum();
            Ok(Value::Float(s))
        }
        _ => Err("dot(a, b) requires two TypedArrays".into()),
    }
}

/// min_elem(ta) → valore minimo
fn ta_min_elem(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::TypedArray(t)) => {
            let d = t.borrow();
            if d.is_empty() { return Err("min_elem() of empty array".into()); }
            let mut m = d.get(0).unwrap().as_float().unwrap();
            for i in 1..d.len() { let x = d.get(i).unwrap().as_float().unwrap(); if x < m { m = x; } }
            match &*d {
                TypedArrayData::Int64(_) | TypedArrayData::Int32(_) => Ok(Value::Int(m as i64)),
                _ => Ok(Value::Float(m)),
            }
        }
        _ => Err("min_elem() requires TypedArray".into()),
    }
}

/// max_elem(ta) → valore massimo
fn ta_max_elem(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::TypedArray(t)) => {
            let d = t.borrow();
            if d.is_empty() { return Err("max_elem() of empty array".into()); }
            let mut m = d.get(0).unwrap().as_float().unwrap();
            for i in 1..d.len() { let x = d.get(i).unwrap().as_float().unwrap(); if x > m { m = x; } }
            match &*d {
                TypedArrayData::Int64(_) | TypedArrayData::Int32(_) => Ok(Value::Int(m as i64)),
                _ => Ok(Value::Float(m)),
            }
        }
        _ => Err("max_elem() requires TypedArray".into()),
    }
}

/// to_list(ta) → Array dinamico con gli stessi valori
fn ta_to_list(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::TypedArray(t)) => {
            let d = t.borrow();
            let v: Vec<Value> = (0..d.len()).map(|i| d.get(i).unwrap()).collect();
            Ok(Value::array(v))
        }
        _ => Err("to_list() requires TypedArray".into()),
    }
}

// ── Helper interni ────────────────────────────────────────────────────────

fn parse_size_dtype(args: &[Value], default_dtype: &str) -> Result<(usize, String), String> {
    match args {
        [Value::Int(n)] => Ok((*n as usize, default_dtype.to_string())),
        [Value::Int(n), Value::Str(dtype)] => Ok((*n as usize, dtype.as_ref().clone())),
        _ => Err(format!("expected (n: Int) or (n: Int, dtype: Str)")),
    }
}
