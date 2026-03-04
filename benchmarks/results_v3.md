# Neba — Benchmark Results v3 (post v0.2.23)
**Data:** 2026-03-04

## Confronto v0.2.20 → v0.2.23

| Benchmark | v0.2.20 | v0.2.23 | Δ |
|-----------|---------|---------|---|
| A1 int loop 10M | 2642 ms | 2324 ms | **-12%** |
| A2 float loop 5M | 959 ms | 826 ms | **-14%** |
| A3 for-range 1M | 79 ms | 57 ms | **-28%** |
| A4 fib(35) | 6437 ms | 5686 ms | **-12%** |
| A5 closure 500k | 189 ms | 167 ms | **-12%** |
| A8 HOF 1M | 436 ms | 376 ms | **-14%** |
| A9 class 100k | 204 ms | 190 ms | **-7%** |
| B2 dot(100k)×100 | 83 ms | 37 ms | **-55%** |
| C1 sin+cos 500k | 435 ms | 383 ms | **-12%** |
| D2 dict str 50k | 72 ms | 60 ms | **-17%** |
| F1 map 100k | 23 ms | 14 ms | **-40%** |
| F2 filter 100k | 17 ms | 14 ms | **-18%** |

**Media miglioramento sui benchmark principali: ~15%**  
Guadagno maggiore su for-range (-28%) e HOF/map (-40%) grazie ai fast-path dei confronti.
