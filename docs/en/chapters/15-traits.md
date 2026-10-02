# Chapter 15: Traits

## What You'll Learn
- What **Traits** are and how Rust implements polymorphism and shared behavior without object-oriented class inheritance.
- How to define a trait using the **`trait`** keyword (`pub trait Taxable`).
- How to implement traits on structs and enums using **`impl Trait for Type`**.
- Providing and overriding **Default Method Implementations** to reduce repetitive boilerplate.
- Constraining generic types using **Trait Bounds** (`fn process<T: Summarizable>(item: &T)`).
- Multiple trait bounds using the **`+` syntax** (`T: Taxable + Summarizable`) and organizing complex bounds using **`where` clauses**.
- Returning types that implement traits using the **`impl Trait`** syntax.
- Essential Rust standard library traits:
  - **`std::fmt::Display`**: User-facing string formatting with `{}`.
  - **`std::fmt::Debug`**: Programmer-facing diagnostics with `{:?}`.
  - **`std::error::Error`**: The official standard error contract.
  - **`Clone`**, **`Copy`**, **`PartialEq`**, and **`Default`**.
- Deriving traits automatically (`#[derive(...)]`) vs. writing manual trait implementations.
- Integrating traits into MiniStore:
  - Defining `Taxable` with default tax calculation logic.
  - Defining `Summarizable` for universal reporting across products, customers, and orders.
  - Implementing `std::fmt::Display` and `std::error::Error` for `StoreError`.
  - Implementing `std::fmt::Display` for `OrderId` and `OrderStatus`.

---

## Why Do We Need This?
In Chapter 14, we discovered **Generics** (`<T>`), which let us write universal containers like `Page<T>` and generic functions like `paginate<T>`.

However, pure generics have a major limitation:
```rust
fn print_tax<T>(item: &T, price_cents: u32) {
    // COMPILE ERROR: Rust doesn't know if T has a `tax_rate()` method!
    let tax = item.tax_rate() * price_cents;
    println!("Tax: {}", tax);
}
```
If `T` can be *any* type in the universe (an integer, a string, a product, a customer), Rust's compiler refuses to allow method calls on `item` because integers don't have a `.tax_rate()` method!

### The Problem in Traditional OOP (Inheritance)
In languages like Java, C++, or C#, shared behavior is typically modeled with **class inheritance**:
```java
// Java / C# style class hierarchy
abstract class TaxableEntity {
    abstract int getTaxRate();
}

class Product extends TaxableEntity { ... }
class Service extends TaxableEntity { ... }
```
While inheritance is common, it carries well-known architectural drawbacks:
1. **The Fragile Base Class Problem**: Changes to a base class can silently break subclasses far down the hierarchy.
2. **Rigid Single Inheritance**: A class can usually only inherit from one parent. What if an item is both `Taxable`, `Auditable`, `Shippable`, and `Discountable`?
3. **Bloated State**: Subclasses inherit internal fields they may not need, coupling data layout with behavior.

### Rust's Solution: Traits
Rust has **no classes** and **no inheritance**.

Instead, Rust completely separates **data** (modeled with `struct` and `enum`) from **behavior** (modeled with `trait`).
- A **Trait** defines an abstract contract of capabilities: what a type can *do*.
- Any struct or enum can implement any number of traits.
- Generics can be restricted so they only accept types that implement specific traits (**Trait Bounds**).

This embraces the classic software engineering principle: **Composition over Inheritance.**

---

## Defining and Implementing a Trait

A trait is declared using the `trait` keyword. Inside the trait block, you specify method signatures:

```rust
pub trait Taxable {
    /// Returns the tax percentage rate (e.g. 15 for 15%).
    fn tax_rate(&self) -> u32;

    /// Calculates the tax amount in cents given a base price in cents.
    /// Provides a default implementation!
    fn calculate_tax(&self, price_cents: u32) -> u32 {
        (price_cents * self.tax_rate()) / 100
    }
}
```

Notice:
1. `tax_rate(&self) -> u32` has only a semicolon `;`—implementors **must** provide their own logic.
2. `calculate_tax(&self, price_cents: u32) -> u32` has a **default implementation** body! Implementors can use it automatically without writing a single line of code.

### Implementing the Trait on a Type
To implement a trait, use `impl TraitName for TypeName`:

```rust
impl Taxable for ProductCategory {
    fn tax_rate(&self) -> u32 {
        self.default_tax_rate()
    }
    // `calculate_tax` is automatically inherited from the default implementation!
}

impl Taxable for Product {
    fn tax_rate(&self) -> u32 {
        self.category.tax_rate()
    }
}
```

Now, any `ProductCategory` or `Product` can call `.tax_rate()` and `.calculate_tax()`:
```rust
let category = ProductCategory::Electronics;
println!("Tax on $100: ${:.2}", category.calculate_tax(10000) as f64 / 100.0);
```

---

## Mental Model: Contracts and Capabilities

Think of a Trait not as a family tree of types, but as a **job qualification** or **capability badge**:

```
           +-----------------------+
           |    Trait: Taxable     |
           |  fn tax_rate(&self)   |
           +-----------------------+
                      ▲
       ┌──────────────┴──────────────┐
       │                             │
+---------------+             +---------------+
|    Product    |             | ProductCategory|
+---------------+             +---------------+
```

Any type that signs the contract and implements `tax_rate` can wear the `Taxable` badge. A single struct can wear as many badges as it qualifies for:
```rust
impl Taxable for Product { ... }
impl Summarizable for Product { ... }
impl Display for Product { ... }
```

---

## Comparison With Other Languages

| Feature | Rust | Java / C# | Go | C++ | TypeScript |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Concept** | **Trait** | Interface | Interface | Pure Virtual Class / Concept | Interface |
| **Implementation** | Explicit (`impl Trait for Type`) | Explicit (`implements Interface`) | Implicit (duck typing by method signature) | Inheritance (`: public Base`) | Structural (implicit shape matching) |
| **Default Methods** | Yes (in trait definition) | Yes (default interface methods) | No | Yes (virtual base class method) | No (interfaces only have signatures) |
| **Dynamic Dispatch** | Opt-in via `&dyn Trait` | Default for interfaces | Default for interfaces | Virtual method table (`vtable`) | Dynamic runtime |
| **Static Dispatch** | **Default** (Monomorphized) | Requires generics with constraints | Moderate | Template concepts | Compile-time erased |

---

## Trait Bounds: Constraining Generics

Now we can solve the problem we hit at the start of this chapter! When writing a generic function, we tell Rust that `T` is not just *any* type, but a type that implements a specific trait:

### 1. The `impl Trait` Syntax (Ergonomic)
For simple functions, you can write `item: &impl Summarizable`:
```rust
pub fn print_summary(item: &impl Summarizable) {
    println!("{}", item.summary());
}
```
This is syntactic sugar for a trait bound.

### 2. Standard Trait Bound Syntax (`<T: Trait>`)
When multiple parameters must share the exact same type:
```rust
pub fn compare_summaries<T: Summarizable>(a: &T, b: &T) {
    println!("A: {}", a.summary());
    println!("B: {}", b.summary());
}
```

### 3. Multiple Trait Bounds (`+`)
If a function requires a type to implement more than one trait, combine them with `+`:
```rust
pub fn print_tax_and_summary<T: Taxable + Summarizable>(item: &T, price: u32) {
    println!("{}: Tax is ${}", item.summary(), item.calculate_tax(price));
}
```

### 4. Clearer Bounds with `where` Clauses
When generic functions have multiple parameters and multiple bounds, putting everything in angle brackets becomes hard to read. Rust provides `where` clauses:

```rust
// Hard to read:
pub fn process<T: Taxable + Summarizable, U: Clone + Display>(t: &T, u: &U) { ... }

// Idiomatic and clean with `where`:
pub fn process<T, U>(t: &T, u: &U)
where
    T: Taxable + Summarizable,
    U: Clone + Display,
{
    // ...
}
```

---

## Essential Standard Library Traits

Rust’s standard library relies heavily on traits. Understanding them is key to writing idiomatic code:

### 1. `std::fmt::Display` vs `std::fmt::Debug`
- **`Debug` (`{:?}`)**: Formats a type for developers and debugging. Can almost always be automatically derived: `#[derive(Debug)]`.
- **`Display` (`{}`)**: Formats a type for human end-users. Cannot be derived because Rust doesn't know how your domain model should look to a user; you must implement it manually.

```rust
impl std::fmt::Display for OrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}
```
Now:
```rust
let id = OrderId(901);
println!("Order ID: {}", id); // Prints: Order ID: #901
```

### 2. `std::error::Error`
All standard Rust errors implement `std::error::Error`. To make a custom error an official standard error, it must implement `Display` and `Debug`:
```rust
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

// Implement the standard Error trait
impl std::error::Error for StoreError {}
```

---

## MiniStore Architecture & Code Implementation

Let us inspect the trait architecture integrated into MiniStore:

```
ministore/
└── src/
    ├── traits.rs           # Taxable, Summarizable, format_tax_summary
    ├── error.rs            # Implements Display + std::error::Error
    ├── models/
    │   ├── customer.rs     # OrderId implements Display; Customer implements Summarizable
    │   ├── order.rs        # OrderStatus implements Display; Order implements Summarizable
    │   └── product.rs      # Product implements Taxable & Summarizable
    ├── lib.rs              # Re-exports traits and 20 unit tests
    └── main.rs             # Demonstrates traits, bounds, and Display formatting
```

### 1. `src/traits.rs`
```rust
use crate::models::{Customer, Order, Product, ProductCategory};

/// Defines sales tax calculation behavior.
pub trait Taxable {
    /// Returns the tax percentage rate (e.g. 15 for 15%).
    fn tax_rate(&self) -> u32;

    /// Calculates the tax amount in cents given a base price in cents.
    ///
    /// Provides a default implementation so implementors only need to supply `tax_rate`.
    fn calculate_tax(&self, price_cents: u32) -> u32 {
        (price_cents * self.tax_rate()) / 100
    }
}

/// Defines human-readable summary generation behavior.
pub trait Summarizable {
    /// Returns a concise single-line summary of the entity.
    fn summary(&self) -> String;
}

// ----------------------------------------------------------------------------
// Implementations for MiniStore Domain Models
// ----------------------------------------------------------------------------

impl Taxable for ProductCategory {
    fn tax_rate(&self) -> u32 {
        self.default_tax_rate()
    }
}

impl Taxable for Product {
    fn tax_rate(&self) -> u32 {
        self.category.tax_rate()
    }
}

impl Summarizable for Product {
    fn summary(&self) -> String {
        format!(
            "Product #{}: {} [{}] - ${:.2}",
            self.id,
            self.name,
            self.sku,
            self.price_cents as f64 / 100.0
        )
    }
}

impl Summarizable for Customer {
    fn summary(&self) -> String {
        format!("Customer #{}: {} <{}>", self.id, self.name, self.email)
    }
}

impl Summarizable for Order {
    fn summary(&self) -> String {
        format!(
            "Order #{}: {} item(s), Total: ${:.2} [{}]",
            self.order_id.0,
            self.items.len(),
            self.total_cents() as f64 / 100.0,
            self.status.display_status()
        )
    }
}

// ----------------------------------------------------------------------------
// Generic Functions with Trait Bounds
// ----------------------------------------------------------------------------

/// Generates a summary for any item that implements `Summarizable`.
pub fn summarize_item<T: Summarizable>(item: &T) -> String {
    item.summary()
}

/// Formats a complete pricing and tax summary for any item that is both
/// `Taxable` and `Summarizable`.
pub fn format_tax_summary<T>(item: &T, price_cents: u32) -> String
where
    T: Taxable + Summarizable,
{
    let tax = item.calculate_tax(price_cents);
    format!(
        "{} | Tax: ${:.2} ({}%)",
        item.summary(),
        tax as f64 / 100.0,
        item.tax_rate()
    )
}
```

### 2. Standard `Display` Implementations

In `src/error.rs`:
```rust
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for StoreError {}
```

In `src/models/customer.rs`:
```rust
impl std::fmt::Display for OrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}
```

In `src/models/order.rs`:
```rust
impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_status())
    }
}
```

### 3. Application Demo in `src/main.rs`
```rust
use ministore::{
    checkout, format_tax_summary, summarize_item, ApiResponse, Catalog, Coupon, Customer, OrderId,
    Page, PaymentMethod, Product, ProductCategory, ShoppingCart,
};

fn main() {
    println!("=== MiniStore: Traits & Shared Behavior (Part III) ===\n");

    let keyboard = Product::new(101, String::from("TECH-KEY-001"), String::from("Mechanical Keyboard"), ProductCategory::Electronics, 12000, 5);
    let customer = Customer::new(301, String::from("Margaret Hamilton"), String::from("margaret@apollo.nasa.gov"), Some(String::from("+1-555-0199")), true);

    // Using trait bounds
    println!("Tax Summary: {}", format_tax_summary(&keyboard, keyboard.price_cents));
    println!("Customer Summary: {}", summarize_item(&customer));

    // Using Display trait: {order.order_id} prints #901!
    // println!("Checkout Order {} created successfully!", order.order_id);
}
```

---

## Common Compiler Errors & How to Fix Them

### 1. `error[E0599]: no method named ... found for type ... in the current scope`
**The Mistake**:
```rust
let product = Product::new(...);
product.summary(); // COMPILE ERROR
```
**Why It Happens**:
In Rust, to call methods defined by a trait, **the trait itself must be in scope**!
**The Fix**:
Bring the trait into scope with a `use` statement:
```rust
use ministore::Summarizable; // Now product.summary() works!
```

---

### 2. The Orphan Rule (`error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate`)
**The Mistake**:
```rust
// Trying to implement a standard trait (Display) for a standard type (Vec<i32>)
impl std::fmt::Display for Vec<i32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { ... }
}
```
**Why It Happens**:
The **Orphan Rule** states: you can implement a trait on a type **only if either the trait or the type was defined in your current crate**.
- If both the trait (`Display`) and the type (`Vec`) come from external crates/stdlib, you cannot implement them. This prevents two different libraries from defining conflicting implementations for the same type.
**The Fix**:
Use the **Newtype Pattern** (wrap the external type in a local tuple struct):
```rust
struct IntList(Vec<i32>);

impl std::fmt::Display for IntList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { ... }
}
```

---

## Idiomatic Rust Best Practices

1. **Prefer `where` Clauses for Non-Trivial Bounds**: If you have more than one trait bound or multiple generic types, move the bounds to a `where` clause at the end of the function signature.
2. **Provide Sensible Default Implementations**: When designing your own traits, implement common default algorithms (like `calculate_tax`) so implementors only need to supply the essential data.
3. **Implement Standard Traits**: Always derive or implement standard traits (`Debug`, `Clone`, `PartialEq`, `Display`, `Default`) for your domain types. This makes your types feel natural to other Rust developers.
4. **Keep Traits Focused (Single Responsibility)**: Prefer multiple small, focused traits (`Taxable`, `Summarizable`) over one giant monolithic trait. Small traits are much easier to implement and compose.

---

## Hands-On Exercises

### Exercise 1: Implement `Discountable` Trait
1. Define a trait `Discountable` with a method `discount_rate(&self) -> u32`.
2. Add a default method `apply_discount(&self, price_cents: u32) -> u32`.
3. Implement `Discountable` for `Coupon` and `Customer` (VIP customers get a flat 10% discount).
4. Write a generic function `calculate_final_price<T: Discountable>(item: &T, base_price: u32) -> u32`.

### Exercise 2: Implement `std::fmt::Display` for `Product`
1. Manually implement `std::fmt::Display` for `Product` in `src/models/product.rs`.
2. Format it as `"[SKU] Title - $Price"`.
3. Update `src/main.rs` to print products using `{product}` instead of manual field access.

---

## Checkpoint

Run the compiler checks and verify all 20 unit tests pass:
```bash
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
cargo run
```

Expected test output:
```text
running 20 tests
test tests::test_cart_item_option_lookup ... ok
test tests::test_catalog_option_lookups ... ok
test tests::test_checkout_error_propagation_and_success ... ok
test tests::test_coupon_discount_and_take ... ok
test tests::test_coupon_validation_error ... ok
test tests::test_customer_optional_phone ... ok
test tests::test_display_trait_implementations ... ok
test tests::test_generic_api_response_wrapper ... ok
test tests::test_generic_catalog_product_pagination ... ok
test tests::test_generic_page_map_transformation ... ok
test tests::test_generic_pagination_with_integers ... ok
test tests::test_generic_trait_bound_functions ... ok
test tests::test_order_cancellation_prevention ... ok
test tests::test_order_status_valid_lifecycle ... ok
test tests::test_order_total_with_payment_fee ... ok
test tests::test_payment_method_fees_and_descriptions ... ok
test tests::test_product_category_tax_rates ... ok
test tests::test_product_stock_reduction_error ... ok
test tests::test_summarizable_trait_on_domain_models ... ok
test tests::test_taxable_trait_and_default_method ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Expected binary output:
```text
=== MiniStore: Traits & Shared Behavior (Part III) ===

1. Catalog initialized with 3 products.

2. Browsing Catalog with Generic Pagination (Page<Product>):
   Page 1 of 2 (Total Items: 3)
   - [TECH-KEY-001] Tenkeyless Mechanical Keyboard ($120.00)
   - [TECH-MOU-002] Ergonomic Wireless Mouse ($45.00)
   Has next page? true
   Transformed to Page<String>: ["Tenkeyless Mechanical Keyboard", "Ergonomic Wireless Mouse"]
   API Page 2 response: 1 product(s) returned out of 3 total.

3. Customer: Margaret Hamilton (+1-555-0199)

4. Shared Behaviors via Traits:
   Tax Summary: Product #101: Tenkeyless Mechanical Keyboard [TECH-KEY-001] - $120.00 | Tax: $18.00 (15%)
   Tax Summary: Product #102: Ergonomic Wireless Mouse [TECH-MOU-002] - $45.00 | Tax: $6.75 (15%)
   Customer Summary: Customer #301: Margaret Hamilton <margaret@apollo.nasa.gov>

5. Processing checkout through modular services...
   Checkout Order #901 created successfully!
   Subtotal: $210.00 | Total: $148.50

6. Order Lifecycle Transitions:
   Order confirmed: Confirmed (Receipt: REC-901-HAMILTON)
   Order shipped:   Shipped (Tracking: TRK-FEDEX-77189)
   Cancellation prevented -> Cannot perform action 'cancel' while order is in 'Shipped (Tracking: TRK-FEDEX-77189)' state
   Final Lifecycle State: Delivered to Customer
   Order Summary: Order #901: 2 item(s), Total: $148.50 [Delivered to Customer]
```

---

## What We Learned
- Why Rust chose traits over class-based inheritance to achieve flexible, modular polymorphism.
- How to define traits with required and default method implementations.
- Constraining generic types using trait bounds (`<T: Trait>`), multiple bounds (`+`), and `where` clauses.
- The importance of standard traits like `Display`, `Debug`, and `std::error::Error`.
- How the Orphan Rule safeguards crate boundaries and prevents conflicting implementations.

---

## What's Next
Now that we can define generic types and constrain them with traits, our programs are powerful and expressive. 
However, when functions and structs hold references rather than owned values, the compiler must guarantee that those references never outlive the data they point to!
In **Chapter 16: Lifetimes**, we will demystify Rust's lifetime annotations (`'a`) and learn how the borrow checker guarantees memory safety across reference boundaries!
