The tree-walk interpreter is now finished.

I think I already got enough out of the parts of the compiler I implemented so far, so I decided to start another PL project instead of continuing with the Lox VM, as soon as I finish integrating all tests from the book's repository and make sure they all pass. I will only keep the AST interpreter in this repository.

Thank you Bob Nystrom!

## Usage instructions

### Run a lox file

```
cargo run --release --bin lox -- my-script.lox
```

### Run the tests

```
cargo test
```
or
```
cargo test --release
```

### Run the benchmarks

```
cargo run --release --bin benchmark
```