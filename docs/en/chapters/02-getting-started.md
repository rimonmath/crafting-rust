# Chapter 2: Getting Started with Rust

## What You'll Learn
- How the official Rust toolchain is structured: `rustup`, `rustc`, and `cargo`.
- How Cargo solves the historical "build system and package manager hell."
- Anatomy of a Cargo project: `Cargo.toml`, `Cargo.lock`, and `src/`.
- The developer feedback loop: why `cargo check` is faster than `cargo build`.
- Bootstrapping our actual **MiniStore** project and running its first test.

---

## Why Do We Need This?
Before writing business logic, production engineering requires a reproducible, deterministic build environment.

In languages like C and C++, managing builds and dependencies across platforms requires orchestrating Makefiles, CMake, pkg-config, Conan, or vcpkg. In Python and JavaScript ecosystems, tools are often fragmented across multiple competing runners (`pip`, `poetry`, `conda`, `npm`, `yarn`, `pnpm`, `webpack`, `vite`).

Rust was designed with a first-class, batteries-included philosophy. Every Rust installation includes a standardized build orchestrator, package manager, documentation generator, formatter, linter, and test harness out of the box: **Cargo**.

---

## The Problem
Modern software development suffers when different engineers on a team have subtle differences in compilers, toolchains, or dependency resolution.

Consider common pain points:
- *"It works on my machine, but fails in CI."*
- Heavy build times slowing down the inner development feedback loop.
- Needing separate third-party libraries (e.g., JUnit, Jest, PyTest) just to write a basic unit test.

Rust eliminates this friction through a unified three-pillar architecture.

---

## Rust Concept: The Toolchain Trinity

```
┌─────────────────────────────────────────────────────────┐
│                         rustup                          │
│        (Toolchain installer, updater & version manager) │
└────────────────────────────┬────────────────────────────┘
                             │ installs & updates
            ┌────────────────┴────────────────┐
            ▼                                 ▼
┌───────────────────────┐         ┌───────────────────────┐
│         cargo         │         │         rustc         │
│  (Package manager &   │────────▶│ (The compiler backend │
│    build orchestrator)│ invokes │    powered by LLVM)   │
└───────────────────────┘         └───────────────────────┘
```

1. **`rustup`**: The toolchain multiplexer (similar to `nvm` or `pyenv`, but official). It manages compiler versions (stable, beta, nightly) and cross-compilation targets (e.g., compiling for Linux x86_64, ARM64, or WebAssembly).
2. **`rustc`**: The Rust compiler. It parses Rust syntax, enforces borrow checker rules, translates code into LLVM Intermediate Representation (IR), and emits native machine code. You rarely invoke `rustc` directly.
3. **`cargo`**: The high-level command-line tool you interact with 99% of the time. It handles dependency resolution, compiles packages, runs tests, and drives benchmarks.

---

## Coming From Other Languages

| Task | Node.js / TS | Go | Python | C / C++ | Rust (Cargo) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Install/Manage Toolchain** | `nvm` / `fnm` | OS Package / `gvm` | `pyenv` / `asdf` | OS Package Manager | `rustup` |
| **Package Manifest** | `package.json` | `go.mod` | `pyproject.toml` | `CMakeLists.txt` | `Cargo.toml` |
| **Lockfile** | `package-lock.json` | `go.sum` | `poetry.lock` | None (manual) | `Cargo.lock` |
| **Fast Type Check** | `tsc --noEmit` | `go vet` | `mypy` | `clang -fsyntax-only` | `cargo check` |
| **Run Tests** | `npm test` (Jest/Vitest) | `go test ./...` | `pytest` | CTest / GoogleTest | `cargo test` |
| **Format Code** | `prettier` / `biome` | `gofmt` | `black` / `ruff` | `clang-format` | `cargo fmt` |
| **Linter** | `eslint` | `golangci-lint` | `flake8` / `ruff` | `clang-tidy` | `cargo clippy` |

---

## Small Example: The Fast Feedback Loop

When developing in Rust, you do not want to wait for full machine code generation every time you save a file.

Rust provides `cargo check`:
```bash
# Full build: Checks code + compiles LLVM IR + links machine binary (slower)
cargo build

# Syntax & Type check: Validates types, ownership, and borrow rules WITHOUT linking binary (very fast)
cargo check
```

Experienced Rust developers keep `cargo check` running continuously in their terminal or IDE for instant sub-second compiler feedback.

---

## Apply To MiniStore
We now initialize the `ministore` application. MiniStore is structured as an application binary package.

To initialize a new binary project:
```bash
cargo new ministore --bin
```

This creates the project skeleton:
```
ministore/
├── Cargo.toml       # Manifest defining package metadata and dependencies
└── src/
    └── main.rs      # The entry point of the binary application
```

---

## Code

### `ministore/Cargo.toml`
```toml
[package]
name = "ministore"
version = "0.1.0"
edition = "2021"

[dependencies]
# We keep dependencies empty for now — we master pure Rust first!
```

### `ministore/src/main.rs`
```rust
fn main() {
    println!("=== MiniStore ===");
    println!("Version: 0.1.0");
    println!("Status: Initialized");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_initialization() {
        let app_name = "MiniStore";
        assert_eq!(app_name, "MiniStore");
    }
}
```

---

## Understanding The Code

### 1. The `println!` Macro
Notice the exclamation mark in `println!(...)`. In Rust, an exclamation mark indicates a **macro invocation**, not a standard function call.

Why is `println!` a macro?
In C, `printf("%s %d", str)` does not verify types at compile time; passing the wrong argument type can cause memory corruption or crashes. In Rust, `println!` inspects the format string **at compile time**, guaranteeing that your arguments match the specifiers before any binary is produced.

### 2. Built-in Testing with `#[cfg(test)]`
- `#[cfg(test)]`: A conditional compilation attribute. The Rust compiler compiles this test module only when running `cargo test`, ensuring your production binary is not bloated with test code.
- `#[test]`: Marks an individual function as a runnable test case.
- `assert_eq!`: Another macro that compares two values and prints a clear diff if they don't match.

---

## Common Mistakes

### 1. Committing the `target/` Directory
When Cargo compiles your code, intermediate artifacts and binaries are stored in `target/`. This directory can grow to hundreds of megabytes. Always ensure `target/` is in your `.gitignore`.

### 2. Manually Editing `Cargo.lock`
- `Cargo.toml`: Written by **you**. Defines semver constraints for direct dependencies.
- `Cargo.lock`: Written by **Cargo**. Pinpoints the exact cryptographically hashed versions of every transitive dependency for deterministic builds. Never edit it manually.

---

## Compiler Errors
What happens when you provide the wrong number of arguments to `println!`?

```rust
fn main() {
    let name = "MiniStore";
    println!("Welcome to {}, Version {}", name); // Missing second argument!
}
```

Because `println!` is a compile-time macro, `rustc` catches this immediately:

```text
error: 2 positional arguments in format string, but there is 1 argument
 --> src/main.rs:3:14
  |
3 |     println!("Welcome to {}, Version {}", name);
  |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^  ----
  |              |                            |
  |              |                            this is the 1st argument
  |              positional arguments 1 and 2 are referenced here, but only 1 argument was provided
```

In other languages, format string mismatches either fail silently or crash at runtime. In Rust, it is rejected before the code can even run.

---

## Practice
Run these commands inside your `ministore/` folder:

1. Fast check:
   ```bash
   cargo check
   ```
2. Run the test suite:
   ```bash
   cargo test
   ```
3. Run the binary application:
   ```bash
   cargo run
   ```
4. Check code formatting:
   ```bash
   cargo fmt -- --check
   ```

---

## Checkpoint
- [x] Installed and understood `rustup`, `rustc`, and `cargo`.
- [x] Initialized the `ministore` Cargo project.
- [x] Mastered the difference between `cargo check`, `cargo build`, and `cargo run`.
- [x] Understood why `println!` is a macro and verified built-in unit tests.

---

## What We Learned
- Cargo is a unified toolchain eliminating the fragmentation seen in other ecosystems.
- `cargo check` provides near-instant type and borrow validation.
- Rust macros (like `println!` and `assert_eq!`) provide compile-time safety that ordinary functions cannot.

---

## What's Next
In **Chapter 3: Rust's Basic Building Blocks**, we will begin modeling MiniStore data by exploring variables, mutability, scalar types, compound types, and variable shadowing.
