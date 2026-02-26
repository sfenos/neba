# Neba Benchmark Results — Baseline v0

Data: 2026-02-26  
Versione: v0.1.10 (bytecode VM, no JIT)  
Hardware: da aggiornare con le specifiche della macchina

---

## Risultati

| # | Benchmark                        | Ops         | Tempo (ms)  |
|---|----------------------------------|-------------|-------------|
| 1 | Integer arithmetic (while)       | 10M ops     | 67,673 ms   |
| 2 | Float arithmetic (while)         | 5M ops      | 24,204 ms   |
| 3 | For-range loop (sum)             | 1M iters    | 2,074 ms    |
| 4 | Recursion — fib(30)              | —           | 8,514 ms    |
| 5 | Closure calls                    | 500k calls  | 2,700 ms    |
| 6 | Array push/pop                   | 100k ops    | 905 ms      |
| 7 | String concatenation             | 10k concat  | 45 ms       |
| 8 | Higher-order functions           | 1M calls    | 8,678 ms    |
| 9 | Class instantiation + method     | 100k        | 3,108 ms    |

---

## Note

- La VM è un **bytecode interpreter** puro, senza JIT né ottimizzazioni.
- I tempi di aritmetica (bench 1-2) sono alti a causa del dispatch per ogni opcode.
- Il for-range (bench 3) è ~32x più veloce del while equivalente grazie all'opcode `IterNext` ottimizzato.
- Le closure (bench 5) funzionano con upvalue reali — nessuna copia per snapshot.
- Le stringhe (bench 7) sono lente per la natura immutabile di `Rc<String>` (ogni concat alloca).

---

## Target futuri

| Versione | Backend         | Target atteso (bench 1) |
|----------|-----------------|--------------------------|
| v0.1.10  | Bytecode VM     | **67,673 ms** ← baseline |
| v0.3.0   | Cranelift JIT   | < 500 ms (target 100x)   |
| v0.4.0   | LLVM            | < 50 ms (target 1000x)   |
