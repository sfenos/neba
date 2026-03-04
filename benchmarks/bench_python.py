import time, math, random

def bench(name, fn, *args):
    t0 = time.perf_counter()
    r = fn(*args)
    t1 = time.perf_counter()
    ms = (t1-t0)*1000
    print(f"  {name}: {ms:.1f}ms  (result={r})")
    return ms

print("=== Python 3.12 Benchmarks ===")

# B1: Fibonacci ricorsivo
def fib(n):
    if n <= 1: return n
    return fib(n-1) + fib(n-2)
bench("B1 fib(35)", fib, 35)

# B2: Int loop 10M
def int_loop(n):
    s = 0
    for i in range(n):
        s += i
    return s
bench("B2 int_loop(10M)", int_loop, 10_000_000)

# B3: Float loop 5M
def float_loop(n):
    s = 0.0
    for i in range(n):
        s += i * 0.5
    return s
bench("B3 float_loop(5M)", float_loop, 5_000_000)

# B4: For range 1M
def range_loop(n):
    s = 0
    for i in range(n):
        s += 1
    return s
bench("B4 range_loop(1M)", range_loop, 1_000_000)

# B5: Closure / HOF
def make_adder(x):
    def add(y): return x + y
    return add
def closure_bench(n):
    adder = make_adder(1)
    s = 0
    for _ in range(n):
        s = adder(s)
    return s
bench("B5 closure(500k)", closure_bench, 500_000)

# B6: Dict ops
def dict_bench(n):
    d = {}
    for i in range(n):
        d[str(i)] = i
    s = 0
    for i in range(n):
        s += d[str(i)]
    return s
bench("B6 dict(100k)", dict_bench, 100_000)

# B7: OOP
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def dist(self):
        return math.sqrt(self.x*self.x + self.y*self.y)

def oop_bench(n):
    s = 0.0
    for i in range(n):
        p = Point(float(i), float(i+1))
        s += p.dist()
    return s
bench("B7 oop(100k)", oop_bench, 100_000)

# B8: Map/Filter HOF
def hof_bench(n):
    data = list(range(n))
    doubled = list(map(lambda x: x*2, data))
    evens = list(filter(lambda x: x%2==0, doubled))
    return len(evens)
bench("B8 hof_map_filter(1M)", hof_bench, 1_000_000)

# B9: String ops
def string_bench(n):
    s = ""
    for i in range(n):
        s = str(i) + " "
    return len(s)
bench("B9 string_concat(100k)", string_bench, 100_000)

# B10: Recursion (factorial)
def fact(n):
    if n <= 1: return 1
    return n * fact(n-1)
def fact_bench(n):
    s = 0
    for _ in range(n):
        s += fact(20)
    return s
bench("B10 factorial(20)x100k", fact_bench, 100_000)
