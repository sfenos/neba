# Tutti i test del workspace
cargo test

# Solo i test della VM (dove sta la suite v0.2.9)
cargo test -p neba_vm

# Solo la suite v0.2.9
cargo test -p neba_vm upvalue_v029

# Con output stampato (utile per debug)
cargo test -p neba_vm upvalue_v029 -- --nocapture

# Un singolo test specifico
cargo test -p neba_vm a1_counter_persists

cargo test -p neba_vm result_v0210

./target/debug/neba test_suite.neba
./target/debug/neba test_v0211.neba
./target/debug/neba test_v0212.neba
./target/debug/neba test_v0215.neba


