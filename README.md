# Neba

**Neba** is a high-performance, dynamically-typed interpreted language with optional static typing, designed around a single philosophy:

> *Simple as Python, fast as C, secure as Rust, parallel as Go, powerful as Julia.*

---

## Status: v0.2.32 — Bytecode VM

```
✅ Bytecode compiler + stack-based VM
✅ Full OOP: classes, traits, polymorphic dispatch
✅ Pattern matching (literals, ranges, Or-patterns, Option/Result)
✅ Closures with mutable upvalues
✅ TypedArray (Float64/Float32/Int64/Int32) + NdArray (multidimensional)
✅ Complete stdlib: math, string, io, collections, random
✅ 80+ global functions
✅ Constant folding, peephole optimizer
✅ 670+ passing tests
```

---

## Quick Start

```bash
# Build
cargo build --release

# Run a file
./target/release/neba my_script.neba

# Run tests
cargo test
```

---

## Language at a Glance

```neba
# Variables
let name = "Neba"
var counter = 0

# Functions with default args
fn greet(name: Str, prefix: Str = "Hello") -> Str
    return f"{prefix}, {name}!"

println(greet("World"))           # Hello, World!
println(greet("World", "Hey"))    # Hey, World!

# Classes and traits
trait Describable
    fn describe(self) -> Str        # abstract method (no body)

    fn print_info(self)             # default implementation
        println(self.describe())

class Point
    x: Float = 0.0
    y: Float = 0.0
    fn __init__(self, x, y)
        self.x = x
        self.y = y

impl Describable for Point
    fn describe(self) -> Str
        return f"Point({self.x}, {self.y})"

let p = Point(3.0, 4.0)
p.print_info()                     # Point(3.0, 4.0)
println(p is Point)                # true
println(p is Describable)          # true

# Pattern matching
fn classify(n)
    match n
        0       => return "zero"
        1..=9   => return "single digit"
        10..=99 => return "double digit"
        _       => return "large"

# Error handling
fn divide(a, b)
    if b == 0
        return Err("division by zero")
    return Ok(a / b)

match divide(10, 2)
    Ok(v)  => println(f"result: {v}")
    Err(e) => println(f"error: {e}")

# HOF with lambdas
let evens   = filter(1..=10, fn(x) x % 2 == 0)  # [2,4,6,8,10]
let squares = map(evens, fn(x) x * x)            # [4,16,36,64,100]
let total   = reduce(squares, fn(a,b) a + b, 0)  # 220

# TypedArray for numerical computation
let data = Float64([1.0, 2.0, 3.0, 4.0, 5.0])
println(sum(data))       # 15.0
println(mean(data))      # 3.0
println(dot(data, data)) # 55.0

# NdArray (multidimensional)
let m = nd.array([[1.0, 2.0], [3.0, 4.0]])
let t = nd.transpose(m)
let r = nd.matmul(m, t)
println(nd.sum(m, 0))    # column sums
```

---

## Global Functions (v0.2.32)

### I/O & Conversion
`print`  `println`  `input`  `str`  `int`  `float`  `bool`  `type`  `repr`

### Type Predicates
`is_int`  `is_float`  `is_str`  `is_bool`  `is_none`  `is_array`  `is_dict`

### Math
`abs`  `min`  `max`  `sum`  `mean`  `pow`  `chr`  `ord`  `hex`  `bin`  `oct`

### Array
`len`  `push`  `pop`  `append`  `insert`  `remove`  `contains`  `sort`  `reverse`
`join`  `range`  `zip`  `enumerate`  `sorted`  `any`  `all`
`flatten`  `unique`  `concat`  `slice`  `index`  `count`  `find`  `find_index`

### Dict
`keys`  `values`  `items`  `has_key`  `del_key`  `merge`  `dict_get`

### String
`upper`  `lower`  `strip`  `split`  `replace`  `find`  `index_of`  `char_at`
`starts_with`  `ends_with`  `contains`  `capitalize`  `title`  `zfill`
`pad_left`  `pad_right`  `is_digit`  `is_alpha`  `repr`

### TypedArray
`Float64`  `Float32`  `Int64`  `Int32`  `TypedArray(n, dtype)`
`zeros`  `ones`  `fill`  `linspace`  `dot`  `min_elem`  `max_elem`  `to_list`

### NdArray (`nd.*`)
`nd.array`  `nd.zeros`  `nd.ones`  `nd.reshape`  `nd.transpose`  `nd.matmul`
`nd.sum`  `nd.mean`  `nd.max`  `nd.min`  `nd.argmax`  `nd.argmin`
`nd.gt`  `nd.lt`  `nd.masked`  `nd.nonzero`  `nd.cumsum`  `nd.flatten`  `nd.slice`

### HOF
`map`  `filter`  `reduce`

---

## Stdlib Modules

| Module | Key functions |
|--------|---------------|
| `math` | `sqrt` `sin` `cos` `log` `exp` `pi` `e` `floor` `ceil` `abs` `sign` `clamp` `gcd` `lcm` `factorial` `is_nan` `is_inf` `is_finite` `random` `randint` |
| `string` | all string ops + `reverse` `char_at` `index_of` `repr` `center` `format` |
| `io` | `read_file` `write_file` `read_lines` `file_exists` `cwd` `listdir` `mkdir` |
| `io.path` | `join` `dirname` `basename` `stem` `ext` `exists` `isfile` `isdir` |
| `collections` | `chunk` `take` `drop` `product` `transpose` `first` `last` `none_of` |
| `random` | `choice` `shuffle` `seed` `sample` |

---

## Roadmap Summary

| Phase | Versions | Goal | Status |
|-------|----------|------|--------|
| 0 | v0.1.x | Lexer, Parser, Tree-walker, REPL | ✅ Done |
| 1 | v0.2.x | Bytecode VM, OOP, Stdlib, NdArray | 🔄 v0.2.32 |
| 2 | v0.3.x | VM Optimizations + Cranelift JIT | 🔜 |
| 3 | v0.4.x | Parallelism, async/await | 🔜 |
| 4 | v0.5.x | LLVM backend, GPU via wgpu | 🔜 |
| 5 | v0.6.x | C/Rust/Python FFI | 🔜 |
| 6 | v0.7.x | Package manager, LSP, tooling | 🔜 |
| 7 | v0.8.x | Full stdlib expansion | 🔜 |
| 8 | v0.9.x–v1.0 | Hardening, spec, stable release | 🔜 |

See [ROADMAP.md](ROADMAP.md) for full details.

---

## Performance (v0.2.21 benchmark gate)

| Benchmark | Neba | CPython 3.12 | Ratio |
|-----------|------|--------------|-------|
| fib(35) recursive | 6437ms | 2747ms | 2.3× slower |
| int loop 1M | 264ns/iter | ~100ns/iter | ~2.6× slower |
| TypedArray sum 1M | 1.6ms | NumPy ~0.4ms | ~4× slower |

*Target for v0.3.x JIT: within 1.5× of CPython on typical workloads.*

---

## License

MIT
