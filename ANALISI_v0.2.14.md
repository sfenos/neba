# Neba — Analisi completa post v0.2.14
*Data: 2026-03-04 — basata su lettura integrale di vm.rs, compiler.rs, stdlib.rs, value.rs, chunk.rs, ast.rs, typecheck/*

---

## 1. BUG CONFERMATI (da fixare subito)

### 🔴 BUG-1 — `__init__` chiamato DUE VOLTE nel costruttore
**File:** `compiler.rs` righe ~1195–1240  
**Causa:** Il blocco `if !init_params.is_empty()` appare letteralmente due volte identico nel metodo `compile_class`. Il secondo blocco aggiunge i parametri ai `ctor.locals` una seconda volta, poi chiama di nuovo `CallMethod __init__`. Quindi `Point(1, 2)` esegue `__init__` due volte.  
**Fix:** Eliminare il secondo blocco duplicato (righe ~1227–1245).

### 🔴 BUG-2 — `kwargs` (named arguments) silenziosamente ignorati
**File:** `compiler.rs` righe 479, 489  
```rust
for (_, v) in kwargs { self.compile_expr(v)?; argc += 1; }
```
Il nome del kwarg viene scartato (`_`). `f(x=1, y=2)` compila come `f(1, 2)` — l'ordine è quello del sito di chiamata, non della firma. Se la firma è `fn f(y, x)` il comportamento è silenziosamente sbagliato.  
**Fix breve termine:** Errore a compile-time se kwargs presenti (non implementato).  
**Fix completo:** Riordinamento degli argomenti in base alla firma (richiede analisi a compile-time).

### 🟡 BUG-3 — `__init__` con parametri: `LoadLocal` carica slot sbagliati
**File:** `compiler.rs` righe ~1208–1222  
Il costruttore crea locali con `ctor.locals.push(...)` ma i parametri del costruttore non sono *effettivamente* sullo stack del costruttore — il costruttore è una closure con arity 0 dal punto di vista della VM. I `LoadLocal(i)` puntano a slot occupati dal frame interno, non ai parametri passati. Questo spiega perché `__init__` con parametri produce comportamenti inaspettati.  
**Fix:** Il costruttore deve avere arity = arity di `__init__`, e ricevere i parametri come locali reali (slot 0..n), poi passarli a `__init__`.

### 🟡 BUG-4 — `pop_scope` in `compile_while` non gestisce `loop_local_counts`
**File:** `compiler.rs` riga ~898  
`compile_while` chiama `self.loop_local_counts.pop()` ma non ha mai fatto `push` — la pop rimuove il contatore del for loop precedente se i loop sono annidati.  
**Fix:** Aggiungere `self.loop_local_counts.push(0)` all'inizio di `compile_while`.

---

## 2. FUNZIONALITÀ MANCANTI / INCOMPLETE

### 2A. Linguaggio

| Feature | Stato | Note |
|---|---|---|
| `mod nome` | ⚠️ warn+no-op | Il parser la supporta, la VM la ignora |
| `use path::to::mod` | ⚠️ warn+no-op | Idem |
| `async fn` / `await` | ⚠️ stub sincrono | `spawn` esegue sincronamente, `await` è no-op |
| Named kwargs (`f(x=1)`) | 🔴 silently broken | Il nome viene scartato, vedi BUG-2 |
| `**kwargs` (variadic named) | ❌ non implementato | Non presente nell'AST |
| `*args` (variadic positional) | ❌ non implementato | Non presente nell'AST |
| Tuple type | ❌ non implementato | Non c'è `Value::Tuple` |
| Set type | ❌ non implementato | Non c'è `Value::Set` |
| Destructuring assign | ❌ non implementato | `let (a, b) = pair` non parsato |
| Multi-target assign | ❌ non implementato | `a = b = 1` |
| Global/nonlocal stmt | ❌ non implementato | Variabili upvalue solo via closure |
| Exception / raise | ❌ non implementato | Solo `Ok/Err` monad |
| Generator / yield | ❌ non implementato | Non presente nell'AST |
| Slice notation | ❌ non implementato | `arr[1:3]` non parsato |
| String slicing | ❌ non implementato | `s[1:5]` |
| Decorator | ❌ non implementato | `@decorator` non nell'AST |
| Walrus operator `:=` | ❌ non implementato | Non nell'AST |
| Type alias | ❌ non implementato | `type Point = ...` |

### 2B. Tipi e contenitori

| Feature | Stato | Note |
|---|---|---|
| `Tuple` | ❌ mancante | `(a, b, c)` — nessun `Value::Tuple` |
| `Set` | ❌ mancante | Nessun `Value::Set` |
| `Dict` lookup | 🟡 O(n) | `Vec<(Value,Value)>` — lookup lineare |
| `HashMap` per Dict | ❌ non ancora | Pianificato v0.2.21 |
| `IntRange` in `len()` | 🟡 non gestito | `len(0..10)` dà errore |
| `IntRange` in `contains()` | 🟡 non gestito | `contains(range, 5)` fallisce |
| `IntRange` in `map/filter` | 🟡 non testato | map potrebbe fallire su IntRange |
| TypedArray: operatori in-place | ❌ mancante | `arr += 1` non supportato |
| TypedArray: reshape/slice avanzato | ❌ mancante | Solo 1D |
| TypedArray: `std`, `var` | ❌ mancante | Statistica base mancante |
| TypedArray: `argmin`/`argmax` | ❌ mancante | Restituisce indice, non valore |
| TypedArray: `abs`, `sqrt` elemento-wise | ❌ mancante | `abs(arr)` non funziona su TypedArray |
| TypedArray: conversione dtypes | ❌ mancante | `arr.astype("Float32")` |
| String: metodo su istanza | 🟡 solo via modulo | `s.upper()` non funziona, serve `string.upper(s)` |
| String: slice `s[1:3]` | ❌ non implementato | |
| String: format con posizionali `{0}` | ❌ mancante | Solo `{key}` da dict |
| Array: metodi su istanza | 🟡 solo `len` | `arr.push()` non funziona, serve `push(arr, v)` |

### 2C. Stdlib mancante

| Modulo/Funzione | Stato | Note |
|---|---|---|
| `random` module | ❌ mancante | `random.randint`, `random.random`, `random.choice`, `random.shuffle` |
| `time` module | ❌ mancante | `time.sleep`, `time.now`, `time.format` (solo `clock()` globale) |
| `os` module | ❌ mancante | `os.getenv`, `os.listdir`, `os.path.join` |
| `json` module | ❌ mancante | `json.parse`, `json.stringify` |
| `re` module (regex) | ❌ mancante | Nessun regex |
| `hash` / `hex` | ❌ mancante | `hash(v)`, `hex(n)`, `bin(n)`, `oct(n)` |
| `zip` globale | 🟡 solo in collections | Non disponibile come `zip(a, b)` globale |
| `enumerate` globale | 🟡 solo in collections | |
| `sorted` globale | 🟡 solo in collections | |
| `sum` globale | 🟡 solo in collections | `sum([1,2,3])` non funziona senza modulo |
| `any` / `all` globali | 🟡 solo in collections | |
| `chr` / `ord` | ❌ mancante | Conversione char↔int |
| `hex` / `bin` / `oct` | ❌ mancante | Rappresentazioni numeriche |
| `round` globale | ❌ mancante | Solo `math.round` |
| `copy` (shallow/deep) | ❌ mancante | `copy(arr)` per clonare |
| `collections.group_by` | ❌ mancante | Raggruppa per chiave |
| `collections.sliding_window` | ❌ mancante | |
| `string.encode`/`decode` | ❌ mancante | UTF-8 bytes |
| `io.stdin` / `io.stderr` | 🟡 parziale | Solo `input()` globale |

### 2D. OOP e trait incompleti

| Feature | Stato | Note |
|---|---|---|
| Ereditarietà | ❌ non implementato | `class B extends A` non nell'AST |
| `super()` | ❌ non implementato | |
| `isinstance` runtime | 🟡 solo `is` operatore | `is` controlla discriminant, non tipo nominale |
| Trait come type bound | ❌ non implementato | Solo dispatch, no static check |
| Operator overloading | ❌ non implementato | `__add__`, `__eq__`, `__str__` non chiamati |
| `__str__` / `__repr__` | ❌ non implementato | `println(obj)` stampa `Instance` |
| `__iter__` custom | ❌ non implementato | Solo Array/Dict/Str/TypedArray/IntRange |
| `__len__` custom | ❌ non implementato | |
| `__eq__` custom | ❌ non implementato | |
| Metodi statici | ❌ non implementato | |
| Classi generiche | ❌ non implementato | `class Stack[T]` |

---

## 3. OTTIMIZZAZIONI IDENTIFICATE (ordine di impatto)

### 3A. 🔥 ALTA PRIORITÀ — pre-JIT

#### O1 — `Dict` → `HashMap` (v0.2.21 roadmap)
**Impatto stimato:** 2–5× sui benchmark con classi (metodi cercati O(n) ad ogni call)  
**Attuale:** `Vec<(Value,Value)>` — `CallMethod` fa `iter().find()` su ogni metodo.  
**Fix:** `HashMap<String, Value>` per Instance.fields. Per dict con chiavi non-stringa, mantenere Vec come fallback.  
**Nota:** Questo impatta anche la ricerca dei metodi nel VM (`Op::GetField`, `Op::CallMethod`).

#### O2 — String interning (v0.2.21 roadmap)
**Impatto:** Confronti stringa O(n) → O(1), riduzione allocazioni  
**Attuale:** Ogni `Value::Str` è un `Rc<String>` indipendente. `"foo" == "foo"` compara byte per byte.  
**Fix:** Tabella globale di interning. `Value::Str(u32)` dove u32 è l'indice nella tabella.

#### O3 — Constant folding nel compiler (v0.2.19 roadmap)
**Impatto:** 1.2–1.5× su calcoli con letterali  
**Attuale:** `2 + 3 * 4` emette 3 const + 2 binop.  
**Fix:** Nella `compile_binary`, se entrambi gli operandi sono `Const` già valutabili, calcola a compile-time.

#### O4 — `Op::Add` specializzato per Int+Int inline (peephole)
**Impatto:** 10–20% su benchmark A1 (integer arithmetic)  
**Attuale:** `op_add` fa match su entrambi i valori — 4 branch prima di arrivare a Int+Int.  
**Fix in vm.rs:** Prima del dispatch generico, fast-path inline:
```rust
Op::Add => {
    // Fast path: Int + Int (caso più comune — nessuna chiamata a fn)
    if let (Some(Value::Int(a)), Some(Value::Int(b))) = 
        (self.stack.get(self.stack.len()-2), self.stack.last()) {
        let result = a + b;
        let len = self.stack.len();
        self.stack.truncate(len - 2);
        self.stack.push(Value::Int(result));
    } else {
        let r = pop!(); let l = pop!(); push!(self.op_add(l, r)?);
    }
}
```

#### O5 — `Vec::pop()` invece di `.last()` + `.clone()` per gli stack ops
**Impatto:** Riduzione clone() nei tipi heap-allocated  
**Attuale:** `pop!()` chiama `self.stack.pop()` che è già corretto, ma `peek!()` fa `.clone()` anche quando non necessario.

#### O6 — `step_limit` check fuori dall'hot path
**Attuale:** Il controllo `if self.step_limit > 0` è dentro `'dispatch: loop` — eseguito ad ogni opcode anche in produzione.  
**Fix già pianificato:** `#[cfg(debug_assertions)]` — in release build eliminare completamente il check.

### 3B. 🟡 MEDIA PRIORITÀ

#### O7 — `Rc<Closure>` → eliminare clone upvalues ad ogni call
**Attuale:** `c.upvalues.clone()` clona il Vec di upvalue ad ogni chiamata di metodo/funzione.  
**Fix:** `upvalues: Rc<Vec<Upvalue>>` — clone O(1) invece di O(n_upvalues).

#### O8 — `Chunk::add_const` deduplication è O(n)
**Attuale:** Ogni `add_const` scorre l'intero pool per trovare duplicati.  
**Fix:** `HashMap<ConstKey, u16>` per lookup O(1).

#### O9 — NaN-boxing per Value (v0.2.17 roadmap)
**Impatto stimato:** 2–4× su benchmark numerici  
Comprimerebbe Value da ~40 byte → 8 byte (u64).  
Architetturalmente complesso — dedicare una versione intera.

#### O10 — `FnProto.defaults: Vec<Value>` clonati ad ogni call
**Attuale:** Quando mancano argomenti opzionali, il VM fa `proto.defaults.get(di).cloned()`.  
**Fix:** `defaults: Rc<Vec<Value>>` — condivisi, non clonati.

### 3C. 🔵 BASSA PRIORITÀ (pre-JIT avanzato)

#### O11 — Peephole: `LoadLocal + LoadLocal + Add` → `AddLocals`
Richiederebbe un nuovo opcode e un post-pass sul bytecode.

#### O12 — Tail call optimization
`return f(args)` → riutilizza il frame invece di allocarne uno nuovo.
Fondamentale per ricorsione profonda (attualmente stack overflow a 256 frame).

#### O13 — Inline caching per GetField/CallMethod
Salvare l'ultima posizione trovata nel Dict per campo; skip lineare se hit.

---

## 4. PROBLEMI ARCHITETTURALI

### A4.1 — `call_value_sync` è una copia quasi-identica del loop principale
**File:** `vm.rs` righe ~770–1050  
Il mini-loop HOF duplica ~250 righe di codice. Qualsiasi fix al loop principale va replicato.  
**Fix ideale:** Unificare — il loop principale gestisce già ricorsione tramite frames, `call_value_sync` potrebbe essere rimosso usando lo stesso meccanismo con un sentinel di "depth target".

### A4.2 — `Dict` con chiavi stringa usato per TUTTO
Campi di istanza, moduli stdlib, dizionari utente — tutti usano `Vec<(Value,Value)>`. La ricerca metodo su una classe con 10 metodi fa 10 comparazioni di stringa ad ogni `CallMethod`.

### A4.3 — `Compiler::new_function` clona tutti i registry
```rust
fn_compiler.class_registry = self.class_registry.clone();
fn_compiler.trait_registry = self.trait_registry.clone();
fn_compiler.impl_registry  = self.impl_registry.clone();
```
Con molte classi questo è O(n_classes) per ogni funzione definita.  
**Fix:** `Arc<HashMap>` condiviso tra tutti i compiler dello stesso script.

### A4.4 — Typecheck non integrato nell'esecuzione
Il crate `neba_typecheck` esiste e ha test, ma `run()` in `lib.rs` non lo chiama mai. Gli errori di tipo sono solo runtime.

---

## 5. PIANO D'AZIONE SUGGERITO

### Sprint immediato (v0.2.15) — Bug critici
1. **Fix BUG-1**: Rimuovere il secondo blocco `__init__` duplicato in `compile_class`
2. **Fix BUG-2**: Errore a compile-time per kwargs nominati (non implementati)
3. **Fix BUG-3**: Costruttore con parametri — fix arity e LoadLocal
4. **Fix BUG-4**: `compile_while` — aggiungere `loop_local_counts.push(0)`
5. **Fix IntRange in `len()`** e in `map/filter`

### v0.2.16 — Qualità OOP
- `__str__` chiamato da `println` e `str()`
- `__eq__` chiamato da `==`
- `instanceof` operatore corretto (tipo nominale, non discriminant)
- Fix `isinstance` per trait (`obj is Trait` → cerca impl)

### v0.2.17 — Dict → HashMap + String interning
Questi due insieme danno il massimo beneficio sui benchmark OOP e string-heavy.

### v0.2.18 — Stdlib gaps prioritari
- `random` module (richiesto da molti use case)
- Funzioni globali: `sum`, `zip`, `enumerate`, `sorted`, `any`, `all`
- `chr` / `ord`
- `copy(arr)` shallow clone

### v0.2.19 — NaN-boxing (Value da 40→8 byte)

### v0.2.20 — Constant folding + peephole

### v0.2.21 — Benchmark gate → JIT

---

## 6. RIEPILOGO PRIORITÀ

```
URGENTE  (rompe codice utente):
  BUG-1  __init__ chiamato 2×
  BUG-2  kwargs silenziosamente sbagliati
  BUG-3  costruttore con parametri broken
  
ALTO     (funzionalità core mancante):
  Dict → HashMap           (perf OOP)
  __str__ / __eq__         (OOP base)
  sum/zip/enumerate globali (ergonomia)
  random module            (use case comune)
  IntRange nei gap stdlib  (consistenza)
  
MEDIO    (ottimizzazioni):
  O4   Add int fast-path
  O6   step_limit fuori hot path  
  O7   upvalues Rc
  O10  defaults Rc
  
BASSO    (architettura futura):
  call_value_sync unification
  Compiler registry Arc
  Typecheck integrato in run()
  Tail call optimization
```
