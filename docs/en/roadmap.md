# Crafting Rust — Learning Roadmap

This curriculum is structured for experienced engineers moving from garbage-collected (Python, Go, Java, TypeScript) or manual memory (C/C++) environments to idiomatic Rust. Over 27 chapters, we will engineer **MiniStore**—a production-ready engine whose final compiled release binary is **under 5MB (only ~4.6MB)** with zero external runtime dependencies.

```
Part 0: Foundations & Setup
  └── Why Rust → Toolchain (cargo) → Types & Immutability → Control Flow → Structs
Part I: Ownership & Memory
  └── Ownership → Borrowing & References → Slices & Strings → Collections
Part II: Modeling Business Logic
  └── Enums & Pattern Matching → Option<T> → Result<T, E> → Modules & Cargo
Part III: High-Performance Abstractions
  └── Generics → Traits & Polymorphism → Lifetimes → Iterators → Closures → Smart Pointers
Part IV: Concurrency & Async
  └── Threads, Channels & Mutexes → Tokio & Async Runtime
Part V: Application Layer
  └── Persistence (Serde) → Axum Web API → Production Readiness (Tracing, Shutdown)
Part VI: Advanced Rust & Metaprogramming
  └── Declarative Macros (macro_rules!) → Unsafe Rust, Raw Pointers & FFI
```

---

## Complete Curriculum Directory

### Part 0 — Starting the Journey
- [1. Why Rust? How We'll Learn Rust](/en/chapters/01-why-rust)
- [2. Getting Started with Rust](/en/chapters/02-getting-started)
- [3. Rust's Basic Building Blocks](/en/chapters/03-basic-building-blocks)
- [4. Control Flow](/en/chapters/04-control-flow)
- [5. Structs: Modeling Real Things](/en/chapters/05-structs)

### Part I — Ownership & Memory
- [6. Ownership](/en/chapters/06-ownership)
- [7. Borrowing and References](/en/chapters/07-borrowing-and-references)
- [8. Strings, Slices and Ownership in Practice](/en/chapters/08-strings-slices-and-ownership)
- [9. Collections](/en/chapters/09-collections)

### Part II — Modeling Business Logic
- [10. Enums and Pattern Matching](/en/chapters/10-enums-and-pattern-matching)
- [11. Option](/en/chapters/11-option)
- [12. Result and Error Handling](/en/chapters/12-result-and-error-handling)
- [13. Modules, Packages and Project Structure](/en/chapters/13-modules-and-project-structure)

### Part III — Making Rust Code Powerful
- [14. Generics](/en/chapters/14-generics)
- [15. Traits](/en/chapters/15-traits)
- [16. Lifetimes](/en/chapters/16-lifetimes)
- [17. Iterators](/en/chapters/17-iterators)
- [18. Closures](/en/chapters/18-closures)
- [19. Smart Pointers](/en/chapters/19-smart-pointers)

### Part IV — Concurrency & Async
- [20. Concurrency](/en/chapters/20-concurrency)
- [21. Async Rust](/en/chapters/21-async-rust)

### Part V — Application Layer
- [22. Persistence in Rust](/en/chapters/22-persistence)
- [23. Building a Rust Web API](/en/chapters/23-web-api)
- [24. Production Rust](/en/chapters/24-production-rust)

### Part VI — Advanced Rust & Metaprogramming
- [25. Declarative Macros](/en/chapters/25-declarative-macros)
- [26. Unsafe Rust & Safe Abstractions](/en/chapters/26-unsafe-rust)

---

## MiniStore Incremental Architecture
- **Stage 1**: In-memory domain models (Products, Customers, Inventory).
- **Stage 2**: Shopping Cart, Sales transactions, and progressive discounts.
- **Stage 3**: Business Rules engine, receipts, and custom reporting iterators.
- **Stage 4**: Multi-threaded concurrency, atomic sales tracking, and thread message channels.
- **Stage 5**: Asynchronous warehouse aggregation and non-blocking payment processing with Tokio.
- **Stage 6**: Crash-resilient file persistence with atomic file writes and point-in-time snapshots.
- **Stage 7**: Full HTTP REST Web API using Axum with type-safe extractors and domain error responses.
- **Stage 8**: Production readiness with 12-factor configuration, structured tracing, health probes, and graceful shutdown.
- **Stage 9**: Custom declarative DSL macros (`product!`, `catalog!`, `cart!`, `calculate_total!`, `assert_in_stock!`).
- **Stage 10**: Low-level memory buffers (`RawBarcodeBuffer`) and C-ABI Foreign Function Interface (FFI).
