use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    /// Variabile non definita
    UndefinedVariable { name: String },
    /// Assegnazione a variabile immutabile o non esistente
    AssignError { message: String },
    /// Tipo sbagliato per un'operazione
    TypeError { message: String },
    /// Divisione per zero
    DivisionByZero,
    /// Indice fuori range
    IndexOutOfBounds { index: i64, len: usize },
    /// Chiamata a non-callable
    NotCallable { type_name: String },
    /// Numero di argomenti errato
    ArityMismatch { name: String, expected: usize, got: usize },
    /// Accesso a campo inesistente
    UnknownField { type_name: String, field: String },
    /// Errore generico (es. da funzioni native)
    Generic { message: String },
    /// Stack overflow (ricorsione infinita)
    StackOverflow,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::UndefinedVariable { name } =>
                write!(f, "[RuntimeError] Undefined variable '{}'", name),
            RuntimeError::AssignError { message } =>
                write!(f, "[RuntimeError] Assignment error: {}", message),
            RuntimeError::TypeError { message } =>
                write!(f, "[RuntimeError] Type error: {}", message),
            RuntimeError::DivisionByZero =>
                write!(f, "[RuntimeError] Division by zero"),
            RuntimeError::IndexOutOfBounds { index, len } =>
                write!(f, "[RuntimeError] Index {} out of bounds for array of length {}", index, len),
            RuntimeError::NotCallable { type_name } =>
                write!(f, "[RuntimeError] '{}' is not callable", type_name),
            RuntimeError::ArityMismatch { name, expected, got } =>
                write!(f, "[RuntimeError] '{}' expects {} argument(s), got {}", name, expected, got),
            RuntimeError::UnknownField { type_name, field } =>
                write!(f, "[RuntimeError] '{}' has no field '{}'", type_name, field),
            RuntimeError::Generic { message } =>
                write!(f, "[RuntimeError] {}", message),
            RuntimeError::StackOverflow =>
                write!(f, "[RuntimeError] Stack overflow (maximum recursion depth exceeded)"),
        }
    }
}

impl std::error::Error for RuntimeError {}

pub type InterpResult = Result<crate::value::Value, RuntimeError>;
