# Chapter 17: Iterators

## What You'll Learn
- What the **`Iterator` Trait** is and how Rust implements declarative, functional sequence processing.
- The three foundational iterator creation patterns:
  - **`.iter()`**: Yields immutable borrowed references (`&T`).
  - **`.iter_mut()`**: Yields mutable borrowed references (`&mut T`).
  - **`.into_iter()`**: Consumes the collection and yields owned values (`T`).
- Implementing the **`IntoIterator`** trait so custom structs (`ShoppingCart`, `Order`, `Catalog`) can be looped over directly with `for item in &cart`.
- **Lazy Evaluation**: Why iterator adapters perform zero computation until driven by a consumer method.
- **Iterator Adapters** (Functional transformations):
  - Transforming elements with **`.map()`**.
  - Filtering items with **`.filter()`**.
  - Slicing sequences with **`.take()`** and **`.skip()`**.
  - Tracking indices with **`.enumerate()`**.
- **Consumer Methods** (Driving evaluation):
  - Collecting results with **`.collect()`** and using the Turbofish (`::<Vec<_>>()`).
  - Aggregating values with **`.sum()`**, **`.count()`**, and **`.fold()`**.
  - Searching sequences with **`.find()`**, **`.position()`**, **`.any()`**, and **`.all()`**.
  - In-place mutation loops with **`.for_each()`**.
- Creating **Custom Iterators**:
  - Implementing `Iterator` with `type Item` and `fn next(&mut self) -> Option<Self::Item>`.
  - Building `CartReportIterator<'a>` to produce line-by-line formatted reports.
  - Building `DiscountTierIter` to generate progressive stepping discount tiers.
- Why Rust's iterators are a **Zero-Cost Abstraction** that often compiles to assembly faster than hand-written `for` loops with array indexing.
- Integrating Iterators into MiniStore:
  - Adding functional queries to `Catalog` (`filter_by_category`, `products_in_price_range`, `total_inventory_valuation`).
  - Adding bulk operations to `ShoppingCart` (`has_product`, `apply_promotional_discount`, `report_iter`).
  - Implementing `IntoIterator` for `&ShoppingCart`, `&mut ShoppingCart`, `ShoppingCart`, `&Order`, and `&Catalog`.

---

## Why Do We Need This?

In traditional programming languages, iterating over collections is commonly done with indexed `for` loops:

```c
// Traditional C / Java indexed loop
for (int i = 0; i < items.length; i++) {
    process(items[i]);
}
```

While familiar, indexed loops suffer from three notable problems:
1. **Off-by-One Errors**: Accidentally using `<=` instead of `<` causes index out-of-bounds crashes.
2. **Bounds Checking Overhead**: In memory-safe languages, every `items[i]` lookup forces the CPU to check if `i < length` on *every single iteration*, creating CPU branch mispredictions.
3. **Imperative Boilerplate**: Filtering, transforming, and aggregating data requires manually declaring temporary accumulator variables, nested `if` statements, and state tracking.

### Rust's Solution: The `Iterator` Trait
In Rust, collections provide iterators. Iterators allow you to express **what** you want to do (filter, map, sum) rather than micromanaging **how** the index counter increments:

```rust
// Declarative, clean, and blazingly fast
let total: u32 = cart.items
    .iter()
    .filter(|item| item.quantity > 1)
    .map(|item| item.line_total())
    .sum();
```

Because the Rust compiler knows the exact start and end bounds of an iterator, it **completely eliminates runtime bounds checking** and can unroll or vectorize the loop into optimal SIMD instructions!

---

## The `Iterator` Trait Definition

All iterators in Rust implement the `Iterator` trait from the standard library:

```rust
pub trait Iterator {
    type Item; // Associated Type: The type of element yielded

    fn next(&mut self) -> Option<Self::Item>;

    // Dozens of default methods provided for free!
    // map, filter, fold, sum, collect, etc.
}
```

Notice how simple this core contract is:
1. You define `type Item`: what type of value the iterator yields.
2. You implement `fn next(&mut self) -> Option<Self::Item>`:
   - Returns `Some(value)` as long as there are items left.
   - Returns `None` once the sequence is exhausted.

Once you provide `next()`, Rust's standard library provides **over 70 adapter and consumer methods for free**!

---

## The Three Iterator Creation Patterns

When working with collections (like `Vec<T>` or `HashMap<K, V>`), there are three standard ways to create an iterator:

| Method | Yielded Item | Ownership Impact | Collection Usable Afterwards? |
| :--- | :--- | :--- | :--- |
| **`.iter()`** | `&T` | Borrows elements immutably | Yes |
| **`.iter_mut()`** | `&mut T` | Borrows elements mutably | Yes |
| **`.into_iter()`** | `T` | Moves/consumes elements | No (consumed) |

### 1. `.iter()` — Immutable Borrowing
Use when you only need to read data:
```rust
for item in cart.items.iter() {
    println!("Item: {} x ${:.2}", item.quantity, item.unit_price_cents as f64 / 100.0);
}
// `cart` is still valid and untouched here!
```

### 2. `.iter_mut()` — In-Place Mutation
Use when you want to modify elements in place without reallocating the collection:
```rust
// Apply a 10% discount to all items in the cart
cart.items.iter_mut().for_each(|item| {
    item.unit_price_cents = (item.unit_price_cents * 90) / 100;
});
```

### 3. `.into_iter()` — Consuming Ownership
Use when you want to transfer ownership of elements (e.g. converting a cart into order items):
```rust
let items: Vec<CartItem> = cart.into_iter().collect();
// `cart` is now consumed and cannot be used!
```

---

## The `IntoIterator` Trait & `for` Loops

In Rust, the `for` loop syntax is syntactic sugar for `IntoIterator`:
```rust
for x in collection {
    // ...
}
```
Is automatically desugared by the compiler into:
```rust
let mut iter = collection.into_iter();
while let Some(x) = iter.next() {
    // ...
}
```

By implementing `IntoIterator` for `&ShoppingCart`, we allow callers to iterate over a cart directly with `for item in &cart`:

```rust
impl<'a> IntoIterator for &'a ShoppingCart {
    type Item = &'a CartItem;
    type IntoIter = std::slice::Iter<'a, CartItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}
```

Now in user code:
```rust
for item in &cart {
    println!("Product #{}", item.product_id);
}
```

---

## Lazy Evaluation: How Iterators Work

> [!IMPORTANT]
> **In Rust, iterators are lazy!**
> Calling an adapter like `.map()` or `.filter()` does **nothing** on its own.

Consider this code:
```rust
let numbers = vec![1, 2, 3, 4, 5];
numbers.iter().map(|x| println!("Processing {}", x)); // WARNING: unused `Map`
```
If you compile this, Rust gives a compiler warning: `unused `Map` that must be used`.
Nothing is printed to the screen! No work was done!

### Why Laziness Is a Superpower
Because iterators are lazy, you can chain multiple operations together:
```rust
let total: u32 = catalog
    .products
    .values()
    .filter(|p| p.category == ProductCategory::Electronics)
    .filter(|p| p.is_in_stock())
    .map(|p| p.price_cents)
    .sum();
```
Rather than creating intermediate vectors for each step, Rust fuses the entire pipeline into a **single, unified loop**. An item flows through the filter, map, and sum in a single pass directly in CPU registers!

---

## Essential Iterator Adapters & Consumers

### 1. Adapters (Produce a New Iterator)
- **`.map(|x| ...)`**: Transforms each element.
- **`.filter(|x| ...)`**: Keeps only elements that satisfy a predicate.
- **`.take(n)`**: Yields only the first `n` elements.
- **`.skip(n)`**: Bypasses the first `n` elements.
- **`.enumerate()`**: Yields `(index, item)` pairs.
- **`.zip(other)`**: Pairs elements from two iterators until one ends.

### 2. Consumers (Drive Execution to Completion)
- **`.collect()`**: Gathers elements into a collection (like `Vec` or `HashMap`).
- **`.sum()` / `.product()`**: Computes arithmetic totals.
- **`.fold(initial, |acc, x| ...)`**: General reduction accumulation.
- **`.find(|x| ...)`**: Returns `Option<&T>` of the first match.
- **`.any(|x| ...)` / `.all(|x| ...)`**: Boolean short-circuit checks.
- **`.for_each(|x| ...)`**: Executes a side-effect closure on every item.

---

## Building Custom Iterators

Implementing `Iterator` on your own domain types is straightforward and unlocks Rust's full functional toolkit.

### Example 1: `DiscountTierIter`
A progressive discount generator that yields stepped percentages (e.g. 5%, 10%, 15%):

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscountTierIter {
    current: u32,
    step: u32,
    max: u32,
}

impl DiscountTierIter {
    pub fn new(step: u32, max: u32) -> Self {
        Self { current: 0, step, max }
    }
}

impl Iterator for DiscountTierIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let next_val = self.current + self.step;
        if next_val <= self.max {
            self.current = next_val;
            Some(self.current)
        } else {
            None
        }
    }
}
```

Now we can use any standard adapter on our custom type:
```rust
let tiers: Vec<u32> = DiscountTierIter::new(5, 20).collect();
// [5, 10, 15, 20]

let sum_over_ten: u32 = DiscountTierIter::new(5, 25)
    .filter(|&rate| rate > 10)
    .sum();
// 15 + 20 + 25 = 60
```

### Example 2: `CartReportIterator<'a>`
A stateful line-item report generator:
```rust
pub struct CartReportIterator<'a> {
    cart: &'a ShoppingCart,
    index: usize,
}

impl<'a> Iterator for CartReportIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.cart.items.len() {
            let item = &self.cart.items[self.index];
            self.index += 1;
            Some(format!(
                "Item #{}: Product #{} (Qty: {}) - ${:.2}",
                self.index,
                item.product_id,
                item.quantity,
                item.line_total() as f64 / 100.0
            ))
        } else {
            None
        }
    }
}
```

---

## Zero-Cost Abstractions: Iterators vs `for` Loops

Bjarne Stroustrup (creator of C++) defined a Zero-Cost Abstraction as:
> *"What you don't use, you don't pay for. And further: what you do use, you couldn't hand code any better."*

Rust's iterators are one of the purest examples of zero-cost abstractions:

```
Functional Rust:
cart.items.iter().map(|i| i.line_total()).sum()
                       │
               LLVM Optimization
                       │
Compiled Assembly:
Direct vectorized addition loop using SIMD registers with 0 bounds checks
```

In benchmarks, idiomatic Rust iterators frequently run **faster** than indexed `for` loops because the compiler has complete mathematical visibility into the iteration bounds!

---

## Comparison With Other Languages

| Feature | Rust | Java Streams | Python | Go | JavaScript |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Model** | **`Iterator` Trait** | Stream API | Generators / `iter()` | Plain loops (No stream/iterator trait until recent iter package) | Iterators / Generators |
| **Evaluation** | **Lazy** | Lazy | Lazy | Imperative eager | Eager methods (`.map`) / Lazy generators |
| **Runtime Overhead** | **Zero** (Inlined to raw machine code) | Boxing / Heap objects / GC pressure | Dynamic interpreter overhead | N/A | Heap arrays for chained methods |
| **Ownership Awareness** | `.iter()`, `.iter_mut()`, `.into_iter()` | Object references only | References | Values/Pointers | References |

---

## MiniStore Architecture & Code Implementation

Here is how iterators are integrated into MiniStore:

```
ministore/
└── src/
    ├── models/
    │   ├── cart.rs         # iter(), iter_mut(), has_product, IntoIterator, CartReportIterator, DiscountTierIter
    │   ├── order.rs        # iter(), total_quantity, IntoIterator
    │   └── mod.rs          # Re-exports custom iterators
    ├── catalog.rs          # filter_by_category, products_in_price_range, total_inventory_valuation
    ├── lib.rs              # Re-exports & 29 passing unit tests
    └── main.rs             # CLI showing catalog queries, cart loops, and discount tiers
```

### 1. `src/catalog.rs`
```rust
impl Catalog {
    pub fn filter_by_category(&self, category: ProductCategory) -> Vec<&Product> {
        self.products
            .values()
            .filter(|p| p.category == category)
            .collect()
    }

    pub fn products_in_price_range(&self, min_cents: u32, max_cents: u32) -> Vec<&Product> {
        self.products
            .values()
            .filter(|p| p.price_cents >= min_cents && p.price_cents <= max_cents)
            .collect()
    }

    pub fn total_inventory_valuation(&self) -> u64 {
        self.products
            .values()
            .map(|p| (p.price_cents as u64) * (p.stock as u64))
            .sum()
    }
}
```

### 2. `src/models/cart.rs`
```rust
impl ShoppingCart {
    pub fn iter(&self) -> std::slice::Iter<'_, CartItem> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, CartItem> {
        self.items.iter_mut()
    }

    pub fn has_product(&self, product_id: u64) -> bool {
        self.items.iter().any(|item| item.product_id == product_id)
    }

    pub fn apply_promotional_discount(&mut self, percentage: u32) {
        self.items.iter_mut().for_each(|item| {
            let discount = (item.unit_price_cents * percentage) / 100;
            item.unit_price_cents = item.unit_price_cents.saturating_sub(discount);
        });
    }

    pub fn report_iter(&self) -> CartReportIterator<'_> {
        CartReportIterator::new(self)
    }
}
```

---

## Common Compiler Errors & How to Fix Them

### 1. `error[E0282]: type annotations needed` on `.collect()`
**The Mistake**:
```rust
let items = cart.items.iter().map(|item| item.product_id).collect(); // COMPILE ERROR
```
**Why It Happens**:
`.collect()` can build many different collections (`Vec`, `HashSet`, `LinkedList`). Rust doesn't know which one you want!
**The Fix**:
Specify the type either on the variable binding or using the **Turbofish (`::<Vec<_>>`)** syntax:
```rust
// Option A: Type annotation on variable
let items: Vec<u64> = cart.items.iter().map(|item| item.product_id).collect();

// Option B: Turbofish syntax
let items = cart.items.iter().map(|item| item.product_id).collect::<Vec<_>>();
```

---

### 2. Mutating a Collection While Iterating Over It
**The Mistake**:
```rust
for item in &cart.items {
    if item.quantity == 0 {
        cart.remove_item(item.product_id); // COMPILE ERROR: cannot borrow cart as mutable
    }
}
```
**Why It Happens**:
`&cart.items` holds an immutable borrow on the cart. Calling `remove_item` requires a mutable borrow. Rust's aliasing rules prevent concurrent mutation and iteration!
**The Fix**:
Use `retain` on `Vec`, or collect keys to remove into a separate vector first:
```rust
cart.items.retain(|item| item.quantity > 0);
```

---

## Idiomatic Rust Best Practices

1. **Prefer Iterator Chains Over Manual Indexing**: Chained methods (`.filter().map().sum()`) are cleaner, more declarative, eliminate off-by-one errors, and allow LLVM to generate optimal assembly.
2. **Use `.for_each()` Only When Side Effects Are Intended**: If you are computing a value, use adapters like `.map()` and consumer methods like `.sum()` or `.collect()`. Use `.for_each()` only when performing mutation or I/O.
3. **Avoid Unnecessary Collections**: Don't call `.collect::<Vec<_>>()` in the middle of a pipeline unless you specifically need random access or sorting. Let the pipeline stay lazy until the final consumer.
4. **Implement `IntoIterator` for Your Domain Collections**: Implementing `IntoIterator` for `&MyCollection` makes your types feel native to Rust's `for` loops.

---

## Hands-On Exercises

### Exercise 1: Finding the Most Expensive Cart Item
1. Write a method on `ShoppingCart`: `pub fn most_expensive_item(&self) -> Option<&CartItem>`.
2. Use the iterator method `.max_by_key(|item| item.unit_price_cents)` to implement it cleanly in a single line.

### Exercise 2: SKU Search Filter
1. In `Catalog`, implement `pub fn search_by_sku_prefix(&self, prefix: &str) -> Vec<&Product>`.
2. Use `.values().filter(|p| p.sku.starts_with(prefix)).collect()`.

---

## Checkpoint

Run compiler checks and verify all 29 unit tests pass:
```bash
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
cargo run
```

Expected test output:
```text
running 29 tests
test tests::test_cart_item_option_lookup ... ok
test tests::test_catalog_iterator_queries ... ok
test tests::test_catalog_option_lookups ... ok
test tests::test_checkout_error_propagation_and_success ... ok
test tests::test_coupon_discount_and_take ... ok
test tests::test_coupon_validation_error ... ok
test tests::test_custom_cart_report_iterator ... ok
test tests::test_custom_discount_tier_iter ... ok
test tests::test_customer_optional_phone ... ok
test tests::test_display_trait_implementations ... ok
test tests::test_generic_api_response_wrapper ... ok
test tests::test_generic_catalog_product_pagination ... ok
test tests::test_generic_page_map_transformation ... ok
test tests::test_generic_pagination_with_integers ... ok
test tests::test_generic_trait_bound_functions ... ok
test tests::test_lifetime_annotated_contact_resolution ... ok
test tests::test_lifetime_annotated_product_comparison ... ok
test tests::test_order_cancellation_prevention ... ok
test tests::test_order_iterator_methods ... ok
test tests::test_order_status_valid_lifecycle ... ok
test tests::test_order_total_with_payment_fee ... ok
test tests::test_payment_method_fees_and_descriptions ... ok
test tests::test_product_category_tax_rates ... ok
test tests::test_product_stock_reduction_error ... ok
test tests::test_shopping_cart_iterators_and_into_iterator ... ok
test tests::test_static_lifetime_policy ... ok
test tests::test_summarizable_trait_on_domain_models ... ok
test tests::test_taxable_trait_and_default_method ... ok
test tests::test_zero_copy_order_receipt_and_traits ... ok

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Expected CLI output:
```text
=== MiniStore: Iterators & Functional Pipelines (Part III) ===

1. Catalog initialized with 3 products.

2. Catalog Iteration & Functional Queries:
   Found 3 Electronics product(s): ["Ergonomic Wireless Mouse", "27-inch 4K IPS Display", "Tenkeyless Mechanical Keyboard"]
   Products under $150: Ergonomic Wireless Mouse ($45.00), Tenkeyless Mechanical Keyboard ($120.00)
   Total Inventory Valuation: $2100.00

3. Browsing Catalog with Generic Pagination (Page<Product>):
   Page 1 of 2 (Total Items: 3)
   - [TECH-KEY-001] Tenkeyless Mechanical Keyboard ($120.00)
   - [TECH-MOU-002] Ergonomic Wireless Mouse ($45.00)
   Has next page? true
   Transformed to Page<String>: ["Tenkeyless Mechanical Keyboard", "Ergonomic Wireless Mouse"]
   API Page 2 response: 1 product(s) returned out of 3 total.

4. Customer: Margaret Hamilton (+1-555-0199)

5. Lifetimes & Reference Safety:
   Higher priced item: Tenkeyless Mechanical Keyboard ($120.00)
   Best contact info: +1-555-0199
   Store Policy ('static): MiniStore Guarantee: 30-Day Hassle-Free Returns & 1-Year Warranty

6. Shared Behaviors via Traits:
   Tax Summary: Product #101: Tenkeyless Mechanical Keyboard [TECH-KEY-001] - $120.00 | Tax: $18.00 (15%)
   Tax Summary: Product #102: Ergonomic Wireless Mouse [TECH-MOU-002] - $45.00 | Tax: $6.75 (15%)
   Customer Summary: Customer #301: Margaret Hamilton <margaret@apollo.nasa.gov>

7. Cart Iteration (IntoIterator & Custom CartReportIterator):
   Iterating over cart items directly:
   -> Product ID #101: Qty 1 @ $120.00 each
   -> Product ID #102: Qty 2 @ $45.00 each
   Custom Cart Line Item Reports:
      Item #1: Product #101 (Qty: 1) - $120.00
      Item #2: Product #102 (Qty: 2) - $90.00
   Progressive Discount Tiers available: [5, 10, 15, 20]%

8. Processing checkout through modular services...
   Checkout Order #901 created successfully!
   Subtotal: $210.00 | Total Units: 3 | Final Total: $148.50

9. Order Lifecycle Transitions:
   Order confirmed: Confirmed (Receipt: REC-901-HAMILTON)
   Order shipped:   Shipped (Tracking: TRK-FEDEX-77189)
   Cancellation prevented -> Cannot perform action 'cancel' while order is in 'Shipped (Tracking: TRK-FEDEX-77189)' state
   Final Lifecycle State: Delivered to Customer
   Order Summary: Order #901: 2 item(s), Total: $148.50 [Delivered to Customer]

10. Zero-Copy Receipt Borrowing Order & Slices (OrderReceipt<'a>):
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
- How the `Iterator` trait abstracts sequence iteration via `next()`.
- The distinction between `.iter()`, `.iter_mut()`, and `.into_iter()`.
- How `IntoIterator` empowers custom domain structs to participate natively in `for` loops.
- Why lazy evaluation enables loop fusion and zero-cost abstractions.
- How to implement custom iterators (`DiscountTierIter`, `CartReportIterator`) for specialized domain pipelines.

---

## What's Next
Iterators become even more expressive when combined with **Closures**—anonymous functions that can capture values from their surrounding environment!
In **Chapter 18: Closures**, we will explore closure syntax, environment capture modes (`Fn`, `FnMut`, `FnOnce`), and how closures power MiniStore's dynamic pricing and validation engines!
