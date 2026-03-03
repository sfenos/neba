# Neba Language Reference — v0.2.13

> Versione: 0.2.13  
> Ultimo aggiornamento: 2026-03-03

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
16. [Pattern matching](#pattern-matching)
17. [Error handling](#error-handling)
18. [Stdlib globali](#stdlib-globali)
19. [Moduli stdlib](#moduli-stdlib)
20. [HOF: map, filter, reduce](#hof-map-filter-reduce)

---

## Panoramica

Neba è un linguaggio con indentazione significativa (come Python), tipizzazione graduale opzionale, e performance orientata a bytecode VM con JIT pianificato.

*Simple as Python, fast as C, secure as Rust, parallel as Go, powerful as Julia.*

**Stato attuale:** bytecode VM (v0.2.x) — compilatore AST→bytecode, stack machine, GC via Rc, stdlib completa.

---

## Esecuzione

```bash
# Esegui un file
cargo run --bin neba -- mio_file.neba

# REPL interattivo
cargo run --bin neba_repl
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

| Tipo           | Esempio                      | Note                            |
|----------------|------------------------------|---------------------------------|
| `Int`          | `42`, `-7`, `0xFF`, `0b1010` | intero a 64 bit con segno       |
| `Float`        | `3.14`, `-0.5`, `2.5e-3`     | floating point 64 bit           |
| `Bool`         | `true`, `false`              |                                 |
| `Str`          | `"ciao"`, `f"valore={x}"`    | UTF-8, f-string supportate      |
| `None`         | `None`                       | assenza di valore               |
| `Array`        | `[1, 2, 3]`                  | lista dinamica eterogenea       |
| `Dict`         | `{"chiave": valore}`         | mappa con ordine di inserimento |
| `Some(v)`      | `Some(42)`                   | valore opzionale presente       |
| `Ok(v)`        | `Ok("risultato")`            | risultato di successo           |
| `Err(v)`       | `Err("messaggio")`           | risultato di errore             |
| `Float64Array` | `ones(100)`                  | array numerico compatto Float64 |
| `Float32Array` | `Float32([1.0, 2.0])`        | array numerico compatto Float32 |
| `Int64Array`   | `Int64([1, 2, 3])`           | array numerico compatto Int64   |
| `Int32Array`   | `Int32([1, 2, 3])`           | array numerico compatto Int32   |

**Promozione automatica:** operazioni tra `Int` e `Float` producono `Float`.

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
| Operatore | Significato      | Esempio        |
|-----------|------------------|----------------|
| `+`       | addizione        | `3 + 4 → 7`   |
| `-`       | sottrazione      | `10 - 3 → 7`  |
| `*`       | moltiplicazione  | `3 * 4 → 12`  |
| `/`       | divisione float  | `7 / 2 → 3.5` |
| `//`      | divisione intera | `7 // 2 → 3`  |
| `%`       | modulo           | `10 % 3 → 1`  |
| `**`      | potenza          | `2 ** 8 → 256`|
| `-x`      | negazione unaria | `-5`           |

### Confronto
| Operatore | Significato       |
|-----------|-------------------|
| `==`      | uguaglianza       |
| `!=`      | disuguaglianza    |
| `<`       | minore            |
| `<=`      | minore o uguale   |
| `>`       | maggiore          |
| `>=`      | maggiore o uguale |
| `is`      | identità di tipo  |

### Logici
| Operatore | Significato                |
|-----------|----------------------------|
| `and`     | AND logico (cortocircuito) |
| `or`      | OR logico (cortocircuito)  |
| `not`     | NOT logico                 |

### Bitwise
`&`, `|`, `^`, `~`, `<<`, `>>`

### Assegnazione composta
`+=`, `-=`, `*=`, `/=`, `%=`

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

# f-string
let nome = "Neba"
let msg  = f"Ciao {nome}!"    # "Ciao Neba!"
let calc = f"2+2={2+2}"       # "2+2=4"
```

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
    case 1       => println("uno")
    case 2 | 3   => println("due o tre")
    case 4..=9   => println("tra 4 e 9")
    case Some(n) => println(f"ha valore {n}")
    case None    => println("nessun valore")
    case _       => println("altro")
```

**Corpo multi-linea** nel case (dal v0.2.12) — dopo `=>` si può aprire un blocco indentato:

```neba
match codice
    case 0 =>
        println("zero")
        return "zero"
    case _ =>
        if codice < 0
            return "negativo"
        else
            return "positivo"
```

`match` è anche un'**espressione**:

```neba
let label = match n
    case 0 => "zero"
    case 1 => "uno"
    case _ => "altro"
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
# Range esclusivo: 0, 1, 2, 3, 4
for i in 0..5
    println(i)

# Range inclusivo: 0, 1, 2, 3, 4, 5
for i in 0..=5
    println(i)
```

### for-in (su array o stringa)

```neba
let nomi = ["Alice", "Bob", "Carlo"]
for nome in nomi
    println(f"Ciao {nome}!")

for c in "ciao"
    println(c)
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
        c += 1      # cattura 'c' come upvalue mutabile
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

Funzioni anonime definibili inline come espressioni (dal v0.2.12):

```neba
# Sintassi: fn(params) => espressione
let quadrato = fn(x) => x * x
quadrato(5)   # 25

# Passate direttamente come argomenti
map([1, 2, 3], fn(x) => x * 2)
filter([1, 2, 3, 4], fn(x) => x % 2 == 0)
reduce([1, 2, 3, 4, 5], fn(acc, x) => acc + x)

# Catturano variabili esterne (sono closure)
let offset = 10
map([1, 2, 3], fn(x) => x + offset)   # [11, 12, 13]

# Con corpo multi-linea (blocco indentato dopo =>)
let classifica = fn(n) =>
    if n > 0
        return "positivo"
    else
        return "non positivo"
```

---

## Classi

```neba
class Cerchio
    raggio: Float = 0.0

    fn __init__(self, r: Float)
        self.raggio = r

    fn area(self) -> Float
        return 3.14159 * self.raggio ** 2

    fn descrivi(self)
        println(f"Cerchio con raggio {self.raggio}")


let c = Cerchio(5.0)
c.descrivi()        # "Cerchio con raggio 5.0"
println(c.area())   # 78.53975
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

---

## Traits

Definiscono interfacce con metodi opzionalmente implementati di default.

```neba
trait Descrivibile
    fn descrivi(self) -> Str

    fn stampa(self)          # metodo con implementazione di default
        println(self.descrivi())


class Gatto
    nome: Str = "Felix"

    fn __init__(self, nome: Str)
        self.nome = nome

impl Descrivibile for Gatto
    fn descrivi(self) -> Str
        return f"Gatto di nome {self.nome}"


let g = Gatto("Micio")
g.stampa()            # usa il default del trait → "Gatto di nome Micio"
g.descrivi()          # "Gatto di nome Micio"

g is Gatto            # true
g is Descrivibile     # true
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

push(numeri, 6)
pop(numeri)
append(numeri, 7)
insert(numeri, 0, 100)
remove(numeri, 99)        # → Bool
contains(numeri, 3)       # true
sort(numeri)              # in-place
reverse(numeri)           # in-place
join(numeri, ", ")        # "1, 2, 3, ..."
```

### Range

```neba
range(5)          # [0, 1, 2, 3, 4]
range(2, 8)       # [2, 3, 4, 5, 6, 7]
range(0, 10, 2)   # [0, 2, 4, 6, 8]
0..5              # range esclusivo (per for-in)
0..=5             # range inclusivo (per for-in)
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

for coppia in items(d)
    println(f"{coppia[0]} → {coppia[1]}")
```

---

## TypedArray

Array con storage compatto per calcolo numerico (dal v0.2.6).

### Costruttori

```neba
let fa  = Float64([1.0, 2.0, 3.0])
let ia  = Int64([1, 2, 3])
let z   = zeros(1000)                  # Float64Array di zeri
let o   = ones(1000)                   # Float64Array di uni
let zi  = zeros(1000, "Int64")
let f   = fill(100, 3.14)
let ls  = linspace(0.0, 1.0, 100)     # 100 valori equispaziati
```

### Accesso e operazioni

```neba
ta[0]             # lettura (0-based)
ta[-1]            # ultimo
ta[1..3]          # slice → nuovo TypedArray
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
```

---

## Pattern matching

```neba
# Letterali e wildcard
match x
    case 0    => println("zero")
    case 1    => println("uno")
    case _    => println("altro")

# Range
match n
    case 1..=5  => println("piccolo")
    case 6..=10 => println("medio")
    case _      => println("fuori range")

# Or-pattern
match c
    case "a" | "e" | "i" | "o" | "u" => println("vocale")
    case _                            => println("consonante")

# Option e Result
match opt
    case Some(v) => println(f"valore: {v}")
    case None    => println("assente")

match risultato
    case Ok(v)  => println(f"successo: {v}")
    case Err(e) => println(f"errore: {e}")

# Corpo multi-linea (dal v0.2.12)
match codice
    case 0 =>
        var msg = "ok"
        println(msg)
    case _ =>
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
    case Ok(v)  => println(f"risultato: {v}")
    case Err(e) => println(f"errore: {e}")

# Metodi
r.is_ok()          # Bool
r.is_err()         # Bool
r.unwrap()         # valore o panic
r.unwrap_or(0.0)   # valore o default

opt.is_some()
opt.is_none()
opt.unwrap()
opt.unwrap_or(0)

# Operatore ? — early return su Err
fn pipeline(path: Str) -> Result
    let contenuto = io.read_file(path)?
    return Ok(len(contenuto))
```

---

## Stdlib globali

### I/O
| Funzione          | Descrizione              |
|-------------------|--------------------------|
| `print(v)`        | stampa senza newline     |
| `println(v)`      | stampa con newline       |
| `input(prompt?)`  | legge riga da stdin      |

### Conversioni
| Funzione     | Descrizione         |
|--------------|---------------------|
| `str(v)`     | converte in stringa |
| `int(v)`     | converte in Int     |
| `float(v)`   | converte in Float   |
| `bool(v)`    | converte in Bool    |
| `typeof(v)`  | nome del tipo       |

### Matematica
| Funzione       | Descrizione                       |
|----------------|-----------------------------------|
| `abs(n)`       | valore assoluto                   |
| `min(a, b)`    | minimo (o minimo di un Array)     |
| `max(a, b)`    | massimo (o massimo di un Array)   |

### Array
| Funzione                     | Descrizione                     |
|------------------------------|---------------------------------|
| `len(a)`                     | lunghezza                       |
| `push(a, v)`                 | aggiunge in fondo               |
| `pop(a)`                     | rimuove e restituisce l'ultimo  |
| `append(a, v)`               | alias di push                   |
| `insert(a, i, v)`            | inserisce a indice i            |
| `remove(a, v)`               | rimuove prima occorrenza → Bool |
| `contains(a, v)`             | Bool                            |
| `sort(a)`                    | ordina in-place                 |
| `reverse(a)`                 | inverte in-place                |
| `join(a, sep?)`              | unisce in stringa               |
| `range(n)` / `range(s,e,step?)` | crea Array di interi         |

### Dict
| Funzione         | Descrizione            |
|------------------|------------------------|
| `keys(d)`        | Array delle chiavi     |
| `values(d)`      | Array dei valori       |
| `items(d)`       | Array di coppie [k,v]  |
| `has_key(d, k)`  | Bool                   |
| `del_key(d, k)`  | rimuove la chiave      |

### TypedArray
| Funzione                    | Descrizione                      |
|-----------------------------|----------------------------------|
| `Float64(arr)`              | costruisce Float64Array          |
| `Float32(arr)`              | costruisce Float32Array          |
| `Int64(arr)`                | costruisce Int64Array            |
| `Int32(arr)`                | costruisce Int32Array            |
| `zeros(n, dtype?)`          | TypedArray di zeri               |
| `ones(n)`                   | Float64Array di uni              |
| `fill(n, v)`                | TypedArray riempito con v        |
| `linspace(start, stop, n)`  | n valori equispaziati            |
| `sum(ta)`                   | somma scalare                    |
| `mean(ta)`                  | media                            |
| `dot(a, b)`                 | prodotto scalare                 |
| `min_elem(ta)`              | elemento minimo                  |
| `max_elem(ta)`              | elemento massimo                 |
| `to_list(ta)`               | converte in Array                |

### HOF
| Funzione                      | Descrizione                            |
|-------------------------------|----------------------------------------|
| `map(array, fn)`              | applica fn a ogni elemento             |
| `filter(array, fn)`           | mantiene elementi dove fn è truthy     |
| `reduce(array, fn)`           | piega con fn, primo elem come accumul. |
| `reduce(array, fn, initial)`  | piega con valore iniziale              |

### Utility
| Funzione             | Descrizione                       |
|----------------------|-----------------------------------|
| `assert(cond, msg?)` | errore se cond è false            |
| `clock()`            | timestamp Unix in secondi (Float) |

---

## Moduli stdlib

Accessibili con la sintassi `modulo.funzione(args)` (dal v0.2.11).

### `math`

Costanti: `math.pi`, `math.e`, `math.tau`, `math.inf`, `math.nan`

| Funzione                  | Descrizione                      |
|---------------------------|----------------------------------|
| `math.sqrt(x)`            | radice quadrata                  |
| `math.pow(b, e)`          | potenza b^e                      |
| `math.exp(x)`             | e^x                              |
| `math.log(x)`             | logaritmo naturale               |
| `math.log(x, b)`          | logaritmo in base b              |
| `math.log2(x)`            | log base 2                       |
| `math.log10(x)`           | log base 10                      |
| `math.sin(x)` / `cos` / `tan` | trigonometria               |
| `math.asin(x)` / `acos` / `atan` | trigonometria inversa   |
| `math.atan2(y, x)`        | arcotangente con quadrante       |
| `math.hypot(x, y)`        | ipotenusa √(x²+y²)              |
| `math.degrees(x)`         | radianti → gradi                 |
| `math.radians(x)`         | gradi → radianti                 |
| `math.floor(x)`           | → Int, arrotonda verso -∞        |
| `math.ceil(x)`            | → Int, arrotonda verso +∞        |
| `math.round(x, d?)`       | arrotonda a d decimali           |
| `math.trunc(x)`           | → Int, tronca verso 0            |
| `math.sign(x)`            | -1, 0 o 1                        |
| `math.clamp(x, lo, hi)`   | limita x nell'intervallo [lo,hi] |
| `math.abs(x)`             | valore assoluto                  |
| `math.gcd(a, b)`          | massimo comun divisore           |
| `math.lcm(a, b)`          | minimo comune multiplo           |
| `math.factorial(n)`       | fattoriale (0–20)                |
| `math.isnan(x)`           | Bool                             |
| `math.isinf(x)`           | Bool                             |

```neba
math.sqrt(9.0)             # 3.0
math.sin(math.pi / 2.0)   # 1.0
math.log(math.e)           # 1.0
math.clamp(5.0, 0.0, 3.0)  # 3.0
math.factorial(5)          # 120
math.degrees(math.pi)      # 180.0
```

---

### `string`

| Funzione                          | Descrizione                          |
|-----------------------------------|--------------------------------------|
| `string.split(s, sep?, n?)`       | divide la stringa                    |
| `string.lines(s)`                 | divide per newline                   |
| `string.chars(s)`                 | Array di caratteri                   |
| `string.strip(s)`                 | rimuove whitespace iniziale/finale   |
| `string.lstrip(s)` / `rstrip(s)` | solo da sinistra / destra            |
| `string.upper(s)` / `lower(s)`   | conversione case                     |
| `string.replace(s, da, a, n?)`   | sostituisce occorrenze               |
| `string.find(s, sub)`            | indice prima occorrenza (-1 se assente) |
| `string.rfind(s, sub)`           | indice ultima occorrenza             |
| `string.count(s, sub)`           | numero di occorrenze                 |
| `string.startswith(s, prefix)`   | Bool                                 |
| `string.endswith(s, suffix)`     | Bool                                 |
| `string.contains(s, sub)`        | Bool                                 |
| `string.repeat(s, n)`            | ripete la stringa                    |
| `string.pad_left(s, w, ch?)`     | padding a sinistra                   |
| `string.pad_right(s, w, ch?)`    | padding a destra                     |
| `string.is_empty(s)`             | Bool                                 |
| `string.format(tmpl, dict)`      | sostituisce `{chiave}` con dict      |

```neba
string.split("a,b,c", ",")                        # ["a", "b", "c"]
string.upper("neba")                               # "NEBA"
string.replace("hello world", "world", "neba")    # "hello neba"
string.pad_left("42", 5, "0")                     # "00042"
string.format("Ciao {nome}!", {"nome": "Neba"})   # "Ciao Neba!"
```

---

### `io`

| Funzione                    | Descrizione                  |
|-----------------------------|------------------------------|
| `io.read_file(path)`        | legge file → Str             |
| `io.write_file(path, s)`    | scrive (crea/sovrascrive)    |
| `io.append_file(path, s)`   | aggiunge in coda             |
| `io.file_exists(path)`      | Bool                         |
| `io.read_lines(path)`       | Array di righe               |
| `io.delete_file(path)`      | elimina il file              |

```neba
io.write_file("/tmp/test.txt", "ciao\n")
let testo = io.read_file("/tmp/test.txt")
let righe = io.read_lines("/tmp/test.txt")
io.file_exists("/tmp/test.txt")    # true
io.delete_file("/tmp/test.txt")
```

---

### `collections`

| Funzione                       | Descrizione                          |
|--------------------------------|--------------------------------------|
| `collections.zip(a, b)`        | coppie parallele                     |
| `collections.enumerate(a, s?)` | aggiunge indice (default start=0)    |
| `collections.flatten(a)`       | appiattisce di un livello            |
| `collections.unique(a)`        | rimuove duplicati                    |
| `collections.sorted(a)`        | copia ordinata (non distruttivo)     |
| `collections.chunk(a, n)`      | divide in sotto-array di n elementi  |
| `collections.take(a, n)`       | primi n elementi                     |
| `collections.drop(a, n)`       | salta i primi n                      |
| `collections.concat(a, b)`     | concatena due array                  |
| `collections.repeat(a, n)`     | ripete l'array n volte               |
| `collections.transpose(m)`     | trasposta di matrice                 |
| `collections.sum(a)`           | somma numerica                       |
| `collections.product(a)`       | prodotto numerico                    |
| `collections.any(a)`           | almeno un truthy                     |
| `collections.all(a)`           | tutti truthy                         |
| `collections.none(a)`          | nessun truthy                        |
| `collections.first(a)`         | Some(primo) o None                   |
| `collections.last(a)`          | Some(ultimo) o None                  |
| `collections.count_by(a, v)`   | occorrenze di v                      |

```neba
collections.zip([1,2,3], ["a","b","c"])   # [[1,"a"],[2,"b"],[3,"c"]]
collections.flatten([[1,2],[3,4]])         # [1, 2, 3, 4]
collections.sorted([3,1,2])              # [1, 2, 3]
collections.chunk([1,2,3,4,5], 2)         # [[1,2],[3,4],[5]]
collections.first([10,20,30])            # Some(10)
```

---

## HOF: map, filter, reduce

Funzioni di ordine superiore built-in (dal v0.2.12). Accettano funzioni nominate o lambda.

```neba
# map — trasforma ogni elemento
map([1, 2, 3, 4], fn(x) => x * 2)                  # [2, 4, 6, 8]
map([1, 2, 3], fn(x) => str(x))                    # ["1", "2", "3"]

# filter — seleziona elementi
filter([1,2,3,4,5,6], fn(x) => x % 2 == 0)         # [2, 4, 6]
filter(["hi","hello","ok"], fn(s) => len(s) > 2)   # ["hello"]

# reduce — piega l'array
reduce([1,2,3,4,5], fn(acc, x) => acc + x)          # 15
reduce([1,2,3,4,5], fn(acc, x) => acc * x)          # 120
reduce([1,2,3], fn(acc, x) => acc + x, 10)          # 16

# Pipeline composta
reduce(
    map(
        filter([1,2,3,4,5,6], fn(x) => x % 2 == 0),
        fn(x) => x * x
    ),
    fn(acc, x) => acc + x
)
# → 4 + 16 + 36 = 56

# Con funzione nominata
fn is_positive(n: Int) -> Bool
    return n > 0

filter([-3, -1, 0, 2, 5], is_positive)   # [2, 5]
```

### Regole

- `map(array, fn)` → Array della stessa lunghezza
- `filter(array, fn)` → Array con lunghezza ≤ originale
- `reduce(array, fn)` → richiede array non vuoto senza `initial`
- `reduce(array, fn, initial)` → funziona anche su array vuoto
- Le lambda catturano variabili esterne come closure
- Chiamate multi-riga supportate (parentesi aperte sopprimono i newline):

```neba
var risultato = map(
    range(100),
    fn(x) => x * x
)
```

---

## Prossimamente (v0.3.x)

- **Cranelift JIT** — compilazione nativa delle funzioni hot
- **Hindley-Milner type inference** — inferenza di tipo completa
- **GC concurrent v2** — pause ridotte in ambienti multi-thread
- **Profiler v1** — flamegraph, rilevamento funzioni hot
- **Benchmark suite v2** — confronto bytecode vs JIT
