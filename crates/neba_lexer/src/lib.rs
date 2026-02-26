pub mod error;
pub mod lexer;
pub mod token;

pub use error::{LexError, LexResult};
pub use lexer::Lexer;
pub use token::{lookup_keyword, Span, Token, TokenKind};

pub fn tokenize(source: &str) -> (Vec<Token>, Vec<LexError>) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    (tokens, lexer.errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        let (tokens, errors) = tokenize(src);
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors);
        tokens.into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_empty_source() {
        let (tokens, errors) = tokenize("");
        assert!(errors.is_empty());
        assert_eq!(tokens[0].kind, TokenKind::Eof);
    }

    #[test]
    fn test_integer_literals() {
        let k = kinds("42 0xFF 0o77 0b1010 1_000_000");
        assert_eq!(k[0], TokenKind::IntLiteral(42));
        assert_eq!(k[1], TokenKind::IntLiteral(0xFF));
        assert_eq!(k[2], TokenKind::IntLiteral(0o77));
        assert_eq!(k[3], TokenKind::IntLiteral(0b1010));
        assert_eq!(k[4], TokenKind::IntLiteral(1_000_000));
    }

    #[test]
    fn test_float_literals() {
        let k = kinds("3.14 2.0e10 1.5E-3");
        assert_eq!(k[0], TokenKind::FloatLiteral(3.14));
        assert_eq!(k[1], TokenKind::FloatLiteral(2.0e10));
        assert_eq!(k[2], TokenKind::FloatLiteral(1.5e-3));
    }

    #[test]
    fn test_string_literal() {
        let (tokens, errors) = tokenize(r#""hello world""#);
        assert!(errors.is_empty());
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral("hello world".to_string()));
    }

    #[test]
    fn test_fstring_literal() {
        let (tokens, errors) = tokenize(r#"f"hello {name}""#);
        assert!(errors.is_empty());
        assert_eq!(tokens[0].kind, TokenKind::FStringLiteral("hello {name}".to_string()));
    }

    #[test]
    fn test_bool_and_none() {
        let k = kinds("true false None");
        assert_eq!(k[0], TokenKind::BoolLiteral(true));
        assert_eq!(k[1], TokenKind::BoolLiteral(false));
        assert_eq!(k[2], TokenKind::NoneLiteral);
    }

    #[test]
    fn test_keywords() {
        let k = kinds("let var fn if else while for return");
        assert_eq!(k[0], TokenKind::Let);
        assert_eq!(k[1], TokenKind::Var);
        assert_eq!(k[2], TokenKind::Fn);
        assert_eq!(k[3], TokenKind::If);
        assert_eq!(k[4], TokenKind::Else);
        assert_eq!(k[5], TokenKind::While);
        assert_eq!(k[6], TokenKind::For);
        assert_eq!(k[7], TokenKind::Return);
    }

    #[test]
    fn test_operators() {
        let k = kinds("+ - * / // ** % == != <= >= -> =>");
        assert_eq!(k[0], TokenKind::Plus);
        assert_eq!(k[1], TokenKind::Minus);
        assert_eq!(k[2], TokenKind::Star);
        assert_eq!(k[3], TokenKind::Slash);
        assert_eq!(k[4], TokenKind::SlashSlash);
        assert_eq!(k[5], TokenKind::StarStar);
        assert_eq!(k[6], TokenKind::Percent);
        assert_eq!(k[7], TokenKind::EqualEqual);
        assert_eq!(k[8], TokenKind::BangEqual);
        assert_eq!(k[9], TokenKind::LessEqual);
        assert_eq!(k[10], TokenKind::GreaterEqual);
        assert_eq!(k[11], TokenKind::Arrow);
        assert_eq!(k[12], TokenKind::FatArrow);
    }

    #[test]
    fn test_identifiers() {
        let k = kinds("foo bar_baz _private MyClass");
        assert_eq!(k[0], TokenKind::Identifier("foo".to_string()));
        assert_eq!(k[1], TokenKind::Identifier("bar_baz".to_string()));
        assert_eq!(k[2], TokenKind::Identifier("_private".to_string()));
        assert_eq!(k[3], TokenKind::Identifier("MyClass".to_string()));
    }

    #[test]
    fn test_indentation() {
        let src = "if x\n    let y = 1\n";
        let k = kinds(src);
        assert!(k.contains(&TokenKind::Indent));
        assert!(k.contains(&TokenKind::Dedent));
    }

    #[test]
    fn test_comment_skipped() {
        let k = kinds("let x = 1 # commento");
        assert_eq!(k[0], TokenKind::Let);
        assert!(!k.iter().any(|t| matches!(t, TokenKind::Unknown(_))));
    }

    #[test]
    fn test_escape_sequences() {
        let (tokens, errors) = tokenize(r#""\n\t\\""#);
        assert!(errors.is_empty());
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral("\n\t\\".to_string()));
    }

    #[test]
    fn test_spawn_keyword() {
        let k = kinds("spawn task()");
        assert_eq!(k[0], TokenKind::Spawn);
    }

    #[test]
    fn test_range_operators() {
        let k = kinds("0..10 0..=10");
        assert_eq!(k[1], TokenKind::DotDot);
        assert_eq!(k[4], TokenKind::DotDotEqual);
    }

    #[test]
    fn test_unterminated_string_error() {
        let (_tokens, errors) = tokenize("\"hello");
        assert!(!errors.is_empty());
        assert!(matches!(errors[0], LexError::UnterminatedString { .. }));
    }

    #[test]
    fn test_tab_error() {
        let (_tokens, errors) = tokenize("if x\n\tlet y = 1");
        assert!(!errors.is_empty());
        assert!(matches!(errors[0], LexError::TabSpaceMixing { .. }));
    }
}
