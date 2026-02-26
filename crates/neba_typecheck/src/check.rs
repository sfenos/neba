use std::collections::HashMap;
use neba_parser::ast::*;
use crate::types::Type;
use crate::error::TypeError;
use crate::env::{TypeEnv, ClassInfo};
use crate::infer::infer_expr;

/// Verifica un singolo statement.
pub fn check_stmt(stmt: &Stmt, env: &mut TypeEnv, errors: &mut Vec<TypeError>) {
    let span = stmt.span.clone();
    match &stmt.inner {

        // ── Dichiarazioni variabile ────────────────────────────────────────
        StmtKind::Let { name, ty, value } => {
            let val_ty = infer_expr(value, env, errors);
            let declared_ty = ty.as_ref().map(|t| Type::from_ast(&t.inner));
            let final_ty = if let Some(dt) = &declared_ty {
                if !dt.is_compatible(&val_ty) {
                    errors.push(TypeError::type_mismatch(dt, &val_ty, value.span.clone()));
                }
                dt.clone()
            } else {
                val_ty
            };
            env.define(name, final_ty, false);
        }

        StmtKind::Var { name, ty, value } => {
            let val_ty = infer_expr(value, env, errors);
            let declared_ty = ty.as_ref().map(|t| Type::from_ast(&t.inner));
            let final_ty = if let Some(dt) = &declared_ty {
                if !dt.is_compatible(&val_ty) {
                    errors.push(TypeError::type_mismatch(dt, &val_ty, value.span.clone()));
                }
                dt.clone()
            } else {
                val_ty
            };
            env.define(name, final_ty, true);
        }

        // ── Assegnazione ──────────────────────────────────────────────────
        StmtKind::Assign { target, op: _, value } => {
            let val_ty = infer_expr(value, env, errors);
            if let ExprKind::Ident(name) = &target.inner {
                match env.lookup(name) {
                    Some(info) if !info.mutable => {
                        errors.push(TypeError::assign_immutable(name, span));
                    }
                    Some(info) => {
                        let existing = info.ty.clone();
                        if !existing.is_compatible(&val_ty) {
                            errors.push(TypeError::type_mismatch(&existing, &val_ty, value.span.clone()));
                        }
                    }
                    None => {
                        errors.push(TypeError::undefined(name, target.span.clone()));
                    }
                }
            } else {
                infer_expr(target, env, errors);
            }
        }

        // ── Funzione ──────────────────────────────────────────────────────
        StmtKind::Fn { name, params, return_ty, body, .. } => {
            // Costruisci il tipo della funzione
            let param_types: Vec<Type> = params.iter()
                .map(|p| p.ty.as_ref().map(|t| Type::from_ast(&t.inner)).unwrap_or(Type::Unknown))
                .collect();
            let ret_ty = return_ty.as_ref()
                .map(|t| Type::from_ast(&t.inner))
                .unwrap_or(Type::Unknown);

            let fn_ty = Type::Fn {
                params: param_types.clone(),
                ret:    Box::new(ret_ty.clone()),
            };
            env.define(name, fn_ty, false);

            // Analizza il corpo in uno scope nuovo
            env.push_scope();
            env.push_return(ret_ty.clone());
            for (param, pty) in params.iter().zip(param_types.iter()) {
                env.define(&param.name, pty.clone(), true);
            }
            check_block(body, env, errors);
            env.pop_return();
            env.pop_scope();
        }

        // ── Classe ────────────────────────────────────────────────────────
        StmtKind::Class { name, fields, methods, impls } => {
            let mut field_map = HashMap::new();
            for f in fields {
                let fty = f.ty.as_ref()
                    .map(|t| Type::from_ast(&t.inner))
                    .unwrap_or(Type::Unknown);
                field_map.insert(f.name.clone(), fty);
            }

            let mut method_map = HashMap::new();
            for m in methods.iter().chain(impls.iter()) {
                if let StmtKind::Fn { name: mname, params, return_ty, .. } = &m.inner {
                    let param_types: Vec<Type> = params.iter()
                        .filter(|p| p.name != "self")
                        .map(|p| p.ty.as_ref().map(|t| Type::from_ast(&t.inner)).unwrap_or(Type::Unknown))
                        .collect();
                    let ret = return_ty.as_ref()
                        .map(|t| Type::from_ast(&t.inner))
                        .unwrap_or(Type::None);
                    method_map.insert(mname.clone(), Type::Fn {
                        params: param_types,
                        ret: Box::new(ret),
                    });
                }
            }

            env.register_class(name, ClassInfo { fields: field_map, methods: method_map });

            // Il tipo della classe stessa è un costruttore
            env.define(name, Type::Class(name.clone()), false);

            // Analizza i corpi dei metodi
            for m in methods.iter().chain(impls.iter()) {
                if let StmtKind::Fn { params, return_ty, body, .. } = &m.inner {
                    env.push_scope();
                    env.define("self", Type::Class(name.clone()), false);
                    let ret_ty = return_ty.as_ref()
                        .map(|t| Type::from_ast(&t.inner))
                        .unwrap_or(Type::None);
                    env.push_return(ret_ty);
                    for p in params.iter().filter(|p| p.name != "self") {
                        let pty = p.ty.as_ref()
                            .map(|t| Type::from_ast(&t.inner))
                            .unwrap_or(Type::Unknown);
                        env.define(&p.name, pty, true);
                    }
                    check_block(body, env, errors);
                    env.pop_return();
                    env.pop_scope();
                }
            }
        }

        // ── While ─────────────────────────────────────────────────────────
        StmtKind::While { condition, body } => {
            let ct = infer_expr(condition, env, errors);
            if !matches!(ct, Type::Bool | Type::Unknown | Type::Any) {
                errors.push(TypeError::type_mismatch(&Type::Bool, &ct, condition.span.clone()));
            }
            env.push_scope();
            check_block(body, env, errors);
            env.pop_scope();
        }

        // ── For ───────────────────────────────────────────────────────────
        StmtKind::For { var, iterable, body } => {
            let iter_ty = infer_expr(iterable, env, errors);
            let elem_ty = match iter_ty.iter_element() {
                Some(t) => t,
                None => {
                    errors.push(TypeError::not_iterable(&iter_ty, iterable.span.clone()));
                    Type::Unknown
                }
            };
            env.push_scope();
            env.define(var, elem_ty, true);
            check_block(body, env, errors);
            env.pop_scope();
        }

        // ── Return ────────────────────────────────────────────────────────
        StmtKind::Return(expr) => {
            let ret_ty = match expr {
                Some(e) => infer_expr(e, env, errors),
                None    => Type::None,
            };
            if let Some(expected) = env.expected_return() {
                let expected = expected.clone();
                if !matches!(expected, Type::Unknown | Type::Any)
                    && !expected.is_compatible(&ret_ty)
                {
                    let span = expr.as_ref().map(|e| e.span.clone()).unwrap_or(span);
                    errors.push(TypeError::return_mismatch(&expected, &ret_ty, span));
                }
            }
        }

        // ── Espressione come statement ─────────────────────────────────────
        StmtKind::Expr(e) => { infer_expr(e, env, errors); }

        // ── Trait / Impl / Mod / Use / Break / Continue / Pass ───────────
        StmtKind::Trait { .. } | StmtKind::Impl { .. }
        | StmtKind::Mod(_) | StmtKind::Use(_)
        | StmtKind::Break | StmtKind::Continue | StmtKind::Pass => {}
    }
}

/// Verifica un blocco di statement.
pub fn check_block(stmts: &[Stmt], env: &mut TypeEnv, errors: &mut Vec<TypeError>) {
    for stmt in stmts {
        check_stmt(stmt, env, errors);
    }
}

/// Verifica un intero programma.
pub fn check_program(program: &neba_parser::ast::Program, env: &mut TypeEnv, errors: &mut Vec<TypeError>) {
    // Prima passata: registra tutte le funzioni e classi top-level
    // (per gestire forward references)
    for stmt in &program.stmts {
        pre_register(stmt, env);
    }
    // Seconda passata: verifica tutto
    check_block(&program.stmts, env, errors);
}

/// Prima passata: registra fn e class globali senza verificare i corpi.
fn pre_register(stmt: &Stmt, env: &mut TypeEnv) {
    match &stmt.inner {
        StmtKind::Fn { name, params, return_ty, .. } => {
            let param_types: Vec<Type> = params.iter()
                .map(|p| p.ty.as_ref().map(|t| Type::from_ast(&t.inner)).unwrap_or(Type::Unknown))
                .collect();
            let ret = return_ty.as_ref()
                .map(|t| Type::from_ast(&t.inner))
                .unwrap_or(Type::Unknown);
            env.define(name, Type::Fn { params: param_types, ret: Box::new(ret) }, false);
        }
        StmtKind::Class { name, .. } => {
            env.define(name, Type::Class(name.clone()), false);
        }
        _ => {}
    }
}
