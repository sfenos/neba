use std::fmt;
use neba_parser::ast::TypeKind;

/// Il sistema di tipi di Neba.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitivi
    Int,
    Float,
    Bool,
    Str,
    None,

    // Contenitori
    Array(Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),

    // Funzione: parametri + tipo di ritorno
    Fn { params: Vec<Type>, ret: Box<Type> },

    // Classe definita dall'utente
    Class(String),

    // Tipo non ancora determinato (variabile di tipo durante inferenza)
    Unknown,

    // Escape hatch: accetta qualsiasi tipo (spawn, mod, ecc.)
    Any,
}

impl Type {
    /// Converte un'annotazione dell'AST nel nostro Type.
    pub fn from_ast(tk: &TypeKind) -> Self {
        match tk {
            TypeKind::Named(n) => match n.as_str() {
                "Int"   => Type::Int,
                "Float" => Type::Float,
                "Bool"  => Type::Bool,
                "Str"   => Type::Str,
                "None"  => Type::None,
                "Any"   => Type::Any,
                other   => Type::Class(other.to_string()),
            },
            TypeKind::Generic(name, args) => match name.as_str() {
                "Array" => {
                    let inner = args.first()
                        .map(|a| Type::from_ast(&a.inner))
                        .unwrap_or(Type::Unknown);
                    Type::Array(Box::new(inner))
                }
                "Option" => {
                    let inner = args.first()
                        .map(|a| Type::from_ast(&a.inner))
                        .unwrap_or(Type::Unknown);
                    Type::Option(Box::new(inner))
                }
                "Result" => {
                    let ok = args.first()
                        .map(|a| Type::from_ast(&a.inner))
                        .unwrap_or(Type::Unknown);
                    let err = args.get(1)
                        .map(|a| Type::from_ast(&a.inner))
                        .unwrap_or(Type::Unknown);
                    Type::Result(Box::new(ok), Box::new(err))
                }
                other => Type::Class(other.to_string()),
            },
            TypeKind::Error => Type::Unknown,
        }
    }

    /// Restituisce true se i due tipi sono compatibili
    /// (uno può essere assegnato all'altro).
    pub fn is_compatible(&self, other: &Type) -> bool {
        // Any è compatibile con tutto
        if matches!(self, Type::Any) || matches!(other, Type::Any) {
            return true;
        }
        // Unknown è compatibile con tutto (non ancora determinato)
        if matches!(self, Type::Unknown) || matches!(other, Type::Unknown) {
            return true;
        }
        // Int è compatibile con Float (promozione automatica)
        if matches!((self, other), (Type::Int, Type::Float) | (Type::Float, Type::Int)) {
            return true;
        }
        // Compatibilità ricorsiva per i contenitori
        match (self, other) {
            (Type::Array(a),       Type::Array(b))       => a.is_compatible(b),
            (Type::Option(a),      Type::Option(b))      => a.is_compatible(b),
            (Type::Result(a1, a2), Type::Result(b1, b2)) =>
                a1.is_compatible(b1) && a2.is_compatible(b2),
            _ => self == other,
        }
    }

    /// Unifica due tipi: restituisce il tipo risultante o None se incompatibili.
    pub fn unify(a: &Type, b: &Type) -> Option<Type> {
        if matches!(a, Type::Any) || matches!(b, Type::Any) {
            return Some(Type::Any);
        }
        if matches!(a, Type::Unknown) { return Some(b.clone()); }
        if matches!(b, Type::Unknown) { return Some(a.clone()); }
        // Promozione Int → Float
        if matches!((a, b), (Type::Int, Type::Float) | (Type::Float, Type::Int)) {
            return Some(Type::Float);
        }
        if a == b { Some(a.clone()) } else { None }
    }

    /// True se il tipo supporta operatori aritmetici (+, -, *, /).
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::Unknown | Type::Any)
    }

    /// True se il tipo supporta confronto d'ordine (<, <=, >, >=).
    pub fn is_ordered(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::Str | Type::Unknown | Type::Any)
    }

    /// Tipo di un elemento di un iterabile.
    pub fn iter_element(&self) -> Option<Type> {
        match self {
            Type::Array(inner) => Some(*inner.clone()),
            Type::Str          => Some(Type::Str),
            Type::Unknown | Type::Any => Some(Type::Unknown),
            _ => None,
        }
    }

    /// Nome human-readable del tipo.
    pub fn name(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int          => write!(f, "Int"),
            Type::Float        => write!(f, "Float"),
            Type::Bool         => write!(f, "Bool"),
            Type::Str          => write!(f, "Str"),
            Type::None         => write!(f, "None"),
            Type::Array(t)     => write!(f, "Array[{}]", t),
            Type::Option(t)    => write!(f, "Option[{}]", t),
            Type::Result(t, e) => write!(f, "Result[{}, {}]", t, e),
            Type::Fn { params, ret } => {
                let ps: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                write!(f, "Fn[{}] -> {}", ps.join(", "), ret)
            }
            Type::Class(n) => write!(f, "{}", n),
            Type::Unknown  => write!(f, "?"),
            Type::Any      => write!(f, "Any"),
        }
    }
}
