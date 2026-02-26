use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::chunk::{read_i16, read_u16, Chunk, FnProto};
use crate::compiler::ClassInfo;
use crate::error::{VmError, VmResult};
use crate::opcode::Op;
use crate::stdlib;
use crate::value::{Closure, Instance, Upvalue, Value};

const STACK_MAX:  usize = 4096;
const FRAMES_MAX: usize = 256;

// ── Call frame ────────────────────────────────────────────────────────────

struct CallFrame {
    /// Puntatore al prototipo della funzione in esecuzione.
    chunk: Rc<Chunk>,
    /// Instruction pointer (indice in chunk.code).
    ip: usize,
    /// Indice nel value_stack dove iniziano i locali di questo frame.
    base: usize,
    /// Nome della funzione (per messaggi di errore).
    name: String,
    /// Upvalue catturati dalla closure corrente.
    upvalues: Vec<Upvalue>,
}

// ── VM ────────────────────────────────────────────────────────────────────

pub struct Vm {
    stack:   Vec<Value>,
    frames:  Vec<CallFrame>,
    globals: HashMap<String, (Value, bool)>, // (valore, è_mutabile)
    class_registry: HashMap<String, ClassInfo>,
}

impl Vm {
    pub fn new() -> Self {
        let mut vm = Vm {
            stack:          Vec::with_capacity(256),
            frames:         Vec::with_capacity(32),
            globals:        HashMap::new(),
            class_registry: HashMap::new(),
        };
        stdlib::register_globals(&mut vm.globals);
        vm
    }

    // ── Esecuzione ────────────────────────────────────────────────────────

    pub fn run_chunk(&mut self, chunk: Chunk) -> VmResult<Value> {
        // Installa il class_registry dal chunk top-level (viene popolato dal compiler)
        // In v0.2.0 il class_registry è gestito inline dalla VM durante l'esecuzione
        // di MakeInstance.

        let rc_chunk = Rc::new(chunk);
        self.frames.push(CallFrame {
            chunk:    rc_chunk,
            ip:       0,
            base:     0,
            name:     "<script>".to_string(),
            upvalues: Vec::new(),
        });

        loop {
            let result = self.step();
            match result {
                Ok(Some(v)) => {
                    self.frames.pop();
                    return Ok(v);
                }
                Ok(None) => continue,
                Err(e) => {
                    // Stampa il traceback
                    let trace = self.build_trace();
                    eprintln!("{}", trace);
                    self.frames.clear();
                    self.stack.clear();
                    return Err(e);
                }
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

    // ── Step: esegue una singola istruzione ───────────────────────────────

    fn step(&mut self) -> Result<Option<Value>, VmError> {
        let frame = self.frames.last_mut().unwrap();
        let ip = frame.ip;
        let op_byte = frame.chunk.code[ip];
        let op = Op::from_u8(op_byte).ok_or_else(|| VmError::Generic(format!("bad opcode {}", op_byte)))?;
        frame.ip += 1;

        // Legge gli operandi (clona il chunk pointer per evitare borrow multipli)
        let chunk = Rc::clone(&frame.chunk);
        let base  = frame.base;
        let mut ip = frame.ip; // ip dopo l'opcode — MUTABILE: si aggiorna ad ogni lettura

        macro_rules! read_u8  { () => {{
            let v = chunk.code[ip];
            ip += 1;
            self.frames.last_mut().unwrap().ip = ip;
            v
        }} }
        macro_rules! read_u16 { () => {{
            let v = crate::chunk::read_u16(&chunk.code, ip);
            ip += 2;
            self.frames.last_mut().unwrap().ip = ip;
            v
        }} }
        macro_rules! read_i16 { () => {{
            let v = crate::chunk::read_i16(&chunk.code, ip);
            ip += 2;
            self.frames.last_mut().unwrap().ip = ip;
            v
        }} }

        macro_rules! pop   { () => { self.stack.pop().ok_or_else(|| VmError::Generic("stack underflow".into()))? } }
        macro_rules! peek  { () => { self.stack.last().ok_or_else(|| VmError::Generic("stack empty".into()))?.clone() } }
        macro_rules! push  { ($v:expr) => { self.stack.push($v) } }

        match op {
            // ── Costanti ──────────────────────────────────────────────────
            Op::Const => {
                let idx = read_u16!();
                let v = chunk.constants[idx as usize].clone();
                push!(v);
            }
            Op::True  => push!(Value::Bool(true)),
            Op::False => push!(Value::Bool(false)),
            Op::Nil   => push!(Value::None),

            // ── Stack ─────────────────────────────────────────────────────
            Op::Pop  => { pop!(); }
            Op::Dup  => { let v = peek!(); push!(v); }
            Op::PopN => {
                let n = read_u8!() as usize;
                let len = self.stack.len();
                self.stack.truncate(len - n);
            }

            // ── Locali ────────────────────────────────────────────────────
            Op::LoadLocal => {
                let idx = read_u8!() as usize;
                let v = self.stack[base + idx].clone();
                push!(v);
            }
            Op::StoreLocal => {
                let idx = read_u8!() as usize;
                let v = pop!();
                self.stack[base + idx] = v;
            }

            // ── Upvalue ───────────────────────────────────────────────────
            Op::LoadUpval => {
                let idx = read_u8!() as usize;
                let v = self.frames.last().unwrap().upvalues[idx].value.clone();
                push!(v);
            }
            Op::StoreUpval => {
                let idx = read_u8!() as usize;
                let v = pop!();
                self.frames.last_mut().unwrap().upvalues[idx].value = v;
            }

            // ── Globali ───────────────────────────────────────────────────
            Op::LoadGlobal => {
                let idx  = read_u16!() as usize;
                let name = &chunk.names[idx];
                let v = self.globals.get(name)
                    .map(|(v, _)| v.clone())
                    .ok_or_else(|| VmError::UndefinedVariable(name.clone()))?;
                push!(v);
            }
            Op::StoreGlobal => {
                let idx  = read_u16!() as usize;
                let name = chunk.names[idx].clone();
                let v = pop!();
                if let Some((cur, mutable)) = self.globals.get_mut(&name) {
                    if !*mutable {
                        return Err(VmError::AssignImmutable(name));
                    }
                    *cur = v;
                } else {
                    return Err(VmError::UndefinedVariable(name));
                }
            }
            Op::DefGlobal => {
                let idx     = read_u16!() as usize;
                let mutable = read_u8!() != 0;
                let name    = chunk.names[idx].clone();
                let v = pop!();
                self.globals.insert(name, (v, mutable));
            }

            // ── Aritmetica ────────────────────────────────────────────────
            Op::Add => { let r = pop!(); let l = pop!(); push!(self.op_add(l, r)?); }
            Op::Sub => { let r = pop!(); let l = pop!(); push!(self.op_sub(l, r)?); }
            Op::Mul => { let r = pop!(); let l = pop!(); push!(self.op_mul(l, r)?); }
            Op::Div => { let r = pop!(); let l = pop!(); push!(self.op_div(l, r)?); }
            Op::IntDiv => { let r = pop!(); let l = pop!(); push!(self.op_intdiv(l, r)?); }
            Op::Mod => { let r = pop!(); let l = pop!(); push!(self.op_mod(l, r)?); }
            Op::Pow => { let r = pop!(); let l = pop!(); push!(self.op_pow(l, r)?); }
            Op::Neg => {
                let v = pop!();
                push!(match v {
                    Value::Int(n)   => Value::Int(-n),
                    Value::Float(f) => Value::Float(-f),
                    _ => return Err(VmError::TypeError(format!("unary '-' on {}", v.type_name()))),
                });
            }

            // ── Bitwise ───────────────────────────────────────────────────
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

            // ── Confronto ─────────────────────────────────────────────────
            Op::Eq => { let r = pop!(); let l = pop!(); push!(Value::Bool(l == r)); }
            Op::Ne => { let r = pop!(); let l = pop!(); push!(Value::Bool(l != r)); }
            Op::Lt => { let r = pop!(); let l = pop!(); push!(Value::Bool(l <  r)); }
            Op::Le => { let r = pop!(); let l = pop!(); push!(Value::Bool(l <= r)); }
            Op::Gt => { let r = pop!(); let l = pop!(); push!(Value::Bool(l >  r)); }
            Op::Ge => { let r = pop!(); let l = pop!(); push!(Value::Bool(l >= r)); }

            // ── Logica ────────────────────────────────────────────────────
            Op::Not => { let v = pop!(); push!(Value::Bool(!v.is_truthy())); }
            Op::JumpFalsePeek => {
                let offset = read_i16!();
                if !peek!().is_truthy() {
                    let ip = self.frames.last_mut().unwrap().ip;
                    self.frames.last_mut().unwrap().ip = (ip as isize + offset as isize) as usize;
                }
            }
            Op::JumpTruePeek => {
                let offset = read_i16!();
                if peek!().is_truthy() {
                    let ip = self.frames.last_mut().unwrap().ip;
                    self.frames.last_mut().unwrap().ip = (ip as isize + offset as isize) as usize;
                }
            }

            // ── Salti ─────────────────────────────────────────────────────
            Op::Jump => {
                let offset = read_i16!();
                let ip = self.frames.last_mut().unwrap().ip;
                self.frames.last_mut().unwrap().ip = (ip as isize + offset as isize) as usize;
            }
            Op::JumpFalse => {
                let offset = read_i16!();
                let v = pop!();
                if !v.is_truthy() {
                    let ip = self.frames.last_mut().unwrap().ip;
                    self.frames.last_mut().unwrap().ip = (ip as isize + offset as isize) as usize;
                }
            }
            Op::JumpTrue => {
                let offset = read_i16!();
                let v = pop!();
                if v.is_truthy() {
                    let ip = self.frames.last_mut().unwrap().ip;
                    self.frames.last_mut().unwrap().ip = (ip as isize + offset as isize) as usize;
                }
            }

            // ── Funzioni ──────────────────────────────────────────────────
            Op::MakeClosure => {
                let idx        = read_u16!() as usize;
                let n_upvalues = read_u8!() as usize;
                let proto      = Rc::new(chunk.fn_protos[idx].clone());
                let mut upvalues = Vec::with_capacity(n_upvalues);
                if n_upvalues > 0 {
                    let start = self.stack.len() - n_upvalues;
                    for v in self.stack.drain(start..) {
                        upvalues.push(Upvalue { value: v });
                    }
                }
                let closure = Closure { proto, upvalues };
                push!(Value::Closure(Rc::new(closure)));
            }

            Op::Call => {
                let argc = read_u8!() as usize;
                if self.frames.len() >= FRAMES_MAX {
                    return Err(VmError::StackOverflow);
                }
                let fn_idx = self.stack.len() - argc - 1;
                let callee = self.stack[fn_idx].clone();
                match callee {
                    Value::NativeFn(_, f) => {
                        let args: Vec<Value> = self.stack.drain(fn_idx..).skip(1).collect();
                        let result = f(&args).map_err(VmError::Generic)?;
                        push!(result);
                    }
                    Value::Closure(c) => {
                        let proto = &c.proto;
                        if argc < proto.arity || argc > proto.max_arity {
                            return Err(VmError::ArityMismatch {
                                name: proto.name.clone(),
                                       expected: proto.arity,
                                       got: argc,
                            });
                        }
                        let missing = proto.max_arity - argc;
                        for i in 0..missing {
                            let def_idx = proto.defaults.len().saturating_sub(missing - i);
                            let def = proto.defaults.get(def_idx).cloned().unwrap_or(Value::None);
                            push!(def);
                        }
                        let new_base = fn_idx + 1;
                        self.stack[fn_idx] = Value::None;
                        let frame = CallFrame {
                            chunk:    Rc::new(proto.chunk.clone()),
                            ip:       0,
                            base:     new_base,
                            name:     proto.name.clone(),
                            upvalues: c.upvalues.clone(),
                        };
                        self.frames.push(frame);
                        return Ok(None);
                    }
                    other => {
                        return Err(VmError::NotCallable(other.type_name().to_string()));
                    }
                }
            }

            Op::CallMethod => {
                let name_idx = read_u16!() as usize;
                let argc     = read_u8!() as usize;
                let name     = chunk.names[name_idx].clone();

                // Stack: [..., obj, arg0, ..., argN-1]
                let obj_idx = self.stack.len() - argc - 1;
                let obj     = self.stack[obj_idx].clone();

                // Recupera il metodo dall'istanza
                let method = match &obj {
                    Value::Instance(inst) => {
                        inst.borrow().fields.get(&name).cloned()
                        .ok_or_else(|| VmError::UnknownField {
                            type_name: inst.borrow().class_name.clone(),
                                    field: name.clone(),
                        })?
                    }
                    other => {
                        return Err(VmError::UnknownField {
                            type_name: other.type_name().to_string(),
                                   field: name.clone(),
                        });
                    }
                };

                // Inserisci self come primo argomento (prima degli altri args)
                self.stack.insert(obj_idx + 1, obj);

                match method {
                    Value::Closure(c) => {
                        let proto   = &c.proto;
                        // let real_argc = argc + 1; // +1 per self
                        // if real_argc < proto.arity || real_argc > proto.max_arity {
                        //     return Err(VmError::ArityMismatch {
                        //         name: proto.name.clone(),
                        //                expected: proto.arity,
                        //                got: real_argc,
                        //     });
                        // }
                        // let missing = proto.max_arity - real_argc;

                        // L'arietà non conta self — il check usa argc originale
                        if argc < proto.arity || argc > proto.max_arity {
                            return Err(VmError::ArityMismatch {
                                name: proto.name.clone(),
                                       expected: proto.arity,
                                       got: argc,
                            });
                        }
                        let missing = proto.max_arity - argc;

                        for i in 0..missing {
                            let def_idx = proto.defaults.len().saturating_sub(missing - i);
                            let def = proto.defaults.get(def_idx).cloned().unwrap_or(Value::None);
                            push!(def);
                        }
                        let new_base = obj_idx + 1;
                        self.stack[obj_idx] = Value::None; // placeholder
                        let frame = CallFrame {
                            chunk:    Rc::new(proto.chunk.clone()),
                            ip:       0,
                            base:     new_base,
                            name:     proto.name.clone(),
                            upvalues: c.upvalues.clone(),
                        };
                        self.frames.push(frame);
                        return Ok(None);
                    }
                    Value::NativeFn(_, f) => {
                        let args: Vec<Value> = self.stack.drain(obj_idx..).skip(1).collect();
                        let result = f(&args).map_err(VmError::Generic)?;
                        push!(result);
                    }
                    other => return Err(VmError::NotCallable(other.type_name().to_string())),
                }
            }

            Op::Return => {
                let result = pop!();
                let frame = self.frames.pop().unwrap();
                self.stack.truncate(frame.base - 1);
                let is_done = self.frames.is_empty();
                push!(result.clone());
                if is_done {
                    return Ok(Some(result));
                }
            }

            Op::ReturnNil => {
                let frame = self.frames.pop().unwrap();
                self.stack.truncate(frame.base - 1);
                push!(Value::None);
                if self.frames.is_empty() {
                    return Ok(Some(Value::None));
                }
            }

            // ── Collezioni ────────────────────────────────────────────────
            Op::MakeArray => {
                let count = read_u16!() as usize;
                let start = self.stack.len() - count;
                let items: Vec<Value> = self.stack.drain(start..).collect();
                push!(Value::array(items));
            }

            Op::GetIndex => {
                let idx_v = pop!();
                let obj   = pop!();
                push!(self.eval_index(obj, idx_v)?);
            }

            Op::SetIndex => {
                let val  = pop!();
                let idx_v = pop!();
                let obj   = pop!();
                match (obj, &idx_v) {
                    (Value::Array(arr), Value::Int(i)) => {
                        let len = arr.borrow().len();
                        let i = self.resolve_idx(*i, len)?;
                        arr.borrow_mut()[i] = val;
                    }
                    _ => return Err(VmError::TypeError("index assignment requires Array".into())),
                }
            }

            Op::MakeRange => {
                let inclusive = read_u8!() != 0;
                let end   = pop!();
                let start = pop!();
                match (&start, &end) {
                    (Value::Int(s), Value::Int(e)) => {
                        let v: Vec<Value> = if inclusive {
                            (*s..=*e).map(Value::Int).collect()
                        } else {
                            (*s..*e).map(Value::Int).collect()
                        };
                        push!(Value::array(v));
                    }
                    _ => return Err(VmError::TypeError("range bounds must be Int".into())),
                }
            }

            // ── Classi / istanze ──────────────────────────────────────────
            Op::GetField => {
                let idx  = read_u16!() as usize;
                let name = chunk.names[idx].clone();
                let obj  = pop!();
                push!(self.get_field(obj, &name)?);
            }

            Op::SetField => {
                let idx  = read_u16!() as usize;
                let name = chunk.names[idx].clone();
                let val  = pop!();
                let obj  = pop!();
                match obj {
                    Value::Instance(inst) => {
                        inst.borrow_mut().fields.insert(name, val);
                    }
                    _ => return Err(VmError::TypeError(format!("cannot set field on {}", obj.type_name()))),
                }
            }

            Op::MakeInstance => {
                let idx       = read_u16!() as usize;
                let class_name = chunk.names[idx].clone();
                let inst = Instance::new(&class_name);
                push!(Value::Instance(Rc::new(RefCell::new(inst))));
            }

            // ── Option / Result ───────────────────────────────────────────
            Op::MakeSome => { let v = pop!(); push!(Value::Some_(Box::new(v))); }
            Op::MakeOk   => { let v = pop!(); push!(Value::Ok_(Box::new(v)));   }
            Op::MakeErr  => { let v = pop!(); push!(Value::Err_(Box::new(v)));  }

            // ── Membership ────────────────────────────────────────────────
            Op::In => {
                let haystack = pop!();
                let needle   = pop!();
                push!(Value::Bool(self.eval_in(needle, haystack)?));
            }
            Op::NotIn => {
                let haystack = pop!();
                let needle   = pop!();
                push!(Value::Bool(!self.eval_in(needle, haystack)?));
            }
            Op::Is => {
                let r = pop!(); let l = pop!();
                push!(Value::Bool(std::mem::discriminant(&l) == std::mem::discriminant(&r)));
            }

            // ── Pattern matching helpers ───────────────────────────────────
            Op::IsSome => {
                let offset = read_i16!();
                if !matches!(peek!(), Value::Some_(_)) {
                    let ip = self.frames.last_mut().unwrap().ip;
                    self.frames.last_mut().unwrap().ip = (ip as isize + offset as isize) as usize;
                }
            }
            Op::IsNone => {
                let offset = read_i16!();
                if !matches!(peek!(), Value::None) {
                    let ip = self.frames.last_mut().unwrap().ip;
                    self.frames.last_mut().unwrap().ip = (ip as isize + offset as isize) as usize;
                }
            }
            Op::IsOk => {
                let offset = read_i16!();
                if !matches!(peek!(), Value::Ok_(_)) {
                    let ip = self.frames.last_mut().unwrap().ip;
                    self.frames.last_mut().unwrap().ip = (ip as isize + offset as isize) as usize;
                }
            }
            Op::IsErr => {
                let offset = read_i16!();
                if !matches!(peek!(), Value::Err_(_)) {
                    let ip = self.frames.last_mut().unwrap().ip;
                    self.frames.last_mut().unwrap().ip = (ip as isize + offset as isize) as usize;
                }
            }
            Op::Unwrap => {
                let v = pop!();
                let inner = match v {
                    Value::Some_(inner) | Value::Ok_(inner) | Value::Err_(inner) => *inner,
                    _ => return Err(VmError::TypeError(format!("cannot unwrap {}", v.type_name()))),
                };
                push!(inner);
            }
            Op::MatchLit => {
                let cidx   = read_u16!() as usize;
                let offset = read_i16!();
                let lit    = chunk.constants[cidx].clone();
                if peek!() != lit {
                    let ip = self.frames.last_mut().unwrap().ip;
                    self.frames.last_mut().unwrap().ip = (ip as isize + offset as isize) as usize;
                }
            }
            Op::MatchRange => {
                let lo_idx  = read_u16!() as usize;
                let hi_idx  = read_u16!() as usize;
                let incl    = read_u8!() != 0;
                let offset  = read_i16!();
                let lo = match &chunk.constants[lo_idx] { Value::Int(n) => *n, _ => return Err(VmError::TypeError("range pattern needs Int".into())) };
                let hi = match &chunk.constants[hi_idx] { Value::Int(n) => *n, _ => return Err(VmError::TypeError("range pattern needs Int".into())) };
                let matched = match peek!() {
                    Value::Int(n) => if incl { n >= lo && n <= hi } else { n >= lo && n < hi },
                    _ => false,
                };
                if !matched {
                    let ip = self.frames.last_mut().unwrap().ip;
                    self.frames.last_mut().unwrap().ip = (ip as isize + offset as isize) as usize;
                }
            }

            // ── Iterazione ────────────────────────────────────────────────
            Op::IntoIter => {
                let v = pop!();
                let arr = match v {
                    Value::Array(a) => a,
                    Value::Str(s)   => {
                        let chars: Vec<Value> = s.chars().map(|c| Value::str(c.to_string())).collect();
                        Rc::new(RefCell::new(chars))
                    }
                    _ => return Err(VmError::TypeError(format!("'{}' is not iterable", v.type_name()))),
                };
                // Push: __Iter sentinel (l'array + pos=0)
                // In realtà usiamo due local: l'array e la posizione
                // Pushiamo l'array e poi 0
                push!(Value::Array(arr));
            }

            Op::IterNext => {
                let iter_local = read_u8!() as usize;
                let var_local  = read_u8!() as usize;
                let offset     = read_i16!();
                // Leggi posizione dal pos_local (iter_local+1)
                let pos_local = iter_local + 1;
                let pos = match &self.stack[base + pos_local] {
                    Value::Int(n) => *n as usize,
                    _ => 0,
                };
                let arr_val = self.stack[base + iter_local].clone();
                let arr = match &arr_val {
                    Value::Array(a) => a.clone(),
                    _ => return Err(VmError::TypeError("IterNext on non-array".into())),
                };
                let len = arr.borrow().len();
                if pos >= len {
                    // Iterazione finita
                    let ip = self.frames.last_mut().unwrap().ip;
                    self.frames.last_mut().unwrap().ip = (ip as isize + offset as isize) as usize;
                } else {
                    // Aggiorna var_local con il valore corrente
                    let item = arr.borrow()[pos].clone();
                    self.stack[base + var_local] = item;
                    // Incrementa pos_local
                    self.stack[base + pos_local] = Value::Int((pos + 1) as i64);
                }
            }

            // ── F-string ──────────────────────────────────────────────────
            Op::BuildStr => {
                let n     = read_u16!() as usize;
                let start = self.stack.len() - n;
                let parts: Vec<String> = self.stack.drain(start..).map(|v| v.to_string()).collect();
                push!(Value::str(parts.join("")));
            }
            Op::ToStr => {
                let v = pop!();
                push!(Value::str(v.to_string()));
            }

            // ── Misc ──────────────────────────────────────────────────────
            Op::Nop  => {}
            Op::Halt => {
                let result = self.stack.pop().unwrap_or(Value::None);
                return Ok(Some(result));
            }
        }

        Ok(None)
    }

    // ── Operazioni aritmetiche ────────────────────────────────────────────

    fn op_add(&self, l: Value, r: Value) -> VmResult {
        match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a),   Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a + *b as f64)),
            (Value::Str(a),   Value::Str(b))   => Ok(Value::str(format!("{}{}", a, b))),
            _ => Err(VmError::TypeError(format!("'+' between {} and {}", l.type_name(), r.type_name()))),
        }
    }
    fn op_sub(&self, l: Value, r: Value) -> VmResult {
        match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a),   Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a - *b as f64)),
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
            _ => Err(VmError::TypeError(format!("'*' between {} and {}", l.type_name(), r.type_name()))),
        }
    }
    fn op_div(&self, l: Value, r: Value) -> VmResult {
        let b = r.as_float().ok_or_else(|| VmError::TypeError(format!("'/' on {}", r.type_name())))?;
        if b == 0.0 { return Err(VmError::DivisionByZero); }
        let a = l.as_float().ok_or_else(|| VmError::TypeError(format!("'/' on {}", l.type_name())))?;
        Ok(Value::Float(a / b))
    }
    fn op_intdiv(&self, l: Value, r: Value) -> VmResult {
        match (&l, &r) {
            (_, Value::Int(0)) => Err(VmError::DivisionByZero),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
            _ => {
                let b = r.as_float().unwrap();
                if b == 0.0 { return Err(VmError::DivisionByZero); }
                Ok(Value::Int((l.as_float().unwrap() / b).floor() as i64))
            }
        }
    }
    fn op_mod(&self, l: Value, r: Value) -> VmResult {
        match (&l, &r) {
            (_, Value::Int(0)) => Err(VmError::DivisionByZero),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
            _ => {
                let b = r.as_float().unwrap();
                if b == 0.0 { return Err(VmError::DivisionByZero); }
                Ok(Value::Float(l.as_float().unwrap() % b))
            }
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
                "&"  => a & b, "|" => a | b, "^" => a ^ b,
                "<<" => a << b, ">>" => a >> b, _ => unreachable!(),
            })),
            _ => Err(VmError::TypeError(format!("'{}' requires Int", op))),
        }
    }

    // ── Field access ──────────────────────────────────────────────────────

    pub fn get_field(&self, obj: Value, field: &str) -> VmResult {
        match &obj {
            Value::Instance(inst) => {
                if let Some(v) = inst.borrow().fields.get(field).cloned() {
                    return Ok(v);
                }
                Err(VmError::UnknownField {
                    type_name: inst.borrow().class_name.clone(),
                    field: field.to_string(),
                })
            }
            Value::Array(arr) => match field {
                "len" => Ok(Value::Int(arr.borrow().len() as i64)),
                _ => Err(VmError::UnknownField { type_name: "Array".into(), field: field.into() }),
            },
            Value::Str(s) => {
                let s = s.clone();
                match field {
                    "len"   => Ok(Value::Int(s.chars().count() as i64)),
                    "upper" => Ok(Value::NativeFn("upper".into(), {
                        let su = s.to_uppercase();
                        // Non possiamo catturare 's' direttamente con fn pointer
                        // usiamo un workaround: built-in che usa ToStr
                        |_: &[Value]| Ok(Value::None) // placeholder
                    })),
                    _ => Err(VmError::UnknownField { type_name: "Str".into(), field: field.into() }),
                }
            }
            _ => Err(VmError::UnknownField {
                type_name: obj.type_name().to_string(),
                field: field.to_string(),
            }),
        }
    }

    // ── Index access ──────────────────────────────────────────────────────

    fn eval_index(&self, obj: Value, idx: Value) -> VmResult {
        let i = match &idx { Value::Int(n) => *n, _ => return Err(VmError::TypeError("index must be Int".into())) };
        match obj {
            Value::Array(arr) => {
                let len = arr.borrow().len();
                let a = self.resolve_idx(i, len)?;
                Ok(arr.borrow()[a].clone())
            }
            Value::Str(s) => {
                let chars: Vec<char> = s.chars().collect();
                let a = self.resolve_idx(i, chars.len())?;
                Ok(Value::str(chars[a].to_string()))
            }
            _ => Err(VmError::TypeError(format!("cannot index {}", obj.type_name()))),
        }
    }

    fn resolve_idx(&self, i: i64, len: usize) -> VmResult<usize> {
        let a = if i < 0 { len as i64 + i } else { i };
        if a < 0 || a as usize >= len {
            Err(VmError::IndexOutOfBounds { index: i, len })
        } else {
            Ok(a as usize)
        }
    }

    // ── Membership ────────────────────────────────────────────────────────

    fn eval_in(&self, needle: Value, haystack: Value) -> VmResult<bool> {
        match haystack {
            Value::Array(arr) => Ok(arr.borrow().contains(&needle)),
            Value::Str(s)     => match &needle {
                Value::Str(n) => Ok(s.contains(n.as_str())),
                _             => Ok(false),
            },
            _ => Err(VmError::TypeError(format!("'in' on {}", haystack.type_name()))),
        }
    }
}

impl Default for Vm {
    fn default() -> Self { Self::new() }
}
