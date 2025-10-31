# 🚀 bf-rs

`bf-rs` is intended to be a high-performance Rust toolkit for Brainfuck, featuring a compile-time interpreter and an optimizing transpiler, both implemented as procedural macros.
This was mostly a learning project for me and proc macros.

This project provides two powerful macros:

* **`const_bf!`:** A compile-time **interpreter** that executes Brainfuck code during your build and embeds the final `String` result directly into your binary.

* **`bf!`:** A compile-time **transpiler** that converts Brainfuck code into highly optimized, native Rust code, which is then compiled by LLVM.

## ✨ Features

* **Dual-Mode Operation:** Choose between compile-time interpretation or native code transpilation.

* **Flexible Parsing:** Accepts Brainfuck code as raw tokens (`+++`), *almost* string literals (`"..."`), or from external files via `include_str!`.

* **Configurable Environment:** This doesn't work yet: Set the cell type (e.g., `u8`), these do: total memory cells, and negative pointer space.

* **Extremely Performant:** The `bf!` transpiler is blisteringly fast, leveraging LLVM's optimizer to achieve massive speedups over interpreted code.

## 📦 Crate

The macros are provided by the `bf_macros` crate.

Add this to your `Cargo.toml`:
```toml
[dependencies]
bf_macros = { path = "path/to/bf_macros" } # Or from crates.io later.
```
## Quick Start

### 1. `const_bf!` (The Interpreter)

Use this when you want to run Brainfuck code at compile-time and store **only the result** in your program. This is perfect for compile-time generation of data.
```rust
use bf_macros::const_bf;
fn main() {
    // This BF code is executed by the compiler.
    // The result variable will just be a "Hello World!\n" string.
    let result = const_bf!(<u8, 30000, 0>
        ++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.
        >---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
    );
    println!("{}", result);
    assert_eq!(result, "Hello World!\n");
}
```
You can also include code from an external file:

use bf_macros::const_bf;let hello_from_file = const_bf!(<u8, 30000, 0>include_str!("hello.b"));
### 2. `bf!` (The Transpiler)

Use this when you want to compile your Brainfuck code into a native, executable part of your program. The macro expands into optimized Rust code that will run when your *users* run the program.
```rust
use bf_macros::bf;

fn main() {
    // This BF code is translated into Rust code,
    // which is then compiled.
    // The code runs when main() is called.
    let result = bf!(<u8, 30000, 0>
      ++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
    );
    println!("{}", result);
}
```
## ⚙️ Macro Configuration

Both macros use the same configuration format:

`macro_name!(<cell_type, cells, negative_ptrs> "...")`

* **`cell_type`:** The data type for each memory cell (e.g., `u8`, `u16`, `i8`).

* **`cells`:** The total number of cells to allocate for the main memory tape (e.g., `30000`).

* **`negative_ptrs`:** The number of "negative" cells to allocate, allowing the pointer to move left from the starting position without panicking. The pointer will start at index `negative_ptrs`.

## 🔥 Performance

This toolkit is built for speed. The `bf!` transpiler, in particular, demonstrates the power of LLVM optimizations.

A complex test program (the "counter killer" script) that performs **5.3 billion** operations was benchmarked:

* **`const_bf!` (Interpreter):**

    * **Time:** \~58.2 seconds (at *compile-time*)

    * **Result:** The interpreter executed all 5.3 billion steps during the build.

* **`bf!` (Transpiler):**

    * **Time:** \~546.5 milliseconds (at *run-time*, compiled in release mode)

    * **Result:** The transpiler generated simple Rust code, which LLVM then optimized into native machine code.

This **\~107x speedup** is achieved because the transpiler exposes the program's logic directly to the Rust/LLVM optimizer, which can combine operations (`+++` becomes `*cell += 3`), unroll loops, and use CPU registers for maximum efficiency.

## 🔬 Building & Testing

This project is a Cargo workspace.

* **To run tests:**
`cargo tests`
* **To run benchmarks (using Criterion):**
`cargo bench`

## 🗺️ Roadmap

* \[ \] Fix String Literal token parsing.

* \[ \] Make the u8 in the generic vector actually change cell size.

* \[ \] Add optimizations like Run-Length Encoding (RLE) to the `const_bf!` interpreter to speed up compile-time execution.

* \[ \] Publish to `crates.io`.
