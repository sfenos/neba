# Neba Language Reference — v0.1.x

> Versione: 0.1.9  
> Ultimo aggiornamento: 2026-02-26

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
11. [Classi](#classi)
12. [Collezioni](#collezioni)
13. [Pattern matching](#pattern-matching)
14. [Stdlib](#stdlib)

---

## Panoramica

Neba è un linguaggio di programmazione con indentazione significativa (come Python), tipizzazione graduale (opzionale), e performance orientata al bytecode VM con JIT futuro.

Filosofia: *simple as Python, fast as C, secure as Rust, parallel as Go, powerful as Julia.*

---

## Esecuzione

```bash
# Esegui un file
neba mio_file.neba

# REPL interattivo
neba
```

---

## Sintassi base

Neba usa l'**indentazione** (4 spazi) per delimitare i blocchi — non parentesi graffe.  
I commenti iniziano con `#`.

```neba
# Questo è un commento
println("Ciao, Neba!")
```

---

## Tipi

| Tipo      | Esempio                  | Note                        |
|-----------|--------------------------|-----------------------------|
| `Int`     | `42`, `-7`, `0`          | intero a 64 bit con segno   |
| `Float`   | `3.14`, `-0.5`, `1.0`    | floating point 64 bit       |
| `Bool`    | `true`, `false`          |                             |
| `Str`     | `"ciao"`, `f"valore={x}"`| UTF-8, f-string supportate  |
| `None`    | `None`                   | assenza di valore           |
| `Array`   | `[1, 2, 3]`              | lista dinamica              |
| `Some(v)` | `Some(42)`               | valore opzionale presente   |
| `Ok(v)`   | `Ok("ok")`               | risultato di successo       |
| `Err(v)`  | `Err("errore")`          | risultato di errore         |

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

---

## Operatori

### Aritmetici
| Operatore | Significato        | Esempio       |
|-----------|--------------------|---------------|
| `+`       | addizione          | `3 + 4 → 7`  |
| `-`       | sottrazione        | `10 - 3 → 7` |
| `*`       | moltiplicazione    | `3 * 4 → 12` |
| `/`       | divisione float    | `7 / 2 → 3.5`|
| `//`      | divisione intera   | `7 // 2 → 3` |
| `%`       | modulo             | `10 % 3 → 1` |
| `**`      | potenza            | `2 ** 8 → 256`|
| `-x`      | negazione unaria   | `-5`          |

### Confronto
| Operatore | Significato          |
|-----------|----------------------|
| `==`      | uguaglianza          |
| `!=`      | disuguaglianza       |
| `<`       | minore               |
| `<=`      | minore o uguale      |
| `>`       | maggiore             |
| `>=`      | maggiore o uguale    |

### Logici
| Operatore | Significato          |
|-----------|----------------------|
| `and`     | AND logico (cortocircuito) |
| `or`      | OR logico (cortocircuito)  |
| `not`     | NOT logico           |

### Bitwise
`&`, `|`, `^`, `~`, `<<`, `>>`

### Assegnazione composta
`+=`, `-=`, `*=`, `/=`, `%=`

### Membership
```neba
3 in [1, 2, 3]       # true
"lo" in "ciao"        # true
5 not in [1, 2, 3]   # true
```

---

## Stringhe

```neba
let s = "ciao"

# Concatenazione
let saluto = s + " mondo"    # "ciao mondo"

# Ripetizione
let rip = s * 3              # "ciaociaociao"

# Lunghezza
let n = len(s)               # 4

# Indicizzazione (0-based)
let primo = s[0]             # "c"
let ultimo = s[-1]           # "o"

# f-string
let nome = "Neba"
let msg = f"Ciao {nome}!"    # "Ciao Neba!"
let calc = f"2+2={2+2}"      # "2+2=4"
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
    1       => println("uno")
    2 | 3   => println("due o tre")
    4..=9   => println("tra 4 e 9")
    Some(n) => println(f"ha valore {n}")
    None    => println("nessun valore")
    _       => println("altro")
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

### for-in (su array)

```neba
let nomi = ["Alice", "Bob", "Carlo"]
for nome in nomi
    println(f"Ciao {nome}!")
```

### break e continue

```neba
for i in 0..100
    if i == 5
        break       # esce dal loop

for i in 0..10
    if i % 2 != 0
        continue    # salta le iterazioni dispari
    println(i)
```

---

## Funzioni

```neba
# Definizione base
fn somma(a: Int, b: Int) -> Int
    return a + b

# Parametri con default
fn saluta(nome: str, prefisso: str = "Ciao") -> str
    return f"{prefisso} {nome}!"

# Chiamata
somma(3, 4)          # 7
saluta("Neba")       # "Ciao Neba!"
saluta("Neba", "Hey") # "Hey Neba!"
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
fn make_adder(n: Int)
    fn adder(x: Int) -> Int
        return x + n    # cattura 'n' dal frame padre
    return adder

let add5 = make_adder(5)
add5(3)    # 8
add5(10)   # 15
```

### Funzioni come valori

```neba
fn applica(f, x)
    return f(x)

fn doppio(n: Int) -> Int
    return n * 2

applica(doppio, 7)   # 14
```

---

## Classi

```neba
class Cerchio
    raggio: Float

    fn __init__(self, r: Float)
        self.raggio = r

    fn area(self) -> Float
        return 3.14159 * self.raggio ** 2

    fn descrivi(self)
        println(f"Cerchio con raggio {self.raggio}")


# Istanziazione
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
println(p.distanza())   # 5.0
```

---

## Collezioni

### Array

```neba
# Creazione
let numeri = [1, 2, 3, 4, 5]
let vuoto: Array = []

# Accesso (0-based)
numeri[0]       # 1
numeri[-1]      # 5

# Modifica
numeri[0] = 99

# Lunghezza
len(numeri)     # 5

# push / pop
push(numeri, 6)   # aggiunge in fondo
let ultimo = pop(numeri)  # rimuove e restituisce l'ultimo
```

### Range come array

```neba
let r = 0..5      # [0, 1, 2, 3, 4]
let r2 = 0..=5    # [0, 1, 2, 3, 4, 5]
```

---

## Pattern matching

```neba
let risultato: Ok(Int) | Err(Str) = Ok(42)

match risultato
    Ok(n)  => println(f"Successo: {n}")
    Err(e) => println(f"Errore: {e}")

# Con Some/None
let opt: Some(Int) | None = Some(7)
match opt
    Some(v) => println(f"Valore: {v}")
    None    => println("Nessun valore")

# Con wildcard
match x
    0     => println("zero")
    1..=9 => println("cifra")
    _     => println("altro")
```

---

## Stdlib

### I/O
| Funzione         | Descrizione                        |
|------------------|------------------------------------|
| `print(v)`       | stampa senza newline               |
| `println(v)`     | stampa con newline                 |
| `input(prompt?)` | legge riga da stdin                |

### Conversioni
| Funzione    | Descrizione                    |
|-------------|--------------------------------|
| `str(v)`    | converte in stringa            |
| `int(v)`    | converte in intero             |
| `float(v)`  | converte in float              |
| `bool(v)`   | converte in booleano           |

### Matematica
| Funzione       | Descrizione                          |
|----------------|--------------------------------------|
| `abs(n)`       | valore assoluto                      |
| `min(a, b)`    | minimo tra due valori (o di array)   |
| `max(a, b)`    | massimo tra due valori (o di array)  |

### Array
| Funzione         | Descrizione                        |
|------------------|------------------------------------|
| `len(a)`         | lunghezza di array o stringa       |
| `push(a, v)`     | aggiunge elemento in fondo         |
| `pop(a)`         | rimuove e restituisce l'ultimo     |
| `range(n)`       | array da 0 a n-1                   |

### Utility
| Funzione       | Descrizione                          |
|----------------|--------------------------------------|
| `typeof(v)`    | restituisce il tipo come stringa     |
| `assert(cond)` | lancia errore se cond è false        |

---

## Prossimamente (v0.2.x)

- **Traits** — interfacce e dispatch polimorfico
- **Typed arrays** — `Array[Float64]`, `Array[Int32]`
- **Error handling** — `Result[T, E]`, operatore `?`
- **Standard library** — moduli math, io, string, collections
- **JIT compilation** — Cranelift backend (v0.3.0)
