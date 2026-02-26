use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    // ── Compile-time ──────────────────────────────────────────────────────
    CompileError(String),

    // ── Runtime ───────────────────────────────────────────────────────────
    UndefinedVariable(String),
    AssignImmutable(String),
    TypeError(String),
    DivisionByZero,
    IndexOutOfBounds { index: i64, len: usize },
    NotCallable(String),
    ArityMismatch { name: String, expected: usize, got: usize },
    UnknownField { type_name: String, field: String },
    StackOverflow,
    Generic(String),
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::CompileError(m)       => write!(f, "[CompileError] {}", m),
            VmError::UndefinedVariable(n)  => write!(f, "[RuntimeError] Undefined variable '{}'", n),
            VmError::AssignImmutable(n)    => write!(f, "[RuntimeError] Cannot assign to immutable variable '{}'", n),
            VmError::TypeError(m)          => write!(f, "[RuntimeError] Type error: {}", m),
            VmError::DivisionByZero        => write!(f, "[RuntimeError] Division by zero"),
            VmError::IndexOutOfBounds { index, len }
                => write!(f, "[RuntimeError] Index {} out of bounds for length {}", index, len),
            VmError::NotCallable(t)        => write!(f, "[RuntimeError] '{}' is not callable", t),
            VmError::ArityMismatch { name, expected, got }
                => write!(f, "[RuntimeError] '{}' expects {} arg(s), got {}", name, expected, got),
            VmError::UnknownField { type_name, field }
                => write!(f, "[RuntimeError] '{}' has no field '{}'", type_name, field),
            VmError::StackOverflow         => write!(f, "[RuntimeError] Stack overflow"),
            VmError::Generic(m)            => write!(f, "[RuntimeError] {}", m),
        }
    }
}

impl std::error::Error for VmError {}

pub type VmResult<T = crate::value::Value> = Result<T, VmError>;
