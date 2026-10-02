# Chapter 14: Generics

## What You'll Learn
- What **Generics** are and why they are essential for writing reusable, type-safe code without duplicating logic.
- How to define and invoke **Generic Functions** using type parameter syntax (`fn foo<T>(item: T)`).
- How to design **Generic Structs** (such as `Page<T>` or `Pair<K, V>`) that hold data of any type.
- How generic types empower standard enums like **`Option<T>`** and **`Result<T, E>`**, and how to write custom generic enums like **`ApiResponse<T>`**.
- Implementing methods on generic types using `impl<T> StructName<T>`, and introducing secondary type parameters on methods (`fn map<U>(self, f: fn(T) -> U) -> Page<U>`).
- How Rust achieves **Zero-Cost Abstractions** via **Monomorphization** at compile time: generating dedicated, fully-optimized machine code for each concrete type without runtime overhead or garbage-collection boxing.
- Applying generics to MiniStore:
  - Designing a reusable, universal **`Page<T>`** container.
  - Writing a generic **`paginate<T>`** function that chunks any collection into pages.
  - Adding catalog pagination for products (`catalog.paginate(page, per_page)`).
  - Building a generic **`ApiResponse<T>`** transport wrapper for e-commerce API endpoints.

---

## Why Do We Need This?
Consider the common requirements of an e-commerce platform like MiniStore:
1. Customers need to browse products across numbered pages (`ProductPage`).
2. Warehouse staff need to review incoming orders in paginated lists (`OrderPage`).
3. Store administrators need to audit customer accounts across pages (`CustomerPage`).

### The Problem: Code Duplication
Without generics, how would we write this? We would be forced to create a separate struct for every type:

```rust
// A page of Products
pub struct ProductPage {
    pub items: Vec<Product>,
    pub page: usize,
    pub per_page: usize,
    pub total_items: usize,
}

// A page of Orders
pub struct OrderPage {
    pub items: Vec<Order>,
    pub page: usize,
    pub per_page: usize,
    pub total_items: usize,
}

// A page of Customers
pub struct CustomerPage {
    pub items: Vec<Customer>,
    pub page: usize,
    pub per_page: usize,
    pub total_items: usize,
}
```

Notice the problem:
- The pagination logic (`total_pages()`, `has_next()`, `has_previous()`, slicing bounds) is **100% identical** across all three structs.
- If we want to paginate 10 different types, we must write and maintain 10 copy-pasted structs and 10 sets of identical methods!
- If a bug is discovered in our `total_pages()` calculation, we must manually locate and fix it in 10 different places.

### The Traditional Dynamic Fallback (and why Rust rejects it)
In dynamically typed languages (like Python or JavaScript), functions accept anything, but you lose compile-time type safety entirely.

In languages like older Java or C, developers used generic base pointers like `void*` or `Object`:
```java
// Java pre-generics (Object boxing)
public class Page {
    public Object[] items;
    // ...
}
```
This approach brings severe penalties:
1. **Loss of Type Safety**: You can accidentally insert a `Customer` into a product page, and the compiler will not notice until the program crashes at runtime with a `ClassCastException`.
2. **Runtime Overhead**: Primitive types must be "boxed" into heap-allocated objects, adding pointer chasing and cache misses.

**The Rust Solution**: **Generics.**
Generics allow you to write algorithms and data structures using **type placeholders** (like `T`). The compiler guarantees compile-time type safety and delivers **maximum runtime performance** with zero boxing overhead.

---

## Core Rust Concept: Generics

Generics allow us to replace concrete types (like `u32`, `String`, or `Product`) with an abstract type parameter, conventionally named `T` (short for *Type*).

### 1. Generic Functions
To make a function generic, specify the type parameter in angle brackets `<T>` immediately after the function name and before the parameter list:

```rust
// Concrete function: only works on i32
fn first_i32(list: &[i32]) -> Option<&i32> {
    list.first()
}

// Generic function: works on ANY type T!
fn first_element<T>(list: &[T]) -> Option<&T> {
    list.first()
}

fn main() {
    let numbers = vec![10, 20, 30];
    let names = vec![String::from("Alice"), String::from("Bob")];

    // Rust automatically infers T = i32
    let num = first_element(&numbers);

    // Rust automatically infers T = String
    let name = first_element(&names);
}
```

Notice:
- `T` stands for any type that the caller chooses.
- You do not need to specify `first_element::<i32>(&numbers)` explicitly (though you can using the *turbofish* syntax `::<T>`); the Rust compiler infers `T` from the arguments passed.

---

## Generic Structs

You can use generic type parameters in `struct` definitions to create universal containers:

```rust
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: usize,
    pub per_page: usize,
    pub total_items: usize,
}
```

Now, `Page<Product>`, `Page<Order>`, `Page<Customer>`, and even `Page<i32>` all share the same definition:
```rust
let product_page: Page<Product> = Page {
    items: vec![/* products */],
    page: 1,
    per_page: 10,
    total_items: 45,
};

let number_page: Page<u32> = Page {
    items: vec![1, 2, 3],
    page: 1,
    per_page: 3,
    total_items: 3,
};
```

### Multiple Type Parameters
If a struct needs more than one placeholder type, you can list multiple parameters separated by commas:

```rust
pub struct KeyValue<K, V> {
    pub key: K,
    pub value: V,
}

fn main() {
    // K is String, V is u32
    let item_price = KeyValue {
        key: String::from("TECH-KEY-001"),
        value: 12000,
    };

    // K is u64 (ID), V is bool (active status)
    let user_status = KeyValue {
        key: 301,
        value: true,
    };
}
```

---

## Generic Enums: You Already Know Them!

If generics feel new, you have actually been using them since Chapter 11 and 12!

Look at the standard library definitions of `Option` and `Result`:
```rust
// Option has one generic type parameter: T
pub enum Option<T> {
    Some(T),
    None,
}

// Result has two generic type parameters: T (success) and E (error)
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
```
Because `Option<T>` is generic, it can hold an integer (`Option<u32>`), a customer (`Option<Customer>`), or a cart (`Option<ShoppingCart>`) without needing custom enums for each.

### Custom Generic Enums
In MiniStore, we can define a generic enum for API responses:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiResponse<T> {
    Success { data: T, total: usize },
    Error { message: String },
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T, total: usize) -> Self {
        Self::Success { data, total }
    }

    pub fn err(message: String) -> Self {
        Self::Error { message }
    }
}
```

---

## Methods on Generic Types (`impl<T>`)

When declaring methods for a generic struct, you must declare the type parameter after `impl` so Rust knows that `T` is a generic type rather than a concrete type:

```rust
impl<T> Page<T> {
    pub fn new(items: Vec<T>, page: usize, per_page: usize, total_items: usize) -> Self {
        Self { items, page, per_page, total_items }
    }

    pub fn total_pages(&self) -> usize {
        if self.per_page == 0 {
            0
        } else {
            self.total_items.div_ceil(self.per_page)
        }
    }

    pub fn has_next(&self) -> bool {
        self.page > 0 && self.page < self.total_pages()
    }

    pub fn has_previous(&self) -> bool {
        self.page > 1 && self.page <= self.total_pages() + 1
    }
}
```

### Introducing New Type Parameters in Methods
A method on a generic struct `Page<T>` can introduce its *own* additional generic parameter `U`. 

For example, transforming a `Page<T>` into a `Page<U>` by mapping each element:
```rust
impl<T> Page<T> {
    /// Transforms Page<T> into Page<U> using a transformation function
    pub fn map<U>(self, transform: fn(T) -> U) -> Page<U> {
        let mapped_items: Vec<U> = self.items.into_iter().map(transform).collect();
        Page {
            items: mapped_items,
            page: self.page,
            per_page: self.per_page,
            total_items: self.total_items,
        }
    }
}
```
Here, `T` is defined at the struct level, while `U` is declared specifically for the `map` method!

---

## Mental Model: Monomorphization (Zero-Cost Abstractions)

How does Rust execute generic code so quickly?

In languages like Java or Python, generic types incur runtime penalties:
- **Java**: Uses *Type Erasure*. At compile time, `List<Integer>` and `List<String>` are stripped down to `List<Object>`. Primitive types must be boxed, and every access requires a hidden runtime type cast.
- **Python**: Dynamically evaluates types on every instruction, resulting in massive interpreter overhead.

### How Rust Compiles Generics
Rust uses a process called **Monomorphization** (turning generic code into mono-morphic, or single-formed, code):

```
                      Generic Source Code:
                         Page<T>
                            │
               Compile-Time Monomorphization
                            │
             ┌──────────────┴──────────────┐
             ▼                             ▼
Generated for T = Product:      Generated for T = i32:
     struct Page_Product {          struct Page_i32 {
         items: Vec<Product>,           items: Vec<i32>,
         page: usize,                   page: usize,
         ...                            ...
     }                              }
```

1. During compilation, `rustc` scans your entire project to see which concrete types are used with `Page<T>`.
2. For each concrete type used (e.g., `Page<Product>` and `Page<i32>`), the compiler generates a specialized, dedicated copy of the struct and its methods behind the scenes.
3. Every method call is direct and can be fully inlined by LLVM.

### The Tradeoff
- **Runtime Performance**: **Zero overhead.** Generic code runs at the exact same speed as handwritten, type-specific code. No boxing, no runtime checks, no pointer chasing.
- **Binary Size**: Because the compiler duplicates functions for each concrete type, the resulting compiled binary can be slightly larger ("code bloat"). In practice, this cost is minimal compared to the enormous speed and memory benefits.

---

## Comparison: Generics Across Languages

| Language | Mechanism | Type Safety | Runtime Overhead | Code Bloat |
| :--- | :--- | :--- | :--- | :--- |
| **Rust** | **Monomorphization** at compile time | 100% checked before code generation | **Zero runtime cost** (direct calls, inlining) | Slight increase in binary size |
| **C++** | Templates (duck typing at instantiation) | Checked only when instantiated | Zero runtime cost | High template bloat |
| **Java** | Type Erasure (erased to `Object`) | Compile-time check, runtime casting | High (boxing, pointer chasing, cache misses) | None (single class file) |
| **Go (1.18+)** | GC-shape stashing + dictionary passing | Checked at compile time | Low to moderate (interface dictionary lookups) | Low |
| **TypeScript** | Type Erasure (stripped to plain JS) | Compile-time only (bypassed by `any`) | None added by TS, but JS is dynamically typed | None |

---

## MiniStore Implementation

In MiniStore, we introduce generic pagination and API response envelopes.

### 1. `src/models/page.rs`
Defines `Page<T>`, the universal `paginate<T>` slicing function, and `ApiResponse<T>`:

```rust
/// Generic pagination container for MiniStore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: usize,
    pub per_page: usize,
    pub total_items: usize,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, page: usize, per_page: usize, total_items: usize) -> Self {
        Self {
            items,
            page,
            per_page,
            total_items,
        }
    }

    pub fn total_pages(&self) -> usize {
        if self.per_page == 0 {
            0
        } else {
            self.total_items.div_ceil(self.per_page)
        }
    }

    pub fn has_next(&self) -> bool {
        self.page > 0 && self.page < self.total_pages()
    }

    pub fn has_previous(&self) -> bool {
        self.page > 1 && self.page <= self.total_pages() + 1
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Transforms a `Page<T>` into a `Page<U>` using a function pointer.
    pub fn map<U>(self, transform: fn(T) -> U) -> Page<U> {
        let mapped_items = self.items.into_iter().map(transform).collect();
        Page {
            items: mapped_items,
            page: self.page,
            per_page: self.per_page,
            total_items: self.total_items,
        }
    }
}

/// Generic function that partitions an owned vector into a paginated container.
pub fn paginate<T>(items: Vec<T>, page: usize, per_page: usize) -> Page<T> {
    let total_items = items.len();
    if per_page == 0 || page == 0 {
        return Page::new(Vec::new(), page, per_page, total_items);
    }

    let start_index = (page - 1) * per_page;
    if start_index >= total_items {
        return Page::new(Vec::new(), page, per_page, total_items);
    }

    let end_index = (start_index + per_page).min(total_items);
    let page_items: Vec<T> = items
        .into_iter()
        .skip(start_index)
        .take(end_index - start_index)
        .collect();

    Page::new(page_items, page, per_page, total_items)
}

/// Generic API response wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiResponse<T> {
    Success { data: T, total: usize },
    Error { message: String },
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T, total: usize) -> Self {
        Self::Success { data, total }
    }

    pub fn err(message: String) -> Self {
        Self::Error { message }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    pub fn data(&self) -> Option<&T> {
        match self {
            Self::Success { data, .. } => Some(data),
            Self::Error { .. } => None,
        }
    }
}
```

### 2. Updating `src/catalog.rs`
We add pagination to `Catalog`:

```rust
impl Catalog {
    // ...

    /// Returns a sorted list of all products in the catalog by ID.
    pub fn get_products(&self) -> Vec<Product> {
        let mut list: Vec<Product> = self.products.values().cloned().collect();
        list.sort_by_key(|p| p.id);
        list
    }

    /// Paginates products using the generic `Page<T>` abstraction.
    pub fn paginate(&self, page: usize, per_page: usize) -> crate::models::Page<Product> {
        crate::models::paginate(self.get_products(), page, per_page)
    }
}
```

### 3. Demonstrating Generics in `src/main.rs`

```rust
use ministore::{
    checkout, ApiResponse, Catalog, Coupon, Customer, OrderId, Page, PaymentMethod, Product,
    ProductCategory, ShoppingCart,
};

fn main() {
    println!("=== MiniStore: Generics & Monomorphization (Part III) ===\n");

    // 1. Initializing Catalog with Products
    let mut catalog = Catalog::new();
    catalog.add_product(Product::new(101, String::from("TECH-KEY-001"), String::from("Tenkeyless Mechanical Keyboard"), ProductCategory::Electronics, 12000, 5));
    catalog.add_product(Product::new(102, String::from("TECH-MOU-002"), String::from("Ergonomic Wireless Mouse"), ProductCategory::Electronics, 4500, 10));
    catalog.add_product(Product::new(103, String::from("TECH-MON-003"), String::from("27-inch 4K IPS Display"), ProductCategory::Electronics, 35000, 3));

    println!("1. Catalog initialized with {} products.", catalog.total_products());

    // 2. Generic Pagination Demonstration (Page<Product>)
    println!("\n2. Browsing Catalog with Generic Pagination (Page<Product>):");
    let product_page: Page<Product> = catalog.paginate(1, 2);
    println!(
        "   Page {} of {} (Total Items: {})",
        product_page.page,
        product_page.total_pages(),
        product_page.total_items
    );
    for item in &product_page.items {
        println!(
            "   - [{}] {} (${:.2})",
            item.sku, item.name, item.price_cents as f64 / 100.0
        );
    }
    println!("   Has next page? {}", product_page.has_next());

    // Generic transformation: Page<Product> -> Page<String>
    let name_page: Page<String> = product_page.map(|p| p.name);
    println!("   Transformed to Page<String>: {:?}", name_page.items);

    // Generic API response container
    let api_response = ApiResponse::ok(catalog.paginate(2, 2), catalog.total_products());
    if let ApiResponse::Success { data, total } = api_response {
        println!(
            "   API Page 2 response: {} product(s) returned out of {} total.",
            data.item_count(),
            total
        );
    }

    // 3. Customer Profile
    let customer = Customer::new(
        301,
        String::from("Margaret Hamilton"),
        String::from("margaret@apollo.nasa.gov"),
        Some(String::from("+1-555-0199")),
        true,
    );
    println!("\n3. Customer: {} ({})", customer.name, customer.formatted_phone());

    // 4. Shopping Cart setup
    let mut cart = ShoppingCart::new();
    cart.add_item(101, 1, 12000);
    cart.add_item(102, 2, 4500);

    let coupon = Coupon::new(String::from("LAUNCH20"), 20);

    // 5. Checkout
    println!("\n4. Processing checkout through modular services...");
    match checkout(
        OrderId(901),
        customer,
        &mut cart,
        &mut catalog,
        PaymentMethod::CreditCard { last_four: String::from("9876") },
        Some(coupon),
    ) {
        Ok(mut order) => {
            println!("   Checkout Order #{} created successfully!", order.order_id.0);
            println!(
                "   Subtotal: ${:.2} | Total: ${:.2}",
                order.subtotal_cents() as f64 / 100.0,
                order.total_cents() as f64 / 100.0
            );

            // 6. Order Lifecycle Transitions
            println!("\n5. Order Lifecycle Transitions:");
            order.confirm(String::from("REC-901-HAMILTON")).unwrap();
            println!("   Order confirmed: {}", order.status.display_status());

            order.ship(String::from("TRK-FEDEX-77189")).unwrap();
            println!("   Order shipped:   {}", order.status.display_status());

            match order.cancel(String::from("Buyer changed mind")) {
                Ok(()) => println!("   Order cancelled!"),
                Err(err) => println!("   Cancellation prevented -> {}", err.message()),
            }

            order.mark_delivered().unwrap();
            println!("   Final Lifecycle State: {}", order.status.display_status());
        }
        Err(err) => println!("   Checkout failed: {}", err.message()),
    }
}
```

---

## Common Compiler Errors & How to Fix Them

### 1. `error[E0412]: cannot find type in this scope`
**The Mistake**:
```rust
struct Page {
    items: Vec<T>, // Compiler doesn't know what `T` is!
}
```
**The Fix**:
You must declare the type parameter `<T>` after the struct name:
```rust
struct Page<T> {
    items: Vec<T>,
}
```

---

### 2. `error[E0107]: struct takes 1 generic argument but 0 generic arguments were supplied`
**The Mistake**:
```rust
fn print_page(p: Page) { /* ... */ }
```
**The Fix**:
A generic struct is not a complete concrete type by itself—it is a *blueprint* for a type. You must either specify the concrete type or pass along a generic parameter:
```rust
// Specifying concrete type:
fn print_page(p: Page<Product>) { /* ... */ }

// Or keeping the function generic:
fn print_page<T>(p: Page<T>) { /* ... */ }
```

---

### 3. Missing `<T>` on `impl` block
**The Mistake**:
```rust
impl Page<T> { // Error: cannot find type `T` in this scope
    fn len(&self) -> usize { self.items.len() }
}
```
**The Fix**:
Always write `impl<T> Page<T>`. The first `<T>` declares the generic type parameter for the implementation block, and the second `Page<T>` specifies that it applies to the generic struct:
```rust
impl<T> Page<T> {
    fn len(&self) -> usize { self.items.len() }
}
```

---

## Idiomatic Rust Best Practices

1. **Use Single Uppercase Letters for Placeholders**: By Rust convention, generic parameters are single uppercase letters (`T` for Type, `E` for Error, `K` for Key, `V` for Value). If more descriptive names are needed, use CamelCase (e.g., `Item`, `Payload`).
2. **Do Not Over-Genericize Early**: Only reach for generics when you have an actual need to reuse the same data structure or logic across multiple distinct types. Unnecessary generics add cognitive overhead to your code.
3. **Rely on Type Inference**: Let the compiler deduce generic types automatically whenever possible. Use explicit turbofish syntax (`parse::<u32>()`) only when the compiler cannot infer the target type.
4. **Leverage Monomorphization for High-Throughput Paths**: Because generic methods are inlined by LLVM, you get maximum possible CPU performance with zero virtual dispatch penalty.

---

## Hands-On Exercises

### Exercise 1: Universal Search Filter
Write a generic function `filter_by<T>(items: Vec<T>, predicate: fn(&T) -> bool) -> Vec<T>` that takes ownership of a vector and returns only items that satisfy the predicate function. Test it with both a vector of integers and a vector of products.

### Exercise 2: Generic `Cache<K, V>` Struct
Create a generic `Cache<K, V>` struct in `src/models/cache.rs` backed by a `std::collections::HashMap<K, V>`:
- Add a constructor `Cache::new()`.
- Add `insert(&mut self, key: K, value: V)`.
- Add `get(&self, key: &K) -> Option<&V>`.
- Test caching customer lookups and product prices!

---

## Checkpoint

Run the test suite and verify all 16 tests pass:
```bash
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
cargo run
```

Expected test output:
```text
running 16 tests
test tests::test_cart_item_option_lookup ... ok
test tests::test_catalog_option_lookups ... ok
test tests::test_checkout_error_propagation_and_success ... ok
test tests::test_coupon_discount_and_take ... ok
test tests::test_coupon_validation_error ... ok
test tests::test_customer_optional_phone ... ok
test tests::test_generic_api_response_wrapper ... ok
test tests::test_generic_catalog_product_pagination ... ok
test tests::test_generic_page_map_transformation ... ok
test tests::test_generic_pagination_with_integers ... ok
test tests::test_order_cancellation_prevention ... ok
test tests::test_order_status_valid_lifecycle ... ok
test tests::test_order_total_with_payment_fee ... ok
test tests::test_payment_method_fees_and_descriptions ... ok
test tests::test_product_category_tax_rates ... ok
test tests::test_product_stock_reduction_error ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Expected binary output:
```text
=== MiniStore: Generics & Monomorphization (Part III) ===

1. Catalog initialized with 3 products.

2. Browsing Catalog with Generic Pagination (Page<Product>):
   Page 1 of 2 (Total Items: 3)
   - [TECH-KEY-001] Tenkeyless Mechanical Keyboard ($120.00)
   - [TECH-MOU-002] Ergonomic Wireless Mouse ($45.00)
   Has next page? true
   Transformed to Page<String>: ["Tenkeyless Mechanical Keyboard", "Ergonomic Wireless Mouse"]
   API Page 2 response: 1 product(s) returned out of 3 total.

3. Customer: Margaret Hamilton (+1-555-0199)

4. Processing checkout through modular services...
   Checkout Order #901 created successfully!
   Subtotal: $210.00 | Total: $148.50

5. Order Lifecycle Transitions:
   Order confirmed: Confirmed (Receipt: REC-901-HAMILTON)
   Order shipped:   Shipped (Tracking: TRK-FEDEX-77189)
   Cancellation prevented -> Cannot perform action 'cancel' while order is in 'Shipped (Tracking: TRK-FEDEX-77189)' state
   Final Lifecycle State: Delivered to Customer
```

---

## What We Learned
- Why generics are essential for the DRY (Don't Repeat Yourself) principle while preserving compile-time type safety.
- How to declare generic functions (`fn func<T>(...)`), structs (`struct Container<T>`), and enums (`enum Envelope<T>`).
- How standard enums `Option<T>` and `Result<T, E>` utilize generics.
- The mechanics of **monomorphization**: how Rust generates specialized, zero-cost machine code for every concrete type used.
- Implementing generic methods with `impl<T>` and introducing new method-level type parameters (`map<U>`).
- Extending MiniStore with a production-grade generic `Page<T>` and `ApiResponse<T>` module.

---

## What's Next
In this chapter, we learned that a generic parameter `T` can stand for *any* type in the universe. 
However, what happens when we need `T` to have specific capabilities—such as being printable, comparable, or cloneable?
In **Chapter 15: Traits**, we will discover Rust's system for defining shared behavior and constraining generic types with trait bounds!
