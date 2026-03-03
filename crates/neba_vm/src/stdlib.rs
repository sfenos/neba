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
    // ── Stdlib modules (v0.2.11) ─────────────────────────────────────────
    globals.insert("math".to_string(),        (make_math_module(),        false));
    globals.insert("string".to_string(),      (make_string_module(),      false));
    globals.insert("io".to_string(),          (make_io_module(),          false));
    globals.insert("collections".to_string(), (make_collections_module(), false));
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

// ── Stdlib modules v0.2.11 ────────────────────────────────────────────────

/// Costruisce il modulo `math` come Dict di funzioni native + costanti.
pub fn make_math_module() -> Value {
    fn entry(name: &str, f: fn(&[Value]) -> Result<Value, String>) -> (Value, Value) {
        (Value::str(name), Value::NativeFn(name.to_string(), f))
    }
    let pairs = vec![
        // Costanti
        (Value::str("pi"),  Value::Float(std::f64::consts::PI)),
        (Value::str("e"),   Value::Float(std::f64::consts::E)),
        (Value::str("tau"), Value::Float(std::f64::consts::TAU)),
        (Value::str("inf"), Value::Float(f64::INFINITY)),
        (Value::str("nan"), Value::Float(f64::NAN)),
        // Funzioni
        entry("sqrt",   math_sqrt),
        entry("pow",    math_pow),
        entry("exp",    math_exp),
        entry("log",    math_log),
        entry("log2",   math_log2),
        entry("log10",  math_log10),
        entry("sin",    math_sin),
        entry("cos",    math_cos),
        entry("tan",    math_tan),
        entry("asin",   math_asin),
        entry("acos",   math_acos),
        entry("atan",   math_atan),
        entry("atan2",  math_atan2),
        entry("floor",  math_floor),
        entry("ceil",   math_ceil),
        entry("round",  math_round),
        entry("trunc",  math_trunc),
        entry("sign",   math_sign),
        entry("clamp",  math_clamp),
        entry("gcd",    math_gcd),
        entry("lcm",    math_lcm),
        entry("isnan",  math_isnan),
        entry("isinf",  math_isinf),
        entry("degrees",math_degrees),
        entry("radians",math_radians),
        entry("hypot",  math_hypot),
        entry("factorial", math_factorial),
    ];
    Value::dict(pairs)
}

fn as_f64(v: &Value, ctx: &str) -> Result<f64, String> {
    v.as_float().ok_or_else(|| format!("{}: expected numeric, got {}", ctx, v.type_name()))
}

fn math_sqrt(args: &[Value]) -> Result<Value, String> {
    let x = as_f64(args.first().ok_or("sqrt() requires 1 argument")?, "sqrt")?;
    if x < 0.0 { return Err(format!("sqrt() of negative number: {}", x)); }
    Ok(Value::Float(x.sqrt()))
}
fn math_pow(args: &[Value]) -> Result<Value, String> {
    match args {
        [base, exp] => {
            let b = as_f64(base, "pow")?;
            let e = as_f64(exp, "pow")?;
            Ok(Value::Float(b.powf(e)))
        }
        _ => Err("pow(base, exp) requires 2 arguments".into()),
    }
}
fn math_exp(args: &[Value]) -> Result<Value, String> {
    let x = as_f64(args.first().ok_or("exp() requires 1 argument")?, "exp")?;
    Ok(Value::Float(x.exp()))
}
fn math_log(args: &[Value]) -> Result<Value, String> {
    match args {
        [x] => {
            let v = as_f64(x, "log")?;
            if v <= 0.0 { return Err(format!("log() of non-positive: {}", v)); }
            Ok(Value::Float(v.ln()))
        }
        [x, base] => {
            let v = as_f64(x, "log")?;
            let b = as_f64(base, "log")?;
            if v <= 0.0 { return Err(format!("log() of non-positive: {}", v)); }
            if b <= 0.0 || b == 1.0 { return Err(format!("log() invalid base: {}", b)); }
            Ok(Value::Float(v.log(b)))
        }
        _ => Err("log(x) or log(x, base) requires 1–2 arguments".into()),
    }
}
fn math_log2(args: &[Value]) -> Result<Value, String> {
    let x = as_f64(args.first().ok_or("log2() requires 1 argument")?, "log2")?;
    if x <= 0.0 { return Err(format!("log2() of non-positive: {}", x)); }
    Ok(Value::Float(x.log2()))
}
fn math_log10(args: &[Value]) -> Result<Value, String> {
    let x = as_f64(args.first().ok_or("log10() requires 1 argument")?, "log10")?;
    if x <= 0.0 { return Err(format!("log10() of non-positive: {}", x)); }
    Ok(Value::Float(x.log10()))
}
fn math_sin(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Float(as_f64(args.first().ok_or("sin() requires 1 argument")?, "sin")?.sin()))
}
fn math_cos(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Float(as_f64(args.first().ok_or("cos() requires 1 argument")?, "cos")?.cos()))
}
fn math_tan(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Float(as_f64(args.first().ok_or("tan() requires 1 argument")?, "tan")?.tan()))
}
fn math_asin(args: &[Value]) -> Result<Value, String> {
    let x = as_f64(args.first().ok_or("asin() requires 1 argument")?, "asin")?;
    if x < -1.0 || x > 1.0 { return Err(format!("asin() domain error: {}", x)); }
    Ok(Value::Float(x.asin()))
}
fn math_acos(args: &[Value]) -> Result<Value, String> {
    let x = as_f64(args.first().ok_or("acos() requires 1 argument")?, "acos")?;
    if x < -1.0 || x > 1.0 { return Err(format!("acos() domain error: {}", x)); }
    Ok(Value::Float(x.acos()))
}
fn math_atan(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Float(as_f64(args.first().ok_or("atan() requires 1 argument")?, "atan")?.atan()))
}
fn math_atan2(args: &[Value]) -> Result<Value, String> {
    match args {
        [y, x] => Ok(Value::Float(as_f64(y, "atan2")?.atan2(as_f64(x, "atan2")?))),
        _ => Err("atan2(y, x) requires 2 arguments".into()),
    }
}
fn math_floor(args: &[Value]) -> Result<Value, String> {
    let x = as_f64(args.first().ok_or("floor() requires 1 argument")?, "floor")?;
    Ok(Value::Int(x.floor() as i64))
}
fn math_ceil(args: &[Value]) -> Result<Value, String> {
    let x = as_f64(args.first().ok_or("ceil() requires 1 argument")?, "ceil")?;
    Ok(Value::Int(x.ceil() as i64))
}
fn math_round(args: &[Value]) -> Result<Value, String> {
    match args {
        [x] => {
            let v = as_f64(x, "round")?;
            Ok(Value::Int(v.round() as i64))
        }
        [x, Value::Int(decimals)] => {
            let v = as_f64(x, "round")?;
            let factor = 10f64.powi(*decimals as i32);
            Ok(Value::Float((v * factor).round() / factor))
        }
        _ => Err("round(x) or round(x, decimals) requires 1–2 arguments".into()),
    }
}
fn math_trunc(args: &[Value]) -> Result<Value, String> {
    let x = as_f64(args.first().ok_or("trunc() requires 1 argument")?, "trunc")?;
    Ok(Value::Int(x.trunc() as i64))
}
fn math_sign(args: &[Value]) -> Result<Value, String> {
    let x = as_f64(args.first().ok_or("sign() requires 1 argument")?, "sign")?;
    Ok(Value::Int(if x > 0.0 { 1 } else if x < 0.0 { -1 } else { 0 }))
}
fn math_clamp(args: &[Value]) -> Result<Value, String> {
    match args {
        [x, lo, hi] => {
            let v  = as_f64(x,  "clamp")?;
            let lo = as_f64(lo, "clamp")?;
            let hi = as_f64(hi, "clamp")?;
            Ok(Value::Float(v.clamp(lo, hi)))
        }
        _ => Err("clamp(x, min, max) requires 3 arguments".into()),
    }
}
fn math_gcd(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Int(a), Value::Int(b)] => {
            let mut a = a.unsigned_abs();
            let mut b = b.unsigned_abs();
            while b != 0 { let t = b; b = a % b; a = t; }
            Ok(Value::Int(a as i64))
        }
        _ => Err("gcd(a, b) requires 2 Int arguments".into()),
    }
}
fn math_lcm(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Int(a), Value::Int(b)] => {
            if *a == 0 || *b == 0 { return Ok(Value::Int(0)); }
            let mut ua = a.unsigned_abs();
            let mut ub = b.unsigned_abs();
            let orig_a = ua;
            let orig_b = ub;
            while ub != 0 { let t = ub; ub = ua % ub; ua = t; }
            Ok(Value::Int((orig_a / ua * orig_b) as i64))
        }
        _ => Err("lcm(a, b) requires 2 Int arguments".into()),
    }
}
fn math_isnan(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Float(f)) => Ok(Value::Bool(f.is_nan())),
        Some(_) => Ok(Value::Bool(false)),
        None => Err("isnan() requires 1 argument".into()),
    }
}
fn math_isinf(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Float(f)) => Ok(Value::Bool(f.is_infinite())),
        Some(_) => Ok(Value::Bool(false)),
        None => Err("isinf() requires 1 argument".into()),
    }
}
fn math_degrees(args: &[Value]) -> Result<Value, String> {
    let x = as_f64(args.first().ok_or("degrees() requires 1 argument")?, "degrees")?;
    Ok(Value::Float(x.to_degrees()))
}
fn math_radians(args: &[Value]) -> Result<Value, String> {
    let x = as_f64(args.first().ok_or("radians() requires 1 argument")?, "radians")?;
    Ok(Value::Float(x.to_radians()))
}
fn math_hypot(args: &[Value]) -> Result<Value, String> {
    match args {
        [x, y] => Ok(Value::Float(as_f64(x, "hypot")?.hypot(as_f64(y, "hypot")?))),
        _ => Err("hypot(x, y) requires 2 arguments".into()),
    }
}
fn math_factorial(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Int(n)) if *n >= 0 => {
            if *n > 20 { return Err(format!("factorial({}) overflows Int", n)); }
            Ok(Value::Int((1..=*n as u64).product::<u64>() as i64))
        }
        Some(Value::Int(n)) => Err(format!("factorial() requires non-negative Int, got {}", n)),
        _ => Err("factorial(n) requires 1 Int argument".into()),
    }
}

// ── String module ─────────────────────────────────────────────────────────

/// Costruisce il modulo `string` come Dict di funzioni native.
pub fn make_string_module() -> Value {
    fn entry(name: &str, f: fn(&[Value]) -> Result<Value, String>) -> (Value, Value) {
        (Value::str(name), Value::NativeFn(name.to_string(), f))
    }
    let pairs = vec![
        entry("split",      str_split),
        entry("strip",      str_strip),
        entry("lstrip",     str_lstrip),
        entry("rstrip",     str_rstrip),
        entry("upper",      str_upper),
        entry("lower",      str_lower),
        entry("replace",    str_replace),
        entry("find",       str_find),
        entry("rfind",      str_rfind),
        entry("count",      str_count),
        entry("startswith", str_startswith),
        entry("endswith",   str_endswith),
        entry("repeat",     str_repeat),
        entry("pad_left",   str_pad_left),
        entry("pad_right",  str_pad_right),
        entry("chars",      str_chars),
        entry("lines",      str_lines),
        entry("trim",       str_strip),   // alias
        entry("contains",   str_contains_fn),
        entry("is_empty",   str_is_empty),
        entry("index",      str_find),    // alias
        entry("format",     str_format),
    ];
    Value::dict(pairs)
}

fn get_str<'a>(v: &'a Value, ctx: &str) -> Result<&'a str, String> {
    match v {
        Value::Str(s) => Ok(s.as_str()),
        _ => Err(format!("{}: expected Str, got {}", ctx, v.type_name())),
    }
}

fn str_split(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s)] => {
            let parts: Vec<Value> = s.split_whitespace().map(|p| Value::str(p)).collect();
            Ok(Value::array(parts))
        }
        [Value::Str(s), Value::Str(sep)] => {
            let parts: Vec<Value> = s.split(sep.as_str()).map(|p| Value::str(p)).collect();
            Ok(Value::array(parts))
        }
        [Value::Str(s), Value::Str(sep), Value::Int(n)] => {
            let parts: Vec<Value> = s.splitn(*n as usize + 1, sep.as_str()).map(|p| Value::str(p)).collect();
            Ok(Value::array(parts))
        }
        _ => Err("string.split(s, sep?) requires Str".into()),
    }
}
fn str_strip(args: &[Value]) -> Result<Value, String> {
    let s = get_str(args.first().ok_or("strip() requires 1 argument")?, "strip")?;
    Ok(Value::str(s.trim()))
}
fn str_lstrip(args: &[Value]) -> Result<Value, String> {
    let s = get_str(args.first().ok_or("lstrip() requires 1 argument")?, "lstrip")?;
    Ok(Value::str(s.trim_start()))
}
fn str_rstrip(args: &[Value]) -> Result<Value, String> {
    let s = get_str(args.first().ok_or("rstrip() requires 1 argument")?, "rstrip")?;
    Ok(Value::str(s.trim_end()))
}
fn str_upper(args: &[Value]) -> Result<Value, String> {
    let s = get_str(args.first().ok_or("upper() requires 1 argument")?, "upper")?;
    Ok(Value::str(s.to_uppercase()))
}
fn str_lower(args: &[Value]) -> Result<Value, String> {
    let s = get_str(args.first().ok_or("lower() requires 1 argument")?, "lower")?;
    Ok(Value::str(s.to_lowercase()))
}
fn str_replace(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Str(from), Value::Str(to)] => {
            Ok(Value::str(s.replace(from.as_str(), to.as_str())))
        }
        [Value::Str(s), Value::Str(from), Value::Str(to), Value::Int(n)] => {
            Ok(Value::str(s.replacen(from.as_str(), to.as_str(), *n as usize)))
        }
        _ => Err("string.replace(s, from, to, n?) requires 3–4 Str arguments".into()),
    }
}
fn str_find(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Str(sub)] => {
            Ok(Value::Int(s.find(sub.as_str()).map(|i| i as i64).unwrap_or(-1)))
        }
        _ => Err("string.find(s, sub) requires 2 Str arguments".into()),
    }
}
fn str_rfind(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Str(sub)] => {
            Ok(Value::Int(s.rfind(sub.as_str()).map(|i| i as i64).unwrap_or(-1)))
        }
        _ => Err("string.rfind(s, sub) requires 2 Str arguments".into()),
    }
}
fn str_count(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Str(sub)] => {
            if sub.is_empty() { return Err("string.count: empty substring".into()); }
            Ok(Value::Int(s.matches(sub.as_str()).count() as i64))
        }
        _ => Err("string.count(s, sub) requires 2 Str arguments".into()),
    }
}
fn str_startswith(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Str(prefix)] => Ok(Value::Bool(s.starts_with(prefix.as_str()))),
        _ => Err("string.startswith(s, prefix) requires 2 Str arguments".into()),
    }
}
fn str_endswith(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Str(suffix)] => Ok(Value::Bool(s.ends_with(suffix.as_str()))),
        _ => Err("string.endswith(s, suffix) requires 2 Str arguments".into()),
    }
}
fn str_repeat(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Int(n)] => {
            if *n < 0 { return Err(format!("string.repeat: negative count {}", n)); }
            Ok(Value::str(s.repeat(*n as usize)))
        }
        _ => Err("string.repeat(s, n) requires Str and Int".into()),
    }
}
fn str_pad_left(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Int(width)] => {
            let w = *width as usize;
            let len = s.chars().count();
            if len >= w { return Ok(Value::Str(s.clone())); }
            Ok(Value::str(format!("{:>width$}", s.as_str(), width = w)))
        }
        [Value::Str(s), Value::Int(width), Value::Str(ch)] => {
            let w = *width as usize;
            let pad_char = ch.chars().next().unwrap_or(' ');
            let len = s.chars().count();
            if len >= w { return Ok(Value::Str(s.clone())); }
            let padding: String = std::iter::repeat(pad_char).take(w - len).collect();
            Ok(Value::str(format!("{}{}", padding, s.as_str())))
        }
        _ => Err("string.pad_left(s, width, ch?) requires Str, Int and optional Str".into()),
    }
}
fn str_pad_right(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Int(width)] => {
            let w = *width as usize;
            Ok(Value::str(format!("{:<width$}", s.as_str(), width = w)))
        }
        [Value::Str(s), Value::Int(width), Value::Str(ch)] => {
            let w = *width as usize;
            let pad_char = ch.chars().next().unwrap_or(' ');
            let len = s.chars().count();
            if len >= w { return Ok(Value::Str(s.clone())); }
            let padding: String = std::iter::repeat(pad_char).take(w - len).collect();
            Ok(Value::str(format!("{}{}", s.as_str(), padding)))
        }
        _ => Err("string.pad_right(s, width, ch?) requires Str, Int and optional Str".into()),
    }
}
fn str_chars(args: &[Value]) -> Result<Value, String> {
    let s = get_str(args.first().ok_or("chars() requires 1 argument")?, "chars")?;
    Ok(Value::array(s.chars().map(|c| Value::str(c.to_string())).collect()))
}
fn str_lines(args: &[Value]) -> Result<Value, String> {
    let s = get_str(args.first().ok_or("lines() requires 1 argument")?, "lines")?;
    Ok(Value::array(s.lines().map(|l| Value::str(l)).collect()))
}
fn str_contains_fn(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Str(sub)] => Ok(Value::Bool(s.contains(sub.as_str()))),
        _ => Err("string.contains(s, sub) requires 2 Str arguments".into()),
    }
}
fn str_is_empty(args: &[Value]) -> Result<Value, String> {
    let s = get_str(args.first().ok_or("is_empty() requires 1 argument")?, "is_empty")?;
    Ok(Value::Bool(s.is_empty()))
}
/// string.format(template, dict_or_values...) — sostituisce {key} con valori dal dict
fn str_format(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(tmpl), Value::Dict(d)] => {
            let mut result = tmpl.as_ref().clone();
            for (k, v) in d.borrow().iter() {
                if let Value::Str(key) = k {
                    let placeholder = format!("{{{}}}", key.as_str());
                    result = result.replace(&placeholder, &v.to_string());
                }
            }
            Ok(Value::str(result))
        }
        _ => Err("string.format(template, dict) requires Str and Dict".into()),
    }
}

// ── IO module ─────────────────────────────────────────────────────────────

/// Costruisce il modulo `io` come Dict di funzioni per I/O file.
pub fn make_io_module() -> Value {
    fn entry(name: &str, f: fn(&[Value]) -> Result<Value, String>) -> (Value, Value) {
        (Value::str(name), Value::NativeFn(name.to_string(), f))
    }
    let pairs = vec![
        entry("read_file",   io_read_file),
        entry("write_file",  io_write_file),
        entry("append_file", io_append_file),
        entry("file_exists", io_file_exists),
        entry("read_lines",  io_read_lines),
        entry("delete_file", io_delete_file),
    ];
    Value::dict(pairs)
}

fn io_read_file(args: &[Value]) -> Result<Value, String> {
    let path = get_str(args.first().ok_or("io.read_file() requires 1 argument")?, "io.read_file")?;
    std::fs::read_to_string(path)
        .map(|s| Value::str(s))
        .map_err(|e| format!("io.read_file('{}'): {}", path, e))
}
fn io_write_file(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(path), Value::Str(content)] => {
            std::fs::write(path.as_str(), content.as_str())
                .map(|_| Value::None)
                .map_err(|e| format!("io.write_file('{}'): {}", path, e))
        }
        _ => Err("io.write_file(path, content) requires 2 Str arguments".into()),
    }
}
fn io_append_file(args: &[Value]) -> Result<Value, String> {
    use std::io::Write;
    match args {
        [Value::Str(path), Value::Str(content)] => {
            std::fs::OpenOptions::new()
                .create(true).append(true).open(path.as_str())
                .and_then(|mut f| f.write_all(content.as_bytes()))
                .map(|_| Value::None)
                .map_err(|e| format!("io.append_file('{}'): {}", path, e))
        }
        _ => Err("io.append_file(path, content) requires 2 Str arguments".into()),
    }
}
fn io_file_exists(args: &[Value]) -> Result<Value, String> {
    let path = get_str(args.first().ok_or("io.file_exists() requires 1 argument")?, "io.file_exists")?;
    Ok(Value::Bool(std::path::Path::new(path).exists()))
}
fn io_read_lines(args: &[Value]) -> Result<Value, String> {
    let path = get_str(args.first().ok_or("io.read_lines() requires 1 argument")?, "io.read_lines")?;
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("io.read_lines('{}'): {}", path, e))?;
    Ok(Value::array(content.lines().map(|l| Value::str(l)).collect()))
}
fn io_delete_file(args: &[Value]) -> Result<Value, String> {
    let path = get_str(args.first().ok_or("io.delete_file() requires 1 argument")?, "io.delete_file")?;
    std::fs::remove_file(path)
        .map(|_| Value::None)
        .map_err(|e| format!("io.delete_file('{}'): {}", path, e))
}

// ── Collections module ────────────────────────────────────────────────────

/// Costruisce il modulo `collections` come Dict di funzioni.
/// Nota: HOF (map, filter, reduce) richiedono callback nel VM — implementati come
/// funzioni globali separate; il modulo espone utility non-HOF.
pub fn make_collections_module() -> Value {
    fn entry(name: &str, f: fn(&[Value]) -> Result<Value, String>) -> (Value, Value) {
        (Value::str(name), Value::NativeFn(name.to_string(), f))
    }
    let pairs = vec![
        entry("zip",        col_zip),
        entry("enumerate",  col_enumerate),
        entry("flatten",    col_flatten),
        entry("unique",     col_unique),
        entry("sorted",     col_sorted),
        entry("chunk",      col_chunk),
        entry("take",       col_take),
        entry("drop",       col_drop),
        entry("count_by",   col_count_by_value),
        entry("sum",        col_sum),
        entry("product",    col_product),
        entry("any",        col_any),
        entry("all",        col_all),
        entry("none",       col_none),
        entry("first",      col_first),
        entry("last",       col_last),
        entry("concat",     col_concat),
        entry("repeat",     col_repeat),
        entry("transpose",  col_transpose),
    ];
    Value::dict(pairs)
}

fn col_zip(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(a), Value::Array(b)] => {
            let a = a.borrow();
            let b = b.borrow();
            let result: Vec<Value> = a.iter().zip(b.iter())
                .map(|(x, y)| Value::array(vec![x.clone(), y.clone()]))
                .collect();
            Ok(Value::array(result))
        }
        _ => Err("collections.zip(a, b) requires 2 Arrays".into()),
    }
}
fn col_enumerate(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => {
            let start = if let Some(Value::Int(s)) = args.get(1) { *s } else { 0 };
            let result: Vec<Value> = a.borrow().iter().enumerate()
                .map(|(i, v)| Value::array(vec![Value::Int(i as i64 + start), v.clone()]))
                .collect();
            Ok(Value::array(result))
        }
        _ => Err("collections.enumerate(array, start?) requires Array".into()),
    }
}
fn col_flatten(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => {
            let mut result = Vec::new();
            for item in a.borrow().iter() {
                match item {
                    Value::Array(inner) => result.extend(inner.borrow().iter().cloned()),
                    other => result.push(other.clone()),
                }
            }
            Ok(Value::array(result))
        }
        _ => Err("collections.flatten(array) requires Array".into()),
    }
}
fn col_unique(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => {
            let mut seen: Vec<Value> = Vec::new();
            for item in a.borrow().iter() {
                if !seen.contains(item) { seen.push(item.clone()); }
            }
            Ok(Value::array(seen))
        }
        _ => Err("collections.unique(array) requires Array".into()),
    }
}
fn col_sorted(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => {
            let mut v = a.borrow().clone();
            v.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
            Ok(Value::array(v))
        }
        _ => Err("collections.sorted(array) requires Array".into()),
    }
}
fn col_chunk(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(a), Value::Int(n)] => {
            if *n <= 0 { return Err("collections.chunk: chunk size must be > 0".into()); }
            let n = *n as usize;
            let a = a.borrow();
            let chunks: Vec<Value> = a.chunks(n)
                .map(|c| Value::array(c.to_vec()))
                .collect();
            Ok(Value::array(chunks))
        }
        _ => Err("collections.chunk(array, n) requires Array and Int".into()),
    }
}
fn col_take(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(a), Value::Int(n)] => {
            let n = (*n).max(0) as usize;
            Ok(Value::array(a.borrow().iter().take(n).cloned().collect()))
        }
        _ => Err("collections.take(array, n) requires Array and Int".into()),
    }
}
fn col_drop(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(a), Value::Int(n)] => {
            let n = (*n).max(0) as usize;
            Ok(Value::array(a.borrow().iter().skip(n).cloned().collect()))
        }
        _ => Err("collections.drop(array, n) requires Array and Int".into()),
    }
}
fn col_count_by_value(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(a), val] => {
            let count = a.borrow().iter().filter(|v| *v == val).count();
            Ok(Value::Int(count as i64))
        }
        _ => Err("collections.count_by(array, value) requires Array and value".into()),
    }
}
fn col_sum(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => {
            let arr = a.borrow();
            if arr.is_empty() { return Ok(Value::Int(0)); }
            let mut total = 0.0f64;
            let mut all_int = true;
            for v in arr.iter() {
                match v {
                    Value::Int(n) => total += *n as f64,
                    Value::Float(f) => { total += f; all_int = false; }
                    _ => return Err(format!("collections.sum: non-numeric element {}", v.type_name())),
                }
            }
            if all_int { Ok(Value::Int(total as i64)) } else { Ok(Value::Float(total)) }
        }
        _ => Err("collections.sum(array) requires Array".into()),
    }
}
fn col_product(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => {
            let arr = a.borrow();
            if arr.is_empty() { return Ok(Value::Int(1)); }
            let mut total = 1.0f64;
            let mut all_int = true;
            for v in arr.iter() {
                match v {
                    Value::Int(n) => total *= *n as f64,
                    Value::Float(f) => { total *= f; all_int = false; }
                    _ => return Err(format!("collections.product: non-numeric element {}", v.type_name())),
                }
            }
            if all_int { Ok(Value::Int(total as i64)) } else { Ok(Value::Float(total)) }
        }
        _ => Err("collections.product(array) requires Array".into()),
    }
}
fn col_any(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => Ok(Value::Bool(a.borrow().iter().any(|v| v.is_truthy()))),
        _ => Err("collections.any(array) requires Array".into()),
    }
}
fn col_all(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => Ok(Value::Bool(a.borrow().iter().all(|v| v.is_truthy()))),
        _ => Err("collections.all(array) requires Array".into()),
    }
}
fn col_none(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => Ok(Value::Bool(!a.borrow().iter().any(|v| v.is_truthy()))),
        _ => Err("collections.none(array) requires Array".into()),
    }
}
fn col_first(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => {
            Ok(a.borrow().first().cloned().map(|v| Value::Some_(Box::new(v))).unwrap_or(Value::None))
        }
        _ => Err("collections.first(array) requires Array".into()),
    }
}
fn col_last(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => {
            Ok(a.borrow().last().cloned().map(|v| Value::Some_(Box::new(v))).unwrap_or(Value::None))
        }
        _ => Err("collections.last(array) requires Array".into()),
    }
}
fn col_concat(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(a), Value::Array(b)] => {
            let mut result = a.borrow().clone();
            result.extend(b.borrow().iter().cloned());
            Ok(Value::array(result))
        }
        _ => Err("collections.concat(a, b) requires 2 Arrays".into()),
    }
}
fn col_repeat(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(a), Value::Int(n)] => {
            if *n < 0 { return Err(format!("collections.repeat: negative count {}", n)); }
            let arr = a.borrow();
            let result: Vec<Value> = arr.iter().cloned().cycle().take(arr.len() * *n as usize).collect();
            Ok(Value::array(result))
        }
        _ => Err("collections.repeat(array, n) requires Array and Int".into()),
    }
}
fn col_transpose(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(outer)) => {
            let outer = outer.borrow();
            if outer.is_empty() { return Ok(Value::array(vec![])); }
            let row_len = match &outer[0] {
                Value::Array(r) => r.borrow().len(),
                _ => return Err("collections.transpose: inner elements must be Arrays".into()),
            };
            let mut result: Vec<Vec<Value>> = (0..row_len).map(|_| Vec::new()).collect();
            for row in outer.iter() {
                match row {
                    Value::Array(r) => {
                        let r = r.borrow();
                        if r.len() != row_len {
                            return Err("collections.transpose: rows must have equal length".into());
                        }
                        for (col_idx, val) in r.iter().enumerate() {
                            result[col_idx].push(val.clone());
                        }
                    }
                    _ => return Err("collections.transpose: inner elements must be Arrays".into()),
                }
            }
            Ok(Value::array(result.into_iter().map(Value::array).collect()))
        }
        _ => Err("collections.transpose(matrix) requires Array of Arrays".into()),
    }
}
