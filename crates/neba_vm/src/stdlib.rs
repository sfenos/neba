use std::collections::HashMap;
use rustc_hash::FxHashMap;
use crate::value::{Value, TypedArrayData, Dtype, NdArray};
pub fn register_globals(globals: &mut FxHashMap<String, (Value, bool)>) {
    macro_rules! reg {
        ($name:expr, $fn:expr) => {
            globals.insert($name.to_string(), (Value::native_fn($name, $fn), false));
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
    reg!("type",     neba_type);  // alias
    reg!("abs",      neba_abs);
    reg!("min",      neba_min);
    reg!("max",      neba_max);
    reg!("range",    neba_range);
    reg!("push",     neba_push);
    reg!("pop",      neba_pop);
    reg!("assert",   neba_assert);
    reg!("clock",    neba_clock);
    reg!("time_ms",  neba_time_ms);
    // ── Globali aggiuntivi (v0.2.15) ──────────────────────────────────────
    reg!("sum",       neba_sum);
    reg!("zip",       neba_zip);
    reg!("enumerate", neba_enumerate);
    reg!("sorted",    neba_sorted);
    reg!("any",       neba_any);
    reg!("all",       neba_all);
    reg!("chr",       neba_chr);
    reg!("ord",       neba_ord);
    reg!("copy",      neba_copy);
    reg!("hex",       neba_hex);
    reg!("bin",       neba_bin);
    reg!("oct",       neba_oct);
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
    // ── String convenience globals (v0.2.25) ─────────────────────────────
    reg!("upper",      str_upper);
    reg!("lower",      str_lower);
    reg!("strip",      str_strip);
    reg!("lstrip",     str_lstrip);
    reg!("rstrip",     str_rstrip);
    reg!("split",      str_split);
    reg!("replace",    str_replace);
    reg!("find",       str_find);
    reg!("startswith", str_startswith);
    reg!("starts_with",str_startswith);  // alias
    reg!("endswith",   str_endswith);
    reg!("ends_with",  str_endswith);    // alias
    reg!("capitalize", str_capitalize);
    reg!("title",      str_title);
    reg!("format",     neba_format);
    reg!("zfill",      str_zfill);
    // ── TypedArray (v0.2.6) ───────────────────────────────────────────────
    register_typed_array_globals(globals);
    register_nd_module(globals);
    // ── Stdlib modules (v0.2.11) ─────────────────────────────────────────
    globals.insert("math".to_string(),        (make_math_module(),        false));
    globals.insert("string".to_string(),      (make_string_module(),      false));
    globals.insert("io".to_string(),          (make_io_module(),          false));
    globals.insert("collections".to_string(), (make_collections_module(), false));
    globals.insert("random".to_string(),      (make_random_module(),      false));
    // ── HOF: map / filter / reduce (v0.2.12) ─────────────────────────────
    // Il body è un placeholder — vengono intercettati in Op::Call dalla VM
    // prima del dispatch normale, quindi questa fn non viene mai eseguita.
    reg!("map",    hof_map_stub);
    reg!("filter", hof_filter_stub);
    reg!("reduce", hof_reduce_stub);
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
        Some(Value::Array(a))      => Ok(Value::Int(a.borrow().len() as i64)),
        Some(Value::Str(s))        => Ok(Value::Int(s.chars().count() as i64)),
        Some(Value::Dict(d))       => Ok(Value::Int(d.borrow().len() as i64)),
        Some(Value::TypedArray(t)) => Ok(Value::Int(t.borrow().len() as i64)),
        Some(Value::IntRange(s, e, inc)) => {
            let len = if *inc { (e - s + 1).max(0) } else { (e - s).max(0) };
            Ok(Value::Int(len))
        }
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
        match args.first() {
            Some(Value::Array(a)) => {
                let b = a.borrow(); if b.is_empty() { return Err("min() of empty array".into()); }
                b.clone()
            }
            Some(Value::TypedArray(ta)) => {
                let b = ta.borrow();
                if b.is_empty() { return Err("min() of empty TypedArray".into()); }
                return match &*b {
                    TypedArrayData::Float64(v) => Ok(Value::Float(v.iter().cloned().fold(f64::INFINITY, f64::min))),
                    TypedArrayData::Float32(v) => Ok(Value::Float(v.iter().cloned().fold(f32::INFINITY, f32::min) as f64)),
                    TypedArrayData::Int64(v)   => Ok(Value::Int(*v.iter().min().unwrap())),
                    TypedArrayData::Int32(v)   => Ok(Value::Int(*v.iter().min().unwrap() as i64)),
                };
            }
            _ => args.to_vec()
        }
    } else { args.to_vec() };
    items.into_iter().reduce(|a, b| if a <= b { a } else { b })
        .ok_or_else(|| "min() requires at least 1 argument".into())
}
fn neba_max(args: &[Value]) -> Result<Value, String> {
    let items: Vec<Value> = if args.len() == 1 {
        match args.first() {
            Some(Value::Array(a)) => {
                let b = a.borrow(); if b.is_empty() { return Err("max() of empty array".into()); }
                b.clone()
            }
            Some(Value::TypedArray(ta)) => {
                let b = ta.borrow();
                if b.is_empty() { return Err("max() of empty TypedArray".into()); }
                return match &*b {
                    TypedArrayData::Float64(v) => Ok(Value::Float(v.iter().cloned().fold(f64::NEG_INFINITY, f64::max))),
                    TypedArrayData::Float32(v) => Ok(Value::Float(v.iter().cloned().fold(f32::NEG_INFINITY, f32::max) as f64)),
                    TypedArrayData::Int64(v)   => Ok(Value::Int(*v.iter().max().unwrap())),
                    TypedArrayData::Int32(v)   => Ok(Value::Int(*v.iter().max().unwrap() as i64)),
                };
            }
            _ => args.to_vec()
        }
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

fn neba_time_ms(_args: &[Value]) -> Result<Value, String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis() as i64;
    Ok(Value::Int(ms))
}

// ── Nuove funzioni globali v0.2.15 ────────────────────────────────────────

/// sum(array|range|typedarray) → Int|Float
fn neba_sum(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::TypedArray(t)) => {
            let d = t.borrow();
            return Ok(match &*d {
                TypedArrayData::Float64(v) => Value::Float(v.iter().sum()),
                TypedArrayData::Float32(v) => Value::Float(v.iter().map(|&x| x as f64).sum()),
                TypedArrayData::Int64(v)   => Value::Int(v.iter().sum()),
                TypedArrayData::Int32(v)   => Value::Int(v.iter().map(|&x| x as i64).sum()),
            });
        }
        Some(Value::IntRange(s, e, inc)) => {
            // Formula di Gauss — O(1), zero allocazioni
            let (s, e, inc) = (*s, *e, *inc);
            return Ok(if inc {
                let n = (e - s + 1).max(0);
                Value::Int(n * (s + e) / 2)
            } else {
                let n = (e - s).max(0);
                if n == 0 { Value::Int(0) } else { Value::Int(n * (s + (e - 1)) / 2) }
            });
        }
        Some(Value::Array(a)) => {
            let arr = a.borrow();
            if arr.is_empty() { return Ok(Value::Int(0)); }
            let mut isum = 0i64;
            let mut fsum = 0.0f64;
            let mut is_float = false;
            for v in arr.iter() {
                match v {
                    Value::Int(n)   => { isum += n; fsum += *n as f64; }
                    Value::Float(f) => { fsum += f; is_float = true; }
                    _ => return Err(format!("sum(): non-numeric element {}", v.type_name())),
                }
            }
            Ok(if is_float { Value::Float(fsum) } else { Value::Int(isum) })
        }
        _ => Err("sum() requires Array, Range, or TypedArray".into()),
    }
}

/// zip(a, b) → Array di [a[i], b[i]]
fn neba_zip(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(a), Value::Array(b)] => {
            let a = a.borrow(); let b = b.borrow();
            let result = a.iter().zip(b.iter())
                .map(|(x, y)| Value::array(vec![x.clone(), y.clone()]))
                .collect();
            Ok(Value::array(result))
        }
        _ => Err("zip(a, b) requires 2 Arrays".into()),
    }
}

/// enumerate(array, start=0) → Array di [index, value]
fn neba_enumerate(args: &[Value]) -> Result<Value, String> {
    let (arr, start) = match args {
        [Value::Array(a)]                    => (a, 0i64),
        [Value::Array(a), Value::Int(s)]     => (a, *s),
        _ => return Err("enumerate(array, start=0) requires Array".into()),
    };
    let result = arr.borrow().iter().enumerate()
        .map(|(i, v)| Value::array(vec![Value::Int(i as i64 + start), v.clone()]))
        .collect();
    Ok(Value::array(result))
}

/// sorted(array) → nuova Array ordinata (non modifica l'originale)
fn neba_sorted(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => {
            let mut v = a.borrow().clone();
            v.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
            Ok(Value::array(v))
        }
        _ => Err("sorted(array) requires Array".into()),
    }
}

/// any(array) → Bool: true se almeno un elemento è truthy
fn neba_any(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => Ok(Value::Bool(a.borrow().iter().any(|v| v.is_truthy()))),
        _ => Err("any(array) requires Array".into()),
    }
}

/// all(array) → Bool: true se tutti gli elementi sono truthy
fn neba_all(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a)) => Ok(Value::Bool(a.borrow().iter().all(|v| v.is_truthy()))),
        _ => Err("all(array) requires Array".into()),
    }
}

/// chr(n) → Str: carattere Unicode con codepoint n
fn neba_chr(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Int(n)) => {
            let n = *n;
            if n < 0 || n > 0x10FFFF {
                return Err(format!("chr(): codepoint {} out of range 0–1114111", n));
            }
            match char::from_u32(n as u32) {
                Some(c) => Ok(Value::str(c.to_string())),
                None    => Err(format!("chr(): {} is not a valid Unicode codepoint", n)),
            }
        }
        _ => Err("chr(n) requires Int".into()),
    }
}

/// ord(s) → Int: codepoint Unicode del primo carattere
fn neba_ord(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Str(s)) => {
            match s.chars().next() {
                Some(c) => Ok(Value::Int(c as i64)),
                None    => Err("ord(): empty string".into()),
            }
        }
        _ => Err("ord(s) requires Str".into()),
    }
}

/// copy(array) → Array: copia superficiale dell'array
fn neba_copy(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(a))      => Ok(Value::array(a.borrow().clone())),
        Some(Value::Dict(d))       => Ok(Value::dict_from_map(d.borrow().clone())),
        Some(Value::TypedArray(t)) => Ok(Value::typed_array(t.borrow().clone())),
        Some(v) => Err(format!("copy() not supported for {}", v.type_name())),
        None    => Err("copy() requires 1 argument".into()),
    }
}

/// hex(n) → Str: rappresentazione esadecimale ("0x1f")
fn neba_hex(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Int(n)) => Ok(Value::str(format!("0x{:x}", n))),
        _ => Err("hex(n) requires Int".into()),
    }
}

/// bin(n) → Str: rappresentazione binaria ("0b1010")
fn neba_bin(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Int(n)) => Ok(Value::str(format!("0b{:b}", n))),
        _ => Err("bin(n) requires Int".into()),
    }
}

/// oct(n) → Str: rappresentazione ottale ("0o17")
fn neba_oct(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Int(n)) => Ok(Value::str(format!("0o{:o}", n))),
        _ => Err("oct(n) requires Int".into()),
    }
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
        [Value::Dict(d), key] => Ok(Value::Bool(d.borrow().contains_key(key))),
        _ => Err("has_key(dict, key) requires Dict and a key".into()),
    }
}

/// del_key(dict, key) → None (rimuove la chiave se presente)
fn neba_del_key(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Dict(d), key] => {
            d.borrow_mut().shift_remove(key);
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
        Some(Value::Array(arr)) => {
            arr.borrow_mut().reverse();
            Ok(Value::None)
        }
        Some(Value::Str(s)) => {
            Ok(Value::str(s.chars().rev().collect::<String>()))
        }
        _ => Err("reverse(array) requires an Array or Str".into()),
    }
}

/// join(array, separator) → Str
/// format("{} {}", arg1, arg2) — sostituisce {} in ordine con gli argomenti
fn neba_format(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Str(tmpl)) => {
            // Controllo se è la vecchia sintassi {key}: dict come secondo arg
            if args.len() == 2 {
                if let Value::Dict(d) = &args[1] {
                    let mut result = tmpl.as_ref().clone();
                    for (k, v) in d.borrow().iter() {
                        if let Value::Str(key) = k {
                            result = result.replace(&format!("{{{}}}", key.as_str()), &v.to_string());
                        }
                    }
                    return Ok(Value::str(result));
                }
            }
            // Sintassi positionale: format("{} + {} = {}", 1, 2, 3)
            let mut result = String::new();
            let mut arg_idx = 1usize;
            let mut chars = tmpl.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == '{' {
                    if chars.peek() == Some(&'}') {
                        chars.next();
                        let val = args.get(arg_idx).unwrap_or(&Value::None);
                        result.push_str(&val.to_string());
                        arg_idx += 1;
                    } else {
                        result.push(ch);
                    }
                } else {
                    result.push(ch);
                }
            }
            Ok(Value::str(result))
        }
        _ => Err("format(template, ...args) requires a Str template".into()),
    }
}

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



/// Registra le funzioni TypedArray nei globals (chiamata da register_globals)
pub fn register_typed_array_globals(globals: &mut FxHashMap<String, (Value, bool)>) {
    macro_rules! reg {
        ($name:expr, $fn:expr) => {
            globals.insert($name.to_string(), (Value::native_fn($name, $fn), false));
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
    // Operazioni (sum è gestita da neba_sum globale che copre Array/Range/TypedArray)
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
    // zeros(r, c, ...) → NdArray 2D/ND se ≥2 Int args
    // zeros([r,c,...])  → NdArray via nd_zeros
    // zeros(n)          → TypedArray 1D (backward compat)
    let all_ints = !args.is_empty() && args.iter().all(|v| matches!(v, Value::Int(_)));
    if args.len() >= 2 && all_ints {
        let shape: Vec<usize> = args.iter().map(|v| if let Value::Int(n) = v { *n as usize } else { 1 }).collect();
        return Ok(Value::nd_array(crate::value::NdArray::zeros(shape, crate::value::Dtype::Float64)));
    }
    if matches!(args.first(), Some(Value::Array(_))) {
        return nd_zeros(args);
    }
    let (n, dtype) = parse_size_dtype(args, "Float64")?;
    Ok(match dtype.as_str() {
        "Float64" => Value::typed_array(TypedArrayData::Float64(vec![0.0f64; n])),
        "Float32" => Value::typed_array(TypedArrayData::Float32(vec![0.0f32; n])),
        "Int64"   => Value::typed_array(TypedArrayData::Int64(vec![0i64; n])),
        "Int32"   => Value::typed_array(TypedArrayData::Int32(vec![0i32; n])),
        d => return Err(format!("zeros: unknown dtype '{}'", d)),
    })
}

/// ones(n) → Float64Array; ones(r, c, ...) → NdArray di uni
fn ta_ones(args: &[Value]) -> Result<Value, String> {
    let all_ints = !args.is_empty() && args.iter().all(|v| matches!(v, Value::Int(_)));
    if args.len() >= 2 && all_ints {
        let shape: Vec<usize> = args.iter().map(|v| if let Value::Int(n) = v { *n as usize } else { 1 }).collect();
        let mut nd = crate::value::NdArray::zeros(shape, crate::value::Dtype::Float64);
        for i in 0..nd.size() { nd.data.set(i, Value::Float(1.0)).unwrap(); }
        return Ok(Value::nd_array(nd));
    }
    if matches!(args.first(), Some(Value::Array(_))) {
        return nd_ones(args);
    }
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
        (Value::str(name), Value::native_fn(name, f))
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
        // v0.2.18
        entry("sinh",    math_sinh),
        entry("cosh",    math_cosh),
        entry("tanh",    math_tanh),
        entry("cbrt",    math_cbrt),
        entry("log1p",   math_log1p),
        entry("expm1",   math_expm1),
        entry("abs",     math_abs),
        entry("random",  math_random),
        entry("randint", math_randint),
        entry("choice",  math_choice),
        entry("shuffle", math_shuffle),
        entry("seed",    math_seed),
        entry("sample",  math_sample),
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
        (Value::str(name), Value::native_fn(name, f))
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
        entry("startswith",  str_startswith),
        entry("starts_with", str_startswith),  // alias
        entry("endswith",    str_endswith),
        entry("ends_with",   str_endswith),    // alias
        entry("repeat",     str_repeat),
        entry("pad_left",   str_pad_left),
        entry("pad_right",  str_pad_right),
        entry("chars",      str_chars),
        entry("lines",      str_lines),
        entry("trim",       str_strip),   // alias
        entry("contains",   str_contains_fn),
        entry("join",       str_join),
        entry("is_empty",   str_is_empty),
        entry("index",      str_find),    // alias
        entry("format",     str_format),
        // v0.2.18
        entry("zfill",      str_zfill),
        entry("center",     str_center),
        entry("ljust",      str_ljust),
        entry("rjust",      str_rjust),
        entry("to_int",     str_to_int),
        entry("to_float",   str_to_float),
        entry("is_digit",   str_is_digit),
        entry("is_alpha",   str_is_alpha),
        entry("is_alnum",   str_is_alnum),
        entry("is_upper",   str_is_upper),
        entry("is_lower",   str_is_lower),
        entry("capitalize", str_capitalize),
        entry("title",      str_title),
        entry("slice",      str_slice),
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

fn str_join(args: &[Value]) -> Result<Value, String> {
    // string.join(sep, array)  OR  string.join(array, sep)
    match args {
        [Value::Str(sep), Value::Array(arr)] | [Value::Array(arr), Value::Str(sep)] => {
            let parts: Vec<String> = arr.borrow().iter().map(|v| v.to_string()).collect();
            Ok(Value::str(parts.join(sep.as_str())))
        }
        _ => Err("string.join(sep, array) requires Str sep and Array".into()),
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
        (Value::str(name), Value::native_fn(name, f))
    }
    let pairs = vec![
        entry("read_file",   io_read_file),
        entry("write_file",  io_write_file),
        entry("append_file", io_append_file),
        entry("file_exists", io_file_exists),
        entry("read_lines",  io_read_lines),
        entry("delete_file", io_delete_file),
        // v0.2.18: nuove io functions
        entry("listdir",    io_listdir),
        entry("cwd",        io_cwd),
        entry("mkdir",      io_mkdir),
        // io.path come sotto-dizionario
        (Value::str("path"), {
            fn e(name: &str, f: fn(&[Value]) -> Result<Value, String>) -> (Value, Value) {
                (Value::str(name), Value::native_fn(name, f))
            }
            Value::dict(vec![
                e("join",     io_path_join),
                e("dirname",  io_path_dirname),
                e("basename", io_path_basename),
                e("stem",     io_path_stem),
                e("ext",      io_path_ext),
                e("exists",   io_path_exists),
                e("isfile",   io_path_isfile),
                e("isdir",    io_path_isdir),
            ])
        }),
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
        (Value::str(name), Value::native_fn(name, f))
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

// ── HOF stubs (mai eseguiti — intercettati in Op::Call dalla VM) ──────────
fn hof_map_stub(_: &[Value]) -> Result<Value, String> {
    Err("map: should have been intercepted by VM HOF dispatch".into())
}
fn hof_filter_stub(_: &[Value]) -> Result<Value, String> {
    Err("filter: should have been intercepted by VM HOF dispatch".into())
}
fn hof_reduce_stub(_: &[Value]) -> Result<Value, String> {
    Err("reduce: should have been intercepted by VM HOF dispatch".into())
}

// ═══════════════════════════════════════════════════════════════════════════
// v0.2.18 — math extensions, string extensions, random module, io.path
// ═══════════════════════════════════════════════════════════════════════════

// ── math: sinh / cosh / tanh / cbrt / log1p / expm1 ─────────────────────

fn math_sinh(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Float(as_f64(args.first().ok_or("sinh() requires 1 argument")?, "sinh")?.sinh()))
}
fn math_cosh(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Float(as_f64(args.first().ok_or("cosh() requires 1 argument")?, "cosh")?.cosh()))
}
fn math_tanh(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Float(as_f64(args.first().ok_or("tanh() requires 1 argument")?, "tanh")?.tanh()))
}
fn math_cbrt(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Float(as_f64(args.first().ok_or("cbrt() requires 1 argument")?, "cbrt")?.cbrt()))
}
fn math_log1p(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Float(as_f64(args.first().ok_or("log1p() requires 1 argument")?, "log1p")?.ln_1p()))
}
fn math_expm1(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Float(as_f64(args.first().ok_or("expm1() requires 1 argument")?, "expm1")?.exp_m1()))
}
fn math_abs(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("math.abs() requires 1 argument")? {
        Value::Int(n)   => Ok(Value::Int(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        v => Err(format!("math.abs(): expected numeric, got {}", v.type_name())),
    }
}

// ── math: random (LCG semplice — senza dipendenze esterne) ───────────────
// Stato condiviso tramite thread_local — sufficiente per uso single-thread
use std::cell::Cell;
thread_local! {
    static RNG_STATE: Cell<u64> = Cell::new(12345678901234567u64);
}
fn lcg_next() -> u64 {
    RNG_STATE.with(|s| {
        // Parametri da Knuth MMIX
        let v = s.get().wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        s.set(v); v
    })
}

fn math_random(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() { return Err("math.random() takes no arguments".into()); }
    let bits = lcg_next();
    // Mappa [0, 2^53) → [0.0, 1.0)
    Ok(Value::Float((bits >> 11) as f64 / (1u64 << 53) as f64))
}
fn math_randint(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Int(lo), Value::Int(hi)] => {
            if lo > hi { return Err(format!("randint({}, {}): lo > hi", lo, hi)); }
            let range = (*hi - *lo + 1) as u64;
            Ok(Value::Int(*lo + (lcg_next() % range) as i64))
        }
        _ => Err("math.randint(lo, hi) requires 2 Int arguments".into()),
    }
}
fn math_choice(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("math.choice() requires 1 argument")? {
        Value::Array(a) => {
            let arr = a.borrow();
            if arr.is_empty() { return Err("math.choice(): empty array".into()); }
            let idx = lcg_next() as usize % arr.len();
            Ok(arr[idx].clone())
        }
        v => Err(format!("math.choice(): expected Array, got {}", v.type_name())),
    }
}
fn math_shuffle(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("math.shuffle() requires 1 argument")? {
        Value::Array(a) => {
            let mut arr = a.borrow_mut();
            let n = arr.len();
            // Fisher-Yates
            for i in (1..n).rev() {
                let j = lcg_next() as usize % (i + 1);
                arr.swap(i, j);
            }
            Ok(Value::None)
        }
        v => Err(format!("math.shuffle(): expected Array, got {}", v.type_name())),
    }
}
fn math_seed(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("math.seed() requires 1 argument")? {
        Value::Int(n) => { RNG_STATE.with(|s| s.set(*n as u64)); Ok(Value::None) }
        v => Err(format!("math.seed(): expected Int, got {}", v.type_name())),
    }
}
fn math_sample(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Array(a), Value::Int(k)] => {
            let arr = a.borrow();
            let k = *k as usize;
            if k > arr.len() { return Err(format!("math.sample(): k={} > len={}", k, arr.len())); }
            let mut indices: Vec<usize> = (0..arr.len()).collect();
            // Partial Fisher-Yates — seleziona k elementi
            for i in 0..k {
                let j = i + lcg_next() as usize % (arr.len() - i);
                indices.swap(i, j);
            }
            let result: Vec<Value> = indices[..k].iter().map(|&i| arr[i].clone()).collect();
            Ok(Value::array(result))
        }
        _ => Err("math.sample(array, k) requires Array and Int".into()),
    }
}

// ── string extensions ─────────────────────────────────────────────────────

fn str_zfill(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Int(w)] => {
            let w = *w as usize;
            let s = s.as_str();
            if s.len() >= w { return Ok(Value::str(s)); }
            let pad = w - s.len();
            // Gestisce il segno: "-42".zfill(5) → "-0042"
            if s.starts_with('-') {
                Ok(Value::str(format!("-{}{}", "0".repeat(pad), &s[1..])))
            } else {
                Ok(Value::str(format!("{}{}", "0".repeat(pad), s)))
            }
        }
        _ => Err("string.zfill(s, width) requires Str and Int".into()),
    }
}
fn str_center(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Int(w)] => {
            let w = *w as usize; let s = s.as_str(); let len = s.chars().count();
            if len >= w { return Ok(Value::str(s)); }
            let total_pad = w - len;
            let left = total_pad / 2; let right = total_pad - left;
            Ok(Value::str(format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))))
        }
        [Value::Str(s), Value::Int(w), Value::Str(fill)] => {
            let w = *w as usize; let s = s.as_str(); let len = s.chars().count();
            let fc = fill.chars().next().unwrap_or(' ');
            if len >= w { return Ok(Value::str(s)); }
            let total_pad = w - len;
            let left = total_pad / 2; let right = total_pad - left;
            Ok(Value::str(format!("{}{}{}", fc.to_string().repeat(left), s, fc.to_string().repeat(right))))
        }
        _ => Err("string.center(s, width, fill?) requires Str and Int".into()),
    }
}
fn str_ljust(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Int(w)] => {
            let w = *w as usize; let s = s.as_str(); let len = s.chars().count();
            if len >= w { return Ok(Value::str(s)); }
            Ok(Value::str(format!("{}{}", s, " ".repeat(w - len))))
        }
        [Value::Str(s), Value::Int(w), Value::Str(fill)] => {
            let w = *w as usize; let s = s.as_str(); let len = s.chars().count();
            let fc = fill.chars().next().unwrap_or(' ');
            if len >= w { return Ok(Value::str(s)); }
            Ok(Value::str(format!("{}{}", s, fc.to_string().repeat(w - len))))
        }
        _ => Err("string.ljust(s, width, fill?) requires Str and Int".into()),
    }
}
fn str_rjust(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Int(w)] => {
            let w = *w as usize; let s = s.as_str(); let len = s.chars().count();
            if len >= w { return Ok(Value::str(s)); }
            Ok(Value::str(format!("{}{}", " ".repeat(w - len), s)))
        }
        [Value::Str(s), Value::Int(w), Value::Str(fill)] => {
            let w = *w as usize; let s = s.as_str(); let len = s.chars().count();
            let fc = fill.chars().next().unwrap_or(' ');
            if len >= w { return Ok(Value::str(s)); }
            Ok(Value::str(format!("{}{}", fc.to_string().repeat(w - len), s)))
        }
        _ => Err("string.rjust(s, width, fill?) requires Str and Int".into()),
    }
}
fn str_to_int(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s)] => s.trim().parse::<i64>()
            .map(Value::Int)
            .map_err(|_| format!("string.to_int(): cannot parse {:?} as Int", s.as_str())),
        [Value::Str(s), Value::Int(base)] => i64::from_str_radix(s.trim(), *base as u32)
            .map(Value::Int)
            .map_err(|_| format!("string.to_int(): cannot parse {:?} in base {}", s.as_str(), base)),
        _ => Err("string.to_int(s, base=10) requires Str".into()),
    }
}
fn str_to_float(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("string.to_float() requires 1 argument")? {
        Value::Str(s) => s.trim().parse::<f64>()
            .map(Value::Float)
            .map_err(|_| format!("string.to_float(): cannot parse {:?} as Float", s.as_str())),
        v => Err(format!("string.to_float(): expected Str, got {}", v.type_name())),
    }
}
fn str_is_digit(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("string.is_digit() requires 1 argument")? {
        Value::Str(s) => Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))),
        v => Err(format!("string.is_digit(): expected Str, got {}", v.type_name())),
    }
}
fn str_is_alpha(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("string.is_alpha() requires 1 argument")? {
        Value::Str(s) => Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| c.is_alphabetic()))),
        v => Err(format!("string.is_alpha(): expected Str, got {}", v.type_name())),
    }
}
fn str_is_alnum(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("string.is_alnum() requires 1 argument")? {
        Value::Str(s) => Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| c.is_alphanumeric()))),
        v => Err(format!("string.is_alnum(): expected Str, got {}", v.type_name())),
    }
}
fn str_is_upper(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("string.is_upper() requires 1 argument")? {
        Value::Str(s) => Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| !c.is_lowercase()))),
        v => Err(format!("string.is_upper(): expected Str, got {}", v.type_name())),
    }
}
fn str_is_lower(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("string.is_lower() requires 1 argument")? {
        Value::Str(s) => Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| !c.is_uppercase()))),
        v => Err(format!("string.is_lower(): expected Str, got {}", v.type_name())),
    }
}
fn str_capitalize(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("string.capitalize() requires 1 argument")? {
        Value::Str(s) => {
            let mut chars = s.chars();
            let result = match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
            };
            Ok(Value::str(result))
        }
        v => Err(format!("string.capitalize(): expected Str, got {}", v.type_name())),
    }
}
fn str_title(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("string.title() requires 1 argument")? {
        Value::Str(s) => {
            let result = s.split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            Ok(Value::str(result))
        }
        v => Err(format!("string.title(): expected Str, got {}", v.type_name())),
    }
}
fn str_slice(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s), Value::Int(start)] => {
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len() as i64;
            let i = if *start < 0 { (len + start).max(0) as usize } else { (*start as usize).min(chars.len()) };
            Ok(Value::str(chars[i..].iter().collect::<String>()))
        }
        [Value::Str(s), Value::Int(start), Value::Int(end)] => {
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len() as i64;
            let i = if *start < 0 { (len + start).max(0) as usize } else { (*start as usize).min(chars.len()) };
            let j = if *end < 0 { (len + end).max(0) as usize } else { (*end as usize).min(chars.len()) };
            Ok(Value::str(chars[i..j.max(i)].iter().collect::<String>()))
        }
        _ => Err("string.slice(s, start, end?) requires Str and Int".into()),
    }
}

// ── io.path extensions ────────────────────────────────────────────────────

fn io_path_join(args: &[Value]) -> Result<Value, String> {
    use std::path::PathBuf;
    let parts: Vec<&str> = args.iter().map(|v| match v {
        Value::Str(s) => Ok(s.as_str()),
        _ => Err(format!("io.path.join(): expected Str, got {}", v.type_name())),
    }).collect::<Result<Vec<_>, _>>()?;
    if parts.is_empty() { return Err("io.path.join() requires at least 1 argument".into()); }
    let mut path = PathBuf::from(parts[0]);
    for p in &parts[1..] { path.push(p); }
    Ok(Value::str(path.to_string_lossy().to_string()))
}
fn io_path_dirname(args: &[Value]) -> Result<Value, String> {
    use std::path::Path;
    match args.first().ok_or("io.path.dirname() requires 1 argument")? {
        Value::Str(s) => Ok(Value::str(
            Path::new(s.as_str()).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
        )),
        v => Err(format!("io.path.dirname(): expected Str, got {}", v.type_name())),
    }
}
fn io_path_basename(args: &[Value]) -> Result<Value, String> {
    use std::path::Path;
    match args.first().ok_or("io.path.basename() requires 1 argument")? {
        Value::Str(s) => Ok(Value::str(
            Path::new(s.as_str()).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
        )),
        v => Err(format!("io.path.basename(): expected Str, got {}", v.type_name())),
    }
}
fn io_path_stem(args: &[Value]) -> Result<Value, String> {
    use std::path::Path;
    match args.first().ok_or("io.path.stem() requires 1 argument")? {
        Value::Str(s) => Ok(Value::str(
            Path::new(s.as_str()).file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
        )),
        v => Err(format!("io.path.stem(): expected Str, got {}", v.type_name())),
    }
}
fn io_path_ext(args: &[Value]) -> Result<Value, String> {
    use std::path::Path;
    match args.first().ok_or("io.path.ext() requires 1 argument")? {
        Value::Str(s) => Ok(Value::str(
            Path::new(s.as_str()).extension().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
        )),
        v => Err(format!("io.path.ext(): expected Str, got {}", v.type_name())),
    }
}
fn io_path_exists(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("io.path.exists() requires 1 argument")? {
        Value::Str(s) => Ok(Value::Bool(std::path::Path::new(s.as_str()).exists())),
        v => Err(format!("io.path.exists(): expected Str, got {}", v.type_name())),
    }
}
fn io_path_isfile(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("io.path.isfile() requires 1 argument")? {
        Value::Str(s) => Ok(Value::Bool(std::path::Path::new(s.as_str()).is_file())),
        v => Err(format!("io.path.isfile(): expected Str, got {}", v.type_name())),
    }
}
fn io_path_isdir(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("io.path.isdir() requires 1 argument")? {
        Value::Str(s) => Ok(Value::Bool(std::path::Path::new(s.as_str()).is_dir())),
        v => Err(format!("io.path.isdir(): expected Str, got {}", v.type_name())),
    }
}
fn io_listdir(args: &[Value]) -> Result<Value, String> {
    let path = match args.first() {
        Some(Value::Str(s)) => s.as_str().to_string(),
        None => ".".to_string(),
        Some(v) => return Err(format!("io.listdir(): expected Str, got {}", v.type_name())),
    };
    let entries = std::fs::read_dir(&path)
        .map_err(|e| format!("io.listdir('{}'): {}", path, e))?
        .filter_map(|e| e.ok())
        .map(|e| Value::str(e.file_name().to_string_lossy().to_string()))
        .collect();
    Ok(Value::array(entries))
}
fn io_cwd(_args: &[Value]) -> Result<Value, String> {
    std::env::current_dir()
        .map(|p| Value::str(p.to_string_lossy().to_string()))
        .map_err(|e| format!("io.cwd(): {}", e))
}
fn io_mkdir(args: &[Value]) -> Result<Value, String> {
    match args.first().ok_or("io.mkdir() requires 1 argument")? {
        Value::Str(s) => std::fs::create_dir_all(s.as_str())
            .map(|_| Value::None)
            .map_err(|e| format!("io.mkdir('{}'): {}", s, e)),
        v => Err(format!("io.mkdir(): expected Str, got {}", v.type_name())),
    }
}

/// Modulo `random` standalone — alias delle funzioni math.random*
pub fn make_random_module() -> Value {
    fn entry(name: &str, f: fn(&[Value]) -> Result<Value, String>) -> (Value, Value) {
        (Value::str(name), Value::native_fn(name, f))
    }
    Value::dict(vec![
        entry("random",  math_random),
        entry("randint", math_randint),
        entry("choice",  math_choice),
        entry("shuffle", math_shuffle),
        entry("seed",    math_seed),
        entry("sample",  math_sample),
    ])
}

// ── NdArray stdlib (v0.2.25) ──────────────────────────────────────────────

use crate::value::NdArray as Nd;

fn parse_shape(v: &Value) -> Result<Vec<usize>, String> {
    match v {
        Value::Array(a) => {
            a.borrow().iter().map(|x| match x {
                Value::Int(n) if *n > 0 => Ok(*n as usize),
                _ => Err(format!("shape elements must be positive Int, got {}", x)),
            }).collect()
        }
        Value::Int(n) if *n > 0 => Ok(vec![*n as usize]),
        _ => Err(format!("shape must be Int or [Int, ...], got {}", v)),
    }
}

fn parse_dtype_opt(args: &[Value], idx: usize) -> Dtype {
    if let Some(Value::Str(s)) = args.get(idx) {
        match s.as_str() {
            "Float32" => Dtype::Float32,
            "Int64"   => Dtype::Int64,
            "Int32"   => Dtype::Int32,
            _         => Dtype::Float64,
        }
    } else { Dtype::Float64 }
}

// nd_zeros(shape, dtype?)
fn nd_zeros(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() { return Err("nd.zeros(shape, dtype?) requires shape".into()); }
    let shape = parse_shape(&args[0])?;
    let dtype = parse_dtype_opt(args, 1);
    Ok(Value::nd_array(Nd::zeros(shape, dtype)))
}

// nd_ones(shape, dtype?)
fn nd_ones(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() { return Err("nd.ones(shape, dtype?) requires shape".into()); }
    let shape = parse_shape(&args[0])?;
    let dtype = parse_dtype_opt(args, 1);
    let mut nd = Nd::zeros(shape, dtype);
    for i in 0..nd.size() { nd.data.set(i, Value::Float(1.0)).unwrap(); }
    Ok(Value::nd_array(nd))
}

// nd_full(shape, fill_value, dtype?)
fn nd_full(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 { return Err("nd.full(shape, value, dtype?) requires 2 args".into()); }
    let shape = parse_shape(&args[0])?;
    let dtype = parse_dtype_opt(args, 2);
    let fill = args[1].clone();
    let mut nd = Nd::zeros(shape, dtype);
    for i in 0..nd.size() { nd.data.set(i, fill.clone()).unwrap(); }
    Ok(Value::nd_array(nd))
}

// nd_eye(n, dtype?)
fn nd_eye(args: &[Value]) -> Result<Value, String> {
    let n = match args.first() {
        Some(Value::Int(n)) => *n as usize,
        // nd.eye([3]) — compatibile con nd.zeros([3,3])
        Some(Value::Array(a)) => match a.borrow().first() {
            Some(Value::Int(n)) => *n as usize,
            _ => return Err("nd.eye([n]) first element must be Int".into()),
        },
        _ => return Err("nd.eye(n) or nd.eye([n]) requires Int".into()),
    };
    let dtype = parse_dtype_opt(args, 1);
    let mut nd = Nd::zeros(vec![n, n], dtype);
    for i in 0..n { nd.data.set(i*n + i, Value::Float(1.0)).unwrap(); }
    Ok(Value::nd_array(nd))
}

// nd_arange(start, stop, step?, dtype?)
fn nd_arange(args: &[Value]) -> Result<Value, String> {
    let (start, stop, step) = match args {
        [Value::Float(s), Value::Float(e)] => (*s, *e, 1.0f64),
        [Value::Int(s), Value::Int(e)] => (*s as f64, *e as f64, 1.0),
        [Value::Float(s), Value::Float(e), Value::Float(st)] => (*s, *e, *st),
        [Value::Int(s), Value::Int(e), Value::Int(st)] => (*s as f64, *e as f64, *st as f64),
        [Value::Float(s), Value::Float(e), Value::Int(st)] => (*s, *e, *st as f64),
        [Value::Int(s), Value::Int(e), Value::Float(st)] => (*s as f64, *e as f64, *st),
        _ => return Err("nd.arange(start, stop[, step]) requires numeric args".into()),
    };
    if step == 0.0 { return Err("nd.arange: step cannot be zero".into()); }
    let n = ((stop - start) / step).ceil().max(0.0) as usize;
    let data: Vec<f64> = (0..n).map(|i| start + i as f64 * step).collect();
    Ok(Value::nd_array(Nd { shape: vec![n], data: TypedArrayData::Float64(data) }))
}

// nd_linspace(start, stop, num)
fn nd_linspace(args: &[Value]) -> Result<Value, String> {
    let (start, stop, n) = match args {
        [a, b, Value::Int(n)] => {
            let s = a.as_float().ok_or("linspace: start must be numeric")?;
            let e = b.as_float().ok_or("linspace: stop must be numeric")?;
            (s, e, *n as usize)
        }
        _ => return Err("nd.linspace(start, stop, n) requires 3 args".into()),
    };
    if n == 0 { return Ok(Value::nd_array(Nd { shape: vec![0], data: TypedArrayData::Float64(vec![]) })); }
    if n == 1 { return Ok(Value::nd_array(Nd { shape: vec![1], data: TypedArrayData::Float64(vec![start]) })); }
    let data: Vec<f64> = (0..n).map(|i| start + (stop - start) * i as f64 / (n-1) as f64).collect();
    Ok(Value::nd_array(Nd { shape: vec![n], data: TypedArrayData::Float64(data) }))
}

// nd_from_list(list) — converte Array annidata in NdArray 2D/3D
fn nd_from_list(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Array(outer)) => {
            let rows = outer.borrow();
            if rows.is_empty() { return Ok(Value::nd_array(Nd::zeros(vec![0], Dtype::Float64))); }
            // Guarda il primo elemento per determinare la struttura
            match &rows[0] {
                Value::Array(inner_0) => {
                    let cols = inner_0.borrow().len();
                    let m = rows.len();
                    let mut data = Vec::with_capacity(m * cols);
                    for row in rows.iter() {
                        match row {
                            Value::Array(r) => for v in r.borrow().iter() { data.push(v.as_float().unwrap_or(0.0)); },
                            _ => return Err("nd.array: all rows must be arrays".into()),
                        }
                    }
                    Ok(Value::nd_array(Nd { shape: vec![m, cols], data: TypedArrayData::Float64(data) }))
                }
                v if v.as_float().is_some() => {
                    let data: Vec<f64> = rows.iter().map(|v| v.as_float().unwrap_or(0.0)).collect();
                    Ok(Value::nd_array(Nd { shape: vec![rows.len()], data: TypedArrayData::Float64(data) }))
                }
                _ => Err("nd.array: unsupported structure".into()),
            }
        }
        _ => Err("nd.array(list) requires an Array".into()),
    }
}

fn require_nd<'a>(v: &'a Value, fn_name: &str) -> Result<std::cell::Ref<'a, Nd>, String> {
    match v {
        Value::NdArray(nd) => Ok(nd.borrow()),
        _ => Err(format!("{}: expected NdArray, got {}", fn_name, v.type_name())),
    }
}

// nd_shape(arr)
fn nd_shape(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.shape")?;
    Ok(Value::array(nd.shape.iter().map(|&s| Value::Int(s as i64)).collect()))
}

// nd_ndim(arr)
fn nd_ndim(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.ndim")?;
    Ok(Value::Int(nd.ndim() as i64))
}

// nd_size(arr)
fn nd_size(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.size")?;
    Ok(Value::Int(nd.size() as i64))
}

// nd_reshape(arr, shape)
fn nd_reshape(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 { return Err("nd.reshape(arr, shape)".into()); }
    let nd = require_nd(&args[0], "nd.reshape")?;
    let new_shape = parse_shape(&args[1])?;
    Ok(Value::nd_array(nd.reshape(new_shape)?))
}

// nd_transpose(arr, axes?)
fn nd_transpose(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() { return Err("nd.transpose(arr)".into()); }
    let nd = require_nd(&args[0], "nd.transpose")?;
    let axes = if args.len() > 1 {
        if let Some(Value::Array(a)) = args.get(1) {
            Some(a.borrow().iter().map(|v| match v { Value::Int(n) => Ok(*n as usize), _ => Err("axes must be Int".to_string()) }).collect::<Result<Vec<_>,_>>()?)
        } else { None }
    } else { None };
    Ok(Value::nd_array(nd.transpose_axes(axes)?))
}

// nd_matmul(a, b)
fn nd_matmul(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 { return Err("nd.matmul(a, b)".into()); }
    let a = require_nd(&args[0], "nd.matmul")?;
    let b = require_nd(&args[1], "nd.matmul")?;
    Ok(Value::nd_array(a.matmul(&b)?))
}

// nd_sum(arr, axis?)
fn nd_sum(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() { return Err("nd.sum(arr, axis?)".into()); }
    let nd = require_nd(&args[0], "nd.sum")?;
    match args.get(1) {
        Some(Value::Int(axis)) => Ok(Value::nd_array(nd.sum_axis(*axis as usize)?)),
        _ => Ok(Value::Float(nd.sum_all())),
    }
}

// nd_mean(arr, axis?)
fn nd_mean(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() { return Err("nd.mean(arr, axis?)".into()); }
    let nd = require_nd(&args[0], "nd.mean")?;
    match args.get(1) {
        Some(Value::Int(axis)) => {
            let s = nd.sum_axis(*axis as usize)?;
            let div = nd.shape[*axis as usize] as f64;
            Ok(Value::nd_array(s.ewise_scalar(div, |a, b| a / b)))
        }
        _ => Ok(Value::Float(nd.mean_all())),
    }
}

// nd_min/max/argmin/argmax
fn nd_min(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.min")?;
    Ok(Value::Float(nd.min_all()))
}
fn nd_max(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.max")?;
    Ok(Value::Float(nd.max_all()))
}
fn nd_argmin(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.argmin")?;
    Ok(Value::Int(nd.argmin() as i64))
}
fn nd_argmax(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.argmax")?;
    Ok(Value::Int(nd.argmax() as i64))
}

// nd_std(arr, ddof?)
fn nd_std(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.std")?;
    let ddof = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => 0 };
    Ok(Value::Float(nd.std_dev(ddof)))
}

// nd_var(arr, ddof?)
fn nd_var(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.var")?;
    let ddof = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => 0 };
    let s = nd.std_dev(ddof);
    Ok(Value::Float(s * s))
}

// nd_cumsum(arr)
fn nd_cumsum(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.cumsum")?;
    Ok(Value::nd_array(nd.cumsum()))
}

// nd_flatten(arr)
fn nd_flatten(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.flatten")?;
    Ok(Value::nd_array(nd.flatten()))
}

// nd_clip(arr, lo, hi)
fn nd_clip(args: &[Value]) -> Result<Value, String> {
    if args.len() < 3 { return Err("nd.clip(arr, lo, hi)".into()); }
    let nd = require_nd(&args[0], "nd.clip")?;
    let lo = args[1].as_float().ok_or("nd.clip: lo must be numeric")?;
    let hi = args[2].as_float().ok_or("nd.clip: hi must be numeric")?;
    Ok(Value::nd_array(nd.clip(lo, hi)))
}

// nd_where(cond, a, b)
fn nd_where(args: &[Value]) -> Result<Value, String> {
    if args.len() < 3 { return Err("nd.where(cond, a, b)".into()); }
    let cond = require_nd(&args[0], "nd.where")?;
    let a    = require_nd(&args[1], "nd.where")?;
    let b    = require_nd(&args[2], "nd.where")?;
    Ok(Value::nd_array(Nd::where_cond(&cond, &a, &b)?))
}

// nd_abs(arr)
fn nd_abs_fn(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.abs")?;
    Ok(Value::nd_array(nd.ewise_unary(f64::abs)))
}

// nd_sqrt(arr)
fn nd_sqrt_fn(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.sqrt")?;
    Ok(Value::nd_array(nd.ewise_unary(f64::sqrt)))
}

// nd_exp(arr)
fn nd_exp_fn(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.exp")?;
    Ok(Value::nd_array(nd.ewise_unary(f64::exp)))
}

// nd_log(arr)
fn nd_log_fn(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.log")?;
    Ok(Value::nd_array(nd.ewise_unary(f64::ln)))
}

// nd_add/sub/mul/div(a, b)
fn nd_add(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 { return Err("nd.add(a, b)".into()); }
    let a = require_nd(&args[0], "nd.add")?;
    let b = require_nd(&args[1], "nd.add")?;
    Ok(Value::nd_array(a.ewise_op(&b, |x, y| x + y)?))
}
fn nd_sub(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 { return Err("nd.sub(a, b)".into()); }
    let a = require_nd(&args[0], "nd.sub")?;
    let b = require_nd(&args[1], "nd.sub")?;
    Ok(Value::nd_array(a.ewise_op(&b, |x, y| x - y)?))
}
fn nd_mul(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 { return Err("nd.mul(a, b)".into()); }
    let a = require_nd(&args[0], "nd.mul")?;
    let b = require_nd(&args[1], "nd.mul")?;
    Ok(Value::nd_array(a.ewise_op(&b, |x, y| x * y)?))
}
fn nd_div(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 { return Err("nd.div(a, b)".into()); }
    let a = require_nd(&args[0], "nd.div")?;
    let b = require_nd(&args[1], "nd.div")?;
    Ok(Value::nd_array(a.ewise_op(&b, |x, y| x / y)?))
}

// nd_dot(a, b) — matmul per 2D, dot product per 1D
fn nd_dot(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 { return Err("nd.dot(a, b)".into()); }
    let a = require_nd(&args[0], "nd.dot")?;
    let b = require_nd(&args[1], "nd.dot")?;
    if a.ndim() == 1 && b.ndim() == 1 {
        if a.size() != b.size() { return Err(format!("nd.dot: size mismatch {} vs {}", a.size(), b.size())); }
        let s: f64 = (0..a.size()).map(|i| a.data.get(i).unwrap().as_float().unwrap_or(0.0) * b.data.get(i).unwrap().as_float().unwrap_or(0.0)).sum();
        return Ok(Value::Float(s));
    }
    Ok(Value::nd_array(a.matmul(&b)?))
}

// nd_hstack(list_of_arrays) — concatena orizzontalmente (colonne)
fn nd_hstack(args: &[Value]) -> Result<Value, String> {
    let arrays = match args.first() {
        Some(Value::Array(a)) => a.borrow().clone(),
        _ => return Err("nd.hstack([a, b, ...])".into()),
    };
    if arrays.is_empty() { return Err("nd.hstack: empty list".into()); }
    let nds: Vec<_> = arrays.iter().map(|v| match v {
        Value::NdArray(nd) => Ok(nd.borrow().clone()),
        _ => Err(format!("nd.hstack: expected NdArray, got {}", v.type_name())),
    }).collect::<Result<_,_>>()?;
    let rows = nds[0].shape.get(0).copied().unwrap_or(1);
    let total_cols: usize = nds.iter().map(|nd| nd.shape.get(1).copied().unwrap_or(nd.size())).sum();
    let mut out = Nd::zeros(vec![rows, total_cols], Dtype::Float64);
    let mut col_offset = 0;
    for nd in &nds {
        let cols = nd.shape.get(1).copied().unwrap_or(nd.size());
        for r in 0..rows {
            for c in 0..cols {
                let v = nd.data.get(r * cols + c).unwrap();
                out.data.set(r * total_cols + col_offset + c, v).unwrap();
            }
        }
        col_offset += cols;
    }
    Ok(Value::nd_array(out))
}

// nd_vstack(list_of_arrays) — concatena verticalmente (righe)
fn nd_vstack(args: &[Value]) -> Result<Value, String> {
    let arrays = match args.first() {
        Some(Value::Array(a)) => a.borrow().clone(),
        _ => return Err("nd.vstack([a, b, ...])".into()),
    };
    if arrays.is_empty() { return Err("nd.vstack: empty list".into()); }
    let nds: Vec<_> = arrays.iter().map(|v| match v {
        Value::NdArray(nd) => Ok(nd.borrow().clone()),
        _ => Err(format!("nd.vstack: expected NdArray, got {}", v.type_name())),
    }).collect::<Result<_,_>>()?;
    let cols = nds[0].shape.get(1).copied().unwrap_or(nds[0].size());
    let total_rows: usize = nds.iter().map(|nd| nd.shape.get(0).copied().unwrap_or(1)).sum();
    let mut out_data: Vec<f64> = Vec::with_capacity(total_rows * cols);
    for nd in &nds {
        for i in 0..nd.size() { out_data.push(nd.data.get(i).unwrap().as_float().unwrap_or(0.0)); }
    }
    Ok(Value::nd_array(Nd { shape: vec![total_rows, cols], data: TypedArrayData::Float64(out_data) }))
}

// nd_diag(v) — crea matrice diagonale da 1D, oppure estrae diagonale da 2D
fn nd_diag(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.diag")?;
    if nd.ndim() == 1 {
        let n = nd.size();
        let mut out = Nd::zeros(vec![n, n], Dtype::Float64);
        for i in 0..n { out.data.set(i*n+i, nd.data.get(i).unwrap()).unwrap(); }
        Ok(Value::nd_array(out))
    } else if nd.ndim() == 2 {
        let (rows, cols) = (nd.shape[0], nd.shape[1]);
        let n = rows.min(cols);
        let mut out = Nd::zeros(vec![n], Dtype::Float64);
        for i in 0..n { out.data.set(i, nd.data.get(i*cols+i).unwrap()).unwrap(); }
        Ok(Value::nd_array(out))
    } else {
        Err("nd.diag: requires 1D or 2D array".into())
    }
}

// nd_norm(arr, ord?) — norma L2 di default
fn nd_norm(args: &[Value]) -> Result<Value, String> {
    let nd = require_nd(args.first().unwrap_or(&Value::None), "nd.norm")?;
    let ord = match args.get(1) { Some(Value::Int(n)) => *n, _ => 2 };
    match ord {
        1 => Ok(Value::Float(nd.ewise_unary(f64::abs).sum_all())),
        2 => {
            let ss: f64 = (0..nd.size()).map(|i| { let x = nd.data.get(i).unwrap().as_float().unwrap_or(0.0); x*x }).sum();
            Ok(Value::Float(ss.sqrt()))
        }
        _ => Err(format!("nd.norm: ord={} not supported (use 1 or 2)", ord)),
    }
}

// nd_astype(arr, dtype_str)
fn nd_astype(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 { return Err("nd.astype(arr, dtype)".into()); }
    let nd_src = require_nd(&args[0], "nd.astype")?;
    let dtype = match &args[1] {
        Value::Str(s) => match s.as_str() {
            "Float32" => Dtype::Float32,
            "Int64"   => Dtype::Int64,
            "Int32"   => Dtype::Int32,
            _         => Dtype::Float64,
        },
        _ => return Err("nd.astype: dtype must be a string".into()),
    };
    let mut out = Nd::zeros(nd_src.shape.clone(), dtype);
    for i in 0..nd_src.size() { out.data.set(i, nd_src.data.get(i).unwrap()).unwrap(); }
    Ok(Value::nd_array(out))
}

// nd_get(arr, i, j, ...) — accesso multidimensionale
fn nd_get(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() { return Err("nd.get(arr, indices) or nd.get(arr, i, j, ...)".into()); }
    let nd = require_nd(&args[0], "nd.get")?;
    // Accetta sia nd.get(arr, [i,j]) che nd.get(arr, i, j)
    let indices: Vec<usize> = if args.len() == 2 {
        if let Value::Array(a) = &args[1] {
            a.borrow().iter().map(|v| match v {
                Value::Int(n) => Ok(*n as usize),
                _ => Err(format!("nd.get: indices must be Int, got {}", v.type_name())),
            }).collect::<Result<_,_>>()?
        } else {
            match &args[1] {
                Value::Int(n) => vec![*n as usize],
                _ => return Err(format!("nd.get: index must be Int or Array, got {}", args[1].type_name())),
            }
        }
    } else {
        args[1..].iter().map(|v| match v {
            Value::Int(n) => Ok(*n as usize),
            _ => Err(format!("nd.get: indices must be Int, got {}", v.type_name())),
        }).collect::<Result<_,_>>()?
    };
    nd.get_nd(&indices)
}

// nd_set(arr, [i,j,...], value) or nd_set(arr, i, j, ..., value) — scrittura multidimensionale
fn nd_set(args: &[Value]) -> Result<Value, String> {
    if args.len() < 3 { return Err("nd.set(arr, [i,j], value) or nd.set(arr, i, j, ..., value)".into()); }
    match &args[0] {
        Value::NdArray(rc) => {
            let mut nd = rc.borrow_mut();
            let value = args.last().unwrap().clone();
            // Accetta sia nd.set(arr, [i,j], val) che nd.set(arr, i, j, val)
            let indices: Vec<usize> = if args.len() == 3 {
                if let Value::Array(a) = &args[1] {
                    a.borrow().iter().map(|v| match v {
                        Value::Int(n) => Ok(*n as usize),
                        _ => Err(format!("nd.set: indices must be Int")),
                    }).collect::<Result<_,_>>()?
                } else {
                    match &args[1] {
                        Value::Int(n) => vec![*n as usize],
                        _ => return Err("nd.set: index must be Int or Array".into()),
                    }
                }
            } else {
                args[1..args.len()-1].iter().map(|v| match v {
                    Value::Int(n) => Ok(*n as usize),
                    _ => Err(format!("nd.set: indices must be Int")),
                }).collect::<Result<_,_>>()?
            };
            nd.set_nd(&indices, value)?;
            Ok(Value::None)
        }
        _ => Err("nd.set: first arg must be NdArray".into()),
    }
}

/// Registra il modulo `nd` come Dict di funzioni NdArray.
pub fn register_nd_module(globals: &mut rustc_hash::FxHashMap<String, (Value, bool)>) {
    use indexmap::IndexMap;
    fn entry(name: &str, f: fn(&[Value]) -> Result<Value, String>) -> (Value, Value) {
        (Value::str(name), Value::native_fn(name, f))
    }
    let mut map: IndexMap<Value, Value> = IndexMap::new();
    for (k, v) in [
        entry("zeros",     nd_zeros),
        entry("ones",      nd_ones),
        entry("full",      nd_full),
        entry("eye",       nd_eye),
        entry("arange",    nd_arange),
        entry("linspace",  nd_linspace),
        entry("array",     nd_from_list),
        entry("shape",     nd_shape),
        entry("ndim",      nd_ndim),
        entry("size",      nd_size),
        entry("reshape",   nd_reshape),
        entry("transpose", nd_transpose),
        entry("T",         nd_transpose),
        entry("matmul",    nd_matmul),
        entry("dot",       nd_dot),
        entry("sum",       nd_sum),
        entry("mean",      nd_mean),
        entry("min",       nd_min),
        entry("max",       nd_max),
        entry("argmin",    nd_argmin),
        entry("argmax",    nd_argmax),
        entry("std",       nd_std),
        entry("var",       nd_var),
        entry("cumsum",    nd_cumsum),
        entry("flatten",   nd_flatten),
        entry("clip",      nd_clip),
        entry("where_",    nd_where),
        entry("abs",       nd_abs_fn),
        entry("sqrt",      nd_sqrt_fn),
        entry("exp",       nd_exp_fn),
        entry("log",       nd_log_fn),
        entry("add",       nd_add),
        entry("sub",       nd_sub),
        entry("mul",       nd_mul),
        entry("div",       nd_div),
        entry("hstack",    nd_hstack),
        entry("vstack",    nd_vstack),
        entry("diag",      nd_diag),
        entry("norm",      nd_norm),
        entry("astype",    nd_astype),
        entry("get",       nd_get),
        entry("set",       nd_set),
    ] { map.insert(k, v); }
    globals.insert("nd".into(), (Value::dict_from_map(map), true));
}
