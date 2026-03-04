# Neba Benchmark Results — v2 (Gate v0.2.21)

Data: 2026-03-04
Versione: v0.2.21 — fast-path Int ops + FxHashMap globals + constant folding
Hardware: Linux x86_64

Per eseguire:
```bash
cargo run --release --bin neba -- benchmarks/bench_v2.neba
```

> **Nota metodologica:** i benchmark v2 usano variabili *locali* (dentro funzioni),
> coerentemente con il codice reale. Le variabili globali a livello script sono ~2×
> più lente per il lookup HashMap — comportamento normale e documentato.

---

## Sezione A — Benchmark classici (evoluzione storica)

| # | Benchmark | v0 tree-walk | v0.2.14 globali | **v0.2.21 locali** | Target gate v2 | Stato |
|---|-----------|-------------|-----------------|---------------------|----------------|-------|
| A1 | Int arith 10M | 67,673 ms | 5,126 ms | **2,828 ms** | < 3,000 ms | ✅ |
| A2 | Float arith 5M | 24,204 ms | 1,671 ms | **932 ms** | — | ✅ |
| A3 | For-range 1M | 2,074 ms | 104 ms | **72 ms** | < 80 ms | ✅ |
| A4 | fib(30) | 8,514 ms | 431 ms | **698 ms** | < 350 ms | ❌ |
| A5 | Closure 500k | 2,700 ms | 152 ms | **198 ms** | < 120 ms | ❌ |
| A6 | Array push/pop 100k | 905 ms | 70 ms | **45 ms** | — | ✅ |
| A7 | String concat 10k | 45 ms | 5.2 ms | **3.8 ms** | — | ✅ |
| A8 | HOF 1M | 8,678 ms | 525 ms | **468 ms** | < 400 ms | ❌ |
| A9 | Class ctor+method 100k | 3,108 ms | 214 ms | **213 ms** | — | ✅ |

---

## Sezione B — TypedArray

| # | Benchmark | v0.2.14 | **v0.2.21** | Target | Stato |
|---|-----------|---------|-------------|--------|-------|
| B1 | ones(1M) + sum | 4.05 ms | **7.5 ms** | — | — |
| B2 | dot(100k) × 100 | 55 ms | **85.5 ms** | — | — |
| B3 | linspace(1M) + mean | 0.49 ms | **9.2 ms** | < 1.5 ms | ❌ |

> B3 regressione: `linspace()` è ora 9.2ms vs 0.49ms in v0.2.14.
> Causa: `fill()` interno chiama iterazione diversa dopo refactoring stdlib v0.2.18.
> Da investigare in v0.2.23.

---

## Sezione C — Stdlib math (variabili locali)

| # | Benchmark | v0.2.14 | **v0.2.21** | Δ |
|---|-----------|---------|-------------|---|
| C1 | sin+cos 500k | 511 ms | **474 ms** | +7% |
| C2 | sqrt+log 1M | 912 ms | **795 ms** | +13% |

---

## Sezione D — Dict O(1)

| # | Benchmark | **v0.2.21** |
|---|-----------|-------------|
| D1 | insert 100k int keys + lookup | **41.8 ms** |
| D2 | 50k string keys + has_key | **76.4 ms** |

---

## Sezione E — Constant folding

| # | Benchmark | **v0.2.21** | Note |
|---|-----------|-------------|------|
| E1 | 1M iters con costante foldato | **184 ms** | CF_A=26 foldato ✅ |

---

## Sezione F — HOF

| # | Benchmark | **v0.2.21** |
|---|-----------|-------------|
| F1 | map double 100k | **16 ms** |
| F2 | filter evens 100k | **16.7 ms** |
| F3 | reduce sum 100k | **14.9 ms** |

---

## Sezione G — String stdlib

| # | Benchmark | **v0.2.21** |
|---|-----------|-------------|
| G1 | split+upper 50k | **69 ms** |

---

## Analisi gate v0.2.21

### Superati ✅
- A1 int arithmetic: **2,828ms** (target < 3,000ms) — fast-path Add/Mul/IntDiv/Mod attivo
- A2 float arithmetic: **932ms** — buono
- A3 for-range 1M: **72ms** (target < 80ms) — ✅
- A6 array ops: **45ms** — eccellente
- A7 string concat: **3.8ms** — eccellente
- A9 class: **213ms** — stabile

### Non superati ❌
- A4 fib(30): **698ms** (target < 350ms) — chiamate ricorsive costose, nessun fast-path call
- A5 closure 500k: **198ms** (target < 120ms) — overhead upvalue/Rc
- A8 HOF 1M: **468ms** (target < 400ms) — vicino, ma call overhead
- B3 linspace+mean: **9.2ms** (target < 1.5ms) — regressione stdlib da investigare

### Decisione: ❌ NO-GO per JIT — gate v3 in v0.2.25

I 3 miss principali (A4/A5/A8) dipendono tutti dall'overhead di *chiamata funzione*
(frame push, locals setup, return). Il fix corretto è:
1. **v0.2.22** — Bug fix OOP già pianificata
2. **v0.2.23** — Ottimizzare il call overhead: frame leggero, inline threshold piccole fn
3. **v0.2.24** — Investigare B3 linspace, tail call, Rc<Vec> upvalues
4. **v0.2.25** — Gate v3 (nuovi target)

---

## Target gate v3 (v0.2.25)

| Benchmark | v0.2.21 | Target v3 |
|-----------|---------|-----------|
| A4 fib(30) | 698 ms | < 300 ms |
| A5 closure 500k | 198 ms | < 100 ms |
| A8 HOF 1M | 468 ms | < 300 ms |
| B3 linspace+mean 1M | 9.2 ms | < 2 ms |
