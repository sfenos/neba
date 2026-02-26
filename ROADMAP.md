# Neba — Roadmap

Ultimo aggiornamento: 2026-02-26

---

## Stato attuale: v0.1.9 + v0.2.3 ✅

---

## Milestone completate

| Versione | Obiettivo | Data |
|----------|-----------|------|
| v0.1.0 | Lexer: tokenizer completo, indentazione significativa, f-string, numeri | 2026-02-24 |
| v0.1.1 | Parser + AST: ricorsivo discendente, Pratt, if/match come espressioni | 2026-02-25 |
| v0.1.2 | Tree-walking interpreter: variabili, funzioni, classi, pattern matching, stdlib base | 2026-02-25 |
| v0.1.3 | Tree-walking interpreter: aritmetica interi e float | 2026-02-26 |
| v0.1.4 | REPL: loop interattivo, comando neba | 2026-02-26 |
| v0.1.5 | Stringhe base, print statement | 2026-02-26 |
| v0.1.6 | If/else, operatori di confronto | 2026-02-26 |
| v0.1.7 | While loop, for-range loop (for i in 0..10) | 2026-02-26 |
| v0.1.8 | Funzioni: definizione, chiamata, ricorsione, closures | 2026-02-26 |
| v0.1.9 | Modulo system base: stdlib abs/min/max/push/pop | 2026-02-26 |
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

## Roadmap v0.2.x

| Versione | Obiettivo |
|----------|-----------|
| v0.2.4 | Traits: definition, implementation, dispatch |
| v0.2.5 | Lists and Dicts (dynamic, GC-managed) |
| v0.2.6 | Native typed Array[T]: Float64, Int64, Int32, Float32 |
| v0.2.7 | Array operations: indexing (0-based), slicing, basic arithmetic |
| v0.2.8 | Error handling: Result[T,E], match, ? operator |
| v0.2.9 | Standard library v0: math, io, string, collections |
| v0.2.10 | Benchmark suite v1 + documentation v1 |

---

## Roadmap v0.1.x

| Versione | Obiettivo |
|----------|-----------|
| v0.1.10 | Benchmark suite v0 + documentation v0 |

---

## Roadmap futura

| Versione | Obiettivo |
|----------|-----------|
| v0.3.0 | Cranelift JIT + specializzazione |
| v0.3.1 | Scheduler parallelo + async/await reale |
| v0.4.0 | LLVM backend + output eseguibile |
| v0.5.0 | Interop C/Rust/Python |
| v0.6.0 | Package manager + formatter + LSP |
| v0.7.0 | Standard library completa |
| v1.0.0 | Stable release |

---

## Note architetturali

- **Interprete:** tree-walking (crate `neba_interpreter`) — usato per prototipazione rapida
- **VM:** bytecode stack machine (crate `neba_vm`) — pipeline principale di esecuzione
- **Type checker:** inferenza di tipi (crate `neba_typecheck`) — integrazione in corso
- **GC attuale:** reference counting via `Rc<RefCell<_>>` — GC generazionale pianificato in v0.3.x
- **Parallelismo:** `spawn`/`await` stub sincroni — concorrenza reale in v0.3.1
- **JIT:** pianificato in v0.3.0 con Cranelift, poi LLVM in v0.4.0
