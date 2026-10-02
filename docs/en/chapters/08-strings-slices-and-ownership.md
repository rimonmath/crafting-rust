# Chapter 8: Strings, Slices and Ownership in Practice

## What You'll Learn
- The difference between owned collections and **Slices**: zero-copy borrowing of contiguous memory.
- The physical memory representation of a slice: the **16-byte fat pointer** (`ptr` + `len`).
- **`String` vs. `&str`**: owned dynamic heap buffer vs. borrowed read-only string slice.
- **Deref Coercion**: why idiomatic Rust functions accept `&str` rather than `&String`.
- **Array Slices (`&[T]`) and Mutable Slices (`&mut [T]`)**: borrowing sub-ranges of arrays and vectors for batch operations.
- **UTF-8 Safety & Char Boundaries**: why Rust strings cannot be indexed by direct integer (`s[0]` is forbidden) and how to slice text safely.
- How the **Borrow Checker** prevents memory invalidation when slicing collections.
- Applying slices to MiniStore: parsing product SKU prefixes (`"TECH-KEY-001"` -> `"TECH"`), universal coupon code validation, batch price markdown calculations, and catalog inventory summarization without memory duplication.

---

## Why Do We Need This?
In [Chapter 7: Borrowing and References](/en/chapters/07-borrowing-and-references), we learned how to borrow an entire struct or variable using `&T` or `&mut T`. But real-world applications rarely deal only with whole objects.

Consider working with text and lists in MiniStore:
1. Every product SKU is formatted like `"TECH-KEY-001"`. We frequently need the department code (`"TECH"`). If extracting `"TECH"` requires creating a brand new heap `String`, our server will allocate and free tiny heap buffers millions of times a day.
2. When validating a coupon code entered by a user, should our function require a heap-allocated `String`, or should it accept string literals (`"SAVE10"`) too?
3. When running a seasonal sale on a subset of products in our inventory, should we clone the array just to compute discounts, or can we borrow a view of just that sub-range in place?

```text
Full Heap String (24 bytes on stack):
┌──────────────┬─────┬──────────┐
│ ptr          │ len │ capacity │ ──► ['T','E','C','H','-','K','E','Y','-','0','0','1'] (Heap)
└──────────────┴─────┴──────────┘      ▲
                                       │
String Slice &str (16 bytes on stack): │
┌──────────────┬─────┐                 │
│ ptr          │ len │ ────────────────┘ (len = 4 bytes: "TECH")
└──────────────┴─────┘
```

**Rust's Solution**: **Slices (`&str` and `&[T]`)**. A slice does not own data; it is a **fat pointer** that references an existing, contiguous sequence of elements in memory. It provides **zero-copy, zero-allocation views** with compile-time safety guarantees.

---

## The Problem
In languages without Rust's slice and ownership model:
- **C**: Strings are null-terminated character pointers (`char*`). To get a substring without modifying the original string in place with `\0`, you must call `malloc` and `strncpy`. Forgetting to allocate space for the null terminator causes buffer overflows and remote code execution vulnerabilities.
- **Java**: Strings are immutable objects on the heap. Prior to Java 7u6, `substring()` shared the underlying `char[]` buffer, which caused severe memory leaks (a tiny substring could keep a 50MB string in memory forever). Modern Java fixed this by making `substring()` copy the character array every time, trading memory leaks for GC allocation overhead.
- **Python / Go**: Python creates a new string object on every slice (`s[0:4]`). Go has string slices and slice headers (`ptr, len`), but Go strings are byte slices without compile-time guarantees against concurrent mutations or invalid UTF-8 sequences.

MiniStore needs:
- To inspect parts of strings and arrays with **zero runtime allocation**.
- To accept both literals (`"SAVE10"`) and user inputs (`String`) via a single clean function signature.
- Compile-time guarantees that the underlying buffer cannot be reallocated or deallocated while a slice is borrowing it.

---

## Rust Concept

### 1. What is a Slice?
A slice is a two-word object called a **fat pointer**:
1. A pointer (`ptr`) to the starting element in memory.
2. The number of elements (`len`) in the slice.

On a 64-bit architecture, a slice is always exactly **16 bytes** (8 bytes for pointer + 8 bytes for length), regardless of how much data it references!

### 2. `String` vs. `&str`
Rust distinguishes clearly between owned strings and borrowed string slices:

| Feature | `String` | `&str` (String Slice) |
| :--- | :--- | :--- |
| **Ownership** | Owns the buffer on the Heap | Borrows an existing UTF-8 buffer |
| **Size on Stack** | 24 bytes (`ptr`, `len`, `capacity`) | 16 bytes (`ptr`, `len`) |
| **Mutability** | Can grow, shrink, append | Immutable view into memory |
| **Allocation** | Dynamically allocated via OS allocator | Zero allocation; points to existing memory |
| **Where it points** | Always points to the Heap | Can point to Heap, Stack, or Binary static text (`&'static str`) |

```rust
// 1. String literal stored in the read-only segment of the compiled binary:
let greeting: &'static str = "Hello, MiniStore!";

// 2. Heap-allocated, owned String:
let mut dynamic_name = String::from("Wireless Mouse");

// 3. String slice borrowing from dynamic_name:
let word: &str = &dynamic_name[0..8]; // "Wireless"
```

### 3. Deref Coercion: The Power of `&str`
Why should you almost always use `&str` instead of `&String` in function parameters?

```rust
// AVOID: Forces caller to provide &String
fn validate_code_bad(code: &String) -> bool { ... }

// PREFER: Idiomatic Rust!
fn validate_code_good(code: &str) -> bool { ... }
```

Because `String` implements the `Deref` trait with target `str`, Rust automatically converts a reference to a `String` (`&String`) into a string slice (`&str`). This is called **Deref Coercion**.

By declaring `code: &str`:
- Callers with a `String` can pass `&my_string`.
- Callers with a string literal can pass `"SUMMER25"`.
- Callers with a subslice can pass `&my_string[0..4]`.
One signature cleanly supports all three without memory overhead!

### 4. UTF-8 Safety: Why `s[0]` is Forbidden
In Python or Java, you can write `s[0]` to get the first character. In Rust:

```rust
let s = String::from("hello");
// let c = s[0]; // COMPILER ERROR: `String` cannot be indexed by `{integer}`
```

**Why?** Rust strings are encoded strictly as **UTF-8**. In UTF-8:
- ASCII characters (A-Z, 0-9) take 1 byte.
- Latin, Greek, and Cyrillic characters take 2 bytes.
- Bengali, Hindi, Arabic, and CJK characters take 3 bytes.
- Emojis take 4 bytes.

If Rust allowed `s[0]`, indexing the Bengali word `"বই"` (where `'ব'` takes 3 bytes: `0xE0, 0xA6, 0xAC`) would return the raw invalid byte `0xE0`, which is not a valid character!

When you slice a string:
```rust
let s = "Hello";
let slice = &s[0..2]; // "He"
```
The indices `0..2` are **byte offsets**, not character offsets. If you slice in the middle of a multi-byte UTF-8 character, Rust will panic at runtime to protect string validity. Use `.is_char_boundary(idx)` or `.chars()` for safe boundary checking.

### 5. Array Slices (`&[T]` and `&mut [T]`)
Just as `&str` is a slice of a string, `&[T]` is a slice of an array or vector:

```rust
let numbers: [u32; 5] = [10, 20, 30, 40, 50]; // 20 bytes on stack

// Immutable slice:
let first_three: &[u32] = &numbers[0..3]; // [10, 20, 30]

// Mutable slice (in-place modification without moving):
let mut prices = [1000, 2000, 3000];
let first_two: &mut [u32] = &mut prices[0..2];
first_two[0] = 800; // prices is now [800, 2000, 3000]
```

### 6. Slices as Borrows: Preventing Invalidation
A slice is a borrow (`&` or `&mut`). This means the **Aliasing XOR Mutability** rule applies:

```rust
let mut s = String::from("hello world");
let word = &s[0..5]; // Immutable borrow of `s`

// s.clear(); // COMPILER ERROR! Cannot borrow `s` as mutable while borrowed as immutable
println!("The word is: {word}");
```

If Rust allowed `s.clear()` while `word` existed, `s` would deallocate its heap buffer, leaving `word` pointing to freed memory. The Borrow Checker prevents this at compile time.

---

## Coming From Other Languages

| Concept | C / C++ | Java | Python | Go | Rust |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **String Slice** | `std::string_view` (C++17). Dangling views easily cause UB. | None. `substring()` copies the underlying buffer. | New string copy allocated for every slice. | Slice header `(ptr, len)`. No compiler protection against concurrent mutation. | `&str`: 16-byte fat pointer verified by the borrow checker. |
| **Memory Cost** | Zero (pointer + len) but unsafe. | New object allocation + array copy. | New object allocation + memory copy. | Zero allocation. | **Zero allocation + 100% memory safety**. |
| **Array Sub-range** | Pointer arithmetic (`ptr + offset`). No bounds checks. | `Arrays.copyOfRange` (allocates fresh array). | New list copy (`list[1:3]`). | Slice `slice[1:3]`. | `&[T]` / `&mut [T]` with compile-time borrow tracking. |
| **Encoding** | Arbitrary bytes or `wchar_t`. | UTF-16 (`char` is 2 bytes). | Unicode code points. | UTF-8 bytes (no validity enforcement). | Guaranteed valid UTF-8. Non-boundary slice panics. |

---

## Small Example
Here is a concise demonstration of string slicing and array slicing:

```rust
fn print_prefix(text: &str, byte_count: usize) {
    if text.is_char_boundary(byte_count) {
        println!("Prefix: {}", &text[..byte_count]);
    } else {
        println!("Invalid UTF-8 character boundary!");
    }
}

fn sum_slice(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for &n in numbers {
        sum += n;
    }
    sum
}

fn main() {
    let message = String::from("MiniStore Inventory");
    print_prefix(&message, 9); // "MiniStore"

    let data = [10, 20, 30, 40, 50];
    let total = sum_slice(&data[1..4]); // Sums [20, 30, 40] = 90
    println!("Total: {total}");
}
```

---

## Apply To MiniStore
In MiniStore:
1. **SKU Parsing**: Product SKUs are structured as `DEPT-ITEM-ID` (e.g. `"TECH-KEY-001"`). Using `department_code(&self) -> &str`, we extract `"TECH"` with zero heap allocation.
2. **Safe Title Truncation**: Displaying product listings on mobile screens requires shortening product descriptions without splitting multi-byte UTF-8 glyphs. `truncated_name(&self, max_bytes: usize) -> &str` verifies UTF-8 boundaries before slicing.
3. **Universal Coupon Validation**: `validate_coupon(code: &str)` accepts string literals (`"SAVE10"`), heap strings (`&user_input`), and subslices alike.
4. **Batch Markdown on Array Slices**: `apply_batch_markdown(prices: &mut [u32], discount_cents: u32)` discounts inventory sub-ranges in place.
5. **Inventory Valuation**: `summarize_inventory(catalog: &[Product])` calculates store-wide stock counts and dollar valuations over any slice of products.

---

## Code
Below is the complete, runnable code for `ministore/src/main.rs`:

```rust
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
            // Find the nearest valid UTF-8 character boundary at or below max_bytes
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
}

impl Customer {
    pub fn new(id: u64, name: String, email: String, is_vip: bool) -> Self {
        Self {
            id,
            name,
            email,
            is_vip,
        }
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
    }

    /// Borrows `&mut self` mutably: updates email address in place.
    pub fn update_email(&mut self, new_email: String) {
        self.email = new_email;
    }
}

/// OrderId implements `Copy`: 8-byte scalar on the stack, never moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderId(pub u64);

/// An unconfirmed order owning its items.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingOrder {
    pub order_id: OrderId,
    pub customer: Customer,
    pub product: Product,
    pub quantity: u32,
}

/// Finalized invoice/receipt produced when checkout consumes `PendingOrder`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmedReceipt {
    pub receipt_id: String,
    pub order_id: OrderId,
    pub customer_name: String,
    pub product_name: String,
    pub quantity: u32,
    pub total_cents: u32,
}

// ============================================================================
// String & Array Slice Functions: Zero-Copy Views Into Memory
// ============================================================================

/// Borrows `&Product` immutably: calculates subtotal without taking ownership.
pub fn calculate_line_total(product: &Product, quantity: u32) -> u32 {
    product.price_cents * quantity
}

/// Borrows `&Customer` immutably: calculates discount based on VIP status.
pub fn calculate_discount(customer: &Customer, subtotal_cents: u32) -> u32 {
    if customer.is_vip {
        (subtotal_cents * 10) / 100 // 10% discount for VIPs
    } else {
        0
    }
}

/// Borrows an array slice of prices `&[u32]`.
/// Works seamlessly with fixed stack arrays `[u32; N]` or slices without copying data.
pub fn calculate_batch_total(prices: &[u32]) -> u32 {
    let mut total = 0;
    for &price in prices {
        total += price;
    }
    total
}

/// Borrows a mutable slice `&mut [u32]` to apply markdown discounts in-place.
pub fn apply_batch_markdown(prices: &mut [u32], markdown_cents: u32) {
    for price in prices.iter_mut() {
        if *price > markdown_cents {
            *price -= markdown_cents;
        } else {
            *price = 0;
        }
    }
}

/// Calculates inventory totals using a slice of Products `&[Product]`.
/// Returns `(total_units, total_valuation_cents)`.
pub fn summarize_inventory(catalog: &[Product]) -> (u32, u32) {
    let mut total_units = 0;
    let mut total_cents = 0;
    for product in catalog {
        total_units += product.stock;
        total_cents += product.price_cents * product.stock;
    }
    (total_units, total_cents)
}

/// Accepts any string slice `&str` (literal, heap slice, or &String via Deref coercion).
/// Validates promo codes and returns discount percentage.
pub fn validate_coupon(code: &str) -> Result<u32, &'static str> {
    let trimmed = code.trim();
    if trimmed.eq_ignore_ascii_case("SAVE10") {
        Ok(10)
    } else if trimmed.eq_ignore_ascii_case("SUMMER25") {
        Ok(25)
    } else if trimmed.is_empty() {
        Err("Coupon code cannot be empty")
    } else {
        Err("Invalid coupon code")
    }
}

/// Formats a single receipt line item using string slices `&str` to avoid unnecessary String copies.
pub fn format_line_summary(sku: &str, name: &str, price_cents: u32) -> String {
    format!("[{sku}] {name} - ${:.2}", price_cents as f64 / 100.0)
}

/// Borrows both `&Customer` and `&Product` immutably to render a preview.
pub fn print_order_preview(customer: &Customer, product: &Product, quantity: u32) {
    let subtotal = calculate_line_total(product, quantity);
    let discount = calculate_discount(customer, subtotal);
    let final_total = subtotal - discount;

    println!("--- Order Preview (Borrowed Read-Only) ---");
    println!("Customer: {}", customer.display_badge());
    println!(
        "Item:     {}",
        format_line_summary(&product.sku, &product.name, product.price_cents)
    );
    println!("Dept:     {}", product.department_code());
    println!("Quantity: {}", quantity);
    println!("Subtotal: ${:.2}", subtotal as f64 / 100.0);
    println!("Discount: ${:.2}", discount as f64 / 100.0);
    println!("Estimate: ${:.2}", final_total as f64 / 100.0);
}

/// Consumes ownership of `PendingOrder` by value (Move Semantics).
pub fn finalize_order(order: PendingOrder) -> ConfirmedReceipt {
    let subtotal = calculate_line_total(&order.product, order.quantity);
    let discount = calculate_discount(&order.customer, subtotal);
    let total_cents = subtotal - discount;

    ConfirmedReceipt {
        receipt_id: format!("REC-{}-{}", order.order_id.0, order.customer.id),
        order_id: order.order_id,
        customer_name: order.customer.name,
        product_name: order.product.name,
        quantity: order.quantity,
        total_cents,
    }
}

fn main() {
    println!("=== MiniStore: Strings, Slices & Zero-Copy Views ===\n");

    // 1. String Slices (&str) vs Owned Strings (String)
    let sample_sku = String::from("TECH-KEY-001");
    // Slicing without allocating: 16-byte fat pointer (ptr + len)
    let dept_slice: &str = &sample_sku[0..4];
    let id_slice: &str = &sample_sku[5..];
    println!("1. String Slice Decomposition (Zero Allocations):");
    println!("   Full SKU:    {sample_sku} (Heap String: 24 bytes on stack)");
    println!("   Dept Slice:  {dept_slice} (Slice view: 16 bytes on stack)");
    println!("   ID Slice:    {id_slice}\n");

    // 2. Deref Coercion: &String automatically coerces to &str
    println!("2. Universal &str Function Parameters & Deref Coercion:");
    let coupon_literal = "  SAVE10  "; // &'static str
    let coupon_heap = String::from("SUMMER25"); // String

    println!(
        "   Validating literal: {:?}",
        validate_coupon(coupon_literal)
    );
    println!(
        "   Validating heap string (&String coerces to &str): {:?}",
        validate_coupon(&coupon_heap)
    );

    // 3. Products with SKU Slices and Safe Truncation
    let keyboard = Product::new(
        501,
        String::from("TECH-KEY-001"),
        String::from("RGB Mechanical Gaming Keyboard with Hot-Swap Switches"),
        12000,
        15,
    );

    println!("\n3. Product Slice Methods:");
    println!("   Product SKU:       {}", keyboard.sku);
    println!("   Department:        {}", keyboard.department_code());
    println!(
        "   Short Title (25B): {}...",
        keyboard.truncated_name(25)
    );
    println!(
        "   Matches 'TECH':    {}",
        keyboard.matches_sku_prefix("TECH")
    );

    // 4. Array Slices (&[T]) & Mutable Slices (&mut [T])
    println!("\n4. Array Slices (&[T]) for Batch Pricing:");
    let mut clearance_prices = [2500, 4500, 12000, 8000]; // Stack array [u32; 4]
    println!(
        "   Initial prices:    {:?} (Total = ${:.2})",
        clearance_prices,
        calculate_batch_total(&clearance_prices) as f64 / 100.0
    );

    // Pass a subslice `&mut clearance_prices[0..2]` to apply markdown only to first two items
    apply_batch_markdown(&mut clearance_prices[0..2], 500);
    println!(
        "   After $5 markdown on first 2 items: {:?}",
        clearance_prices
    );

    // 5. Catalog Slices (&[Product])
    let catalog = [
        keyboard.clone(),
        Product::new(
            502,
            String::from("OFFC-CHAIR-002"),
            String::from("Ergonomic Mesh Chair"),
            35000,
            8,
        ),
        Product::new(
            503,
            String::from("TECH-MOU-003"),
            String::from("Wireless Laser Mouse"),
            4500,
            25,
        ),
    ];

    let (total_units, valuation) = summarize_inventory(&catalog);
    println!("\n5. Catalog Summary (Calculated via &[Product] slice):");
    println!("   Total Units:     {total_units}");
    println!("   Total Valuation: ${:.2}", valuation as f64 / 100.0);

    // 6. Preview and Checkout
    let customer = Customer::new(
        101,
        String::from("Ada Lovelace"),
        String::from("ada@analytical.org"),
        true,
    );

    println!("\n6. Order Preview & Checkout:");
    print_order_preview(&customer, &keyboard, 2);

    let pending_order = PendingOrder {
        order_id: OrderId(9001),
        customer,
        product: keyboard,
        quantity: 2,
    };

    let receipt = finalize_order(pending_order);
    println!("\n--- Checkout Confirmed ---");
    println!("Receipt: {}", receipt.receipt_id);
    println!("Item:    {} x {}", receipt.product_name, receipt.quantity);
    println!("Paid:    ${:.2}", receipt.total_cents as f64 / 100.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_slice_department_code() {
        let p = Product::new(
            1,
            String::from("ELEC-MON-001"),
            String::from("4K Monitor"),
            40000,
            5,
        );
        assert_eq!(p.department_code(), "ELEC");
        assert!(p.matches_sku_prefix("ELEC"));
        assert!(!p.matches_sku_prefix("FURN"));
    }

    #[test]
    fn test_truncated_name_ascii_and_boundaries() {
        let p = Product::new(
            2,
            String::from("BOOK-RUST-01"),
            String::from("Rust Programming in Depth"),
            5000,
            20,
        );
        assert_eq!(p.truncated_name(4), "Rust");
        assert_eq!(p.truncated_name(100), "Rust Programming in Depth");

        // UTF-8 multi-byte test (Bengali character: 'ব' is 3 bytes: 0xE0, 0xA6, 0xAC)
        let p_utf8 = Product::new(
            3,
            String::from("BOOK-BN-02"),
            String::from("বই"),
            1000,
            10,
        );
        // Slicing at 2 bytes would slice inside 'ব'; truncated_name snaps safely to 0
        assert_eq!(p_utf8.truncated_name(2), "");
        assert_eq!(p_utf8.truncated_name(3), "ব");
    }

    #[test]
    fn test_validate_coupon_deref_coercion() {
        // String literal (&'static str)
        assert_eq!(validate_coupon("SAVE10"), Ok(10));
        assert_eq!(validate_coupon("  save10  "), Ok(10));

        // Owned String coerced via &coupon
        let dynamic_code = String::from("SUMMER25");
        assert_eq!(validate_coupon(&dynamic_code), Ok(25));

        assert!(validate_coupon("UNKNOWN").is_err());
        assert!(validate_coupon("   ").is_err());
    }

    #[test]
    fn test_array_slices_and_mutable_markdown() {
        let mut prices = [1000, 2500, 5000];

        // Slice calculation:
        assert_eq!(calculate_batch_total(&prices), 8500);
        assert_eq!(calculate_batch_total(&prices[1..3]), 7500);

        // Mutable slice markdown on first two elements only:
        apply_batch_markdown(&mut prices[0..2], 500);
        assert_eq!(prices, [500, 2000, 5000]);
    }

    #[test]
    fn test_catalog_inventory_summary() {
        let items = [
            Product::new(1, String::from("A-1"), String::from("Item 1"), 100, 10),
            Product::new(2, String::from("A-2"), String::from("Item 2"), 200, 5),
        ];

        let (units, valuation) = summarize_inventory(&items);
        assert_eq!(units, 15);
        assert_eq!(valuation, (100 * 10) + (200 * 5)); // 1000 + 1000 = 2000
    }
}
```

---

## Understanding The Code

### 1. Zero-Allocation Substring via Range Slicing
In `department_code`:
```rust
match self.sku.find('-') {
    Some(idx) => &self.sku[..idx],
    None => &self.sku[..],
}
```
`&self.sku[..idx]` computes a 16-byte slice starting at `self.sku`'s heap buffer pointer with length `idx`. No string allocation happens. The department code is returned as a borrow tied to the lifetime of `&self`.

### 2. Slicing Sub-ranges of Arrays with `&mut [T]`
In `apply_batch_markdown(&mut clearance_prices[0..2], 500)`:
We passed `&mut [u32]` referencing only indices 0 and 1. The remaining elements at indices 2 and 3 are untouched. Slices give precise, in-place access to bounded sub-ranges.

---

## Common Mistakes

### 1. Slicing Inside a Multi-Byte UTF-8 Character
```rust
let text = "বাংলা"; // Each character takes 3 bytes
let bad_slice = &text[0..2]; // RUNTIME PANIC: byte index 2 is not a char boundary; it is inside 'ব' (bytes 0..3)
```
Always use `.is_char_boundary(idx)` or iterate through `.chars()` when slicing non-ASCII strings.

### 2. Writing `fn foo(s: &String)` Instead of `fn foo(s: &str)`
Accepting `&String` forces the caller to allocate an owned `String` even if they only have a static string literal `"hello"`. Always prefer `&str` for function parameters.

---

## Compiler Errors

### Error E0502: Mutating a String While Holding a Slice View
Consider this code:

```rust
let mut sku = String::from("TECH-KEY-001");
let dept = &sku[0..4]; // Immutable borrow of `sku` via slice
sku.push_str("-DISCOUNT"); // Attempt to mutate `sku`
println!("Dept: {dept}"); // Slice used here
```

The Rust compiler rejects this:

```text
error[E0502]: cannot borrow `sku` as mutable because it is also borrowed as immutable
  --> src/main.rs:188:5
   |
187|     let dept = &sku[0..4];
   |                 --- immutable borrow occurs here
188|     sku.push_str("-DISCOUNT");
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
189|     println!("Dept: {dept}");
   |                     ------ immutable borrow later used here
```

**Why this protects you**: If `push_str` caused the heap buffer to exceed its capacity, the operating system would reallocate the buffer at a different address. If Rust allowed this, `dept` would point to deallocated garbage memory. The Borrow Checker prevents this memory corruption at compile time.

---

## Practice
1. **Extract Item ID**: Add a method `pub fn item_id(&self) -> &str` to `Product` that returns the slice following the first hyphen in `sku` (e.g. `"KEY-001"` from `"TECH-KEY-001"`).
2. **Find High-Value Items**: Write a function `filter_high_value(prices: &[u32], threshold_cents: u32) -> usize` that returns the count of items in the price slice exceeding the threshold.
3. **Verify with Tests**: Add unit tests for `item_id` and `filter_high_value` in `tests`.
4. Run `cargo test` to verify your solution.

---

## Checkpoint
- [x] Understood the difference between owned `String` and borrowed `&str`.
- [x] Understood the memory anatomy of a slice as a 16-byte fat pointer (`ptr` + `len`).
- [x] Mastered Deref coercion and why function parameters should take `&str`.
- [x] Sliced arrays (`&[T]`) and applied in-place mutations on mutable slices (`&mut [T]`).
- [x] Understood UTF-8 byte indexing rules and character boundaries.
- [x] Witnessed how the borrow checker prevents slice invalidation upon reallocation.

---

## What We Learned
- Slices provide zero-copy, zero-allocation views into contiguous memory segments.
- `&str` and `&[T]` decouple functions from concrete container types, making code reusable for static data, heap data, and sub-ranges.
- Rust guarantees that data referenced by a slice cannot be modified or reallocated while the slice is in scope.

---

## What's Next
Now that we understand ownership, borrowing, references, and slices, how do we store and manipulate dynamic groups of items? In **Chapter 9: Collections**, we will explore Rust's standard heap collections: `Vec<T>`, `HashMap<K, V>`, and `HashSet<T>`.
