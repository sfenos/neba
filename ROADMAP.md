# Neba — Roadmap

Ultimo aggiornamento: 2026-03-04

---

## Stato attuale: v0.2.20 ✅

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
| v0.2.15 | **Bug fix sprint:** `__init__` 2×, costruttore con parametri, kwargs, `while`+`continue`, IntRange in HOF/len/sum; +12 globali (sum/zip/enumerate/sorted/any/all/chr/ord/copy/hex/bin/oct) | 2026-03-04 |
| v0.2.16 | **OOP:** operatore `is` corretto — `obj is ClassName` e `obj is Trait` via `SetTraits` opcode + `Instance.traits: Vec<String>` | 2026-03-04 |
| v0.2.17 | **Dict O(1):** `Vec<(Value,Value)>` → `IndexMap<Value,Value>` (ordine inserimento preservato); `impl Hash for Value` su tutti i 19 variant | 2026-03-04 |
| v0.2.18 | **Stdlib expansion +42 funzioni:** math (sinh/cosh/tanh/cbrt/abs/log1p/expm1), random (LCG Knuth: random/randint/choice/shuffle/seed/sample), string (zfill/center/ljust/rjust/to_int/to_float/is_digit/is_alpha/capitalize/title/slice), io (listdir/cwd/mkdir), io.path (join/dirname/basename/stem/ext/exists/isfile/isdir) | 2026-03-04 |
| v0.2.19 | **Value 40→24 bytes (-40%):** `NativeFn(Rc<String>,fn)`; zero-alloc Dict lookup via `Equivalent<Value> for str`; `Array[range]` e `Str[range]` slice con IntRange | 2026-03-04 |
| v0.2.20 | **Constant folding:** tutti gli operatori su literal Int/Float/Bool/Str risolti a compile-time; promozione Int/Float automatica; folding ricorsivo sull'AST (`2+3*4` → `Const(14)`) | 2026-03-04 |

---

## Prossimi passi immediati

| Versione | Obiettivo |
|----------|-----------|
| **v0.2.21** | **Benchmark gate** — misura performance attuale su suite completa (fib/loop/OOP/array/string); confronto con target; decisione go/no-go per JIT Cranelift |
| **v0.2.22** | **Bug fix OOP:** `__init__(self)` senza parametri non esegue il body; `str(instance)` non chiama `__str__`; `min/max` su TypedArray restituisce array; `sorted()` ignora comparatore custom |
| **v0.2.23** | **Ottimizzazioni VM hot path:** fast-path inline `Int+Int` in `Op::Add`; `step_limit` con `#[cfg(debug_assertions)]`; `upvalues: Rc<Vec<Upvalue>>` |
| **v0.2.24** | **Architettura:** unifica `call_value_sync` con loop principale (~250 righe duplicate); `Compiler` registry con `Arc<HashMap>` invece di clone per ogni funzione |

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
| v0.2.0 | Bytecode instruction set + compiler AST→bytecode | ✅ |
| v0.2.1 | Stack-based VM bytecode + type checker (stub) | ✅ |
| v0.2.2 | Classi + trait dispatch (interpreter) | ✅ |
| v0.2.3 | Classes VM: definizione, istanziazione, metodi, self, __init__ | ✅ |
| v0.2.4 | Traits: definizione, implementazione, dispatch | ✅ |
| v0.2.5 | Lists e Dicts (dinamici) | ✅ |
| v0.2.6 | Native typed Array[T]: Float64, Int64, Int32, Float32 | ✅ |
| v0.2.7 | Array operations: indexing, slicing, aritmetica | ✅ |
| v0.2.8 | Match expression: bugfix completo | ✅ |
| v0.2.9 | Mutable upvalue: `Rc<RefCell<Value>>` | ✅ |
| v0.2.10 | Error handling: `?`, Option/Result methods | ✅ |
| v0.2.11 | Standard library v0 | ✅ |
| v0.2.12 | HOF: map, filter, reduce + match multi-line | ✅ |
| v0.2.13 | Benchmark suite v1 | ✅ |
| v0.2.14 | Performance: FnProto `Rc<Chunk>`, lazy IntRange — VM 11.8× più veloce | ✅ |
| v0.2.15 | Bug fix sprint: __init__, costruttore, continue, +12 globali | ✅ |
| v0.2.16 | OOP: operatore `is` corretto per classi e trait | ✅ |
| v0.2.17 | Dict O(1): IndexMap + Hash per Value | ✅ |
| v0.2.18 | Stdlib expansion: +42 funzioni (math/random/string/io/io.path) | ✅ |
| v0.2.19 | Value 40→24 bytes, zero-alloc Dict lookup, Array/Str slice IntRange | ✅ |
| v0.2.20 | Constant folding: tutti gli operatori su literal | ✅ |
| v0.2.21 | Benchmark gate — misura e go/no-go per JIT | 🔜 |
| v0.2.22 | Bug fix OOP: __init__ no-params, __str__, min/max TypedArray | 🔜 |
| v0.2.23 | Ottimizzazioni VM hot path: Add fast-path, upvalues Rc | 🔜 |
| v0.2.24 | Architettura: unifica call_value_sync, Compiler registry Arc | 🔜 |

### Bug noti al 2026-03-04 (da ANALISI_v0.2.14.md)

| ID | Descrizione | Priorità | Target |
|----|-------------|----------|--------|
| BUG-OOP-1 | `__init__(self)` senza parametri non esegue il body | 🔴 Alta | v0.2.22 |
| BUG-OOP-2 | `str(instance)` / f-string non chiama `__str__` | 🔴 Alta | v0.2.22 |
| BUG-OOP-3 | `min/max` su TypedArray restituisce l'array invece del valore | 🟡 Media | v0.2.22 |
| BUG-OOP-4 | `sorted()` ignora comparatore custom | 🟡 Media | v0.2.22 |
| BUG-LANG-1 | Named kwargs (`f(x=1)`) silenziosamente ignorati — nome scartato | 🔴 Alta | v0.2.24 |
| BUG-ARCH-1 | `call_value_sync` duplica ~250 righe del loop principale | 🟡 Media | v0.2.24 |

### Funzionalità mancanti note (post-v1.0 o non pianificate a breve)

| Feature | Note |
|---------|------|
| Ereditarietà (`class B extends A`) | Non nell'AST |
| Operator overloading (`__eq__`, `__add__`, `__str__`) | Non dispatchati |
| `__iter__` custom | Solo tipi built-in iterabili |
| Tuple, Set | Nessun `Value::Tuple`, `Value::Set` |
| Destructuring assign (`let (a, b) = ...`) | Non parsato |
| `async`/`await` reale | Stub sincrono |
| `mod`/`use` reali | warn + no-op |
| `*args`, `**kwargs` variadici | Non nell'AST |
| String/Array methods su istanza | Solo via funzioni globali (es. `string.upper(s)`) |

**Benchmark target (v0.2.x completa):** VM 15× più veloce del tree-walker. Loop su 1M interi < 1ms.

---

## Phase 2 — VM Optimization & JIT Cranelift (v0.3.x)

**Goal:** Ottimizzazioni VM sistematiche, poi JIT-compile funzioni hot via Cranelift.

> Fase riorganizzata rispetto alla roadmap originale: inserita un'intera fase di
> ottimizzazione VM *prima* del JIT, come indicato dall'analisi v0.2.14.
> Il NaN-boxing (24→8 byte) è spostato qui come fase dedicata — unsafe + 750+ match arms
> da refactorare richiedono una versione intera.

| Versione | Obiettivo |
|----------|-----------|
| v0.3.0 | **VM hot path:** Op::Add/Sub/Mul fast-path Int+Int inline; `step_limit` `#[cfg(debug_assertions)]`; `upvalues: Rc<Vec<Upvalue>>` |
| v0.3.1 | **String interning:** tabella globale di deduplicazione; confronti O(1); allocazioni ridotte |
| v0.3.2 | **NaN-boxing (unsafe):** Value 24→8 byte; `u64` con tag nei bit NaN; refactor 750+ match arms |
| v0.3.3 | **Peephole avanzato:** eliminazione `Const+Pop`, `Jump offset=0`, `Not+Not` con remap completo offset di salto; dead code elimination |
| v0.3.4 | **Inline caching** per `GetField`/`CallMethod`: salva ultima posizione nel Dict; skip lookup su cache hit |
| v0.3.5 | **JIT Cranelift integration:** compila funzioni hot a native code; call-count threshold; fallback all'interprete |
| v0.3.6 | **JIT type specialization:** versioni specializzate per tipo concreto (Int path, Float path) |
| v0.3.7 | **Tail call optimization:** `return f(args)` riutilizza il frame; ricorsione profonda senza stack overflow |
| v0.3.8 | **SIMD auto-vectorization** per Array ops (via Cranelift SIMD) |
| v0.3.9 | **Concurrent GC v2:** concurrent mark phase, pause ridotte |
| v0.3.10 | Profiler v1 + Benchmark suite v2 + documentation v2 |

**Benchmark target (v0.3.10):** Loop numerici entro 3× di Rust equivalente. Array ops entro 2× di NumPy.

---

## Phase 3 — Parallelism & Concurrency (v0.4.x)

**Goal:** Multi-core automatico, async/await reale, SIMD a piena larghezza.

| Versione | Obiettivo |
|----------|-----------|
| v0.4.0 | Work-stealing task scheduler (Tokio/Rayon hybrid) |
| v0.4.1 | Keyword `spawn` per task leggeri (ora stub sincrono) |
| v0.4.2 | `async`/`await` per I/O concurrency (ora stub) |
| v0.4.3 | Parallel array ops (`pmap`, `preduce`, loop auto-paralleli) |
| v0.4.4 | SIMD a piena larghezza AVX2/AVX-512 via runtime feature detection |
| v0.4.5 | Concurrent GC v3: collection completamente concorrente con i task |
| v0.4.6 | Message passing via canali (ispirato a Go) |
| v0.4.7 | Collezioni stdlib lock-free |
| v0.4.8 | Race condition detection in debug mode |
| v0.4.9 | Profiler v2: concurrency profiling, thread utilization |
| v0.4.10 | Benchmark suite v3 + documentation v3 |

**Benchmark target (v0.4.10):** Parallel sum scala linearmente su 8 core. Nessun bottleneck GIL.

---

## Phase 4 — LLVM Backend & GPU (v0.5.x)

**Goal:** LLVM per ottimizzazione critical path. GPU via wgpu. Output eseguibile.

| Versione | Obiettivo |
|----------|-----------|
| v0.5.0 | LLVM IR generation per funzioni type-specializzate |
| v0.5.1 | LLVM O2/O3 pipeline per hot paths |
| v0.5.2 | Profile-guided optimization (PGO) dal profiler runtime |
| v0.5.3 | Integrazione `wgpu`: GPU array backend |
| v0.5.4 | GPU dispatch automatico per array ops grandi |
| v0.5.5 | GPU kernel specialization via compute shaders |
| v0.5.6 | Executable output: `neba build` emette binary statico |
| v0.5.7 | Cross-compilation support (x86_64, aarch64, WASM) |
| v0.5.8 | Link-time optimization (LTO) per `neba build` |
| v0.5.9 | Profiler v3: GPU profiling |
| v0.5.10 | Benchmark suite v4 + documentation v4 |

**Benchmark target (v0.5.10):** GPU array ops entro 2× di CUDA/Metal. Binari statici funzionanti.

---

## Phase 5 — Interoperability (v0.6.x)

**Goal:** FFI C/C++/Rust, interop Python.

| Versione | Obiettivo |
|----------|-----------|
| v0.6.0 | C FFI: dichiarazioni `extern "C"`, chiamare C da Neba |
| v0.6.1 | Struct C-ABI compatibili, passaggio puntatori |
| v0.6.2 | C++ interop via bridge `extern "C"` |
| v0.6.3 | Rust FFI: chiamare funzioni `#[no_mangle]` da Neba |
| v0.6.4 | Funzioni Neba chiamabili da C/Rust |
| v0.6.5 | Python interop v1: chiamare funzioni Python |
| v0.6.6 | Python interop v2: import moduli Python (`import py.numpy as np`) |
| v0.6.7 | Python interop v3: array Neba → NumPy senza copia (buffer protocol) |
| v0.6.8 | Neba callable da Python come extension module |
| v0.6.9 | FFI documentation + safety guide |
| v0.6.10 | Benchmark suite v5 + documentation v5 |

---

## Phase 6 — Ecosystem & Tooling (v0.7.x)

**Goal:** Package manager, formatter, linter, test runner, LSP.

| Versione | Obiettivo |
|----------|-----------|
| v0.7.0 | Formato manifest `neba.toml` |
| v0.7.1 | Package manager `neba pkg`: install, remove, update |
| v0.7.2 | Registry protocol (HTTP-based, come crates.io) |
| v0.7.3 | Formatter `neba fmt`: stile canonico |
| v0.7.4 | Linter `neba lint`: stile, correttezza, performance hints |
| v0.7.5 | Test runner `neba test`: framework di test built-in |
| v0.7.6 | Doc generator `neba doc` |
| v0.7.7 | LSP server per IDE |
| v0.7.8 | REPL avanzato: syntax highlighting, autocomplete |
| v0.7.9 | Integrazione profiler con flamegraph export |
| v0.7.10 | Benchmark suite v6 + documentation v6 |

---

## Phase 7 — Standard Library Expansion (v0.8.x)

**Goal:** Stdlib completa per tutti i domini comuni.

> Molte funzioni sono già implementate in v0.2.x. Questa fase completa i moduli
> mancanti identificati nell'analisi v0.2.14.

| Modulo | Contenuto | Stato |
|--------|-----------|-------|
| `math` | Trig, exp, iperboliche, statistiche, random | ✅ parziale (v0.2.11/v0.2.18) |
| `string` | Padding, parsing, predicati, case, slice | ✅ parziale (v0.2.11/v0.2.18) |
| `random` | LCG PRNG, seed, randint, choice, shuffle, sample | ✅ (v0.2.18) |
| `io` | File I/O, listdir, cwd, mkdir, io.path | ✅ parziale (v0.2.11/v0.2.18) |
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

**Goal:** Stabilità, performance hardening, specifica formale, production-ready.

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

- **GC:** Attualmente Rc + RefCell (reference counting). Target: concurrent generational GC in v0.3.9
- **JIT:** Cranelift in v0.3.5, LLVM in v0.5.x. Prerequisito obbligatorio: VM optimization v0.3.0–v0.3.4
- **Value size:** 24 byte (post v0.2.19). Target NaN-boxing 8 byte: v0.3.2 (unsafe, fase dedicata — richiede refactor 750+ match arms)
- **Dict:** IndexMap O(1) con ordine di inserimento preservato (post v0.2.17)
- **Constant folding:** tutti gli operatori su literal risolti a compile-time (post v0.2.20)
- **Peephole con eliminazione istruzioni:** rimandato a v0.3.3 — richiede remap completo degli offset di salto
- **Parallelismo:** Work-stealing scheduler, nessun GIL (v0.4.x)
- **GPU:** wgpu compute shaders (v0.5.x)
- **Interop:** C, Rust, Python (v0.6.x)
- **Indexing:** 0-based in tutto il linguaggio
