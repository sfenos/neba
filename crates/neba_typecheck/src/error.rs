use std::fmt;
use neba_lexer::Span;
use crate::types::Type;

/// Severità di un diagnostico.
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

/// Un errore/warning prodotto dal type checker.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub severity: Severity,
    pub message:  String,
    pub span:     Span,
}

impl TypeError {
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        TypeError { severity: Severity::Error, message: message.into(), span }
    }
    pub fn warning(message: impl Into<String>, span: Span) -> Self {
        TypeError { severity: Severity::Warning, message: message.into(), span }
    }

    // ── Costruttori per i casi comuni ─────────────────────────────────────

    pub fn type_mismatch(expected: &Type, got: &Type, span: Span) -> Self {
        Self::error(
            format!("type mismatch: expected '{}', got '{}'", expected, got),
            span,
        )
    }

    pub fn binary_op(op: &str, left: &Type, right: &Type, span: Span) -> Self {
        Self::error(
            format!("operator '{}' cannot be applied to '{}' and '{}'", op, left, right),
            span,
        )
    }

    pub fn not_callable(ty: &Type, span: Span) -> Self {
        Self::error(format!("'{}' is not callable", ty), span)
    }

    pub fn arity(name: &str, expected: usize, got: usize, span: Span) -> Self {
        Self::error(
            format!("'{}' expects {} argument(s), got {}", name, expected, got),
            span,
        )
    }

    pub fn undefined(name: &str, span: Span) -> Self {
        Self::error(format!("undefined variable '{}'", name), span)
    }

    pub fn not_iterable(ty: &Type, span: Span) -> Self {
        Self::error(format!("'{}' is not iterable", ty), span)
    }

    pub fn assign_immutable(name: &str, span: Span) -> Self {
        Self::error(format!("cannot assign to immutable variable '{}'", name), span)
    }

    pub fn return_mismatch(expected: &Type, got: &Type, span: Span) -> Self {
        Self::error(
            format!("return type mismatch: function declares '{}', got '{}'", expected, got),
            span,
        )
    }

    pub fn unknown_field(ty: &str, field: &str, span: Span) -> Self {
        Self::error(format!("'{}' has no field '{}'", ty, field), span)
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sev = match self.severity {
            Severity::Error   => "error",
            Severity::Warning => "warning",
        };
        write!(f, "[{}] {}:{}: {}", sev, self.span.line, self.span.column, self.message)
    }
}
