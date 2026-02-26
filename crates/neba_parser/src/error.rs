use neba_lexer::{Span, TokenKind};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken    { expected: String, found: TokenKind, span: Span },
    UnexpectedEof      { expected: String, span: Span },
    InvalidAssignTarget{ span: Span },
    MissingIndent      { span: Span },
    MissingDedent      { span: Span },
    InvalidPattern     { span: Span },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, found, span } =>
                write!(f, "[ParseError] Expected {} but found {:?} at line {}, column {}", expected, found, span.line, span.column),
            ParseError::UnexpectedEof { expected, span } =>
                write!(f, "[ParseError] Expected {} but reached end of file at line {}", expected, span.line),
            ParseError::InvalidAssignTarget { span } =>
                write!(f, "[ParseError] Invalid assignment target at line {}, column {}", span.line, span.column),
            ParseError::MissingIndent { span } =>
                write!(f, "[ParseError] Expected indented block at line {}", span.line),
            ParseError::MissingDedent { span } =>
                write!(f, "[ParseError] Missing dedent at line {}", span.line),
            ParseError::InvalidPattern { span } =>
                write!(f, "[ParseError] Invalid pattern in match arm at line {}, column {}", span.line, span.column),
        }
    }
}

impl std::error::Error for ParseError {}

pub type ParseResult<T> = Result<T, ParseError>;
