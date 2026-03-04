# Neba — Roadmap

Ultimo aggiornamento: 2026-03-04 (v0.2.32)

---

## Stato attuale: v0.2.32 ✅

---

## Milestone completate

| Versione | Obiettivo | Data |
|----------|-----------|------|
| v0.1.0 | Rust workspace setup, CI, project layout | 2026-02-24 |
| v0.1.1 | Lexer: tokenizer completo, indentazione significativa, f-string, numeri | 2026-02-24 |
| v0.1.2 | Parser + AST: ricorsivo discendente, Pratt, if/match come espressioni | 2026-02-25 |
| v0.1.3 | Tree-walking interpreter: aritmetica interi e float | 2026-02-26 |
| v0.1.4 | REPL: loop interattivo, comando neba | 2026-02-26 |
| v0.1.5 | Stringhe base, print statement | 2026-02-26 |
| v0.1.6 | If/else, operatori di confronto | 2026-02-26 |
| v0.1.7 | While loop, for-range loop (`for i in 0..10`) | 2026-02-26 |
| v0.1.8 | Funzioni: definizione, chiamata, ricorsione, closures | 2026-02-26 |
| v0.1.9 | Modulo system base: stdlib abs/min/max/push/pop | 2026-02-26 |
| v0.1.10 | Benchmark suite v0 + documentation v0 (LANGUAGE.md) | 2026-02-26 |
| v0.2.0 | Bytecode VM: compiler AST→bytecode, stack machine, opcode set completo | 2026-02-25 |
| v0.2.1 | Type checker + inferenza di tipi (stub) | 2026-02-25 |
| v0.2.2 | Classi + trait dispatch (interpreter) | 2026-02-25 |
| v0.2.3 | Classes VM: definizione, istanziazione, metodi, self, __init__ | 2026-03-03 |
| v0.2.4 | Traits: definizione, impl, dispatch nella VM | 2026-03-03 |
| v0.2.5 | Lists e Dicts (dinamici, GC-managed) | 2026-03-03 |
| v0.2.6 | Native typed Array[T]: Float64, Int64, Int32, Float32 | 2026-03-03 |
| v0.2.7 | Array operations: indexing (0-based), slicing, aritmetica element-wise | 2026-03-03 |
| v0.2.8 | Match expression: bugfix completo (parser, compiler, VM) | 2026-03-03 |
| v0.2.9 | Mutable upvalue: `Rc<RefCell<Value>>` — closure counter/accumulatore | 2026-03-03 |
| v0.2.10 | Error handling: operatore `?`, `.is_ok()`, `.is_err()`, `.unwrap()` | 2026-03-03 |
| v0.2.11 | Standard library v0: math, io, string, collections | 2026-03-03 |
| v0.2.12 | HOF: map, filter, reduce + lambda expressions + match multi-line | 2026-03-03 |
| v0.2.13 | Benchmark suite v1 | 2026-03-03 |
| v0.2.14 | **Performance:** `FnProto.chunk: Rc<Chunk>` (elimina clone), lazy `IntRange` (elimina Vec) — VM **11.8×** più veloce del tree-walker | 2026-03-04 |
| v0.2.15 | **Bug fix sprint:** `__init__` 2×, costruttore con parametri, kwargs, `while`+`continue`, IntRange in HOF/len/sum; +12 globali | 2026-03-04 |
| v0.2.16 | **OOP:** operatore `is` — `obj is ClassName` e `obj is Trait` via `SetTraits` opcode | 2026-03-04 |
| v0.2.17 | **Dict O(1):** `Vec<(Value,Value)>` → `IndexMap<Value,Value>` (ordine inserimento preservato) | 2026-03-04 |
| v0.2.18 | **Stdlib expansion +42 funzioni:** math (sinh/cosh/tanh/cbrt/random), string (zfill/center/pad_left/is_digit), io, io.path | 2026-03-04 |
| v0.2.19 | **Value 40→24 bytes (-40%):** zero-alloc Dict lookup; `Array[range]` e `Str[range]` slice con IntRange | 2026-03-04 |
| v0.2.20 | **Constant folding:** tutti gli operatori su literal Int/Float/Bool/Str risolti a compile-time | 2026-03-04 |
| v0.2.21 | **Benchmark gate** — fib(35) 6437ms (2.3× CPython), int loop 264ns/iter. GO per v0.3.x. | 2026-03-04 |
| v0.2.22 | **Bug fix OOP:** `__init__` no-params, `str(instance)` chiama `__str__`, min/max su TypedArray | 2026-03-04 |
| v0.2.23 | **VM hot path:** LoadLocal0-3/StoreLocal0-3 specializzati; fast-path Int+Int inline; upvalues O(1) | 2026-03-04 |
| v0.2.24 | **Architettura:** unifica `call_value_sync`; Compiler registry clone ottimizzato | 2026-03-04 |
| v0.2.25 | **NdArray v1:** multidimensionale NumPy-like, `nd.*` module, broadcast, axis reductions | 2026-03-04 |
| v0.2.26 | **Peephole optimizer:** Const+Pop→Nop, Not+Not→Nop, Jump offset=0→Nop (reale con remap offset) | 2026-03-04 |
| v0.2.27 | **NdArray v2:** slicing 2D, boolean masking, `nd.gt/lt/ge/le/eq/ne`, `nd.masked`, `nd.nonzero` | 2026-03-04 |
| v0.2.28 | **NdArray v3:** multi-index r/w `m[[i,j]]=val`, axis reductions, advanced broadcasting `[3,1]+[3,3]` | 2026-03-04 |
| v0.2.29 | **NdArray view semantics:** `Rc<RefCell<TypedArrayData>>` — `m[i][j]=val` modifica originale; view condividono buffer O(1) | 2026-03-04 |
| v0.2.30 | **Stdlib audit fix:** 16 funzioni globali mancanti (`flatten/unique/concat/slice/index/count/merge/dict_get`), type predicates (`is_int/float/str/bool/none/array/dict`), `TypedArray(n,dtype)`, `math.is_nan/is_inf/is_finite`, `none` keyword lowercase, **parser bug fix** trait consecutivi senza body | 2026-03-04 |
| v0.2.31 | **Fix da audit profondo:** `unwrap_err()`, `value()`, `type(fn)→"Function"`, `type(range)→"Range"`, parser fix keyword-dopo-dot (`collections.none`), `collections.none_of` alias, float IEEE 754 (`1.0/0.0→Inf`) | 2026-03-04 |
| v0.2.32 | **find/find_index HOF, sorted(reverse), repr, string.reverse:** `find(arr,fn)`, `find_index(arr,fn)`, `sorted(arr,true)` reverse flag, `repr(v)`, `string.reverse(s)`, `string.char_at`, `string.index_of` | 2026-03-04 |

---

## Prossimi passi

| Versione | Obiettivo |
|----------|-----------|
| **v0.2.33** | **Stdlib audit finale:** `string.format` positional (`"{} {}"` → `format("a","b")`), `string.split` senza separatore → chars, `sorted(arr,key=fn)` con key extractor, `zip` con N array, `enumerate` con start, `random.choice` globale |
| **v0.2.34** | **IO migliorato:** `io.read_file` con encoding, `io.write_file` con append mode, `io.path.exists` robusto, `io.env` (getenv/setenv) |
| **v0.2.35** | **String methods su istanza:** `"hello".upper()`, `"hello".split(",")` — dispatch tramite VM method call su Str |

---

## Phase 0 — Foundation (v0.1.x) ✅ COMPLETATA

| Versione | Obiettivo |
|----------|-----------|
| v0.1.0 ✅ | Rust workspace setup, CI, project layout |
| v0.1.1 ✅ | Lexer: tokenizer completo |
| v0.1.2 ✅ | Parser + AST |
| v0.1.3 ✅ | Tree-walking interpreter |
| v0.1.4 ✅ | REPL interattivo |
| v0.1.5 ✅ | Stringhe base, print |
| v0.1.6 ✅ | If/else, confronti |
| v0.1.7 ✅ | While e for-range loop |
| v0.1.8 ✅ | Funzioni, ricorsione, closures |
| v0.1.9 ✅ | Modulo system base |
| v0.1.10 ✅ | Benchmark suite v0 + LANGUAGE.md |

**Risultati:** fib(30) = 8,514 ms. REPL in <200ms.

---

## Phase 1 — Bytecode VM & Core Types (v0.2.x) — IN CORSO

**Goal:** VM bytecode completa, classi, traits, array tipizzati, error handling, stdlib robusta, performance pre-JIT.

| Versione | Obiettivo | Stato |
|----------|-----------|-------|
| v0.2.0–v0.2.13 | VM, OOP, HOF, stdlib base | ✅ |
| v0.2.14–v0.2.20 | Performance, bug fix, ottimizzazioni | ✅ |
| v0.2.21 | Benchmark gate | ✅ |
| v0.2.22–v0.2.24 | VM hot path, OOP fix, architettura | ✅ |
| v0.2.25–v0.2.29 | NdArray completo con view semantics | ✅ |
| v0.2.30–v0.2.32 | Stdlib audit completo, 16 gap risolti, HOF avanzati | ✅ |
| v0.2.33–v0.2.35 | Stdlib finale, IO, string methods | 🔜 |

### Funzionalità presenti in v0.2.32

**Tipi:** `Int` `Float` `Bool` `Str` `None` `Array` `Dict` `Some` `Ok` `Err` `TypedArray` `NdArray` `Function` `Range`

**OOP:** classi con campi tipizzati e default, `__init__`, `__str__`, metodi, `self`, operatore `is` per classi e trait, traits con metodi astratti e default, `impl Trait for Class`

**Control flow:** `if/elif/else`, `while`, `for-in` (range/array/dict), `break`, `continue`, `match` (literal, range, Or-pattern, Option, Result, wildcard)

**Funzioni:** default args, closures con upvalue mutabili, lambda inline (`fn(x) x*2`), lambda arrow (`fn(x) => x*2`), HOF (`map/filter/reduce/find/find_index`)

**Error handling:** `Ok/Err`, `Some/None`, `.is_ok()` `.is_err()` `.unwrap()` `.unwrap_or()` `.unwrap_err()` `.value()`, operatore `?`

**Stdlib:** 80+ globali, 6 moduli (`math`, `string`, `io`, `io.path`, `collections`, `random`)

**NdArray:** multidimensionale, view semantics, broadcast NumPy-rules, axis reductions, boolean masking, matmul, reshape, transpose, argmax/argmin, cumsum

**Ottimizzazioni:** constant folding, peephole optimizer, LoadLocal0-3 specializzati, fast-path Int+Int, lazy IntRange, Value 24 byte

### Bug noti e limitazioni (v0.2.32)

| Descrizione | Workaround | Target |
|-------------|------------|--------|
| `string.format("{} {}", a, b)` usa solo `{chiave}` con dict | Usare f-string | v0.2.33 |
| `string.split("hello")` → `["hello"]` non chars | Usare `string.chars(s)` | v0.2.33 |
| `var`/`let` non supportati dentro lambda inline | Usare funzioni nominate | v0.3.x |
| Ereditarietà (`class B extends A`) non nell'AST | Usare traits con default | post-v1.0 |
| `*args`, `**kwargs` variadici non nell'AST | — | post-v1.0 |
| `async`/`await` reale | Stub sincrono | v0.4.x |
| String methods su istanza (`"hi".upper()`) | Usare `upper("hi")` | v0.2.35 |
| Destructuring assign (`let (a,b) = ...`) | — | v0.3.x |

---

## Phase 2 — VM Optimization & JIT Cranelift (v0.3.x)

**Goal:** Ottimizzazioni VM sistematiche, poi JIT-compile funzioni hot via Cranelift.

| Versione | Obiettivo |
|----------|-----------|
| v0.3.0 | **String interning:** tabella globale di deduplicazione; confronti O(1) |
| v0.3.1 | **NaN-boxing (unsafe):** Value 24→8 byte; `u64` con tag nei bit NaN; refactor 750+ match arms |
| v0.3.2 | **Inline caching** per `GetField`/`CallMethod`: salva ultima posizione nel Dict; skip lookup su cache hit |
| v0.3.3 | **JIT Cranelift integration:** compila funzioni hot a native code; call-count threshold; fallback all'interprete |
| v0.3.4 | **JIT type specialization:** versioni specializzate per tipo concreto (Int path, Float path) |
| v0.3.5 | **Tail call optimization:** `return f(args)` riutilizza il frame |
| v0.3.6 | **SIMD auto-vectorization** per array ops (via Cranelift SIMD) |
| v0.3.7 | **Concurrent GC v2:** concurrent mark phase, pause ridotte |
| v0.3.8 | Profiler v1 + Benchmark suite v2 + documentation v2 |

**Benchmark target (v0.3.x):** Loop numerici entro 3× di Rust. Array ops entro 2× di NumPy.

---

## Phase 3 — Parallelism & Concurrency (v0.4.x)

| Versione | Obiettivo |
|----------|-----------|
| v0.4.0 | Work-stealing task scheduler (Tokio/Rayon hybrid) |
| v0.4.1 | Keyword `spawn` per task leggeri |
| v0.4.2 | `async`/`await` per I/O concurrency |
| v0.4.3 | Parallel array ops (`pmap`, `preduce`) |
| v0.4.4 | SIMD a piena larghezza AVX2/AVX-512 |
| v0.4.5 | Concurrent GC v3 |
| v0.4.6 | Message passing via canali (ispirato a Go) |
| v0.4.7–v0.4.10 | Lock-free collections, race detection, profiler v2, docs |

**Benchmark target:** Parallel sum scala linearmente su 8 core. Nessun bottleneck GIL.

---

## Phase 4 — LLVM Backend & GPU (v0.5.x)

| Versione | Obiettivo |
|----------|-----------|
| v0.5.0–v0.5.2 | LLVM IR generation, O2/O3 pipeline, PGO |
| v0.5.3–v0.5.5 | GPU via wgpu: array backend, dispatch automatico, kernel specialization |
| v0.5.6–v0.5.8 | Executable output (`neba build`), cross-compilation, LTO |
| v0.5.9–v0.5.10 | GPU profiler, Benchmark suite v4, docs |

---

## Phase 5 — Interoperability (v0.6.x)

| Versione | Obiettivo |
|----------|-----------|
| v0.6.0–v0.6.4 | C FFI, C++ bridge, Rust FFI, Neba callable da C/Rust |
| v0.6.5–v0.6.8 | Python interop: chiamare Python, import moduli, buffer protocol, extension module |
| v0.6.9–v0.6.10 | FFI docs, safety guide, benchmark |

---

## Phase 6 — Ecosystem & Tooling (v0.7.x)

| Versione | Obiettivo |
|----------|-----------|
| v0.7.0–v0.7.2 | `neba.toml`, package manager `neba pkg`, registry |
| v0.7.3–v0.7.6 | Formatter `neba fmt`, linter `neba lint`, test runner `neba test`, doc generator |
| v0.7.7–v0.7.10 | LSP server, REPL avanzato, profiler flamegraph, docs |

---

## Phase 7 — Standard Library Expansion (v0.8.x)

| Modulo | Contenuto | Stato |
|--------|-----------|-------|
| `math` | Trig, exp, iperboliche, statistiche, random | ✅ completo (v0.2.x) |
| `string` | Padding, parsing, predicati, case, slice | ✅ quasi completo (v0.2.x) |
| `random` | LCG PRNG, seed, randint, choice, shuffle, sample | ✅ (v0.2.18) |
| `io` | File I/O, listdir, cwd, mkdir, io.path | ✅ parziale |
| `collections` | chunk, take, drop, product, transpose, first, last | ✅ (v0.2.x) |
| `time` | sleep, now, format, duration, timezone | ❌ mancante |
| `os` | getenv, environ, process, signals | ❌ mancante |
| `json` | parse, stringify, pretty-print | ❌ mancante |
| `re` | Espressioni regolari (match, search, sub) | ❌ mancante |
| `net` | HTTP client/server, TCP/UDP, WebSocket | ❌ mancante |
| `csv` | CSV reader/writer | ❌ mancante |
| `crypto` | Hashing, AES, RSA | ❌ mancante |
| `data` | DataFrame-like, Series | ❌ mancante |
| `plot` | Plotting base (terminal + SVG) | ❌ mancante |

---

## Phase 8 — Hardening & v1.0 (v0.9.x → v1.0)

| Versione | Obiettivo |
|----------|-----------|
| v0.9.0 | Language specification document (grammatica formale, semantica) |
| v0.9.1 | Test suite completa: 10.000+ test su tutti i componenti |
| v0.9.2 | Fuzz testing: lexer, parser, VM |
| v0.9.3 | Memory safety audit |
| v0.9.4 | Security review del layer FFI |
| v0.9.5 | Performance regression guard (CI blocca merge con regressione >5%) |
| v0.9.6 | Cross-platform validation: Linux, macOS, Windows |
| v0.9.7 | Embeddability API: Neba come scripting engine in app Rust |
| v0.9.8 | Final documentation pass |
| v0.9.9 | Release candidate |
| v1.0.0 | **Stable release** — API frozen, backward compat garantita |

---

## Note architetturali

- **GC:** Rc + RefCell (reference counting). Target: concurrent generational GC in v0.3.7
- **JIT:** Cranelift in v0.3.3, LLVM in v0.5.x. Prerequisito: VM optimization v0.3.0–v0.3.2
- **Value size:** 24 byte (post v0.2.19). Target NaN-boxing 8 byte: v0.3.1
- **Dict:** IndexMap O(1) con ordine di inserimento preservato (v0.2.17+)
- **Constant folding:** tutti gli operatori su literal risolti a compile-time (v0.2.20+)
- **Peephole:** Const+Pop→Nop, Not+Not→Nop, Jump offset=0→Nop con remap offset (v0.2.26+)
- **NdArray:** view semantics con `Rc<RefCell<TypedArrayData>>` — modifiche propagano attraverso view (v0.2.29+)
- **Float:** IEEE 754 — `1.0/0.0 = Inf`, `0.0/0.0 = NaN`; solo `Int/Int` lancia DivisionByZero (v0.2.31+)
- **Trait methods:** body vuoto = firma astratta (non viene ereditato come default); solo body con codice reale = default (v0.2.30+)
