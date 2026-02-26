use neba_lexer::{Span, Token, TokenKind};
use crate::ast::*;
use crate::error::{ParseError, ParseResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    None=0, Or=1, And=2, Not=3, Compare=4, Range=5,
    BitOr=6, BitXor=7, BitAnd=8, Shift=9, Add=10,
    Mul=11, Unary=12, Power=13, Call=14,
}

fn infix_prec(tok: &TokenKind) -> Option<(Prec, bool)> {
    match tok {
        TokenKind::Or | TokenKind::Pipe2                            => Some((Prec::Or,      false)),
        TokenKind::And | TokenKind::Ampersand2                      => Some((Prec::And,     false)),
        TokenKind::EqualEqual | TokenKind::BangEqual
        | TokenKind::Less | TokenKind::LessEqual
        | TokenKind::Greater | TokenKind::GreaterEqual
        | TokenKind::Is | TokenKind::In                             => Some((Prec::Compare, false)),
        TokenKind::DotDot | TokenKind::DotDotEqual                  => Some((Prec::Range,   false)),
        TokenKind::Pipe                                             => Some((Prec::BitOr,   false)),
        TokenKind::Caret                                            => Some((Prec::BitXor,  false)),
        TokenKind::Ampersand                                        => Some((Prec::BitAnd,  false)),
        TokenKind::LessLess | TokenKind::GreaterGreater             => Some((Prec::Shift,   false)),
        TokenKind::Plus | TokenKind::Minus                          => Some((Prec::Add,     false)),
        TokenKind::Star | TokenKind::Slash
        | TokenKind::SlashSlash | TokenKind::Percent                => Some((Prec::Mul,     false)),
        TokenKind::StarStar                                         => Some((Prec::Power,   true)),
        TokenKind::LParen | TokenKind::LBracket | TokenKind::Dot   => Some((Prec::Call,    false)),
        _ => None,
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0, errors: Vec::new() }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }
    fn peek_kind(&self) -> &TokenKind { &self.peek().kind }
    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() - 1 { self.pos += 1; }
        tok
    }
    fn current_span(&self) -> Span { self.peek().span.clone() }
    fn match_tok(&mut self, kind: &TokenKind) -> bool {
        if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind) {
            self.advance(); true
        } else { false }
    }
    fn expect(&mut self, kind: &TokenKind, label: &str) -> ParseResult<Token> {
        if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind) {
            Ok(self.advance().clone())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: label.to_string(),
                found: self.peek_kind().clone(),
                span: self.current_span(),
            })
        }
    }
    fn skip_newlines(&mut self) {
        while matches!(self.peek_kind(), TokenKind::Newline) { self.advance(); }
    }
    fn expect_newline(&mut self) {
        if matches!(self.peek_kind(), TokenKind::Newline | TokenKind::Eof) { self.advance(); }
    }
    fn error_expr(&mut self, err: ParseError) -> Expr {
        let span = self.current_span();
        self.errors.push(err);
        while !matches!(self.peek_kind(), TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent) {
            self.advance();
        }
        Node::new(ExprKind::Error, span)
    }
    fn error_stmt(&mut self, err: ParseError) -> Stmt {
        let span = self.current_span();
        self.errors.push(err);
        while !matches!(self.peek_kind(), TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent) {
            self.advance();
        }
        self.expect_newline();
        Node::new(StmtKind::Expr(Node::new(ExprKind::Error, span.clone())), span)
    }

    pub fn parse(&mut self) -> Program {
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek_kind(), TokenKind::Eof) {
            stmts.push(self.parse_stmt());
            self.skip_newlines();
        }
        Program { stmts }
    }

    fn parse_stmt(&mut self) -> Stmt {
        let span = self.current_span();
        match self.peek_kind().clone() {
            TokenKind::Let      => self.parse_let(false),
            TokenKind::Var      => self.parse_let(true),
            TokenKind::Fn       => self.parse_fn(false),
            TokenKind::Async    => self.parse_async_fn(),
            TokenKind::Class    => self.parse_class(),
            TokenKind::Trait    => self.parse_trait(),
            TokenKind::Impl     => self.parse_impl(),
            TokenKind::While    => self.parse_while(),
            TokenKind::For      => self.parse_for(),
            TokenKind::Return   => self.parse_return(),
            TokenKind::Break    => { self.advance(); self.expect_newline(); Node::new(StmtKind::Break, span) }
            TokenKind::Continue => { self.advance(); self.expect_newline(); Node::new(StmtKind::Continue, span) }
            TokenKind::Pass     => { self.advance(); self.expect_newline(); Node::new(StmtKind::Pass, span) }
            TokenKind::Mod      => self.parse_mod(),
            TokenKind::Use      => self.parse_use(),
            _                   => self.parse_expr_or_assign(),
        }
    }

    fn parse_let(&mut self, is_var: bool) -> Stmt {
        let span = self.current_span();
        self.advance();
        let name = match self.peek_kind().clone() {
            TokenKind::Identifier(s) => { self.advance(); s }
            _ => return self.error_stmt(ParseError::UnexpectedToken {
                expected: "identifier".to_string(), found: self.peek_kind().clone(), span: self.current_span(),
            }),
        };
        let ty = if self.match_tok(&TokenKind::Colon) { Some(self.parse_type()) } else { None };
        if let Err(e) = self.expect(&TokenKind::Equal, "'='") { return self.error_stmt(e); }
        let value = self.parse_expr(Prec::None);
        self.expect_newline();
        let kind = if is_var { StmtKind::Var { name, ty, value } } else { StmtKind::Let { name, ty, value } };
        Node::new(kind, span)
    }

    fn parse_fn(&mut self, is_async: bool) -> Stmt {
        let span = self.current_span();
        self.advance();
        let name = match self.peek_kind().clone() {
            TokenKind::Identifier(s) => { self.advance(); s }
            _ => return self.error_stmt(ParseError::UnexpectedToken {
                expected: "function name".to_string(), found: self.peek_kind().clone(), span: self.current_span(),
            }),
        };
        if let Err(e) = self.expect(&TokenKind::LParen, "'('") { return self.error_stmt(e); }
        let params = self.parse_params();
        if let Err(e) = self.expect(&TokenKind::RParen, "')'") { return self.error_stmt(e); }
        let return_ty = if self.match_tok(&TokenKind::Arrow) { Some(self.parse_type()) } else { None };
        self.expect_newline();
        let body = self.parse_block();
        Node::new(StmtKind::Fn { name, params, return_ty, body, is_async }, span)
    }

    fn parse_async_fn(&mut self) -> Stmt { self.advance(); self.parse_fn(true) }

    fn parse_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::RParen | TokenKind::Eof) {
            let span = self.current_span();
            let name = match self.peek_kind().clone() {
                TokenKind::Self_         => { self.advance(); "self".to_string() }
                TokenKind::Identifier(s) => { self.advance(); s }
                _ => break,
            };
            let ty      = if self.match_tok(&TokenKind::Colon) { Some(self.parse_type()) } else { None };
            let default = if self.match_tok(&TokenKind::Equal) { Some(self.parse_expr(Prec::None)) } else { None };
            params.push(Param { name, ty, default, span });
            if !self.match_tok(&TokenKind::Comma) { break; }
        }
        params
    }

    fn parse_class(&mut self) -> Stmt {
        let span = self.current_span();
        self.advance();
        let name = match self.peek_kind().clone() {
            TokenKind::Identifier(s) => { self.advance(); s }
            _ => return self.error_stmt(ParseError::UnexpectedToken {
                expected: "class name".to_string(), found: self.peek_kind().clone(), span: self.current_span(),
            }),
        };
        self.expect_newline();
        if let Err(e) = self.expect(&TokenKind::Indent, "indented class body") { return self.error_stmt(e); }
        let mut fields = Vec::new(); let mut methods = Vec::new(); let mut impls = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek_kind(), TokenKind::Dedent | TokenKind::Eof) {
            match self.peek_kind().clone() {
                TokenKind::Fn | TokenKind::Async => methods.push(self.parse_stmt()),
                TokenKind::Impl                  => impls.push(self.parse_stmt()),
                TokenKind::Newline               => { self.advance(); }
                _                                => fields.push(self.parse_field()),
            }
        }
        self.match_tok(&TokenKind::Dedent);
        Node::new(StmtKind::Class { name, fields, methods, impls }, span)
    }

    fn parse_field(&mut self) -> Field {
        let span = self.current_span();
        let name = match self.peek_kind().clone() {
            TokenKind::Identifier(s) => { self.advance(); s }
            _ => { self.advance(); "?".to_string() }
        };
        let ty      = if self.match_tok(&TokenKind::Colon) { Some(self.parse_type()) } else { None };
        let default = if self.match_tok(&TokenKind::Equal) { Some(self.parse_expr(Prec::None)) } else { None };
        self.expect_newline();
        Field { name, ty, default, span }
    }

    fn parse_trait(&mut self) -> Stmt {
        let span = self.current_span();
        self.advance();
        let name = match self.peek_kind().clone() {
            TokenKind::Identifier(s) => { self.advance(); s }
            _ => return self.error_stmt(ParseError::UnexpectedToken {
                expected: "trait name".to_string(), found: self.peek_kind().clone(), span: self.current_span(),
            }),
        };
        self.expect_newline();
        if let Err(e) = self.expect(&TokenKind::Indent, "indented trait body") { return self.error_stmt(e); }
        let mut methods = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek_kind(), TokenKind::Dedent | TokenKind::Eof) {
            if matches!(self.peek_kind(), TokenKind::Newline) { self.advance(); continue; }
            methods.push(self.parse_stmt());
        }
        self.match_tok(&TokenKind::Dedent);
        Node::new(StmtKind::Trait { name, methods }, span)
    }

    fn parse_impl(&mut self) -> Stmt {
        let span = self.current_span();
        self.advance();
        let trait_name = match self.peek_kind().clone() {
            TokenKind::Identifier(s) => { self.advance(); s }
            _ => return self.error_stmt(ParseError::UnexpectedToken {
                expected: "trait name".to_string(), found: self.peek_kind().clone(), span: self.current_span(),
            }),
        };
        let for_type = if self.match_tok(&TokenKind::For) {
            match self.peek_kind().clone() {
                TokenKind::Identifier(s) => { self.advance(); Some(s) }
                _ => None,
            }
        } else { None };
        self.expect_newline();
        if let Err(e) = self.expect(&TokenKind::Indent, "indented impl body") { return self.error_stmt(e); }
        let mut methods = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek_kind(), TokenKind::Dedent | TokenKind::Eof) {
            if matches!(self.peek_kind(), TokenKind::Newline) { self.advance(); continue; }
            methods.push(self.parse_stmt());
        }
        self.match_tok(&TokenKind::Dedent);
        Node::new(StmtKind::Impl { trait_name, for_type, methods }, span)
    }

    fn parse_while(&mut self) -> Stmt {
        let span = self.current_span();
        self.advance();
        let condition = self.parse_expr(Prec::None);
        self.expect_newline();
        let body = self.parse_block();
        Node::new(StmtKind::While { condition, body }, span)
    }

    fn parse_for(&mut self) -> Stmt {
        let span = self.current_span();
        self.advance();
        let var = match self.peek_kind().clone() {
            TokenKind::Identifier(s) => { self.advance(); s }
            _ => return self.error_stmt(ParseError::UnexpectedToken {
                expected: "loop variable".to_string(), found: self.peek_kind().clone(), span: self.current_span(),
            }),
        };
        if let Err(e) = self.expect(&TokenKind::In, "'in'") { return self.error_stmt(e); }
        let iterable = self.parse_expr(Prec::None);
        self.expect_newline();
        let body = self.parse_block();
        Node::new(StmtKind::For { var, iterable, body }, span)
    }

    fn parse_return(&mut self) -> Stmt {
        let span = self.current_span();
        self.advance();
        let value = if matches!(self.peek_kind(), TokenKind::Newline | TokenKind::Eof) {
            None
        } else { Some(self.parse_expr(Prec::None)) };
        self.expect_newline();
        Node::new(StmtKind::Return(value), span)
    }

    fn parse_mod(&mut self) -> Stmt {
        let span = self.current_span();
        self.advance();
        let name = match self.peek_kind().clone() {
            TokenKind::Identifier(s) => { self.advance(); s }
            _ => "?".to_string(),
        };
        self.expect_newline();
        Node::new(StmtKind::Mod(name), span)
    }

    fn parse_use(&mut self) -> Stmt {
        let span = self.current_span();
        self.advance();
        let mut path = Vec::new();
        loop {
            match self.peek_kind().clone() {
                TokenKind::Identifier(s) => { self.advance(); path.push(s); }
                _ => break,
            }
            if !self.match_tok(&TokenKind::ColonColon) { break; }
        }
        self.expect_newline();
        Node::new(StmtKind::Use(path), span)
    }

    fn parse_expr_or_assign(&mut self) -> Stmt {
        let span = self.current_span();
        let expr = self.parse_expr(Prec::None);
        let op = match self.peek_kind() {
            TokenKind::Equal        => Some(AssignOp::Assign),
            TokenKind::PlusEqual    => Some(AssignOp::AddAssign),
            TokenKind::MinusEqual   => Some(AssignOp::SubAssign),
            TokenKind::StarEqual    => Some(AssignOp::MulAssign),
            TokenKind::SlashEqual   => Some(AssignOp::DivAssign),
            TokenKind::PercentEqual => Some(AssignOp::ModAssign),
            _ => None,
        };
        if let Some(op) = op {
            match &expr.inner {
                ExprKind::Ident(_) | ExprKind::Field { .. } | ExprKind::Index { .. } => {}
                _ => self.errors.push(ParseError::InvalidAssignTarget { span: expr.span.clone() }),
            }
            self.advance();
            let value = self.parse_expr(Prec::None);
            self.expect_newline();
            return Node::new(StmtKind::Assign { target: expr, op, value }, span);
        }
        self.expect_newline();
        Node::new(StmtKind::Expr(expr), span)
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        if !matches!(self.peek_kind(), TokenKind::Indent) {
            self.errors.push(ParseError::MissingIndent { span: self.current_span() });
            return Vec::new();
        }
        self.advance();
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek_kind(), TokenKind::Dedent | TokenKind::Eof) {
            if matches!(self.peek_kind(), TokenKind::Newline) { self.advance(); continue; }
            stmts.push(self.parse_stmt());
        }
        self.match_tok(&TokenKind::Dedent);
        stmts
    }

    fn parse_type(&mut self) -> TypeExpr {
        let span = self.current_span();
        match self.peek_kind().clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                if self.match_tok(&TokenKind::LBracket) {
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parse_type());
                        if !self.match_tok(&TokenKind::Comma) { break; }
                    }
                    self.match_tok(&TokenKind::RBracket);
                    Node::new(TypeKind::Generic(name, args), span)
                } else {
                    Node::new(TypeKind::Named(name), span)
                }
            }
            _ => {
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "type".to_string(), found: self.peek_kind().clone(), span: span.clone(),
                });
                Node::new(TypeKind::Error, span)
            }
        }
    }

    fn parse_expr(&mut self, min_prec: Prec) -> Expr {
        let mut left = self.parse_prefix();
        loop {
            let kind = self.peek_kind().clone();
            // not in
            if kind == TokenKind::Not {
                let span = self.current_span();
                let next_is_in = self.tokens.get(self.pos + 1).map_or(false, |t| t.kind == TokenKind::In);
                if next_is_in && Prec::Compare > min_prec {
                    self.advance(); self.advance();
                    let right = self.parse_expr(Prec::Compare);
                    left = Node::new(ExprKind::Binary { op: BinOp::NotIn, left: Box::new(left), right: Box::new(right) }, span);
                    continue;
                }
            }
            let (prec, right_assoc) = match infix_prec(&kind) { Some(p) => p, None => break };
            if prec <= min_prec && !right_assoc { break; }
            if prec < min_prec { break; }
            left = self.parse_infix(left, prec, right_assoc);
        }
        left
    }

    fn parse_prefix(&mut self) -> Expr {
        let span = self.current_span();
        match self.peek_kind().clone() {
            TokenKind::IntLiteral(n)     => { self.advance(); Node::new(ExprKind::Int(n), span) }
            TokenKind::FloatLiteral(f)   => { self.advance(); Node::new(ExprKind::Float(f), span) }
            TokenKind::BoolLiteral(b)    => { self.advance(); Node::new(ExprKind::Bool(b), span) }
            TokenKind::StringLiteral(s)  => { self.advance(); Node::new(ExprKind::Str(s), span) }
            TokenKind::FStringLiteral(s) => { self.advance(); Node::new(ExprKind::FStr(s), span) }
            TokenKind::NoneLiteral       => { self.advance(); Node::new(ExprKind::None, span) }
            TokenKind::Identifier(s)     => { self.advance(); Node::new(ExprKind::Ident(s), span) }
            TokenKind::Self_             => { self.advance(); Node::new(ExprKind::Ident("self".to_string()), span) }
            TokenKind::Minus => {
                self.advance();
                let op = self.parse_expr(Prec::Unary);
                Node::new(ExprKind::Unary { op: UnaryOp::Neg, operand: Box::new(op) }, span)
            }
            TokenKind::Not => {
                self.advance();
                let op = self.parse_expr(Prec::Not);
                Node::new(ExprKind::Unary { op: UnaryOp::Not, operand: Box::new(op) }, span)
            }
            TokenKind::Tilde => {
                self.advance();
                let op = self.parse_expr(Prec::Unary);
                Node::new(ExprKind::Unary { op: UnaryOp::BitNot, operand: Box::new(op) }, span)
            }
            TokenKind::LParen => {
                self.advance();
                let e = self.parse_expr(Prec::None);
                self.match_tok(&TokenKind::RParen);
                e
            }
            TokenKind::LBracket => self.parse_array_literal(),
            TokenKind::If       => self.parse_if_expr(),
            TokenKind::Match    => self.parse_match_expr(),
            TokenKind::Spawn => {
                self.advance();
                let e = self.parse_expr(Prec::None);
                Node::new(ExprKind::Spawn(Box::new(e)), span)
            }
            TokenKind::Await => {
                self.advance();
                let e = self.parse_expr(Prec::None);
                Node::new(ExprKind::Await(Box::new(e)), span)
            }
            TokenKind::Some => {
                self.advance(); self.match_tok(&TokenKind::LParen);
                let e = self.parse_expr(Prec::None);
                self.match_tok(&TokenKind::RParen);
                Node::new(ExprKind::Some(Box::new(e)), span)
            }
            TokenKind::Ok => {
                self.advance(); self.match_tok(&TokenKind::LParen);
                let e = self.parse_expr(Prec::None);
                self.match_tok(&TokenKind::RParen);
                Node::new(ExprKind::Ok(Box::new(e)), span)
            }
            TokenKind::Err => {
                self.advance(); self.match_tok(&TokenKind::LParen);
                let e = self.parse_expr(Prec::None);
                self.match_tok(&TokenKind::RParen);
                Node::new(ExprKind::Err(Box::new(e)), span)
            }
            other => self.error_expr(ParseError::UnexpectedToken {
                expected: "expression".to_string(), found: other, span: span.clone(),
            }),
        }
    }

    fn parse_infix(&mut self, left: Expr, prec: Prec, right_assoc: bool) -> Expr {
        let span = left.span.clone();
        let kind = self.peek_kind().clone();
        match &kind {
            TokenKind::LParen => {
                self.advance();
                let (args, kwargs) = self.parse_call_args();
                self.match_tok(&TokenKind::RParen);
                Node::new(ExprKind::Call { callee: Box::new(left), args, kwargs }, span)
            }
            TokenKind::LBracket => {
                self.advance();
                let index = self.parse_expr(Prec::None);
                self.match_tok(&TokenKind::RBracket);
                Node::new(ExprKind::Index { object: Box::new(left), index: Box::new(index) }, span)
            }
            TokenKind::Dot => {
                self.advance();
                let field = match self.peek_kind().clone() {
                    TokenKind::Identifier(s) => { self.advance(); s }
                    _ => "?".to_string(),
                };
                Node::new(ExprKind::Field { object: Box::new(left), field }, span)
            }
            TokenKind::DotDot => {
                self.advance();
                let right = self.parse_expr(Prec::Add);
                Node::new(ExprKind::Range { start: Box::new(left), end: Box::new(right), inclusive: false }, span)
            }
            TokenKind::DotDotEqual => {
                self.advance();
                let right = self.parse_expr(Prec::Add);
                Node::new(ExprKind::Range { start: Box::new(left), end: Box::new(right), inclusive: true }, span)
            }
            _ => {
                let op = self.token_to_binop(&kind);
                self.advance();
                let next_prec = if right_assoc { Prec::try_from(prec as u8 - 1).unwrap_or(Prec::None) } else { prec };
                let right = self.parse_expr(next_prec);
                Node::new(ExprKind::Binary { op, left: Box::new(left), right: Box::new(right) }, span)
            }
        }
    }

    fn token_to_binop(&self, kind: &TokenKind) -> BinOp {
        match kind {
            TokenKind::Plus          => BinOp::Add,
            TokenKind::Minus         => BinOp::Sub,
            TokenKind::Star          => BinOp::Mul,
            TokenKind::Slash         => BinOp::Div,
            TokenKind::SlashSlash    => BinOp::IntDiv,
            TokenKind::Percent       => BinOp::Mod,
            TokenKind::StarStar      => BinOp::Pow,
            TokenKind::EqualEqual    => BinOp::Eq,
            TokenKind::BangEqual     => BinOp::Ne,
            TokenKind::Less          => BinOp::Lt,
            TokenKind::LessEqual     => BinOp::Le,
            TokenKind::Greater       => BinOp::Gt,
            TokenKind::GreaterEqual  => BinOp::Ge,
            TokenKind::And | TokenKind::Ampersand2 => BinOp::And,
            TokenKind::Or  | TokenKind::Pipe2      => BinOp::Or,
            TokenKind::Ampersand     => BinOp::BitAnd,
            TokenKind::Pipe          => BinOp::BitOr,
            TokenKind::Caret         => BinOp::BitXor,
            TokenKind::LessLess      => BinOp::Shl,
            TokenKind::GreaterGreater=> BinOp::Shr,
            TokenKind::Is            => BinOp::Is,
            TokenKind::In            => BinOp::In,
            _                        => BinOp::Add,
        }
    }

    fn parse_call_args(&mut self) -> (Vec<Expr>, Vec<(String, Expr)>) {
        let mut args = Vec::new(); let mut kwargs = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::RParen | TokenKind::Eof) {
            let is_kwarg = matches!(self.peek_kind(), TokenKind::Identifier(_))
                && self.tokens.get(self.pos + 1).map_or(false, |t| t.kind == TokenKind::Equal);
            if is_kwarg {
                let name = match self.peek_kind().clone() {
                    TokenKind::Identifier(s) => { self.advance(); s }
                    _ => break,
                };
                self.advance();
                kwargs.push((name, self.parse_expr(Prec::None)));
            } else {
                args.push(self.parse_expr(Prec::None));
            }
            if !self.match_tok(&TokenKind::Comma) { break; }
        }
        (args, kwargs)
    }

    fn parse_array_literal(&mut self) -> Expr {
        let span = self.current_span();
        self.advance();
        let mut items = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::RBracket | TokenKind::Eof) {
            items.push(self.parse_expr(Prec::None));
            if !self.match_tok(&TokenKind::Comma) { break; }
        }
        self.match_tok(&TokenKind::RBracket);
        Node::new(ExprKind::Array(items), span)
    }

    fn parse_if_expr(&mut self) -> Expr {
        let span = self.current_span();
        self.advance();
        let condition = self.parse_expr(Prec::None);
        self.expect_newline();
        let then_block = self.parse_block();
        let mut elif_branches = Vec::new();
        let mut else_block = None;
        loop {
            self.skip_newlines();
            match self.peek_kind().clone() {
                TokenKind::Elif => {
                    self.advance();
                    let cond = self.parse_expr(Prec::None);
                    self.expect_newline();
                    elif_branches.push((cond, self.parse_block()));
                }
                TokenKind::Else => {
                    self.advance();
                    self.expect_newline();
                    else_block = Some(self.parse_block());
                    break;
                }
                _ => break,
            }
        }
        Node::new(ExprKind::If { condition: Box::new(condition), then_block, elif_branches, else_block }, span)
    }

    fn parse_match_expr(&mut self) -> Expr {
        let span = self.current_span();
        self.advance();
        let subject = self.parse_expr(Prec::None);
        self.expect_newline();
        if !matches!(self.peek_kind(), TokenKind::Indent) {
            self.errors.push(ParseError::MissingIndent { span: self.current_span() });
            return Node::new(ExprKind::Match { subject: Box::new(subject), arms: Vec::new() }, span);
        }
        self.advance();
        let mut arms = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek_kind(), TokenKind::Dedent | TokenKind::Eof) {
            if matches!(self.peek_kind(), TokenKind::Newline) { self.advance(); continue; }
            if !matches!(self.peek_kind(), TokenKind::Case) { break; }
            let arm_span = self.current_span();
            self.advance();
            let pattern = self.parse_pattern();
            let body = if self.match_tok(&TokenKind::FatArrow) {
                let stmt = self.parse_stmt();
                vec![stmt]
            } else {
                self.expect_newline();
                self.parse_block()
            };
            arms.push(MatchArm { pattern, body, span: arm_span });
        }
        self.match_tok(&TokenKind::Dedent);
        Node::new(ExprKind::Match { subject: Box::new(subject), arms }, span)
    }

    fn parse_pattern(&mut self) -> Pattern {
        let mut pats = vec![self.parse_single_pattern()];
        while self.match_tok(&TokenKind::Pipe) { pats.push(self.parse_single_pattern()); }
        if pats.len() == 1 { pats.remove(0) } else { Pattern::Or(pats) }
    }

    fn parse_single_pattern(&mut self) -> Pattern {
        match self.peek_kind().clone() {
            TokenKind::Underscore      => { self.advance(); Pattern::Wildcard }
            TokenKind::IntLiteral(n)   => { self.advance(); self.maybe_range_pattern(Pattern::Literal(ExprKind::Int(n))) }
            TokenKind::FloatLiteral(f) => { self.advance(); Pattern::Literal(ExprKind::Float(f)) }
            TokenKind::StringLiteral(s)=> { self.advance(); Pattern::Literal(ExprKind::Str(s)) }
            TokenKind::BoolLiteral(b)  => { self.advance(); Pattern::Literal(ExprKind::Bool(b)) }
            TokenKind::NoneLiteral     => { self.advance(); Pattern::Literal(ExprKind::None) }
            TokenKind::Some | TokenKind::Ok | TokenKind::Err => {
                let name = match self.peek_kind() {
                    TokenKind::Some => "Some", TokenKind::Ok => "Ok", _ => "Err",
                }.to_string();
                self.advance();
                let mut inner = Vec::new();
                if self.match_tok(&TokenKind::LParen) {
                    inner.push(self.parse_pattern());
                    self.match_tok(&TokenKind::RParen);
                }
                Pattern::Constructor(name, inner)
            }
            TokenKind::Identifier(s) => {
                self.advance();
                if self.match_tok(&TokenKind::LParen) {
                    let mut inner = Vec::new();
                    while !matches!(self.peek_kind(), TokenKind::RParen | TokenKind::Eof) {
                        inner.push(self.parse_pattern());
                        if !self.match_tok(&TokenKind::Comma) { break; }
                    }
                    self.match_tok(&TokenKind::RParen);
                    Pattern::Constructor(s, inner)
                } else {
                    Pattern::Ident(s)
                }
            }
            _ => {
                self.errors.push(ParseError::InvalidPattern { span: self.current_span() });
                Pattern::Error
            }
        }
    }

    fn maybe_range_pattern(&mut self, start: Pattern) -> Pattern {
        match self.peek_kind() {
            TokenKind::DotDot      => { self.advance(); let end = self.parse_single_pattern(); Pattern::Range { start: Box::new(start), end: Box::new(end), inclusive: false } }
            TokenKind::DotDotEqual => { self.advance(); let end = self.parse_single_pattern(); Pattern::Range { start: Box::new(start), end: Box::new(end), inclusive: true  } }
            _ => start,
        }
    }
}

impl Prec {
    fn try_from(v: u8) -> Option<Self> {
        match v {
            0 => Some(Prec::None), 1 => Some(Prec::Or),    2 => Some(Prec::And),
            3 => Some(Prec::Not),  4 => Some(Prec::Compare),5 => Some(Prec::Range),
            6 => Some(Prec::BitOr),7 => Some(Prec::BitXor), 8 => Some(Prec::BitAnd),
            9 => Some(Prec::Shift),10=> Some(Prec::Add),    11=> Some(Prec::Mul),
            12=> Some(Prec::Unary),13=> Some(Prec::Power),  14=> Some(Prec::Call),
            _ => None,
        }
    }
}
