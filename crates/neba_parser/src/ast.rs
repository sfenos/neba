use neba_lexer::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Node<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> Node<T> {
    pub fn new(inner: T, span: Span) -> Self {
        Node { inner, span }
    }
}

pub type Expr     = Node<ExprKind>;
pub type Stmt     = Node<StmtKind>;
pub type TypeExpr = Node<TypeKind>;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Named(String),
    Generic(String, Vec<TypeExpr>),
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    FStr(String),
    None,
    Ident(String),
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Unary  { op: UnaryOp, operand: Box<Expr> },
    Call   { callee: Box<Expr>, args: Vec<Expr>, kwargs: Vec<(String, Expr)> },
    Field  { object: Box<Expr>, field: String },
    Index  { object: Box<Expr>, index: Box<Expr> },
    Array(Vec<Expr>),
    Range  { start: Box<Expr>, end: Box<Expr>, inclusive: bool },
    If {
        condition: Box<Expr>,
        then_block: Vec<Stmt>,
        elif_branches: Vec<(Expr, Vec<Stmt>)>,
        else_block: Option<Vec<Stmt>>,
    },
    Match  { subject: Box<Expr>, arms: Vec<MatchArm> },
    Spawn(Box<Expr>),
    Await(Box<Expr>),
    Some(Box<Expr>),
    Ok(Box<Expr>),
    Err(Box<Expr>),
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Literal(ExprKind),
    Ident(String),
    Constructor(String, Vec<Pattern>),
    Range { start: Box<Pattern>, end: Box<Pattern>, inclusive: bool },
    Or(Vec<Pattern>),
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, IntDiv, Mod, Pow,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
    Is, In, NotIn,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp { Neg, Not, BitNot }

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Assign, AddAssign, SubAssign, MulAssign, DivAssign, ModAssign,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Let    { name: String, ty: Option<TypeExpr>, value: Expr },
    Var    { name: String, ty: Option<TypeExpr>, value: Expr },
    Assign { target: Expr, op: AssignOp, value: Expr },
    Fn {
        name: String,
        params: Vec<Param>,
        return_ty: Option<TypeExpr>,
        body: Vec<Stmt>,
        is_async: bool,
    },
    Class  { name: String, fields: Vec<Field>, methods: Vec<Stmt>, impls: Vec<Stmt> },
    Trait  { name: String, methods: Vec<Stmt> },
    Impl   { trait_name: String, for_type: Option<String>, methods: Vec<Stmt> },
    While  { condition: Expr, body: Vec<Stmt> },
    For    { var: String, iterable: Expr, body: Vec<Stmt> },
    Return(Option<Expr>),
    Break,
    Continue,
    Pass,
    Mod(String),
    Use(Vec<String>),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Option<TypeExpr>,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Option<TypeExpr>,
    pub default: Option<Expr>,
    pub span: Span,
}
