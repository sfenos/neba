# Neba — Design Decisions

| Topic | Decision |
|-------|----------|
| Indentation | 4 spazi (tab = errore) |
| Naming | snake_case / PascalCase / UPPER_SNAKE |
| Parallelismo | keyword `spawn expr` |
| Null safety | Option[T] con Some/None |
| String interpolation | f"text {expr}" |
| Memory | GC concorrente (pianificato) |
| Mutabilità | let (immutabile) / var (mutabile) |
