# Chapter 16: Lifetimes

## What You'll Learn
- What **Lifetimes** are and how Rust guarantees memory safety for references without a Garbage Collector (GC).
- The role of the **Borrow Checker** in detecting and preventing **Dangling References** at compile time.
- Lifetime annotation syntax: **`'a`**, **`'b`**, and how to read lifetime parameters.
- Why lifetime annotations **do not change how long values live**, but rather describe the relationship between the lifespans of multiple references.
- Lifetime annotations in functions: when they are mandatory and how they constrain return values.
- The **Three Lifetime Elision Rules** that allow omitting annotations in common method patterns.
- Structs holding references: Designing high-performance **zero-copy structs** (`OrderReceipt<'a>`) that borrow data without heap cloning.
- Method definitions on structs with lifetimes (`impl<'a> OrderReceipt<'a>`).
- The special **`'static`** lifetime: string literals, global constants, and the `T: 'static` trait bound.
- Common compiler errors and how to fix them:
  - `error[E0106]: missing lifetime specifier`
  - `error[E0515]: cannot return reference to local variable`
  - `error[E0597]: borrowed value does not live long enough`
- Integrating Lifetimes into MiniStore:
  - Developing `OrderReceipt<'a>` for zero-copy invoice and slip generation.
  - Implementing `find_higher_priced<'a>` to compare borrowed `Product` references.
  - Implementing `best_contact_info<'a>` to dynamically select between optional reference inputs.
  - Adding `store_policy() -> &'static str` for zero-allocation policy slogans.

---

## Why Do We Need This?

In Chapters 6 and 7, we learned about **Ownership** and **Borrowing**:
- When a variable goes out of scope, Rust automatically drops it and frees its memory.
- We can borrow references (`&T` or `&mut T`) to read or mutate data without taking ownership.

Most of the time, Rust's compiler figures out the validity of references automatically:
```rust
fn print_length(s: &str) {
    println!("Length: {}", s.len());
}
```
Here, `s` is borrowed, read, and discarded before the function ends. Everything is simple and unambiguous.

However, consider this common scenario:
```rust
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

If you try to compile this code, Rust **refuses to compile**:
```text
error[E0106]: missing lifetime specifier
 --> src/main.rs:1:33
  |
1 | fn longest(x: &str, y: &str) -> &str {
  |               ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the
          signature does not say whether it is borrowed from `x` or `y`
```

### The Problem: Ambiguous Reference Origin
Look closely at the error message: Rust doesn't know whether the returned reference points to the data in `x` or the data in `y`!
- If the caller passes `x` with a lifespan of 10 seconds, but `y` with a lifespan of 2 seconds, which lifespan applies to the returned reference?
- If the return value is stored in a variable that outlives `y`, but `longest` happened to return `y`, reading that returned reference would cause a **use-after-free** or **dangling pointer** crash!

### How Other Languages Deal With This
1. **C / C++**: Leaves reference validity entirely to the programmer. If a function returns a reference to temporary memory or an object that is subsequently destroyed, the program compiles cleanly, but crashes with a **Segmentation Fault** or silently corrupts memory at runtime.
2. **Java / Go / C# / Python**: Solves the problem by putting all objects on the heap and running a **Garbage Collector (GC)**. If a reference exists anywhere, the GC keeps the object alive in memory. While safe, this introduces runtime pauses, significant memory overhead, and CPU cache inefficiency.
3. **Rust**: Rust provides **memory safety without a garbage collector**. The compiler's **Borrow Checker** verifies at compile time that every reference is guaranteed to point to valid memory for as long as the reference exists.

To make this possible when functions or structs deal with references across boundaries, Rust asks us for **Lifetime Annotations** (`'a`).

---

## What Are Lifetimes?

A **lifetime** is the span of code during which a reference is valid.

```rust
{
    let r;                // ---------+-- 'a
                          //          |
    {                     //          |
        let x = 5;        // -+-- 'b  |
        r = &x;           //  |       |
    }                     // -+       |
                          //          |
    println!("r: {}", r); // ---------+ // ERROR: `x` dropped while borrowed!
}
```

In the diagram above:
- The lifetime of `r` is `'a`.
- The lifetime of `x` is `'b`.
- `'b` is strictly shorter than `'a`. When the inner scope ends, `x` is destroyed.
- `r` is left pointing to deallocated stack memory! Rust's borrow checker rejects this at compile time.

### Lifetime Annotation Syntax
Lifetime parameters start with an apostrophe `'` and are typically given short, lowercase names like `'a`, `'b`, or `'c`:
- `&i32` — a reference without an explicit lifetime annotation.
- `&'a i32` — a reference with an explicit lifetime `'a`.
- `&'a mut i32` — a mutable reference with an explicit lifetime `'a`.

> [!IMPORTANT]
> **Lifetime annotations do not change how long any value actually lives!**
> Just as generic type parameters `<T>` do not change what a type is, lifetime parameters `<'a>` merely describe the relationship between the lifespans of multiple references so the borrow checker can prove they are safe.

---

## Lifetime Annotations in Functions

Let us fix our `longest` function from earlier:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

### What Does This Signature Mean?
1. `<'a>` declares a generic lifetime parameter `'a`.
2. `x: &'a str` and `y: &'a str` declare that both `x` and `y` are references that live **at least as long as** `'a`.
3. `-> &'a str` guarantees that the returned reference will also live for **at least `'a`**.

In practice, `'a` will be equal to the **intersection (the smaller)** of the lifetimes of `x` and `y`.

### Practical MiniStore Example: Comparing Products
In MiniStore, suppose we want a helper function that takes two borrowed products and returns a reference to whichever product has the higher price:

```rust
pub fn find_higher_priced<'a>(p1: &'a Product, p2: &'a Product) -> &'a Product {
    if p1.price_cents >= p2.price_cents {
        p1
    } else {
        p2
    }
}
```

Let's test how the compiler enforces this:
```rust
let keyboard = Product::new(101, ...); // Lives for outer scope
let result;
{
    let mouse = Product::new(102, ...); // Lives for inner scope
    result = find_higher_priced(&keyboard, &mouse);
    println!("Higher: {}", result.name); // OK: mouse is still alive here!
}
// println!("Higher: {}", result.name); // COMPILE ERROR: mouse has been dropped!
```
The borrow checker ensures that `result` cannot be used outside the inner scope because it might point to `mouse`, which no longer exists!

---

## The Three Lifetime Elision Rules

You may wonder: *Why haven't we had to write `'a` on every function we wrote in previous chapters?*

In the early days of Rust (pre-1.0), programmers had to write lifetime annotations on *every single reference*. The Rust core team analyzed thousands of functions and discovered that common patterns occurred repeatedly.

The compiler therefore implements **Lifetime Elision Rules**—deterministic heuristics that allow omitting lifetime annotations in 90% of everyday code.

### Rule 1: Each elided lifetime in the inputs gets a distinct lifetime parameter
```rust
fn print(s: &str)               // becomes -> fn print<'a>(s: &'a str)
fn pair(x: &str, y: &str)       // becomes -> fn pair<'a, 'b>(x: &'a str, y: &'b str)
```

### Rule 2: If there is exactly one input lifetime parameter, that lifetime is assigned to all output lifetimes
```rust
fn first_word(s: &str) -> &str  // becomes -> fn first_word<'a>(s: &'a str) -> &'a str
```
Because there is only one input reference, the returned reference can *only* come from that one input!

### Rule 3: If there are multiple input parameters, but one of them is `&self` or `&mut self`, the lifetime of `self` is assigned to all output lifetimes
```rust
impl Product {
    fn name(&self) -> &str      // becomes -> fn name<'a>(&'a self) -> &'a str
}
```
This makes writing methods on structs completely natural and boilerplate-free.

> [!NOTE]
> If none of these three rules uniquely resolves the return lifetime (like in our `find_higher_priced` or `longest` function with two independent input references and a returned reference), the compiler raises `error[E0106]` and asks you to annotate explicitly.

---

## Structs Holding References: Zero-Copy Architecture

So far, all our structs (`Product`, `Customer`, `Order`) have owned their data using heap-allocated `String` and `Vec`.

While owning data is straightforward, in high-throughput systems (like an e-commerce checkout engine printing thousands of receipts per second), allocating and cloning strings for temporary view objects creates unnecessary memory churn.

By holding **references** instead of owned values, we achieve **Zero-Copy Architecture**:

```rust
/// A zero-copy receipt view that borrows an `Order`, customer name, and cashier note.
pub struct OrderReceipt<'a> {
    pub order: &'a Order,
    pub customer_name: &'a str,
    pub cashier_note: &'a str,
}
```

### Why Does the Struct Need `<'a>`?
If you omit `<'a>`:
```rust
pub struct OrderReceipt {
    pub order: &Order, // COMPILE ERROR: missing lifetime specifier
}
```
Rust refuses to compile because:
1. A struct's instance lives in memory.
2. The references inside that struct point to *other* data living in memory.
3. The lifetime parameter `<'a>` tells the compiler: **An `OrderReceipt` instance can never outlive the `Order` or strings it points to!**

### Implementing Methods on Lifetime Structs
When writing an `impl` block for a struct with lifetimes, declare `'a` after `impl`:

```rust
impl<'a> OrderReceipt<'a> {
    pub fn new(order: &'a Order, customer_name: &'a str, cashier_note: &'a str) -> Self {
        Self {
            order,
            customer_name,
            cashier_note,
        }
    }

    /// Notice: Returning &'a str ties the output to the customer string,
    /// NOT to the temporary `&self` reference!
    pub fn customer_name(&self) -> &'a str {
        self.customer_name
    }

    pub fn note(&self) -> &'a str {
        self.cashier_note
    }

    pub fn generate_slip(&self) -> String {
        format!(
            "=== RECEIPT: {} ===\nCustomer: {}\nItems: {}\nTotal: ${:.2}\nStatus: {}\nNote: {}\n===================",
            self.order.order_id,
            self.customer_name,
            self.order.items.len(),
            self.order.total_cents() as f64 / 100.0,
            self.order.status,
            self.cashier_note
        )
    }
}
```

---

## The Special `'static` Lifetime

Rust has one reserved, special lifetime name: **`'static`**.

A reference with a `'static` lifetime can live for the **entire duration of the program**.

### 1. String Literals
All string literals (`"..."`) have the `'static` lifetime because their raw bytes are hardcoded directly into the binary's read-only data segment:
```rust
pub fn store_policy() -> &'static str {
    "MiniStore Guarantee: 30-Day Hassle-Free Returns & 1-Year Warranty"
}
```
You can return a string literal from any function without worrying about scopes or lifetimes!

### 2. Trait Bound `T: 'static`
You will often see `T: 'static` in generic code or multi-threaded Rust.
It does **not** mean that the value lives forever; it means that the type `T` **does not contain any non-static references** (i.e. it is an owned type like `String` or `u32`, or only holds `'static` references).

---

## Comparison With Other Languages

| Dimension | Rust | C / C++ | Go | Java / C# |
| :--- | :--- | :--- | :--- | :--- |
| **Reference Safety** | **Compile-time guarantee** (Borrow Checker) | Manual (Undefined behavior on mistake) | Runtime Garbage Collector | Runtime Garbage Collector |
| **Dangling Pointers** | **Impossible** in safe code | Very common bug | Impossible (GC keeps alive) | Impossible (GC keeps alive) |
| **Runtime Cost** | **Zero runtime cost** (Erased at compile time) | Zero runtime cost | GC overhead, pause times | GC overhead, boxing, cache misses |
| **Syntax Overhead** | Explicit `'a` in ambiguous signatures | None (Pointers unchecked) | None (Runtime managed) | None (Runtime managed) |
| **Zero-Copy Views** | First-class and 100% safe | Fast but dangerous | Safe via slices (`[]byte`) | ByteBuffers / String views |

---

## MiniStore Architecture & Code Implementation

Here is how lifetime-enabled features are integrated into MiniStore:

```
ministore/
└── src/
    ├── models/
    │   ├── receipt.rs      # OrderReceipt<'a>, find_higher_priced, best_contact_info, store_policy
    │   └── mod.rs          # Re-exports receipt module and items
    ├── lib.rs              # Re-exports receipt items & 24 passing unit tests
    └── main.rs             # CLI showing zero-copy receipts, slip generation, and lifetimes
```

### 1. `src/models/receipt.rs`
```rust
use crate::models::{Order, OrderId, Product};
use crate::traits::Summarizable;

#[derive(Debug, Clone, PartialEq)]
pub struct OrderReceipt<'a> {
    pub order: &'a Order,
    pub customer_name: &'a str,
    pub cashier_note: &'a str,
}

impl<'a> OrderReceipt<'a> {
    pub fn new(order: &'a Order, customer_name: &'a str, cashier_note: &'a str) -> Self {
        Self {
            order,
            customer_name,
            cashier_note,
        }
    }

    pub fn customer_name(&self) -> &'a str {
        self.customer_name
    }

    pub fn note(&self) -> &'a str {
        self.cashier_note
    }

    pub fn order_id(&self) -> OrderId {
        self.order.order_id
    }

    pub fn generate_slip(&self) -> String {
        format!(
            "=== RECEIPT: {} ===\nCustomer: {}\nItems: {}\nTotal: ${:.2}\nStatus: {}\nNote: {}\n===================",
            self.order.order_id,
            self.customer_name,
            self.order.items.len(),
            self.order.total_cents() as f64 / 100.0,
            self.order.status,
            self.cashier_note
        )
    }
}

impl<'a> std::fmt::Display for OrderReceipt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Receipt for Order {} ({}) - Total: ${:.2}",
            self.order.order_id,
            self.customer_name,
            self.order.total_cents() as f64 / 100.0
        )
    }
}

impl<'a> Summarizable for OrderReceipt<'a> {
    fn summary(&self) -> String {
        format!(
            "Receipt: Order {} for {} | Total: ${:.2}",
            self.order.order_id,
            self.customer_name,
            self.order.total_cents() as f64 / 100.0
        )
    }
}

pub fn find_higher_priced<'a>(p1: &'a Product, p2: &'a Product) -> &'a Product {
    if p1.price_cents >= p2.price_cents {
        p1
    } else {
        p2
    }
}

pub fn best_contact_info<'a>(primary_phone: Option<&'a str>, fallback_email: &'a str) -> &'a str {
    match primary_phone {
        Some(phone) if !phone.trim().is_empty() => phone,
        _ => fallback_email,
    }
}

pub fn store_policy() -> &'static str {
    "MiniStore Guarantee: 30-Day Hassle-Free Returns & 1-Year Warranty"
}
```

---

## Common Compiler Errors & How to Fix Them

### 1. `error[E0515]: cannot return reference to local variable`
**The Mistake**:
```rust
fn create_receipt_note() -> &str {
    let note = String::from("Customer paid in cash");
    &note // COMPILE ERROR: returns a reference to data owned by the current function
}
```
**Why It Happens**:
The local variable `note` is dropped at the end of the function scope. Returning `&note` would return a pointer to deallocated stack memory!
**The Fix**:
Return the **owned** `String`, or borrow from a caller-provided variable:
```rust
fn create_receipt_note() -> String {
    String::from("Customer paid in cash")
}
```

---

### 2. `error[E0597]: borrowed value does not live long enough`
**The Mistake**:
```rust
let receipt;
{
    let local_order = Order::new(...);
    receipt = OrderReceipt::new(&local_order, "Alice", "Paid");
} // local_order dropped here!
println!("{}", receipt.order_id()); // COMPILE ERROR
```
**Why It Happens**:
`receipt` expects `local_order` to live as long as it does. But `local_order` died when its scope closed.
**The Fix**:
Ensure the borrowed data lives at least as long as the struct holding the reference:
```rust
let local_order = Order::new(...);
let receipt = OrderReceipt::new(&local_order, "Alice", "Paid");
println!("{}", receipt.order_id());
```

---

## Idiomatic Rust Best Practices

1. **Rely on Lifetime Elision When Possible**: Don't annotate lifetimes if the compiler can deduce them using the elision rules. Unnecessary annotations clutter code.
2. **Prefer Owned Types for Long-Lived Domain Entities**: Use owned types (`String`, `Vec`) for structs that represent long-lived entities (like `Order` or `Product`). Use references in structs (`OrderReceipt<'a>`) for short-lived views, parsers, and zero-copy reports.
3. **Keep Lifetime Scopes Focused**: If a struct holds references, avoid storing it in long-lived state caches where lifetime tracking can become restrictive.
4. **Use Meaningful Lifetime Names When Necessary**: While `'a` is the standard convention, for structs with multiple independent lifetimes, descriptive names like `'order` and `'note` clarify relationships.

---

## Hands-On Exercises

### Exercise 1: Zero-Copy Search Match
1. Create a struct `SearchMatch<'a>` that borrows `product: &'a Product` and `matched_query: &'a str`.
2. Implement a method `format_result(&self) -> String` that prints `"[Matched 'query']: Product Name ($Price)"`.
3. Verify that creating multiple `SearchMatch` objects incurs zero heap allocations for the product name.

### Exercise 2: Lifetime-Annotated Slicer
1. Write a function `first_matching_tag<'a>(tags: &'a [String], prefix: &str) -> Option<&'a str>`.
2. It should return a borrowed `&str` slice from the matching `String` inside the slice.
3. Test that the returned string slice lives as long as the `Vec<String>`.

---

## Checkpoint

Run compiler checks and verify all 24 unit tests pass:
```bash
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
cargo run
```

Expected test output:
```text
running 24 tests
test tests::test_cart_item_option_lookup ... ok
test tests::test_coupon_validation_error ... ok
test tests::test_display_trait_implementations ... ok
test tests::test_customer_optional_phone ... ok
test tests::test_checkout_error_propagation_and_success ... ok
test tests::test_generic_api_response_wrapper ... ok
test tests::test_catalog_option_lookups ... ok
test tests::test_coupon_discount_and_take ... ok
test tests::test_generic_catalog_product_pagination ... ok
test tests::test_generic_page_map_transformation ... ok
test tests::test_generic_pagination_with_integers ... ok
test tests::test_generic_trait_bound_functions ... ok
test tests::test_lifetime_annotated_contact_resolution ... ok
test tests::test_lifetime_annotated_product_comparison ... ok
test tests::test_order_cancellation_prevention ... ok
test tests::test_order_status_valid_lifecycle ... ok
test tests::test_order_total_with_payment_fee ... ok
test tests::test_payment_method_fees_and_descriptions ... ok
test tests::test_product_category_tax_rates ... ok
test tests::test_product_stock_reduction_error ... ok
test tests::test_static_lifetime_policy ... ok
test tests::test_summarizable_trait_on_domain_models ... ok
test tests::test_taxable_trait_and_default_method ... ok
test tests::test_zero_copy_order_receipt_and_traits ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Expected CLI output:
```text
=== MiniStore: Lifetimes & Reference Safety (Part III) ===

1. Catalog initialized with 3 products.

2. Browsing Catalog with Generic Pagination (Page<Product>):
   Page 1 of 2 (Total Items: 3)
   - [TECH-KEY-001] Tenkeyless Mechanical Keyboard ($120.00)
   - [TECH-MOU-002] Ergonomic Wireless Mouse ($45.00)
   Has next page? true
   Transformed to Page<String>: ["Tenkeyless Mechanical Keyboard", "Ergonomic Wireless Mouse"]
   API Page 2 response: 1 product(s) returned out of 3 total.

3. Customer: Margaret Hamilton (+1-555-0199)

4. Lifetimes in Functions & Reference Safety:
   Higher priced item: Tenkeyless Mechanical Keyboard ($120.00)
   Best contact info: +1-555-0199
   Store Policy ('static): MiniStore Guarantee: 30-Day Hassle-Free Returns & 1-Year Warranty

5. Shared Behaviors via Traits:
   Tax Summary: Product #101: Tenkeyless Mechanical Keyboard [TECH-KEY-001] - $120.00 | Tax: $18.00 (15%)
   Tax Summary: Product #102: Ergonomic Wireless Mouse [TECH-MOU-002] - $45.00 | Tax: $6.75 (15%)
   Customer Summary: Customer #301: Margaret Hamilton <margaret@apollo.nasa.gov>

6. Processing checkout through modular services...
   Checkout Order #901 created successfully!
   Subtotal: $210.00 | Total: $148.50

7. Order Lifecycle Transitions:
   Order confirmed: Confirmed (Receipt: REC-901-HAMILTON)
   Order shipped:   Shipped (Tracking: TRK-FEDEX-77189)
   Cancellation prevented -> Cannot perform action 'cancel' while order is in 'Shipped (Tracking: TRK-FEDEX-77189)' state
   Final Lifecycle State: Delivered to Customer
   Order Summary: Order #901: 2 item(s), Total: $148.50 [Delivered to Customer]

8. Zero-Copy Receipt Borrowing Order & Slices (OrderReceipt<'a>):
   Display format: Receipt for Order #901 (Margaret Hamilton) - Total: $148.50
   Trait Summary:  Receipt: Order #901 for Margaret Hamilton | Total: $148.50

--- Printed Slip ---
=== RECEIPT: #901 ===
Customer: Margaret Hamilton
Items: 2
Total: $148.50
Status: Delivered to Customer
Note: VIP Client - Express courier delivery verified
===================
--------------------
```

---

## What We Learned
- Why lifetimes exist: providing compile-time reference validity without garbage collector overhead.
- How generic lifetime annotations (`'a`) describe relationships between borrowed references.
- How the three lifetime elision rules save boilerplate in standard functions and methods.
- How to design zero-copy structs (`OrderReceipt<'a>`) that borrow data safely.
- The meaning of `'static` for string literals and global memory.

---

## What's Next
Now that we have mastered Ownership, Generics, Traits, and Lifetimes, we are ready to explore one of Rust's most expressive and idiomatic features: **Iterators**!
In **Chapter 17: Iterators**, we will learn how Rust processes sequences with functional adapters (`map`, `filter`, `fold`) while achieving zero-cost abstraction through lazy evaluation!
