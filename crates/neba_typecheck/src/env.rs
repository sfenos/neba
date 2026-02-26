use std::collections::HashMap;
use crate::types::Type;

/// Informazioni su una variabile nell'ambiente.
#[derive(Debug, Clone)]
pub struct VarInfo {
    pub ty:      Type,
    pub mutable: bool,
}

/// Frame di un singolo scope.
#[derive(Debug, Clone, Default)]
struct Frame {
    vars: HashMap<String, VarInfo>,
}

/// Ambiente dei tipi: stack di frame per gestire scope annidati.
#[derive(Debug, Clone)]
pub struct TypeEnv {
    frames: Vec<Frame>,
    /// Tipi di ritorno attesi per le funzioni annidate (stack).
    pub return_stack: Vec<Type>,
    /// Registro delle classi: nome → (campi, metodi)
    pub classes: HashMap<String, ClassInfo>,
}

/// Informazioni su una classe definita dall'utente.
#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub fields:  HashMap<String, Type>,
    pub methods: HashMap<String, Type>,
}

impl TypeEnv {
    pub fn new() -> Self {
        let mut env = TypeEnv {
            frames:       vec![Frame::default()],
            return_stack: Vec::new(),
            classes:      HashMap::new(),
        };
        // Built-in globali
        env.register_builtins();
        env
    }

    fn register_builtins(&mut self) {
        use Type::*;
        let builtins: &[(&str, Type)] = &[
            ("print",   Fn { params: vec![Any], ret: Box::new(None) }),
            ("println", Fn { params: vec![Any], ret: Box::new(None) }),
            ("input",   Fn { params: vec![Str], ret: Box::new(Str)  }),
            ("len",     Fn { params: vec![Any], ret: Box::new(Int)  }),
            ("str",     Fn { params: vec![Any], ret: Box::new(Str)  }),
            ("int",     Fn { params: vec![Any], ret: Box::new(Int)  }),
            ("float",   Fn { params: vec![Any], ret: Box::new(Float)}),
            ("bool",    Fn { params: vec![Any], ret: Box::new(Bool) }),
            ("typeof",  Fn { params: vec![Any], ret: Box::new(Str)  }),
            ("abs",     Fn { params: vec![Any], ret: Box::new(Float)}),
            ("min",     Fn { params: vec![Any], ret: Box::new(Any)  }),
            ("max",     Fn { params: vec![Any], ret: Box::new(Any)  }),
            ("range",   Fn { params: vec![Int, Int], ret: Box::new(Array(Box::new(Int))) }),
            ("push",    Fn { params: vec![Any, Any], ret: Box::new(None) }),
            ("pop",     Fn { params: vec![Any], ret: Box::new(Any)  }),
            ("assert",  Fn { params: vec![Bool], ret: Box::new(None)}),
        ];
        for (name, ty) in builtins {
            self.define(name, ty.clone(), false);
        }
    }

    // ── Gestione scope ────────────────────────────────────────────────────

    pub fn push_scope(&mut self) {
        self.frames.push(Frame::default());
    }

    pub fn pop_scope(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    // ── Variabili ─────────────────────────────────────────────────────────

    /// Definisce una nuova variabile nello scope corrente.
    pub fn define(&mut self, name: &str, ty: Type, mutable: bool) {
        if let Some(frame) = self.frames.last_mut() {
            frame.vars.insert(name.to_string(), VarInfo { ty, mutable });
        }
    }

    /// Cerca una variabile risalendo gli scope.
    pub fn lookup(&self, name: &str) -> Option<&VarInfo> {
        for frame in self.frames.iter().rev() {
            if let Some(info) = frame.vars.get(name) {
                return Some(info);
            }
        }
        None
    }

    /// Cerca una variabile (mut) per aggiornarne il tipo.
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut VarInfo> {
        for frame in self.frames.iter_mut().rev() {
            if frame.vars.contains_key(name) {
                return frame.vars.get_mut(name);
            }
        }
        None
    }

    // ── Tipi di ritorno ───────────────────────────────────────────────────

    pub fn push_return(&mut self, ty: Type) { self.return_stack.push(ty); }
    pub fn pop_return(&mut self)            { self.return_stack.pop(); }
    pub fn expected_return(&self) -> Option<&Type> { self.return_stack.last() }

    // ── Classi ────────────────────────────────────────────────────────────

    pub fn register_class(&mut self, name: &str, info: ClassInfo) {
        self.classes.insert(name.to_string(), info);
    }

    pub fn lookup_class(&self, name: &str) -> Option<&ClassInfo> {
        self.classes.get(name)
    }
}
