use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::chunk::{read_i16, read_u16, Chunk, FnProto};
use crate::compiler::ClassInfo;
use crate::error::{VmError, VmResult};
use crate::opcode::Op;
use crate::stdlib;
use crate::value::{Closure, Instance, Upvalue, Value, TypedArrayData};

const STACK_MAX:  usize = 4096;
const FRAMES_MAX: usize = 256;

// ── Aritmetica element-wise per TypedArray (v0.2.6/v0.2.7) ───────────────

fn typed_binop(
    l: &Value,
    r: &Value,
    op_f: impl Fn(f64, f64) -> f64,
    op_i: impl Fn(i64, i64) -> i64,
) -> VmResult {
    let (ta_val, scalar_val, ta_is_left) = match (l, r) {
        (Value::TypedArray(_), Value::TypedArray(_)) => {
            return typed_binop_arrays(l, r, op_f, op_i);
        }
        (Value::TypedArray(_), scalar) => (l, scalar, true),
        (scalar, Value::TypedArray(_)) => (r, scalar, false),
        _ => unreachable!(),
    };

    let ta = if let Value::TypedArray(t) = ta_val { t.borrow() } else { unreachable!() };

    match &*ta {
        TypedArrayData::Float64(v) => {
            let s = scalar_val.as_float()
                .ok_or_else(|| VmError::TypeError("TypedArray op: scalar must be numeric".into()))?;
            let out: Vec<f64> = if ta_is_left { v.iter().map(|&x| op_f(x, s)).collect() }
                                else          { v.iter().map(|&x| op_f(s, x)).collect() };
            Ok(Value::typed_array(TypedArrayData::Float64(out)))
        }
        TypedArrayData::Float32(v) => {
            let s = scalar_val.as_float()
                .ok_or_else(|| VmError::TypeError("TypedArray op: scalar must be numeric".into()))? as f32;
            let out: Vec<f32> = if ta_is_left { v.iter().map(|&x| op_f(x as f64, s as f64) as f32).collect() }
                                else          { v.iter().map(|&x| op_f(s as f64, x as f64) as f32).collect() };
            Ok(Value::typed_array(TypedArrayData::Float32(out)))
        }
        TypedArrayData::Int64(v) => {
            if let Some(s) = scalar_val.as_float() {
                let out: Vec<f64> = if ta_is_left { v.iter().map(|&x| op_f(x as f64, s)).collect() }
                                    else          { v.iter().map(|&x| op_f(s, x as f64)).collect() };
                return Ok(Value::typed_array(TypedArrayData::Float64(out)));
            }
            let s = match scalar_val { Value::Int(n) => *n, _ => return Err(VmError::TypeError("Int64Array op: scalar must be Int or Float".into())) };
            let out: Vec<i64> = if ta_is_left { v.iter().map(|&x| op_i(x, s)).collect() }
                                else          { v.iter().map(|&x| op_i(s, x)).collect() };
            Ok(Value::typed_array(TypedArrayData::Int64(out)))
        }
        TypedArrayData::Int32(v) => {
            if let Some(s) = scalar_val.as_float() {
                let out: Vec<f64> = if ta_is_left { v.iter().map(|&x| op_f(x as f64, s)).collect() }
                                    else          { v.iter().map(|&x| op_f(s, x as f64)).collect() };
                return Ok(Value::typed_array(TypedArrayData::Float64(out)));
            }
            let s = match scalar_val { Value::Int(n) => *n as i32, _ => return Err(VmError::TypeError("Int32Array op: scalar must be Int or Float".into())) };
            let out: Vec<i32> = if ta_is_left { v.iter().map(|&x| op_i(x as i64, s as i64) as i32).collect() }
                                else          { v.iter().map(|&x| op_i(s as i64, x as i64) as i32).collect() };
            Ok(Value::typed_array(TypedArrayData::Int32(out)))
        }
    }
}

fn typed_binop_arrays(
    l: &Value,
    r: &Value,
    op_f: impl Fn(f64, f64) -> f64,
    op_i: impl Fn(i64, i64) -> i64,
) -> VmResult {
    let (tl, tr) = match (l, r) {
        (Value::TypedArray(a), Value::TypedArray(b)) => (a.borrow(), b.borrow()),
        _ => unreachable!(),
    };
    if tl.len() != tr.len() {
        return Err(VmError::TypeError(format!("TypedArray size mismatch: {} vs {}", tl.len(), tr.len())));
    }
    let len = tl.len();
    match (&*tl, &*tr) {
        (TypedArrayData::Float64(a), TypedArrayData::Float64(b)) => {
            Ok(Value::typed_array(TypedArrayData::Float64(a.iter().zip(b.iter()).map(|(&x,&y)| op_f(x,y)).collect())))
        }
        (TypedArrayData::Int64(a), TypedArrayData::Int64(b)) => {
            Ok(Value::typed_array(TypedArrayData::Int64(a.iter().zip(b.iter()).map(|(&x,&y)| op_i(x,y)).collect())))
        }
        (TypedArrayData::Int32(a), TypedArrayData::Int32(b)) => {
            Ok(Value::typed_array(TypedArrayData::Int32(a.iter().zip(b.iter()).map(|(&x,&y)| op_i(x as i64,y as i64) as i32).collect())))
        }
        (TypedArrayData::Float32(a), TypedArrayData::Float32(b)) => {
            Ok(Value::typed_array(TypedArrayData::Float32(a.iter().zip(b.iter()).map(|(&x,&y)| op_f(x as f64,y as f64) as f32).collect())))
        }
        _ => {
            let a: Vec<f64> = (0..len).map(|i| tl.get(i).unwrap().as_float().unwrap()).collect();
            let b: Vec<f64> = (0..len).map(|i| tr.get(i).unwrap().as_float().unwrap()).collect();
            Ok(Value::typed_array(TypedArrayData::Float64(a.iter().zip(b.iter()).map(|(&x,&y)| op_f(x,y)).collect())))
        }
    }
}

// ── Call frame ────────────────────────────────────────────────────────────

struct CallFrame {
    chunk:    Rc<Chunk>,
    ip:       usize,
    base:     usize,
    name:     String,
    upvalues: Vec<Upvalue>,
}

// ── VM ────────────────────────────────────────────────────────────────────

pub struct Vm {
    stack:          Vec<Value>,
    frames:         Vec<CallFrame>,
    globals:        HashMap<String, (Value, bool)>,
    class_registry: HashMap<String, ClassInfo>,
    step_limit:     u64,
}

impl Vm {
    pub fn new() -> Self {
        let mut vm = Vm {
            stack:          Vec::with_capacity(256),
            frames:         Vec::with_capacity(32),
            globals:        HashMap::new(),
            class_registry: HashMap::new(),
            step_limit:     0,
        };
        stdlib::register_globals(&mut vm.globals);
        vm
    }

    pub fn set_step_limit(&mut self, limit: u64) { self.step_limit = limit; }

    // ── run_chunk — dispatch loop monolitico (v0.2.14) ────────────────────
    //
    // Eliminati rispetto a v0.2.13:
    //   • fn step(): rimosso — ogni istruzione non è più una Rust function call
    //   • Rc::clone(&frame.chunk): rimosso — uso raw pointer *const Chunk
    //   • self.frames.last_mut().unwrap().ip = ip in ogni macro: rimosso —
    //     ip è una variabile locale, scritta nel frame solo su call/return
    //
    // SAFETY: chunk_ptr punta a Rc<Chunk> owned da self.frames.last().chunk.
    //   Il Rc non viene droppato mentre il frame esiste. I contenuti di Chunk
    //   non vengono mai mutati durante l'esecuzione. chunk_ptr viene ricaricato
    //   (load_frame!) dopo ogni cambio di frame.

    pub fn run_chunk(&mut self, chunk: Chunk) -> VmResult<Value> {
        let rc_chunk = Rc::new(chunk);
        self.frames.push(CallFrame {
            chunk: rc_chunk, ip: 0, base: 0,
            name: "<script>".to_string(), upvalues: Vec::new(),
        });

        let mut ip:   usize = 0;
        let mut base: usize = 0;
        // SAFETY: vedi commento sopra.
        let mut chunk_ptr: *const Chunk = Rc::as_ptr(&self.frames.last().unwrap().chunk);

        macro_rules! chunk     { () => { unsafe { &*chunk_ptr } } }
        macro_rules! read_u8   { () => {{ let v = chunk!().code[ip]; ip += 1; v }} }
        macro_rules! read_u16  { () => {{ let v = crate::chunk::read_u16(&chunk!().code, ip); ip += 2; v }} }
        macro_rules! read_i16  { () => {{ let v = crate::chunk::read_i16(&chunk!().code, ip); ip += 2; v }} }
        macro_rules! pop   { () => { self.stack.pop().ok_or_else(|| VmError::Generic("stack underflow".into()))? } }
        macro_rules! peek  { () => { self.stack.last().ok_or_else(|| VmError::Generic("stack empty".into()))?.clone() } }
        macro_rules! push  { ($v:expr) => { self.stack.push($v) } }
        macro_rules! save_ip   { () => { self.frames.last_mut().unwrap().ip = ip; } }
        macro_rules! load_frame { () => {{
            let f = self.frames.last().unwrap();
            chunk_ptr = Rc::as_ptr(&f.chunk);  // SAFETY: frame appena pushato/attivo
            ip   = f.ip;
            base = f.base;
        }} }

        let mut step_count: u64 = 0;

        'dispatch: loop {
            if self.step_limit > 0 {
                step_count += 1;
                if step_count > self.step_limit {
                    let trace = self.build_trace();
                    eprintln!("{}", trace);
                    self.frames.clear(); self.stack.clear();
                    return Err(VmError::Generic(format!("step limit {} exceeded (infinite loop?)", self.step_limit)));
                }
            }

            let op_byte = chunk!().code[ip];
            let op = Op::from_u8(op_byte)
                .ok_or_else(|| VmError::Generic(format!("bad opcode {}", op_byte)))?;
            ip += 1;

            match op {
                Op::Const  => { let idx = read_u16!() as usize; let v = chunk!().constants[idx].clone(); push!(v); }
                Op::True   => push!(Value::Bool(true)),
                Op::False  => push!(Value::Bool(false)),
                Op::Nil    => push!(Value::None),

                Op::Pop  => { pop!(); }
                Op::Dup  => { let v = peek!(); push!(v); }
                Op::Swap => { let t = pop!(); let u = pop!(); push!(t); push!(u); }
                Op::PopN => { let n = read_u8!() as usize; let l = self.stack.len(); self.stack.truncate(l - n); }

                Op::LoadLocal  => { let idx = read_u8!() as usize; let v = self.stack[base + idx].clone(); push!(v); }
                Op::StoreLocal => { let idx = read_u8!() as usize; let v = pop!(); self.stack[base + idx] = v; }

                Op::LoadUpval  => { let idx = read_u8!() as usize; let v = self.frames.last().unwrap().upvalues[idx].value.borrow().clone(); push!(v); }
                Op::StoreUpval => { let idx = read_u8!() as usize; let v = pop!(); *self.frames.last_mut().unwrap().upvalues[idx].value.borrow_mut() = v; }

                Op::LoadGlobal => {
                    let idx = read_u16!() as usize;
                    let name = &chunk!().names[idx];
                    let v = self.globals.get(name).map(|(v,_)| v.clone())
                        .ok_or_else(|| VmError::UndefinedVariable(name.clone()))?;
                    push!(v);
                }
                Op::StoreGlobal => {
                    let idx = read_u16!() as usize;
                    let name = chunk!().names[idx].clone();
                    let v = pop!();
                    match self.globals.get_mut(&name) {
                        Some((cur, true))  => *cur = v,
                        Some((_, false))   => return Err(VmError::AssignImmutable(name)),
                        None               => return Err(VmError::UndefinedVariable(name)),
                    }
                }
                Op::DefGlobal => {
                    let idx = read_u16!() as usize; let mutable = read_u8!() != 0;
                    let name = chunk!().names[idx].clone(); let v = pop!();
                    self.globals.insert(name, (v, mutable));
                }

                Op::Add    => { let r = pop!(); let l = pop!(); push!(self.op_add(l, r)?); }
                Op::Sub    => { let r = pop!(); let l = pop!(); push!(self.op_sub(l, r)?); }
                Op::Mul    => { let r = pop!(); let l = pop!(); push!(self.op_mul(l, r)?); }
                Op::Div    => { let r = pop!(); let l = pop!(); push!(self.op_div(l, r)?); }
                Op::IntDiv => { let r = pop!(); let l = pop!(); push!(self.op_intdiv(l, r)?); }
                Op::Mod    => { let r = pop!(); let l = pop!(); push!(self.op_mod(l, r)?); }
                Op::Pow    => { let r = pop!(); let l = pop!(); push!(self.op_pow(l, r)?); }
                Op::Neg    => {
                    let v = pop!();
                    push!(match v {
                        Value::Int(n)   => Value::Int(-n),
                        Value::Float(f) => Value::Float(-f),
                        _ => return Err(VmError::TypeError(format!("unary '-' on {}", v.type_name()))),
                    });
                }

                Op::BitAnd => { let r = pop!(); let l = pop!(); push!(self.op_bit(l, r, "&")?); }
                Op::BitOr  => { let r = pop!(); let l = pop!(); push!(self.op_bit(l, r, "|")?); }
                Op::BitXor => { let r = pop!(); let l = pop!(); push!(self.op_bit(l, r, "^")?); }
                Op::Shl    => { let r = pop!(); let l = pop!(); push!(self.op_bit(l, r, "<<")?); }
                Op::Shr    => { let r = pop!(); let l = pop!(); push!(self.op_bit(l, r, ">>")?); }
                Op::BitNot => {
                    let v = pop!();
                    push!(match v {
                        Value::Int(n) => Value::Int(!n),
                        _ => return Err(VmError::TypeError(format!("'~' on {}", v.type_name()))),
                    });
                }

                Op::Eq => { let r = pop!(); let l = pop!(); push!(Value::Bool(l == r)); }
                Op::Ne => { let r = pop!(); let l = pop!(); push!(Value::Bool(l != r)); }
                Op::Lt => { let r = pop!(); let l = pop!(); push!(Value::Bool(l <  r)); }
                Op::Le => { let r = pop!(); let l = pop!(); push!(Value::Bool(l <= r)); }
                Op::Gt => { let r = pop!(); let l = pop!(); push!(Value::Bool(l >  r)); }
                Op::Ge => { let r = pop!(); let l = pop!(); push!(Value::Bool(l >= r)); }

                Op::Not => { let v = pop!(); push!(Value::Bool(!v.is_truthy())); }

                // Salti: modificano solo il registro locale `ip`, zero write su self.frames
                Op::Jump          => { let o = read_i16!(); ip = (ip as isize + o as isize) as usize; }
                Op::JumpFalse     => { let o = read_i16!(); let v = pop!(); if !v.is_truthy() { ip = (ip as isize + o as isize) as usize; } }
                Op::JumpTrue      => { let o = read_i16!(); let v = pop!(); if  v.is_truthy() { ip = (ip as isize + o as isize) as usize; } }
                Op::JumpFalsePeek => { let o = read_i16!(); if !peek!().is_truthy() { ip = (ip as isize + o as isize) as usize; } }
                Op::JumpTruePeek  => { let o = read_i16!(); if  peek!().is_truthy() { ip = (ip as isize + o as isize) as usize; } }

                Op::MakeClosure => {
                    let idx = read_u16!() as usize; let n_up = read_u8!() as usize;
                    let proto = Rc::new(chunk!().fn_protos[idx].clone());
                    let mut upvalues = Vec::with_capacity(n_up);
                    if n_up > 0 {
                        let start = self.stack.len() - n_up;
                        for v in self.stack.drain(start..) {
                            upvalues.push(Upvalue { value: Rc::new(RefCell::new(v)) });
                        }
                    }
                    push!(Value::Closure(Rc::new(Closure { proto, upvalues })));
                }

                Op::Call => {
                    let argc = read_u8!() as usize;
                    if self.frames.len() >= FRAMES_MAX { return Err(VmError::StackOverflow); }
                    let fn_idx = self.stack.len() - argc - 1;
                    let callee = self.stack[fn_idx].clone();

                    // HOF: map / filter / reduce
                    if let Value::NativeFn(ref name, _) = callee {
                        match name.as_str() {
                            "map" => {
                                if argc != 2 { return Err(VmError::Generic("map(array, fn) requires 2 arguments".into())); }
                                let args: Vec<Value> = self.stack.drain(fn_idx..).skip(1).collect();
                                let (arr, cb) = (args[0].clone(), args[1].clone());
                                let items: Vec<Value> = match &arr {
                                    Value::Array(a) => a.borrow().clone(),
                                    Value::IntRange(s, e, inc) => {
                                        let (s, e, inc) = (*s, *e, *inc);
                                        if inc { (s..=e).map(Value::Int).collect() }
                                        else   { (s..e).map(Value::Int).collect() }
                                    }
                                    _ => return Err(VmError::TypeError(format!("map: first argument must be Array or Range, got {}", arr.type_name()))),
                                };
                                let mut result = Vec::with_capacity(items.len());
                                save_ip!();
                                for item in items { result.push(self.call_value_sync(cb.clone(), vec![item])?); }
                                push!(Value::array(result));
                                continue 'dispatch;
                            }
                            "filter" => {
                                if argc != 2 { return Err(VmError::Generic("filter(array, fn) requires 2 arguments".into())); }
                                let args: Vec<Value> = self.stack.drain(fn_idx..).skip(1).collect();
                                let (arr, cb) = (args[0].clone(), args[1].clone());
                                let items: Vec<Value> = match &arr {
                                    Value::Array(a) => a.borrow().clone(),
                                    Value::IntRange(s, e, inc) => {
                                        let (s, e, inc) = (*s, *e, *inc);
                                        if inc { (s..=e).map(Value::Int).collect() }
                                        else   { (s..e).map(Value::Int).collect() }
                                    }
                                    _ => return Err(VmError::TypeError(format!("filter: first argument must be Array or Range, got {}", arr.type_name()))),
                                };
                                let mut result = Vec::new();
                                save_ip!();
                                for item in items { let k = self.call_value_sync(cb.clone(), vec![item.clone()])?; if k.is_truthy() { result.push(item); } }
                                push!(Value::array(result));
                                continue 'dispatch;
                            }
                            "reduce" => {
                                if argc != 2 && argc != 3 { return Err(VmError::Generic("reduce requires 2-3 arguments".into())); }
                                let args: Vec<Value> = self.stack.drain(fn_idx..).skip(1).collect();
                                let (arr, cb) = (args[0].clone(), args[1].clone());
                                let items: Vec<Value> = match &arr {
                                    Value::Array(a) => a.borrow().clone(),
                                    Value::IntRange(s, e, inc) => {
                                        let (s, e, inc) = (*s, *e, *inc);
                                        if inc { (s..=e).map(Value::Int).collect() }
                                        else   { (s..e).map(Value::Int).collect() }
                                    }
                                    _ => return Err(VmError::TypeError(format!("reduce: first argument must be Array or Range, got {}", arr.type_name()))),
                                };
                                let (mut acc, si) = if argc == 3 { (args[2].clone(), 0) }
                                    else { if items.is_empty() { return Err(VmError::Generic("reduce() of empty array with no initial value".into())); } (items[0].clone(), 1) };
                                save_ip!();
                                for item in &items[si..] { acc = self.call_value_sync(cb.clone(), vec![acc, item.clone()])?; }
                                push!(acc);
                                continue 'dispatch;
                            }
                            _ => {}
                        }
                    }

                    match callee {
                        Value::NativeFn(_, f) => {
                            let args: Vec<Value> = self.stack.drain(fn_idx..).skip(1).collect();
                            push!(f(&args).map_err(VmError::Generic)?);
                        }
                        Value::Closure(c) => {
                            let proto = &c.proto;
                            if argc < proto.arity || argc > proto.max_arity {
                                return Err(VmError::ArityMismatch { name: proto.name.clone(), expected: proto.arity, got: argc });
                            }
                            let missing = proto.max_arity - argc;
                            for i in 0..missing {
                                let di = proto.defaults.len().saturating_sub(missing - i);
                                push!(proto.defaults.get(di).cloned().unwrap_or(Value::None));
                            }
                            let new_base = fn_idx + 1;
                            self.stack[fn_idx] = Value::None;
                            save_ip!();
                            self.frames.push(CallFrame {
                                chunk: Rc::clone(&proto.chunk), ip: 0,
                                base: new_base, name: proto.name.clone(),
                                upvalues: c.upvalues.clone(),
                            });
                            load_frame!();
                        }
                        other => return Err(VmError::NotCallable(other.type_name().to_string())),
                    }
                }

                Op::CallMethod => {
                    let name_idx = read_u16!() as usize;
                    let argc     = read_u8!() as usize;
                    let name     = chunk!().names[name_idx].clone();
                    let obj_idx  = self.stack.len() - argc - 1;
                    let obj      = self.stack[obj_idx].clone();

                    // Built-in Result/Option methods
                    let builtin_result = match (&obj, name.as_str()) {
                        (Value::Ok_(_)|Value::Err_(_), "is_ok")  => { self.stack.drain(obj_idx..); Some(Value::Bool(matches!(obj, Value::Ok_(_)))) }
                        (Value::Ok_(_)|Value::Err_(_), "is_err") => { self.stack.drain(obj_idx..); Some(Value::Bool(matches!(obj, Value::Err_(_)))) }
                        (Value::Ok_(inner), "unwrap")   => { self.stack.drain(obj_idx..); Some(*inner.clone()) }
                        (Value::Err_(e), "unwrap")      => return Err(VmError::Generic(format!("unwrap() chiamato su Err({})", e))),
                        (Value::Ok_(inner), "unwrap_or")=> { self.stack.drain(obj_idx..); Some(*inner.clone()) }
                        (Value::Err_(_), "unwrap_or")   => { let d = if argc > 0 { self.stack.drain(obj_idx..).nth(1).unwrap_or(Value::None) } else { Value::None }; Some(d) }
                        (Value::Some_(_), "is_some")    => { self.stack.drain(obj_idx..); Some(Value::Bool(true)) }
                        (Value::None, "is_some")        => { self.stack.drain(obj_idx..); Some(Value::Bool(false)) }
                        (Value::Some_(_), "is_none") | (Value::None, "is_none") => { let r = matches!(obj, Value::None); self.stack.drain(obj_idx..); Some(Value::Bool(r)) }
                        (Value::Some_(inner), "unwrap") => { self.stack.drain(obj_idx..); Some(*inner.clone()) }
                        (Value::None, "unwrap")         => return Err(VmError::Generic("unwrap() chiamato su None".into())),
                        (Value::Some_(inner), "unwrap_or") => { self.stack.drain(obj_idx..); Some(*inner.clone()) }
                        (Value::None, "unwrap_or")      => { let d = if argc > 0 { self.stack.drain(obj_idx..).nth(1).unwrap_or(Value::None) } else { Value::None }; Some(d) }
                        _ => None,
                    };
                    if let Some(result) = builtin_result { push!(result); continue 'dispatch; }

                    let method = match &obj {
                        Value::Instance(inst) => inst.borrow().fields.get(&name).cloned()
                            .ok_or_else(|| VmError::UnknownField { type_name: inst.borrow().class_name.clone(), field: name.clone() })?,
                        Value::Dict(d) => { let key = Value::str(&name); d.borrow().get(&key).cloned()
                            .ok_or_else(|| VmError::UnknownField { type_name: "Dict".into(), field: name.clone() })? }
                        other => return Err(VmError::UnknownField { type_name: other.type_name().to_string(), field: name.clone() }),
                    };

                    let is_module = matches!(obj, Value::Dict(_));
                    if !is_module { self.stack.insert(obj_idx + 1, obj); }
                    else          { self.stack.remove(obj_idx); }

                    match method {
                        Value::Closure(c) => {
                            let proto = &c.proto;
                            if argc < proto.arity || argc > proto.max_arity {
                                return Err(VmError::ArityMismatch { name: proto.name.clone(), expected: proto.arity, got: argc });
                            }
                            let missing = proto.max_arity - argc;
                            for i in 0..missing {
                                let di = proto.defaults.len().saturating_sub(missing - i);
                                push!(proto.defaults.get(di).cloned().unwrap_or(Value::None));
                            }
                            let new_base = obj_idx + 1;
                            self.stack[obj_idx] = Value::None;
                            save_ip!();
                            self.frames.push(CallFrame {
                                chunk: Rc::clone(&proto.chunk), ip: 0,
                                base: new_base, name: proto.name.clone(),
                                upvalues: c.upvalues.clone(),
                            });
                            load_frame!();
                        }
                        Value::NativeFn(_, f) => {
                            let args: Vec<Value> = if is_module { self.stack.drain(obj_idx..).collect() }
                                                   else         { self.stack.drain(obj_idx..).skip(1).collect() };
                            push!(f(&args).map_err(VmError::Generic)?);
                        }
                        other => return Err(VmError::NotCallable(other.type_name().to_string())),
                    }
                }

                Op::Return => {
                    let result = pop!();
                    let frame  = self.frames.pop().unwrap();
                    self.stack.truncate(frame.base - 1);
                    push!(result.clone());
                    if self.frames.is_empty() { return Ok(result); }
                    load_frame!();
                }

                Op::ReturnNil => {
                    let frame = self.frames.pop().unwrap();
                    self.stack.truncate(frame.base - 1);
                    push!(Value::None);
                    if self.frames.is_empty() { return Ok(Value::None); }
                    load_frame!();
                }

                Op::MakeArray => {
                    let count = read_u16!() as usize;
                    let start = self.stack.len() - count;
                    let items: Vec<Value> = self.stack.drain(start..).collect();
                    push!(Value::array(items));
                }
                Op::MakeDict => {
                    let count = read_u16!() as usize;
                    let start = self.stack.len() - count * 2;
                    let flat: Vec<Value> = self.stack.drain(start..).collect();
                    let pairs: Vec<(Value, Value)> = flat.chunks(2).map(|c| (c[0].clone(), c[1].clone())).collect();
                    push!(Value::dict(pairs));
                }
                Op::GetIndex => { let i = pop!(); let o = pop!(); push!(self.eval_index(o, i)?); }
                Op::SetIndex  => {
                    let val = pop!(); let idx_v = pop!(); let obj = pop!();
                    match (obj, &idx_v) {
                        (Value::Array(arr), Value::Int(i)) => { let len = arr.borrow().len(); let i = self.resolve_idx(*i, len)?; arr.borrow_mut()[i] = val; }
                        (Value::TypedArray(t), Value::Int(i)) => { let len = t.borrow().len(); let i = crate::value::resolve_idx(*i, len).map_err(VmError::Generic)?; t.borrow_mut().set(i, val).map_err(VmError::TypeError)?; }
                        (Value::Dict(d), key) => { d.borrow_mut().insert(key.clone(), val); }
                        _ => return Err(VmError::TypeError("index assignment requires Array, TypedArray or Dict".into())),
                    }
                }
                Op::MakeRange => {
                    let inc = read_u8!() != 0; let end = pop!(); let start = pop!();
                    match (&start, &end) {
                        (Value::Int(s), Value::Int(e)) => {
                            // v0.2.14: range lazy — nessuna allocazione Vec<Value>
                            push!(Value::IntRange(*s, *e, inc));
                        }
                        _ => return Err(VmError::TypeError("range bounds must be Int".into())),
                    }
                }
                Op::GetField => {
                    let idx = read_u16!() as usize; let name = chunk!().names[idx].clone(); let obj = pop!();
                    push!(self.get_field(obj, &name)?);
                }
                Op::SetField => {
                    let idx = read_u16!() as usize; let name = chunk!().names[idx].clone();
                    let val = pop!(); let obj = pop!();
                    match obj { Value::Instance(inst) => { inst.borrow_mut().fields.insert(name, val); }
                        _ => return Err(VmError::TypeError(format!("cannot set field on {}", obj.type_name()))) }
                }
                Op::MakeInstance => {
                    let idx = read_u16!() as usize; let cn = chunk!().names[idx].clone();
                    push!(Value::Instance(Rc::new(RefCell::new(Instance::new(&cn)))));
                }
                Op::SetTraits => {
                    let n = read_u8!() as usize;
                    let mut traits = Vec::with_capacity(n);
                    for _ in 0..n { let idx = read_u16!() as usize; traits.push(chunk!().names[idx].clone()); }
                    if let Value::Instance(inst) = peek!() { inst.borrow_mut().traits = traits; }
                }
                Op::MakeSome => { let v = pop!(); push!(Value::Some_(Box::new(v))); }
                Op::MakeOk   => { let v = pop!(); push!(Value::Ok_(Box::new(v)));   }
                Op::MakeErr  => { let v = pop!(); push!(Value::Err_(Box::new(v)));  }
                Op::Propagate => {
                    let v = pop!();
                    match v {
                        Value::Ok_(inner) => push!(*inner),
                        Value::Err_(e) => {
                            let err_val = Value::Err_(e);
                            if self.frames.len() <= 1 {
                                let frame = self.frames.pop().unwrap();
                                self.stack.truncate(frame.base.saturating_sub(1));
                                return Ok(err_val);
                            }
                            let frame = self.frames.pop().unwrap();
                            self.stack.truncate(frame.base - 1);
                            push!(err_val);
                            if self.frames.is_empty() { return Ok(pop!()); }
                            load_frame!();
                        }
                        other => return Err(VmError::TypeError(format!("operatore ? applicato a {} (richiede Ok o Err)", other.type_name()))),
                    }
                }
                Op::In    => { let h = pop!(); let n = pop!(); push!(Value::Bool(self.eval_in(n, h)?)); }
                Op::NotIn => { let h = pop!(); let n = pop!(); push!(Value::Bool(!self.eval_in(n, h)?)); }
                Op::Is    => {
                    let r = pop!(); let l = pop!();
                    let result = match (&l, &r) {
                        (Value::Instance(inst), Value::Str(s)) if !s.starts_with("trait:") => {
                            inst.borrow().class_name == s.as_str()
                        }
                        (Value::Instance(inst), Value::Str(s)) if s.starts_with("trait:") => {
                            let tn = &s["trait:".len()..];
                            inst.borrow().traits.iter().any(|t| t == tn)
                        }
                        (Value::Instance(a), Value::Instance(b)) => {
                            a.borrow().class_name == b.borrow().class_name
                        }
                        _ => std::mem::discriminant(&l) == std::mem::discriminant(&r),
                    };
                    push!(Value::Bool(result));
                }

                Op::IsSome    => { let o = read_i16!(); if !matches!(peek!(), Value::Some_(_)) { ip = (ip as isize + o as isize) as usize; } }
                Op::IsNone    => { let o = read_i16!(); if !matches!(peek!(), Value::None)     { ip = (ip as isize + o as isize) as usize; } }
                Op::IsOk      => { let o = read_i16!(); if !matches!(peek!(), Value::Ok_(_))   { ip = (ip as isize + o as isize) as usize; } }
                Op::IsErr     => { let o = read_i16!(); if !matches!(peek!(), Value::Err_(_))  { ip = (ip as isize + o as isize) as usize; } }
                Op::Unwrap    => {
                    let v = pop!();
                    let inner = match v { Value::Some_(i)|Value::Ok_(i)|Value::Err_(i) => *i, _ => return Err(VmError::TypeError(format!("cannot unwrap {}", v.type_name()))) };
                    push!(inner);
                }
                Op::MatchLit => {
                    let ci = read_u16!() as usize; let o = read_i16!();
                    let lit = chunk!().constants[ci].clone();
                    if peek!() != lit { ip = (ip as isize + o as isize) as usize; }
                }
                Op::MatchRange => {
                    let li = read_u16!() as usize; let hi = read_u16!() as usize;
                    let incl = read_u8!() != 0; let off = read_i16!();
                    let lo = match &chunk!().constants[li] { Value::Int(n) => n, _ => return Err(VmError::TypeError("range pattern needs Int".into())) };
                    let hi_v = match &chunk!().constants[hi] { Value::Int(n) => n, _ => return Err(VmError::TypeError("range pattern needs Int".into())) };
                    let matched = match peek!() { Value::Int(n) => if incl { n >= *lo && n <= *hi_v } else { n >= *lo && n < *hi_v }, _ => false };
                    if !matched { ip = (ip as isize + off as isize) as usize; }
                }

                Op::IntoIter => {
                    let v = pop!();
                    let arr = match v {
                        // v0.2.14: IntRange è già lazy, non serve convertire in Vec
                        Value::IntRange(_, _, _) => { push!(v); continue 'dispatch; }
                        Value::Array(a) => a,
                        Value::Dict(d)  => { let pairs: Vec<Value> = d.borrow().iter().map(|(k,v): (&Value,&Value)| Value::array(vec![k.clone(),v.clone()])).collect(); Rc::new(RefCell::new(pairs)) }
                        Value::TypedArray(t) => { let d = t.borrow(); let elems: Vec<Value> = (0..d.len()).map(|i| d.get(i).unwrap()).collect(); Rc::new(RefCell::new(elems)) }
                        Value::Str(s)   => { let chars: Vec<Value> = s.chars().map(|c| Value::str(c.to_string())).collect(); Rc::new(RefCell::new(chars)) }
                        _ => return Err(VmError::TypeError(format!("'{}' is not iterable", v.type_name()))),
                    };
                    push!(Value::Array(arr));
                }
                Op::IterNext => {
                    let il = read_u8!() as usize; let vl = read_u8!() as usize; let off = read_i16!();
                    let pl  = il + 1;

                    match self.stack[base + il].clone() {
                        // v0.2.14: range intero lazy — nessuna allocazione, solo aritmetica
                        Value::IntRange(start, end, inclusive) => {
                            let pos = match &self.stack[base + pl] { Value::Int(n) => *n, _ => 0 };
                            let actual = start + pos;
                            let done = if inclusive { actual > end } else { actual >= end };
                            if done {
                                ip = (ip as isize + off as isize) as usize;
                            } else {
                                self.stack[base + vl] = Value::Int(actual);
                                self.stack[base + pl] = Value::Int(pos + 1);
                            }
                        }
                        Value::Array(arr) => {
                            let pos = match &self.stack[base + pl] { Value::Int(n) => *n as usize, _ => 0 };
                            let len = arr.borrow().len();
                            if pos >= len { ip = (ip as isize + off as isize) as usize; }
                            else {
                                let item = arr.borrow()[pos].clone();
                                self.stack[base + vl] = item;
                                self.stack[base + pl] = Value::Int((pos + 1) as i64);
                            }
                        }
                        _ => return Err(VmError::TypeError("IterNext on non-iterable".into())),
                    }
                }

                Op::BuildStr => {
                    let n = read_u16!() as usize; let start = self.stack.len() - n;
                    let parts: Vec<String> = self.stack.drain(start..).map(|v| v.to_string()).collect();
                    push!(Value::str(parts.join("")));
                }
                Op::ToStr => { let v = pop!(); push!(Value::str(v.to_string())); }
                Op::Nop   => {}
                Op::Halt  => { return Ok(self.stack.pop().unwrap_or(Value::None)); }
            }
        }
    }

    fn build_trace(&self) -> String {
        let mut out = String::from("Traceback:\n");
        for frame in self.frames.iter().rev() {
            let line = frame.chunk.line_at(frame.ip.saturating_sub(1));
            out.push_str(&format!("  at {} (line {})\n", frame.name, line));
        }
        out
    }

    // ── Operazioni aritmetiche ────────────────────────────────────────────

    fn op_add(&self, l: Value, r: Value) -> VmResult {
        match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a),   Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a + *b as f64)),
            (Value::Str(a),   Value::Str(b))   => Ok(Value::str(format!("{}{}", a, b))),
            (Value::TypedArray(_), _) | (_, Value::TypedArray(_)) => typed_binop(&l, &r, |a,b| a+b, |a,b| a+b),
            _ => Err(VmError::TypeError(format!("'+' between {} and {}", l.type_name(), r.type_name()))),
        }
    }
    fn op_sub(&self, l: Value, r: Value) -> VmResult {
        match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a),   Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a - *b as f64)),
            (Value::TypedArray(_), _) | (_, Value::TypedArray(_)) => typed_binop(&l, &r, |a,b| a-b, |a,b| a-b),
            _ => Err(VmError::TypeError(format!("'-' between {} and {}", l.type_name(), r.type_name()))),
        }
    }
    fn op_mul(&self, l: Value, r: Value) -> VmResult {
        match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a),   Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a * *b as f64)),
            (Value::Str(s),   Value::Int(n))   => Ok(Value::str(s.repeat((*n).max(0) as usize))),
            (Value::Int(n),   Value::Str(s))   => Ok(Value::str(s.repeat((*n).max(0) as usize))),
            (Value::TypedArray(_), _) | (_, Value::TypedArray(_)) => typed_binop(&l, &r, |a,b| a*b, |a,b| a*b),
            _ => Err(VmError::TypeError(format!("'*' between {} and {}", l.type_name(), r.type_name()))),
        }
    }
    fn op_div(&self, l: Value, r: Value) -> VmResult {
        match (&l, &r) {
            (Value::TypedArray(_), _) | (_, Value::TypedArray(_)) => typed_binop(&l, &r, |a,b| a/b, |a,b| a/b),
            _ => {
                let b = r.as_float().ok_or_else(|| VmError::TypeError(format!("'/' on {}", r.type_name())))?;
                if b == 0.0 { return Err(VmError::DivisionByZero); }
                let a = l.as_float().ok_or_else(|| VmError::TypeError(format!("'/' on {}", l.type_name())))?;
                Ok(Value::Float(a / b))
            }
        }
    }
    fn op_intdiv(&self, l: Value, r: Value) -> VmResult {
        match (&l, &r) {
            (_, Value::Int(0)) => Err(VmError::DivisionByZero),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
            _ => { let b = r.as_float().unwrap(); if b == 0.0 { return Err(VmError::DivisionByZero); } Ok(Value::Int((l.as_float().unwrap() / b).floor() as i64)) }
        }
    }
    fn op_mod(&self, l: Value, r: Value) -> VmResult {
        match (&l, &r) {
            (_, Value::Int(0)) => Err(VmError::DivisionByZero),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
            _ => { let b = r.as_float().unwrap(); if b == 0.0 { return Err(VmError::DivisionByZero); } Ok(Value::Float(l.as_float().unwrap() % b)) }
        }
    }
    fn op_pow(&self, l: Value, r: Value) -> VmResult {
        match (&l, &r) {
            (Value::Int(a), Value::Int(b)) if *b >= 0 => Ok(Value::Int((*a).pow(*b as u32))),
            _ => {
                let a = l.as_float().ok_or_else(|| VmError::TypeError("'**' on non-numeric".into()))?;
                let b = r.as_float().ok_or_else(|| VmError::TypeError("'**' on non-numeric".into()))?;
                Ok(Value::Float(a.powf(b)))
            }
        }
    }
    fn op_bit(&self, l: Value, r: Value, op: &str) -> VmResult {
        match (&l, &r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(match op {
                "&" => a & b, "|" => a | b, "^" => a ^ b, "<<" => a << b, ">>" => a >> b, _ => unreachable!(),
            })),
            _ => Err(VmError::TypeError(format!("'{}' requires Int", op))),
        }
    }

    // ── Field / Index / Membership ────────────────────────────────────────

    pub fn get_field(&self, obj: Value, field: &str) -> VmResult {
        match &obj {
            Value::Instance(inst) => {
                if let Some(v) = inst.borrow().fields.get(field).cloned() { return Ok(v); }
                Err(VmError::UnknownField { type_name: inst.borrow().class_name.clone(), field: field.to_string() })
            }
            Value::Dict(d) => {
                let key = Value::str(field);
                d.borrow().get(&key).cloned()
                    .ok_or_else(|| VmError::UnknownField { type_name: "Dict".into(), field: field.to_string() })
            }
            Value::Array(arr) => match field {
                "len" => Ok(Value::Int(arr.borrow().len() as i64)),
                _ => Err(VmError::UnknownField { type_name: "Array".into(), field: field.into() }),
            },
            Value::Str(s) => match field {
                "len" => Ok(Value::Int(s.chars().count() as i64)),
                _ => Err(VmError::UnknownField { type_name: "Str".into(), field: field.into() }),
            },
            _ => Err(VmError::UnknownField { type_name: obj.type_name().to_string(), field: field.to_string() }),
        }
    }

    fn eval_index(&self, obj: Value, idx: Value) -> VmResult {
        match &obj {
            Value::Dict(d) => {
                d.borrow().get(&idx).cloned()
                    .ok_or_else(|| VmError::Generic(format!("key not found in Dict: {}", idx)))
            }
            Value::TypedArray(t) => {
                let ta = t.borrow(); let len = ta.len();
                match &idx {
                    Value::Int(i) => {
                        let ui = crate::value::resolve_idx(*i, len).map_err(VmError::Generic)?;
                        ta.get(ui).ok_or_else(|| VmError::IndexOutOfBounds { index: *i, len })
                    }
                    // Lazy range (v0.2.14) — genera gli indici senza allocare un Vec intermedio
                    Value::IntRange(start, end, inclusive) => {
                        let indices: Vec<i64> = if *inclusive {
                            (*start..=*end).collect()
                        } else {
                            (*start..*end).collect()
                        };
                        let sliced = ta.slice_indices(&indices, len).map_err(VmError::Generic)?;
                        Ok(Value::typed_array(sliced))
                    }
                    Value::Array(arr) => {
                        let indices: Vec<i64> = arr.borrow().iter()
                            .map(|v| if let Value::Int(n) = v { Ok(*n) } else { Err(VmError::TypeError("slice indices must be Int".into())) })
                            .collect::<Result<_,_>>()?;
                        let sliced = ta.slice_indices(&indices, len).map_err(VmError::Generic)?;
                        Ok(Value::typed_array(sliced))
                    }
                    _ => Err(VmError::TypeError(format!("TypedArray index must be Int or Range, got {}", idx.type_name()))),
                }
            }
            _ => {
                let i = match &idx { Value::Int(n) => n, _ => return Err(VmError::TypeError("array/string index must be Int".into())) };
                match obj {
                    Value::Array(arr) => { let len = arr.borrow().len(); let a = self.resolve_idx(*i, len)?; Ok(arr.borrow()[a].clone()) }
                    Value::Str(s) => { let chars: Vec<char> = s.chars().collect(); let a = self.resolve_idx(*i, chars.len())?; Ok(Value::str(chars[a].to_string())) }
                    _ => Err(VmError::TypeError(format!("cannot index {}", obj.type_name()))),
                }
            }
        }
    }

    fn resolve_idx(&self, i: i64, len: usize) -> VmResult<usize> {
        let a = if i < 0 { len as i64 + i } else { i };
        if a < 0 || a as usize >= len { Err(VmError::IndexOutOfBounds { index: i, len }) }
        else { Ok(a as usize) }
    }

    fn eval_in(&self, needle: Value, haystack: Value) -> VmResult<bool> {
        match haystack {
            Value::Array(arr) => Ok(arr.borrow().contains(&needle)),
            Value::Dict(d)    => Ok(d.borrow().contains_key(&needle)),
            Value::Str(s)     => match &needle { Value::Str(n) => Ok(s.contains(n.as_str())), _ => Ok(false) },
            Value::IntRange(start, end, inclusive) => match &needle {
                Value::Int(n) => Ok(if inclusive { *n >= start && *n <= end } else { *n >= start && *n < end }),
                _ => Ok(false),
            },
            _ => Err(VmError::TypeError(format!("'in' on {}", haystack.type_name()))),
        }
    }

    // ── call_value_sync — mini loop per HOF ───────────────────────────────
    // Usato da map/filter/reduce. Usa lo stesso pattern unsafe del loop principale.
    // Supporta solo opcode presenti nelle closure HOF tipiche.

    pub fn call_value_sync(&mut self, callee: Value, args: Vec<Value>) -> VmResult<Value> {
        match callee {
            Value::NativeFn(_, f) => f(&args).map_err(VmError::Generic),
            Value::Closure(c) => {
                let depth    = self.frames.len();
                let base_idx = self.stack.len();
                self.stack.push(Value::None);
                for a in args { self.stack.push(a); }
                let proto    = &c.proto;
                let new_base = base_idx + 1;
                self.frames.push(CallFrame {
                    chunk: Rc::clone(&proto.chunk), ip: 0,
                    base: new_base, name: proto.name.clone(),
                    upvalues: c.upvalues.clone(),
                });

                let mut ip:   usize = 0;
                let mut base: usize = new_base;
                // SAFETY: il frame è appena stato pushato.
                let mut chunk_ptr: *const Chunk = Rc::as_ptr(&self.frames.last().unwrap().chunk);

                macro_rules! cc     { () => { unsafe { &*chunk_ptr } } }
                macro_rules! ru8    { () => {{ let v = cc!().code[ip]; ip += 1; v }} }
                macro_rules! ru16   { () => {{ let v = crate::chunk::read_u16(&cc!().code, ip); ip += 2; v }} }
                macro_rules! ri16   { () => {{ let v = crate::chunk::read_i16(&cc!().code, ip); ip += 2; v }} }
                macro_rules! cp     { () => { self.stack.pop().ok_or_else(|| VmError::Generic("stack underflow".into()))? } }
                macro_rules! ck     { () => { self.stack.last().ok_or_else(|| VmError::Generic("stack empty".into()))?.clone() } }
                macro_rules! ps     { ($v:expr) => { self.stack.push($v) } }
                macro_rules! sip    { () => { self.frames.last_mut().unwrap().ip = ip; } }
                macro_rules! lf     { () => {{ let f = self.frames.last().unwrap(); chunk_ptr = Rc::as_ptr(&f.chunk); ip = f.ip; base = f.base; }} }

                loop {
                    let op_byte = cc!().code[ip];
                    let op = Op::from_u8(op_byte).ok_or_else(|| VmError::Generic(format!("bad opcode {}", op_byte)))?;
                    ip += 1;

                    match op {
                        Op::Const  => { let i = ru16!() as usize; let v = cc!().constants[i].clone(); ps!(v); }
                        Op::True   => ps!(Value::Bool(true)),
                        Op::False  => ps!(Value::Bool(false)),
                        Op::Nil    => ps!(Value::None),
                        Op::Pop    => { cp!(); }
                        Op::Dup    => { let v = ck!(); ps!(v); }
                        Op::Swap   => { let t = cp!(); let u = cp!(); ps!(t); ps!(u); }
                        Op::PopN   => { let n = ru8!() as usize; let l = self.stack.len(); self.stack.truncate(l - n); }
                        Op::LoadLocal  => { let i = ru8!() as usize; let v = self.stack[base+i].clone(); ps!(v); }
                        Op::StoreLocal => { let i = ru8!() as usize; let v = cp!(); self.stack[base+i] = v; }
                        Op::LoadUpval  => { let i = ru8!() as usize; let v = self.frames.last().unwrap().upvalues[i].value.borrow().clone(); ps!(v); }
                        Op::StoreUpval => { let i = ru8!() as usize; let v = cp!(); *self.frames.last_mut().unwrap().upvalues[i].value.borrow_mut() = v; }
                        Op::LoadGlobal  => { let i = ru16!() as usize; let n = &cc!().names[i]; let v = self.globals.get(n).map(|(v,_)| v.clone()).ok_or_else(|| VmError::UndefinedVariable(n.clone()))?; ps!(v); }
                        Op::StoreGlobal => { let i = ru16!() as usize; let n = cc!().names[i].clone(); let v = cp!(); match self.globals.get_mut(&n) { Some((c,true)) => *c=v, Some((_,false)) => return Err(VmError::AssignImmutable(n)), None => return Err(VmError::UndefinedVariable(n)) } }
                        Op::DefGlobal   => { let i = ru16!() as usize; let m = ru8!()!=0; let n = cc!().names[i].clone(); let v = cp!(); self.globals.insert(n,(v,m)); }
                        Op::Add    => { let r = cp!(); let l = cp!(); ps!(self.op_add(l,r)?); }
                        Op::Sub    => { let r = cp!(); let l = cp!(); ps!(self.op_sub(l,r)?); }
                        Op::Mul    => { let r = cp!(); let l = cp!(); ps!(self.op_mul(l,r)?); }
                        Op::Div    => { let r = cp!(); let l = cp!(); ps!(self.op_div(l,r)?); }
                        Op::IntDiv => { let r = cp!(); let l = cp!(); ps!(self.op_intdiv(l,r)?); }
                        Op::Mod    => { let r = cp!(); let l = cp!(); ps!(self.op_mod(l,r)?); }
                        Op::Pow    => { let r = cp!(); let l = cp!(); ps!(self.op_pow(l,r)?); }
                        Op::Neg    => { let v = cp!(); ps!(match v { Value::Int(n) => Value::Int(-n), Value::Float(f) => Value::Float(-f), _ => return Err(VmError::TypeError(format!("unary '-' on {}", v.type_name()))) }); }
                        Op::BitAnd => { let r = cp!(); let l = cp!(); ps!(self.op_bit(l,r,"&")?); }
                        Op::BitOr  => { let r = cp!(); let l = cp!(); ps!(self.op_bit(l,r,"|")?); }
                        Op::BitXor => { let r = cp!(); let l = cp!(); ps!(self.op_bit(l,r,"^")?); }
                        Op::Shl    => { let r = cp!(); let l = cp!(); ps!(self.op_bit(l,r,"<<")?); }
                        Op::Shr    => { let r = cp!(); let l = cp!(); ps!(self.op_bit(l,r,">>")?); }
                        Op::BitNot => { let v = cp!(); ps!(match v { Value::Int(n) => Value::Int(!n), _ => return Err(VmError::TypeError(format!("'~' on {}", v.type_name()))) }); }
                        Op::Eq  => { let r = cp!(); let l = cp!(); ps!(Value::Bool(l==r)); }
                        Op::Ne  => { let r = cp!(); let l = cp!(); ps!(Value::Bool(l!=r)); }
                        Op::Lt  => { let r = cp!(); let l = cp!(); ps!(Value::Bool(l< r)); }
                        Op::Le  => { let r = cp!(); let l = cp!(); ps!(Value::Bool(l<=r)); }
                        Op::Gt  => { let r = cp!(); let l = cp!(); ps!(Value::Bool(l> r)); }
                        Op::Ge  => { let r = cp!(); let l = cp!(); ps!(Value::Bool(l>=r)); }
                        Op::Not => { let v = cp!(); ps!(Value::Bool(!v.is_truthy())); }
                        Op::Jump          => { let o = ri16!(); ip = (ip as isize + o as isize) as usize; }
                        Op::JumpFalse     => { let o = ri16!(); let v = cp!(); if !v.is_truthy() { ip = (ip as isize+o as isize) as usize; } }
                        Op::JumpTrue      => { let o = ri16!(); let v = cp!(); if  v.is_truthy() { ip = (ip as isize+o as isize) as usize; } }
                        Op::JumpFalsePeek => { let o = ri16!(); if !ck!().is_truthy() { ip = (ip as isize+o as isize) as usize; } }
                        Op::JumpTruePeek  => { let o = ri16!(); if  ck!().is_truthy() { ip = (ip as isize+o as isize) as usize; } }
                        Op::MakeClosure => {
                            let i = ru16!() as usize; let nu = ru8!() as usize;
                            let proto = Rc::new(cc!().fn_protos[i].clone());
                            let mut upvalues = Vec::with_capacity(nu);
                            if nu > 0 { let s = self.stack.len()-nu; for v in self.stack.drain(s..) { upvalues.push(Upvalue { value: Rc::new(RefCell::new(v)) }); } }
                            ps!(Value::Closure(Rc::new(Closure { proto, upvalues })));
                        }
                        Op::Call => {
                            let argc = ru8!() as usize;
                            let fi = self.stack.len() - argc - 1;
                            let callee = self.stack[fi].clone();
                            match callee {
                                Value::NativeFn(_, f) => { let args: Vec<Value> = self.stack.drain(fi..).skip(1).collect(); ps!(f(&args).map_err(VmError::Generic)?); }
                                Value::Closure(c2) => {
                                    let p = &c2.proto;
                                    if argc < p.arity || argc > p.max_arity { return Err(VmError::ArityMismatch { name: p.name.clone(), expected: p.arity, got: argc }); }
                                    let miss = p.max_arity - argc;
                                    for i2 in 0..miss { let di = p.defaults.len().saturating_sub(miss-i2); ps!(p.defaults.get(di).cloned().unwrap_or(Value::None)); }
                                    let nb = fi + 1; self.stack[fi] = Value::None;
                                    sip!();
                                    self.frames.push(CallFrame { chunk: Rc::clone(&p.chunk), ip: 0, base: nb, name: p.name.clone(), upvalues: c2.upvalues.clone() });
                                    lf!();
                                }
                                other => return Err(VmError::NotCallable(other.type_name().to_string())),
                            }
                        }
                        Op::Return => {
                            let result = cp!();
                            let frame  = self.frames.pop().unwrap();
                            self.stack.truncate(frame.base - 1);
                            if self.frames.len() <= depth {
                                self.stack.truncate(base_idx);
                                return Ok(result);
                            }
                            ps!(result);
                            lf!();
                        }
                        Op::ReturnNil => {
                            let frame = self.frames.pop().unwrap();
                            self.stack.truncate(frame.base - 1);
                            if self.frames.len() <= depth {
                                self.stack.truncate(base_idx);
                                return Ok(Value::None);
                            }
                            ps!(Value::None);
                            lf!();
                        }
                        Op::GetField => { let i = ru16!() as usize; let n = cc!().names[i].clone(); let o = cp!(); ps!(self.get_field(o, &n)?); }
                        Op::SetField => { let i = ru16!() as usize; let n = cc!().names[i].clone(); let v = cp!(); let o = cp!(); match o { Value::Instance(inst) => { inst.borrow_mut().fields.insert(n,v); } _ => return Err(VmError::TypeError(format!("cannot set field on {}", o.type_name()))) } }
                        Op::MakeInstance => { let i = ru16!() as usize; let cn = cc!().names[i].clone(); ps!(Value::Instance(Rc::new(RefCell::new(Instance::new(&cn))))); }
                        Op::SetTraits => { let n = ru8!() as usize; let mut tr = Vec::with_capacity(n); for _ in 0..n { let i = ru16!() as usize; tr.push(cc!().names[i].clone()); } if let Some(Value::Instance(inst)) = self.stack.last() { inst.borrow_mut().traits = tr; } }
                        Op::MakeArray => { let c = ru16!() as usize; let s = self.stack.len()-c; let items: Vec<Value> = self.stack.drain(s..).collect(); ps!(Value::array(items)); }
                        Op::MakeDict  => { let c = ru16!() as usize; let s = self.stack.len()-c*2; let flat: Vec<Value> = self.stack.drain(s..).collect(); let pairs: Vec<(Value,Value)> = flat.chunks(2).map(|c| (c[0].clone(),c[1].clone())).collect(); ps!(Value::dict(pairs)); }
                        Op::GetIndex  => { let i = cp!(); let o = cp!(); ps!(self.eval_index(o,i)?); }
                        Op::MakeSome  => { let v = cp!(); ps!(Value::Some_(Box::new(v))); }
                        Op::MakeOk    => { let v = cp!(); ps!(Value::Ok_(Box::new(v)));   }
                        Op::MakeErr   => { let v = cp!(); ps!(Value::Err_(Box::new(v)));  }
                        Op::In    => { let h = cp!(); let n = cp!(); ps!(Value::Bool(self.eval_in(n,h)?)); }
                        Op::NotIn => { let h = cp!(); let n = cp!(); ps!(Value::Bool(!self.eval_in(n,h)?)); }
                        Op::Is    => { let r = cp!(); let l = cp!(); let res = match (&l,&r) { (Value::Instance(i),Value::Str(s)) if !s.starts_with("trait:") => i.borrow().class_name==s.as_str(), (Value::Instance(i),Value::Str(s)) if s.starts_with("trait:") => { let tn=&s["trait:".len()..]; i.borrow().traits.iter().any(|t|t==tn) } (Value::Instance(a),Value::Instance(b)) => a.borrow().class_name==b.borrow().class_name, _ => std::mem::discriminant(&l)==std::mem::discriminant(&r) }; ps!(Value::Bool(res)); }
                        Op::IsSome => { let o = ri16!(); if !matches!(ck!(), Value::Some_(_)) { ip = (ip as isize+o as isize) as usize; } }
                        Op::IsNone => { let o = ri16!(); if !matches!(ck!(), Value::None)     { ip = (ip as isize+o as isize) as usize; } }
                        Op::IsOk   => { let o = ri16!(); if !matches!(ck!(), Value::Ok_(_))   { ip = (ip as isize+o as isize) as usize; } }
                        Op::IsErr  => { let o = ri16!(); if !matches!(ck!(), Value::Err_(_))  { ip = (ip as isize+o as isize) as usize; } }
                        Op::Unwrap => { let v = cp!(); let inner = match v { Value::Some_(i)|Value::Ok_(i)|Value::Err_(i) => *i, _ => return Err(VmError::TypeError(format!("cannot unwrap {}", v.type_name()))) }; ps!(inner); }
                        Op::BuildStr => { let n = ru16!() as usize; let s = self.stack.len()-n; let parts: Vec<String> = self.stack.drain(s..).map(|v| v.to_string()).collect(); ps!(Value::str(parts.join(""))); }
                        Op::ToStr => { let v = cp!(); ps!(Value::str(v.to_string())); }
                        Op::Nop  => {}
                        Op::Halt => { let r = self.stack.pop().unwrap_or(Value::None); self.stack.truncate(base_idx); return Ok(r); }
                        Op::IntoIter => {
                            let v = cp!();
                            let arr = match v {
                                Value::IntRange(_, _, _) => { ps!(v); continue; }
                                Value::Array(a) => a,
                                Value::Dict(d) => { let p: Vec<Value> = d.borrow().iter().map(|(k,v): (&Value,&Value)| Value::array(vec![k.clone(),v.clone()])).collect(); Rc::new(RefCell::new(p)) }
                                Value::TypedArray(t) => { let d = t.borrow(); let e: Vec<Value> = (0..d.len()).map(|i| d.get(i).unwrap()).collect(); Rc::new(RefCell::new(e)) }
                                Value::Str(s) => { let ch: Vec<Value> = s.chars().map(|c| Value::str(c.to_string())).collect(); Rc::new(RefCell::new(ch)) }
                                _ => return Err(VmError::TypeError(format!("'{}' is not iterable", v.type_name()))),
                            };
                            ps!(Value::Array(arr));
                        }
                        Op::IterNext => {
                            let il = ru8!() as usize; let vl = ru8!() as usize; let off = ri16!();
                            let pl = il+1;
                            match self.stack[base+il].clone() {
                                Value::IntRange(start, end, inclusive) => {
                                    let pos = match &self.stack[base+pl] { Value::Int(n) => *n, _ => 0 };
                                    let actual = start + pos;
                                    let done = if inclusive { actual > end } else { actual >= end };
                                    if done { ip = (ip as isize+off as isize) as usize; }
                                    else { self.stack[base+vl] = Value::Int(actual); self.stack[base+pl] = Value::Int(pos+1); }
                                }
                                Value::Array(arr) => {
                                    let pos = match &self.stack[base+pl] { Value::Int(n) => *n as usize, _ => 0 };
                                    let len = arr.borrow().len();
                                    if pos >= len { ip = (ip as isize+off as isize) as usize; }
                                    else { let item = arr.borrow()[pos].clone(); self.stack[base+vl] = item; self.stack[base+pl] = Value::Int((pos+1) as i64); }
                                }
                                _ => return Err(VmError::TypeError("IterNext on non-iterable".into())),
                            }
                        }
                        Op::MatchLit => { let ci = ru16!() as usize; let o = ri16!(); let lit = cc!().constants[ci].clone(); if ck!() != lit { ip = (ip as isize+o as isize) as usize; } }
                        Op::MatchRange => {
                            let li = ru16!() as usize; let hii = ru16!() as usize; let incl = ru8!()!=0; let off = ri16!();
                            let lo = match &cc!().constants[li] { Value::Int(n) => n, _ => return Err(VmError::TypeError("range pattern needs Int".into())) };
                            let hi_v = match &cc!().constants[hii] { Value::Int(n) => n, _ => return Err(VmError::TypeError("range pattern needs Int".into())) };
                            let m = match ck!() { Value::Int(n) => if incl { n>=*lo && n<=*hi_v } else { n>=*lo && n<*hi_v }, _ => false };
                            if !m { ip = (ip as isize+off as isize) as usize; }
                        }
                        Op::MakeRange => {
                            let inc = ru8!()!=0; let end = cp!(); let start = cp!();
                            match (&start, &end) {
                                (Value::Int(s), Value::Int(e)) => { ps!(Value::IntRange(*s, *e, inc)); }
                                _ => return Err(VmError::TypeError("range bounds must be Int".into())),
                            }
                        }
                        Op::SetIndex => {
                            let val = cp!(); let idx_v = cp!(); let obj = cp!();
                            match (obj, &idx_v) {
                                (Value::Array(arr), Value::Int(i)) => { let len = arr.borrow().len(); let i = self.resolve_idx(*i,len)?; arr.borrow_mut()[i] = val; }
                                (Value::TypedArray(t), Value::Int(i)) => { let len = t.borrow().len(); let i = crate::value::resolve_idx(*i,len).map_err(VmError::Generic)?; t.borrow_mut().set(i,val).map_err(VmError::TypeError)?; }
                                (Value::Dict(d), key) => { d.borrow_mut().insert(key.clone(), val); }
                                _ => return Err(VmError::TypeError("index assignment requires Array, TypedArray or Dict".into())),
                            }
                        }
                        Op::Propagate => {
                            let v = cp!();
                            match v {
                                Value::Ok_(inner) => ps!(*inner),
                                Value::Err_(e) => {
                                    let ev = Value::Err_(e);
                                    if self.frames.len() <= depth+1 { self.stack.truncate(base_idx); return Ok(ev); }
                                    let frame = self.frames.pop().unwrap(); self.stack.truncate(frame.base-1);
                                    ps!(ev);
                                    if self.frames.len() <= depth { self.stack.truncate(base_idx); return Ok(cp!()); }
                                    lf!();
                                }
                                other => return Err(VmError::TypeError(format!("operatore ? applicato a {} (richiede Ok o Err)", other.type_name()))),
                            }
                        }
                        Op::CallMethod => {
                            // CallMethod in HOF closure — gestione completa
                            let ni = ru16!() as usize; let argc = ru8!() as usize;
                            let name = cc!().names[ni].clone();
                            let oi = self.stack.len() - argc - 1;
                            let obj = self.stack[oi].clone();

                            let method = match &obj {
                                Value::Instance(inst) => inst.borrow().fields.get(&name).cloned()
                                    .ok_or_else(|| VmError::UnknownField { type_name: inst.borrow().class_name.clone(), field: name.clone() })?,
                                Value::Dict(d) => { let key = Value::str(&name); d.borrow().get(&key).cloned()
                                    .ok_or_else(|| VmError::UnknownField { type_name: "Dict".into(), field: name.clone() })? }
                                other => return Err(VmError::UnknownField { type_name: other.type_name().to_string(), field: name.clone() }),
                            };
                            let is_mod = matches!(obj, Value::Dict(_));
                            if !is_mod { self.stack.insert(oi+1, obj); } else { self.stack.remove(oi); }
                            match method {
                                Value::Closure(c2) => {
                                    let p = &c2.proto;
                                    if argc < p.arity || argc > p.max_arity { return Err(VmError::ArityMismatch { name: p.name.clone(), expected: p.arity, got: argc }); }
                                    let miss = p.max_arity - argc;
                                    for i2 in 0..miss { let di = p.defaults.len().saturating_sub(miss-i2); ps!(p.defaults.get(di).cloned().unwrap_or(Value::None)); }
                                    let nb = oi+1; self.stack[oi] = Value::None;
                                    sip!();
                                    self.frames.push(CallFrame { chunk: Rc::clone(&p.chunk), ip: 0, base: nb, name: p.name.clone(), upvalues: c2.upvalues.clone() });
                                    lf!();
                                }
                                Value::NativeFn(_, f) => {
                                    let args: Vec<Value> = if is_mod { self.stack.drain(oi..).collect() } else { self.stack.drain(oi..).skip(1).collect() };
                                    ps!(f(&args).map_err(VmError::Generic)?);
                                }
                                other => return Err(VmError::NotCallable(other.type_name().to_string())),
                            }
                        }
                    }
                }
            }
            other => Err(VmError::NotCallable(other.type_name().to_string())),
        }
    }
}

impl Default for Vm {
    fn default() -> Self { Self::new() }
}
