use std::rc::Rc;

use crate::opcode::Op;
use crate::value::Value;

/// Descrizione di un upvalue catturato da una closure.
#[derive(Debug, Clone)]
pub struct UpvalueDesc {
    /// true = il valore si trova in un local del frame padre
    /// false = il valore si trova in un upvalue del frame padre
    pub is_local: bool,
    pub index: u8,
}

/// Prototipo di funzione: tutto ciò che è noto a compile-time.
#[derive(Debug, Clone)]
pub struct FnProto {
    pub name: String,
    /// Numero di parametri obbligatori
    pub arity: usize,
    /// Numero totale di parametri (inclusi quelli con default)
    pub max_arity: usize,
    /// Bytecode (Rc evita il clone dell'intero chunk ad ogni call — v0.2.14)
    pub chunk: Rc<Chunk>,
    /// Upvalue catturati
    pub upvalues: Vec<UpvalueDesc>,
    /// Indici nelle costanti del chunk per i valori di default
    pub defaults: Vec<Value>,
    /// true se async (stub in v0.2.0)
    pub is_async: bool,
}

/// Un Chunk contiene il bytecode e tutti i dati associati
/// a una singola unità di compilazione (funzione o script top-level).
#[derive(Debug, Clone, Default)]
pub struct Chunk {
    /// Bytecode grezzo
    pub code: Vec<u8>,
    /// Pool di costanti (Int, Float, Bool, Str, None, FnProto)
    pub constants: Vec<Value>,
    /// Pool di nomi (stringhe per globali e campi)
    pub names: Vec<String>,
    /// Prototipi di funzione definiti in questo chunk
    pub fn_protos: Vec<FnProto>,
    /// Mappa offset→riga sorgente per debug/error reporting
    lines: Vec<(usize, u32)>, // (offset_start, line)
    /// Indice per deduplicazione O(1) in add_const (solo scalari: Int/Float/Bool/Str/None)
    const_index: std::collections::HashMap<ConstKey, u16>,
}

/// Chiave hashable per valori scalari nel pool costanti.
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
enum ConstKey {
    Int(i64),
    // Float serializzato come bits per Eq/Hash
    Float(u64),
    Bool(bool),
    Str(String),
    None,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Emit ──────────────────────────────────────────────────────────────

    pub fn emit(&mut self, op: Op, line: u32) -> usize {
        let offset = self.code.len();
        self.code.push(op as u8);
        match self.lines.last() {
            Some(&(_, l)) if l == line => {}
            _ => self.lines.push((offset, line)),
        }
        offset
    }

    pub fn emit_u8(&mut self, b: u8) {
        self.code.push(b);
    }

    pub fn emit_u16(&mut self, v: u16) {
        self.code.push((v & 0xFF) as u8);
        self.code.push((v >> 8) as u8);
    }

    pub fn emit_i16(&mut self, v: i16) {
        self.emit_u16(v as u16);
    }

    /// Emette un Jump con offset placeholder (0), restituisce l'offset del placeholder.
    pub fn emit_jump(&mut self, op: Op, line: u32) -> usize {
        self.emit(op, line);
        let patch = self.code.len();
        self.emit_i16(0);
        patch
    }

    /// Fa il patch di un jump precedentemente emesso con emit_jump.
    pub fn patch_jump(&mut self, patch: usize) {
        let offset = self.code.len() as isize - (patch as isize + 2);
        let v = offset as i16;
        self.code[patch]     = (v as u16 & 0xFF) as u8;
        self.code[patch + 1] = (v as u16 >> 8) as u8;
    }

    /// Emette un loop-back jump (offset negativo verso `loop_start`).
    pub fn emit_loop(&mut self, loop_start: usize, line: u32) {
        self.emit(Op::Jump, line);
        let offset = loop_start as isize - self.code.len() as isize - 2;
        self.emit_i16(offset as i16);
    }

    // ── Pool di costanti ──────────────────────────────────────────────────

    /// Peephole optimizer (v0.2.20) — sostituisce sequenze ridondanti a lunghezza invariata.
    ///
    /// Ottimizzazioni applicate (tutte preservano la lunghezza del bytecode per sicurezza dei salti):
    ///   Not + Not → Nop + Nop     — doppia negazione logica (rimpiazza con noop locale)
    ///   Neg + Neg → Nop + Nop     — doppio negativo numerico
    ///
    /// Nota: le eliminazioni di istruzioni (Const+Pop ecc.) richiedono un remap completo
    /// degli offset di salto — implementate nel constant folding a livello AST invece.
    /// Peephole optimizer: sostituisce pattern ridondanti con Nop.
    /// Sicuro: non sposta byte, gli offset di salto restano validi.
    pub fn peephole_optimize(&mut self) {
        let code = &mut self.code;
        let len = code.len();
        let mut i = 0usize;
        while i < len {
            let Some(op0) = Op::from_u8(code[i]) else { i += 1; continue; };
            // SetTraits ha lunghezza variabile: [u8 count] + count*[u16 name_idx]
            // Non ottimizzabile — saltiamo tutta l'istruzione
            if op0 == Op::SetTraits {
                let count = if i + 1 < len { code[i + 1] as usize } else { 0 };
                i += 1 + 1 + count * 2;
                continue;
            }
            let op0_size = 1 + op0.operand_bytes();
            let next = i + op0_size;
            if next < len {
                let Some(op1) = Op::from_u8(code[next]) else { i += op0_size; continue; };
                // Saltiamo anche se op1 è SetTraits
                if op1 == Op::SetTraits { i += op0_size; continue; }
                let op1_size = 1 + op1.operand_bytes();
                match (op0, op1) {
                    // Const [u16] Pop -> Nop x3 + Nop
                    (Op::Const, Op::Pop) => {
                        code[i] = Op::Nop as u8; code[i+1] = Op::Nop as u8; code[i+2] = Op::Nop as u8;
                        code[next] = Op::Nop as u8;
                        i += op0_size + op1_size; continue;
                    }
                    // True/False/Nil + Pop -> Nop Nop
                    (Op::True | Op::False | Op::Nil, Op::Pop) => {
                        code[i] = Op::Nop as u8; code[next] = Op::Nop as u8;
                        i += op0_size + op1_size; continue;
                    }
                    // Not Not -> Nop Nop
                    (Op::Not, Op::Not) => {
                        code[i] = Op::Nop as u8; code[next] = Op::Nop as u8;
                        i += op0_size + op1_size; continue;
                    }
                    // Jump offset=0 -> Nop x3 (jump a se stesso)
                    (Op::Jump, _) if i + 2 < len => {
                        let offset = i16::from_le_bytes([code[i+1], code[i+2]]);
                        if offset == 0 {
                            code[i] = Op::Nop as u8; code[i+1] = Op::Nop as u8; code[i+2] = Op::Nop as u8;
                        }
                    }
                    _ => {}
                }
            }
            i += op0_size;
        }
    }
    /// Aggiunge una costante al pool, restituisce l'indice.
    /// Deduplicazione semplice per Int/Bool/None/Str.
    pub fn add_const(&mut self, v: Value) -> u16 {
        // Deduplicazione O(1) per scalari via HashMap
        let key = match &v {
            Value::Int(n)   => Some(ConstKey::Int(*n)),
            Value::Float(f) => Some(ConstKey::Float(f.to_bits())),
            Value::Bool(b)  => Some(ConstKey::Bool(*b)),
            Value::Str(s)   => Some(ConstKey::Str(s.as_ref().clone())),
            Value::None     => Some(ConstKey::None),
            _               => None,
        };
        if let Some(k) = key {
            if let Some(&idx) = self.const_index.get(&k) { return idx; }
            let idx = self.constants.len() as u16;
            self.const_index.insert(k, idx);
            self.constants.push(v);
            return idx;
        }
        // FnProto e altri tipi non deduplicati: aggiungi sempre
        let idx = self.constants.len() as u16;
        self.constants.push(v);
        idx
    }

    /// Aggiunge un nome al pool, restituisce l'indice.
    pub fn add_name(&mut self, name: &str) -> u16 {
        if let Some(i) = self.names.iter().position(|n| n == name) {
            return i as u16;
        }
        let idx = self.names.len() as u16;
        self.names.push(name.to_string());
        idx
    }

    /// Aggiunge un FnProto al pool, restituisce l'indice.
    pub fn add_fn_proto(&mut self, proto: FnProto) -> u16 {
        let idx = self.fn_protos.len() as u16;
        self.fn_protos.push(proto);
        idx
    }

    // ── Debug ─────────────────────────────────────────────────────────────

    pub fn line_at(&self, offset: usize) -> u32 {
        let mut current = 0u32;
        for &(start, line) in &self.lines {
            if start > offset { break; }
            current = line;
        }
        current
    }

    pub fn disassemble(&self, name: &str) -> String {
        let mut out = format!("=== {} ===\n", name);
        let mut i = 0;
        while i < self.code.len() {
            let byte = self.code[i];
            let op = Op::from_u8(byte).unwrap_or_else(|| panic!("bad opcode {}", byte));
            let line = self.line_at(i);
            out.push_str(&format!("{:04}  {:3}  {:16}", i, line, format!("{:?}", op)));

            match op {
                Op::Const => {
                    let idx = read_u16(&self.code, i + 1);
                    out.push_str(&format!("  #{} {:?}", idx,
                        self.constants.get(idx as usize).map(|v| format!("{}", v)).unwrap_or("?".into())));
                }
                Op::LoadLocal | Op::StoreLocal | Op::LoadUpval | Op::StoreUpval | Op::Call | Op::PopN => {
                    out.push_str(&format!("  {}", self.code[i + 1]));
                }
                Op::LoadGlobal | Op::StoreGlobal | Op::GetField | Op::SetField
                | Op::MakeInstance | Op::MakeClosure | Op::MakeArray | Op::BuildStr => {
                    let idx = read_u16(&self.code, i + 1);
                    let name = if matches!(op, Op::GetField | Op::SetField | Op::MakeInstance | Op::LoadGlobal | Op::StoreGlobal) {
                        self.names.get(idx as usize).cloned().unwrap_or_default()
                    } else if matches!(op, Op::MakeClosure) {
                        self.fn_protos.get(idx as usize).map(|p| p.name.clone()).unwrap_or_default()
                    } else { String::new() };
                    out.push_str(&format!("  #{} {}", idx, name));
                }
                Op::DefGlobal => {
                    let idx = read_u16(&self.code, i + 1);
                    let mut_ = self.code[i + 3];
                    let name = self.names.get(idx as usize).cloned().unwrap_or_default();
                    out.push_str(&format!("  #{} {} mut={}", idx, name, mut_));
                }
                Op::Jump | Op::JumpFalse | Op::JumpTrue | Op::JumpFalsePeek | Op::JumpTruePeek
                | Op::IsSome | Op::IsNone | Op::IsOk | Op::IsErr => {
                    let offset = read_i16(&self.code, i + 1);
                    let target = (i as isize + 3 + offset as isize) as usize;
                    out.push_str(&format!("  {:+} → {}", offset, target));
                }
                Op::MakeRange => {
                    out.push_str(&format!("  incl={}", self.code[i + 1]));
                }
                Op::IterNext => {
                    let iter_l = self.code[i + 1];
                    let var_l  = self.code[i + 2];
                    let jmp    = read_i16(&self.code, i + 3);
                    out.push_str(&format!("  iter={} var={} done→{:+}", iter_l, var_l, jmp));
                }
                Op::MatchLit => {
                    let cidx = read_u16(&self.code, i + 1);
                    let jmp  = read_i16(&self.code, i + 3);
                    out.push_str(&format!("  #{} {:+}", cidx, jmp));
                }
                Op::MatchRange => {
                    let lo   = read_u16(&self.code, i + 1);
                    let hi   = read_u16(&self.code, i + 3);
                    let incl = self.code[i + 5];
                    let jmp  = read_i16(&self.code, i + 6);
                    out.push_str(&format!("  lo=#{} hi=#{} incl={} {:+}", lo, hi, incl, jmp));
                }
                _ => {}
            }
            out.push('\n');
            // SetTraits ha lunghezza variabile: 1 opcode + 1 count + count*2 name_idx
            if op == Op::SetTraits {
                let count = if i + 1 < self.code.len() { self.code[i + 1] as usize } else { 0 };
                i += 1 + 1 + count * 2;
            } else {
                i += 1 + op.operand_bytes();
            }
        }
        out
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

pub fn read_u16(code: &[u8], offset: usize) -> u16 {
    code[offset] as u16 | ((code[offset + 1] as u16) << 8)
}

pub fn read_i16(code: &[u8], offset: usize) -> i16 {
    read_u16(code, offset) as i16
}

fn values_equal(a: &Value, b: &Value) -> bool {
    use Value::*;
    match (a, b) {
        (Int(x),  Int(y))  => x == y,
        (Float(x),Float(y))=> x == y,
        (Bool(x), Bool(y)) => x == y,
        (Str(x),  Str(y))  => x == y,
        (None,    None)    => true,
        _ => false,
    }
}
