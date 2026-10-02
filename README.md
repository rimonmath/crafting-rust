<div align="center">

# 🦀 Crafting Rust

### _From First Principles to a Production-Ready Engine_

A comprehensive, project-based Rust engineering curriculum. Master memory safety, fearless concurrency, async runtimes, REST APIs, metaprogramming, and safe systems abstractions by building a real-world engine from scratch.

[![VitePress](https://img.shields.io/badge/Docs-VitePress-646CFF?style=for-the-badge&logo=vite&logoColor=white)](https://rimonmath.github.io/crafting-rust/)
[![Rust](https://img.shields.io/badge/Rust-2024%20%2F%202021_Edition-dea584?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/Unit_Tests-63%2F63_Passing-success?style=for-the-badge&logo=checkmarx&logoColor=white)](https://github.com/rimonmath/crafting-rust)
[![Binary Size](<https://img.shields.io/badge/Release_Binary-%3C5MB_(4.6MB)-success?style=for-the-badge&logo=rust>)](ministore/)
[![Examples](https://img.shields.io/badge/Chapter_Examples-26_Crates-blue?style=for-the-badge)](https://github.com/rimonmath/crafting-rust/tree/main/examples)
[![Bilingual](https://img.shields.io/badge/Language-English_%7C_%E0%A6%AC%E0%A6%BE%E0%A6%82%E0%A6%B2%E0%A6%BE-orange?style=for-the-badge)](#curriculum)

[📖 Read Online (English)](https://rimonmath.github.io/crafting-rust/en/) &nbsp;•&nbsp;
[📖 বাংলায় পড়ুন (Bengali)](https://rimonmath.github.io/crafting-rust/bn/) &nbsp;•&nbsp;
[🗺️ Full Curriculum Outline](https://rimonmath.github.io/crafting-rust/en/outline) &nbsp;•&nbsp;
[📦 Codebase Tour](ministore/)

---

</div>

## 🌟 Why Crafting Rust?

Most programming books teach language syntax in isolation with toy examples like `Animal::speak()` or `calculate_fibonacci(10)`. When developers attempt to build real backend services or system tools, they hit a brick wall when confronted with ownership, lifetime annotations, thread synchronization, and async runtimes.

**Crafting Rust** takes the opposite approach:

- **Project-Driven Engineering**: You build **MiniStore**—a production-ready, modular commerce and inventory engine.
- **Ultra-Lean Footprint (<5MB)**: The complete production engine—including the Tokio runtime, Axum HTTP REST server, Serde JSON persistence, and telemetry—compiles into a standalone release binary of **less than 5MB (only ~4.6MB)** with zero external runtime dependencies.
- **First Principles**: Every concept is introduced because our application genuinely requires it to solve a concrete architectural challenge.
- **Zero to Production**: Starts from scalar types and ownership, progressing through multithreaded order workers, Tokio async pipelines, Axum REST web services, structured tracing, declarative macros, and safe abstractions over raw memory buffers.
- **Bilingual by Design**: Fully written with equal depth and parity in both **English** and **বাংলা (Bengali)**.
- **100% Tested & Compilable**: Every single chapter has a dedicated, runnable standalone crate inside [`examples/`](examples/) verified by automated tests.

---

## 🧭 The Teaching Philosophy

```
┌──────────┐     ┌──────────┐     ┌──────────────┐     ┌───────────┐     ┌──────────┐
│ Problem  │ ──> │   Need   │ ──> │ Rust Concept │ ──> │ MiniStore │ ──> │ Automated│
│ (Crisis) │     │ (Design) │     │ (The Tool)   │     │ Milestone │     │  Tests   │
└──────────┘     └──────────┘     └──────────────┘     └───────────┘     └──────────┘
```

1. **Problem**: We face a real software defect or limitation (e.g., race conditions, mutable state corruption, unhandled errors, memory bloat).
2. **Need**: We determine what architectural guarantee is missing.
3. **Rust Concept**: We explore the idiomatic Rust mechanism designed to guarantee safety at zero runtime cost.
4. **MiniStore Integration**: We incorporate the concept directly into the engine codebase.
5. **Validation**: We verify our implementation with rigorous unit and integration tests.

---

## 📚 Curriculum & Chapter Directory

| #            | Chapter Title                 | Core Rust Concepts                                                             |                           English                            |                            বাংলা                             |              Standalone Example              |
| ------------ | :---------------------------- | :----------------------------------------------------------------------------- | :----------------------------------------------------------: | :----------------------------------------------------------: | :------------------------------------------: |
| **Part 0**   | **Starting the Journey**      |                                                                                |                                                              |                                                              |                                              |
| 01           | Why Rust? How We'll Learn     | Affine types, zero-cost abstractions, compiler pair programmer                 |           [Read](docs/en/chapters/01-why-rust.md)            |           [পড়ুন](docs/bn/chapters/01-why-rust.md)            | [`examples/chapter-01`](examples/chapter-01) |
| 02           | Getting Started with Rust     | `rustup`, `cargo`, project structure, compilation pipeline                     |        [Read](docs/en/chapters/02-getting-started.md)        |        [পড়ুন](docs/bn/chapters/02-getting-started.md)        | [`examples/chapter-02`](examples/chapter-02) |
| 03           | Basic Building Blocks         | Primitives, immutability, expressions, integer arithmetic                      |     [Read](docs/en/chapters/03-basic-building-blocks.md)     |     [পড়ুন](docs/bn/chapters/03-basic-building-blocks.md)     | [`examples/chapter-03`](examples/chapter-03) |
| 04           | Control Flow                  | `if/else` expressions, `loop`, `while`, `for`, `break` with value              |         [Read](docs/en/chapters/04-control-flow.md)          |         [পড়ুন](docs/bn/chapters/04-control-flow.md)          | [`examples/chapter-04`](examples/chapter-04) |
| 05           | Structs: Modeling Real Things | Classic structs, tuple structs, `impl`, associated methods                     |            [Read](docs/en/chapters/05-structs.md)            |            [পড়ুন](docs/bn/chapters/05-structs.md)            | [`examples/chapter-05`](examples/chapter-05) |
| **Part I**   | **Ownership & Memory**        |                                                                                |                                                              |                                                              |                                              |
| 06           | Ownership                     | Move semantics, stack vs heap allocation, RAII, Drop trait                     |           [Read](docs/en/chapters/06-ownership.md)           |           [পড়ুন](docs/bn/chapters/06-ownership.md)           | [`examples/chapter-06`](examples/chapter-06) |
| 07           | Borrowing & References        | Shared references (`&T`), mutable references (`&mut T`), aliasing rules        |   [Read](docs/en/chapters/07-borrowing-and-references.md)    |   [পড়ুন](docs/bn/chapters/07-borrowing-and-references.md)    | [`examples/chapter-07`](examples/chapter-07) |
| 08           | Strings, Slices & Ownership   | `String` vs `&str`, fat pointers, zero-copy slicing, UTF-8 safety              | [Read](docs/en/chapters/08-strings-slices-and-ownership.md)  | [পড়ুন](docs/bn/chapters/08-strings-slices-and-ownership.md)  | [`examples/chapter-08`](examples/chapter-08) |
| 09           | Collections                   | Heap vectors (`Vec<T>`), hash maps (`HashMap<K, V>`), `Entry` API              |          [Read](docs/en/chapters/09-collections.md)          |          [পড়ুন](docs/bn/chapters/09-collections.md)          | [`examples/chapter-09`](examples/chapter-09) |
| **Part II**  | **Modeling Business Logic**   |                                                                                |                                                              |                                                              |                                              |
| 10           | Enums & Pattern Matching      | Tagged unions, data-carrying variants, exhaustive `match`                      |  [Read](docs/en/chapters/10-enums-and-pattern-matching.md)   |  [পড়ুন](docs/bn/chapters/10-enums-and-pattern-matching.md)   | [`examples/chapter-10`](examples/chapter-10) |
| 11           | Option: Vanquishing Null      | `Option<T>`, `Some`/`None`, combinators (`map`, `and_then`), `if let`          |            [Read](docs/en/chapters/11-option.md)             |            [পড়ুন](docs/bn/chapters/11-option.md)             | [`examples/chapter-11`](examples/chapter-11) |
| 12           | Result & Error Handling       | Recoverable errors, `Result<T, E>`, `?` operator, custom Error enums           |   [Read](docs/en/chapters/12-result-and-error-handling.md)   |   [পড়ুন](docs/bn/chapters/12-result-and-error-handling.md)   | [`examples/chapter-12`](examples/chapter-12) |
| 13           | Modules & Project Structure   | Encapsulation, `pub`, privacy boundaries, multi-file codebases                 | [Read](docs/en/chapters/13-modules-and-project-structure.md) | [পড়ুন](docs/bn/chapters/13-modules-and-project-structure.md) | [`examples/chapter-13`](examples/chapter-13) |
| **Part III** | **Making Rust Code Powerful** |                                                                                |                                                              |                                                              |                                              |
| 14           | Generics                      | Type abstraction, monomorphization, generic structs & methods                  |           [Read](docs/en/chapters/14-generics.md)            |           [পড়ুন](docs/bn/chapters/14-generics.md)            | [`examples/chapter-14`](examples/chapter-14) |
| 15           | Traits                        | Shared interfaces, trait bounds, orphan rule, dynamic dispatch (`dyn`)         |            [Read](docs/en/chapters/15-traits.md)             |            [পড়ুন](docs/bn/chapters/15-traits.md)             | [`examples/chapter-15`](examples/chapter-15) |
| 16           | Lifetimes                     | Static borrow checking, lifetime parameters (`'a`), elision rules              |           [Read](docs/en/chapters/16-lifetimes.md)           |           [পড়ুন](docs/bn/chapters/16-lifetimes.md)           | [`examples/chapter-16`](examples/chapter-16) |
| 17           | Iterators                     | Zero-cost abstractions, `Iterator` trait, lazy chains (`map`, `filter`)        |           [Read](docs/en/chapters/17-iterators.md)           |           [পড়ুন](docs/bn/chapters/17-iterators.md)           | [`examples/chapter-17`](examples/chapter-17) |
| 18           | Closures                      | Anonymous functions, closure traits (`Fn`, `FnMut`, `FnOnce`), `move`          |           [Read](docs/en/chapters/18-closures.md)            |           [পড়ুন](docs/bn/chapters/18-closures.md)            | [`examples/chapter-18`](examples/chapter-18) |
| 19           | Smart Pointers                | `Box<T>`, `Rc<T>`, `Arc<T>`, interior mutability (`RefCell<T>`)                |        [Read](docs/en/chapters/19-smart-pointers.md)         |        [পড়ুন](docs/bn/chapters/19-smart-pointers.md)         | [`examples/chapter-19`](examples/chapter-19) |
| **Part IV**  | **Concurrency & Async**       |                                                                                |                                                              |                                                              |                                              |
| 20           | Fearless Concurrency          | OS threads, `std::sync::mpsc` channels, shared state via `Arc<Mutex<T>>`       |          [Read](docs/en/chapters/20-concurrency.md)          |          [পড়ুন](docs/bn/chapters/20-concurrency.md)          | [`examples/chapter-20`](examples/chapter-20) |
| 21           | Async Rust                    | Tokio runtime, cooperative multitasking, `.await`, `tokio::join!`              |          [Read](docs/en/chapters/21-async-rust.md)           |          [পড়ুন](docs/bn/chapters/21-async-rust.md)           | [`examples/chapter-21`](examples/chapter-21) |
| **Part V**   | **Application Layer**         |                                                                                |                                                              |                                                              |                                              |
| 22           | Persistence in Rust           | Serde JSON serialization, atomic temporary file swaps, durability              |          [Read](docs/en/chapters/22-persistence.md)          |          [পড়ুন](docs/bn/chapters/22-persistence.md)          | [`examples/chapter-22`](examples/chapter-22) |
| 23           | Building a Rust Web API       | Axum HTTP server, state injection, REST routing, status codes                  |            [Read](docs/en/chapters/23-web-api.md)            |            [পড়ুন](docs/bn/chapters/23-web-api.md)            | [`examples/chapter-23`](examples/chapter-23) |
| 24           | Production Rust               | Structured logging (`tracing`), health/readiness probes, graceful shutdown     |        [Read](docs/en/chapters/24-production-rust.md)        |        [পড়ুন](docs/bn/chapters/24-production-rust.md)        | [`examples/chapter-24`](examples/chapter-24) |
| **Part VI**  | **Advanced Rust & Beyond**    |                                                                                |                                                              |                                                              |                                              |
| 25           | Declarative Macros            | Metaprogramming with `macro_rules!`, designators (`$ident`, `$expr`, `$block`) |      [Read](docs/en/chapters/25-declarative-macros.md)       |      [পড়ুন](docs/bn/chapters/25-declarative-macros.md)       | [`examples/chapter-25`](examples/chapter-25) |
| 26           | Unsafe & Safe Abstractions    | Raw pointers (`*const`, `*mut`), `std::alloc`, safe buffer abstractions        |          [Read](docs/en/chapters/26-unsafe-rust.md)          |          [পড়ুন](docs/bn/chapters/26-unsafe-rust.md)          | [`examples/chapter-26`](examples/chapter-26) |
| 27           | Conclusion & Beyond           | Course summary, Rust ecosystem map, career & systems mastery                   |     [Read](docs/en/chapters/27-conclusion-and-beyond.md)     |     [পড়ুন](docs/bn/chapters/27-conclusion-and-beyond.md)     |        [Outlines](docs/en/outline.md)        |

---

## 🛠️ Project Structure

```text
crafting-rust/ (or crafting-rust/)
├── ministore/                  # The complete, production-grade engine crate
│   ├── Cargo.toml
│   └── src/
│       ├── models/             # Domain entities (Product, Customer, Cart, Order, Page, Receipt, Session)
│       │   ├── mod.rs
│       │   ├── product.rs
│       │   ├── customer.rs
│       │   ├── cart.rs
│       │   ├── order.rs
│       │   ├── page.rs
│       │   ├── receipt.rs
│       │   └── session.rs
│       ├── async_services.rs   # Async checkout pipeline & mock payment client
│       ├── catalog.rs          # Inventory catalog & pagination queries
│       ├── checkout.rs         # Transactional checkout & stock reduction
│       ├── concurrency.rs      # Thread pools, mpsc channels & Arc<Mutex<SalesMetrics>>
│       ├── config.rs           # 12-factor configuration & environment loading
│       ├── error.rs            # Custom domain StoreError enum & Result types
│       ├── macros.rs           # Domain-specific declarative macros (product!, cart!, etc.)
│       ├── persistence.rs      # Atomic file persistence & snapshot backup engine
│       ├── promotions.rs       # Dynamic closure promotion pipelines & auditor
│       ├── traits.rs           # Shared behaviors (Taxable, Summarizable)
│       ├── unsafe_utils.rs     # Safe high-speed byte buffer over raw memory
│       ├── web_api.rs          # Axum HTTP routes, handlers & state injection
│       ├── lib.rs              # Public library export & 63 unit/integration tests
│       └── main.rs             # CLI application & web server entrypoint
│
├── examples/                   # 26 Standalone, compilable chapter crates
│   ├── chapter-01/             # From Chapter 1 foundational code...
│   └── ... chapter-26/         # ...to Chapter 26 unsafe memory buffer demo
│
└── docs/                       # Bilingual VitePress documentation
    ├── en/                     # Full English edition (Chapters 1-27)
    └── bn/                     # Full Bengali edition (Chapters 1-27)
```

---

## 🚀 Quickstart & Development

### 1. Run the Documentation Site Locally

```bash
# Clone the repository
git clone https://github.com/rimonmath/crafting-rust.git
cd crafting-rust

# Install documentation dependencies
npm install

# Start the VitePress live-reload server
npm run docs:dev
```

Now navigate to `http://localhost:5173/` in your browser.

### 2. Build, Test & Run MiniStore

The core engine resides in the `ministore/` crate:

```bash
# Navigate to the ministore engine directory
cd ministore

# 1. Build development binary (debug)
cargo build

# 2. Build optimized production binary (release) — produces an ultra-lean <5MB (~4.6MB) standalone binary
cargo build --release

# 3. Run all 63 unit and integration tests (+ 5 doc-tests)
cargo test

# 4. Run the 20-step complete architectural showcase tour
cargo run

# 5. Launch the live, long-running HTTP Web Server (Axum REST API on port 3000)
cargo run -- server

# Alternatively, build, test, or run directly from the repo root via --manifest-path:
cargo build --manifest-path ministore/Cargo.toml
cargo build --release --manifest-path ministore/Cargo.toml
cargo test --manifest-path ministore/Cargo.toml
cargo run --manifest-path ministore/Cargo.toml
cargo run --manifest-path ministore/Cargo.toml -- server

# Run the compiled binary directly without cargo:
./target/release/ministore server
```

### 3. Run Standalone Chapter Examples

Each chapter contains an isolated, self-contained example crate in `examples/`:

```bash
# Example: Run Chapter 20 (Concurrency)
cd examples/chapter-20
cargo run

# Example: Run Chapter 23 (Axum Web API)
cd examples/chapter-23
cargo run

# Example: Run Chapter 26 (Unsafe Rust & FFI)
cd examples/chapter-26
cargo run

# Alternatively, run directly from the repo root via --manifest-path:
cargo run --manifest-path examples/chapter-20/Cargo.toml
```

### 4. Build Static Documentation Bundle

```bash
npm run docs:build
```

---

## 🤝 Contributing & Community

Contributions, typo fixes, and community translations are warmly welcome!

1. Fork the Project.
2. Create your Feature Branch (`git checkout -b feature/AmazingExplanation`).
3. Commit your Changes (`git commit -m 'Add awesome explanation'`).
4. Push to the Branch (`git push origin feature/AmazingExplanation`).
5. Open a Pull Request.

---

## ✍️ Author

**Mamunur Rashid** ([@rimonmath](https://github.com/rimonmath))

- **Creator & Author**: Creator of *Crafting Rust*, passionate about systems programming, clean backend architectures, and practical developer education.
- **GitHub**: [@rimonmath](https://github.com/rimonmath)
- **Email**: [rimonmath@gmail.com](mailto:rimonmath@gmail.com)

---

## 📄 License

This book and all accompanying code examples are open source and available under the [MIT License](LICENSE).
