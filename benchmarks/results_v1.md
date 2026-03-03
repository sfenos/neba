# Neba Benchmark Results — v1

Data: —  
Versione: v0.2.13 — Bytecode VM + Stdlib + HOF  
Hardware: —

Per eseguire:
```bash
cargo run --bin neba -- benchmarks/bench_v1.neba
```

---

## Sezione A — Benchmark classici (confronto con v0)

| # | Benchmark | Ops | v0 (ms) | v1 (ms) | Δ |
|---|-----------|-----|---------|---------|---|
| A1 | Integer arithmetic (while) | 10M ops | 67,673 |81,383 | — |
| A2 | Float arithmetic (while) | 5M ops | 24,204 | 24,768 | — |
| A3 | For-range loop (sum) | 1M iters | 2,074 | 2,160 | — |
| A4 | Recursion — fib(30) | — | 8,514 | 8,852 | — |
| A5 | Closure calls | 500k calls | 2,700 | 2,876 | — |
| A6 | Array push/pop | 100k ops | 905 | 934 | — |
| A7 | String concatenation | 10k concat | 45 | 46 | — |
| A8 | Higher-order fn (manuale) | 1M calls | 8,678 | 8,839 | — |
| A9 | Class instantiation + method | 100k | 3,108 | 3,210 | — |

---

## Sezione B — TypedArray

| # | Benchmark | Ops | Tempo (ms) |
|---|-----------|-----|------------|
| B1 | TypedArray Float64 sum | 1M elem | 10.2 |
| B2 | TypedArray dot product ×100 | 100k elem | 810.3 |
| B3 | linspace(1M) + mean | 1M elem | 63.9 |

---

## Sezione C — Stdlib math

| # | Benchmark | Ops | Tempo (ms) |
|---|-----------|-----|------------|
| C1 | math.sin + math.cos | 500k | 5,813 |
| C2 | math.sqrt + math.log | 1M | 10,369 |
| C3 | math.floor + math.ceil | 2M | 12,258 |

---

## Sezione D — Stdlib string

| # | Benchmark | Ops | Tempo (ms) |
|---|-----------|-----|------------|
| D1 | string.split + join | 50k | 668 |
| D2 | string.replace + upper | 100k | 765 |
| D3 | string.find | 200k×2 | 1586 |

---

## Sezione E — HOF: map, filter, reduce

| # | Benchmark | Ops | Tempo (ms) |
|---|-----------|-----|------------|
| E1 | map double | 100k elem | 150 |
| E2 | filter evens | 100k elem | 196 |
| E3 | reduce sum | 100k elem | 147 |
| E4 | map+filter+reduce pipeline | 100k elem | 291 |
| E5a | Loop manuale ×100 | 10k×100 | 5,233 |
| E5b | HOF pipeline ×100 | 10k×100 | 3,401 |

---

## Sezione F — Collections

| # | Benchmark | Ops | Tempo (ms) |
|---|-----------|-----|------------|
| F1 | collections.sorted ×10 | 10k×10 | 14.1 |
| F2 | collections.zip | 50k pairs | 8.1 |
| F2b | collections.enumerate | 50k | 7.74 |
| F3 | collections.flatten | 10k×10 | 3.5 |

---

## Note

- I benchmark A1-A9 sono identici alla suite v0 — il confronto misura il miglioramento della VM bytecode rispetto all'interprete tree-walking baseline (v0.1.x).
- I TypedArray (sezione B) usano storage compatto `Vec<f64>`/`Vec<i64>` — la sum di 1M elementi dovrebbe essere < 5ms.
- Le HOF (sezione E) hanno overhead per il mini run-loop `call_value_sync` — E5 confronta direttamente HOF vs loop manuale per quantificare il costo.
- Target per v0.3.x (Cranelift JIT): A1 < 500ms, A4 < 100ms, E1-E4 < 50ms.

---

## Confronto storico

| Benchmark | v0 (tree-walk) | v1 (bytecode) | v1 speedup |
|-----------|---------------|---------------|------------|
| A1 int arith 10M | 67,673 ms | — | — |
| A4 fib(30) | 8,514 ms | — | — |
| A8 HOF 1M | 8,678 ms | — | — |
