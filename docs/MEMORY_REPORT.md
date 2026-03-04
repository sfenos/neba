# Neba v0.2.26 — Memory Usage Report
**Data:** 2026-03-04

## Benchmark memoria

| Workload | Neba RSS | Python RSS | Note |
|----------|----------|------------|------|
| Array 100k Int + Dict 50k + NdArray 500×500 | **6 MB** | **49 MB** | Neba 8× più efficiente |
| OOP: 100k istanze create/distrutte | **5.6 MB** | — | No memory leak rilevato |
| Baseline (programma vuoto) | ~2.5 MB | ~12 MB | |

## Strutture dati e layout

| Tipo | Dimensione | Note |
|------|-----------|------|
| `Value` | **24 byte** | post v0.2.19 (era 40 byte) |
| `Value::Int(i64)` | 24B | tag + i64 |
| `Value::Float(f64)` | 24B | tag + f64 |
| `Value::Str(Rc<String>)` | 24B | puntatore Rc |
| `Value::Array(Rc<RefCell<Vec<Value>>>)` | 24B | puntatore Rc |
| `Value::NdArray(Rc<RefCell<NdArray>>)` | 24B | puntatore Rc |
| `NdArray` 100×100 Float64 | 80 kB | puro dato (100×100×8B) |
| `CallFrame` | ~128B | ip, base, upvalues Rc |

## Gestione memoria

- **Reference counting** (Rc + RefCell): GC deterministico, nessuna pausa
- **Upvalues**: `Rc<Vec<Upvalue>>` condiviso tra clone della stessa closure (v0.2.23)
- **Dict**: `IndexMap<Value,Value>` — ordine inserimento preservato
- **IntRange**: lazy (0 alloc) — `for i in 0..1_000_000` usa 0 memoria aggiuntiva

## Limitazioni note

- **Cicli Rc**: Oggetti che si referenziano mutuamente non vengono liberati.
  Attualmente non è possibile creare cicli in Neba (nessuna self-reference su instance field).
  Target: concurrent generational GC in v0.3.9
- **String dedup**: stringhe identiche non condividono memoria.
  String interning sperimentato in v0.2.26 ma rimosso per regressione su benchmark.
  Da riprogettare come fase dedicata.
- **Stack**: fisso a 4096 slot × 24B = ~96kB per VM

## Target post-GC (v0.3.9)
- NaN-boxing (v0.3.2): `Value` 24→8 byte — 3× meno memoria per array/stack
- Concurrent GC: gestione cicli, pause ridotte
