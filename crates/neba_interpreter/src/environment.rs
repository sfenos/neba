use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// Un singolo frame dello scope.
#[derive(Debug, Clone)]
struct Frame {
    vars: HashMap<String, (Value, bool)>, // (valore, è_mutabile)
}

impl Frame {
    fn new() -> Self {
        Frame { vars: HashMap::new() }
    }
}

/// Environment con scope annidati (stack di frame).
/// Clonare un Env produce una "closure" che condivide i frame superiori.
#[derive(Debug, Clone)]
pub struct Env {
    // Stack di frame: il primo è il globale, l'ultimo è il locale corrente.
    // Usiamo Rc<RefCell<_>> per permettere la condivisione tra closures.
    frames: Vec<Rc<RefCell<Frame>>>,
}

impl Env {
    /// Crea un nuovo environment vuoto (solo frame globale).
    pub fn new() -> Self {
        Env { frames: vec![Rc::new(RefCell::new(Frame::new()))] }
    }

    /// Apre un nuovo scope (push frame).
    pub fn push_scope(&mut self) {
        self.frames.push(Rc::new(RefCell::new(Frame::new())));
    }

    /// Chiude lo scope corrente (pop frame).
    pub fn pop_scope(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    /// Definisce una variabile nel frame corrente.
    /// `mutable` = true per `var`, false per `let`.
    pub fn define(&mut self, name: impl Into<String>, value: Value, mutable: bool) {
        let frame = self.frames.last().unwrap();
        frame.borrow_mut().vars.insert(name.into(), (value, mutable));
    }

    /// Legge una variabile cercando dallo scope corrente verso il globale.
    pub fn get(&self, name: &str) -> Option<Value> {
        for frame in self.frames.iter().rev() {
            if let Some((val, _)) = frame.borrow().vars.get(name) {
                return Some(val.clone());
            }
        }
        None
    }

    /// Aggiorna una variabile esistente. Errore se non esiste o non è mutabile.
    pub fn set(&mut self, name: &str, value: Value) -> Result<(), String> {
        for frame in self.frames.iter().rev() {
            let mut frame = frame.borrow_mut();
            if let Some((val, mutable)) = frame.vars.get_mut(name) {
                if !*mutable {
                    return Err(format!("cannot assign to immutable variable '{}'", name));
                }
                *val = value;
                return Ok(());
            }
        }
        Err(format!("undefined variable '{}'", name))
    }

    /// Crea una snapshot dell'environment corrente (per chiusure di funzioni).
    /// I frame sono condivisi (Rc), quindi le closure vedono le modifiche successive
    /// alle variabili catturate — comportamento identico a Python.
    pub fn snapshot(&self) -> Env {
        Env { frames: self.frames.clone() }
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}
