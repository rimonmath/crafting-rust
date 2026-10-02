# Chapter 6: Ownership

## What You'll Learn
- The core problem of computer memory management: Manual Allocation vs. Garbage Collection vs. **Rust's Ownership Model**.
- The **Three Golden Rules of Ownership** that the Rust compiler enforces at compile time.
- The physical reality of memory: **Stack vs. Heap**.
- **Move Semantics**: Why assigning or passing heap-allocated data invalidates the source variable.
- The difference between implicit stack copying (**`Copy` trait**) and explicit deep cloning (**`Clone` trait**).
- Transferring ownership across function boundaries: consuming parameters and returning values.
- Automatic resource cleanup without garbage collection via **Scope and RAII (`Drop`)**.
- Applying ownership transitions to MiniStore: consuming a `PendingOrder` to produce a `ConfirmedReceipt`, making duplicate checkouts impossible at the type level.

---

## Why Do We Need This?
Software programs must allocate and manage computer memory. Historically, programming languages fell into two opposing camps:

1. **Manual Memory Management (C, C++)**:
   The programmer manually allocates memory (`malloc`, `new`) and frees it (`free`, `delete`).
   - *Advantage*: Absolute performance, zero runtime overhead, predictable execution.
   - *Fatal Flaw*: Humans make mistakes. Forgetting to free memory creates **memory leaks**. Freeing memory twice causes **double-free vulnerabilities**. Accessing memory after it was freed causes **use-after-free exploits** and segmentation faults. Roughly 70% of all major security vulnerabilities in modern C/C++ codebases stem from memory safety bugs.

2. **Garbage Collection (Java, Go, C#, Python, JavaScript)**:
   A background runtime engine periodically pauses application threads to trace object graphs and deallocate unreachable memory.
   - *Advantage*: Prevents most use-after-free and double-free bugs.
   - *Fatal Flaw*: Unpredictable GC pause latencies ("stop-the-world"), significantly higher RAM consumption (often 2x–4x the actual payload), and runtime CPU overhead that renders them unsuitable for low-latency systems.

**Rust's Breakthrough**: Memory safety **without** a garbage collector. Memory is managed through a strict system of **Ownership** checked entirely at compile time. If your code compiles, it is mathematically guaranteed to be free of dangling pointers, double frees, and data races—with zero runtime overhead.

---

## The Problem
Consider a common workflow in an e-commerce platform like MiniStore:

```text
Customer Cart / Pending Order ──► Finalize Checkout ──► Confirmed Receipt
```

In traditional languages:
- If `pending_order` remains accessible after checkout, another thread or asynchronous task might submit it again, creating a **duplicate charge** or double inventory deduction.
- Defensive developers make defensive copies or add manual boolean flags like `is_processed = true`, which are easily bypassed or forgotten.

With Rust's ownership system, passing `pending_order` into `finalize_order(order: PendingOrder)` **moves** ownership. The original variable becomes completely invalid. The compiler forbids any subsequent use of the original order!

---

## Rust Concept

### 1. The Three Rules of Ownership
1. **Each value in Rust has an owner.**
2. **There can only be one owner at a time.**
3. **When the owner goes out of scope, the value is dropped.**

### 2. Stack vs. Heap
- **The Stack**:
  - Stores values whose size is fixed and known at compile time (e.g., `u32`, `i64`, `bool`, `[u32; 4]`, and tuple structs like `OrderId(u64)`).
  - Extremely fast: pushing and popping from the stack is a simple CPU pointer increment/decrement.
- **The Heap**:
  - Stores data whose size is dynamic or unknown at compile time (e.g., `String`, `Vec<T>`).
  - The operating system allocator finds an open memory segment, reserves it, and returns a pointer.

```text
Stack (Fixed 24-byte Descriptor)            Heap (Dynamically Allocated)
┌──────────────┬─────┬──────────┐            ┌───┬───┬───┬───┬───┬───┬───┐
│ Pointer (ptr)│ Len │ Capacity │ ─────────► │ H │ o │ p │ p │ e │ r │ ! │
└──────────────┴─────┴──────────┘            └───┴───┴───┴───┴───┴───┴───┘
```

A `String` variable on the stack consists of three `usize` words (24 bytes on 64-bit systems):
1. A pointer to the heap buffer holding the bytes.
2. The current length (`len`).
3. The allocated buffer capacity (`capacity`).

### 3. Move Semantics: Preventing Double-Free
What happens when you assign a heap-owning type to another variable?

```rust
let s1 = String::from("hello");
let s2 = s1; // OWNERSHIP MOVES to s2!
// println!("{s1}"); // COMPILER ERROR: borrow of moved value: `s1`
```

In Python or Java, `s2 = s1` copies the reference; both variables point to the same object in memory. If Rust did that, when both `s1` and `s2` exit their scope, both would try to free the exact same heap memory block—a critical security disaster known as a **double free**.

In C++, `std::string` makes a deep copy by default, which can silently destroy performance in loops.

**Rust's Solution**: Rust copies the 24-byte stack descriptor from `s1` to `s2`, and immediately **invalidates** `s1`. This operation is called a **Move**. Because `s1` is no longer valid, only `s2` will free the memory when it goes out of scope.

### 4. The `Copy` Trait vs. The `Clone` Trait
- **`Copy`**:
  Types that reside entirely on the stack (such as integers, floats, booleans, and structs composed exclusively of `Copy` types) implement the `Copy` trait. Assignment (`let b = a;`) makes a bitwise copy on the stack. The original `a` remains completely valid.
- **`Clone`**:
  Types that manage heap resources (`String`, `Product`) cannot implement `Copy`. If you genuinely want an independent deep copy of the heap allocation, you must explicitly call `.clone()`.

```rust
let id1 = OrderId(101);
let id2 = id1; // COPIED: both id1 and id2 are usable!

let p1 = Product::new(1, String::from("Keyboard"), 8999, 5);
// let p2 = p1;        // MOVED: p1 is invalidated
let p2 = p1.clone();   // CLONED: p1 and p2 both own independent heap buffers
```

### 5. Ownership and Functions
Passing a variable to a function follows the identical rules as assignment:
- Passing a `Copy` type copies the value.
- Passing a non-`Copy` type **moves** ownership into the function's parameter. The caller loses access!

```rust
fn print_customer(c: Customer) {
    println!("{}", c.name);
} // `c` is dropped here, its memory freed!

let cust = Customer::new(1, String::from("Ada"), String::from("a@b.com"), true);
print_customer(cust);
// println!("{}", cust.name); // ERROR: use of moved value `cust`
```

---

## Coming From Other Languages

| Concept | C / C++ | Java / Go / C# | Python / JS | Rust |
| :--- | :--- | :--- | :--- | :--- |
| **Memory Deallocation** | Manual (`free`, `delete`) | Background Garbage Collector | Reference counting & GC | **Deterministic compile-time RAII (`Drop`)** |
| **Assignment of Heap Types** | C++: Deep copy by default (or manual `std::move`) | Shared reference pointer | Shared reference pointer | **Move semantics (source invalidated)** |
| **Performance Overhead** | Zero overhead | Stop-the-world GC pauses & RAM bloat | Interpreter & GC overhead | **Zero runtime overhead** |
| **Double Free Safety** | Prone to crashes & exploits | Prevented by GC | Prevented by GC | **Prevented statically by compiler** |
| **Deep Copies** | Copy constructor | `clone()` / manual builder | `copy.deepcopy()` | **Explicit `.clone()` via `Clone` trait** |

---

## Small Example: Move vs. Clone

```rust
fn main() {
    let original = String::from("Rust Engine");

    // 1. Move
    let moved = original;
    // println!("{original}"); // ERROR: value used here after move

    // 2. Clone
    let cloned = moved.clone();
    println!("Both valid: moved = '{moved}', cloned = '{cloned}'");
}
```

---

## Apply To MiniStore
In MiniStore:
1. `OrderId` is a lightweight `Copy` tuple struct on the stack.
2. `Product` and `Customer` hold heap-allocated `String` fields.
3. `finalize_order(order: PendingOrder) -> ConfirmedReceipt` takes full ownership of `order` by value.
   - The pending order is consumed.
   - Its inner strings (`customer.name`, `product.name`) are moved directly into the new `ConfirmedReceipt` without redundant allocations.
   - It is impossible for callers to accidentally re-process or re-submit the same order.

---

## Code

### `src/main.rs`
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub name: String,
    pub price_cents: u32,
    pub stock: u32,
}

impl Product {
    pub fn new(id: u64, name: String, price_cents: u32, stock: u32) -> Self {
        Self {
            id,
            name,
            price_cents,
            stock,
        }
    }

    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    pub fn reduce_stock(&mut self, quantity: u32) -> Result<u32, &'static str> {
        if quantity > self.stock {
            Err("Insufficient stock available")
        } else {
            self.stock -= quantity;
            Ok(self.stock)
        }
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
}

/// OrderId implements `Copy`: simple stack value, never moved
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderId(pub u64);

/// An unconfirmed shopping order owning its line items
#[derive(Debug, Clone, PartialEq)]
pub struct PendingOrder {
    pub order_id: OrderId,
    pub customer: Customer,
    pub product: Product,
    pub quantity: u32,
}

/// Finalized invoice/receipt created when a PendingOrder is processed.
/// By taking ownership of `PendingOrder`, the order is consumed and cannot be re-processed!
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmedReceipt {
    pub receipt_id: String,
    pub order_id: OrderId,
    pub customer_name: String,
    pub product_name: String,
    pub quantity: u32,
    pub total_cents: u32,
}

/// Consumes ownership of a `PendingOrder` by value (Move Semantics).
/// The caller cannot reuse `order` after passing it here.
pub fn finalize_order(order: PendingOrder) -> ConfirmedReceipt {
    let subtotal = order.product.price_cents * order.quantity;
    let discount = if order.customer.is_vip {
        (subtotal * 10) / 100
    } else {
        0
    };
    let total_cents = subtotal - discount;

    ConfirmedReceipt {
        receipt_id: format!("REC-{}-{}", order.order_id.0, order.customer.id),
        order_id: order.order_id,
        customer_name: order.customer.name, // ownership of `String` moves into receipt
        product_name: order.product.name,   // ownership of `String` moves into receipt
        quantity: order.quantity,
        total_cents,
    }
}

fn main() {
    println!("=== MiniStore: Ownership, Move Semantics & Memory ===");

    // 1. Stack Allocation & Copy Trait: Primitive scalar types and Copy structs
    let order_id_1 = OrderId(9001);
    let order_id_2 = order_id_1; // Copied! Both remain fully valid on the stack.
    println!(
        "Copy Demonstration: id_1 = {:?}, id_2 = {:?}",
        order_id_1, order_id_2
    );

    // 2. Heap Allocation: String owns its character buffer on the heap
    let customer_name = String::from("Grace Hopper");
    let customer = Customer::new(
        201,
        customer_name, // Ownership of heap buffer MOVES into `customer`
        String::from("grace@example.com"),
        true,
    );
    // Note: `customer_name` is no longer valid here! Its ownership moved into `customer`.
    println!(
        "Customer Created: {} (VIP: {})",
        customer.name, customer.is_vip
    );

    // 3. Move Semantics vs Explicit Clone
    let product_original = Product::new(
        501,
        String::from("4K Ultra-Wide Monitor"),
        49999, // $499.99
        8,
    );

    // Explicit deep clone: allocates a separate String buffer on the heap
    let product_for_order = product_original.clone();
    println!(
        "Original Product retained: {} (Stock: {})",
        product_original.name, product_original.stock
    );
    println!(
        "Cloned Product for Order: {} (Stock: {})",
        product_for_order.name, product_for_order.stock
    );

    // 4. Moving ownership into an order pipeline
    let pending_order = PendingOrder {
        order_id: order_id_1,
        customer: customer.clone(),
        product: product_for_order,
        quantity: 1,
    };

    println!(
        "\nPending Order #{} created for {}",
        pending_order.order_id.0, pending_order.customer.name
    );

    // 5. Transferring ownership into `finalize_order` (Consuming the order)
    // `pending_order` is MOVED into `finalize_order`. It cannot be used again!
    let receipt = finalize_order(pending_order);

    println!("\n--- Order Confirmed (Ownership Consumed) ---");
    println!("Receipt ID: {}", receipt.receipt_id);
    println!("Customer:   {}", receipt.customer_name);
    println!(
        "Product:    {} x {}",
        receipt.product_name, receipt.quantity
    );
    println!("Total Paid: ${:.2}", receipt.total_cents as f64 / 100.0);

    // 6. Demonstrating Scope & RAII (Resource Acquisition Is Initialization)
    {
        println!("\n--- Entering Temporary Promotion Scope ---");
        let promo_code = String::from("SPRING_CLEANUP_2026");
        println!("Promo code active: {promo_code}");
        // When this block ends, `promo_code` goes out of scope and its heap buffer is automatically dropped!
    }
    println!(
        "Exited promotion scope: heap memory was automatically freed without garbage collection!"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_trait_preserves_original() {
        let id_a = OrderId(101);
        let id_b = id_a; // Copy
        assert_eq!(id_a, id_b);
        assert_eq!(id_a.0, 101);
        assert_eq!(id_b.0, 101);
    }

    #[test]
    fn test_clone_creates_independent_heap_allocation() {
        let original = Product::new(1, String::from("Item A"), 1000, 10);
        let mut cloned = original.clone();

        // Mutating cloned does not mutate original
        cloned.reduce_stock(5).unwrap();
        assert_eq!(cloned.stock, 5);
        assert_eq!(original.stock, 10);
        assert_eq!(original.name, cloned.name);
    }

    #[test]
    fn test_finalize_order_consumes_and_transforms() {
        let customer = Customer::new(1, String::from("Alice"), String::from("a@test.com"), true);
        let product = Product::new(10, String::from("Desk Pad"), 2000, 5); // $20.00
        let order = PendingOrder {
            order_id: OrderId(77),
            customer,
            product,
            quantity: 2, // 2 * $20.00 = $40.00 ($4000 cents)
        };

        // VIP discount: 10% off $4000 = $400 -> $3600 cents ($36.00)
        let receipt = finalize_order(order);
        assert_eq!(receipt.order_id, OrderId(77));
        assert_eq!(receipt.customer_name, "Alice");
        assert_eq!(receipt.product_name, "Desk Pad");
        assert_eq!(receipt.quantity, 2);
        assert_eq!(receipt.total_cents, 3600);
    }
}
```

---

## Understanding The Code

1. **Zero-Copy Field Reuse**:
   ```rust
   customer_name: order.customer.name,
   product_name: order.product.name,
   ```
   Because `finalize_order` owns `order`, it can destructure it and move the heap pointers of `name` directly into `ConfirmedReceipt`. No new heap allocations or string copies take place!

2. **Stack vs Heap in Structs**:
   - `OrderId`: Derives `Copy`, so it copies its 8-byte integer without moving.
   - `Product`: Contains `name: String`, so it cannot be `Copy`. Assigning it moves ownership unless `.clone()` is explicitly invoked.

---

## Common Mistakes

### 1. Reusing a Moved Variable
```rust
let name = String::from("Keyboard");
let p = Product::new(1, name, 5000, 2);
// println!("{name}"); // ERROR!
```
Once `name` was passed into `Product::new`, its ownership moved. `name` ceases to exist in the current scope.

### 2. Excessive Cloning
Developers coming from Java or Python often sprinkle `.clone()` everywhere to appease the compiler. Cloning duplicates heap memory buffers, which hurts performance.
> [!TIP]
> In **Chapter 7: Borrowing and References**, we will learn how to read and mutate data without moving or cloning via references (`&T` and `&mut T`).

---

## Compiler Errors
What happens when you try to access a variable after it has moved?

```rust
fn main() {
    let order = PendingOrder { ... };
    let receipt = finalize_order(order);
    println!("Processed order: {:?}", order); // ERROR!
}
```

Compiler output:
```text
error[E0382]: borrow of moved value: `order`
  --> src/main.rs:160:39
   |
158|     let receipt = finalize_order(order);
   |                                  ----- value moved here
159|     println!("Processed order: {:?}", order);
   |                                       ^^^^^ value borrowed here after move
   |
   = note: this error has occurred because `order` has type `PendingOrder`, which does not implement the `Copy` trait
```
The compiler prevents you from accidentally referencing stale or consumed state.

---

## Practice
1. Write a function `cancel_order(order: PendingOrder) -> String` that consumes the order and returns a cancellation confirmation message with the order ID.
2. In `tests`, write a test verifying that calling `cancel_order` returns `"Order #123 has been cancelled"`.
3. Try calling `finalize_order` on an order you already passed to `cancel_order`, observe the compiler error, and then delete the invalid line.
4. Run `cargo test` to verify your solution.

---

## Checkpoint
- [x] Understood the Three Golden Rules of Ownership.
- [x] Contrasted Stack memory with Heap allocations.
- [x] Mastered Move semantics and why Rust invalidates moved sources.
- [x] Distinguished `Copy` types (implicit stack duplicate) from `Clone` (explicit heap allocation).
- [x] Passed owned parameters into functions to enforce one-way state transitions.

---

## What We Learned
- Rust achieves memory safety without Garbage Collection pauses through deterministic compile-time ownership tracking.
- Move semantics eliminate double-free errors while enabling zero-cost transfers of heap resources.
- The type system can enforce business invariants (like non-repeatable checkout) simply through ownership rules.

---

## What's Next
Passing ownership to a function means you cannot use the value again. But what if a function just needs to inspect or temporarily modify data? In **Chapter 7: Borrowing and References**, we will master `&` and `&mut`, references, and the borrowing rules that prevent data races.
