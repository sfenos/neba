# Neba — CHANGELOG

Tutte le modifiche rilevanti del progetto sono documentate qui.
Il formato segue [Keep a Changelog](https://keepachangelog.com/it/1.0.0/).

---

## [0.1.2] — Tree-walking Interpreter — 2026-02-25

### Aggiunto
- **`neba_interpreter`** — crate completo con interprete tree-walking
  - `value.rs` — tipo runtime `Value`: Int, Float, Bool, Str, None, Some, Ok, Err, Array, Function, NativeFunction, Instance
  - `environment.rs` — scope annidati con `let` (immutabile) e `var` (mutabile), snapshot per chiusure
  - `error.rs` — errori runtime tipizzati: UndefinedVariable, AssignError, TypeError, DivisionByZero, IndexOutOfBounds, NotCallable, ArityMismatch, UnknownField, StackOverflow, Generic
  - `interpreter.rs` — interprete completo con `ClassMeta` registry
  - `stdlib.rs` — libreria standard: print, println, input, len, str, int, float, bool, typeof, abs, min, max, range, push, pop, assert
- **Funzionalità interpretate**
  - Aritmetica completa: +, -, \*, /, //, %, \*\* con promozione Int→Float automatica
  - Concatenazione stringhe con +, ripetizione con \* n
  - Confronto: ==, !=, <, <=, >, >=
  - Logica: and, or, not (con short-circuit)
  - Bitwise: &, |, ^, ~, <<, >>
  - Membership: in, not in (su Array e Str)
  - Operatori composti: +=, -=, \*=, /=, %=
  - f-string: interpolazione runtime con `{expr}` annidata
  - Indici negativi su Array e Str
  - Range: `0..n` (esclusivo), `0..=n` (inclusivo)
  - for/while con break e continue
  - if/elif/else come espressioni
  - match come espressione con pattern: Wildcard, Literal, Ident (binding), Constructor (Some/None/Ok/Err), Range, Or
  - Funzioni con parametri, default, chiusure (closure) e ricorsione
  - Classi con campi, metodi, field access/set
  - Costruttore automatico dalla definizione della classe
  - `__init__` opzionale per inizializzazione custom
  - spawn/await come stub sincroni (warning a runtime)
  - MAX_DEPTH = 50 per rilevamento stack overflow sicuro
- **CLI `neba`** aggiornato a v0.1.2 — esegue programmi completi
- **REPL `neba_repl`** completamente riscritto — interprete completo, output colorato ANSI, blocchi multi-riga (fn/class/if/while…), comandi speciali `:quit`, `:clear`, `:help`, banner ASCII-art
- 76 test unitari nell'interprete

### Modificato
- `value::Value::NativeFunction` — cambiato da `fn` pointer a `Rc<dyn Fn>` per supportare closures catturate nei metodi built-in di Str
- `value::Value` — `Debug` implementato manualmente (non derivabile con `Rc<dyn Fn>`)
- Built-in `type()` rinominato in `typeof()` per conflitto con keyword `type` del lexer

### Noto
- `spawn` è sincrono in v0.1.x — la concorrenza reale arriverà in v0.3.x
- `mod` e `use` sono stub — i moduli arriveranno in v0.2.x
- Trait e impl sono parsati ma non verificati a runtime (nessun duck typing enforcement)

---

## [0.1.1] — Parser + AST — 2026-02-25

### Aggiunto
- **`neba_parser`** — crate completo con parser ricorsivo discendente + Pratt
  - `ast.rs` — AST tipizzato: `Node<T>`, `Expr`, `Stmt`, `TypeExpr`, `Program`, `Pattern`, `MatchArm`, `Param`, `Field`
  - `error.rs` — 6 tipi di errore con posizione: UnexpectedToken, UnexpectedEof, InvalidAssignTarget, MissingIndent, MissingDedent, InvalidPattern
  - `parser.rs` — parser Pratt per espressioni, ricorsivo discendente per statement
  - `lib.rs` — API pubblica + 50 test unitari
- **Costrutti parsati**
  - Letterali: Int, Float, Bool, Str, FStr, None
  - Espressioni: binarie (con precedenza corretta), unarie, chiamate, field access, index, array, range, if/elif/else, match, spawn, await, Some/Ok/Err
  - Statement: let, var, assign (con operatori composti), fn, async fn, class, trait, impl, while, for, return, break, continue, pass, mod, use
  - Tipi: Named (`Int`), Generic (`Option[T]`, `Result[T, E]`)
  - Pattern: Wildcard, Literal, Ident, Constructor, Range, Or (`|`)
  - Error recovery: il parser continua dopo errori e accumula tutti gli errori
- **Decisioni di design**
  - if/match sono **espressioni** (restituiscono un valore)
  - I tipi built-in sono identificatori generici — il parser non li distingue
  - Error recovery attivo per migliore developer experience
- **CLI `neba`** aggiornato a v0.1.1 — stampa l'AST di primo livello

### Modificato
- `neba/src/main.rs` — aggiornato per usare `neba_parser::parse()` invece del tokenizer diretto

---

## [0.1.0] — Lexer — 2026-02-24

### Aggiunto
- Workspace Cargo con struttura multi-crate:
  ```
  neba/
  ├── crates/
  │   ├── neba_lexer/        ← v0.1.0
  │   ├── neba_parser/       ← stub
  │   ├── neba_interpreter/  ← stub
  │   ├── neba_repl/         ← stub
  │   └── neba/              ← CLI binary
  ├── tests/lexer_tests/
  └── docs/dev/
  ```
- **`neba_lexer`** — lexer completo
  - `token.rs` — ~70 varianti di TokenKind, Span con line/column/byte offset
  - `lexer.rs` — indentazione significativa a 4 spazi, stringhe triple, f-string, numerici hex/oct/bin/exp con separatori `_`, commenti `#`, operatori multi-carattere
  - `error.rs` — 6 tipi di errore: UnexpectedCharacter, UnterminatedString, InvalidEscapeSequence, InvalidNumber, InconsistentIndentation, TabSpaceMixing
  - `lib.rs` — API pubblica `tokenize()` + 17 test unitari
- **CLI `neba`** v0.1.0 — tokenizza un file e stampa i token
- **Documenti** in `docs/dev/`: `decisions.md`, `roadmap.md`

### Decisioni di design
- Indentazione: 4 spazi obbligatori (tab = errore)
- Naming: snake_case (var/fn), PascalCase (tipi), UPPER_SNAKE (costanti)
- Parallelismo: keyword `spawn expr`
- Null safety: Option[T] con Some/None
- String interpolation: f"text {expr}"
- Mutabilità: `let` (immutabile) / `var` (mutabile)

---

## Roadmap futura

| Versione | Obiettivo |
|----------|-----------|
| v0.2.0   | Bytecode VM + Garbage Collector generazionale |
| v0.2.1   | Sistema di tipi + inferenza |
| v0.2.2   | Classi complete + trait dispatch |
| v0.2.3   | REPL interattivo con history e completamento |
| v0.3.0   | Cranelift JIT + specializzazione |
| v0.3.1   | Scheduler parallelo + async/await reale |
| v0.4.0   | Backend LLVM + output eseguibile |
| v0.5.0   | Interop C/Rust/Python |
| v0.6.0   | Package manager + formatter + LSP |
| v0.7.0   | Standard library |
| v1.0.0   | Stable release |

---

## [0.2.0] — 2026-02-25

### Aggiunto
- **Crate `neba_vm`**: bytecode compiler + stack machine VM
- **`opcode.rs`**: set completo di istruzioni (LOAD/STORE, aritmetica, confronto, salti, call/return, iterazione, classi, f-string, match, Option/Result)
- **`chunk.rs`**: `Chunk` (sequenza di bytecode + costanti + nomi + info debug), `FnProto` (prototipo funzione), `UpvalueDesc`
- **`value.rs`**: `Value` enum con `Closure`, `Instance`, `NativeFn` — valori heap-allocated via `Rc<RefCell<_>>`
- **`compiler.rs`**: visita AST → bytecode; gestione scope/locali/globali, chiusure, classi, pattern matching, f-string, loop for/while con break/continue
- **`vm.rs`**: loop di dispatch `step()`, call frame stack, built-in NativeFn, iterazione array/stringa, operatori bitwise/logici
- **`stdlib.rs`**: 15 built-in registrati come `NativeFn` (print, len, str, int, float, push, pop, range, abs, min, max, assert, typeof…)
- **`error.rs`**: `VmError` con StackOverflow, TypeError, ArityMismatch, ecc.
- **`lib.rs`**: API pubblica `run(source)`, `eval(source) -> Value`, 61 test di integrazione

### Corretto
- Bug in `compile_for`: i locali impliciti `__iter`/`__pos`/`var` venivano rimossi da `pop_scope` del body anziché dallo scope del for — causava panic `index out of bounds` su tutti i loop `for` con range. Fix: `push_scope()` prima di aggiungere i locali impliciti, `pop_scope()` per chiuderli invece del manuale `locals.pop() × 3 + PopN 3`

### Statistiche
- Test totali: **204** (lexer 17 + parser 50 + interpreter 76 + vm 61)
- Zero errori di compilazione, 1 warning inoffensivo

### Note
- `spawn`/`await` rimangono stub sincroni (pianificato v0.3.1)
- `mod`/`use` emettono warning (pianificato v0.2.x)
- GC: attualmente Rc<RefCell<_>> — mark & sweep generazionale pianificato in iterazione successiva

---

## [0.2.0] — 2026-02-25

### Aggiunto
- **Bytecode VM** (`neba_vm`) — stack machine completa: compilatore AST→bytecode, esecutore bytecode, GC via Rc
- **61 nuovi test** per la VM (204 totali nel workspace)
- **Opcode set completo**: costanti, locali, globali, aritmetica, bitwise, confronto, salti, funzioni, array, classi, Option/Result, pattern matching, iteratori, f-string
- **Compilatore** (`compiler.rs`): visita l'AST e genera Chunk con bytecode ottimizzato (deduplicazione costanti, patch dei jump, scoping a due livelli per for-loop)
- **VM** (`vm.rs`): loop `step()` con macro per lettura operandi, call frames, stack di valori, tabella globali, traceback degli errori
- **Standard library** (`stdlib.rs`): 16 funzioni native registrate come NativeFn

### Bug corretti durante sviluppo
- `ip` immutabile nelle macro di lettura operandi → letture multi-byte (DefGlobal, IterNext, MatchRange) leggevano offset errati
- `StoreGlobal`/`StoreLocal` usavano `peek` invece di `pop` → valori residui corrompevano lo stack
- `SetField`/`SetIndex` pushavano `None` dopo l'operazione → stack leak
- `compile_assign` emetteva `Nil` superfluo come "valore di ritorno" delle assegnazioni-statement
- Costruttore di classi emetteva `Pop` dopo ogni `SetField` (non più necessario)
- Scoping del `for`: i meta-locali (iter/pos/var) venivano distrutti da `pop_scope` del body; risolto con doppio scope annidato

### Noti limiti in v0.2.0
- `spawn`/`await` sono sincroni (stub)
- `mod`/`use` emettono warning ma non importano moduli
- Trait/impl vengono compilati (no-op) ma non enforced
- GC è reference counting (Rc) — no cicli; GC generazionale pianificato in v0.2.x

### Roadmap aggiornata
- v0.1.3 (REPL readline) → spostato a v0.2.3
