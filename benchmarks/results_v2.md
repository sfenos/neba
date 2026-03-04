# Neba — Benchmark Results v2
**Versione:** v0.2.20  
**Data:** 2026-03-04  
**Macchina:** Linux x86_64, cargo release build  

---

## Risultati

| ID | Benchmark | Tempo | Throughput |
|----|-----------|-------|------------|
| A1 | Integer arithmetic (10M ops) | 2642 ms | 3.8M ops/s |
| A2 | Float arithmetic (5M ops) | 959 ms | 5.2M ops/s |
| A3 | For-range loop (1M iters) | 79 ms | 12.6M iter/s |
| A4 | Ricorsione fib(35) | 6437 ms | — |
| A5 | Closure calls (500k) | 189 ms | 2.6M call/s |
| A6 | Array push/pop (100k) | 44 ms | 2.3M op/s |
| A7 | String concat (10k) | 3.3 ms | 3M concat/s |
| A8 | Higher-order fn (1M) | 436 ms | 2.3M call/s |
| A9 | Class ctor + method (100k) | 204 ms | 490k inst/s |
| B1 | TypedArray ones(1M) + sum | 6.3 ms | 158M elem/s |
| B2 | TypedArray dot(100k) × 100 | 83 ms | 120M elem/s |
| B3 | linspace(1M) + mean | 9 ms | 111M elem/s |
| C1 | math.sin + math.cos (500k) | 435 ms | 1.1M call/s |
| C2 | math.sqrt + math.log (1M) | 761 ms | 1.3M call/s |
| D1 | Dict insert 100k int keys | 32 ms | 3.1M op/s |
| D2 | Dict 50k string keys + has_key | 72 ms | 694k op/s |
| E1 | Constant fold loop (1M) | 172 ms | 5.8M iter/s |
| F1 | map double (100k) | 23 ms | 4.3M elem/s |
| F2 | filter evens (100k) | 17 ms | 5.9M elem/s |
| F3 | reduce sum (100k) | 15 ms | 6.5M elem/s |
| G1 | string.split + upper (50k) | 70 ms | 714k op/s |

---

## Analisi per categoria

### 🟢 Eccellenti (già veloci, JIT non urgente)
- **TypedArray** (B1–B3): 111–158M elem/s → già eseguono in Rust nativo, il JIT non aiuterebbe
- **HOF** (F1–F3): 4–6M elem/s → ragionevoli, overhead minimo
- **String concat** (A7): 3.3ms per 10k → buono

### 🟡 Accettabili (JIT li migliorerà 2–4×)
- **For-range** (A3): 79ms / 1M iters → 79ns/iter, decente per un interprete
- **Closure** (A5): 189ms / 500k → 378ns/call
- **Dict** (D1–D2): 32–72ms → O(1) ma overhead interpreter
- **HOF calls** (A8): 436ms / 1M → 436ns/call

### 🔴 Lenti (bottleneck principale, target JIT)
- **A1 integer arithmetic**: 2642ms / 10M → 264ns per iterazione (ciclo while con 6 operatori)
- **A4 fib(35)**: 6437ms → CPython ≈ 2800ms, siamo 2.3× più lenti di Python
- **A2 float arithmetic**: 959ms / 5M → 192ns/iter

### Confronto con Python (CPython 3.12 stimato)
| Benchmark | Neba v0.2.20 | CPython ~3.12 | Rapporto |
|-----------|-------------|---------------|----------|
| fib(35) | 6437 ms | ~2800 ms | 2.3× più lento |
| int loop 1M | ~264 ms | ~60 ms | 4.4× più lento |
| float loop 1M | ~192 ms | ~80 ms | 2.4× più lento |
| TypedArray sum 1M | 6.3 ms | ~1 ms (NumPy) | 6.3× più lento |

---

## Diagnosi bottleneck

Il ciclo di dispatch principale (`'dispatch: loop`) esegue per ogni opcode:
1. Fetch opcode dal bytecode (`code[ip]`)
2. Decode con `Op::from_u8`
3. Match arm → esecuzione
4. Incremento `ip`

Per `n = n + i * 2 - i // 3 + i % 7` in A1, ogni iterazione emette:
- **9 istruzioni** (LoadLocal×3, Const×2, Mul, IntDiv, Mod, Add, Sub, Add, StoreLocal) → ~264ns / 9 ≈ 29ns/opcode

29ns/opcode è il costo attuale. Target post-JIT: 1–3ns/opcode per hot loops.

**Guadagni attesi dalle ottimizzazioni v0.2.23 (pre-JIT):**
- Op::Add fast-path Int+Int: -15% su A1/A4
- `step_limit` fuori dal hot path (`#[cfg]`): -5% globale
- Upvalues `Rc<Vec>`: -10% su A5/A8

**Guadagni attesi da JIT Cranelift (v0.3.5):**
- A1 integer loops: 5–10× (target: ~300–500ms)
- A4 fib(35): 5–8× (target: ~800–1200ms)
- A8 HOF: 3–5× (dipende da inline)

---

## Verdetto benchmark gate

**GO per JIT Cranelift** ✅ — ma con prerequisiti:

### Pre-JIT obbligatori (v0.2.21–v0.2.24)
1. **v0.2.22**: fix bug OOP (`__str__`, `__init__` no-params, `min/max` TypedArray)  
2. **v0.2.23**: ottimizzazioni VM hot path (Add fast-path, step_limit cfg)  
3. **v0.2.24**: unificazione `call_value_sync` + Compiler registry Arc  

### Motivazione
I TypedArray sono già al livello target (Rust nativo). Il bottleneck reale è il dispatch
loop per codice interpretato puro — esattamente ciò che il JIT risolve. Procedere a
v0.3.x dopo aver fixato i bug OOP e ottimizzato l'hot path.

---

## Target per v0.3.10 (post-JIT)
| Benchmark | Attuale | Target |
|-----------|---------|--------|
| fib(35) | 6437 ms | < 1000 ms |
| int loop 10M | 2642 ms | < 400 ms |
| float loop 5M | 959 ms | < 200 ms |
| closure 500k | 189 ms | < 60 ms |
| TypedArray 1M sum | 6.3 ms | < 3 ms |
