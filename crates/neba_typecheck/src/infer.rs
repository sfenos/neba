use neba_lexer::Span;
use neba_parser::ast::*;
use crate::types::Type;
use crate::error::TypeError;
use crate::env::TypeEnv;

/// Inferisce il tipo di un'espressione, accumulando errori in `errors`.
pub fn infer_expr(expr: &Expr, env: &mut TypeEnv, errors: &mut Vec<TypeError>) -> Type {
    let span = expr.span.clone();
    match &expr.inner {
        // ── Letterali ─────────────────────────────────────────────────────
        ExprKind::Int(_)   => Type::Int,
        ExprKind::Float(_) => Type::Float,
        ExprKind::Bool(_)  => Type::Bool,
        ExprKind::Str(_) | ExprKind::FStr(_) => Type::Str,
        ExprKind::None     => Type::None,
        ExprKind::Error    => Type::Unknown,

        // ── Identificatore ────────────────────────────────────────────────
        ExprKind::Ident(name) => {
            match env.lookup(name) {
                Some(info) => info.ty.clone(),
                None => {
                    errors.push(TypeError::undefined(name, span));
                    Type::Unknown
                }
            }
        }

        // ── Operatori unari ───────────────────────────────────────────────
        ExprKind::Unary { op, operand } => {
            let t = infer_expr(operand, env, errors);
            match op {
                UnaryOp::Neg => {
                    if !t.is_numeric() && !matches!(t, Type::Unknown | Type::Any) {
                        errors.push(TypeError::error(
                            format!("unary '-' cannot be applied to '{}'", t), span,
                        ));
                    }
                    t
                }
                UnaryOp::Not => Type::Bool,
                UnaryOp::BitNot => {
                    if !matches!(t, Type::Int | Type::Unknown | Type::Any) {
                        errors.push(TypeError::error(
                            format!("unary '~' requires Int, got '{}'", t), span,
                        ));
                    }
                    Type::Int
                }
            }
        }

        // ── Operatori binari ──────────────────────────────────────────────
        ExprKind::Binary { op, left, right } => {
            let lt = infer_expr(left, env, errors);
            let rt = infer_expr(right, env, errors);
            infer_binary(op, &lt, &rt, span, errors)
        }

        // ── Array ─────────────────────────────────────────────────────────
        ExprKind::Array(elems) => {
            if elems.is_empty() {
                return Type::Array(Box::new(Type::Unknown));
            }
            let first = infer_expr(&elems[0], env, errors);
            let mut unified = first.clone();
            for elem in elems.iter().skip(1) {
                let t = infer_expr(elem, env, errors);
                match Type::unify(&unified, &t) {
                    Some(u) => unified = u,
                    None => {
                        errors.push(TypeError::error(
                            format!("array elements have inconsistent types: '{}' and '{}'", unified, t),
                            elem.span.clone(),
                        ));
                    }
                }
            }
            Type::Array(Box::new(unified))
        }

        // ── Range ─────────────────────────────────────────────────────────
        ExprKind::Range { start, end, .. } => {
            let st = infer_expr(start, env, errors);
            let et = infer_expr(end, env, errors);
            if !matches!(st, Type::Int | Type::Unknown | Type::Any) {
                errors.push(TypeError::error(
                    format!("range start must be Int, got '{}'", st), start.span.clone(),
                ));
            }
            if !matches!(et, Type::Int | Type::Unknown | Type::Any) {
                errors.push(TypeError::error(
                    format!("range end must be Int, got '{}'", et), end.span.clone(),
                ));
            }
            Type::Array(Box::new(Type::Int))
        }

        // ── Chiamata ──────────────────────────────────────────────────────
        ExprKind::Call { callee, args, .. } => {
            let callee_ty = infer_expr(callee, env, errors);
            for arg in args { infer_expr(arg, env, errors); }
            match &callee_ty {
                Type::Fn { params, ret } => {
                    // Verifica arietà (tollerante: Any params = variadic)
                    let is_variadic = params.len() == 1 && matches!(params[0], Type::Any);
                    if !is_variadic && args.len() != params.len() {
                        let name = match &callee.inner {
                            ExprKind::Ident(n) => n.as_str(),
                            _ => "<fn>",
                        };
                        errors.push(TypeError::arity(name, params.len(), args.len(), span));
                    }
                    *ret.clone()
                }
                Type::Unknown | Type::Any => Type::Unknown,
                // Costruttore di classe
                Type::Class(name) => Type::Class(name.clone()),
                other => {
                    errors.push(TypeError::not_callable(other, span));
                    Type::Unknown
                }
            }
        }

        // ── Accesso campo ─────────────────────────────────────────────────
        ExprKind::Field { object, field } => {
            let obj_ty = infer_expr(object, env, errors);
            match &obj_ty {
                Type::Class(name) => {
                    if let Some(info) = env.lookup_class(name).cloned() {
                        if let Some(ft) = info.fields.get(field.as_str()).cloned() {
                            ft
                        } else if let Some(mt) = info.methods.get(field.as_str()).cloned() {
                            mt
                        } else {
                            errors.push(TypeError::unknown_field(name, field, span));
                            Type::Unknown
                        }
                    } else {
                        // Classe non ancora registrata — tollerante
                        Type::Unknown
                    }
                }
                // Built-in su stringhe e array
                Type::Str => match field.as_str() {
                    "len" | "upper" | "lower" | "trim" => Type::Int,
                    _ => {
                        errors.push(TypeError::unknown_field("Str", field, span));
                        Type::Unknown
                    }
                },
                Type::Array(_) => match field.as_str() {
                    "len" => Type::Int,
                    _ => {
                        errors.push(TypeError::unknown_field("Array", field, span));
                        Type::Unknown
                    }
                },
                Type::Unknown | Type::Any => Type::Unknown,
                other => {
                    errors.push(TypeError::error(
                        format!("'{}' has no fields", other), span,
                    ));
                    Type::Unknown
                }
            }
        }

        // ── Indice ────────────────────────────────────────────────────────
        ExprKind::Index { object, index } => {
            let obj_ty = infer_expr(object, env, errors);
            let idx_ty = infer_expr(index, env, errors);
            if !matches!(idx_ty, Type::Int | Type::Unknown | Type::Any) {
                errors.push(TypeError::error(
                    format!("index must be Int, got '{}'", idx_ty),
                    index.span.clone(),
                ));
            }
            match &obj_ty {
                Type::Array(inner) => *inner.clone(),
                Type::Str          => Type::Str,
                Type::Unknown | Type::Any => Type::Unknown,
                other => {
                    errors.push(TypeError::error(
                        format!("'{}' is not indexable", other), span,
                    ));
                    Type::Unknown
                }
            }
        }

        // ── Option / Result ───────────────────────────────────────────────
        ExprKind::Some(inner) => {
            let t = infer_expr(inner, env, errors);
            Type::Option(Box::new(t))
        }
        ExprKind::Ok(inner) => {
            let t = infer_expr(inner, env, errors);
            Type::Result(Box::new(t), Box::new(Type::Unknown))
        }
        ExprKind::Err(inner) => {
            let t = infer_expr(inner, env, errors);
            Type::Result(Box::new(Type::Unknown), Box::new(t))
        }

        // ── Spawn / Await ─────────────────────────────────────────────────
        ExprKind::Spawn(e) | ExprKind::Await(e) => {
            infer_expr(e, env, errors);
            Type::Any
        }

        // ── If come espressione ───────────────────────────────────────────
        ExprKind::If { condition, then_block, elif_branches, else_block } => {
            let ct = infer_expr(condition, env, errors);
            if !matches!(ct, Type::Bool | Type::Unknown | Type::Any) {
                errors.push(TypeError::type_mismatch(&Type::Bool, &ct, condition.span.clone()));
            }
            env.push_scope();
            let then_ty = infer_block(then_block, env, errors);
            env.pop_scope();

            for (cond, block) in elif_branches {
                infer_expr(cond, env, errors);
                env.push_scope();
                infer_block(block, env, errors);
                env.pop_scope();
            }

            if let Some(else_b) = else_block {
                env.push_scope();
                let else_ty = infer_block(else_b, env, errors);
                env.pop_scope();
                Type::unify(&then_ty, &else_ty).unwrap_or(Type::Unknown)
            } else {
                then_ty
            }
        }

        // ── Match ─────────────────────────────────────────────────────────
        ExprKind::Match { subject, arms } => {
            infer_expr(subject, env, errors);
            let mut result_ty = Type::Unknown;
            for arm in arms {
                env.push_scope();
                // Variabili legate dai pattern
                bind_pattern_vars(&arm.pattern, env);
                let arm_ty = infer_block(&arm.body, env, errors);
                env.pop_scope();
                result_ty = Type::unify(&result_ty, &arm_ty).unwrap_or(Type::Unknown);
            }
            result_ty
        }
    }
}

/// Inferisce il tipo "restituito" da un blocco di statement
/// (ultimo statement-espressione, o None).
pub fn infer_block(stmts: &[Stmt], env: &mut TypeEnv, errors: &mut Vec<TypeError>) -> Type {
    let mut last = Type::None;
    for stmt in stmts {
        if let StmtKind::Expr(e) = &stmt.inner {
            last = infer_expr(e, env, errors);
        } else {
            crate::check::check_stmt(stmt, env, errors);
            last = Type::None;
        }
    }
    last
}

/// Aggiunge nell'ambiente le variabili legate da un pattern.
fn bind_pattern_vars(pat: &Pattern, env: &mut TypeEnv) {
    match pat {
        Pattern::Ident(name) => {
            env.define(name, Type::Unknown, true);
        }
        Pattern::Constructor(_, inner) => {
            for p in inner { bind_pattern_vars(p, env); }
        }
        Pattern::Or(pats) => {
            if let Some(p) = pats.first() { bind_pattern_vars(p, env); }
        }
        Pattern::Range { start, end, .. } => {
            bind_pattern_vars(start, env);
            bind_pattern_vars(end, env);
        }
        _ => {}
    }
}

/// Tipo risultante di un'operazione binaria.
fn infer_binary(op: &BinOp, lt: &Type, rt: &Type, span: Span, errors: &mut Vec<TypeError>) -> Type {
    let op_str = op_name(op);
    match op {
        // Aritmetica
        BinOp::Add => {
            // Str + Str = Str (concatenazione)
            if matches!((lt, rt), (Type::Str, Type::Str)) { return Type::Str; }
            // Numerico + Numerico
            if lt.is_numeric() && rt.is_numeric() {
                return Type::unify(lt, rt).unwrap_or(Type::Float);
            }
            if matches!(lt, Type::Unknown | Type::Any) || matches!(rt, Type::Unknown | Type::Any) {
                return Type::Unknown;
            }
            errors.push(TypeError::binary_op(op_str, lt, rt, span));
            Type::Unknown
        }
        BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Pow => {
            if lt.is_numeric() && rt.is_numeric() {
                return Type::unify(lt, rt).unwrap_or(Type::Float);
            }
            if matches!(lt, Type::Unknown | Type::Any) || matches!(rt, Type::Unknown | Type::Any) {
                return Type::Unknown;
            }
            errors.push(TypeError::binary_op(op_str, lt, rt, span));
            Type::Unknown
        }
        BinOp::IntDiv | BinOp::Mod => {
            if matches!(lt, Type::Unknown | Type::Any) || matches!(rt, Type::Unknown | Type::Any) {
                return Type::Int;
            }
            if !matches!(lt, Type::Int | Type::Float) || !matches!(rt, Type::Int | Type::Float) {
                errors.push(TypeError::binary_op(op_str, lt, rt, span));
            }
            Type::Int
        }

        // Confronto
        BinOp::Eq | BinOp::Ne => Type::Bool,
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            if !lt.is_ordered() || !rt.is_ordered() {
                errors.push(TypeError::binary_op(op_str, lt, rt, span));
            }
            Type::Bool
        }

        // Logici
        BinOp::And | BinOp::Or => Type::Bool,

        // Bitwise
        BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => {
            if !matches!(lt, Type::Int | Type::Unknown | Type::Any) ||
               !matches!(rt, Type::Int | Type::Unknown | Type::Any) {
                errors.push(TypeError::binary_op(op_str, lt, rt, span));
            }
            Type::Int
        }

        // Membership
        BinOp::In | BinOp::NotIn => Type::Bool,
        BinOp::Is => Type::Bool,
    }
}

fn op_name(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add    => "+",
        BinOp::Sub    => "-",
        BinOp::Mul    => "*",
        BinOp::Div    => "/",
        BinOp::IntDiv => "//",
        BinOp::Mod    => "%",
        BinOp::Pow    => "**",
        BinOp::Eq     => "==",
        BinOp::Ne     => "!=",
        BinOp::Lt     => "<",
        BinOp::Le     => "<=",
        BinOp::Gt     => ">",
        BinOp::Ge     => ">=",
        BinOp::And    => "and",
        BinOp::Or     => "or",
        BinOp::BitAnd => "&",
        BinOp::BitOr  => "|",
        BinOp::BitXor => "^",
        BinOp::Shl    => "<<",
        BinOp::Shr    => ">>",
        BinOp::In     => "in",
        BinOp::NotIn  => "not in",
        BinOp::Is     => "is",
    }
}
