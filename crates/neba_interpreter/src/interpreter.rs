use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use neba_parser::ast::*;
use crate::environment::Env;
use crate::error::{InterpResult, RuntimeError};
use crate::value::{FunctionDef, Instance, Value};
use crate::stdlib;

const MAX_DEPTH: usize = 50;

// ── Metadati di una classe (registrati al momento della definizione) ────────

#[derive(Debug, Clone)]
pub struct ClassMeta {
    pub fields:  Vec<Field>,
    pub methods: Vec<Stmt>,
    pub impls:   Vec<Stmt>,
}

// ── Interpreter ────────────────────────────────────────────────────────────

pub struct Interpreter {
    pub env: Env,
    pub class_registry: HashMap<String, ClassMeta>,
    depth: usize,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interp = Self {
            env: Env::new(),
            class_registry: HashMap::new(),
            depth: 0,
        };
        stdlib::register(&mut interp.env);
        interp
    }

    // ── Programma ─────────────────────────────────────────────────────────

    pub fn run(&mut self, program: &Program) -> Result<(), RuntimeError> {
        for stmt in &program.stmts {
            match self.exec_stmt(stmt)? {
                Value::__Return(_) => break,
                _ => {}
            }
        }
        Ok(())
    }

    // ── Statement ─────────────────────────────────────────────────────────

    pub fn exec_stmt(&mut self, stmt: &Stmt) -> InterpResult {
        match &stmt.inner {
            StmtKind::Let { name, value, .. } => {
                let v = self.eval_expr(value)?;
                self.env.define(name, v, false);
                Ok(Value::None)
            }
            StmtKind::Var { name, value, .. } => {
                let v = self.eval_expr(value)?;
                self.env.define(name, v, true);
                Ok(Value::None)
            }
            StmtKind::Assign { target, op, value } => {
                let rhs = self.eval_expr(value)?;
                self.do_assign(target, op, rhs)
            }
            StmtKind::Fn { name, params, body, is_async, .. } => {
                let f = Value::Function(Rc::new(FunctionDef {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: self.env.snapshot(),
                    is_async: *is_async,
                }));
                self.env.define(name, f, false);
                Ok(Value::None)
            }
            StmtKind::Class { name, fields, methods, impls } => {
                self.class_registry.insert(name.clone(), ClassMeta {
                    fields: fields.clone(),
                    methods: methods.clone(),
                    impls: impls.clone(),
                });
                // Il costruttore è una funzione con nome uguale alla classe
                let ctor = Value::Function(Rc::new(FunctionDef {
                    name: name.clone(),
                    params: Vec::new(),
                    body: Vec::new(),
                    closure: self.env.snapshot(),
                    is_async: false,
                }));
                self.env.define(name, ctor, false);
                Ok(Value::None)
            }
            StmtKind::Return(expr) => {
                let v = match expr { Some(e) => self.eval_expr(e)?, None => Value::None };
                Ok(Value::__Return(Box::new(v)))
            }
            StmtKind::While { condition, body } => {
                loop {
                    if !self.eval_expr(condition)?.is_truthy() { break; }
                    match self.exec_block(body)? {
                        Value::__Break               => break,
                        Value::__Continue            => continue,
                        v @ Value::__Return(_) => return Ok(v),
                        _ => {}
                    }
                }
                Ok(Value::None)
            }
            StmtKind::For { var, iterable, body } => {
                let iter_val = self.eval_expr(iterable)?;
                let items = self.to_iter(iter_val)?;
                for item in items {
                    self.env.push_scope();
                    self.env.define(var, item, true);
                    let r = self.exec_block_raw(body)?;
                    self.env.pop_scope();
                    match r {
                        Value::__Break               => break,
                        Value::__Continue            => continue,
                        v @ Value::__Return(_) => return Ok(v),
                        _ => {}
                    }
                }
                Ok(Value::None)
            }
            StmtKind::Break    => Ok(Value::__Break),
            StmtKind::Continue => Ok(Value::__Continue),
            StmtKind::Pass     => Ok(Value::None),
            StmtKind::Trait { .. } | StmtKind::Impl { .. } => Ok(Value::None),
            StmtKind::Mod(n)  => { eprintln!("[warn] mod '{}' not yet supported", n); Ok(Value::None) }
            StmtKind::Use(p)  => { eprintln!("[warn] use '{}' not yet supported", p.join("::")); Ok(Value::None) }
            StmtKind::Expr(e) => self.eval_expr(e),
        }
    }

    // ── Blocchi ────────────────────────────────────────────────────────────

    pub fn exec_block(&mut self, stmts: &[Stmt]) -> InterpResult {
        self.env.push_scope();
        let r = self.exec_block_raw(stmts);
        self.env.pop_scope();
        r
    }

    fn exec_block_raw(&mut self, stmts: &[Stmt]) -> InterpResult {
        let mut last = Value::None;
        for s in stmts {
            last = self.exec_stmt(s)?;
            if matches!(last, Value::__Return(_) | Value::__Break | Value::__Continue) {
                return Ok(last);
            }
        }
        Ok(last)
    }

    // ── Assegnazione ───────────────────────────────────────────────────────

    fn do_assign(&mut self, target: &Expr, op: &AssignOp, rhs: Value) -> InterpResult {
        match &target.inner {
            ExprKind::Ident(name) => {
                let val = self.apply_op(op, name, rhs)?;
                self.env.set(name, val)
                    .map_err(|e| RuntimeError::AssignError { message: e })?;
            }
            ExprKind::Index { object, index } => {
                let obj = self.eval_expr(object)?;
                let idx = self.eval_expr(index)?;
                if let Value::Array(arr) = obj {
                    let i = self.idx(idx, arr.borrow().len())?;
                    let new_val = if let AssignOp::Assign = op {
                        rhs
                    } else {
                        let cur = arr.borrow()[i].clone();
                        self.compute(op, cur, rhs)?
                    };
                    arr.borrow_mut()[i] = new_val;
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "index assignment requires Array".to_string(),
                    });
                }
            }
            ExprKind::Field { object, field } => {
                let obj = self.eval_expr(object)?;
                if let Value::Instance(inst) = obj {
                    let new_val = if let AssignOp::Assign = op {
                        rhs
                    } else {
                        let cur = inst.borrow().fields.get(field)
                            .cloned().unwrap_or(Value::None);
                        self.compute(op, cur, rhs)?
                    };
                    inst.borrow_mut().set(field, new_val);
                } else {
                    return Err(RuntimeError::TypeError {
                        message: format!("cannot set field on {}", obj.type_name()),
                    });
                }
            }
            _ => return Err(RuntimeError::AssignError {
                message: "invalid assignment target".to_string(),
            }),
        }
        Ok(Value::None)
    }

    fn apply_op(&mut self, op: &AssignOp, name: &str, rhs: Value) -> InterpResult {
        if let AssignOp::Assign = op { return Ok(rhs); }
        let cur = self.env.get(name)
            .ok_or_else(|| RuntimeError::UndefinedVariable { name: name.to_string() })?;
        self.compute(op, cur, rhs)
    }

    fn compute(&self, op: &AssignOp, lhs: Value, rhs: Value) -> InterpResult {
        match op {
            AssignOp::Assign    => Ok(rhs),
            AssignOp::AddAssign => self.add(lhs, rhs),
            AssignOp::SubAssign => self.sub(lhs, rhs),
            AssignOp::MulAssign => self.mul(lhs, rhs),
            AssignOp::DivAssign => self.div(lhs, rhs),
            AssignOp::ModAssign => self.modulo(lhs, rhs),
        }
    }

    // ── Espressioni ────────────────────────────────────────────────────────

    pub fn eval_expr(&mut self, expr: &Expr) -> InterpResult {
        match &expr.inner {
            ExprKind::Int(n)    => Ok(Value::Int(*n)),
            ExprKind::Float(f)  => Ok(Value::Float(*f)),
            ExprKind::Bool(b)   => Ok(Value::Bool(*b)),
            ExprKind::Str(s)    => Ok(Value::Str(s.clone())),
            ExprKind::None      => Ok(Value::None),
            ExprKind::FStr(t)   => self.eval_fstring(t),
            ExprKind::Ident(n)  => self.env.get(n)
                .ok_or_else(|| RuntimeError::UndefinedVariable { name: n.clone() }),

            ExprKind::Unary { op, operand } => {
                let v = self.eval_expr(operand)?;
                self.eval_unary(op, v)
            }
            ExprKind::Binary { op, left, right } => {
                // Short-circuit
                match op {
                    BinOp::And => {
                        let l = self.eval_expr(left)?;
                        if !l.is_truthy() { return Ok(Value::Bool(false)); }
                        return Ok(Value::Bool(self.eval_expr(right)?.is_truthy()));
                    }
                    BinOp::Or => {
                        let l = self.eval_expr(left)?;
                        if l.is_truthy() { return Ok(Value::Bool(true)); }
                        return Ok(Value::Bool(self.eval_expr(right)?.is_truthy()));
                    }
                    _ => {}
                }
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_binary(op, l, r)
            }

            ExprKind::If { condition, then_block, elif_branches, else_block } => {
                if self.eval_expr(condition)?.is_truthy() {
                    return self.exec_block(then_block);
                }
                for (c, b) in elif_branches {
                    if self.eval_expr(c)?.is_truthy() {
                        return self.exec_block(b);
                    }
                }
                if let Some(b) = else_block { return self.exec_block(b); }
                Ok(Value::None)
            }

            ExprKind::Match { subject, arms } => {
                let val = self.eval_expr(subject)?;
                for arm in arms {
                    if self.match_pat(&arm.pattern, &val)? {
                        self.env.push_scope();
                        self.bind_pat(&arm.pattern, &val);
                        let r = self.exec_block_raw(&arm.body)?;
                        self.env.pop_scope();
                        return Ok(r);
                    }
                }
                Ok(Value::None)
            }

            ExprKind::Call { callee, args, kwargs } => {
                let fv = self.eval_expr(callee)?;
                let mut av: Vec<Value> = args.iter()
                    .map(|a| self.eval_expr(a))
                    .collect::<Result<_, _>>()?;
                for (_, v) in kwargs { av.push(self.eval_expr(v)?); }
                self.call(fv, av)
            }

            ExprKind::Field { object, field } => {
                let obj = self.eval_expr(object)?;
                self.get_field(obj, field)
            }
            ExprKind::Index { object, index } => {
                let obj = self.eval_expr(object)?;
                let i   = self.eval_expr(index)?;
                self.eval_index(obj, i)
            }
            ExprKind::Array(items) => {
                let vs: Vec<Value> = items.iter()
                    .map(|i| self.eval_expr(i))
                    .collect::<Result<_, _>>()?;
                Ok(Value::Array(Rc::new(RefCell::new(vs))))
            }
            ExprKind::Range { start, end, inclusive } => {
                let s = self.eval_expr(start)?;
                let e = self.eval_expr(end)?;
                self.eval_range(s, e, *inclusive)
            }

            ExprKind::Spawn(inner) => {
                eprintln!("[warn] spawn is synchronous in v0.1.x");
                self.eval_expr(inner)
            }
            ExprKind::Await(inner) => self.eval_expr(inner),

            ExprKind::Some(inner) => { let v = self.eval_expr(inner)?; Ok(Value::Some(Box::new(v))) }
            ExprKind::Ok(inner)   => { let v = self.eval_expr(inner)?; Ok(Value::Ok(Box::new(v)))  }
            ExprKind::Err(inner)  => { let v = self.eval_expr(inner)?; Ok(Value::Err(Box::new(v))) }

            ExprKind::Error => Err(RuntimeError::Generic { message: "AST error node".to_string() }),
        }
    }

    // ── f-string ───────────────────────────────────────────────────────────

    fn eval_fstring(&mut self, template: &str) -> InterpResult {
        let mut out = String::new();
        let chars: Vec<char> = template.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '{' && chars.get(i + 1) != Some(&'{') {
                let start = i + 1;
                let mut depth = 1usize;
                let mut j = start;
                while j < chars.len() && depth > 0 {
                    match chars[j] { '{' => depth += 1, '}' => depth -= 1, _ => {} }
                    j += 1;
                }
                let expr_src: String = chars[start..j - 1].iter().collect();
                let (prog, _, _) = neba_parser::parse(&expr_src);
                if let Some(stmt) = prog.stmts.first() {
                    if let StmtKind::Expr(e) = &stmt.inner {
                        out.push_str(&self.eval_expr(e)?.to_string());
                    }
                }
                i = j;
            } else if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
                out.push('{'); i += 2;
            } else if chars[i] == '}' && chars.get(i + 1) == Some(&'}') {
                out.push('}'); i += 2;
            } else {
                out.push(chars[i]); i += 1;
            }
        }
        Ok(Value::Str(out))
    }

    // ── Operatori unari ────────────────────────────────────────────────────

    fn eval_unary(&self, op: &UnaryOp, v: Value) -> InterpResult {
        match op {
            UnaryOp::Neg => match v {
                Value::Int(n)   => Ok(Value::Int(-n)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err(RuntimeError::TypeError { message: format!("unary '-' on {}", v.type_name()) }),
            },
            UnaryOp::Not    => Ok(Value::Bool(!v.is_truthy())),
            UnaryOp::BitNot => match v {
                Value::Int(n) => Ok(Value::Int(!n)),
                _ => Err(RuntimeError::TypeError { message: format!("'~' on {}", v.type_name()) }),
            },
        }
    }

    // ── Operatori binari ───────────────────────────────────────────────────

    fn eval_binary(&self, op: &BinOp, l: Value, r: Value) -> InterpResult {
        match op {
            BinOp::Add    => self.add(l, r),
            BinOp::Sub    => self.sub(l, r),
            BinOp::Mul    => self.mul(l, r),
            BinOp::Div    => self.div(l, r),
            BinOp::IntDiv => self.intdiv(l, r),
            BinOp::Mod    => self.modulo(l, r),
            BinOp::Pow    => self.pow(l, r),
            BinOp::Eq     => Ok(Value::Bool(l == r)),
            BinOp::Ne     => Ok(Value::Bool(l != r)),
            BinOp::Lt     => Ok(Value::Bool(l <  r)),
            BinOp::Le     => Ok(Value::Bool(l <= r)),
            BinOp::Gt     => Ok(Value::Bool(l >  r)),
            BinOp::Ge     => Ok(Value::Bool(l >= r)),
            BinOp::And | BinOp::Or => unreachable!("handled above"),
            BinOp::BitAnd => self.bitwise(l, r, "&"),
            BinOp::BitOr  => self.bitwise(l, r, "|"),
            BinOp::BitXor => self.bitwise(l, r, "^"),
            BinOp::Shl    => self.bitwise(l, r, "<<"),
            BinOp::Shr    => self.bitwise(l, r, ">>"),
            BinOp::Is     => Ok(Value::Bool(
                std::mem::discriminant(&l) == std::mem::discriminant(&r)
            )),
            BinOp::In    => self.eval_in(l, r),
            BinOp::NotIn => {
                let found = self.eval_in(l, r)?;
                Ok(Value::Bool(!found.is_truthy()))
            }
        }
    }

    fn add(&self, l: Value, r: Value) -> InterpResult {
        match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a),   Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a + *b as f64)),
            (Value::Str(a),   Value::Str(b))   => Ok(Value::Str(format!("{}{}", a, b))),
            _ => Err(RuntimeError::TypeError { message: format!("'+' between {} and {}", l.type_name(), r.type_name()) }),
        }
    }
    fn sub(&self, l: Value, r: Value) -> InterpResult {
        match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a),   Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a - *b as f64)),
            _ => Err(RuntimeError::TypeError { message: format!("'-' between {} and {}", l.type_name(), r.type_name()) }),
        }
    }
    fn mul(&self, l: Value, r: Value) -> InterpResult {
        match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a),   Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a * *b as f64)),
            (Value::Str(s),   Value::Int(n))   => Ok(Value::Str(s.repeat((*n).max(0) as usize))),
            (Value::Int(n),   Value::Str(s))   => Ok(Value::Str(s.repeat((*n).max(0) as usize))),
            _ => Err(RuntimeError::TypeError { message: format!("'*' between {} and {}", l.type_name(), r.type_name()) }),
        }
    }
    fn div(&self, l: Value, r: Value) -> InterpResult {
        let b = r.as_float().ok_or_else(|| RuntimeError::TypeError { message: format!("'/' on {}", r.type_name()) })?;
        if b == 0.0 { return Err(RuntimeError::DivisionByZero); }
        let a = l.as_float().ok_or_else(|| RuntimeError::TypeError { message: format!("'/' on {}", l.type_name()) })?;
        Ok(Value::Float(a / b))
    }
    fn intdiv(&self, l: Value, r: Value) -> InterpResult {
        match (&l, &r) {
            (_, Value::Int(0)) => Err(RuntimeError::DivisionByZero),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
            _ => {
                let b = r.as_float().ok_or_else(|| RuntimeError::TypeError { message: "'//' on non-numeric".to_string() })?;
                if b == 0.0 { return Err(RuntimeError::DivisionByZero); }
                Ok(Value::Int((l.as_float().unwrap() / b).floor() as i64))
            }
        }
    }
    fn modulo(&self, l: Value, r: Value) -> InterpResult {
        match (&l, &r) {
            (_, Value::Int(0)) => Err(RuntimeError::DivisionByZero),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
            _ => {
                let b = r.as_float().ok_or_else(|| RuntimeError::TypeError { message: "'%' on non-numeric".to_string() })?;
                if b == 0.0 { return Err(RuntimeError::DivisionByZero); }
                Ok(Value::Float(l.as_float().unwrap() % b))
            }
        }
    }
    fn pow(&self, l: Value, r: Value) -> InterpResult {
        match (&l, &r) {
            (Value::Int(a), Value::Int(b)) if *b >= 0 => Ok(Value::Int((*a).pow(*b as u32))),
            _ => {
                let a = l.as_float().ok_or_else(|| RuntimeError::TypeError { message: "'**' on non-numeric".to_string() })?;
                let b = r.as_float().ok_or_else(|| RuntimeError::TypeError { message: "'**' on non-numeric".to_string() })?;
                Ok(Value::Float(a.powf(b)))
            }
        }
    }
    fn bitwise(&self, l: Value, r: Value, op: &str) -> InterpResult {
        match (&l, &r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(match op {
                "&"  => a & b, "|" => a | b, "^" => a ^ b,
                "<<" => a << b, ">>" => a >> b, _ => unreachable!(),
            })),
            _ => Err(RuntimeError::TypeError { message: format!("'{}' requires Int", op) }),
        }
    }
    fn eval_in(&self, needle: Value, haystack: Value) -> InterpResult {
        match haystack {
            Value::Array(arr) => Ok(Value::Bool(arr.borrow().contains(&needle))),
            Value::Str(s) => match needle {
                Value::Str(n) => Ok(Value::Bool(s.contains(n.as_str()))),
                _             => Ok(Value::Bool(false)),
            },
            _ => Err(RuntimeError::TypeError { message: format!("'in' on {}", haystack.type_name()) }),
        }
    }

    // ── Range ─────────────────────────────────────────────────────────────

    fn eval_range(&self, start: Value, end: Value, inclusive: bool) -> InterpResult {
        match (&start, &end) {
            (Value::Int(s), Value::Int(e)) => {
                let v: Vec<Value> = if inclusive {
                    (*s..=*e).map(Value::Int).collect()
                } else {
                    (*s..*e).map(Value::Int).collect()
                };
                Ok(Value::Array(Rc::new(RefCell::new(v))))
            }
            _ => Err(RuntimeError::TypeError { message: "range bounds must be Int".to_string() }),
        }
    }

    // ── Field access ──────────────────────────────────────────────────────

    pub fn get_field(&mut self, obj: Value, field: &str) -> InterpResult {
        match &obj {
            Value::Instance(inst) => {
                // 1. Campi dell'istanza
                if let Some(v) = inst.borrow().fields.get(field).cloned() {
                    return Ok(v);
                }
                // 2. Metodi della classe (metodi + impl)
                let class_name = inst.borrow().class_name.clone();
                if let Some(meta) = self.class_registry.get(&class_name).cloned() {
                    let all: Vec<Stmt> = meta.methods.iter().cloned()
                        .chain(meta.impls.iter().flat_map(|i| {
                            if let StmtKind::Impl { methods, .. } = &i.inner {
                                methods.clone()
                            } else {
                                Vec::new()
                            }
                        }))
                        .collect();
                    for m in all {
                        if let StmtKind::Fn { name, params, body, is_async, .. } = &m.inner {
                            if name == field {
                                let mut closure = self.env.snapshot();
                                closure.push_scope();
                                closure.define("self", Value::Instance(inst.clone()), false);
                                return Ok(Value::Function(Rc::new(FunctionDef {
                                    name: name.clone(), params: params.clone(),
                                    body: body.clone(), closure, is_async: *is_async,
                                })));
                            }
                        }
                    }
                }
                Err(RuntimeError::UnknownField {
                    type_name: inst.borrow().class_name.clone(),
                    field: field.to_string(),
                })
            }
            Value::Array(arr) => match field {
                "len" => Ok(Value::Int(arr.borrow().len() as i64)),
                _ => Err(RuntimeError::UnknownField { type_name: "Array".to_string(), field: field.to_string() }),
            },
            Value::Str(s) => {
                let s = s.clone();
                match field {
                    "len"   => Ok(Value::Int(s.chars().count() as i64)),
                    "upper" => Ok(Value::NativeFunction("upper".into(), std::rc::Rc::new(move |_: Vec<Value>| Ok(Value::Str(s.to_uppercase()))))),
                    "lower" => { let s2 = s.clone(); Ok(Value::NativeFunction("lower".into(), std::rc::Rc::new(move |_: Vec<Value>| Ok(Value::Str(s2.to_lowercase()))))) }
                    "trim"  => { let s2 = s.clone(); Ok(Value::NativeFunction("trim".into(),  std::rc::Rc::new(move |_: Vec<Value>| Ok(Value::Str(s2.trim().to_string()))))) }
                    _ => Err(RuntimeError::UnknownField { type_name: "Str".to_string(), field: field.to_string() }),
                }
            }
            _ => Err(RuntimeError::UnknownField {
                type_name: obj.type_name().to_string(),
                field: field.to_string(),
            }),
        }
    }

    // ── Index ─────────────────────────────────────────────────────────────

    fn eval_index(&self, obj: Value, idx: Value) -> InterpResult {
        let i = match &idx { Value::Int(n) => *n, _ => return Err(RuntimeError::TypeError { message: "index must be Int".to_string() }) };
        match obj {
            Value::Array(arr) => {
                let len = arr.borrow().len();
                let actual = self.idx_val(i, len)?;
                Ok(arr.borrow()[actual].clone())
            }
            Value::Str(s) => {
                let chars: Vec<char> = s.chars().collect();
                let actual = self.idx_val(i, chars.len())?;
                Ok(Value::Str(chars[actual].to_string()))
            }
            _ => Err(RuntimeError::TypeError { message: format!("cannot index {}", obj.type_name()) }),
        }
    }

    fn idx(&self, idx: Value, len: usize) -> Result<usize, RuntimeError> {
        match idx { Value::Int(i) => self.idx_val(i, len), _ => Err(RuntimeError::TypeError { message: "index must be Int".to_string() }) }
    }
    fn idx_val(&self, i: i64, len: usize) -> Result<usize, RuntimeError> {
        let a = if i < 0 { len as i64 + i } else { i };
        if a < 0 || a as usize >= len { Err(RuntimeError::IndexOutOfBounds { index: i, len }) }
        else { Ok(a as usize) }
    }

    // ── Chiamata ──────────────────────────────────────────────────────────

    pub fn call(&mut self, func: Value, args: Vec<Value>) -> InterpResult {
        if self.depth >= MAX_DEPTH { return Err(RuntimeError::StackOverflow); }
        match func {
            Value::NativeFunction(_, f) => {
                f(args).map_err(|e| RuntimeError::Generic { message: e })
            }
            Value::Function(def) => {
                self.depth += 1;
                // Costruttore di classe?
                if let Some(meta) = self.class_registry.get(&def.name).cloned() {
                    let inst = Rc::new(RefCell::new(Instance::new(&def.name)));
                    for field in &meta.fields {
                        let v = if let Some(e) = &field.default {
                            self.eval_expr(e)?
                        } else {
                            Value::None
                        };
                        inst.borrow_mut().set(&field.name, v);
                    }
                    // __init__ se esiste
                    let has_init = meta.methods.iter().any(|m| matches!(&m.inner, StmtKind::Fn { name, .. } if name == "__init__"));
                    if has_init {
                        let init = self.get_field(Value::Instance(inst.clone()), "__init__")?;
                        self.call(init, args)?;
                    }
                    self.depth -= 1;
                    return Ok(Value::Instance(inst));
                }
                let r = self.call_fn(&def, args);
                self.depth -= 1;
                r
            }
            other => Err(RuntimeError::NotCallable { type_name: other.type_name().to_string() }),
        }
    }

    fn call_fn(&mut self, def: &FunctionDef, args: Vec<Value>) -> InterpResult {
        let non_self: Vec<_> = def.params.iter().filter(|p| p.name != "self").collect();
        let required = non_self.iter().filter(|p| p.default.is_none()).count();
        let expected = non_self.len();
        if args.len() < required || args.len() > expected {
            return Err(RuntimeError::ArityMismatch { name: def.name.clone(), expected, got: args.len() });
        }
        let saved = std::mem::replace(&mut self.env, def.closure.clone());
        self.env.push_scope();
        let mut it = args.into_iter();
        for p in &def.params {
            if p.name == "self" { continue; }
            let v = match it.next() {
                Some(v) => v,
                None    => self.eval_expr(p.default.as_ref().unwrap())?,
            };
            self.env.define(&p.name, v, true);
        }
        let r = self.exec_block_raw(&def.body);
        self.env.pop_scope();
        self.env = saved;
        match r? {
            Value::__Return(v) => Ok(*v),
            other              => Ok(other),
        }
    }

    // ── Iterazione ────────────────────────────────────────────────────────

    fn to_iter(&self, val: Value) -> Result<Vec<Value>, RuntimeError> {
        match val {
            Value::Array(arr) => Ok(arr.borrow().clone()),
            Value::Str(s)     => Ok(s.chars().map(|c| Value::Str(c.to_string())).collect()),
            _ => Err(RuntimeError::TypeError { message: format!("'{}' is not iterable", val.type_name()) }),
        }
    }

    // ── Pattern matching ───────────────────────────────────────────────────

    fn match_pat(&self, pat: &Pattern, val: &Value) -> Result<bool, RuntimeError> {
        match pat {
            Pattern::Wildcard   => Ok(true),
            Pattern::Ident(_)   => Ok(true),
            Pattern::Literal(lit) => {
                let lv = match lit {
                    ExprKind::Int(n)   => Value::Int(*n),
                    ExprKind::Float(f) => Value::Float(*f),
                    ExprKind::Bool(b)  => Value::Bool(*b),
                    ExprKind::Str(s)   => Value::Str(s.clone()),
                    ExprKind::None     => Value::None,
                    _ => return Ok(false),
                };
                Ok(*val == lv)
            }
            Pattern::Constructor(name, inner) => match (name.as_str(), val) {
                ("Some", Value::Some(v)) => if inner.is_empty() { Ok(true) } else { self.match_pat(&inner[0], v) },
                ("None", Value::None)    => Ok(true),
                ("Ok",   Value::Ok(v))   => if inner.is_empty() { Ok(true) } else { self.match_pat(&inner[0], v) },
                ("Err",  Value::Err(v))  => if inner.is_empty() { Ok(true) } else { self.match_pat(&inner[0], v) },
                _ => Ok(false),
            },
            Pattern::Range { start, end, inclusive } => {
                if let Value::Int(n) = val {
                    let s = self.pat_as_int(start)?;
                    let e = self.pat_as_int(end)?;
                    Ok(if *inclusive { *n >= s && *n <= e } else { *n >= s && *n < e })
                } else { Ok(false) }
            }
            Pattern::Or(pats) => {
                for p in pats { if self.match_pat(p, val)? { return Ok(true); } }
                Ok(false)
            }
            Pattern::Error => Ok(false),
        }
    }

    fn pat_as_int(&self, pat: &Pattern) -> Result<i64, RuntimeError> {
        match pat {
            Pattern::Literal(ExprKind::Int(n)) => Ok(*n),
            _ => Err(RuntimeError::TypeError { message: "range pattern requires Int".to_string() }),
        }
    }

    fn bind_pat(&mut self, pat: &Pattern, val: &Value) {
        match pat {
            Pattern::Ident(name) => { self.env.define(name, val.clone(), true); }
            Pattern::Constructor(_, inner) => {
                let inner_val: Option<&Value> = match val {
                    Value::Some(v) | Value::Ok(v) | Value::Err(v) => Some(v.as_ref()),
                    _ => None,
                };
                if let Some(v) = inner_val {
                    for p in inner { self.bind_pat(p, v); }
                }
            }
            Pattern::Or(pats) => {
                if let Some(p) = pats.first() { self.bind_pat(p, val); }
            }
            _ => {}
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self { Self::new() }
}
