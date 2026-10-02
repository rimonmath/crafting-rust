# Chapter 7: Borrowing and References

## What You'll Learn
- Why moving ownership for every operation is inefficient and unergonomic.
- The concept of **Borrowing** and how it separates access from ownership.
- **Shared References (`&T`)**: Read-only access allowing multiple simultaneous readers.
- **Mutable References (`&mut T`)**: Exclusive access allowing in-place data mutation.
- The Golden Rule of Rust Memory Safety: **Aliasing XOR Mutability** (Readers OR one Writer, never both).
- How the compiler guarantees **No Dangling References** without a Garbage Collector or runtime overhead.
- How **Non-Lexical Lifetimes (NLL)** determine when a borrow ends.
- Method receivers compared: `self` (consume) vs. `&self` (inspect) vs. `&mut self` (modify).
- Applying borrowing to MiniStore: order previews, in-place inventory restocking, and customer profile updates without unnecessary cloning or moving.

---

## Why Do We Need This?
In [Chapter 6: Ownership](/en/chapters/06-ownership), we discovered Rust's primary mechanism for memory safety: **Move Semantics**. When a variable is passed into a function by value, ownership transfers to that function, and the caller can no longer use it:

```rust
fn print_product(product: Product) {
    println!("Product: {}", product.name);
} // `product` is dropped from heap memory here!

let keyboard = Product::new(1, String::from("Keychron K2"), 8500, 10);
print_product(keyboard);
// println!("{}", keyboard.name); // ERROR: borrow of moved value: `keyboard`
```

If we want to keep using `keyboard` after printing it, how would we solve this with ownership alone?

1. **Defensive Cloning (`.clone()`)**:
   ```rust
   print_product(keyboard.clone()); // Allocates fresh heap memory just to read text!
   ```
   *Flaw*: Allocating heap buffers solely to inspect data destroys performance and wastes RAM.

2. **Returning Ownership in Tuples**:
   ```rust
   fn print_product(product: Product) -> Product {
       println!("Product: {}", product.name);
       product
   }
   let keyboard = print_product(keyboard); // Threading ownership back and forth
   ```
   *Flaw*: Convoluted, verbose, and completely unworkable for real-world applications where data is read by multiple subsystems.

**Rust's Solution**: **Borrowing and References**. Instead of transferring ownership of the value, you can create a *reference* (`&`) to lend temporary access to the value. When the function finishes, the borrow ends, and the original owner retains full control of the resource.

---

## The Problem
Consider an e-commerce catalog and cart in MiniStore:

```text
Product in Store Catalog ──► Show on Storefront (Read)
                         ──► Calculate Cart Total (Read)
                         ──► Restock Inventory (Write in place)
                         ──► Confirm Checkout Order (Consume)
```

In languages without Rust's borrowing rules:
- **C/C++**: Pointers allow in-place mutation and sharing, but there is no compile-time enforcement against **dangling pointers** (reading memory after it was freed) or **data races** (one thread reading while another writes).
- **Java/Python/Go**: Every object variable is a shared pointer. Any method can secretly mutate state behind your back, leading to subtle race conditions, **iterator invalidation**, and uncontrollable side effects.

MiniStore needs:
1. To display order previews and calculate discounts without destroying or cloning product and customer records.
2. To modify product stock and update customer profiles in-place safely.
3. A guarantee that nobody can mutate inventory while someone else is actively calculating a checkout receipt from it.

---

## Rust Concept

### 1. References and Borrowing
A reference is like a pointer: it contains the memory address of the target value. Unlike raw pointers in C/C++, Rust references are **guaranteed to always point to valid data of a specific type**. Creating a reference is called **borrowing**.

```text
Variable `product` (Owner on Stack):
┌──────────────┬──────────────────┬─────────────┬───────┐
│ id: 501      │ name: (ptr,len)  │ price: 8500 │ stock │
└──────────────┴─────────┬────────┴─────────────┴───────┘
                         │
Ref `&product`           ▼
┌────────────────────────┐
│ Pointer to `product`   │
└────────────────────────┘
```

When a reference goes out of scope, the data it points to is **not dropped**, because the reference does not own the data.

### 2. Shared References (`&T`)
A shared reference (`&T`) grants **read-only** access. You can create as many simultaneous shared references as you want.

```rust
let p = Product::new(1, String::from("Mouse"), 2500, 10);
let r1 = &p;
let r2 = &p;
let r3 = &p;
println!("Name: {}, Price: {}", r1.name, r2.price_cents); // Perfectly safe!
```

Because all holders are readers, none can mutate the data, guaranteeing that the data will not change unexpectedly beneath any reader.

### 3. Mutable References (`&mut T`)
A mutable reference (`&mut T`) grants **exclusive read-write** access. Through a mutable reference, you can modify the underlying value without transferring ownership.

To create a mutable reference:
1. The underlying variable must be declared with `mut`.
2. The reference must be created with `&mut`.

```rust
let mut p = Product::new(1, String::from("Mouse"), 2500, 10);
let p_mut = &mut p;
p_mut.stock += 5; // Mutates `p` directly through the reference
```

### 4. The Golden Rule: Aliasing XOR Mutability
This is the single most important rule in Rust:

> **At any given time, you can have EITHER:**
> - **Any number of immutable references (`&T`), OR**
> - **Exactly one mutable reference (`&mut T`).**
> 
> **Never both at the same time.**

```text
   ┌────────────────────────────────────────────────────────┐
   │             Rust's Borrow Checker Matrix              │
   ├──────────────────────────┬─────────────────────────────┤
   │ Multiple Shared (&T)?    │ ALLOWED (Many readers)      │
   │ One Mutable (&mut T)?    │ ALLOWED (One writer)        │
   │ Shared (&T) + Mut (&mut)?│ FORBIDDEN (Compiler Error!) │
   │ Multiple Mut (&mut T)?   │ FORBIDDEN (Compiler Error!) │
   └──────────────────────────┴─────────────────────────────┘
```

Why does Rust forbid having a mutable reference alongside shared references?
If code could mutate a value while other parts of the program are reading it, the readers might observe corrupted state, invalid pointers, or data races! By enforcing **Aliasing XOR Mutability** at compile time, Rust completely prevents data races and pointer invalidation.

### 5. Non-Lexical Lifetimes (NLL)
In early versions of Rust, a reference's borrow lasted until the closing curly brace `}` of its enclosing block. Since Rust 2018, the compiler uses **Non-Lexical Lifetimes (NLL)**:
A borrow ends at the **last point where the reference is actually used**, not at the end of the scope block.

```rust
let mut product = Product::new(1, String::from("Desk"), 15000, 2);

let r1 = &product;
println!("Reading product: {}", r1.name); // `r1` is used here for the last time.
// -> The immutable borrow ends right HERE!

let r2 = &mut product; // LEGAL! `r1` is no longer active.
r2.stock += 1;
```

---

## Coming From Other Languages

| Concept | C / C++ | Go / Java / Python / C# | Rust |
| :--- | :--- | :--- | :--- |
| **Passing to function** | Value or pointer (`*p`) or ref (`&r`). No compiler tracking. | References by default. Everything is aliased and mutable. | By Value (`T`), Shared Ref (`&T`), or Mutable Ref (`&mut T`). |
| **Dangling references** | Common bug (`return &local_var;`). Leads to crashes/exploits. | Impossible (GC keeps objects alive on heap forever). | Impossible (Compiler proves lifetime validity at compile time). |
| **Data Races** | Runtime disaster (UB, undefined behavior). Requires manual locks. | Common runtime bugs. Race detector needed. | Prevented at compile time by the Borrow Checker. |
| **Aliasing & Mutation** | Allowed everywhere; causes iterator invalidation. | Allowed everywhere; causes `ConcurrentModificationException`. | Forbidden at compile time by **Aliasing XOR Mutability**. |
| **Memory Cleanup** | Manual (`delete`, `free`). | Background Garbage Collector pause. | Deterministic RAII when owner exits scope. |

---

## Small Example
Here is a minimal demonstration comparing `&T`, `&mut T`, and the borrow checker:

```rust
fn main() {
    let mut greeting = String::from("Hello");

    // 1. Multiple shared references
    let r1 = &greeting;
    let r2 = &greeting;
    println!("{r1} and {r2}"); // Both reading simultaneously

    // 2. Mutable reference (after r1 and r2 are done)
    let r_mut = &mut greeting;
    r_mut.push_str(", Rust!");
    println!("{r_mut}"); // greeting is now "Hello, Rust!"

    // 3. Attempting to mix shared and mutable:
    // let ref_read = &greeting;
    // let ref_write = &mut greeting; // COMPILER ERROR!
    // println!("{ref_read}");
}
```

---

## Apply To MiniStore
In MiniStore:
1. **Order Preview**: A customer wants to see the total price and VIP discount before purchasing. We pass `&Product` and `&Customer` to `calculate_line_total`, `calculate_discount`, and `print_order_preview`. The customer and product are borrowed read-only, so their owners can still use them afterward.
2. **Stock Adjustments & Profile Updates**: When inventory arrives or a customer qualifies for VIP status, we pass `&mut self` to `restock` and `upgrade_to_vip`. State changes in place without copying.
3. **Checkout Finalization**: When the order is confirmed, we pass `PendingOrder` by value to `finalize_order(order: PendingOrder)`. This consumes the pending order, guaranteeing it can never be submitted again.

---

## Code
Here is the complete, runnable code for `ministore/src/main.rs`:

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

    /// Borrows `&self` immutably: checks stock without consuming or mutating the product.
    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    /// Borrows `&self` immutably: formats price for presentation.
    pub fn formatted_price(&self) -> String {
        format!("${:.2}", self.price_cents as f64 / 100.0)
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
// Borrowing Demonstration Functions: Shared (`&T`) & Mutable (`&mut T`)
// ============================================================================

/// Borrows `&Product` immutably: calculates subtotal without taking ownership.
/// The caller retains full ownership of `product`!
pub fn calculate_line_total(product: &Product, quantity: u32) -> u32 {
    product.price_cents * quantity
}

/// Borrows `&Customer` immutably: calculates discount based on VIP status.
/// The caller retains full ownership of `customer`!
pub fn calculate_discount(customer: &Customer, subtotal_cents: u32) -> u32 {
    if customer.is_vip {
        (subtotal_cents * 10) / 100 // 10% discount for VIPs
    } else {
        0
    }
}

/// Borrows both `&Customer` and `&Product` immutably to render a preview.
/// Neither value is moved or cloned.
pub fn print_order_preview(customer: &Customer, product: &Product, quantity: u32) {
    let subtotal = calculate_line_total(product, quantity);
    let discount = calculate_discount(customer, subtotal);
    let final_total = subtotal - discount;

    println!("--- Order Preview (Borrowed Read-Only) ---");
    println!("Customer: {}", customer.display_badge());
    println!(
        "Item:     {} x {} @ {}",
        product.name,
        quantity,
        product.formatted_price()
    );
    println!("Subtotal: ${:.2}", subtotal as f64 / 100.0);
    println!("Discount: ${:.2}", discount as f64 / 100.0);
    println!("Estimate: ${:.2}", final_total as f64 / 100.0);
}

/// Consumes ownership of `PendingOrder` by value (Move Semantics).
/// Contrast this with the borrowed functions above: once passed here, `order` cannot be used again!
pub fn finalize_order(order: PendingOrder) -> ConfirmedReceipt {
    let subtotal = calculate_line_total(&order.product, order.quantity);
    let discount = calculate_discount(&order.customer, subtotal);
    let total_cents = subtotal - discount;

    ConfirmedReceipt {
        receipt_id: format!("REC-{}-{}", order.order_id.0, order.customer.id),
        order_id: order.order_id,
        customer_name: order.customer.name, // Ownership of heap String moves to receipt
        product_name: order.product.name,   // Ownership of heap String moves to receipt
        quantity: order.quantity,
        total_cents,
    }
}

fn main() {
    println!("=== MiniStore: Borrowing, References & Aliasing XOR Mutability ===\n");

    // 1. Shared References (&T): Multiple Readers Without Moving Ownership
    let customer = Customer::new(
        101,
        String::from("Ada Lovelace"),
        String::from("ada@analytical.org"),
        false,
    );

    let mut product = Product::new(501, String::from("Mechanical Keyboard"), 12000, 15);

    println!("1. Shared References (&T) in Action:");
    // Both references borrow simultaneously and peacefully:
    let ref1 = &product;
    let ref2 = &product;
    println!(
        "   Simultaneous shared borrows: ref1: {}, ref2 price: {}",
        ref1.name,
        ref2.formatted_price()
    );

    // Call functions borrowing `&customer` and `&product`:
    print_order_preview(&customer, &product, 2);

    // Notice: `customer` and `product` were NOT moved! We can continue using them:
    println!(
        "\n   Original product still valid after preview: {} (Stock: {})",
        product.name, product.stock
    );

    // 2. Mutable References (&mut T): Exclusive In-Place Modification
    println!("\n2. Mutable References (&mut T) in Action:");
    // Borrow mutably to restock:
    product.restock(5);
    println!("   Restocked product: new stock = {}", product.stock);

    // Update price through a mutable borrow:
    let product_mut_ref = &mut product;
    product_mut_ref.update_price(11500); // On sale for $115.00!
    println!(
        "   Price updated via mutable reference: {}",
        product_mut_ref.formatted_price()
    );

    // 3. Non-Lexical Lifetimes (NLL) and The Golden Rule (Aliasing XOR Mutability):
    // Once `product_mut_ref` is no longer used, we can take a shared borrow again:
    let shared_after_mutation = &product;
    println!(
        "   New shared borrow after mutation completed: {} @ {}",
        shared_after_mutation.name,
        shared_after_mutation.formatted_price()
    );

    // 4. Modifying Customer via Mutable Reference:
    let mut customer_mutable = customer;
    println!("\n3. Mutating Customer Profile In-Place:");
    println!("   Before upgrade: {}", customer_mutable.display_badge());
    customer_mutable.upgrade_to_vip();
    customer_mutable.update_email(String::from("ada.lovelace@computing.org"));
    println!(
        "   After upgrade:  {} ({})",
        customer_mutable.display_badge(),
        customer_mutable.email
    );

    // 5. Finalizing an Order (Contrasting Borrowing with Moving Ownership):
    println!("\n4. Finalizing Order (Moving Ownership):");
    let pending_order = PendingOrder {
        order_id: OrderId(7001),
        customer: customer_mutable,
        product,
        quantity: 2,
    };

    // `finalize_order` takes ownership of `pending_order`:
    let receipt = finalize_order(pending_order);
    println!("   Receipt issued: {}", receipt.receipt_id);
    println!("   Customer:       {}", receipt.customer_name);
    println!(
        "   Product:        {} x {}",
        receipt.product_name, receipt.quantity
    );
    println!(
        "   Total Paid:     ${:.2}",
        receipt.total_cents as f64 / 100.0
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_borrowing_allows_multiple_readers() {
        let product = Product::new(1, String::from("USB-C Cable"), 1500, 50);

        let ref_a = &product;
        let ref_b = &product;

        // Multiple shared borrows can read simultaneously
        assert_eq!(ref_a.id, ref_b.id);
        assert_eq!(ref_a.price_cents, 1500);
        assert_eq!(ref_b.formatted_price(), "$15.00");
        assert!(ref_a.is_in_stock());
    }

    #[test]
    fn test_preview_does_not_consume_values() {
        let customer = Customer::new(1, String::from("Bob"), String::from("bob@test.com"), true);
        let product = Product::new(2, String::from("Mousepad"), 2000, 10);

        // Pass by shared reference:
        let total = calculate_line_total(&product, 3);
        let discount = calculate_discount(&customer, total);

        assert_eq!(total, 6000);
        assert_eq!(discount, 600); // 10% VIP discount on 6000

        // Customer and Product are NOT moved, still fully accessible!
        assert_eq!(customer.name, "Bob");
        assert_eq!(product.name, "Mousepad");
        assert_eq!(product.stock, 10);
    }

    #[test]
    fn test_mutable_borrowing_updates_in_place() {
        let mut product = Product::new(3, String::from("Ergo Chair"), 35000, 5);

        // Mutate via method borrowing &mut self
        product.restock(10);
        assert_eq!(product.stock, 15);

        product.reduce_stock(3).expect("Reduction should succeed");
        assert_eq!(product.stock, 12);

        // Mutate price via explicit &mut borrow
        let ref_mut = &mut product;
        ref_mut.update_price(32000);
        assert_eq!(ref_mut.price_cents, 32000);

        // Original variable reflects the in-place mutations
        assert_eq!(product.stock, 12);
        assert_eq!(product.price_cents, 32000);
    }

    #[test]
    fn test_customer_mutations_via_ref() {
        let mut customer = Customer::new(
            10,
            String::from("Carol"),
            String::from("carol@old.com"),
            false,
        );

        assert_eq!(customer.display_badge(), "[Standard Member] Carol");
        assert!(!customer.is_vip);

        customer.upgrade_to_vip();
        customer.update_email(String::from("carol@new.com"));

        assert_eq!(customer.display_badge(), "[VIP Member] Carol");
        assert!(customer.is_vip);
        assert_eq!(customer.email, "carol@new.com");
    }

    #[test]
    fn test_finalize_order_consumes_and_calculates_vip_discount() {
        let customer = Customer::new(1, String::from("Alice"), String::from("a@test.com"), true);
        let product = Product::new(10, String::from("Desk Pad"), 2000, 5);
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

### 1. Method Receivers: `self` vs `&self` vs `&mut self`
Look closely at how methods are declared on `Product`:

```rust
impl Product {
    // 1. &self: Borrows product read-only
    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    // 2. &mut self: Borrows product mutably for in-place updates
    pub fn restock(&mut self, additional_units: u32) {
        self.stock += additional_units;
    }
}
```
- **`&self`**: Borrows the instance immutably. The method can read fields but cannot alter them. Callers retain ownership.
- **`&mut self`**: Borrows the instance mutably. The method has exclusive access and can modify fields. Callers retain ownership after the method finishes.
- **`self`**: Takes ownership by value. The method consumes the instance; the caller cannot use it again (e.g. `finalize_order`).

### 2. Auto-Dereferencing (The Dot Operator)
Notice that inside `calculate_line_total(product: &Product, quantity: u32)`:
```rust
product.price_cents * quantity
```
Even though `product` is a reference `&Product`, we do not have to write `(*product).price_cents`. Rust's dot operator `.` automatically follows the reference (called **auto-dereferencing**).

---

## Common Mistakes

### 1. Mutating Through a Shared Reference
```rust
fn apply_sale(product: &Product) {
    product.price_cents = 1000; // COMPILER ERROR: cannot assign to `product.price_cents`
}
```
A shared reference `&T` is strictly immutable. If you need to mutate, the parameter must be `&mut Product`, and the caller must provide a mutable reference to a `mut` variable.

### 2. Retaining a Shared Reference While Mutating
```rust
let mut product = Product::new(1, String::from("Pen"), 200, 10);
let r1 = &product; // Immutable borrow begins
product.restock(5); // ERROR: cannot borrow `product` as mutable while also borrowed as immutable
println!("{}", r1.stock); // `r1` used here
```
Because `r1` is read on the last line, its borrow overlaps with `product.restock(5)`. The compiler forbids this to prevent data corruption.

---

## Compiler Errors

### Error E0502: Cannot Borrow as Mutable While Also Borrowed as Immutable
Suppose you write:

```rust
let mut product = Product::new(1, String::from("Webcam"), 4500, 5);
let name_ref = &product.name;
product.restock(10);
println!("Product: {name_ref}");
```

The Rust compiler rejects this immediately:

```text
error[E0502]: cannot borrow `product` as mutable because it is also borrowed as immutable
  --> src/main.rs:185:5
   |
184|     let name_ref = &product.name;
   |                    ------------- immutable borrow occurs here
185|     product.restock(10);
   |     ^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
186|     println!("Product: {name_ref}");
   |                        ---------- immutable borrow later used here
```

**Why this matters**: In languages like C++, mutating a container or object while holding a reference to one of its fields can reallocate memory, leaving `name_ref` pointing to garbage. In Rust, this bug cannot even compile.

---

## Practice
1. **Add Price Surcharge**: Write a function `add_tax(product: &mut Product, tax_percentage: u32)` that increases `product.price_cents` by the given percentage in place.
2. **Bulk Stock Checker**: Write a function `is_any_out_of_stock(p1: &Product, p2: &Product) -> bool` that takes two shared references and returns `true` if either product has `stock == 0`.
3. **Verify in Tests**: Add unit tests in `src/main.rs` to verify that `add_tax` modifies the price accurately without transferring ownership.
4. Run `cargo test` to ensure all tests pass.

---

## Checkpoint
- [x] Mastered the difference between passing by value (ownership transfer) and passing by reference (borrowing).
- [x] Understood shared references (`&T`) and multiple reader access.
- [x] Understood mutable references (`&mut T`) and exclusive single writer access.
- [x] Understood the Aliasing XOR Mutability rule and why it guarantees thread and memory safety.
- [x] Understood Non-Lexical Lifetimes (NLL) and when borrows end.
- [x] Utilized `&self` and `&mut self` methods on structs to build clean MiniStore APIs.

---

## What We Learned
- Borrowing eliminates unnecessary memory cloning by lending temporary read or write access.
- The Borrow Checker enforces that data cannot be mutated while it is being read, preventing data races and iterator invalidation before the code ever runs.
- MiniStore can now inspect products and customers during preview workflows, adjust inventory in-place, and consume orders upon final checkout.

---

## What's Next
Now that we understand borrowing single objects, what about parts of collections or strings? In **Chapter 8: Strings, Slices and Ownership in Practice**, we will explore string slices (`&str`), array slices (`&[T]`), and how Rust manages zero-copy views into contiguous memory buffers.
