# Neba — Roadmap

Ultimo aggiornamento: 2026-02-26

---

## Stato attuale: v0.2.3 + v0.1.10 ✅

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
| v0.1.7 | While loop, for-range loop (for i in 0..10) | 2026-02-26 |
| v0.1.8 | Funzioni: definizione, chiamata, ricorsione, closures | 2026-02-26 |
| v0.1.9 | Modulo system base: stdlib abs/min/max/push/pop | 2026-02-26 |
| v0.1.10 | Benchmark suite v0 + documentation v0 (LANGUAGE.md) | 2026-02-26 |
| v0.2.0 | Bytecode VM: compiler AST→bytecode, stack machine, opcode set completo | 2026-02-25 |
| v0.2.1 | Type checker + inferenza di tipi | 2026-02-25 |
| v0.2.2 | Classi + trait dispatch (interpreter) | 2026-02-25 |
| v0.2.3 | Classes VM: definizione, istanziazione, metodi, self, __init__ | 2026-02-26 |

---

## In corso

| Versione | Obiettivo |
|----------|-----------|
| v0.2.4 | Traits: definizione, impl, dispatch nella VM |

---

## Phase 0 — Foundation (v0.1.x) ✅ COMPLETATA

| Versione | Obiettivo |
|----------|-----------|
| v0.1.0 ✅ | Rust workspace setup, CI, project layout |
| v0.1.1 ✅ | Lexer: tokenizer completo |
| v0.1.2 ✅ | Parser + AST |
| v0.1.3 ✅ | Tree-walking interpreter: aritmetica |
| v0.1.4 ✅ | REPL interattivo |
| v0.1.5 ✅ | Stringhe base, print |
| v0.1.6 ✅ | If/else, confronti |
| v0.1.7 ✅ | While e for-range loop |
| v0.1.8 ✅ | Funzioni, ricorsione, closures |
| v0.1.9 ✅ | Modulo system base |
| v0.1.10 ✅ | Benchmark suite v0 + documentation v0 |

**Target raggiunto:** fib(30) = 8,514 ms. REPL avviato in <200ms.

---

## Phase 1 — Bytecode VM & Core Types (v0.2.x)

**Goal:** VM bytecode completa, GC, classi, traits, array tipizzati, error handling, stdlib.

| Versione | Obiettivo |
|----------|-----------|
| v0.2.0 ✅ | Bytecode instruction set + compiler AST→bytecode |
| v0.2.1 ✅ | Stack-based VM bytecode (+ type checker) |
| v0.2.2 ✅ | Classi + trait dispatch (interpreter) |
| v0.2.3 ✅ | Classes VM: definizione, istanziazione, metodi, self, __init__ |
| v0.2.4 🔜 | Traits: definition, implementation, dispatch |
| v0.2.5 | Lists and Dicts (dynamic, GC-managed) |
| v0.2.6 | Native typed Array[T]: Float64, Int64, Int32, Float32 |
| v0.2.7 | Array operations: indexing (0-based), slicing, basic arithmetic |
| v0.2.8 | Error handling: Result[T,E], match, ? operator |
| v0.2.9 | Standard library v0: math, io, string, collections |
| v0.2.10 | Benchmark suite v1 + documentation v1 |

**Benchmark target (v0.2.10):** 5× speedup su fibonacci rispetto v0.1. Array sum di 1M floats in <5ms.

---

## Phase 2 — Cranelift JIT & Type Inference (v0.3.x)

**Goal:** JIT-compile le funzioni hot. Type inference completa. Auto-specializzazione.

| Versione | Obiettivo |
|----------|-----------|
| v0.3.0 | HIR: high-level IR con variabili di tipo |
| v0.3.1 | Hindley-Milner type inference engine |
| v0.3.2 | Bytecode annotato con tipi per JIT hints |
| v0.3.3 | Cranelift integration: compila funzioni tipizzate a native code |
| v0.3.4 | Call-count threshold: funzioni hot auto-promosse a JIT |
| v0.3.5 | Specializzazione: JIT genera versioni specializzate per tipo concreto |
| v0.3.6 | Concurrent GC v2: concurrent mark phase, pause più brevi |
| v0.3.7 | SIMD auto-vectorization per Array ops (via Cranelift SIMD) |
| v0.3.8 | Inlining di funzioni piccole nel JIT |
| v0.3.9 | Profiler v1: flamegraph, rilevamento funzioni hot |
| v0.3.10 | Benchmark suite v2 + documentation v2 |

**Benchmark target (v0.3.10):** Loop numerici entro 3× di Rust equivalente. Array ops entro 2× di NumPy.

---

## Phase 3 — Parallelism & Concurrency (v0.4.x)

**Goal:** Multi-core automatico, async/await, SIMD a piena larghezza.

| Versione | Obiettivo |
|----------|-----------|
| v0.4.0 | Work-stealing task scheduler (Tokio/Rayon hybrid) |
| v0.4.1 | Keyword `spawn` per task leggeri |
| v0.4.2 | `async`/`await` per I/O concurrency |
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
| v0.5.8 | Link-time optimization (LTO) per neba build |
| v0.5.9 | Profiler v3: GPU profiling |
| v0.5.10 | Benchmark suite v4 + documentation v4 |

**Benchmark target (v0.5.10):** GPU array ops entro 2× di CUDA/Metal. Build produce binari statici funzionanti.

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

**Goal:** Stdlib ricca per tutti i domini comuni.

| Modulo | Contenuto |
|--------|-----------|
| `std.math` | Trig, exp, statistiche, operazioni matriciali |
| `std.io` | File I/O, buffered reader/writer, directory |
| `std.net` | HTTP client/server, TCP/UDP, WebSocket |
| `std.json` | Parsing/serializzazione JSON |
| `std.csv` | CSV reader/writer |
| `std.time` | Date, time, duration, timezone |
| `std.random` | PRNG, crypto-random |
| `std.regex` | Espressioni regolari |
| `std.os` | Process, environment, signals |
| `std.crypto` | Hashing, AES, RSA |
| `std.data` | DataFrame-like, Series |
| `std.plot` | Plotting base (terminal + SVG) |

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

- **GC:** Concurrent generational mark-and-sweep (Go/Julia-style) — attualmente Rc (stub)
- **JIT:** Cranelift (v0.3.x) poi LLVM (v0.5.x)
- **Parallelismo:** Work-stealing scheduler, nessun GIL (v0.4.x)
- **GPU:** wgpu compute shaders (v0.5.x)
- **Interop:** C, Rust, Python (v0.6.x)
- **Indexing:** 0-based in tutto il linguaggio
