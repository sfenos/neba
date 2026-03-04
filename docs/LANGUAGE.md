# Neba Language Reference — v0.2.32

> Versione: 0.2.32
> Ultimo aggiornamento: 2026-03-04

---

## Indice

1. [Panoramica](#panoramica)
2. [Esecuzione](#esecuzione)
3. [Sintassi base](#sintassi-base)
4. [Tipi](#tipi)
5. [Variabili](#variabili)
6. [Operatori](#operatori)
7. [Stringhe](#stringhe)
8. [Controllo di flusso](#controllo-di-flusso)
9. [Loop](#loop)
10. [Funzioni](#funzioni)
11. [Lambda](#lambda)
12. [Classi](#classi)
13. [Traits](#traits)
14. [Collezioni](#collezioni)
15. [TypedArray](#typedarray)
16. [NdArray](#ndarray)
17. [Pattern matching](#pattern-matching)
18. [Error handling](#error-handling)
19. [Stdlib globali](#stdlib-globali)
20. [Moduli stdlib](#moduli-stdlib)
21. [HOF: map, filter, reduce, find](#hof)

---

## Panoramica

Neba è un linguaggio con indentazione significativa (come Python), tipizzazione graduale opzionale, e performance orientata a bytecode VM con JIT pianificato.

*Simple as Python, fast as C, secure as Rust, parallel as Go, powerful as Julia.*

**Stato attuale:** bytecode VM (v0.2.x) — compilatore AST→bytecode, stack machine, GC via Rc, stdlib completa (80+ funzioni globali, 6 moduli).

---

## Esecuzione

```bash
# Build
cargo build --release

# Esegui un file
./target/release/neba mio_file.neba

# Con cargo (development)
cargo run --bin neba -- mio_file.neba
```

---

## Sintassi base

Neba usa l'**indentazione a 4 spazi** per delimitare i blocchi. I tab sono un errore.
I commenti iniziano con `#`.

```neba
# Questo è un commento
println("Ciao, Neba!")
```

---

## Tipi

| Tipo | Esempio | Note |
|------|---------|------|
| `Int` | `42`, `-7`, `0xFF`, `0b1010` | intero a 64 bit con segno |
| `Float` | `3.14`, `-0.5`, `2.5e-3` | floating point 64 bit (IEEE 754) |
| `Bool` | `true`, `false` | |
| `Str` | `"ciao"`, `f"valore={x}"` | UTF-8, f-string supportate |
| `None` / `none` | `None` | assenza di valore (`none` è alias) |
| `Array` | `[1, 2, 3]` | lista dinamica eterogenea |
| `Dict` | `{"chiave": valore}` | mappa con ordine di inserimento |
| `Some(v)` | `Some(42)` | valore opzionale presente |
| `Ok(v)` | `Ok("risultato")` | risultato di successo |
| `Err(v)` | `Err("messaggio")` | risultato di errore |
| `Function` | `fn(x) x*2` | funzione o closure |
| `Range` | `0..10`, `0..=10` | range intero lazy |
| `Float64Array` | `Float64([1.0, 2.0])` | array numerico compatto Float64 |
| `Float32Array` | `Float32([1.0, 2.0])` | array numerico compatto Float32 |
| `Int64Array` | `Int64([1, 2, 3])` | array numerico compatto Int64 |
| `Int32Array` | `Int32([1, 2, 3])` | array numerico compatto Int32 |
| `NdArray` | `nd.array([[1.0,2.0],[3.0,4.0]])` | array multidimensionale |

**Promozione automatica:** operazioni tra `Int` e `Float` producono `Float`.
**Float IEEE 754:** `1.0/0.0 → Inf`, `0.0/0.0 → NaN`. Solo `Int/Int` lancia errore.

---

## Variabili

```neba
# Immutabile (default)
let x = 42
let nome = "Neba"

# Mutabile
var contatore = 0
contatore += 1

# Tipo annotato (opzionale)
let pi: Float = 3.14159
var flag: Bool = true
```

Le variabili `let` non possono essere riassegnate. Le variabili `var` possono.
**Indexing: 0-based** in tutto il linguaggio.

---

## Operatori

### Aritmetici
| Operatore | Significato | Esempio |
|-----------|-------------|---------|
| `+` | addizione | `3 + 4 → 7` |
| `-` | sottrazione | `10 - 3 → 7` |
| `*` | moltiplicazione | `3 * 4 → 12` |
| `/` | divisione float | `7 / 2 → 3.5` |
| `//` | divisione intera | `7 // 2 → 3` |
| `%` | modulo | `10 % 3 → 1` |
| `**` | potenza | `2 ** 8 → 256` |
| `-x` | negazione unaria | `-5` |

### Confronto
| Operatore | Significato |
|-----------|-------------|
| `==` | uguaglianza |
| `!=` | disuguaglianza |
| `<` `<=` `>` `>=` | confronto |
| `is` | tipo/trait identity — `obj is ClassName`, `obj is TraitName` |

### Logici
`and`  `or`  `not`

### Bitwise
`&`  `|`  `^`  `~`  `<<`  `>>`

### Assegnazione composta
`+=`  `-=`  `*=`  `/=`  `%=`

### Membership
```neba
3 in [1, 2, 3]       # true
"lo" in "ciao"       # true
5 not in [1, 2, 3]   # true
```

### Operatore `?` (propagazione errore)
```neba
fn leggi_file(path: Str) -> Result
    let contenuto = io.read_file(path)?   # early return Err se fallisce
    return Ok(contenuto)
```

---

## Stringhe

```neba
let s = "ciao"

# Concatenazione e ripetizione
let saluto = s + " mondo"    # "ciao mondo"
let rip    = "ab" * 3        # "ababab"

# Indicizzazione (0-based)
let primo  = s[0]    # "c"
let ultimo = s[-1]   # "o"

# Lunghezza
len(s)               # 4

# f-string (interpolazione)
let nome = "Neba"
let msg  = f"Ciao {nome}!"    # "Ciao Neba!"
let calc = f"2+2={2+2}"       # "2+2=4"
let expr = f"{math.sqrt(9.0)} è la radice di 9"
```

Vedi [string module](#string) per funzioni avanzate.

---

## Controllo di flusso

### if / elif / else

```neba
if x > 10
    println("grande")
elif x > 5
    println("medio")
else
    println("piccolo")
```

`if` è anche un'**espressione**:
```neba
let descrizione = if x > 0
    "positivo"
else
    "non positivo"
```

### match

```neba
match valore
    1       => println("uno")
    2 | 3   => println("due o tre")
    4..=9   => println("tra 4 e 9")
    Some(n) => println(f"ha valore {n}")
    None    => println("nessun valore")
    _       => println("altro")
```

**Corpo multi-linea** — dopo `=>` si può aprire un blocco indentato:
```neba
match codice
    0 =>
        println("zero")
        return "zero"
    _ =>
        if codice < 0
            return "negativo"
        else
            return "positivo"
```

`match` è anche un'**espressione**:
```neba
let label = match n
    0 => "zero"
    1 => "uno"
    _ => "altro"
```

---

## Loop

### while
```neba
var i = 0
while i < 10
    println(i)
    i += 1
```

### for-range
```neba
for i in 0..5     # 0, 1, 2, 3, 4 (esclusivo)
    println(i)

for i in 0..=5    # 0, 1, 2, 3, 4, 5 (inclusivo)
    println(i)
```

### for-in (array, dict, stringa)
```neba
let nomi = ["Alice", "Bob", "Carlo"]
for nome in nomi
    println(f"Ciao {nome}!")

for c in "ciao"
    println(c)

for k in keys(d)
    println(f"{k} → {d[k]}")
```

### break e continue
```neba
for i in 0..100
    if i == 5
        break

for i in 0..10
    if i % 2 != 0
        continue
    println(i)
```

---

## Funzioni

```neba
# Definizione base
fn somma(a: Int, b: Int) -> Int
    return a + b

# Parametri con default
fn saluta(nome: Str, prefisso: Str = "Ciao") -> Str
    return f"{prefisso} {nome}!"

somma(3, 4)            # 7
saluta("Neba")         # "Ciao Neba!"
saluta("Neba", "Hey")  # "Hey Neba!"
```

### Ricorsione
```neba
fn fattoriale(n: Int) -> Int
    if n <= 1
        return 1
    return n * fattoriale(n - 1)

fattoriale(5)   # 120
```

### Closures
```neba
fn make_counter(start: Int)
    var c = start
    fn next() -> Int
        c += 1
        return c
    return next

let cnt = make_counter(0)
cnt()   # 1
cnt()   # 2
cnt()   # 3
```

### Funzioni come valori
```neba
fn applica(f, x: Int) -> Int
    return f(x)

fn doppio(n: Int) -> Int
    return n * 2

applica(doppio, 7)   # 14
```

---

## Lambda

Funzioni anonime definibili inline come espressioni.

```neba
# Sintassi compatta (senza =>)
let quadrato = fn(x) x * x
quadrato(5)   # 25

# Sintassi arrow
let cubo = fn(x) => x * x * x

# Passate direttamente come argomenti
map([1, 2, 3], fn(x) x * 2)
filter([1, 2, 3, 4], fn(x) x % 2 == 0)
find([10, 20, 30], fn(x) x > 15)        # 20

# Catturano variabili esterne (sono closure)
let offset = 10
map([1, 2, 3], fn(x) x + offset)   # [11, 12, 13]

# Con corpo multi-linea
let classifica = fn(n) =>
    if n > 0
        return "positivo"
    else
        return "non positivo"
```

> **Nota:** `var` e `let` non sono supportati dentro lambda inline.
> Per logica complessa, usare funzioni nominate.

---

## Classi

```neba
class Cerchio
    raggio: Float = 0.0

    fn __init__(self, r: Float)
        self.raggio = r

    fn area(self) -> Float
        return math.pi * self.raggio ** 2

    fn __str__(self) -> Str
        return f"Cerchio(r={self.raggio})"


let c = Cerchio(5.0)
println(str(c))       # Cerchio(r=5.0)
println(c.area())     # 78.53981...
println(c is Cerchio) # true
```

### Campi con default
```neba
class Punto
    x: Float = 0.0
    y: Float = 0.0

    fn distanza(self) -> Float
        return (self.x ** 2 + self.y ** 2) ** 0.5

let p = Punto()
p.x = 3.0
p.y = 4.0
p.distanza()   # 5.0
```

### Method chaining
```neba
class Builder
    parts: Array = []
    fn __init__(self)
        self.parts = []
    fn add(self, v)
        append(self.parts, v)
        return self
    fn build(self)
        return join(self.parts, ", ")

let result = Builder().add("a").add("b").add("c").build()
# "a, b, c"
```

---

## Traits

Definiscono interfacce. I metodi senza body sono **astratti** (la classe deve implementarli). I metodi con body forniscono un'**implementazione di default**.

```neba
trait Descrivibile
    fn descrivi(self) -> Str    # astratto — la classe deve implementarlo

    fn stampa(self)             # default — può essere overridato
        println(self.descrivi())


class Gatto
    nome: Str = "Felix"

    fn __init__(self, nome: Str)
        self.nome = nome

impl Descrivibile for Gatto
    fn descrivi(self) -> Str
        return f"Gatto di nome {self.nome}"


let g = Gatto("Micio")
g.stampa()            # "Gatto di nome Micio"
g.descrivi()          # "Gatto di nome Micio"

g is Gatto            # true
g is Descrivibile     # true
```

### Più traits sullo stesso tipo
```neba
trait Drawable
    fn draw(self)           # astratto

trait Serializable
    fn serialize(self)
        return str(self)    # default

class Widget
    name: Str = ""
    fn __init__(self, n)
        self.name = n

impl Drawable for Widget
    fn draw(self)
        println(f"Drawing {self.name}")

impl Serializable for Widget  # usa il default di serialize

let w = Widget("button")
w.draw()         # Drawing button
w.serialize()    # usa __str__ via default
```

### Traits consecutivi senza body
```neba
# Supportato da v0.2.30 — più trait astratti consecutivi
trait A
    fn method_a(self)

trait B
    fn method_b(self)

trait C
    fn method_c(self)
        return "default C"
```

---

## Collezioni

### Array

```neba
let numeri = [1, 2, 3, 4, 5]

numeri[0]       # 1 (0-based)
numeri[-1]      # 5
len(numeri)     # 5
numeri[0] = 99

push(numeri, 6)        # aggiunge in fondo
pop(numeri)            # rimuove e restituisce l'ultimo
append(numeri, 7)      # alias di push
insert(numeri, 0, 100) # inserisce a indice 0
remove(numeri, 99)     # rimuove prima occorrenza → Bool
contains(numeri, 3)    # true
sort(numeri)           # in-place, crescente
reverse(numeri)        # in-place
join(numeri, ", ")     # "1, 2, 3, ..."

# Funzioni v0.2.30+
flatten([[1,2],[3,4]])         # [1, 2, 3, 4]
unique([1,2,2,3,1])            # [1, 2, 3]
concat([1,2], [3,4])           # [1, 2, 3, 4]
slice([1,2,3,4,5], 1, 4)       # [2, 3, 4]
index([10,20,30], 20)          # 1 (-1 se assente)
count([1,2,1,3], 1)            # 2
find([10,20,30], fn(x) x>15)   # 20 (None se assente)
find_index([10,20,30], fn(x) x>15)  # 1 (-1 se assente)
sorted([3,1,2])                # [1, 2, 3] (non distruttivo)
sorted([3,1,2], true)          # [3, 2, 1] (reverse)
sorted([3,1,2], fn(a,b) b-a)   # [3, 2, 1] (comparatore custom)
```

### Range

```neba
range(5)          # [0, 1, 2, 3, 4]
range(2, 8)       # [2, 3, 4, 5, 6, 7]
range(0, 10, 2)   # [0, 2, 4, 6, 8]
0..5              # Range esclusivo (lazy, per for-in)
0..=5             # Range inclusivo (lazy, per for-in)
len(0..10)        # 10
```

### Dict

```neba
var d = {"nome": "Neba", "versione": 2}

d["nome"]               # "Neba"
d["autore"] = "sfenos"

keys(d)                 # ["nome", "versione", "autore"]
values(d)               # [...]
items(d)                # [["nome", "Neba"], ...]
has_key(d, "nome")      # true
del_key(d, "nome")
len(d)

# Funzioni v0.2.30+
merge({"a":1}, {"b":2})        # {"a":1, "b":2} (d2 vince su chiavi duplicate)
dict_get(d, "chiave", default) # valore o default se assente

for coppia in items(d)
    println(f"{coppia[0]} → {coppia[1]}")
```

---

## TypedArray

Array con storage compatto per calcolo numerico.

### Costruttori

```neba
let fa  = Float64([1.0, 2.0, 3.0])
let ia  = Int64([1, 2, 3])
let z   = zeros(1000)                  # Float64Array di zeri
let o   = ones(1000)                   # Float64Array di uni
let f   = fill(100, 3.14)
let ls  = linspace(0.0, 1.0, 100)     # 100 valori equispaziati

# v0.2.30: costruttore con dimensione
let ta  = TypedArray(5, "Float64")     # 5 zeri Float64
let tb  = TypedArray(3, "Int32")       # 3 zeri Int32
let f64 = Float64(10)                  # 10 zeri Float64
```

### Accesso e operazioni

```neba
ta[0]             # lettura (0-based)
ta[-1]            # ultimo
ta[0] = 99.0      # scrittura

sum(ta)           # somma scalare
mean(ta)          # media
dot(a, b)         # prodotto scalare
min_elem(ta)
max_elem(ta)
to_list(ta)       # converte in Array
len(ta)
```

### Aritmetica element-wise

```neba
let a = Float64([1.0, 2.0, 3.0])
let b = Float64([4.0, 5.0, 6.0])

a + b       # Float64([5.0, 7.0, 9.0])
a * 2.0     # Float64([2.0, 4.0, 6.0])
a - b       # Float64([-3.0, -3.0, -3.0])
a / b       # element-wise division
```

---

## NdArray

Array multidimensionale, semantica NumPy-like (dal v0.2.25).

### Costruzione

```neba
let a = nd.array([1.0, 2.0, 3.0])              # 1D
let m = nd.array([[1.0,2.0],[3.0,4.0]])         # 2D
let z = nd.zeros(3, 4)                          # 3×4 di zeri
let o = nd.ones(2, 3)                           # 2×3 di uni
```

### Proprietà

```neba
m.shape    # [2, 2]
m.ndim     # 2
m.size     # 4
```

### Accesso e view semantics

```neba
m[0]              # prima riga (view — condivide il buffer)
m[0][1]           # elemento [0][1]
m[0][1] = 99.0    # modifica l'originale (view semantics)
m[[0,1]]          # accesso multi-indice — equivalente a m[0][1]
m[[0,1]] = 5.0    # scrittura multi-indice
```

### Operazioni

```neba
# Aritmetica element-wise
nd.add(a, b)      # a + b element-wise
nd.sub(a, b)
nd.mul(a, b)
nd.div(a, b)

# Broadcasting (regole NumPy)
nd.add(nd.zeros(3,1), nd.zeros(3,3))   # broadcast [3,1]+[3,3] → [3,3]

# Algebra lineare
nd.matmul(m, nd.transpose(m))
nd.transpose(m)
nd.reshape(m, [4])    # reshape a [4]
nd.flatten(m)         # → array 1D

# Aggregazioni (con axis opzionale)
nd.sum(m)             # scalare totale
nd.sum(m, 0)          # somma per colonne → array [2]
nd.mean(m, 1)         # media per righe
nd.max(m, 0)
nd.min(m, 1)

# Statistiche avanzate
nd.argmax(a)          # indice del massimo
nd.argmin(a)
nd.cumsum(a)          # somma cumulativa

# Boolean masking
let mask = nd.gt(a, 2.0)      # maschera booleana dove a > 2
nd.masked(a, mask)            # elementi dove mask è true
nd.nonzero(mask)              # indici dove mask è non-zero

# Slicing 2D
nd.slice(m, [0,2], [0,2])     # sottoarray righe 0..2, colonne 0..2
```

---

## Pattern matching

```neba
# Letterali e wildcard
match x
    0    => println("zero")
    1    => println("uno")
    _    => println("altro")

# Range (inclusivo)
match n
    1..=5  => println("piccolo")
    6..=10 => println("medio")
    _      => println("fuori range")

# Or-pattern
match c
    "a" | "e" | "i" | "o" | "u" => println("vocale")
    _                            => println("consonante")

# Stringhe e bool
match s
    "hello" => return "greeting"
    "bye"   => return "farewell"
    _       => return "unknown"

# Option e Result
match opt
    Some(v) => println(f"valore: {v}")
    None    => println("assente")

match risultato
    Ok(v)  => println(f"successo: {v}")
    Err(e) => println(f"errore: {e}")

# Corpo multi-linea
match codice
    0 =>
        var msg = "ok"
        println(msg)
    _ =>
        if codice < 0
            println("negativo")
        else
            println("positivo")
```

---

## Error handling

```neba
fn dividi(a: Float, b: Float) -> Result
    if b == 0.0
        return Err("divisione per zero")
    return Ok(a / b)

match dividi(10.0, 0.0)
    Ok(v)  => println(f"risultato: {v}")
    Err(e) => println(f"errore: {e}")

# Metodi su Result
r.is_ok()           # Bool
r.is_err()          # Bool
r.unwrap()          # valore o panic
r.unwrap_or(0.0)    # valore o default
r.unwrap_err()      # errore o panic (v0.2.31+)
r.value()           # alias di unwrap() per Ok; restituisce inner per Err

# Metodi su Option
opt.is_some()
opt.is_none()
opt.unwrap()
opt.unwrap_or(0)
opt.value()         # alias di unwrap()

# Operatore ? — early return su Err
fn pipeline(path: Str) -> Result
    let contenuto = io.read_file(path)?
    return Ok(len(contenuto))
```

---

## Stdlib globali

### I/O
| Funzione | Descrizione |
|----------|-------------|
| `print(v)` | stampa senza newline |
| `println(v)` | stampa con newline |
| `input(prompt?)` | legge riga da stdin |

### Conversioni
| Funzione | Descrizione |
|----------|-------------|
| `str(v)` | converte in stringa (chiama `__str__` su istanze) |
| `int(v)` | converte in Int |
| `float(v)` | converte in Float |
| `bool(v)` | converte in Bool |
| `type(v)` | nome del tipo (`"Int"`, `"Str"`, `"Function"`, `"Range"`, ...) |
| `repr(v)` | rappresentazione con quoting (`"hello"` → `"\"hello\""`) |

### Type Predicates (v0.2.30+)
| Funzione | Descrizione |
|----------|-------------|
| `is_int(v)` | Bool |
| `is_float(v)` | Bool |
| `is_str(v)` | Bool |
| `is_bool(v)` | Bool |
| `is_none(v)` | Bool |
| `is_array(v)` | Bool |
| `is_dict(v)` | Bool |

### Matematica
| Funzione | Descrizione |
|----------|-------------|
| `abs(n)` | valore assoluto |
| `min(a, b)` | minimo scalare |
| `max(a, b)` | massimo scalare |
| `sum(a)` | somma Array/TypedArray |
| `mean(a)` | media TypedArray |
| `pow(b, e)` | potenza |
| `chr(n)` | Int → carattere Unicode |
| `ord(c)` | carattere → Int |
| `hex(n)` | Int → stringa esadecimale |
| `bin(n)` | Int → stringa binaria |
| `oct(n)` | Int → stringa ottale |

### Array
| Funzione | Descrizione |
|----------|-------------|
| `len(a)` | lunghezza |
| `push(a, v)` | aggiunge in fondo |
| `pop(a)` | rimuove e restituisce l'ultimo |
| `append(a, v)` | alias di push |
| `insert(a, i, v)` | inserisce a indice i |
| `remove(a, v)` | rimuove prima occorrenza → Bool |
| `contains(a, v)` | Bool |
| `sort(a)` | ordina in-place |
| `reverse(a)` | inverte in-place |
| `join(a, sep?)` | unisce in stringa |
| `range(n)` / `range(s,e,step?)` | crea Array di interi |
| `zip(a, b)` | lista di coppie |
| `enumerate(a)` | lista di `[i, v]` |
| `any(a)` | almeno un truthy |
| `all(a)` | tutti truthy |
| `flatten(a)` | appiattisce di un livello |
| `unique(a)` | rimuove duplicati |
| `concat(a, b)` | concatena due Array |
| `slice(a, lo, hi)` | sotto-array o sotto-stringa |
| `index(a, v)` | primo indice di v (-1 se assente) |
| `count(a, v)` | occorrenze di v |
| `sorted(a)` | copia ordinata |
| `sorted(a, true)` | copia ordinata decrescente |
| `sorted(a, fn)` | copia con comparatore custom |
| `find(a, fn)` | primo elemento che soddisfa fn |
| `find_index(a, fn)` | indice del primo match (-1) |

### Dict
| Funzione | Descrizione |
|----------|-------------|
| `keys(d)` | Array delle chiavi |
| `values(d)` | Array dei valori |
| `items(d)` | Array di coppie `[k, v]` |
| `has_key(d, k)` | Bool |
| `del_key(d, k)` | rimuove la chiave |
| `merge(d1, d2)` | unisce (d2 vince su conflitti) |
| `dict_get(d, k, def?)` | valore o default |

### String globals
| Funzione | Descrizione |
|----------|-------------|
| `upper(s)` / `lower(s)` | conversione case |
| `strip(s)` / `lstrip` / `rstrip` | rimuovi whitespace |
| `split(s, sep?)` | divide la stringa |
| `replace(s, da, a)` | sostituisce occorrenze |
| `find(s, sub)` / `index_of(s, sub)` | primo indice (-1) |
| `contains(s, sub)` | Bool |
| `starts_with(s, p)` / `ends_with(s, p)` | Bool |
| `capitalize(s)` / `title(s)` | maiuscole |
| `pad_left(s, n, ch?)` / `pad_right` | padding |
| `zfill(s, n)` | zero-padding numerico |
| `char_at(s, i)` | carattere alla posizione i |
| `is_digit(s)` / `is_alpha(s)` | predicati |
| `repr(v)` | rappresentazione con quoting |

### TypedArray
| Funzione | Descrizione |
|----------|-------------|
| `Float64(arr\|n)` | Float64Array da lista o di n zeri |
| `Float32(arr)` | Float32Array |
| `Int64(arr)` | Int64Array |
| `Int32(arr)` | Int32Array |
| `TypedArray(n, dtype)` | array di n zeri del dtype specificato |
| `zeros(n)` / `ones(n)` | array Float64 |
| `fill(n, v)` | array riempito con v |
| `linspace(start, stop, n)` | n valori equispaziati |
| `sum(ta)` | somma scalare |
| `mean(ta)` | media |
| `dot(a, b)` | prodotto scalare |
| `min_elem(ta)` / `max_elem(ta)` | elemento min/max |
| `to_list(ta)` | converte in Array |

### Utility
| Funzione | Descrizione |
|----------|-------------|
| `assert(cond, msg?)` | errore se cond è false |
| `clock()` | timestamp Unix in secondi (Float) |
| `time_ms()` | timestamp in millisecondi |

---

## Moduli stdlib

Accessibili con la sintassi `modulo.funzione(args)`.

### `math`

Costanti: `math.pi`, `math.e`, `math.tau`, `math.inf`, `math.nan`

| Funzione | Descrizione |
|----------|-------------|
| `math.sqrt(x)` | radice quadrata |
| `math.pow(b, e)` | potenza b^e |
| `math.exp(x)` | e^x |
| `math.log(x)` / `log(x, b)` | logaritmo naturale o in base b |
| `math.log2(x)` / `log10(x)` | log base 2/10 |
| `math.sin(x)` / `cos` / `tan` | trigonometria |
| `math.asin(x)` / `acos` / `atan` | trigonometria inversa |
| `math.atan2(y, x)` | arcotangente con quadrante |
| `math.sinh(x)` / `cosh` / `tanh` | iperboliche |
| `math.hypot(x, y)` | ipotenusa |
| `math.degrees(x)` / `radians(x)` | conversione angoli |
| `math.floor(x)` / `ceil(x)` / `round(x,d?)` / `trunc(x)` | arrotondamenti |
| `math.sign(x)` | -1, 0 o 1 |
| `math.clamp(x, lo, hi)` | limita nell'intervallo |
| `math.abs(x)` | valore assoluto |
| `math.gcd(a, b)` / `lcm(a, b)` | MCD/mcm |
| `math.factorial(n)` | fattoriale (0–20) |
| `math.cbrt(x)` | radice cubica |
| `math.isnan(x)` / `is_nan(x)` | Bool |
| `math.isinf(x)` / `is_inf(x)` | Bool |
| `math.is_finite(x)` | Bool (v0.2.31+) |
| `math.random()` | Float in [0, 1) |
| `math.randint(a, b)` | Int in [a, b] |

```neba
math.sqrt(9.0)              # 3.0
math.sin(math.pi / 2.0)    # 1.0
math.clamp(5.0, 0.0, 3.0)  # 3.0
math.factorial(5)           # 120
math.is_nan(0.0/0.0)        # true
math.is_finite(1.0)         # true
```

---

### `string`

| Funzione | Descrizione |
|----------|-------------|
| `string.split(s, sep?, n?)` | divide la stringa |
| `string.lines(s)` | divide per newline |
| `string.chars(s)` | Array di caratteri |
| `string.strip(s)` / `lstrip` / `rstrip` | whitespace |
| `string.upper(s)` / `lower(s)` | conversione case |
| `string.replace(s, da, a, n?)` | sostituisce occorrenze (n = max) |
| `string.find(s, sub)` / `index_of` | primo indice (-1) |
| `string.rfind(s, sub)` | ultimo indice |
| `string.count(s, sub)` | numero di occorrenze |
| `string.startswith(s, p)` / `endswith` | Bool |
| `string.contains(s, sub)` | Bool |
| `string.repeat(s, n)` | ripete la stringa |
| `string.pad_left(s, w, ch?)` / `pad_right` | padding |
| `string.center(s, w, ch?)` | centra |
| `string.ljust(s, w)` / `rjust` | allineamento |
| `string.zfill(s, n)` | zero-padding |
| `string.capitalize(s)` / `title(s)` | maiuscole |
| `string.is_digit(s)` | Bool |
| `string.is_alpha(s)` | Bool |
| `string.is_alnum(s)` | Bool |
| `string.is_upper(s)` / `is_lower(s)` | Bool |
| `string.is_empty(s)` | Bool |
| `string.reverse(s)` | inverte la stringa (v0.2.32+) |
| `string.char_at(s, i)` | carattere alla posizione i (v0.2.32+) |
| `string.index_of(s, sub)` | alias di find (v0.2.32+) |
| `string.repr(v)` | rappresentazione con quoting (v0.2.32+) |
| `string.slice(s, lo, hi)` | sotto-stringa |
| `string.format(tmpl, dict)` | sostituisce `{chiave}` con dict |
| `string.to_int(s)` / `to_float(s)` | conversione |

```neba
string.split("a,b,c", ",")                       # ["a", "b", "c"]
string.replace("hello world", "world", "neba")   # "hello neba"
string.pad_left("42", 5, "0")                    # "00042"
string.format("Ciao {nome}!", {"nome": "Neba"})  # "Ciao Neba!"
string.reverse("hello")                           # "olleh"
string.char_at("hello", 1)                        # "e"
```

---

### `io`

| Funzione | Descrizione |
|----------|-------------|
| `io.read_file(path)` | legge file → Str |
| `io.write_file(path, s)` | scrive (crea/sovrascrive) |
| `io.append_file(path, s)` | aggiunge in coda |
| `io.read_lines(path)` | Array di righe |
| `io.file_exists(path)` | Bool |
| `io.delete_file(path)` | elimina il file |
| `io.listdir(path)` | Array di nomi file |
| `io.cwd()` | directory corrente |
| `io.mkdir(path)` | crea directory |

### `io.path`

| Funzione | Descrizione |
|----------|-------------|
| `io.path.join(a, b)` | unisce percorsi |
| `io.path.dirname(p)` | directory genitore |
| `io.path.basename(p)` | nome file |
| `io.path.stem(p)` | nome senza estensione |
| `io.path.ext(p)` | estensione |
| `io.path.exists(p)` | Bool |
| `io.path.isfile(p)` | Bool |
| `io.path.isdir(p)` | Bool |

---

### `collections`

| Funzione | Descrizione |
|----------|-------------|
| `collections.chunk(a, n)` | divide in sotto-array di n |
| `collections.take(a, n)` | primi n elementi |
| `collections.drop(a, n)` | salta i primi n |
| `collections.flatten(a)` | appiattisce di un livello |
| `collections.unique(a)` | rimuove duplicati |
| `collections.concat(a, b)` | concatena due array |
| `collections.repeat(a, n)` | ripete l'array |
| `collections.transpose(m)` | trasposta di matrice di array |
| `collections.sum(a)` | somma numerica |
| `collections.product(a)` | prodotto numerico |
| `collections.any(a)` | almeno un truthy |
| `collections.all(a)` | tutti truthy |
| `collections.none(a)` | nessun truthy |
| `collections.none_of(a)` | alias di none (v0.2.31+) |
| `collections.first(a)` | `Some(primo)` o `None` |
| `collections.last(a)` | `Some(ultimo)` o `None` |
| `collections.count_by(a, v)` | occorrenze di v |
| `collections.sorted(a)` | copia ordinata |

```neba
collections.chunk([1,2,3,4,5], 2)   # [[1,2],[3,4],[5]]
collections.take([1,2,3,4,5], 3)    # [1,2,3]
collections.first([10,20,30])       # Some(10)
collections.none([false, false])    # true
```

---

### `random`

| Funzione | Descrizione |
|----------|-------------|
| `random.choice(a)` | elemento casuale |
| `random.shuffle(a)` | mescola in-place |
| `random.seed(n)` | imposta seed |
| `random.sample(a, n)` | n elementi senza rimpiazzo |

---

## HOF

Funzioni di ordine superiore built-in. Accettano funzioni nominate o lambda.

```neba
# map — trasforma ogni elemento
map([1, 2, 3, 4], fn(x) x * 2)                 # [2, 4, 6, 8]
map(0..10, fn(x) x * x)                         # [0,1,4,9,16,25,36,49,64,81]

# filter — seleziona elementi
filter([1,2,3,4,5,6], fn(x) x % 2 == 0)        # [2, 4, 6]
filter(0..100, fn(x) x % 7 == 0)               # [0,7,14,...,98]

# reduce — piega l'array
reduce([1,2,3,4,5], fn(acc, x) acc + x)         # 15
reduce([1,2,3,4,5], fn(acc, x) acc * x)         # 120
reduce([1,2,3], fn(acc, x) acc + x, 10)         # 16

# find — primo elemento che soddisfa predicato (None se assente)
find([10,20,30,5], fn(x) x > 15)               # 20
find([1,2,3], fn(x) x > 100)                   # None

# find_index — indice del primo match (-1 se assente)
find_index([10,20,30,5], fn(x) x > 15)         # 1

# Pipeline composta
reduce(
    map(
        filter([1,2,3,4,5,6], fn(x) x % 2 == 0),
        fn(x) x * x
    ),
    fn(acc, x) acc + x
)
# → 4 + 16 + 36 = 56

# Con funzione nominata
fn is_positive(n: Int) -> Bool
    return n > 0

filter([-3, -1, 0, 2, 5], is_positive)   # [2, 5]
```

### Regole

- `map` e `filter` accettano Array o Range
- `reduce` richiede array non vuoto senza `initial`
- `find` restituisce `None` (non errore) se nessun elemento soddisfa il predicato
- Le lambda catturano variabili esterne come closure
- Chiamate multi-riga supportate (parentesi aperte sopprimono i newline)

---

## Prossimamente (v0.3.x)

- **Cranelift JIT** — compilazione nativa delle funzioni hot
- **NaN-boxing** — Value 24→8 byte
- **String methods su istanza** — `"hello".upper()`
- **String interning** — confronti O(1)
- **Tail call optimization**
