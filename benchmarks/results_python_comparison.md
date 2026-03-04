# Neba vs Python 3.12 — Benchmark Comparison
**Data:** 2026-03-04  
**Neba version:** v0.2.26  
**Python version:** 3.12.3  
**Hardware:** Linux (container), single-core  

---

## Risultati

| Benchmark | Python 3.12 | Neba v0.2.26 | Ratio |
|-----------|-------------|--------------|-------|
| B1 fib(35) — ricorsione pura | 724 ms | 5770 ms | 8.0× slower |
| B2 int_loop(10M) — loop int | 328 ms | 622 ms | **1.9× slower** |
| B3 float_loop(5M) — loop float | 173 ms | 519 ms | 3.0× slower |
| B4 range_loop(1M) — loop semplice | 25 ms | 58 ms | 2.3× slower |
| B5 closure(500k) — closure counter | 25 ms | 82 ms | 3.3× slower |
| B6 dict(100k) — dict ops | 58 ms | 87 ms | **1.5× slower** |
| B7 oop(100k) — object creation+method | 25 ms | 181 ms | 7.2× slower |
| B10 fact(20)×100k — ricorsione media | 100 ms | 416 ms | 4.2× slower |

**Media geometrica: ~3.3× più lento di CPython 3.12**

---

## Analisi

### Punti di forza
- **Dict ops (1.5×)** — IndexMap O(1) + zero-alloc lookup competitivo
- **Int loop (1.9×)** — fast-path Int+Int inline, buona densità opcode

### Bottleneck principali
- **fib(35) (8×)** — nessuna tail-call optimization, overhead frame altissimo per ricorsione profonda
- **OOP (7.2×)** — istanziazione ripetuta: `__init__` + HashMap fields + alloc per ogni oggetto
- **fact×100k (4.2×)** — idem fib: overhead frame call domina

### Contesto
- **CPython 3.12** ha la Specializing Adaptive Interpreter (PEP 659) — si adatta durante l'esecuzione
- **PyPy** sarebbe ~5-10× più veloce di CPython su questi benchmark
- **Neba post-JIT** (target v0.3.5): atteso 3-10× speedup su hot paths → obiettivo **sotto 1× CPython su loop**

---

## Ottimizzazioni implementate fino a v0.2.26
- LoadLocal0-3 / StoreLocal0-3 specializzati (0 operand read)
- Fast-path Int+Int per tutti gli operatori aritmetici e confronto
- Peephole: `Const+Pop`, `True/False/Nil+Pop`, `Not+Not` → Nop
- `step_limit` sotto `#[cfg(debug_assertions)]` (zero overhead in release)
- `upvalues: Rc<Vec<Upvalue>>` (clone O(1))
- `Chunk::add_const` O(1) con HashMap
- Constant folding a compile-time

---

## Roadmap ottimizzazioni future (pre-JIT)
| Ottimizzazione | Impatto atteso | Target |
|----------------|----------------|--------|
| String interning | -15% allocazioni | v0.3.1 |
| NaN-boxing Value 24→8 byte | -20% cache miss | v0.3.2 |
| Inline caching GetField/CallMethod | -30% OOP overhead | v0.3.4 |
| **JIT Cranelift** | **3-10× speedup** | v0.3.5 |
