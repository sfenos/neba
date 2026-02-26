#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(line: usize, column: usize, start: usize, end: usize) -> Self {
        Span { line, column, start, end }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub lexeme: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, lexeme: impl Into<String>) -> Self {
        Token { kind, span, lexeme: lexeme.into() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    FStringLiteral(String),
    BoolLiteral(bool),
    NoneLiteral,

    // Identifiers
    Identifier(String),

    // Keywords — control flow
    If, Elif, Else, While, For, In, Break, Continue, Return, Match, Case,

    // Keywords — declarations
    Let, Var, Fn, Class, Trait, Impl, Enum, Type, Mod, Use,

    // Keywords — concurrency
    Spawn, Await, Async,

    // Keywords — option/result
    Some, Ok, Err,

    // Keywords — other
    And, Or, Not, Is, As, Super, Self_, New, Delete, Pass,

    // Arithmetic operators
    Plus, Minus, Star, Slash, SlashSlash, Percent, StarStar,

    // Bitwise operators
    Ampersand, Pipe, Caret, Tilde, LessLess, GreaterGreater,

    // Comparison operators
    EqualEqual, BangEqual, Less, LessEqual, Greater, GreaterEqual,

    // Assignment operators
    Equal, PlusEqual, MinusEqual, StarEqual, SlashEqual, PercentEqual,

    // Other operators
    Arrow, FatArrow, Dot, DotDot, DotDotEqual, ColonColon,
    QuestionMark, Bang, At, Pipe2, Ampersand2,

    // Delimiters
    LParen, RParen, LBracket, RBracket, LBrace, RBrace,
    Comma, Colon, Semicolon, Underscore,

    // Indentation
    Newline, Indent, Dedent,

    // Special
    Eof,
    Unknown(char),
}

pub fn lookup_keyword(s: &str) -> Option<TokenKind> {
    match s {
        "if"       => Some(TokenKind::If),
        "elif"     => Some(TokenKind::Elif),
        "else"     => Some(TokenKind::Else),
        "while"    => Some(TokenKind::While),
        "for"      => Some(TokenKind::For),
        "in"       => Some(TokenKind::In),
        "break"    => Some(TokenKind::Break),
        "continue" => Some(TokenKind::Continue),
        "return"   => Some(TokenKind::Return),
        "match"    => Some(TokenKind::Match),
        "case"     => Some(TokenKind::Case),
        "let"      => Some(TokenKind::Let),
        "var"      => Some(TokenKind::Var),
        "fn"       => Some(TokenKind::Fn),
        "class"    => Some(TokenKind::Class),
        "trait"    => Some(TokenKind::Trait),
        "impl"     => Some(TokenKind::Impl),
        "enum"     => Some(TokenKind::Enum),
        "type"     => Some(TokenKind::Type),
        "mod"      => Some(TokenKind::Mod),
        "use"      => Some(TokenKind::Use),
        "spawn"    => Some(TokenKind::Spawn),
        "await"    => Some(TokenKind::Await),
        "async"    => Some(TokenKind::Async),
        "Some"     => Some(TokenKind::Some),
        "None"     => Some(TokenKind::NoneLiteral),
        "Ok"       => Some(TokenKind::Ok),
        "Err"      => Some(TokenKind::Err),
        "and"      => Some(TokenKind::And),
        "or"       => Some(TokenKind::Or),
        "not"      => Some(TokenKind::Not),
        "is"       => Some(TokenKind::Is),
        "as"       => Some(TokenKind::As),
        "super"    => Some(TokenKind::Super),
        "self"     => Some(TokenKind::Self_),
        "new"      => Some(TokenKind::New),
        "delete"   => Some(TokenKind::Delete),
        "pass"     => Some(TokenKind::Pass),
        "true"     => Some(TokenKind::BoolLiteral(true)),
        "false"    => Some(TokenKind::BoolLiteral(false)),
        _          => None,
    }
}
