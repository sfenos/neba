use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use indexmap::{Equivalent, IndexMap};

use crate::chunk::FnProto;

// ── Tipi heap-allocated (gestiti da Rc = GC v0.2.0) ──────────────────────

pub type RcArray    = Rc<RefCell<Vec<Value>>>;
pub type RcInstance = Rc<RefCell<Instance>>;
pub type RcClosure  = Rc<Closure>;
/// Dict: mappa chiave→valore con ordine di inserimento preservato (IndexMap O(1) lookup).
pub type RcDict     = Rc<RefCell<IndexMap<Value, Value>>>;

// ── TypedArray (v0.2.6) ───────────────────────────────────────────────────

/// Dtype di un TypedArray — determina la rappresentazione interna compatta.
#[derive(Debug, Clone, PartialEq)]
pub enum Dtype {
    Float64,
    Float32,
    Int64,
    Int32,
}

impl Dtype {
    pub fn name(&self) -> &'static str {
        match self {
            Dtype::Float64 => "Float64",
            Dtype::Float32 => "Float32",
            Dtype::Int64   => "Int64",
            Dtype::Int32   => "Int32",
        }
    }
    pub fn array_type_name(&self) -> &'static str {
        match self {
            Dtype::Float64 => "Float64Array",
            Dtype::Float32 => "Float32Array",
            Dtype::Int64   => "Int64Array",
            Dtype::Int32   => "Int32Array",
        }
    }
}

/// Contenuto compatto di un TypedArray — usa Vec<T> nativo per efficienza.
#[derive(Debug, Clone)]
pub enum TypedArrayData {
    Float64(Vec<f64>),
    Float32(Vec<f32>),
    Int64(Vec<i64>),
    Int32(Vec<i32>),
}

impl TypedArrayData {
    pub fn dtype(&self) -> Dtype {
        match self {
            TypedArrayData::Float64(_) => Dtype::Float64,
            TypedArrayData::Float32(_) => Dtype::Float32,
            TypedArrayData::Int64(_)   => Dtype::Int64,
            TypedArrayData::Int32(_)   => Dtype::Int32,
        }
    }
    pub fn len(&self) -> usize {
        match self {
            TypedArrayData::Float64(v) => v.len(),
            TypedArrayData::Float32(v) => v.len(),
            TypedArrayData::Int64(v)   => v.len(),
            TypedArrayData::Int32(v)   => v.len(),
        }
    }
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Legge l'elemento all'indice i come Value.
    pub fn get(&self, i: usize) -> Option<Value> {
        match self {
            TypedArrayData::Float64(v) => v.get(i).map(|&x| Value::Float(x)),
            TypedArrayData::Float32(v) => v.get(i).map(|&x| Value::Float(x as f64)),
            TypedArrayData::Int64(v)   => v.get(i).map(|&x| Value::Int(x)),
            TypedArrayData::Int32(v)   => v.get(i).map(|&x| Value::Int(x as i64)),
        }
    }

    /// Scrive il valore v all'indice i (conversione automatica al dtype).
    pub fn set(&mut self, i: usize, v: Value) -> Result<(), String> {
        match self {
            TypedArrayData::Float64(arr) => {
                let x = v.as_float().ok_or_else(|| format!("cannot store {} in Float64Array", v.type_name()))?;
                if i < arr.len() { arr[i] = x; Ok(()) } else { Err(format!("index {} out of bounds", i)) }
            }
            TypedArrayData::Float32(arr) => {
                let x = v.as_float().ok_or_else(|| format!("cannot store {} in Float32Array", v.type_name()))? as f32;
                if i < arr.len() { arr[i] = x; Ok(()) } else { Err(format!("index {} out of bounds", i)) }
            }
            TypedArrayData::Int64(arr) => {
                let x = match v { Value::Int(n) => n, Value::Float(f) => f as i64,
                    _ => return Err(format!("cannot store {} in Int64Array", v.type_name())) };
                if i < arr.len() { arr[i] = x; Ok(()) } else { Err(format!("index {} out of bounds", i)) }
            }
            TypedArrayData::Int32(arr) => {
                let x = match v { Value::Int(n) => n as i32, Value::Float(f) => f as i32,
                    _ => return Err(format!("cannot store {} in Int32Array", v.type_name())) };
                if i < arr.len() { arr[i] = x; Ok(()) } else { Err(format!("index {} out of bounds", i)) }
            }
        }
    }

    /// Estrae gli elementi agli indici in `indices` come nuovo TypedArray (slicing).
    pub fn slice_indices(&self, indices: &[i64], len: usize) -> Result<TypedArrayData, String> {
        match self {
            TypedArrayData::Float64(v) => {
                let mut out = Vec::with_capacity(indices.len());
                for &i in indices { out.push(v[resolve_idx(i, len)?]); }
                Ok(TypedArrayData::Float64(out))
            }
            TypedArrayData::Float32(v) => {
                let mut out = Vec::with_capacity(indices.len());
                for &i in indices { out.push(v[resolve_idx(i, len)?]); }
                Ok(TypedArrayData::Float32(out))
            }
            TypedArrayData::Int64(v) => {
                let mut out = Vec::with_capacity(indices.len());
                for &i in indices { out.push(v[resolve_idx(i, len)?]); }
                Ok(TypedArrayData::Int64(out))
            }
            TypedArrayData::Int32(v) => {
                let mut out = Vec::with_capacity(indices.len());
                for &i in indices { out.push(v[resolve_idx(i, len)?]); }
                Ok(TypedArrayData::Int32(out))
            }
        }
    }
}

/// Risolve un indice (anche negativo) rispetto alla lunghezza dell'array.
pub fn resolve_idx(i: i64, len: usize) -> Result<usize, String> {
    let a = if i < 0 { len as i64 + i } else { i };
    if a < 0 || a as usize >= len {
        Err(format!("index {} out of bounds (len={})", i, len))
    } else {
        Ok(a as usize)
    }
}

pub type RcTypedArray = Rc<RefCell<TypedArrayData>>;

/// Closure = FnProto + upvalue catturati al momento della definizione.
#[derive(Clone)]
pub struct Closure {
    pub proto: Rc<FnProto>,
    /// Rc<Vec> invece di Vec: clone O(1) invece di O(n_upvalues) a ogni chiamata
    pub upvalues: Rc<Vec<Upvalue>>,
}

impl fmt::Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Closure({})", self.proto.name)
    }
}

/// Un upvalue è un valore catturato da un frame padre.
/// Usa `Rc<RefCell<Value>>` per condividere la cella tra closure e CallFrame:
/// `StoreUpval` scrive nella cella condivisa e le mutazioni persistono tra chiamate.
#[derive(Debug, Clone)]
pub struct Upvalue {
    pub value: Rc<RefCell<Value>>,
}

/// Funzione nativa (built-in).
pub type NativeFn = fn(&[Value]) -> Result<Value, String>;

/// Istanza di una classe.
#[derive(Debug, Clone)]
pub struct Instance {
    pub class_name: String,
    pub traits: Vec<String>,
    pub fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(class_name: impl Into<String>) -> Self {
        Instance { class_name: class_name.into(), traits: Vec::new(), fields: HashMap::new() }
    }
}

// ── Value ─────────────────────────────────────────────────────────────────

/// Tipo runtime di Neba (VM v0.2.0).
/// I tipi primitivi sono inline; i tipi heap-allocated usano Rc per v0.2.0.
/// In v0.2.1 Rc verrà sostituito da puntatori GC mark-and-sweep.
#[derive(Clone)]
pub enum Value {
    // Primitivi (no allocation)
    Int(i64),
    Float(f64),
    Bool(bool),
    None,

    // Heap (Rc = reference counting GC)
    Str(Rc<String>),
    Array(RcArray),
    Dict(RcDict),
    /// TypedArray compatto — Float64, Float32, Int64, Int32 (v0.2.6)
    TypedArray(RcTypedArray),
    /// NdArray multidimensionale (v0.2.25) — layout flat row-major
    NdArray(RcNdArray),
    Closure(RcClosure),
    /// Funzione nativa — nome come Rc<String> per ridurre la dimensione di Value (8 byte vs 24)
    NativeFn(Rc<String>, NativeFn),

    // Option / Result
    Some_(Box<Value>),
    Ok_(Box<Value>),
    Err_(Box<Value>),

    // Istanza di classe
    Instance(RcInstance),

    // Sentinel interni (mai esposti all'utente)
    #[doc(hidden)] __Return(Box<Value>),
    #[doc(hidden)] __Break,
    #[doc(hidden)] __Continue,
    #[doc(hidden)] __Iter(RcArray, usize),   // array + posizione corrente
    /// Range intero lazy — evita l'allocazione di Vec<Value> per i for loop (v0.2.14)
    #[doc(hidden)] IntRange(i64, i64, bool), // start, end, inclusive
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n)       => write!(f, "Int({})", n),
            Value::Float(n)     => write!(f, "Float({})", n),
            Value::Bool(b)      => write!(f, "Bool({})", b),
            Value::None         => write!(f, "None"),
            Value::Str(s)       => write!(f, "Str({:?})", s),
            Value::Array(a)     => write!(f, "Array({:?})", a.borrow()),
            Value::Dict(d)      => {
                let items: Vec<String> = d.borrow().iter().map(|(k,v)| format!("{:?}: {:?}", k, v)).collect();
                write!(f, "Dict{{{}}}", items.join(", "))
            }
            Value::TypedArray(t) => {
                let d = t.borrow();
                write!(f, "{}[{}]", d.dtype().name(), d.len())
            }
            Value::NdArray(nd) => {
                let bnd = nd.borrow();
                let dtype_name = bnd.data.borrow().dtype().name().to_string();
                write!(f, "NdArray(shape={:?}, dtype={})", bnd.shape, dtype_name)
            }
            Value::Closure(c)   => write!(f, "Closure({})", c.proto.name),
            Value::NativeFn(n,_)=> write!(f, "NativeFn({})", n),
            Value::Some_(v)     => write!(f, "Some({:?})", v),
            Value::Ok_(v)       => write!(f, "Ok({:?})", v),
            Value::Err_(v)      => write!(f, "Err({:?})", v),
            Value::Instance(i)  => write!(f, "Instance({})", i.borrow().class_name),
            Value::__Return(v)  => write!(f, "__Return({:?})", v),
            Value::__Break      => write!(f, "__Break"),
            Value::__Continue   => write!(f, "__Continue"),
            Value::__Iter(a, i) => write!(f, "__Iter(pos={})", i),
            Value::IntRange(s, e, inc) => write!(f, "IntRange({}..{}{})", s, e, if *inc {"="} else {""}),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n)    => write!(f, "{}", n),
            Value::Float(n)  => {
                if n.fract() == 0.0 { write!(f, "{:.1}", n) } else { write!(f, "{}", n) }
            }
            Value::Bool(b)   => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::None      => write!(f, "None"),
            Value::Str(s)    => write!(f, "{}", s),
            Value::Array(a)  => {
                let items: Vec<String> = a.borrow().iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Dict(d)   => {
                let items: Vec<String> = d.borrow().iter()
                    .map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            Value::TypedArray(t) => {
                let d = t.borrow();
                let dtype = d.dtype().name();
                let items: Vec<String> = (0..d.len())
                    .map(|i| d.get(i).unwrap().to_string())
                    .collect();
                write!(f, "{}[{}]", dtype, items.join(", "))
            }
            Value::NdArray(nd) => {
                fn fmt_nd(nd: &NdArray, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    if nd.ndim() == 1 {
                        let items: Vec<String> = (0..nd.size()).map(|i| nd.get_flat(i).unwrap().to_string()).collect();
                        write!(f, "[{}]", items.join(", "))
                    } else {
                        write!(f, "[")?;
                        for i in 0..nd.shape[0] {
                            if i > 0 { write!(f, ", ")?; }
                            match nd.get_axis0(i).unwrap() {
                                Value::NdArray(sub) => fmt_nd(&sub.borrow(), f)?,
                                v => write!(f, "{}", v)?,
                            }
                        }
                        write!(f, "]")
                    }
                }
                fmt_nd(&nd.borrow(), f)
            }
            Value::Closure(c) => write!(f, "<fn {}>", c.proto.name),
            Value::NativeFn(n, _) => write!(f, "<built-in {}>", n),
            Value::Some_(v)  => write!(f, "Some({})", v),
            Value::Ok_(v)    => write!(f, "Ok({})", v),
            Value::Err_(v)   => write!(f, "Err({})", v),
            Value::Instance(i) => write!(f, "<{} instance>", i.borrow().class_name),
            Value::__Return(v) => write!(f, "{}", v),
            Value::__Break     => write!(f, "<break>"),
            Value::__Continue  => write!(f, "<continue>"),
            Value::__Iter(_, pos) => write!(f, "<iter pos={}>", pos),
            Value::IntRange(s, e, inc) => write!(f, "{}..{}{}", s, if *inc {"="} else {""}, e),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a),   Value::Int(b))   => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a),   Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Int(b))   => *a == (*b as f64),
            (Value::Bool(a),  Value::Bool(b))  => a == b,
            (Value::Str(a),   Value::Str(b))   => a == b,
            (Value::None,     Value::None)     => true,
            (Value::Some_(a), Value::Some_(b)) => a == b,
            (Value::Ok_(a),   Value::Ok_(b))   => a == b,
            (Value::Err_(a),  Value::Err_(b))  => a == b,
            (Value::Array(a), Value::Array(b)) => *a.borrow() == *b.borrow(),
            (Value::Dict(a),  Value::Dict(b))  => *a.borrow() == *b.borrow(),
            // TypedArray: uguaglianza per identità (stesso Rc)
            (Value::TypedArray(a), Value::TypedArray(b)) => Rc::ptr_eq(a, b),
            (Value::NdArray(a),    Value::NdArray(b))    => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

/// Value come chiave di HashMap/IndexMap — necessario per Dict O(1).
/// Float viene hashato come bit pattern (NaN ≠ NaN è accettabile per chiavi di dict).
impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            // Str: hasha solo il contenuto (senza discriminant) per compatibilità
            // con Equivalent<Value> for str — hash("foo") == hash(Value::Str("foo"))
            Value::Str(s)    => { s.hash(state); }
            Value::Int(n)    => { 0u8.hash(state); n.hash(state); }
            Value::Float(f)  => { 1u8.hash(state); f.to_bits().hash(state); }
            Value::Bool(b)   => { 2u8.hash(state); b.hash(state); }
            Value::None      => { 4u8.hash(state); }
            // Tipi non scalari: hash per identità del puntatore
            Value::Array(a)  => { 5u8.hash(state); Rc::as_ptr(a).hash(state); }
            Value::Dict(d)   => { 6u8.hash(state); Rc::as_ptr(d).hash(state); }
            Value::Instance(i) => { 7u8.hash(state); Rc::as_ptr(i).hash(state); }
            Value::Closure(c)  => { 8u8.hash(state); Rc::as_ptr(c).hash(state); }
            Value::TypedArray(t) => { 9u8.hash(state); Rc::as_ptr(t).hash(state); }
            Value::NdArray(nd)   => { 19u8.hash(state); Rc::as_ptr(nd).hash(state); }
            Value::Some_(v)  => { 10u8.hash(state); v.hash(state); }
            Value::Ok_(v)    => { 11u8.hash(state); v.hash(state); }
            Value::Err_(v)   => { 12u8.hash(state); v.hash(state); }
            Value::NativeFn(n, _) => { 13u8.hash(state); n.hash(state); }
            Value::IntRange(s, e, i) => { 14u8.hash(state); s.hash(state); e.hash(state); i.hash(state); }
            Value::__Iter(a, i) => { 15u8.hash(state); Rc::as_ptr(a).hash(state); i.hash(state); }
            Value::__Return(v)  => { 16u8.hash(state); v.hash(state); }
            Value::__Break      => { 17u8.hash(state); }
            Value::__Continue   => { 18u8.hash(state); }
        }
    }
}

/// Permette `dict.get("key")` senza allocare `Value::Str` — zero-alloc lookup per moduli.
/// Usato in VM GetField quando la chiave è un nome di campo noto a compile time.
impl Equivalent<Value> for str {
    fn equivalent(&self, key: &Value) -> bool {
        match key {
            Value::Str(s) => s.as_str() == self,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Int(a),   Value::Int(b))   => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Int(a),   Value::Float(b)) => (*a as f64).partial_cmp(b),
            (Value::Float(a), Value::Int(b))   => a.partial_cmp(&(*b as f64)),
            (Value::Str(a),   Value::Str(b))   => a.as_str().partial_cmp(b.as_str()),
            _ => Option::None,
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b)  => *b,
            Value::Int(n)   => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s)   => !s.is_empty(),
            Value::None     => false,
            Value::Some_(_) => true,
            Value::Array(a) => !a.borrow().is_empty(),
            Value::Dict(d)  => !d.borrow().is_empty(),
            Value::TypedArray(t) => !t.borrow().is_empty(),
            Value::NdArray(nd)   => nd.borrow().size() > 0,
            _               => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_)       => "Int",
            Value::Float(_)     => "Float",
            Value::Bool(_)      => "Bool",
            Value::None         => "None",
            Value::Str(_)       => "Str",
            Value::Array(_)     => "Array",
            Value::Dict(_)      => "Dict",
            Value::TypedArray(t) => t.borrow().dtype().array_type_name(),
            Value::NdArray(_)    => "NdArray",
            Value::Closure(_)   => "Function",
            Value::NativeFn(_,_)=> "NativeFunction",
            Value::Some_(_)     => "Some",
            Value::Ok_(_)       => "Ok",
            Value::Err_(_)      => "Err",
            Value::Instance(_)  => "Instance",
            _                   => "<internal>",
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Int(n)   => Some(*n as f64),
            Value::Float(f) => Some(*f),
            _ => Option::None,
        }
    }

    /// Costruisce un Value::Str da un &str
    pub fn str(s: impl Into<String>) -> Self {
        Value::Str(Rc::new(s.into()))
    }

    /// Costruisce un Value::NativeFn — usa Rc<String> per il nome (riduce size di Value)
    pub fn native_fn(name: impl Into<String>, f: NativeFn) -> Self {
        Value::NativeFn(Rc::new(name.into()), f)
    }

    /// Costruisce un Value::Array da un Vec<Value>
    pub fn array(v: Vec<Value>) -> Self {
        Value::Array(Rc::new(RefCell::new(v)))
    }

    /// Costruisce un Value::Dict da una lista di coppie (chiave, valore).
    /// Preserva l'ordine di inserimento; in caso di chiavi duplicate l'ultima vince.
    pub fn dict(pairs: Vec<(Value, Value)>) -> Self {
        let mut map = IndexMap::with_capacity(pairs.len());
        for (k, v) in pairs { map.insert(k, v); }
        Value::Dict(Rc::new(RefCell::new(map)))
    }

    /// Costruisce un Value::Dict direttamente da un IndexMap.
    pub fn dict_from_map(map: IndexMap<Value, Value>) -> Self {
        Value::Dict(Rc::new(RefCell::new(map)))
    }

    /// Costruisce un Value::TypedArray da TypedArrayData
    pub fn typed_array(data: TypedArrayData) -> Self {
        Value::TypedArray(Rc::new(RefCell::new(data)))
    }

    /// Costruisce un Value::NdArray da NdArray
    pub fn nd_array(nd: NdArray) -> Self {
        Value::NdArray(Rc::new(RefCell::new(nd)))
    }
}




// ── NdArray — array multidimensionale (v0.2.25) ───────────────────────────

/// Array multidimensionale con layout flat (row-major, come NumPy).
/// data è Rc<RefCell<TypedArrayData>> condiviso tra array e view.
/// offset = primo indice valido nel buffer condiviso (0 per owner, >0 per view).
#[derive(Debug, Clone)]
pub struct NdArray {
    pub shape:  Vec<usize>,
    pub offset: usize,
    pub data:   Rc<RefCell<TypedArrayData>>,
}

impl NdArray {
    // ── Costruttori ───────────────────────────────────────────────────────

    /// Crea un NdArray di zeri con forma e dtype dati.
    pub fn zeros(shape: Vec<usize>, dtype: Dtype) -> Self {
        let n = shape.iter().product::<usize>().max(1);
        let data = match dtype {
            Dtype::Float64 => TypedArrayData::Float64(vec![0.0f64; n]),
            Dtype::Float32 => TypedArrayData::Float32(vec![0.0f32; n]),
            Dtype::Int64   => TypedArrayData::Int64(vec![0i64; n]),
            Dtype::Int32   => TypedArrayData::Int32(vec![0i32; n]),
        };
        NdArray { shape, offset: 0, data: Rc::new(RefCell::new(data)) }
    }

    /// Crea da dati flat già esistenti (owner, offset=0).
    pub fn from_flat(data: TypedArrayData, shape: Vec<usize>) -> Result<Self, String> {
        let expected: usize = shape.iter().product();
        if data.len() != expected {
            return Err(format!("shape {:?} requires {} elements, got {}", shape, expected, data.len()));
        }
        Ok(NdArray { shape, offset: 0, data: Rc::new(RefCell::new(data)) })
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    pub fn ndim(&self) -> usize { self.shape.len() }
    pub fn size(&self) -> usize { self.shape.iter().product() }

    /// Indice flat a partire dagli indici N-D (row-major, offset incluso).
    pub fn flat_index(&self, indices: &[usize]) -> Result<usize, String> {
        if indices.len() != self.shape.len() {
            return Err(format!("expected {} indices, got {}", self.shape.len(), indices.len()));
        }
        let mut fi = self.offset;
        // Calcoliamo le strides sul buffer completo (che ha shape.iter().product() elementi dopo offset)
        // ma per un owner contiguo i strides sono semplicemente shape[1]*shape[2]*...
        let mut stride = 1usize;
        let mut strides = vec![1usize; self.shape.len()];
        for d in (0..self.shape.len()).rev() {
            strides[d] = stride;
            stride *= self.shape[d];
        }
        for d in 0..self.shape.len() {
            if indices[d] >= self.shape[d] {
                return Err(format!("index {} out of bounds for dim {} (size {})", indices[d], d, self.shape[d]));
            }
            fi += indices[d] * strides[d];
        }
        Ok(fi)
    }

    // ── Lettura/Scrittura elemento ────────────────────────────────────────

    pub fn get_nd(&self, indices: &[usize]) -> Result<Value, String> {
        let fi = self.flat_index(indices)?;
        self.data.borrow().get(fi).ok_or_else(|| "index out of bounds".to_string())
    }

    pub fn set_nd(&self, indices: &[usize], v: Value) -> Result<(), String> {
        let fi = self.flat_index(indices)?;
        self.data.borrow_mut().set(fi, v)
    }

    /// Legge elemento flat relativo (0-based rispetto a offset).
    #[inline]
    pub fn get_flat(&self, i: usize) -> Option<Value> {
        self.data.borrow().get(self.offset + i)
    }

    /// Scrive elemento flat relativo.
    #[inline]
    pub fn set_flat(&self, i: usize, v: Value) -> Result<(), String> {
        self.data.borrow_mut().set(self.offset + i, v)
    }

    // ── Accesso per asse (VIEW per ndim>1) ────────────────────────────────

    /// `m[i]` — restituisce VIEW della riga i (condivide i dati con self).
    /// Scrivere nella view modifica self.
    pub fn get_axis0(&self, i: usize) -> Result<Value, String> {
        if self.shape.is_empty() || i >= self.shape[0] {
            return Err(format!("index {} out of bounds for shape {:?}", i, self.shape));
        }
        if self.ndim() == 1 {
            return self.data.borrow().get(self.offset + i)
                .ok_or_else(|| "index out of bounds".to_string());
        }
        let row_len: usize = self.shape[1..].iter().product();
        Ok(Value::nd_array(NdArray {
            shape:  self.shape[1..].to_vec(),
            offset: self.offset + i * row_len,
            data:   Rc::clone(&self.data),   // condivide dati!
        }))
    }

    /// `m[i] = row` — copia i dati di row nella riga i di self.
    pub fn set_axis0(&self, i: usize, src: &NdArray) -> Result<(), String> {
        if self.shape.is_empty() || i >= self.shape[0] {
            return Err(format!("index {} out of bounds for shape {:?}", i, self.shape));
        }
        let row_len: usize = self.shape[1..].iter().product();
        if src.size() != row_len {
            return Err(format!("row size mismatch: expected {}, got {}", row_len, src.size()));
        }
        for j in 0..row_len {
            let v = src.get_flat(j).ok_or_else(|| "src out of bounds".to_string())?;
            self.data.borrow_mut().set(self.offset + i * row_len + j, v)?;
        }
        Ok(())
    }

    /// Produzione di dati flat (copiati, normalizzati da offset) per operazioni bulk.
    pub fn to_flat_vec(&self) -> Vec<f64> {
        (0..self.size())
            .map(|i| self.get_flat(i).and_then(|v| v.as_float()).unwrap_or(0.0))
            .collect()
    }

    // ── Operazioni strutturali ────────────────────────────────────────────

    pub fn transpose_axes(&self, axes: Option<Vec<usize>>) -> Result<NdArray, String> {
        let ndim = self.ndim();
        let axes = axes.unwrap_or_else(|| (0..ndim).rev().collect());
        if axes.len() != ndim { return Err("transpose: wrong number of axes".into()); }
        let mut new_shape = vec![0usize; ndim];
        for (i, &a) in axes.iter().enumerate() { new_shape[i] = self.shape[a]; }
        let mut out = NdArray::zeros(new_shape.clone(), self.data.borrow().dtype());
        let old_strides: Vec<usize> = {
            let mut s = vec![1usize; ndim]; for i in (0..ndim-1).rev() { s[i] = s[i+1] * self.shape[i+1]; } s
        };
        let new_strides: Vec<usize> = {
            let mut s = vec![1usize; ndim]; for i in (0..ndim-1).rev() { s[i] = s[i+1] * new_shape[i+1]; } s
        };
        for fi in 0..self.size() {
            let mut rem = fi;
            let mut old_idx = vec![0usize; ndim];
            for d in 0..ndim { old_idx[d] = rem / old_strides[d]; rem %= old_strides[d]; }
            let new_fi: usize = axes.iter().enumerate().map(|(i,&a)| old_idx[a] * new_strides[i]).sum();
            let v = self.get_flat(fi).unwrap_or(Value::None);
            out.set_flat(new_fi, v).unwrap();
        }
        Ok(out)
    }

    pub fn reshape(&self, new_shape: Vec<usize>) -> Result<NdArray, String> {
        let new_size: usize = new_shape.iter().product();
        if new_size != self.size() {
            return Err(format!("reshape: size mismatch {} vs {}", self.size(), new_size));
        }
        // Copia i dati (la reshape non può essere una view senza strides generalizzate)
        let data = TypedArrayData::Float64(self.to_flat_vec());
        Ok(NdArray { shape: new_shape, offset: 0, data: Rc::new(RefCell::new(data)) })
    }

    // ── Operazioni element-wise ───────────────────────────────────────────

    pub fn ewise_op<F: Fn(f64, f64) -> f64>(&self, other: &NdArray, op: F) -> Result<NdArray, String> {
        if self.shape != other.shape {
            return Err(format!("shape mismatch: {:?} vs {:?} (use nd.add/sub/mul/div for broadcast)", self.shape, other.shape));
        }
        let n = self.size();
        let mut out = NdArray::zeros(self.shape.clone(), Dtype::Float64);
        for i in 0..n {
            let a = self.get_flat(i).and_then(|v| v.as_float()).unwrap_or(0.0);
            let b = other.get_flat(i).and_then(|v| v.as_float()).unwrap_or(0.0);
            out.set_flat(i, Value::Float(op(a, b))).unwrap();
        }
        Ok(out)
    }

    pub fn ewise_scalar<F: Fn(f64, f64) -> f64>(&self, scalar: f64, op: F) -> NdArray {
        let mut out = NdArray::zeros(self.shape.clone(), self.data.borrow().dtype());
        for i in 0..self.size() {
            let a = self.get_flat(i).and_then(|v| v.as_float()).unwrap_or(0.0);
            out.set_flat(i, Value::Float(op(a, scalar))).unwrap();
        }
        out
    }

    pub fn ewise_unary<F: Fn(f64) -> f64>(&self, op: F) -> NdArray {
        let mut out = NdArray::zeros(self.shape.clone(), Dtype::Float64);
        for i in 0..self.size() {
            let a = self.get_flat(i).and_then(|v| v.as_float()).unwrap_or(0.0);
            out.set_flat(i, Value::Float(op(a))).unwrap();
        }
        out
    }

    // ── Reduce globale ────────────────────────────────────────────────────

    pub fn sum_all(&self) -> f64   { (0..self.size()).map(|i| self.get_flat(i).and_then(|v| v.as_float()).unwrap_or(0.0)).sum() }
    pub fn mean_all(&self) -> f64  { self.sum_all() / self.size() as f64 }
    pub fn min_all(&self) -> f64   { (0..self.size()).map(|i| self.get_flat(i).and_then(|v| v.as_float()).unwrap_or(f64::INFINITY)).fold(f64::INFINITY, f64::min) }
    pub fn max_all(&self) -> f64   { (0..self.size()).map(|i| self.get_flat(i).and_then(|v| v.as_float()).unwrap_or(f64::NEG_INFINITY)).fold(f64::NEG_INFINITY, f64::max) }
    pub fn argmin(&self) -> usize  {
        let (idx,_) = (0..self.size()).map(|i| (i, self.get_flat(i).and_then(|v| v.as_float()).unwrap_or(f64::INFINITY)))
            .fold((0, f64::INFINITY), |(bi,bv),(i,v)| if v < bv { (i,v) } else { (bi,bv) });
        idx
    }
    pub fn argmax(&self) -> usize  {
        let (idx,_) = (0..self.size()).map(|i| (i, self.get_flat(i).and_then(|v| v.as_float()).unwrap_or(f64::NEG_INFINITY)))
            .fold((0, f64::NEG_INFINITY), |(bi,bv),(i,v)| if v > bv { (i,v) } else { (bi,bv) });
        idx
    }
    pub fn std_dev(&self, ddof: usize) -> f64 {
        let m = self.mean_all(); let n = self.size();
        let var = (0..n).map(|i| { let x = self.get_flat(i).and_then(|v| v.as_float()).unwrap_or(0.0) - m; x*x }).sum::<f64>()
            / (n - ddof).max(1) as f64;
        var.sqrt()
    }
    pub fn cumsum(&self) -> NdArray {
        let mut out = NdArray::zeros(self.shape.clone(), Dtype::Float64);
        let mut acc = 0.0f64;
        for i in 0..self.size() {
            acc += self.get_flat(i).and_then(|v| v.as_float()).unwrap_or(0.0);
            out.set_flat(i, Value::Float(acc)).unwrap();
        }
        out
    }

    // ── Reduce per asse ───────────────────────────────────────────────────

    fn reduce_axis<F: Fn(f64,f64)->f64>(&self, axis: usize, init: f64, op: F) -> Result<NdArray, String> {
        if axis >= self.ndim() { return Err(format!("axis {} out of range for ndim {}", axis, self.ndim())); }
        let mut new_shape = self.shape.clone(); new_shape.remove(axis);
        if new_shape.is_empty() { new_shape = vec![1]; }
        let mut out = NdArray::zeros(new_shape.clone(), Dtype::Float64);
        for i in 0..out.size() { out.set_flat(i, Value::Float(init)).unwrap(); }
        let mut strides = vec![1usize; self.ndim()];
        for i in (0..self.ndim()-1).rev() { strides[i] = strides[i+1] * self.shape[i+1]; }
        let out_strides: Vec<usize> = { let mut s = vec![1usize; out.ndim()]; for i in (0..out.ndim().saturating_sub(1)).rev() { s[i] = s[i+1] * out.shape[i+1]; } s };
        for fi in 0..self.size() {
            let mut rem = fi; let mut mi = vec![0usize; self.ndim()];
            for d in 0..self.ndim() { mi[d] = rem / strides[d]; rem %= strides[d]; }
            let mut out_mi = mi.clone(); out_mi.remove(axis);
            let out_fi: usize = out_mi.iter().zip(out_strides.iter()).map(|(&i,&s)| i*s).sum();
            let cur = out.get_flat(out_fi).and_then(|v| v.as_float()).unwrap_or(init);
            let v   = self.get_flat(fi).and_then(|v| v.as_float()).unwrap_or(0.0);
            out.set_flat(out_fi, Value::Float(op(cur, v))).unwrap();
        }
        Ok(out)
    }

    pub fn sum_axis(&self, axis: usize)  -> Result<NdArray, String> { self.reduce_axis(axis, 0.0, |a,b| a+b) }
    pub fn max_axis(&self, axis: usize)  -> Result<NdArray, String> { self.reduce_axis(axis, f64::NEG_INFINITY, f64::max) }
    pub fn min_axis(&self, axis: usize)  -> Result<NdArray, String> { self.reduce_axis(axis, f64::INFINITY, f64::min) }
    pub fn mean_axis(&self, axis: usize) -> Result<NdArray, String> {
        let s = self.sum_axis(axis)?;
        let n = self.shape[axis] as f64;
        Ok(s.ewise_scalar(n, |a,_| a / n))
    }

    // ── Matmul ────────────────────────────────────────────────────────────

    pub fn matmul(&self, other: &NdArray) -> Result<NdArray, String> {
        if self.ndim() != 2 || other.ndim() != 2 {
            return Err("matmul: entrambi gli array devono essere 2D".into());
        }
        let (m, k, n) = (self.shape[0], self.shape[1], other.shape[1]);
        if k != other.shape[0] { return Err(format!("matmul: shape mismatch {}x{} @ {}x{}", m, k, other.shape[0], n)); }
        let mut out = NdArray::zeros(vec![m, n], Dtype::Float64);
        for i in 0..m { for j in 0..n {
            let mut s = 0.0f64;
            for p in 0..k {
                s += self.get_flat(i*k+p).and_then(|v| v.as_float()).unwrap_or(0.0)
                   * other.get_flat(p*n+j).and_then(|v| v.as_float()).unwrap_or(0.0);
            }
            out.set_flat(i*n+j, Value::Float(s)).unwrap();
        }}
        Ok(out)
    }

    // ── Broadcasting ──────────────────────────────────────────────────────

    pub fn broadcast_op<F: Fn(f64,f64)->f64>(&self, other: &NdArray, op: F) -> Result<NdArray, String> {
        let ndim = self.ndim().max(other.ndim());
        let mut sa = vec![1usize; ndim]; let mut sb = vec![1usize; ndim];
        for (i,&d) in self.shape.iter().rev().enumerate()  { sa[ndim-1-i] = d; }
        for (i,&d) in other.shape.iter().rev().enumerate() { sb[ndim-1-i] = d; }
        let mut out_shape = vec![0usize; ndim];
        for d in 0..ndim {
            if sa[d] == sb[d]  { out_shape[d] = sa[d]; }
            else if sa[d] == 1 { out_shape[d] = sb[d]; }
            else if sb[d] == 1 { out_shape[d] = sa[d]; }
            else { return Err(format!("broadcast: shapes {:?} and {:?} incompatible", self.shape, other.shape)); }
        }
        let mut out = NdArray::zeros(out_shape.clone(), Dtype::Float64);
        let mut out_strides = vec![1usize; ndim];
        for i in (0..ndim-1).rev() { out_strides[i] = out_strides[i+1] * out_shape[i+1]; }
        let sa_strides = broadcast_strides(&sa, &out_shape);
        let sb_strides = broadcast_strides(&sb, &out_shape);
        for fi in 0..out.size() {
            let mut rem = fi; let mut idx = vec![0usize; ndim];
            for d in 0..ndim { idx[d] = rem / out_strides[d]; rem %= out_strides[d]; }
            let ia: usize = idx.iter().zip(sa_strides.iter()).map(|(&i,&s)| i*s).sum();
            let ib: usize = idx.iter().zip(sb_strides.iter()).map(|(&i,&s)| i*s).sum();
            let va = self.get_flat(ia).and_then(|v| v.as_float()).unwrap_or(0.0);
            let vb = other.get_flat(ib).and_then(|v| v.as_float()).unwrap_or(0.0);
            out.set_flat(fi, Value::Float(op(va, vb))).unwrap();
        }
        Ok(out)
    }

    // ── Boolean ───────────────────────────────────────────────────────────

    pub fn cmp_scalar(&self, scalar: f64, op: &str) -> NdArray {
        let mut out = NdArray::zeros(self.shape.clone(), Dtype::Float64);
        for i in 0..self.size() {
            let v = self.get_flat(i).and_then(|x| x.as_float()).unwrap_or(0.0);
            let r = match op {
                ">"  => v > scalar, ">=" => v >= scalar,
                "<"  => v < scalar, "<=" => v <= scalar,
                "==" => v == scalar, "!=" => v != scalar, _ => false,
            };
            out.set_flat(i, Value::Float(if r { 1.0 } else { 0.0 })).unwrap();
        }
        out
    }

    pub fn boolean_select(&self, mask: &NdArray) -> Result<Vec<f64>, String> {
        if self.size() != mask.size() { return Err("mask size mismatch".into()); }
        Ok((0..self.size())
            .filter(|&i| mask.get_flat(i).and_then(|v| v.as_float()).unwrap_or(0.0) != 0.0)
            .map(|i| self.get_flat(i).and_then(|v| v.as_float()).unwrap_or(0.0))
            .collect())
    }

    // ── Slice 2D ─────────────────────────────────────────────────────────

    pub fn slice_2d(&self, r0: usize, r1: usize, c0: usize, c1: usize) -> Result<NdArray, String> {
        if self.ndim() < 2 { return Err("slice_2d requires 2D+".into()); }
        let cols = self.shape[1];
        if r1 > self.shape[0] || c1 > cols { return Err("slice out of bounds".into()); }
        let out_rows = r1 - r0; let out_cols = c1 - c0;
        let mut out = NdArray::zeros(vec![out_rows, out_cols], self.data.borrow().dtype());
        for r in 0..out_rows { for c in 0..out_cols {
            let v = self.get_flat((r0+r)*cols + c0+c).unwrap_or(Value::None);
            out.set_flat(r*out_cols+c, v).unwrap();
        }}
        Ok(out)
    }
}

fn broadcast_strides(shape: &[usize], out_shape: &[usize]) -> Vec<usize> {
    let ndim = out_shape.len();
    let mut strides = vec![0usize; ndim];
    let mut s = 1usize;
    for d in (0..ndim).rev() {
        if shape[d] > 1 { strides[d] = s; s *= shape[d]; }
        else { strides[d] = 0; }
    }
    strides
}

pub type RcNdArray = Rc<RefCell<NdArray>>;

