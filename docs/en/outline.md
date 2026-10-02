# Crafting Rust — Complete Book Outline

A comprehensive chapter-by-chapter directory of **Crafting Rust** (Building MiniStore), detailing the core problems solved, Rust concepts taught, domain models introduced, and runnable example crates.

---

## Part 0 — Starting the Journey

### [Chapter 1: Why Rust? How We'll Learn Rust](/en/chapters/01-why-rust)

- **Problem**: Why do traditional systems languages suffer from memory corruption, segfaults, and concurrency bugs, while garbage-collected languages sacrifice raw performance and predictability?
- **Rust Concepts**: Memory safety without a garbage collector, zero-cost abstractions, compiler as a pair programmer, affine type system.
- **MiniStore Milestone**: First mutable cart calculation logic; setting the pedagogical foundation.
- **Standalone Code**: [`examples/chapter-01`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-01)

### [Chapter 2: Getting Started with Rust](/en/chapters/02-getting-started)

- **Problem**: Setting up a modern, reproducible development environment and understanding the Rust compilation pipeline.
- **Rust Concepts**: `rustup`, `cargo`, project structure, `Cargo.toml`, dependencies, and compilation artifacts.
- **MiniStore Milestone**: Creating the initial `ministore` cargo workspace and CLI entry point.
- **Standalone Code**: [`examples/chapter-02`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-02)

### [Chapter 3: Rust's Basic Building Blocks](/en/chapters/03-basic-building-blocks)

- **Problem**: Storing prices, quantities, and flags safely without silent type coercions or accidental mutations.
- **Rust Concepts**: Primitive scalar types (`u32`, `i32`, `f64`, `bool`, `char`), immutability by default, expressions vs. statements, arithmetic safety.
- **MiniStore Milestone**: Currency representation in integer cents (`price_cents`) to prevent floating-point rounding errors.
- **Standalone Code**: [`examples/chapter-03`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-03)

### [Chapter 4: Control Flow](/en/chapters/04-control-flow)

- **Problem**: Implementing tiered discount brackets and iterative batch operations cleanly.
- **Rust Concepts**: `if/else` expressions, `loop`, `while`, `for` ranges, and returning values from `break`.
- **MiniStore Milestone**: Tiered cart discount calculator based on quantity and subtotal thresholds.
- **Standalone Code**: [`examples/chapter-04`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-04)

### [Chapter 5: Structs: Modeling Real Things](/en/chapters/05-structs)

- **Problem**: Grouping related business attributes into coherent, cohesive domain entities.
- **Rust Concepts**: Named field structs, associated functions (`::new()`), methods (`&self`, `&mut self`), struct update syntax.
- **MiniStore Milestone**: First concrete models for `Product` and `Customer`.
- **Standalone Code**: [`examples/chapter-05`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-05)

---

## Part I — Ownership & Memory

### [Chapter 6: Ownership](/en/chapters/06-ownership)

- **Problem**: How can software allocate and deallocate heap memory automatically without pauses (GC) or memory leaks (manual free)?
- **Rust Concepts**: Stack vs. Heap, the 3 Rules of Ownership, affine move semantics, `Copy` vs. `Clone`.
- **MiniStore Milestone**: Passing products and cart items safely between functions with zero dangling memory.
- **Standalone Code**: [`examples/chapter-06`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-06)

### [Chapter 7: Borrowing and References](/en/chapters/07-borrowing-and-references)

- **Problem**: How to let multiple parts of a program read or update store data without transferring ownership or creating expensive copies.
- **Rust Concepts**: Shared references (`&T`), mutable references (`&mut T`), the Aliasing XOR Mutability rule, non-lexical lifetimes.
- **MiniStore Milestone**: Updating cart items in place and calculating totals via borrowed references.
- **Standalone Code**: [`examples/chapter-07`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-07)

### [Chapter 8: Strings, Slices and Ownership in Practice](/en/chapters/08-strings-slices-and-ownership)

- **Problem**: Parsing SKUs, department codes, and formatting customer contact info without allocating unnecessary strings.
- **Rust Concepts**: `String` (growable heap buffer) vs. `&str` (borrowed string slice), UTF-8 character boundaries, byte indexing safety.
- **MiniStore Milestone**: Zero-copy SKU department parsing (`&sku[..3]`) and safe text truncation.
- **Standalone Code**: [`examples/chapter-08`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-08)

### [Chapter 9: Collections](/en/chapters/09-collections)

- **Problem**: Managing dynamic lists of cart items, fast inventory lookups by SKU, and deduplicating customer tags.
- **Rust Concepts**: `Vec<T>`, `HashMap<K, V>`, `HashSet<T>`, and the Entry API (`.entry().or_insert()`).
- **MiniStore Milestone**: Implementing `Catalog` with $O(1)$ SKU index lookups and customer tag deduplication.
- **Standalone Code**: [`examples/chapter-09`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-09)

---

## Part II — Modeling Business Logic

### [Chapter 10: Enums and Pattern Matching](/en/chapters/10-enums-and-pattern-matching)

- **Problem**: Representing complex order states and payment methods where each variant carries different data payloads.
- **Rust Concepts**: Algebraic Data Types (enums with data), exhaustive `match`, match guards, `if let` / `let else`.
- **MiniStore Milestone**: Order state machine (`OrderStatus`) and payment methods (`PaymentMethod` with transaction references).
- **Standalone Code**: [`examples/chapter-10`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-10)

### [Chapter 11: Option](/en/chapters/11-option)

- **Problem**: Eliminating the "Billion-Dollar Mistake" (null pointer exceptions) when items or coupons may or may not exist.
- **Rust Concepts**: `Option<T>` (`Some` vs. `None`), combinators (`.map()`, `.and_then()`, `.unwrap_or()`), `.take()`.
- **MiniStore Milestone**: Optional customer phone numbers and one-time promotional coupons via `Option::take()`.
- **Standalone Code**: [`examples/chapter-11`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-11)

### [Chapter 12: Result and Error Handling](/en/chapters/12-result-and-error-handling)

- **Problem**: Handling inventory stockouts, invalid coupons, and payment failures without crashing the application.
- **Rust Concepts**: `Result<T, E>` (`Ok` vs. `Err`), the `?` error propagation operator, custom enum errors.
- **MiniStore Milestone**: Comprehensive `StoreError` domain diagnostics and transactional `checkout` validation.
- **Standalone Code**: [`examples/chapter-12`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-12)

### [Chapter 13: Modules, Packages and Project Structure](/en/chapters/13-modules-and-project-structure)

- **Problem**: Organizing growing domain code into maintainable, encapsulated modules and crates.
- **Rust Concepts**: `mod`, `pub`, `pub(crate)`, dual-crate package architecture (`lib.rs` and `main.rs`), `pub use` facade re-exports.
- **MiniStore Milestone**: Refactoring MiniStore into `src/models/`, `src/catalog.rs`, `src/checkout.rs`, and a clean public API facade.
- **Standalone Code**: [`examples/chapter-13`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-13)

---

## Part III — Making Rust Code Powerful

### [Chapter 14: Generics](/en/chapters/14-generics)

- **Problem**: Building reusable pagination containers and API response wrappers without code duplication or runtime boxing.
- **Rust Concepts**: Generic structs, functions, methods, monomorphization, zero runtime overhead.
- **MiniStore Milestone**: Generic `Page<T>` container, pagination engine `paginate()`, and `ApiResponse<T>`.
- **Standalone Code**: [`examples/chapter-14`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-14)

### [Chapter 15: Traits](/en/chapters/15-traits)

- **Problem**: Defining shared behavior contracts across disparate models without inheritance hierarchies.
- **Rust Concepts**: Trait definitions, default implementations, trait bounds (`T: Summarizable + Taxable`), standard library traits (`Display`, `Error`).
- **MiniStore Milestone**: `Taxable` and `Summarizable` traits for Products, Orders, and Customers.
- **Standalone Code**: [`examples/chapter-15`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-15)

### [Chapter 16: Lifetimes](/en/chapters/16-lifetimes)

- **Problem**: Creating zero-copy receipts that borrow from existing orders and customers without dangling references.
- **Rust Concepts**: Generic lifetime parameters (`'a`), lifetime elision rules, `'static` lifetime.
- **MiniStore Milestone**: Zero-copy `OrderReceipt<'a>` and store policy slogan `store_policy() -> &'static str`.
- **Standalone Code**: [`examples/chapter-16`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-16)

### [Chapter 17: Iterators](/en/chapters/17-iterators)

- **Problem**: Transforming, filtering, and summarizing collections cleanly with maximum efficiency.
- **Rust Concepts**: `Iterator` trait, lazy adapters (`map`, `filter`, `fold`), `IntoIterator`, custom iterators.
- **MiniStore Milestone**: Implementing `IntoIterator` for `ShoppingCart` and custom `DiscountTierIter`.
- **Standalone Code**: [`examples/chapter-17`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-17)

### [Chapter 18: Closures](/en/chapters/18-closures)

- **Problem**: Creating dynamic, configurable discount rules and audit trackers that capture surrounding context.
- **Rust Concepts**: Closures (`|x| ...`), `Fn`, `FnMut`, `FnOnce`, environment capture, `move` factories.
- **MiniStore Milestone**: Closure discount factories (`make_vip_discount`), stateful `FnMut` cart discounts, and `DiscountAuditor`.
- **Standalone Code**: [`examples/chapter-18`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-18)

### [Chapter 19: Smart Pointers](/en/chapters/19-smart-pointers)

- **Problem**: Managing recursive category trees, shared customer handles, and RAII session cleanup.
- **Rust Concepts**: `Box<T>`, `Rc<T>`, `RefCell<T>`, interior mutability, `Deref`, `Drop`.
- **MiniStore Milestone**: Recursive `CategoryNode`, shared single-threaded customer handles, and custom `StoreSession<T>`.
- **Standalone Code**: [`examples/chapter-19`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-19)

---

## Part IV — Concurrency & Async

### [Chapter 20: Concurrency](/en/chapters/20-concurrency)

- **Problem**: Processing orders concurrently across CPU cores without data races or deadlocks.
- **Rust Concepts**: OS threads (`std::thread::spawn`), message passing (`std::sync::mpsc`), shared state (`Arc<Mutex<T>>`), `Send` and `Sync`.
- **MiniStore Milestone**: Multi-threaded `ConcurrentSalesTracker` and worker thread batch valuation.
- **Standalone Code**: [`examples/chapter-20`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-20)

### [Chapter 21: Async Rust](/en/chapters/21-async-rust)

- **Problem**: Handling thousands of concurrent I/O-bound operations (remote warehouse lookups, payment gateways) without thread exhaustion.
- **Rust Concepts**: Tokio asynchronous runtime, `async`/`await`, non-blocking I/O, `tokio::join!`, `tokio::sync::mpsc`.
- **MiniStore Milestone**: Asynchronous payment client, distributed warehouse aggregation, and async checkout workflow.
- **Standalone Code**: [`examples/chapter-21`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-21)

---

## Part V — Application Layer

### [Chapter 22: Persistence in Rust](/en/chapters/22-persistence)

- **Problem**: Persisting store data across application restarts without data corruption or partial writes.
- **Rust Concepts**: Serde JSON serialization/deserialization, atomic file replacement via temporary files and `fs::rename`, snapshotting.
- **MiniStore Milestone**: Crash-resilient `StorePersistence` and system backup snapshots (`StoreSnapshot`).
- **Standalone Code**: [`examples/chapter-22`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-22)

### [Chapter 23: Building a Rust Web API](/en/chapters/23-web-api)

- **Problem**: Exposing MiniStore over the network to web browsers and mobile apps via HTTP.
- **Rust Concepts**: Axum framework, macro-free routing, type-safe extractors (`Path`, `Json`, `State`), `IntoResponse` error mapping.
- **MiniStore Milestone**: Complete REST Web API (`/health`, `/api/products`, `/api/orders`, `/api/checkout`).
- **Standalone Code**: [`examples/chapter-23`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-23)

### [Chapter 24: Production Rust](/en/chapters/24-production-rust)

- **Problem**: Preparing a web service for containerized cloud deployment (Kubernetes/Docker).
- **Rust Concepts**: 12-factor configuration (`AppConfig`), structured telemetry (`tracing`, `TraceLayer`), liveness/readiness probes, OS signal graceful shutdown.
- **MiniStore Milestone**: Production telemetry setup, `/health/live`, `/health/ready`, and signal-driven graceful shutdown.
- **Standalone Code**: [`examples/chapter-24`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-24)

---

## Part VI — Advanced Rust & Metaprogramming

### [Chapter 25: Declarative Macros](/en/chapters/25-declarative-macros)

- **Problem**: Eliminating verbose test boilerplate and building expressive domain DSLs.
- **Rust Concepts**: `macro_rules!`, designators (`$expr`, `$ident`, `$ty`), repetition patterns (`$( ... ),*`), macro hygiene, `$crate`.
- **MiniStore Milestone**: Domain macros: `product!`, `catalog!`, `cart!`, `calculate_total!`, and `assert_in_stock!`.
- **Standalone Code**: [`examples/chapter-25`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-25)

### [Chapter 26: Unsafe Rust & Safe Abstractions](/en/chapters/26-unsafe-rust)

- **Problem**: Low-level memory manipulation, hardware-adjacent buffers, and C library interoperability.
- **Rust Concepts**: Raw pointers (`*const T`, `*mut T`, `NonNull<T>`), `std::alloc` (`alloc`, `realloc`, `dealloc`, `Layout`), Foreign Function Interface (`extern "C"`), safe abstraction boundaries.
- **MiniStore Milestone**: `RawBarcodeBuffer` safe heap abstraction, raw pointer swap, and C ABI FFI integration.
- **Standalone Code**: [`examples/chapter-26`](https://github.com/rimonmath/crafting-rust/tree/main/examples/chapter-26)

### [Chapter 27: Conclusion & Where to Go Next](/en/chapters/27-conclusion-and-beyond)

- **Problem**: Reviewing the grand architecture, demystifying library-author internals (Pin, PhantomData, Proc-Macros), and plotting the road ahead.
- **Rust Concepts**: Pinning mechanics, Typestate pattern, procedural macros (`syn`/`quote`), custom executors, ecosystem curation.
- **MiniStore Milestone**: Curriculum graduation and next project guidance.
