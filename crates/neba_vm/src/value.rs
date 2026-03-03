use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::chunk::FnProto;

// ── Tipi heap-allocated (gestiti da Rc = GC v0.2.0) ──────────────────────

pub type RcArray    = Rc<RefCell<Vec<Value>>>;
pub type RcInstance = Rc<RefCell<Instance>>;
pub type RcClosure  = Rc<Closure>;
/// Dict: mappa chiave→valore con ordine di inserimento preservato.
pub type RcDict     = Rc<RefCell<Vec<(Value, Value)>>>;

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
    pub upvalues: Vec<Upvalue>,
}

impl fmt::Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Closure({})", self.proto.name)
    }
}

/// Un upvalue è un valore catturato da un frame padre.
/// In v0.2.0 è sempre "closed" (copiato per valore) al momento della cattura.
#[derive(Debug, Clone)]
pub struct Upvalue {
    pub value: Value,
}

/// Funzione nativa (built-in).
pub type NativeFn = fn(&[Value]) -> Result<Value, String>;

/// Istanza di una classe.
#[derive(Debug, Clone)]
pub struct Instance {
    pub class_name: String,
    pub fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(class_name: impl Into<String>) -> Self {
        Instance { class_name: class_name.into(), fields: HashMap::new() }
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
    Closure(RcClosure),
    NativeFn(String, NativeFn),

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
            // TypedArray: uguaglianza per identità (stesso Rc) — confronto elemento per elemento in futuro
            (Value::TypedArray(a), Value::TypedArray(b)) => Rc::ptr_eq(a, b),
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

    /// Costruisce un Value::Array da un Vec<Value>
    pub fn array(v: Vec<Value>) -> Self {
        Value::Array(Rc::new(RefCell::new(v)))
    }

    /// Costruisce un Value::Dict da un Vec<(Value, Value)>
    pub fn dict(pairs: Vec<(Value, Value)>) -> Self {
        Value::Dict(Rc::new(RefCell::new(pairs)))
    }

    /// Costruisce un Value::TypedArray da TypedArrayData
    pub fn typed_array(data: TypedArrayData) -> Self {
        Value::TypedArray(Rc::new(RefCell::new(data)))
    }
}
