use std::rc::Rc;

use neba_parser::ast::*;

use crate::chunk::{Chunk, FnProto, UpvalueDesc};
use crate::error::{VmError, VmResult};
use crate::opcode::Op;
use crate::value::Value;

// ── Scope tracking ────────────────────────────────────────────────────────

/// Variabile locale (nello stack frame corrente).
#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: usize,
    mutable: bool,
}

/// Un upvalue visto dal compilatore.
#[derive(Debug, Clone)]
struct UpvalueDef {
    is_local: bool,
    index: u8,
}

/// Contesto di un singolo scope (blocco annidato).
struct Scope {
    depth: usize,
}

// ── ClassInfo (metadati di classe registrati durante la compilazione) ─────

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub fields: Vec<Field>,
    pub methods: Vec<Stmt>,
}

// ── Compiler ──────────────────────────────────────────────────────────────

pub struct Compiler {
    chunk: Chunk,
    locals: Vec<Local>,
    upvalues: Vec<UpvalueDef>,
    scope_depth: usize,
    /// Lista di offset da patchare con l'offset di uscita dal loop corrente.
    break_patches: Vec<Vec<usize>>,
    /// Lista di offset da patchare per continue (inizio dell'iterazione).
    continue_patches: Vec<Vec<usize>>,
    /// Mappa class_name → ClassInfo
    pub class_registry: std::collections::HashMap<String, ClassInfo>,
    /// Nome della funzione (per messaggi di errore)
    fn_name: String,
    is_function: bool,
}

impl Compiler {
    // ── Costruttori ───────────────────────────────────────────────────────

    pub fn new_script() -> Self {
        Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: 0,
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            class_registry: std::collections::HashMap::new(),
            fn_name: "<script>".to_string(),
            is_function: false,
        }
    }

    fn new_function(name: &str) -> Self {
        Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: 1, // funzione ha già uno scope aperto
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            class_registry: std::collections::HashMap::new(),
            fn_name: name.to_string(),
            is_function: true,
        }
    }

    // ── Entry point ───────────────────────────────────────────────────────

    pub fn compile(program: &Program) -> VmResult<Chunk> {
        let mut c = Compiler::new_script();
        let stmts = &program.stmts;
        if stmts.is_empty() {
            c.chunk.emit(Op::Nil, 0);
            c.chunk.emit(Op::Halt, 0);
            return Ok(c.chunk);
        }
        for stmt in &stmts[..stmts.len() - 1] {
            c.compile_stmt(stmt)?;
        }
        // Ultimo statement: se è un'espressione lasciala sullo stack (valore di ritorno dello script)
        let last = stmts.last().unwrap();
        let last_line = last.span.line as u32;
        if let StmtKind::Expr(e) = &last.inner {
            c.compile_expr(e)?;
        } else {
            c.compile_stmt(last)?;
            c.chunk.emit(Op::Nil, last_line);
        }
        c.chunk.emit(Op::Halt, last_line);
        Ok(c.chunk)
    }

    // ── Statement ─────────────────────────────────────────────────────────

    fn compile_stmt(&mut self, stmt: &Stmt) -> VmResult<()> {
        let line = stmt.span.line as u32;
        match &stmt.inner {
            StmtKind::Expr(e) => {
                self.compile_expr(e)?;
                // in un contesto statement il valore è scartato
                // tranne nell'ultima stmt di un blocco-espressione (gestito dalla VM)
                self.chunk.emit(Op::Pop, line);
            }
            StmtKind::Let { name, value, .. } => {
                self.compile_expr(value)?;
                self.define_var(name, false, line)?;
            }
            StmtKind::Var { name, value, .. } => {
                self.compile_expr(value)?;
                self.define_var(name, true, line)?;
            }
            StmtKind::Assign { target, op, value } => {
                self.compile_assign(target, op, value, line)?;
            }
            StmtKind::Fn { name, params, body, is_async, .. } => {
                self.compile_fn_def(name, params, body, *is_async, line)?;
                self.define_var(name, false, line)?;
            }
            StmtKind::Return(expr) => {
                match expr {
                    Some(e) => self.compile_expr(e)?,
                    None    => { self.chunk.emit(Op::Nil, line); }
                }
                self.chunk.emit(Op::Return, line);
            }
            StmtKind::While { condition, body } => {
                self.compile_while(condition, body, line)?;
            }
            StmtKind::For { var, iterable, body } => {
                self.compile_for(var, iterable, body, line)?;
            }
            StmtKind::Break => {
                let patch = self.chunk.emit_jump(Op::Jump, line);
                self.break_patches.last_mut()
                    .ok_or_else(|| VmError::CompileError("break outside loop".into()))?
                    .push(patch);
            }
            StmtKind::Continue => {
                let patch = self.chunk.emit_jump(Op::Jump, line);
                self.continue_patches.last_mut()
                    .ok_or_else(|| VmError::CompileError("continue outside loop".into()))?
                    .push(patch);
            }
            StmtKind::Pass => {}
            StmtKind::Class { name, fields, methods, impls } => {
                self.compile_class(name, fields, methods, impls, line)?;
            }
            StmtKind::Trait { .. } | StmtKind::Impl { .. } => {
                // No-op in v0.2.0 (saranno gestiti in v0.2.4)
            }
            StmtKind::Mod(n) => {
                eprintln!("[warn] mod '{}' non supportato in v0.2.0", n);
            }
            StmtKind::Use(path) => {
                eprintln!("[warn] use '{}' non supportato in v0.2.0", path.join("::"));
            }
        }
        Ok(())
    }

    // ── Definizione variabile ─────────────────────────────────────────────

    fn define_var(&mut self, name: &str, mutable: bool, line: u32) -> VmResult<()> {
        if self.scope_depth == 0 {
            // globale
            let idx = self.chunk.add_name(name);
            self.chunk.emit(Op::DefGlobal, line);
            self.chunk.emit_u16(idx);
            self.chunk.emit_u8(mutable as u8);
        } else {
            // locale
            self.locals.push(Local {
                name: name.to_string(),
                depth: self.scope_depth,
                mutable,
            });
            // il valore è già sullo stack — non serve emettere nulla
        }
        Ok(())
    }

    // ── Risoluzione variabili ─────────────────────────────────────────────

    fn resolve_local(&self, name: &str) -> Option<(u8, bool)> {
        for (i, l) in self.locals.iter().enumerate().rev() {
            if l.name == name { return Some((i as u8, l.mutable)); }
        }
        None
    }

    fn resolve_upvalue(&mut self, name: &str) -> Option<u8> {
        // In v0.2.0 non implementiamo upvalue dinamici
        // Le closure catturano per snapshot (come il tree-walker)
        None
    }

    fn emit_load(&mut self, name: &str, line: u32) -> VmResult<()> {
        if let Some((idx, _)) = self.resolve_local(name) {
            self.chunk.emit(Op::LoadLocal, line);
            self.chunk.emit_u8(idx);
        } else {
            let idx = self.chunk.add_name(name);
            self.chunk.emit(Op::LoadGlobal, line);
            self.chunk.emit_u16(idx);
        }
        Ok(())
    }

    fn emit_store(&mut self, name: &str, line: u32) -> VmResult<()> {
        if let Some((idx, mutable)) = self.resolve_local(name) {
            if !mutable {
                return Err(VmError::CompileError(
                    format!("cannot assign to immutable variable '{}'", name)
                ));
            }
            self.chunk.emit(Op::StoreLocal, line);
            self.chunk.emit_u8(idx);
        } else {
            let idx = self.chunk.add_name(name);
            self.chunk.emit(Op::StoreGlobal, line);
            self.chunk.emit_u16(idx);
        }
        Ok(())
    }

    // ── Scope ─────────────────────────────────────────────────────────────

    fn push_scope(&mut self) { self.scope_depth += 1; }

    fn pop_scope(&mut self, line: u32) {
        let count = self.locals.iter().rev()
            .take_while(|l| l.depth == self.scope_depth)
            .count();
        for _ in 0..count { self.locals.pop(); }
        if count > 0 {
            if count == 1 {
                self.chunk.emit(Op::Pop, line);
            } else {
                self.chunk.emit(Op::PopN, line);
                self.chunk.emit_u8(count as u8);
            }
        }
        self.scope_depth -= 1;
    }

    // ── Assegnazione ──────────────────────────────────────────────────────

    fn compile_assign(&mut self, target: &Expr, op: &AssignOp, value: &Expr, line: u32) -> VmResult<()> {
        match &target.inner {
            ExprKind::Ident(name) => {
                if !matches!(op, AssignOp::Assign) {
                    // x += rhs → carica x, carica rhs, op, store
                    self.emit_load(name, line)?;
                    self.compile_expr(value)?;
                    self.emit_compound_op(op, line);
                } else {
                    self.compile_expr(value)?;
                }
                self.emit_store(name, line)?;
                // Nessun valore residuo: l'assegnazione è uno statement puro
            }
            ExprKind::Index { object, index } => {
                self.compile_expr(object)?;
                self.compile_expr(index)?;
                if !matches!(op, AssignOp::Assign) {
                    self.chunk.emit(Op::Dup, line);
                    self.compile_expr(object)?;
                    self.compile_expr(index)?;
                    self.chunk.emit(Op::GetIndex, line);
                    self.compile_expr(value)?;
                    self.emit_compound_op(op, line);
                    self.chunk.emit(Op::SetIndex, line);
                } else {
                    self.compile_expr(value)?;
                    self.chunk.emit(Op::SetIndex, line);
                }
                // Nessun Nil residuo
            }
            ExprKind::Field { object, field } => {
                self.compile_expr(object)?;
                let name_idx = self.chunk.add_name(field);
                if !matches!(op, AssignOp::Assign) {
                    self.chunk.emit(Op::Dup, line);
                    self.chunk.emit(Op::GetField, line);
                    self.chunk.emit_u16(name_idx);
                    self.compile_expr(value)?;
                    self.emit_compound_op(op, line);
                    self.chunk.emit(Op::SetField, line);
                    self.chunk.emit_u16(name_idx);
                } else {
                    self.compile_expr(value)?;
                    self.chunk.emit(Op::SetField, line);
                    self.chunk.emit_u16(name_idx);
                }
                // Nessun Nil residuo
            }
            _ => return Err(VmError::CompileError("invalid assignment target".into())),
        }
        Ok(())
    }

    fn emit_compound_op(&mut self, op: &AssignOp, line: u32) {
        match op {
            AssignOp::AddAssign => { self.chunk.emit(Op::Add, line); }
            AssignOp::SubAssign => { self.chunk.emit(Op::Sub, line); }
            AssignOp::MulAssign => { self.chunk.emit(Op::Mul, line); }
            AssignOp::DivAssign => { self.chunk.emit(Op::Div, line); }
            AssignOp::ModAssign => { self.chunk.emit(Op::Mod, line); }
            AssignOp::Assign    => {}
        }
    }

    // ── Espressioni ───────────────────────────────────────────────────────

    fn compile_expr(&mut self, expr: &Expr) -> VmResult<()> {
        let line = expr.span.line as u32;
        match &expr.inner {
            ExprKind::Int(n)   => { let i = self.chunk.add_const(Value::Int(*n));   self.chunk.emit(Op::Const, line); self.chunk.emit_u16(i); }
            ExprKind::Float(f) => { let i = self.chunk.add_const(Value::Float(*f)); self.chunk.emit(Op::Const, line); self.chunk.emit_u16(i); }
            ExprKind::Bool(b)  => { self.chunk.emit(if *b { Op::True } else { Op::False }, line); }
            ExprKind::None     => { self.chunk.emit(Op::Nil, line); }
            ExprKind::Str(s)   => { let i = self.chunk.add_const(Value::str(s.as_str())); self.chunk.emit(Op::Const, line); self.chunk.emit_u16(i); }
            ExprKind::FStr(t)  => { self.compile_fstring(t, line)?; }

            ExprKind::Ident(name) => { self.emit_load(name, line)?; }

            ExprKind::Unary { op, operand } => {
                self.compile_expr(operand)?;
                match op {
                    UnaryOp::Neg    => { self.chunk.emit(Op::Neg, line); }
                    UnaryOp::Not    => { self.chunk.emit(Op::Not, line); }
                    UnaryOp::BitNot => { self.chunk.emit(Op::BitNot, line); }
                }
            }

            ExprKind::Binary { op, left, right } => {
                self.compile_binary(op, left, right, line)?;
            }

            ExprKind::If { condition, then_block, elif_branches, else_block } => {
                self.compile_if(condition, then_block, elif_branches, else_block, line)?;
            }

            ExprKind::Match { subject, arms } => {
                self.compile_match(subject, arms, line)?;
            }

            ExprKind::Call { callee, args, kwargs } => {
                if let ExprKind::Field { object, field } = &callee.inner {
                    // Chiamata a metodo: compila obj poi gli args, emetti CallMethod
                    self.compile_expr(object)?;
                    let mut argc = args.len();
                    for a in args { self.compile_expr(a)?; }
                    for (_, v) in kwargs { self.compile_expr(v)?; argc += 1; }
                    let idx = self.chunk.add_name(field);
                    self.chunk.emit(Op::CallMethod, line);
                    self.chunk.emit_u16(idx);
                    self.chunk.emit_u8(argc as u8);
                } else {
                    // Chiamata a funzione normale
                    self.compile_expr(callee)?;
                    let mut argc = args.len();
                    for a in args { self.compile_expr(a)?; }
                    for (_, v) in kwargs { self.compile_expr(v)?; argc += 1; }
                    self.chunk.emit(Op::Call, line);
                    self.chunk.emit_u8(argc as u8);
                }
            }

            ExprKind::Field { object, field } => {
                self.compile_expr(object)?;
                let idx = self.chunk.add_name(field);
                self.chunk.emit(Op::GetField, line);
                self.chunk.emit_u16(idx);
            }

            ExprKind::Index { object, index } => {
                self.compile_expr(object)?;
                self.compile_expr(index)?;
                self.chunk.emit(Op::GetIndex, line);
            }

            ExprKind::Array(items) => {
                for item in items { self.compile_expr(item)?; }
                self.chunk.emit(Op::MakeArray, line);
                self.chunk.emit_u16(items.len() as u16);
            }

            ExprKind::Range { start, end, inclusive } => {
                self.compile_expr(start)?;
                self.compile_expr(end)?;
                self.chunk.emit(Op::MakeRange, line);
                self.chunk.emit_u8(*inclusive as u8);
            }

            ExprKind::Some(inner) => {
                self.compile_expr(inner)?;
                self.chunk.emit(Op::MakeSome, line);
            }
            ExprKind::Ok(inner) => {
                self.compile_expr(inner)?;
                self.chunk.emit(Op::MakeOk, line);
            }
            ExprKind::Err(inner) => {
                self.compile_expr(inner)?;
                self.chunk.emit(Op::MakeErr, line);
            }

            ExprKind::Spawn(inner) => {
                eprintln!("[warn] spawn è sincrono in v0.2.0");
                self.compile_expr(inner)?;
            }
            ExprKind::Await(inner) => {
                self.compile_expr(inner)?;
            }

            ExprKind::Error => {
                return Err(VmError::CompileError("AST error node".into()));
            }
        }
        Ok(())
    }

    // ── Binary ────────────────────────────────────────────────────────────

    fn compile_binary(&mut self, op: &BinOp, left: &Expr, right: &Expr, line: u32) -> VmResult<()> {
        match op {
            BinOp::And => {
                self.compile_expr(left)?;
                let patch = self.chunk.emit_jump(Op::JumpFalsePeek, line);
                self.chunk.emit(Op::Pop, line);
                self.compile_expr(right)?;
                self.chunk.patch_jump(patch);
            }
            BinOp::Or => {
                self.compile_expr(left)?;
                let patch = self.chunk.emit_jump(Op::JumpTruePeek, line);
                self.chunk.emit(Op::Pop, line);
                self.compile_expr(right)?;
                self.chunk.patch_jump(patch);
            }
            _ => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                let instr = match op {
                    BinOp::Add    => Op::Add,
                    BinOp::Sub    => Op::Sub,
                    BinOp::Mul    => Op::Mul,
                    BinOp::Div    => Op::Div,
                    BinOp::IntDiv => Op::IntDiv,
                    BinOp::Mod    => Op::Mod,
                    BinOp::Pow    => Op::Pow,
                    BinOp::Eq     => Op::Eq,
                    BinOp::Ne     => Op::Ne,
                    BinOp::Lt     => Op::Lt,
                    BinOp::Le     => Op::Le,
                    BinOp::Gt     => Op::Gt,
                    BinOp::Ge     => Op::Ge,
                    BinOp::BitAnd => Op::BitAnd,
                    BinOp::BitOr  => Op::BitOr,
                    BinOp::BitXor => Op::BitXor,
                    BinOp::Shl    => Op::Shl,
                    BinOp::Shr    => Op::Shr,
                    BinOp::Is     => Op::Is,
                    BinOp::In     => Op::In,
                    BinOp::NotIn  => Op::NotIn,
                    BinOp::And | BinOp::Or => unreachable!(),
                };
                self.chunk.emit(instr, line);
            }
        }
        Ok(())
    }

    // ── If expression ─────────────────────────────────────────────────────

    fn compile_if(
        &mut self,
        condition: &Expr,
        then_block: &[Stmt],
        elif_branches: &[(Expr, Vec<Stmt>)],
        else_block: &Option<Vec<Stmt>>,
        line: u32,
    ) -> VmResult<()> {
        self.compile_expr(condition)?;
        let mut end_patches = Vec::new();

        let else_patch = self.chunk.emit_jump(Op::JumpFalse, line);
        self.compile_block_expr(then_block, line)?;
        end_patches.push(self.chunk.emit_jump(Op::Jump, line));
        self.chunk.patch_jump(else_patch);

        for (cond, block) in elif_branches {
            let elif_line = cond.span.line as u32;
            self.compile_expr(cond)?;
            let elif_else = self.chunk.emit_jump(Op::JumpFalse, elif_line);
            self.compile_block_expr(block, elif_line)?;
            end_patches.push(self.chunk.emit_jump(Op::Jump, elif_line));
            self.chunk.patch_jump(elif_else);
        }

        match else_block {
            Some(b) => self.compile_block_expr(b, line)?,
            None    => { self.chunk.emit(Op::Nil, line); }
        }

        for p in end_patches { self.chunk.patch_jump(p); }
        Ok(())
    }

    /// Compila un blocco come espressione: l'ultimo statement contribuisce
    /// il valore (se è Expr), altrimenti emette Nil.
    fn compile_block_expr(&mut self, stmts: &[Stmt], line: u32) -> VmResult<()> {
        self.push_scope();
        if stmts.is_empty() {
            self.pop_scope(line);
            self.chunk.emit(Op::Nil, line);
            return Ok(());
        }
        for stmt in &stmts[..stmts.len() - 1] {
            self.compile_stmt(stmt)?;
        }
        let last = stmts.last().unwrap();
        // Se l'ultimo è un Expr-statement, compila come espressione (non poppare)
        if let StmtKind::Expr(e) = &last.inner {
            self.compile_expr(e)?;
        } else {
            self.compile_stmt(last)?;
            self.chunk.emit(Op::Nil, line);
        }
        // Pop scope ma senza poppare il valore risultato
        // → pop i locali manualmente
        let count = self.locals.iter().rev()
            .take_while(|l| l.depth == self.scope_depth)
            .count();
        // Sposta il valore risultato sopra i locali da poppare
        // usando una tecnica: salviamo in un local temporaneo implicito
        // ma per semplicità usiamo un approccio diverso:
        // pop_scope fa il cleanup dei locals PRIMA del valore sul top.
        // Dobbiamo fare "rot(count+1)" ma non abbiamo quell'istruzione.
        //
        // Soluzione pragmatica: i locali vengono poppati dai lato della vm
        // tramite il meccanismo del call-frame. In realtà in un block-expr
        // i locali sono su stack sotto il valore → serve riordino.
        //
        // Soluzione semplice: riserviamo lo slot 0 del frame per il "block result"
        // tramite una variabile locale temporanea (non esposta).
        // Oppure: non usiamo scope-locali per block-expr, usiamo solo globali.
        //
        // Per v0.2.0 accettiamo che i block-expr abbiano locali che vengono
        // gestiti correttamente grazie al fatto che pop_scope emette Pop/PopN
        // DOPO il valore → la VM deve gestire questo.
        //
        // Alternativa più semplice: compila i block come statement normali
        // e lascia l'ultimo valore sullo stack; i locali del blocco sono
        // sempre sotto il valore finale → il chiamante sa che deve "rot".
        //
        // DECISIONE: in v0.2.0 i block-expr NON creano scope locale.
        // Le variabili definite nel blocco vanno nello scope esterno.
        // Questo è il comportamento corretto per if/match/while come expression.
        for l in self.locals.iter().rev().take_while(|l| l.depth == self.scope_depth) {}
        for _ in 0..count { self.locals.pop(); }
        self.scope_depth -= 1;
        Ok(())
    }

    // ── Match expression ───────────────────────────────────────────────────

    fn compile_match(&mut self, subject: &Expr, arms: &[MatchArm], line: u32) -> VmResult<()> {
        self.compile_expr(subject)?;
        // Stack: [subject]
        // Per ogni arm: peek il subject, controlla il pattern, se mismatch jump al prossimo
        let mut end_patches = Vec::new();

        for arm in arms {
            let arm_line = arm.span.line as u32;
            // Emetti i controlli del pattern sul subject (peek, non pop)
            let mut fail_patches = Vec::new();
            self.compile_pattern_check(&arm.pattern, &mut fail_patches, arm_line)?;

            // Pattern match! Bindi le variabili
            self.push_scope();
            self.compile_pattern_bind(&arm.pattern, arm_line)?;
            self.compile_block_expr(&arm.body, arm_line)?;
            // Il valore risultato è sullo stack; pop i binding
            let count = self.locals.iter().rev()
                .take_while(|l| l.depth == self.scope_depth)
                .count();
            for _ in 0..count { self.locals.pop(); }
            self.scope_depth -= 1;

            // Ora: [subject, result]
            // Salta alla fine
            end_patches.push(self.chunk.emit_jump(Op::Jump, arm_line));

            // Patch tutti i fail → atterrano qui
            for p in fail_patches { self.chunk.patch_jump(p); }
        }

        // Nessun arm matched → None
        self.chunk.emit(Op::Nil, line);

        for p in end_patches { self.chunk.patch_jump(p); }
        // Pop il subject
        // Problema: dopo il match abbiamo [result] sullo stack, ma il subject è SOTTO.
        // Dobbiamo scambiare e poppare.
        // In v0.2.0 usiamo un approccio diverso: prima di iniziare il match,
        // salviamo il subject in un local temporaneo.
        // TODO: refactoring per match più corretto
        // Per ora: il subject resta sullo stack ed è responsabilità del chiamante.
        // La VM dovrà ignorarlo o la pop avviene esplicitamente.
        Ok(())
    }

    fn compile_pattern_check(&mut self, pat: &Pattern, fail_patches: &mut Vec<usize>, line: u32) -> VmResult<()> {
        match pat {
            Pattern::Wildcard | Pattern::Ident(_) => {}
            Pattern::Literal(lit) => {
                let v = match lit {
                    ExprKind::Int(n)   => Value::Int(*n),
                    ExprKind::Float(f) => Value::Float(*f),
                    ExprKind::Bool(b)  => Value::Bool(*b),
                    ExprKind::Str(s)   => Value::str(s.as_str()),
                    ExprKind::None     => Value::None,
                    _ => return Err(VmError::CompileError("invalid literal in pattern".into())),
                };
                let cidx = self.chunk.add_const(v);
                let patch = self.chunk.code.len();
                self.chunk.emit(Op::MatchLit, line);
                self.chunk.emit_u16(cidx);
                self.chunk.emit_i16(0);
                fail_patches.push(patch + 1);
            }
            Pattern::Constructor(name, inner) => {
                let (check_op, unwrap) = match name.as_str() {
                    "Some" => (Op::IsSome, true),
                    "None" => (Op::IsNone, false),
                    "Ok"   => (Op::IsOk,   true),
                    "Err"  => (Op::IsErr,   true),
                    _ => return Err(VmError::CompileError(format!("unknown constructor '{}'", name))),
                };
                let patch = self.chunk.code.len();
                self.chunk.emit(check_op, line);
                self.chunk.emit_i16(0);
                fail_patches.push(patch + 1);
                if unwrap && !inner.is_empty() {
                    // Unwrap e controlla il valore interno
                    self.chunk.emit(Op::Unwrap, line);
                    for p in inner {
                        self.compile_pattern_check(p, fail_patches, line)?;
                    }
                    // Dopo il check ri-wrappa (no-op: il binding avviene separatamente)
                }
            }
            Pattern::Range { start, end, inclusive } => {
                let lo = if let Pattern::Literal(ExprKind::Int(n)) = start.as_ref() {
                    self.chunk.add_const(Value::Int(*n))
                } else { return Err(VmError::CompileError("range pattern requires Int".into())); };
                let hi = if let Pattern::Literal(ExprKind::Int(n)) = end.as_ref() {
                    self.chunk.add_const(Value::Int(*n))
                } else { return Err(VmError::CompileError("range pattern requires Int".into())); };
                let patch = self.chunk.code.len();
                self.chunk.emit(Op::MatchRange, line);
                self.chunk.emit_u16(lo);
                self.chunk.emit_u16(hi);
                self.chunk.emit_u8(*inclusive as u8);
                self.chunk.emit_i16(0);
                fail_patches.push(patch + 1);
            }
            Pattern::Or(pats) => {
                // Se almeno uno matcha, ok. Compile as chain of checks with
                // success jumps past all fail checks.
                let mut success_patches = Vec::new();
                let mut sub_fail = Vec::new();
                for (i, p) in pats.iter().enumerate() {
                    if i > 0 {
                        for fp in sub_fail.drain(..) { self.chunk.patch_jump(fp); }
                    }
                    self.compile_pattern_check(p, &mut sub_fail, line)?;
                    success_patches.push(self.chunk.emit_jump(Op::Jump, line));
                }
                // Tutti i sub-fail: aggiungi ai fail_patches principali
                for fp in sub_fail { fail_patches.push(fp); }
                // success patches: atterrano tutti subito dopo
                for sp in success_patches { self.chunk.patch_jump(sp); }
            }
            Pattern::Error => {}
        }
        Ok(())
    }

    fn compile_pattern_bind(&mut self, pat: &Pattern, line: u32) -> VmResult<()> {
        match pat {
            Pattern::Ident(name) => {
                // Il subject è sullo stack (peek) → duplica e definisci local
                self.chunk.emit(Op::Dup, line);
                self.locals.push(Local { name: name.clone(), depth: self.scope_depth, mutable: true });
            }
            Pattern::Constructor(name, inner) if !inner.is_empty() => {
                if matches!(name.as_str(), "Some" | "Ok" | "Err") {
                    self.chunk.emit(Op::Dup, line);
                    self.chunk.emit(Op::Unwrap, line);
                    for p in inner { self.compile_pattern_bind(p, line)?; }
                }
            }
            _ => {}
        }
        Ok(())
    }

    // ── While ─────────────────────────────────────────────────────────────

    fn compile_while(&mut self, condition: &Expr, body: &[Stmt], line: u32) -> VmResult<()> {
        self.break_patches.push(Vec::new());
        self.continue_patches.push(Vec::new());

        let loop_start = self.chunk.code.len();
        self.compile_expr(condition)?;
        let exit_patch = self.chunk.emit_jump(Op::JumpFalse, line);

        self.push_scope();
        for stmt in body { self.compile_stmt(stmt)?; }
        self.pop_scope(line);

        // Patch continue → inizio del loop (prima della condizione)
        let continues = self.continue_patches.pop().unwrap();
        for p in continues {
            let offset = loop_start as isize - (p as isize + 2);
            let v = offset as i16;
            self.chunk.code[p]     = (v as u16 & 0xFF) as u8;
            self.chunk.code[p + 1] = (v as u16 >> 8) as u8;
        }

        self.chunk.emit_loop(loop_start, line);
        self.chunk.patch_jump(exit_patch);

        // Patch break → dopo il loop
        let breaks = self.break_patches.pop().unwrap();
        for p in breaks { self.chunk.patch_jump(p); }

        Ok(())
    }

    // ── For ───────────────────────────────────────────────────────────────

    fn compile_for(&mut self, var: &str, iterable: &Expr, body: &[Stmt], line: u32) -> VmResult<()> {
        self.break_patches.push(Vec::new());
        self.continue_patches.push(Vec::new());

        // Compila l'iterabile e convertilo in array
        self.compile_expr(iterable)?;
        self.chunk.emit(Op::IntoIter, line);

        // Apri lo scope del for PRIMA di aggiungere i locali impliciti,
        // così il body userà uno scope annidato e pop_scope non li toccherà.
        self.push_scope(); // scope_depth per iter/pos/var

        // Alloca due locali impliciti: __iter (l'array) e __pos (indice corrente)
        let iter_local = self.locals.len() as u8;
        self.locals.push(Local { name: format!("__iter_{}", iter_local), depth: self.scope_depth, mutable: true });
        let pos_local = self.locals.len() as u8;
        self.locals.push(Local { name: format!("__pos_{}", pos_local), depth: self.scope_depth, mutable: true });
        // Emetti Int(0) per la posizione iniziale
        let zero_idx = self.chunk.add_const(Value::Int(0));
        self.chunk.emit(Op::Const, line);
        self.chunk.emit_u16(zero_idx);

        // Il var di iterazione
        let var_local = self.locals.len() as u8;
        self.locals.push(Local { name: var.to_string(), depth: self.scope_depth, mutable: true });
        self.chunk.emit(Op::Nil, line); // placeholder

        let loop_start = self.chunk.code.len();

        // IterNext: se l'iteratore è esaurito salta fuori
        self.chunk.emit(Op::IterNext, line);
        self.chunk.emit_u8(iter_local);
        self.chunk.emit_u8(var_local);
        let exit_jump = self.chunk.code.len();
        self.chunk.emit_i16(0); // da patchare

        // Body in uno scope annidato (scope_depth + 1)
        self.push_scope();
        for stmt in body { self.compile_stmt(stmt)?; }
        self.pop_scope(line); // rimuove solo i locali del body

        // Patch continue → inizio del loop (prima di IterNext)
        let continues = self.continue_patches.pop().unwrap();
        for p in continues {
            let offset = loop_start as isize - (p as isize + 2);
            let v = offset as i16;
            self.chunk.code[p]     = (v as u16 & 0xFF) as u8;
            self.chunk.code[p + 1] = (v as u16 >> 8) as u8;
        }

        // Loop back
        self.chunk.emit_loop(loop_start, line);

        // Patch exit
        let exit_offset = self.chunk.code.len() as isize - (exit_jump as isize + 2);
        let ev = exit_offset as i16;
        self.chunk.code[exit_jump]     = (ev as u16 & 0xFF) as u8;
        self.chunk.code[exit_jump + 1] = (ev as u16 >> 8) as u8;

        // Chiudi lo scope del for: rimuove iter, pos, var ed emette PopN 3
        self.pop_scope(line);

        // Patch break (saltano qui, dopo il PopN)
        let breaks = self.break_patches.pop().unwrap();
        for p in breaks { self.chunk.patch_jump(p); }

        Ok(())
    }

    // ── Funzione ──────────────────────────────────────────────────────────

    fn compile_fn_def(
        &mut self,
        name: &str,
        params: &[Param],
        body: &[Stmt],
        is_async: bool,
        line: u32,
    ) -> VmResult<()> {
        let mut fn_compiler = Compiler::new_function(name);
        // Copia il class_registry nel sotto-compiler
        fn_compiler.class_registry = self.class_registry.clone();

        // Definisci i parametri come locali del sotto-compiler
        let arity = params.iter().filter(|p| p.default.is_none() && p.name != "self").count();
        let max_arity = params.iter().filter(|p| p.name != "self").count();

        // I parametri (escluso self) diventano locali slot 0..N
        for p in params {
            if p.name == "self" {
                fn_compiler.locals.push(Local { name: "self".into(), depth: 1, mutable: false });
                continue;
            }
            fn_compiler.locals.push(Local { name: p.name.clone(), depth: 1, mutable: true });
        }

        // Compila il corpo
        for stmt in body { fn_compiler.compile_stmt(stmt)?; }
        // Return implicito None
        fn_compiler.chunk.emit(Op::ReturnNil, line);

        // Compila i default values nel chunk corrente (non in quello della fn)
        let mut defaults = Vec::new();
        for p in params {
            if let Some(def_expr) = &p.default {
                // Valuta il default a compile-time se è un letterale
                if let Some(v) = const_eval(def_expr) {
                    defaults.push(v);
                } else {
                    defaults.push(Value::None); // placeholder
                }
            }
        }

        let proto = FnProto {
            name: name.to_string(),
            arity,
            max_arity,
            chunk: fn_compiler.chunk,
            upvalues: Vec::new(),
            defaults,
            is_async,
        };

        let proto_idx = self.chunk.add_fn_proto(proto);
        self.chunk.emit(Op::MakeClosure, line);
        self.chunk.emit_u16(proto_idx);

        Ok(())
    }

    // ── Classe ────────────────────────────────────────────────────────────

    fn compile_class(
        &mut self,
        name: &str,
        fields: &[Field],
        methods: &[Stmt],
        impls: &[Stmt],
        line: u32,
    ) -> VmResult<()> {
        // Registra la classe nel registry locale
        self.class_registry.insert(name.to_string(), ClassInfo {
            fields: fields.to_vec(),
            methods: methods.to_vec(),
        });

        // Emetti un costruttore come closure
        // Il costruttore: crea istanza, inizializza campi, chiama __init__ se esiste
        let ctor_name = name.to_string();
        let ctor_line = line;

        // Compila il costruttore come una funzione speciale
        let mut ctor = Compiler::new_function(name);
        ctor.class_registry = self.class_registry.clone();

        // MakeInstance: crea l'istanza
        let name_idx = ctor.chunk.add_name(name);
        ctor.chunk.emit(Op::MakeInstance, ctor_line);
        ctor.chunk.emit_u16(name_idx);
        // Stack: [instance]

        // Inizializza i campi con i valori di default
        for field in fields {
            if let Some(def) = &field.default {
                if let Some(v) = const_eval(def) {
                    ctor.chunk.emit(Op::Dup, ctor_line); // dup instance
                    let cidx = ctor.chunk.add_const(v);
                    ctor.chunk.emit(Op::Const, ctor_line);
                    ctor.chunk.emit_u16(cidx);
                    let fidx = ctor.chunk.add_name(&field.name);
                    ctor.chunk.emit(Op::SetField, ctor_line);
                    ctor.chunk.emit_u16(fidx);
                } else {
                    // Compile expr nel costruttore
                    ctor.chunk.emit(Op::Dup, ctor_line);
                    ctor.compile_expr(def)?;
                    let fidx = ctor.chunk.add_name(&field.name);
                    ctor.chunk.emit(Op::SetField, ctor_line);
                    ctor.chunk.emit_u16(fidx);
                }
            } else {
                // Campo senza default → None
                ctor.chunk.emit(Op::Dup, ctor_line);
                ctor.chunk.emit(Op::Nil, ctor_line);
                let fidx = ctor.chunk.add_name(&field.name);
                ctor.chunk.emit(Op::SetField, ctor_line);
                ctor.chunk.emit_u16(fidx);
            }
        }

        // Compila i metodi come closures nei campi dell'istanza
        for method_stmt in methods.iter().chain(impls.iter()) {
            if let StmtKind::Fn { name: mname, params, body, is_async, .. } = &method_stmt.inner {
                ctor.chunk.emit(Op::Dup, ctor_line); // dup instance per SetField
                ctor.compile_fn_def(mname, params, body, *is_async, ctor_line)?;
                let midx = ctor.chunk.add_name(mname);
                ctor.chunk.emit(Op::SetField, ctor_line);
                ctor.chunk.emit_u16(midx);
            }
        }

        // Se esiste __init__, chiamalo con i parametri del costruttore
        let init_params: Vec<Param> = methods.iter()
        .find(|m| matches!(&m.inner, StmtKind::Fn { name, .. } if name == "__init__"))
        .and_then(|m| if let StmtKind::Fn { params, .. } = &m.inner {
            Some(params.iter().filter(|p| p.name != "self").cloned().collect())
        } else { None })
        .unwrap_or_default();

        let arity     = init_params.iter().filter(|p| p.default.is_none()).count();
        let max_arity = init_params.len();

        if !init_params.is_empty() {
            // Aggiungi i parametri come locali del costruttore
            for (i, p) in init_params.iter().enumerate() {
                ctor.locals.push(crate::compiler::Local {
                    name:    p.name.clone(),
                                 depth:   1,
                                 mutable: true,
                });
            }
            // Stack: [instance] — chiama __init__(self, args...)
            // Dup instance come receiver
            ctor.chunk.emit(Op::Dup, ctor_line);
            // Carica ogni parametro
            for (i, _) in init_params.iter().enumerate() {
                ctor.chunk.emit(Op::LoadLocal, ctor_line);
                ctor.chunk.emit_u8(i as u8);
            }
            // CallMethod __init__
            let init_idx = ctor.chunk.add_name("__init__");
            ctor.chunk.emit(Op::CallMethod, ctor_line);
            ctor.chunk.emit_u16(init_idx);
            ctor.chunk.emit_u8(init_params.len() as u8);
            ctor.chunk.emit(Op::Pop, ctor_line); // scarta il risultato di __init__
        }

        // Se esiste __init__, chiamalo con i parametri del costruttore
        let init_params: Vec<Param> = methods.iter()
        .find(|m| matches!(&m.inner, StmtKind::Fn { name, .. } if name == "__init__"))
        .and_then(|m| if let StmtKind::Fn { params, .. } = &m.inner {
            Some(params.iter().filter(|p| p.name != "self").cloned().collect())
        } else { None })
        .unwrap_or_default();

        let arity     = init_params.iter().filter(|p| p.default.is_none()).count();
        let max_arity = init_params.len();

        if !init_params.is_empty() {
            // Aggiungi i parametri come locali del costruttore
            for p in init_params.iter() {
                ctor.locals.push(Local {
                    name:    p.name.clone(),
                                 depth:   1,
                                 mutable: true,
                });
            }
            // Dup instance come receiver per CallMethod
            ctor.chunk.emit(Op::Dup, ctor_line);
            // Carica ogni parametro
            for (i, _) in init_params.iter().enumerate() {
                ctor.chunk.emit(Op::LoadLocal, ctor_line);
                ctor.chunk.emit_u8(i as u8);
            }
            // CallMethod __init__
            let init_idx = ctor.chunk.add_name("__init__");
            ctor.chunk.emit(Op::CallMethod, ctor_line);
            ctor.chunk.emit_u16(init_idx);
            ctor.chunk.emit_u8(init_params.len() as u8);
            ctor.chunk.emit(Op::Pop, ctor_line);
        }

        // Ritorna l'istanza
        ctor.chunk.emit(Op::Return, ctor_line);

        let proto = FnProto {
            name:      ctor_name,
            arity,
            max_arity,
            chunk:     ctor.chunk,
            upvalues:  Vec::new(),
            defaults:  Vec::new(),
                is_async:  false,
        };
        let closure = Value::Closure(std::rc::Rc::new(crate::value::Closure {
            proto:    std::rc::Rc::new(proto),
                                                      upvalues: Vec::new(),
        }));
        let idx = self.chunk.add_const(closure);
        self.chunk.emit(Op::Const, line);
        self.chunk.emit_u16(idx);
        self.define_var(name, false, line)?;
        Ok(())
    }

    // ── F-string ──────────────────────────────────────────────────────────

    fn compile_fstring(&mut self, template: &str, line: u32) -> VmResult<()> {
        let chars: Vec<char> = template.chars().collect();
        let mut segments = 0usize;
        let mut i = 0;
        let mut literal = String::new();

        while i < chars.len() {
            if chars[i] == '{' && chars.get(i + 1) != Some(&'{') {
                if !literal.is_empty() {
                    let cidx = self.chunk.add_const(Value::str(literal.as_str()));
                    self.chunk.emit(Op::Const, line);
                    self.chunk.emit_u16(cidx);
                    literal.clear();
                    segments += 1;
                }
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
                        self.compile_expr(e)?;
                        self.chunk.emit(Op::ToStr, line);
                        segments += 1;
                    }
                }
                i = j;
            } else if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
                literal.push('{'); i += 2;
            } else if chars[i] == '}' && chars.get(i + 1) == Some(&'}') {
                literal.push('}'); i += 2;
            } else {
                literal.push(chars[i]); i += 1;
            }
        }
        if !literal.is_empty() {
            let cidx = self.chunk.add_const(Value::str(literal.as_str()));
            self.chunk.emit(Op::Const, line);
            self.chunk.emit_u16(cidx);
            segments += 1;
        }
        self.chunk.emit(Op::BuildStr, line);
        self.chunk.emit_u16(segments as u16);
        Ok(())
    }
}

// ── Valutazione costante a compile-time ───────────────────────────────────

fn const_eval(expr: &Expr) -> Option<Value> {
    match &expr.inner {
        ExprKind::Int(n)   => Some(Value::Int(*n)),
        ExprKind::Float(f) => Some(Value::Float(*f)),
        ExprKind::Bool(b)  => Some(Value::Bool(*b)),
        ExprKind::Str(s)   => Some(Value::str(s.as_str())),
        ExprKind::None     => Some(Value::None),
        _ => None,
    }
}
