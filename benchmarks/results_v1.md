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

| # | Benchmark | Ops | v0 (ms) | v1 (ms) | v1_02.14 (ms) | Δ |
|---|-----------|-----|---------|---------|---------------|---|
| A1 | Integer arithmetic (while) | 10M ops | 67,673 |81,383 | 5126 | 15.8x |
| A2 | Float arithmetic (while) | 5M ops | 24,204 | 24,768 | 1671 | 14.8x |
| A3 | For-range loop (sum) | 1M iters | 2,074 | 2,160 | 104 | 20.7x|
| A4 | Recursion — fib(30) | — | 8,514 | 8,852 | 431 | 20.5x |
| A5 | Closure calls | 500k calls | 2,700 | 2,876 | 152 | 18.9x |
| A6 | Array push/pop | 100k ops | 905 | 934 | 70 | 13.3x |
| A7 | String concatenation | 10k concat | 45 | 46 | 5.2 | 8.8x |
| A8 | Higher-order fn (manuale) | 1M calls | 8,678 | 8,839 | 525 | 16.8x|
| A9 | Class instantiation + method | 100k | 3,108 | 3,210 | 214 | 15x |

---

## Sezione B — TypedArray

| # | Benchmark | Ops | Tempo (ms) | 0.2.14 (ms) | Δ |
|---|-----------|-----|------------|-------------|---|
| B1 | TypedArray Float64 sum | 1M elem | 10.2 | 4.05 | 2.5 |
| B2 | TypedArray dot product ×100 | 100k elem | 810.3 | 55 | 14.7 |
| B3 | linspace(1M) + mean | 1M elem | 63.9 | 0.49 | 130 |

---

## Sezione C — Stdlib math

| # | Benchmark | Ops | Tempo (ms) | 0.2.14 (ms) | Δ |
|---|-----------|-----|------------|-------------|---|
| C1 | math.sin + math.cos | 500k | 5,813 | 511 |  11.3 |
| C2 | math.sqrt + math.log | 1M | 10,369 | 912 |  11.3 |
| C3 | math.floor + math.ceil | 2M | 12,258 | 1030 | 11.9 |

---

## Sezione D — Stdlib string

| # | Benchmark | Ops | Tempo (ms) | 0.2.14 (ms) | Δ |
|---|-----------|-----|------------|-------------|---|
| D1 | string.split + join | 50k | 668 | 73 | 9.1 |
| D2 | string.replace + upper | 100k | 765 | 71 | 10.7 |
| D3 | string.find | 200k×2 | 1586 | 143 | 11.1 |

---

## Sezione E — HOF: map, filter, reduce

| # | Benchmark | Ops | Tempo (ms) | 0.2.14 (ms) | Δ |
|---|-----------|-----|------------|-------------|---|
| E1 | map double | 100k elem | 150 | 14.9 | 10.8 |
| E2 | filter evens | 100k elem | 196 | 20.7 | 9.4 |
| E3 | reduce sum | 100k elem | 147 | 14.4 | 10.2 |
| E4 | map+filter+reduce pipeline | 100k elem | 291 | 27.4 | 10.6 |
| E5a | Loop manuale ×100 | 10k×100 | 5,233 | 297 | 17.6 |
| E5b | HOF pipeline ×100 | 10k×100 | 3,401 | 233 | 14.5 |

---

## Sezione F — Collections

| # | Benchmark | Ops | Tempo (ms) | 0.2.14 (ms) | Δ |
|---|-----------|-----|------------|-------------|---|
| F1 | collections.sorted ×10 | 10k×10 | 14.1 | 2.21| 6.4 |
| F2 | collections.zip | 50k pairs | 8.1 | 5.3 | 1.5 |
| F2b | collections.enumerate | 50k | 7.74 | 5.4 | 1.4 |
| F3 | collections.flatten | 10k×10 | 3.5 | 3.1 | 1.12 |

---

## Note

- I benchmark A1-A9 sono identici alla suite v0 — il confronto misura il miglioramento della VM bytecode rispetto all'interprete tree-walking baseline (v0.1.x).
- I TypedArray (sezione B) usano storage compatto `Vec<f64>`/`Vec<i64>` — la sum di 1M elementi dovrebbe essere < 5ms.
- Le HOF (sezione E) hanno overhead per il mini run-loop `call_value_sync` — E5 confronta direttamente HOF vs loop manuale per quantificare il costo.
- Target per v0.3.x (Cranelift JIT): A1 < 500ms, A4 < 100ms, E1-E4 < 50ms.

---

## Confronto storico

| Benchmark | v0 (tree-walk) | v1 (bytecode) | v0.2.14 (opt) | v1 speedup
|-----------|---------------|---------------|------------|------------|
| A1 int arith 10M | 67,673 |81,383 | 5126 | 15.8x |
| A4 fib(30) | 8,514 | 8,852 | 431 | 20.5x |
| A8 HOF 1M | 8,678 | 8,839 | 525 | 16.8x|
