# Neba — What's New

Riepilogo delle funzionalità disponibili in ogni versione, scritto per l'utente
finale (non per il maintainer). Per i dettagli tecnici vedi `CHANGELOG.md`.

---

## v0.1.2 — L'interprete funziona 🎉

**Data:** 2026-02-25

Neba ora è un linguaggio **eseguibile**. Puoi scrivere codice e vederlo girare.

### Cosa puoi fare oggi

```neba
# Variabili
let nome = "Neba"
var contatore = 0

# Funzioni con ricorsione
fn fattoriale(n: Int) -> Int
    if n <= 1
        return 1
    return n * fattoriale(n - 1)

println(fattoriale(10))   # → 3628800

# Classi
class Punto
    x: Float = 0.0
    y: Float = 0.0

    fn distanza(self) -> Float
        return (self.x ** 2 + self.y ** 2) ** 0.5

var p = Punto()
p.x = 3.0
p.y = 4.0
println(p.distanza())     # → 5.0

# Pattern matching
let valore: Option[Int] = Some(42)

match valore
    case Some(n) => println(f"Trovato: {n}")
    case None    => println("Niente")

# Array e cicli
let numeri = [5, 3, 8, 1, 9, 2]
var somma = 0
for n in numeri
    somma += n
println(f"Somma: {somma}")   # → 28
```

### Built-in disponibili

`print`, `println`, `input`, `len`, `str`, `int`, `float`, `bool`, `typeof`,
`abs`, `min`, `max`, `range`, `push`, `pop`, `assert`

### Come usare Neba

**Eseguire un file:**
```bash
cargo run --bin neba -- mio_programma.neba
```

**REPL interattivo:**
```bash
cargo run --bin neba_repl
```

Comandi REPL: `:help`, `:clear`, `:quit`

---

## v0.1.1 — Il parser capisce il codice

**Data:** 2026-02-25

Neba ora trasforma il codice sorgente in un AST (Abstract Syntax Tree)
strutturato, pronto per essere interpretato o compilato.

### Cosa è stato aggiunto
- Parser ricorsivo discendente con algoritmo Pratt per le espressioni
- `if`/`match` come **espressioni** (restituiscono un valore, come in Rust)
- Pattern matching con: wildcard `_`, letterali, binding di variabili,
  costruttori `Some(x)`, `Ok(v)`, `Err(e)`, range `0..=100`, or-pattern `a | b`
- Error recovery: il parser continua dopo un errore, raccoglie tutti i problemi

---

## v0.1.0 — Il lexer tokenizza il codice

**Data:** 2026-02-24

Primo passo: Neba legge correttamente il testo sorgente.

### Caratteristiche del lexer
- Indentazione significativa a **4 spazi** (tabs = errore con messaggio chiaro)
- F-string: `f"ciao {nome}!"`
- Numeri: `1_000_000`, `0xFF`, `0b1010`, `2.5e-3`
- Stringhe triple: `"""blocco di testo"""`
- Tutti gli operatori Neba: `..`, `..=`, `->`, `=>`, `**`, `//`, `::`

---

## Roadmap a colpo d'occhio

| Versione | Milestone | Stato |
|----------|-----------|-------|
| v0.1.0   | Lexer | ✅ Completato |
| v0.1.1   | Parser + AST | ✅ Completato |
| v0.1.2   | Tree-walking interpreter | ✅ Completato |
| v0.1.3   | REPL migliorato (history, completamento) | 🔜 Prossimo |
| v0.2.0   | Bytecode VM + GC | 📋 Pianificato |
| v0.2.1   | Type checker + inferenza | 📋 Pianificato |
| v0.3.0   | Cranelift JIT | 📋 Pianificato |
| v0.4.0   | LLVM backend + binari nativi | 📋 Pianificato |
| v1.0.0   | Stable release | 🏁 Obiettivo finale |
