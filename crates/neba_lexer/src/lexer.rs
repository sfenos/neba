use crate::error::{LexError, LexResult};
use crate::token::{lookup_keyword, Span, Token, TokenKind};

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
    pending: Vec<Token>,
    pub errors: Vec<LexError>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0],
            pending: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied()?;
        self.pos += 1;
        if ch == '\n' { self.line += 1; self.column = 1; } else { self.column += 1; }
        Some(ch)
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) { self.advance(); true } else { false }
    }

    fn make_token(&self, kind: TokenKind, start: usize, start_col: usize, lexeme: &str) -> Token {
        Token::new(kind, Span::new(self.line, start_col, start, self.pos), lexeme)
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        loop {
            tokens.extend(self.pending.drain(..));
            if self.pos >= self.source.len() {
                tokens.extend(self.close_all_blocks());
                tokens.push(Token::new(TokenKind::Eof, Span::new(self.line, self.column, self.pos, self.pos), ""));
                break;
            }
            match self.next_token() {
                Ok(Some(tok)) => tokens.push(tok),
                Ok(None) => {}
                Err(e) => {
                    self.errors.push(e);
                    let start = self.pos;
                    let col = self.column;
                    let ch = self.advance().unwrap_or('\0');
                    tokens.push(self.make_token(TokenKind::Unknown(ch), start, col, &ch.to_string()));
                }
            }
        }
        tokens
    }

    fn next_token(&mut self) -> LexResult<Option<Token>> {
        if self.column == 1 {
            if let Some(t) = self.process_indentation()? {
                return Ok(Some(t));
            }
        }

        let start = self.pos;
        let start_col = self.column;

        let ch = match self.peek() {
            Some(c) => c,
            None => return Ok(None),
        };

        if ch == ' ' || ch == '\t' || ch == '\r' { self.advance(); return Ok(None); }
        if ch == '#' { while self.peek() != Some('\n') && self.peek().is_some() { self.advance(); } return Ok(None); }
        if ch == '\n' {
            self.advance();
            return Ok(Some(self.make_token(TokenKind::Newline, start, start_col, "\n")));
        }

        self.advance();

        let kind = match ch {
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            '~' => TokenKind::Tilde,
            '@' => TokenKind::At,
            '_' if !self.peek().map_or(false, |c| c.is_alphanumeric() || c == '_') => TokenKind::Underscore,
            '+' => if self.match_char('=') { TokenKind::PlusEqual } else { TokenKind::Plus },
            '-' => if self.match_char('>') { TokenKind::Arrow } else if self.match_char('=') { TokenKind::MinusEqual } else { TokenKind::Minus },
            '*' => if self.match_char('*') { TokenKind::StarStar } else if self.match_char('=') { TokenKind::StarEqual } else { TokenKind::Star },
            '/' => if self.match_char('/') { TokenKind::SlashSlash } else if self.match_char('=') { TokenKind::SlashEqual } else { TokenKind::Slash },
            '%' => if self.match_char('=') { TokenKind::PercentEqual } else { TokenKind::Percent },
            '=' => if self.match_char('=') { TokenKind::EqualEqual } else if self.match_char('>') { TokenKind::FatArrow } else { TokenKind::Equal },
            '!' => if self.match_char('=') { TokenKind::BangEqual } else { TokenKind::Bang },
            '<' => if self.match_char('<') { TokenKind::LessLess } else if self.match_char('=') { TokenKind::LessEqual } else { TokenKind::Less },
            '>' => if self.match_char('>') { TokenKind::GreaterGreater } else if self.match_char('=') { TokenKind::GreaterEqual } else { TokenKind::Greater },
            '&' => if self.match_char('&') { TokenKind::Ampersand2 } else { TokenKind::Ampersand },
            '|' => if self.match_char('|') { TokenKind::Pipe2 } else { TokenKind::Pipe },
            '^' => TokenKind::Caret,
            '?' => TokenKind::QuestionMark,
            ':' => if self.match_char(':') { TokenKind::ColonColon } else { TokenKind::Colon },
            '.' => {
                if self.peek() == Some('.') {
                    self.advance();
                    if self.match_char('=') { TokenKind::DotDotEqual } else { TokenKind::DotDot }
                } else { TokenKind::Dot }
            },
            '"' | '\'' => return Ok(Some(self.lex_string(ch, false, start, start_col)?)),
            'f' if self.peek() == Some('"') || self.peek() == Some('\'') => {
                let quote = self.advance().unwrap();
                return Ok(Some(self.lex_string(quote, true, start, start_col)?));
            },
            c if c.is_ascii_digit() => return Ok(Some(self.lex_number(c, start, start_col)?)),
            c if c.is_alphabetic() || c == '_' => return Ok(Some(self.lex_identifier(c, start, start_col))),
            other => return Err(LexError::UnexpectedCharacter { ch: other, span: Span::new(self.line, start_col, start, self.pos) }),
        };

        let lexeme: String = self.source[start..self.pos].iter().collect();
        Ok(Some(self.make_token(kind, start, start_col, &lexeme)))
    }

    fn process_indentation(&mut self) -> LexResult<Option<Token>> {
        let start = self.pos;
        let mut indent = 0usize;
        loop {
            match self.peek() {
                Some(' ') => { indent += 1; self.advance(); }
                Some('\t') => return Err(LexError::TabSpaceMixing { span: Span::new(self.line, self.column, self.pos, self.pos + 1) }),
                _ => break,
            }
        }
        match self.peek() {
            Some('\n') | Some('#') | None => return Ok(None),
            _ => {}
        }
        let current = *self.indent_stack.last().unwrap_or(&0);
        if indent > current {
            self.indent_stack.push(indent);
            return Ok(Some(Token::new(TokenKind::Indent, Span::new(self.line, 1, start, self.pos), "<indent>")));
        }
        if indent < current {
            let mut dedents = Vec::new();
            while *self.indent_stack.last().unwrap_or(&0) > indent {
                self.indent_stack.pop();
                dedents.push(Token::new(TokenKind::Dedent, Span::new(self.line, 1, start, self.pos), "<dedent>"));
            }
            if *self.indent_stack.last().unwrap_or(&0) != indent {
                return Err(LexError::InconsistentIndentation { span: Span::new(self.line, 1, start, self.pos) });
            }
            if !dedents.is_empty() {
                let first = dedents.remove(0);
                self.pending.extend(dedents);
                return Ok(Some(first));
            }
        }
        Ok(None)
    }

    fn close_all_blocks(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(Token::new(TokenKind::Dedent, Span::new(self.line, self.column, self.pos, self.pos), "<dedent>"));
        }
        tokens
    }

    fn lex_string(&mut self, quote: char, is_fstring: bool, start: usize, start_col: usize) -> LexResult<Token> {
        let triple = self.peek() == Some(quote) && self.peek_next() == Some(quote);
        if triple { self.advance(); self.advance(); }
        let mut content = String::new();
        loop {
            match self.peek() {
                None => return Err(LexError::UnterminatedString { span: Span::new(self.line, start_col, start, self.pos) }),
                Some('\n') if !triple => return Err(LexError::UnterminatedString { span: Span::new(self.line, start_col, start, self.pos) }),
                Some('\\') => {
                    self.advance();
                    let esc_start = self.pos;
                    let esc_col = self.column;
                    match self.advance() {
                        Some('n')  => content.push('\n'),
                        Some('t')  => content.push('\t'),
                        Some('r')  => content.push('\r'),
                        Some('\\') => content.push('\\'),
                        Some('\'') => content.push('\''),
                        Some('"')  => content.push('"'),
                        Some('0')  => content.push('\0'),
                        Some(c) => return Err(LexError::InvalidEscapeSequence { seq: format!("\\{}", c), span: Span::new(self.line, esc_col, esc_start, self.pos) }),
                        None => return Err(LexError::UnterminatedString { span: Span::new(self.line, start_col, start, self.pos) }),
                    }
                }
                Some(c) if c == quote => {
                    if triple {
                        if self.peek_next() == Some(quote) && self.source.get(self.pos + 2).copied() == Some(quote) {
                            self.advance(); self.advance(); self.advance();
                            break;
                        } else { content.push(c); self.advance(); }
                    } else { self.advance(); break; }
                }
                Some(c) => { content.push(c); self.advance(); }
            }
        }
        let lexeme: String = self.source[start..self.pos].iter().collect();
        let kind = if is_fstring { TokenKind::FStringLiteral(content) } else { TokenKind::StringLiteral(content) };
        Ok(Token::new(kind, Span::new(self.line, start_col, start, self.pos), lexeme))
    }

    fn lex_number(&mut self, first: char, start: usize, start_col: usize) -> LexResult<Token> {
        let mut raw = String::new();
        raw.push(first);

        if first == '0' {
            match self.peek() {
                Some('x') | Some('X') => {
                    raw.push(self.advance().unwrap());
                    while self.peek().map_or(false, |c| c.is_ascii_hexdigit() || c == '_') {
                        let c = self.advance().unwrap(); if c != '_' { raw.push(c); }
                    }
                    let val = i64::from_str_radix(&raw[2..], 16).map_err(|_| LexError::InvalidNumber { raw: raw.clone(), span: Span::new(self.line, start_col, start, self.pos) })?;
                    let lexeme: String = self.source[start..self.pos].iter().collect();
                    return Ok(Token::new(TokenKind::IntLiteral(val), Span::new(self.line, start_col, start, self.pos), lexeme));
                }
                Some('o') | Some('O') => {
                    raw.push(self.advance().unwrap());
                    while self.peek().map_or(false, |c| matches!(c, '0'..='7') || c == '_') {
                        let c = self.advance().unwrap(); if c != '_' { raw.push(c); }
                    }
                    let val = i64::from_str_radix(&raw[2..], 8).map_err(|_| LexError::InvalidNumber { raw: raw.clone(), span: Span::new(self.line, start_col, start, self.pos) })?;
                    let lexeme: String = self.source[start..self.pos].iter().collect();
                    return Ok(Token::new(TokenKind::IntLiteral(val), Span::new(self.line, start_col, start, self.pos), lexeme));
                }
                Some('b') | Some('B') => {
                    raw.push(self.advance().unwrap());
                    while self.peek().map_or(false, |c| c == '0' || c == '1' || c == '_') {
                        let c = self.advance().unwrap(); if c != '_' { raw.push(c); }
                    }
                    let val = i64::from_str_radix(&raw[2..], 2).map_err(|_| LexError::InvalidNumber { raw: raw.clone(), span: Span::new(self.line, start_col, start, self.pos) })?;
                    let lexeme: String = self.source[start..self.pos].iter().collect();
                    return Ok(Token::new(TokenKind::IntLiteral(val), Span::new(self.line, start_col, start, self.pos), lexeme));
                }
                _ => {}
            }
        }

        while self.peek().map_or(false, |c| c.is_ascii_digit() || c == '_') {
            let c = self.advance().unwrap(); if c != '_' { raw.push(c); }
        }

        let mut is_float = false;
        if self.peek() == Some('.') && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
            is_float = true;
            raw.push(self.advance().unwrap());
            while self.peek().map_or(false, |c| c.is_ascii_digit() || c == '_') {
                let c = self.advance().unwrap(); if c != '_' { raw.push(c); }
            }
        }
        if self.peek() == Some('e') || self.peek() == Some('E') {
            is_float = true;
            raw.push(self.advance().unwrap());
            if self.peek() == Some('+') || self.peek() == Some('-') { raw.push(self.advance().unwrap()); }
            while self.peek().map_or(false, |c| c.is_ascii_digit()) { raw.push(self.advance().unwrap()); }
        }

        let lexeme: String = self.source[start..self.pos].iter().collect();
        if is_float {
            let val: f64 = raw.parse().map_err(|_| LexError::InvalidNumber { raw: raw.clone(), span: Span::new(self.line, start_col, start, self.pos) })?;
            Ok(Token::new(TokenKind::FloatLiteral(val), Span::new(self.line, start_col, start, self.pos), lexeme))
        } else {
            let val: i64 = raw.parse().map_err(|_| LexError::InvalidNumber { raw: raw.clone(), span: Span::new(self.line, start_col, start, self.pos) })?;
            Ok(Token::new(TokenKind::IntLiteral(val), Span::new(self.line, start_col, start, self.pos), lexeme))
        }
    }

    fn lex_identifier(&mut self, first: char, start: usize, start_col: usize) -> Token {
        let mut name = String::new();
        name.push(first);
        while self.peek().map_or(false, |c| c.is_alphanumeric() || c == '_') {
            name.push(self.advance().unwrap());
        }
        let kind = lookup_keyword(&name).unwrap_or(TokenKind::Identifier(name.clone()));
        Token::new(kind, Span::new(self.line, start_col, start, self.pos), name)
    }
}
