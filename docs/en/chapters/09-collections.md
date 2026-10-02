# Chapter 9: Collections

## What You'll Learn
- Why dynamic heap-allocated collections are required beyond fixed-size stack arrays.
- The standard heap collection tri-factor: **`Vec<T>`**, **`HashMap<K, V>`**, and **`HashSet<T>`**.
- **Vector Internal Anatomy**: pointer, length, capacity, and doubling reallocation mechanics.
- **The Entry API**: idiomatic, zero-lookup-duplication mutations in `HashMap`.
- **Deduplication and Membership**: fast $O(1)$ set queries with `HashSet`.
- The Borrow Checker and Collections: avoiding iterator invalidation and dangling pointers during reallocation.
- How vectors seamlessly coerce to slices (`&Vec<T>` -> `&[T]`).
- Applying collections to MiniStore: building a dynamic multi-item `ShoppingCart`, an $O(1)$ SKU lookup index, department catalog aggregation, and unique customer tags.

---

## Why Do We Need This?
In previous chapters, we modeled single products and used fixed-size arrays like `[u32; 4]`.
However, real-world e-commerce systems cannot predict how many items a shopper will place in their cart or how many products exist in the store catalog:

1. A customer might buy 1 item or 50 items. A fixed array `[CartItem; 10]` would either waste memory or crash when the 11th item is added.
2. Looking up a product by SKU linearly across thousands of items in an array takes $O(N)$ time. We need $O(1)$ key-value lookups.
3. Tracking unique customer interests or tags (e.g. `"vip"`, `"early_adopter"`) requires set semantics where duplicates are automatically rejected.

```text
Fixed Stack Array [T; 4]                 Dynamic Heap Vector Vec<T>
┌────────┬────────┬────────┬────────┐    Stack (24B)             Heap (Dynamically Sized)
│ item 0 │ item 1 │ item 2 │ item 3 │    ┌─────┬─────┬─────┐     ┌───────┬───────┬───────┬───
└────────┴────────┴────────┴────────┘    │ ptr │ len │ cap │ ──► │ val 0 │ val 1 │ val 2 │...
Fixed at compile time                    └─────┴─────┴─────┘     └───────┴───────┴───────┴───
```

**Rust's Solution**: Standard Library Collections (`std::collections`). Built on top of Rust's ownership system, these collections manage dynamic heap memory safely without requiring a Garbage Collector.

---

## The Problem
In languages without Rust's ownership model:
- **C++**: `std::vector` reallocates heap memory when capacity is exceeded. If another part of the program holds a reference or pointer to an element inside the vector (`int* p = &vec[0];`), calling `vec.push_back(...)` silently invalidates the pointer, leading to **dangling pointers, memory corruption, and security exploits**.
- **Java / Python**: Every collection stores references to heap objects. Modifying a collection while iterating over it throws `ConcurrentModificationException` in Java or corrupts iteration in Python.

MiniStore needs:
- A dynamic shopping cart where items can be added, updated, or removed in real time.
- Fast catalog searching by SKU without scanning the entire product list.
- Absolute compile-time prevention of iterator invalidation and dangling references.

---

## Rust Concept

### 1. Dynamic Vectors: `Vec<T>`
A vector is a resizable array allocated on the heap. Like `String`, a `Vec<T>` is represented on the stack by three 64-bit words (24 bytes):
1. A pointer (`ptr`) to the heap buffer.
2. The current number of elements (`len`).
3. The total allocated space before reallocation is needed (`capacity`).

```rust
// 1. Creating an empty vector:
let mut items: Vec<CartItem> = Vec::new();

// 2. Creating with initial elements via the vec! macro:
let mut numbers = vec![10, 20, 30];

// 3. Pushing elements (grows dynamically on the heap):
numbers.push(40);
```

#### Vector Reallocation Mechanics
When `len == capacity` and you call `.push()`, the vector:
1. Allocates a new heap block with double the previous capacity.
2. Moves existing elements into the new block.
3. Deallocates the old heap block.
4. Updates its internal pointer and capacity.

Because `Vec<T>` automatically dereferences to a slice `&[T]`, every slice function we wrote in Chapter 8 accepts vectors directly!

### 2. Hash Maps: `HashMap<K, V>`
`HashMap<K, V>` stores associations between keys of type `K` and values of type `V` using a hashing algorithm (SipHash by default).

```rust
use std::collections::HashMap;

let mut catalog: HashMap<String, Product> = HashMap::new();
catalog.insert(product.sku.clone(), product);

// Fast O(1) lookup:
if let Some(product) = catalog.get("TECH-KEY-001") {
    println!("Found: {}", product.name);
}
```

#### The Idiomatic Entry API
In Python or Java, grouping and counting items requires checking if a key exists, initializing it if absent, and then updating it:
```python
# Python: 2 lookups
if key not in counts:
    counts[key] = 0
counts[key] += 1
```

Rust solves this elegantly with the **Entry API**, which performs only **one hash lookup**:
```rust
// Exactly ONE lookup in the hash table!
*counts.entry(department_name).or_insert(0) += 1;
```
- `.entry(key)` inspects the slot.
- `.or_insert(default)` inserts the default value if vacant and returns a mutable reference `&mut V`.
- `*` dereferences the reference to mutate the value in place!

### 3. Hash Sets: `HashSet<T>`
A `HashSet<T>` is simply a `HashMap<T, ()>` that only tracks unique keys. It is used when you care about membership rather than associated values.

```rust
use std::collections::HashSet;

let mut tags = HashSet::new();
tags.insert(String::from("vip"));
tags.insert(String::from("vip")); // Duplicate: returns false, set remains len 1

assert!(tags.contains("vip"));
```

### 4. The Borrow Checker vs. Collection Mutation
What happens if you hold a reference to an element in a vector and attempt to push a new element?

```rust
let mut v = vec![1, 2, 3];
let first = &v[0]; // Immutable borrow of v's heap buffer

// v.push(4); // COMPILER ERROR: cannot borrow `v` as mutable while borrowed as immutable!
println!("First element: {first}");
```

**Why the compiler stops you**: If pushing `4` triggers a capacity doubling, the entire array is moved to a new memory address. If Rust permitted `v.push(4)`, `first` would point to deallocated heap memory—the classic C++ pointer invalidation bug. Rust eliminates this entire class of vulnerabilities at compile time.

---

## Coming From Other Languages

| Concept | C++ | Java | Python | Go | Rust |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Dynamic Array** | `std::vector<T>` | `ArrayList<T>` | `list` | `[]T` slice | `Vec<T>` |
| **Hash Map** | `std::unordered_map` | `HashMap<K, V>` | `dict` | `map[K]V` | `HashMap<K, V>` |
| **Hash Set** | `std::unordered_set` | `HashSet<T>` | `set` | Simulated via `map[T]struct{}` | `HashSet<T>` |
| **Reallocation Safety** | Dangling pointer risk upon resize. | Safe via GC (pointers remain valid). | Safe via GC. | Safe via GC. | **Compile-time error via Borrow Checker.** |
| **Iterator Mutation** | Undefined Behavior / Crash. | `ConcurrentModificationException` at runtime. | Runtime errors or skipped elements. | Unpredictable iteration behavior. | **Forbidden at compile time.** |

---

## Small Example
Here is a concise example showing `Vec`, `HashMap` with Entry API, and `HashSet`:

```rust
use std::collections::{HashMap, HashSet};

fn main() {
    // 1. Vec
    let mut scores = vec![100, 95];
    scores.push(88);
    println!("Scores count: {}", scores.len());

    // 2. HashMap & Entry API
    let mut word_counts = HashMap::new();
    let words = ["apple", "banana", "apple", "cherry"];
    for &word in &words {
        *word_counts.entry(word).or_insert(0) += 1;
    }
    println!("Apple count: {:?}", word_counts.get("apple")); // Some(2)

    // 3. HashSet
    let mut unique_words = HashSet::new();
    for &word in &words {
        unique_words.insert(word);
    }
    println!("Unique count: {}", unique_words.len()); // 3
}
```

---

## Apply To MiniStore
In MiniStore:
1. **Dynamic Shopping Cart**: We introduce `ShoppingCart`, backed by `Vec<CartItem>`. Callers can add items (which increments quantities if the item is already present) and remove items dynamically.
2. **Multi-Item Checkout Pipeline**: `PendingOrder` now owns `items: Vec<CartItem>`, allowing orders to contain multiple product lines.
3. **SKU Index for $O(1)$ Search**: `build_sku_index` maps product SKUs to products using `HashMap<String, Product>`.
4. **Department Inventory Analytics**: `count_products_by_department` groups products by their department prefix using the Entry API without duplicate hash lookups.
5. **Customer Membership Tags**: `Customer` stores unique tags (such as `"newsletter"`, `"early_adopter"`, and `"vip"`) inside a `HashSet<String>`.

---

## Code
Below is the complete, runnable code for `ministore/src/main.rs`:

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub sku: String,
    pub name: String,
    pub price_cents: u32,
    pub stock: u32,
}

impl Product {
    pub fn new(id: u64, sku: String, name: String, price_cents: u32, stock: u32) -> Self {
        Self {
            id,
            sku,
            name,
            price_cents,
            stock,
        }
    }

    /// Borrows `&self` immutably: checks stock without consuming or mutating the product.
    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    /// Borrows `&self` immutably: formats price for presentation.
    pub fn formatted_price(&self) -> String {
        format!("${:.2}", self.price_cents as f64 / 100.0)
    }

    /// Returns a string slice (`&str`) borrowing the department code from `self.sku`.
    /// Zero heap allocation: returns a 16-byte fat pointer (ptr + len) directly into `sku`.
    pub fn department_code(&self) -> &str {
        match self.sku.find('-') {
            Some(idx) => &self.sku[..idx],
            None => &self.sku[..],
        }
    }

    /// Returns a slice of the product name truncated to `max_bytes` without reallocating,
    /// ensuring the slice boundary respects UTF-8 character boundaries.
    pub fn truncated_name(&self, max_bytes: usize) -> &str {
        if max_bytes >= self.name.len() {
            &self.name[..]
        } else {
            let mut end = max_bytes;
            while end > 0 && !self.name.is_char_boundary(end) {
                end -= 1;
            }
            &self.name[..end]
        }
    }

    /// Checks if the product SKU begins with the requested prefix slice.
    pub fn matches_sku_prefix(&self, prefix: &str) -> bool {
        self.sku.starts_with(prefix)
    }

    /// Borrows `&mut self` mutably: updates inventory count in place.
    pub fn reduce_stock(&mut self, quantity: u32) -> Result<u32, &'static str> {
        if quantity > self.stock {
            Err("Insufficient stock available")
        } else {
            self.stock -= quantity;
            Ok(self.stock)
        }
    }

    /// Borrows `&mut self` mutably: adds inventory units in place.
    pub fn restock(&mut self, additional_units: u32) {
        self.stock += additional_units;
    }

    /// Borrows `&mut self` mutably: adjusts product price.
    pub fn update_price(&mut self, new_price_cents: u32) {
        self.price_cents = new_price_cents;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Customer {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub is_vip: bool,
    pub tags: HashSet<String>,
}

impl Customer {
    pub fn new(id: u64, name: String, email: String, is_vip: bool) -> Self {
        Self {
            id,
            name,
            email,
            is_vip,
            tags: HashSet::new(),
        }
    }

    /// Adds a tag to customer's unique tag set.
    pub fn add_tag(&mut self, tag: &str) -> bool {
        self.tags.insert(tag.to_string())
    }

    /// Checks if customer has a specific tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    /// Borrows `&self` immutably: reads customer data to display membership badge.
    pub fn display_badge(&self) -> String {
        if self.is_vip {
            format!("[VIP Member] {}", self.name)
        } else {
            format!("[Standard Member] {}", self.name)
        }
    }

    /// Borrows `&mut self` mutably: grants VIP membership status in place.
    pub fn upgrade_to_vip(&mut self) {
        self.is_vip = true;
        self.tags.insert(String::from("vip"));
    }

    /// Borrows `&mut self` mutably: updates email address in place.
    pub fn update_email(&mut self, new_email: String) {
        self.email = new_email;
    }
}

/// OrderId implements `Copy`: 8-byte scalar on the stack, never moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(pub u64);

/// Represents a single line item in a shopping cart.
#[derive(Debug, Clone, PartialEq)]
pub struct CartItem {
    pub product_id: u64,
    pub quantity: u32,
    pub unit_price_cents: u32,
}

impl CartItem {
    pub fn new(product_id: u64, quantity: u32, unit_price_cents: u32) -> Self {
        Self {
            product_id,
            quantity,
            unit_price_cents,
        }
    }

    pub fn line_total(&self) -> u32 {
        self.unit_price_cents * self.quantity
    }
}

/// A dynamic, multi-item shopping cart backed by a heap-allocated `Vec<CartItem>`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShoppingCart {
    pub items: Vec<CartItem>,
}

impl ShoppingCart {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Adds a product to the cart. If already present, increments quantity in place.
    pub fn add_item(&mut self, product_id: u64, quantity: u32, unit_price_cents: u32) {
        for item in &mut self.items {
            if item.product_id == product_id {
                item.quantity += quantity;
                return;
            }
        }
        self.items
            .push(CartItem::new(product_id, quantity, unit_price_cents));
    }

    /// Removes an item from the cart by product ID.
    pub fn remove_item(&mut self, product_id: u64) -> bool {
        if let Some(pos) = self
            .items
            .iter()
            .position(|item| item.product_id == product_id)
        {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn total_units(&self) -> u32 {
        self.items.iter().map(|item| item.quantity).sum()
    }

    pub fn subtotal_cents(&self) -> u32 {
        self.items.iter().map(|item| item.line_total()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// An unconfirmed order owning a dynamic list of items.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingOrder {
    pub order_id: OrderId,
    pub customer: Customer,
    pub items: Vec<CartItem>,
}

/// Finalized invoice/receipt produced when checkout consumes `PendingOrder`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmedReceipt {
    pub receipt_id: String,
    pub order_id: OrderId,
    pub customer_name: String,
    pub item_count: u32,
    pub total_cents: u32,
}

// ============================================================================
// Collection Utility Functions (Vec, HashMap, HashSet & Entry API)
// ============================================================================

/// Groups products by department and counts them using `HashMap` and the Entry API.
pub fn count_products_by_department(products: &[Product]) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for product in products {
        let dept = product.department_code().to_string();
        // The Entry API: zero duplicate lookups!
        *counts.entry(dept).or_insert(0) += 1;
    }
    counts
}

/// Indexes an array or slice of products into a `HashMap<String, Product>` by SKU for O(1) lookups.
pub fn build_sku_index(products: &[Product]) -> HashMap<String, Product> {
    let mut map = HashMap::new();
    for product in products {
        map.insert(product.sku.clone(), product.clone());
    }
    map
}

/// Calculates discount for a multi-item subtotal based on VIP status.
pub fn calculate_discount(customer: &Customer, subtotal_cents: u32) -> u32 {
    if customer.is_vip || customer.has_tag("vip") {
        (subtotal_cents * 10) / 100 // 10% discount for VIPs
    } else {
        0
    }
}

/// Deduplicates user IDs using a `HashSet`.
pub fn collect_unique_customers(orders: &[PendingOrder]) -> HashSet<u64> {
    let mut unique_ids = HashSet::new();
    for order in orders {
        unique_ids.insert(order.customer.id);
    }
    unique_ids
}

/// Consumes ownership of `PendingOrder` by value (Move Semantics) to finalize multi-item checkout.
pub fn finalize_order(order: PendingOrder) -> ConfirmedReceipt {
    let subtotal: u32 = order.items.iter().map(|item| item.line_total()).sum();
    let discount = calculate_discount(&order.customer, subtotal);
    let total_cents = subtotal - discount;
    let item_count = order.items.iter().map(|item| item.quantity).sum();

    ConfirmedReceipt {
        receipt_id: format!("REC-{}-{}", order.order_id.0, order.customer.id),
        order_id: order.order_id,
        customer_name: order.customer.name,
        item_count,
        total_cents,
    }
}

fn main() {
    println!("=== MiniStore: Collections (Vec, HashMap, HashSet & Entry API) ===\n");

    // 1. Dynamic Heap Array: Vec<T> for Products & Cart Items
    println!("1. Dynamic Vectors (Vec<T>):");
    let mut catalog_vec: Vec<Product> = vec![
        Product::new(
            101,
            String::from("TECH-KEY-001"),
            String::from("Mechanical Keyboard"),
            12000,
            10,
        ),
        Product::new(
            102,
            String::from("TECH-MOU-002"),
            String::from("Wireless Gaming Mouse"),
            4500,
            25,
        ),
        Product::new(
            103,
            String::from("OFFC-CHR-003"),
            String::from("Ergonomic Desk Chair"),
            35000,
            5,
        ),
    ];

    // Demonstrating dynamic growth via push:
    catalog_vec.push(Product::new(
        104,
        String::from("OFFC-DSK-004"),
        String::from("Standing Desk Frame"),
        45000,
        4,
    ));

    println!("   Catalog count: {} products", catalog_vec.len());
    println!(
        "   Vector capacity: {} (Allocated on heap)",
        catalog_vec.capacity()
    );

    // 2. HashMap<K, V> with O(1) Fast Lookups & The Entry API
    println!("\n2. Fast Key-Value Lookups with HashMap & Entry API:");
    let sku_map = build_sku_index(&catalog_vec);
    if let Some(mouse) = sku_map.get("TECH-MOU-002") {
        println!("   Found product by SKU 'TECH-MOU-002': {}", mouse.name);
    }

    let dept_counts = count_products_by_department(&catalog_vec);
    println!("   Products grouped by department (via Entry API):");
    for (dept, count) in &dept_counts {
        println!("     - {dept}: {count} item(s)");
    }

    // 3. HashSet<T> for Unique Tags & Deduplication
    println!("\n3. Unique Elements with HashSet<T>:");
    let mut customer = Customer::new(
        501,
        String::from("Grace Hopper"),
        String::from("grace@navy.mil"),
        false,
    );
    customer.add_tag("newsletter");
    customer.add_tag("early_adopter");
    customer.add_tag("newsletter"); // Duplicate insertion is ignored

    println!("   Customer: {}", customer.name);
    println!("   Tags set: {:?}", customer.tags);
    println!(
        "   Has 'newsletter' tag: {}",
        customer.has_tag("newsletter")
    );
    customer.upgrade_to_vip();
    println!("   After VIP upgrade: {:?}", customer.tags);

    // 4. Multi-Item ShoppingCart backed by Vec<CartItem>
    println!("\n4. Shopping Cart Operations (Dynamic Items List):");
    let mut cart = ShoppingCart::new();
    cart.add_item(101, 1, 12000); // 1x Keyboard @ $120.00
    cart.add_item(102, 2, 4500); // 2x Mouse @ $45.00
    cart.add_item(101, 1, 12000); // 1 more Keyboard (increments quantity to 2)

    println!("   Total line items in cart: {}", cart.items.len());
    println!("   Total physical units:     {}", cart.total_units());
    println!(
        "   Cart subtotal:            ${:.2}",
        cart.subtotal_cents() as f64 / 100.0
    );

    // 5. Finalizing Multi-Item Order (Ownership Consumption)
    println!("\n5. Multi-Item Order Finalization:");
    let pending_order = PendingOrder {
        order_id: OrderId(8801),
        customer,
        items: cart.items, // Ownership of Vec<CartItem> moves into pending_order
    };

    let receipt = finalize_order(pending_order);
    println!("   Receipt ID:   {}", receipt.receipt_id);
    println!("   Customer:     {}", receipt.customer_name);
    println!("   Total Units:  {}", receipt.item_count);
    println!(
        "   Total Paid:   ${:.2} (10% VIP discount applied)",
        receipt.total_cents as f64 / 100.0
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_shopping_cart_add_and_aggregate() {
        let mut cart = ShoppingCart::new();
        assert!(cart.is_empty());

        cart.add_item(1, 2, 1000); // 2 x $10.00 = $20.00
        cart.add_item(2, 1, 2500); // 1 x $25.00 = $25.00
        cart.add_item(1, 1, 1000); // Adds 1 more to product 1 -> 3 x $10.00 = $30.00

        assert_eq!(cart.items.len(), 2);
        assert_eq!(cart.total_units(), 4);
        assert_eq!(cart.subtotal_cents(), 5500); // $30.00 + $25.00 = $55.00
    }

    #[test]
    fn test_vec_shopping_cart_remove_item() {
        let mut cart = ShoppingCart::new();
        cart.add_item(10, 2, 500);
        cart.add_item(20, 1, 1500);

        assert_eq!(cart.items.len(), 2);
        let removed = cart.remove_item(10);
        assert!(removed);
        assert_eq!(cart.items.len(), 1);
        assert_eq!(cart.items[0].product_id, 20);

        let not_found = cart.remove_item(999);
        assert!(!not_found);
    }

    #[test]
    fn test_hashmap_entry_api_department_counts() {
        let products = [
            Product::new(1, String::from("TECH-1"), String::from("P1"), 100, 5),
            Product::new(2, String::from("TECH-2"), String::from("P2"), 200, 5),
            Product::new(3, String::from("HOME-1"), String::from("P3"), 300, 5),
        ];

        let counts = count_products_by_department(&products);
        assert_eq!(counts.get("TECH"), Some(&2));
        assert_eq!(counts.get("HOME"), Some(&1));
        assert_eq!(counts.get("FOOD"), None);
    }

    #[test]
    fn test_hashset_customer_tags() {
        let mut customer = Customer::new(1, String::from("Alice"), String::from("a@a.com"), false);
        assert!(!customer.has_tag("vip"));

        customer.add_tag("beta_tester");
        customer.add_tag("beta_tester"); // Duplicate
        assert_eq!(customer.tags.len(), 1);
        assert!(customer.has_tag("beta_tester"));

        customer.upgrade_to_vip();
        assert!(customer.is_vip);
        assert!(customer.has_tag("vip"));
    }

    #[test]
    fn test_multi_item_order_finalize() {
        let mut customer = Customer::new(1, String::from("Bob"), String::from("b@b.com"), false);
        customer.upgrade_to_vip();

        let items = vec![
            CartItem::new(1, 2, 2000), // 2 * $20 = $40.00 (4000 cents)
            CartItem::new(2, 1, 6000), // 1 * $60 = $60.00 (6000 cents)
        ]; // Subtotal: 10000 cents ($100.00)

        let order = PendingOrder {
            order_id: OrderId(42),
            customer,
            items,
        };

        // 10% VIP discount on 10000 = 1000 -> Total: 9000 cents ($90.00)
        let receipt = finalize_order(order);
        assert_eq!(receipt.order_id, OrderId(42));
        assert_eq!(receipt.customer_name, "Bob");
        assert_eq!(receipt.item_count, 3);
        assert_eq!(receipt.total_cents, 9000);
    }
}
```

---

## Understanding The Code

### 1. In-Place Quantity Merging in `ShoppingCart`
In `add_item`:
```rust
for item in &mut self.items {
    if item.product_id == product_id {
        item.quantity += quantity;
        return;
    }
}
self.items.push(CartItem::new(product_id, quantity, unit_price_cents));
```
We borrow each element mutably via `&mut self.items`. If the product is already in the cart, we increment `item.quantity` in place and return immediately. If not found after scanning, we push a new `CartItem` to the vector.

### 2. The Power of `entry().or_insert()`
In `count_products_by_department`:
```rust
*counts.entry(dept).or_insert(0) += 1;
```
If the department key exists in the map, `or_insert(0)` returns a mutable reference `&mut u32` to the existing count. If absent, it inserts `0` and returns a mutable reference to the new value. The leading `*` dereferences it so we can increment it in place with `+= 1`.

---

## Common Mistakes

### 1. Mutating a Vector While Iterating Over It
```rust
let mut v = vec![1, 2, 3];
for item in &v {
    if *item == 2 {
        v.push(10); // COMPILER ERROR: cannot borrow `v` as mutable because it is also borrowed as immutable
    }
}
```
In Java, this throws `ConcurrentModificationException` at runtime. In C++, it causes undefined behavior. In Rust, the compiler refuses to compile the bug.

### 2. Manual Lookup-Then-Insert in HashMaps
Avoid doing:
```rust
if !map.contains_key(&key) {
    map.insert(key.clone(), 0);
}
*map.get_mut(&key).unwrap() += 1; // Unnecessary multiple hash calculations!
```
Always use `map.entry(key).or_insert(...)` for optimal performance and safety.

---

## Compiler Errors

### Error E0502: Modifying a Collection While Holding a Reference
Consider:

```rust
let mut cart = vec![CartItem::new(1, 2, 1000)];
let first_item = &cart[0]; // Immutable borrow into vector's heap memory
cart.push(CartItem::new(2, 1, 2500)); // Attempt to mutate vector
println!("First item price: {}", first_item.unit_price_cents);
```

The Rust compiler rejects this immediately:

```text
error[E0502]: cannot borrow `cart` as mutable because it is also borrowed as immutable
  --> src/main.rs:190:5
   |
189|     let first_item = &cart[0];
   |                       ---- immutable borrow occurs here
190|     cart.push(CartItem::new(2, 1, 2500));
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
191|     println!("First item price: {}", first_item.unit_price_cents);
   |                                      ---------- immutable borrow later used here
```

**Why this protects you**: If pushing to `cart` causes `Vec` to reallocate its heap buffer, `first_item` would be left pointing to deallocated memory. Rust guarantees memory safety at compile time.

---

## Practice
1. **Cart Clear**: Add a method `pub fn clear(&mut self)` to `ShoppingCart` that empties all items.
2. **Category Filter**: Write a function `filter_products_by_dept(products: &[Product], dept: &str) -> Vec<Product>` that returns a vector of cloned products belonging to the specified department.
3. **Write Unit Tests**: Add unit tests in `src/main.rs` to verify that `clear` resets `total_units` to `0` and `filter_products_by_dept` returns the correct items.
4. Run `cargo test` to verify your solution.

---

## Checkpoint
- [x] Understood dynamic heap memory allocation in `Vec<T>`.
- [x] Mastered $O(1)$ key-value operations with `HashMap<K, V>`.
- [x] Used the `Entry` API (`.entry().or_insert()`) for zero-duplicate-lookup updates.
- [x] Utilized `HashSet<T>` for unique values and tag management.
- [x] Understood why mutating collections while holding references causes compile-time errors.
- [x] Transformed MiniStore into a multi-item e-commerce system.

---

## What We Learned
- `Vec<T>`, `HashMap<K, V>`, and `HashSet<T>` are the foundation for managing dynamic, real-world data in Rust.
- Rust collections manage their heap memory automatically through ownership and RAII—when a collection exits scope, all its heap memory and elements are dropped deterministically.
- MiniStore now supports dynamic multi-item shopping carts, fast SKU lookups, and customer interest tagging.

---

## What's Next
This concludes **Part I: Ownership**!
In **Part II: Modeling Business Logic**, starting with **Chapter 10: Enums and Pattern Matching**, we will explore how Rust models rich domain states, transitions, and business rules through algebraic data types.
