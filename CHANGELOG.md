# Neba — CHANGELOG

Tutte le modifiche rilevanti del progetto sono documentate qui.
Il formato segue [Keep a Changelog](https://keepachangelog.com/it/1.0.0/).

## [v0.2.15] — 2026-03-04 — Bug fixes critici + globali mancanti

### Bug risolti

- **BUG-1 — `__init__` chiamato due volte**: il blocco di compilazione della chiamata a
  `__init__` era duplicato letteralmente in `compile_class`. Ogni costruzione di oggetto con
  `__init__` eseguiva l'inizializzatore due volte. Rimosso il secondo blocco identico.

- **BUG-2 — `kwargs` silenziosamente sbagliati**: le chiamate con argomenti nominati
  (`f(x=1, y=2)`) compilavano scartando i nomi, passando i valori nell'ordine del sito di
  chiamata. `greet(greeting="Hi", name="Bob")` produceva `"Bob, Hi!"` invece di `"Hi, Bob!"`.
  Ora emette un errore a compile-time esplicito: *"named arguments (kwargs) not yet supported"*.

- **BUG-3 — costruttore con parametri — `arity` errata e `LoadLocal` su slot sbagliati**:
  il `FnProto` del costruttore aveva `arity = 0` quando `__init__` aveva parametri. I locali
  venivano registrati dopo l'emissione del bytecode, causando `LoadLocal` su slot non inizializzati.
  Fix: i locali dei parametri vengono registrati **prima** di emettere qualsiasi bytecode; il
  `FnProto` viene creato con `arity = arity_di___init__`.

- **BUG-4 — `compile_while` mancava di `loop_local_counts.push`**: il metodo faceva
  `loop_local_counts.pop()` in fondo senza mai fare `push`, rimuovendo il contatore del
  `for` loop più interno in caso di `while` annidato in `for`. Aggiunto `push(0)` all'inizio.

### Funzionalità aggiunte

**`IntRange` nei HOF e in `len()`:**
- `map(0..N, fn)`, `filter(0..N, fn)`, `reduce(0..N, fn, init)` ora accettano IntRange
- `len(0..10)` → `10`, `len(0..=10)` → `11`
- `sum(1..=100)` → `5050` (calcolato con formula di Gauss, O(1))

**Nuove funzioni globali:**
| Funzione | Descrizione |
|---|---|
| `sum(array\|range\|typedarray)` | Somma elementi; usa formula Gauss per range |
| `zip(a, b)` | Combina due array in array di coppie |
| `enumerate(array, start=0)` | Array di `[index, value]` |
| `sorted(array)` | Nuova array ordinata (non modifica l'originale) |
| `any(array)` | `true` se almeno un elemento è truthy |
| `all(array)` | `true` se tutti gli elementi sono truthy |
| `chr(n)` | Carattere Unicode dal codepoint |
| `ord(s)` | Codepoint Unicode del primo carattere |
| `copy(array\|dict\|typedarray)` | Copia superficiale |
| `hex(n)` | `"0xff"` |
| `bin(n)` | `"0b1010"` |
| `oct(n)` | `"0o17"` |

**Fix `sum()` unificato:** eliminata la registrazione duplicata `ta_sum` che sovrascriveva
`neba_sum`. La funzione `sum()` globale ora gestisce `Array`, `IntRange`, e `TypedArray`
in un'unica implementazione.

### Test
- 44 nuovi test in `tests/test_v0215.neba`
- 208/208 test unit passati (nessuna regressione)

---

## [0.2.13] — 2026-03-03

### Aggiunto

**Benchmark Suite v1** (`benchmarks/bench_v1.neba` + `benchmarks/results_v1.md`)

Aggiornamento della suite v0 (v0.1.10) — estende i 9 benchmark classici con 6 nuove sezioni che esercitano le feature introdotte in v0.2.x:

- **Sezione A** (benchmark A1–A9): identici alla v0 — permettono il confronto diretto tree-walking interpreter (v0) vs bytecode VM (v1)
- **Sezione B** (TypedArray): `ones(1M) + sum`, `dot(100k) ×100`, `linspace(1M) + mean` — misura il throughput della VM su storage compatto
- **Sezione C** (stdlib math): loop su `sin+cos`, `sqrt+log`, `floor+ceil` — 500k–2M chiamate; misura il costo del dispatch Dict+NativeFn
- **Sezione D** (stdlib string): `split+join` 50k, `replace+upper` 100k, `find` 200k — misura le operazioni su `Rc<String>`
- **Sezione E** (HOF): `map`/`filter`/`reduce` su 100k elementi, pipeline completa, **confronto HOF vs loop manuale** (E5) per quantificare l'overhead di `call_value_sync`
- **Sezione F** (collections): `sorted` 10k×10, `zip`/`enumerate` 50k, `flatten` 10k×10

`results_v1.md` è un template precompilato con i valori v0 già inseriti nella colonna di confronto.

---



### Aggiunto

**HOF: `map`, `filter`, `reduce`**
- `map(array, fn)` → Array — applica `fn` a ogni elemento, restituisce nuovo array
- `filter(array, fn)` → Array — mantiene gli elementi per cui `fn` restituisce truthy
- `reduce(array, fn)` → Value — piega l'array con accumulatore, primo elemento come valore iniziale
- `reduce(array, fn, initial)` → Value — versione con valore iniziale esplicito
- Funzionano con lambda inline (`fn(x) => expr`), funzioni nominate, e closure che catturano variabili esterne
- Combinabili: `reduce(map(filter(...), ...), ...)`

**Architettura HOF:** le HOF richiedono callback nel VM, ma `NativeFn` è `fn(&[Value])` senza accesso alla VM. Soluzione: `call_value_sync` — un mini run-loop annidato che esegue una Closure sincrona all'interno del frame corrente. `map`/`filter`/`reduce` vengono intercettati in `Op::Call` prima del dispatch normale (i `NativeFn` registrati in stdlib sono placeholder che non vengono mai eseguiti).

**Match multi-linea: corpo indentato dopo `=>`**
- Prima: `case pattern => singola_espressione` (solo)
- Ora: se dopo `=>` c'è un newline+indent, il parser legge un blocco completo (if/else, while, variabili locali, return, ecc.)
- Fix in `parse_match_expr` del parser: detect newline post-`=>` e dispatch a `parse_block()`

### Note
- `call_value_sync` non ha limite di step — i loop infiniti dentro le HOF non vengono rilevati. Sarà affrontato in futuro col refactor del step counter condiviso.

---



### Aggiunto

**Standard Library v0: quattro moduli namespace accessibili come `modulo.funzione()`**

#### `math` — Matematica completa
- **Costanti:** `math.pi`, `math.e`, `math.tau`, `math.inf`, `math.nan`
- **Aritmetica:** `sqrt`, `pow`, `exp`, `log(x)`, `log(x, base)`, `log2`, `log10`
- **Trigonometria:** `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2(y,x)`, `hypot`
- **Conversioni angolari:** `degrees`, `radians`
- **Arrotondamento:** `floor`, `ceil`, `round(x)`, `round(x, decimals)`, `trunc`
- **Utilità:** `sign`, `clamp(x, lo, hi)`, `gcd`, `lcm`, `factorial` (0–20)
- **Predicati:** `isnan`, `isinf`

#### `string` — Manipolazione stringhe
- **Splitting/joining:** `split(s)`, `split(s, sep)`, `split(s, sep, n)`, `lines`, `chars`
- **Pulizia:** `strip`, `lstrip`, `rstrip` (alias: `trim`)
- **Case:** `upper`, `lower`
- **Ricerca:** `find`, `rfind`, `count`, `contains`, `startswith`, `endswith`, `index`
- **Trasformazione:** `replace(s, from, to)`, `replace(s, from, to, n)`, `repeat`, `pad_left`, `pad_right`
- **Predicati:** `is_empty`
- **Template:** `format(template, dict)` — sostituisce `{key}` con valori dal dict

#### `io` — I/O su file
- `read_file(path)` → Str
- `write_file(path, content)` → None (crea/sovrascrive)
- `append_file(path, content)` → None
- `file_exists(path)` → Bool
- `read_lines(path)` → Array[Str]
- `delete_file(path)` → None

#### `collections` — Utilità su collezioni
- **Costruzione:** `zip(a, b)`, `enumerate(arr)`, `enumerate(arr, start)`, `concat(a, b)`, `repeat(arr, n)`
- **Trasformazione:** `flatten`, `unique`, `sorted`, `chunk(arr, n)`, `transpose`
- **Selezione:** `take(arr, n)`, `drop(arr, n)`, `first(arr)` → Option, `last(arr)` → Option
- **Aggregazione:** `sum`, `product`, `count_by(arr, value)`
- **Predicati:** `any`, `all`, `none`

### Modificato
- **`vm.rs` `get_field`**: aggiunto supporto Dict — lookup per nome stringa. Questo abilita la sintassi `modulo.funzione(args)` dove il modulo è un Dict di NativeFn.
- **`stdlib.rs` `register_globals`**: registra `math`, `string`, `io`, `collections` come Dict globali.

### Note architetturali
- I moduli sono `Value::Dict` con chiavi `Value::Str` e valori `Value::NativeFn`. Il `GetField` della VM ora risolve i Dict per nome — zero overhead a runtime rispetto alle funzioni globali.
- HOF (`map`, `filter`, `reduce`) richiedono callback nel VM (`NativeFn` è `fn(&[Value]) -> Result<Value, String>`, senza accesso alla VM). Verranno implementati come opcode dedicati in v0.2.12 o tramite refactor `NativeFn` → closure Rust.
- `collections.first`/`last` restituiscono `Option` (Some/None) per uniformità con il tipo system Neba.

### Test
- Aggiunto `test_v0211.neba`: 80+ asserzioni su tutti e 4 i moduli
- math: costanti, trigonometria, arrotondamento, utilità numeriche
- string: split, case, replace, find, pad, format
- io: write/read/append/delete/exists file su `/tmp`
- collections: zip, enumerate, flatten, unique, sort, chunk, sum, any/all/none, transpose

---


## [0.2.10] — Error handling

### Aggiunto
- **`?` operator** (propagazione errori): `ExprKind::Try` in AST, parsing postfix in parser, opcode `Propagate` in VM
  - Su `Ok(v)`: unwrappa e continua con `v`
  - Su `Err(e)`: early return immediato con `Err(e)` dalla funzione corrente
- **Metodi built-in su Result** (via `CallMethod`):
  - `.is_ok()` → Bool
  - `.is_err()` → Bool
  - `.unwrap()` → valore interno (panic su Err)
  - `.unwrap_or(default)` → valore interno o default
- **Metodi built-in su Option**:
  - `.is_some()` → Bool
  - `.is_none()` → Bool
  - `.unwrap()` → valore interno (panic su None)
  - `.unwrap_or(default)` → valore interno o default
- **`ExprKind::Try`** aggiunto a AST, interprete tree-walking, typecheck (stub)

### Statistiche
- Test totali: **208** (neba_vm) — zero regressioni
- Suite `result_v0210_tests`: 44 test

### Note
- Due bug trovati durante lo sviluppo della suite (test corretti, non bug VM):
  - c3: logica test sbagliata (divisore era 2, non 0)
  - h4: sintassi `fn init(v)` non supportata dalla classe — usa field declaration

---

## [0.2.9] — Mutable upvalue fix

### Modificato
- **`value.rs`** — `Upvalue.value` cambiato da `Value` (copia diretta) a `Rc<RefCell<Value>>` (cella condivisa)
- **`vm.rs`** — `MakeClosure`: ogni upvalue catturato è ora wrappato in `Rc::new(RefCell::new(v))`
- **`vm.rs`** — `LoadUpval`: legge via `borrow().clone()`
- **`vm.rs`** — `StoreUpval`: scrive via `borrow_mut()` nella cella condivisa
- **`vm.rs`** — `Call`/`TailCall`: `upvalues.clone()` ora clona gli `Rc` (incrementa refcount, stesso RefCell) — le mutazioni persistono tra chiamate successive

### Bug risolti
- **Closure counter/accumulatore**: chiamate successive alla stessa closure ora vedono le mutazioni degli upvalue delle chiamate precedenti (prima ogni chiamata clonava i valori freschi dalla closure, perdendo le modifiche)
- **Istanze indipendenti**: due closure create dallo stesso factory non condividono stato (celle distinte per istanza)

### Test aggiunti (4 nuovi)
- `t_upvalue_counter_basic` — counter incrementa a 1, 2, 3
- `t_upvalue_counter_five_calls` — 5 chiamate → 5
- `t_upvalue_accumulator` — add(3) + add(4) + add(10) → 17
- `t_upvalue_independent_instances` — due counter indipendenti

### Statistiche
- Test totali: **137** (neba_vm) — zero regressioni

---

## [0.2.8] — bugfix match

### Bug risolti
- **Fix I**: `compile_match` — arm cleanup ora emette `Swap+Pop` per ogni
  binding locale (precedentemente le variabili di binding restavano sullo
  stack come leak, corrompendo i local slot nelle iterazioni successive)
- **Fix J**: `compile_pattern_bind` — `Constructor(Some/Ok/Err, [Ident])`
  registra il local direttamente sul valore unwrappato senza un `Dup`
  extra (eliminato il doppio slot per binding `Some(v)`, `Ok(v)`, `Err(v)`)

### Corretti nella sessione precedente (v0.2.7-fix)
- Fix A/B: `parser.rs` — keyword `case` opzionale negli arm; `Dedent` orfani
  skippati nel top-level (causa principale OOM/loop infinito)
- Fix C/D: `compiler.rs` — offset `fail_patches` per `MatchLit` e `MatchRange`
  corretti (`patch+1` → `patch+3`/`patch+6`)
- Fix E/F: aggiunto `Swap+Pop` finale + opcode `Swap`
- Fix G/H: `Unwrap` in check solo per sub-pattern non-Ident; subject
  salvato come local implicito per correggere slot numbering

### Test
- `test_match.neba`: 11/11 ✅
- Suite totale: 199/199 ✅

## [0.2.7] — TypedArray operations: indexing, slicing, aritmetica — 2026-03-03

### Aggiunto
- **Indexing** su TypedArray: `a[i]` (positivo e negativo), `a[i] = val`
- **Slicing** con range: `a[1..4]` → nuovo TypedArray dello stesso dtype
- **Aritmetica element-wise**: `+`, `-`, `*`, `/` tra TypedArray e scalare o TypedArray
  - Broadcast scalare commutativo: `scalar OP array` e `array OP scalar`
  - Array-array: stessa lunghezza e dtype, o promozione a Float64 per dtype misti
- **Funzioni aggregate**: `sum()`, `mean()`, `dot()`, `min_elem()`, `max_elem()`
- **`to_list()`**: converte TypedArray in Array dinamico
- **Iterazione**: `for x in typed_array` itera sugli elementi come scalari
- **31 nuovi unit test** + **36 test di integrazione** in `test_v026_v027.neba`
  - Include caso realistico: regressione lineare con dot/sum su Float64Array

---

## [0.2.6] — Native TypedArray (Float64, Int64, Int32, Float32) — 2026-03-03

### Aggiunto
- **Tipo `TypedArray`** nella VM: `Value::TypedArray(RcTypedArray)`
  - Rappresentazione interna compatta: `Vec<f64>`, `Vec<f32>`, `Vec<i64>`, `Vec<i32>`
  - Enum `Dtype` e `TypedArrayData` con metodi `get()`, `set()`, `len()`
- **Costruttori stdlib**: `Float64([...])`, `Float32([...])`, `Int64([...])`, `Int32([...])
- **Utilità**: `zeros(n)`, `zeros(n, dtype)`, `ones(n)`, `fill(n, val)`, `linspace(start, stop, n)`
- `len()` e `typeof()` aggiornati per TypedArray
- Stub nel typecheck e nell'interpreter tree-walking

---



### Aggiunto
- **Tipo `Dict`** nella VM (`Value::Dict(RcDict)`, `Vec<(Value, Value)>` con ordine inserimento)
  - Letterale `{"chiave": valore, ...}` e `{}` (dict vuoto)
  - Accesso con `d["key"]`, assegnazione `d["key"] = val`
  - Chiavi di qualsiasi tipo (`Str`, `Int`, ecc.)
  - `in` / `not in` controlla le chiavi
  - Iterazione con `for pair in dict` → ogni passo è `[chiave, valore]`
- **Nuovi opcode:** `MakeDict [u16:count]`
- **Funzioni stdlib Dict:** `keys()`, `values()`, `items()`, `has_key()`, `del_key()`
- **Funzioni stdlib List (Array):** `append()`, `remove()`, `contains()`, `insert()`, `sort()`, `reverse()`, `join()`
- `len()` aggiornato per supportare Dict
- Parser aggiornato: `{key: val}` dict literal, classi e impl senza corpo ora accettati (fix parser)
- Stub Dict nel typecheck (`Type::Any`) e interpreter tree-walking (errore esplicito)
- **23 test di integrazione** in `test_v025.neba`
- **21 nuovi unit test** in `dict_tests` (neba_vm)

### Invariato
- Tutti i 258 test precedenti passano senza modifiche

---



### Aggiunto
- **Sistema trait completo nella bytecode VM**
  - `trait Foo` — definizione di un trait con metodi richiesti e metodi default
  - `impl Trait for Class` — implementazione di un trait su una classe esistente
  - Dispatch dei metodi trait via istanza (i metodi dell'impl diventano campi dell'istanza al momento della costruzione)
  - Metodi **default** del trait: se il body del metodo non è solo `pass`, viene usato automaticamente sulle classi che non lo sovrascrivono
  - Metodi della classe hanno **precedenza** sui default del trait (override implicito)
  - Più classi possono implementare lo stesso trait con dispatch indipendente
- **Pre-pass nel compilatore** per raccogliere `trait` e `impl` prima della compilazione delle classi
  - `trait_registry: HashMap<String, TraitInfo>` — mappa trait_name → metodi (con default)
  - `impl_registry: HashMap<String, Vec<Stmt>>` — mappa class_name → metodi dell'impl
  - Entrambi i registry propagati ai sotto-compilatori (metodi, costruttori)
- **Miglioramenti parser**
  - Classi senza corpo (`class Foo`) ora accettate — utili con i trait
  - `impl Trait for Class` senza corpo ora accettato — usa solo i default del trait
  - `pass` in impl body = impl esplicito vuoto (usa tutti i default del trait)
- **Nuovi struct nel compilatore**
  - `TraitInfo { methods: Vec<Stmt> }` — metadati di un trait
  - `ClassInfo.traits: Vec<String>` — aggiunto campo per i trait implementati (base per `is` futuro)
- **5 nuovi unit test** in `trait_tests` (neba_vm):
  - `t_trait_basic_dispatch` — dispatch base su istanza
  - `t_trait_two_classes_same_trait` — due classi con lo stesso trait
  - `t_trait_default_method` — metodo default dal trait senza override
  - `t_trait_override_default` — override del default con metodo dell'impl
  - `t_trait_multiple_methods` — impl con più metodi

### Invariato
- Tutti i 258 test esistenti passano senza modifiche

---



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

## v0.2.14 — Ottimizzazioni VM: Rc<Chunk> + lazy IntRange (2026-03-04)

### Motivazione
I benchmark v1 rivelavano che la bytecode VM era più lenta del tree-walking
interpreter (A1: 81,383ms vs 67,673ms). Causa: overhead strutturale superiore
al guadagno del bytecode. Due fix eliminano entrambe le cause radice.

### Fix 1: `FnProto.chunk: Rc<Chunk>`
- `chunk.rs`: campo `chunk` in `FnProto` cambiato da `Chunk` a `Rc<Chunk>`
- `compiler.rs`: costruzione FnProto usa `Rc::new(fn_compiler.chunk)`
- `vm.rs`: 5 siti `Rc::new(proto.chunk.clone())` → `Rc::clone(&proto.chunk)`
- Elimina il clone dell'intero bytecode (code, constants, names, fn_protos)
  ad ogni chiamata di funzione. Su fib(30) questo avveniva milioni di volte.

### Fix 2: Lazy integer ranges
- `value.rs`: aggiunto `Value::IntRange(i64, i64, bool)` — start, end, inclusive
- `vm.rs / MakeRange`: emette `IntRange` invece di `Vec<Value::Int>`
- `vm.rs / IntoIter`: passa `IntRange` through senza convertire in Vec
- `vm.rs / IterNext`: branch `IntRange` — usa aritmetica intera pura, zero alloc
- `vm.rs / eval_in`: gestisce `x in 0..N` con aritmetica intera
- Applicato in entrambi i loop: `run_chunk` e `call_value_sync`

### Risultati (release build, hardware: —)

| Benchmark | Prima | Dopo | Speedup |
|-----------|-------|------|---------|
| A1 int arith 10M | 81,383 ms | 5,733 ms | **14.2×** |
| A2 float arith 5M | 24,768 ms | 1,823 ms | **13.6×** |
| A3 for-range 1M | 2,160 ms | 138 ms | **15.7×** |
| A4 fib(30) | 8,852 ms | 553 ms | **16.0×** |
| A5 closure 500k | 2,876 ms | 192 ms | **15.0×** |
| B2 dot product ×100 | 810 ms | 34 ms | **23.8×** |

