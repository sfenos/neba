use crate::token::Span;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    UnexpectedCharacter { ch: char, span: Span },
    UnterminatedString { span: Span },
    InvalidEscapeSequence { seq: String, span: Span },
    InvalidNumber { raw: String, span: Span },
    InconsistentIndentation { span: Span },
    TabSpaceMixing { span: Span },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::UnexpectedCharacter { ch, span } =>
                write!(f, "[LexError] Unexpected character '{}' at line {}, column {}", ch, span.line, span.column),
            LexError::UnterminatedString { span } =>
                write!(f, "[LexError] Unterminated string at line {}, column {}", span.line, span.column),
            LexError::InvalidEscapeSequence { seq, span } =>
                write!(f, "[LexError] Invalid escape sequence '{}' at line {}, column {}", seq, span.line, span.column),
            LexError::InvalidNumber { raw, span } =>
                write!(f, "[LexError] Invalid number '{}' at line {}, column {}", raw, span.line, span.column),
            LexError::InconsistentIndentation { span } =>
                write!(f, "[LexError] Inconsistent indentation at line {}", span.line),
            LexError::TabSpaceMixing { span } =>
                write!(f, "[LexError] Mixed tabs and spaces at line {}", span.line),
        }
    }
}

impl std::error::Error for LexError {}

pub type LexResult<T> = Result<T, LexError>;
