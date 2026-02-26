use std::fmt;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use neba_parser::ast::{Param, Stmt};
use crate::environment::Env;

/// Il tipo runtime di Neba.
/// Tutti i valori sono immutabili dal punto di vista di Value,
/// la mutabilità è gestita dall'Environment tramite Rc<RefCell<_>>.
#[derive(Clone)]
pub enum Value {
    // Primitivi
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),

    // Option
    Some(Box<Value>),
    None,

    // Result
    Ok(Box<Value>),
    Err(Box<Value>),

    // Array (mutabile tramite RefCell)
    Array(Rc<RefCell<Vec<Value>>>),

    // Funzione definita dall'utente
    Function(Rc<FunctionDef>),

    // Funzione nativa (built-in)
    NativeFunction(String, std::rc::Rc<dyn Fn(Vec<Value>) -> Result<Value, String>>),

    // Istanza di classe
    Instance(Rc<RefCell<Instance>>),

    // Valore sentinella usato da return/break/continue
    // (non esposto all'utente, solo per il flow control interno)
    __Return(Box<Value>),
    __Break,
    __Continue,
}


impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(n)    => write!(f, "Int({})", n),
            Value::Float(n)  => write!(f, "Float({})", n),
            Value::Bool(b)   => write!(f, "Bool({})", b),
            Value::Str(s)    => write!(f, "Str({:?})", s),
            Value::None      => write!(f, "None"),
            Value::Some(v)   => write!(f, "Some({:?})", v),
            Value::Ok(v)     => write!(f, "Ok({:?})", v),
            Value::Err(v)    => write!(f, "Err({:?})", v),
            Value::Array(a)  => write!(f, "Array({:?})", a.borrow()),
            Value::Function(d) => write!(f, "Function({})", d.name),
            Value::NativeFunction(n, _) => write!(f, "NativeFunction({})", n),
            Value::Instance(i) => write!(f, "Instance({})", i.borrow().class_name),
            Value::__Return(v) => write!(f, "__Return({:?})", v),
            Value::__Break     => write!(f, "__Break"),
            Value::__Continue  => write!(f, "__Continue"),
        }
    }
}

/// Definizione di una funzione utente.
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub closure: Env,  // cattura l'environment al momento della definizione
    pub is_async: bool,
}

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

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }

    pub fn set(&mut self, name: impl Into<String>, val: Value) {
        self.fields.insert(name.into(), val);
    }
}

// ── Display ────────────────────────────────────────────────────────────────

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n)    => write!(f, "{}", n),
            Value::Float(n)  => {
                if n.fract() == 0.0 { write!(f, "{:.1}", n) } else { write!(f, "{}", n) }
            }
            Value::Bool(b)   => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::Str(s)    => write!(f, "{}", s),
            Value::None      => write!(f, "None"),
            Value::Some(v)   => write!(f, "Some({})", v),
            Value::Ok(v)     => write!(f, "Ok({})", v),
            Value::Err(v)    => write!(f, "Err({})", v),
            Value::Array(arr)=> {
                let items: Vec<String> = arr.borrow().iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Function(def) => write!(f, "<fn {}>", def.name),
            Value::NativeFunction(name, _) => write!(f, "<built-in fn {}>", name),
            Value::Instance(inst) => write!(f, "<{} instance>", inst.borrow().class_name),
            Value::__Return(v) => write!(f, "{}", v),
            Value::__Break    => write!(f, "<break>"),
            Value::__Continue => write!(f, "<continue>"),
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
            (Value::Some(a),  Value::Some(b))  => a == b,
            (Value::Ok(a),    Value::Ok(b))    => a == b,
            (Value::Err(a),   Value::Err(b))   => a == b,
            (Value::Array(a), Value::Array(b)) => *a.borrow() == *b.borrow(),
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
            (Value::Str(a),   Value::Str(b))   => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl Value {
    /// Restituisce true se il valore è "truthy" (usato in if/while)
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b)   => *b,
            Value::Int(n)    => *n != 0,
            Value::Float(f)  => *f != 0.0,
            Value::Str(s)    => !s.is_empty(),
            Value::None      => false,
            Value::Some(_)   => true,
            Value::Array(a)  => !a.borrow().is_empty(),
            _                => true,
        }
    }

    /// Restituisce il nome del tipo (per messaggi di errore)
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_)            => "Int",
            Value::Float(_)          => "Float",
            Value::Bool(_)           => "Bool",
            Value::Str(_)            => "Str",
            Value::None              => "None",
            Value::Some(_)           => "Some",
            Value::Ok(_)             => "Ok",
            Value::Err(_)            => "Err",
            Value::Array(_)          => "Array",
            Value::Function(_)       => "Function",
            Value::NativeFunction(_, _) => "NativeFunction",
            Value::Instance(_)       => "Instance",
            Value::__Return(_)       => "__Return",
            Value::__Break           => "__Break",
            Value::__Continue        => "__Continue",
        }
    }

    /// Tenta la conversione a Int (per operazioni miste)
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Int(n)   => Some(*n as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }
}
