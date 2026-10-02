# Chapter 3: Rust's Basic Building Blocks

## What You'll Learn
- How variables work in Rust: default immutability and explicit mutability (`mut`).
- The power of **Variable Shadowing** and how it differs fundamentally from mutation.
- Rust's scalar data types: integers, floats, booleans, and 4-byte Unicode characters.
- Why e-commerce engines (like MiniStore) store currency in integer cents rather than floats.
- Compound data structures: **Tuples** and stack-allocated **Arrays**.
- Modeling our first inventory metrics and price calculation functions.

---

## Why Do We Need This?
Every application processes data. In dynamically typed languages (Python, JavaScript), variable types can morph arbitrarily, leading to runtime type errors:

```javascript
// JavaScript: Silent bugs in production
let price = "49.99";
let quantity = 2;
let total = price + quantity; // "49.992" — string concatenation instead of math!
```

In weakly typed or legacy compiled languages (C, C++), implicit type coercion and unexpected integer promotions cause security exploits and silent overflow vulnerabilities.

Rust enforces strict static typing with type inference: the compiler almost always infers the correct type, but never allows implicit conversions. If you want to convert a `u32` to an `f64`, you must do so explicitly with `as` or `From::from`.

---

## The Problem
Consider two common challenges in e-commerce applications:

1. **Unintended State Mutation**: An inventory count or order total is modified by a downstream function because the object was passed by reference as mutable.
2. **Type Transformation Inconvenience**: Parsing raw user input (e.g., HTTP query param `"discount": "15"`) into a typed integer (`15`). In many languages, you are forced to invent awkward temporary names like `discount_str` and `discount_int`.

Rust solves both challenges at the language level through **Immutability by Default** and **Variable Shadowing**.

---

## Rust Concept

### 1. Immutability & `mut`
By default, variables declared with `let` are immutable:
```rust
let price = 1000;
// price = 1200; // COMPILER ERROR: cannot assign twice to immutable variable
```

When mutation is truly required by your domain logic, you declare intent explicitly:
```rust
let mut stock = 10;
stock -= 1; // Explicitly allowed
```

### 2. Variable Shadowing
Shadowing allows you to re-declare a variable with the same name using the `let` keyword:

```rust
let discount = "15";              // Type is &str (string slice)
let discount: u32 = discount.parse().unwrap(); // Type is now u32!
```

**Shadowing is not mutation**:
- Shadowing creates a brand new variable that overshadows the previous one within its scope.
- It allows you to **change the type** of the variable without inventing temporary throwaway names.
- The new variable remains immutable unless explicitly marked `mut`.

### 3. Scalar Types

| Category | Types | Details |
| :--- | :--- | :--- |
| **Unsigned Integers** | `u8`, `u16`, `u32`, `u64`, `u128`, `usize` | Positive numbers only. `usize` matches target pointer width (64-bit on x86_64). |
| **Signed Integers** | `i8`, `i16`, `i32`, `i64`, `i128`, `isize` | Two's complement signed integers. |
| **Floats** | `f32`, `f64` | IEEE 754 floating-point. Default is `f64`. |
| **Boolean** | `bool` | `true` or `false` (1 byte in memory). |
| **Character** | `char` | 4-byte Unicode Scalar Value (represents ASCII, emojis, and all international scripts). |

> [!IMPORTANT]
> **Currency in MiniStore**: We store prices as `u32` representing cents (e.g., `$89.99` = `8999`). In floating-point arithmetic, `0.1 + 0.2 == 0.30000000000000004`, which introduces rounding errors that violate financial accounting standards.

### 4. Compound Types

```rust
// Tuple: Fixed size, heterogeneous (can mix types)
let product_tuple: (u64, &str, u32) = (1001, "Keyboard", 8999);
let id = product_tuple.0; // Index access

// Array: Fixed size, homogeneous (same type), stack-allocated
let daily_sales: [u32; 3] = [5, 8, 2];
let day_one = daily_sales[0];
```

---

## Coming From Other Languages

| Concept | Python / JS | Go | Java / C# | Rust |
| :--- | :--- | :--- | :--- | :--- |
| **Default Mutability** | Always mutable | Always mutable | Mutable by default (`final`/`readonly`) | **Immutable by default** |
| **Variable Shadowing** | Overwrites in scope | Re-declaration in same scope is illegal | Illegal in same block | **First-class feature** (`let x = ...`) |
| **Character Type** | 1-char string / UTF-16 | `rune` (alias for `int32`) | `char` (2-byte UTF-16) | `char` (4-byte Unicode scalar) |
| **Fixed Array** | Dynamic `list` / `Array` | `[N]T` (value type) | `T[]` (heap allocated) | `[T; N]` (stack allocated) |

---

## Small Example: Shadowing vs Mutation

```rust
fn main() {
    // Mutation: Type CANNOT change
    let mut stock = 20;
    stock = 15; // OK: same type
    // stock = "out of stock"; // ERROR: mismatched types!

    // Shadowing: Type CAN change, and immutability is preserved
    let raw_input = "  42  ";
    let raw_input: u32 = raw_input.trim().parse().unwrap();
    println!("Parsed: {}", raw_input);
}
```

---

## Apply To MiniStore
In MiniStore, we need to track:
1. Product metadata: unique ID (`u64`), human-readable name (`&str`), price in cents (`u32`).
2. Live inventory stock that decrements upon purchase (`mut stock_quantity: u32`).
3. Parsing promotional discounts and safely computing the final price.
4. Historical sales metrics over a 3-day window using a stack-allocated fixed array.

---

## Code

### `src/main.rs`
```rust
fn main() {
    println!("=== MiniStore: Inventory & Pricing ===");

    // 1. Immutable scalar values (explicit types)
    let product_id: u64 = 1001;
    let product_name: &str = "Mechanical Keyboard";
    let base_price_cents: u32 = 8999; // $89.99

    // 2. Mutable variable (stock changes as sales occur)
    let mut stock_quantity: u32 = 25;
    println!("Product #{product_id}: {product_name}");
    println!("Price: ${:.2}", base_price_cents as f64 / 100.0);
    println!("Initial Stock: {stock_quantity}");

    // Simulate customer purchases 2 units using helper function
    let units_sold: u32 = 2;
    match reduce_stock(stock_quantity, units_sold) {
        Ok(new_stock) => {
            stock_quantity = new_stock;
            println!("Units Sold: {units_sold}");
            println!("Remaining Stock: {stock_quantity}");
        }
        Err(err) => println!("Failed to sell units: {err}"),
    }

    // 3. Variable Shadowing (transforming a discount representation)
    let discount = "10"; // user entered string "10"%
    let discount: u32 = discount.parse().unwrap_or(0); // parsed into integer 10
    let final_price_cents = calculate_discounted_price(base_price_cents, discount);
    let discount_amount = base_price_cents - final_price_cents;
    println!(
        "Discount: {discount}% (-${:.2})",
        discount_amount as f64 / 100.0
    );
    println!("Final Price: ${:.2}", final_price_cents as f64 / 100.0);

    // 4. Compound Types: Tuples and Arrays
    let item_summary: (u64, &str, u32) = (product_id, product_name, final_price_cents);
    println!("Item Summary Tuple: {:?}", item_summary);

    let last_three_days_sales: [u32; 3] = [5, 8, 2];
    let total_recent_sales: u32 =
        last_three_days_sales[0] + last_three_days_sales[1] + last_three_days_sales[2];
    println!("Total Sales (Last 3 Days): {total_recent_sales}");
}

/// Applies a percentage discount in integer arithmetic to prevent floating-point inaccuracies
fn calculate_discounted_price(price_cents: u32, discount_percent: u32) -> u32 {
    let discount_amount = (price_cents * discount_percent) / 100;
    price_cents - discount_amount
}

/// Safely decrements stock, ensuring we do not underflow
fn reduce_stock(current_stock: u32, quantity: u32) -> Result<u32, &'static str> {
    if quantity > current_stock {
        Err("Insufficient stock")
    } else {
        Ok(current_stock - quantity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discount_calculation() {
        let original_price = 10000; // $100.00
        let discounted = calculate_discounted_price(original_price, 20); // 20% off
        assert_eq!(discounted, 8000); // $80.00
    }

    #[test]
    fn test_stock_reduction_success() {
        let stock = 10;
        let updated = reduce_stock(stock, 3).expect("Should succeed");
        assert_eq!(updated, 7);
    }

    #[test]
    fn test_stock_reduction_insufficient() {
        let stock = 2;
        let result = reduce_stock(stock, 5);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient stock");
    }

    #[test]
    fn test_shadowing_type_change() {
        let input = "42";
        let input: u32 = input.parse().unwrap();
        assert_eq!(input, 42);
    }
}
```

---

## Understanding The Code

1. **Integer Money Math**:
   ```rust
   let discount_amount = (price_cents * discount_percent) / 100;
   ```
   Integer division in Rust truncates towards zero. In financial calculation, working in cents guarantees deterministic rounding without floating-point inaccuracies.

2. **Shadowing for Parsing**:
   ```rust
   let discount = "10";
   let discount: u32 = discount.parse().unwrap_or(0);
   ```
   Here `discount` begins as an immutable string slice `&str` and is shadowed into an immutable `u32`. The original string is no longer accessible by this name, eliminating the risk of accidental usage.

3. **Tuple Indexing & Debug Printing**:
   - `item_summary.0`, `item_summary.1`: Tuples use zero-indexed dot notation.
   - `{:?}`: The **Debug format specifier**, used when printing compound types that implement the `Debug` trait.

---

## Common Mistakes

### 1. Trying to Change a Mutable Variable's Type
```rust
let mut price = 100;
price = "120"; // ERROR!
```
`mut` allows changing the **value**, but the **type** remains strictly locked to `i32`. To change the type, use **shadowing** with `let price = ...`.

### 2. Assuming Unsigned Integers Silently Wrap
In C, decrementing an unsigned integer past zero wraps around silently. In Rust:
```rust
let stock: u32 = 0;
// let next_stock = stock - 1; // PANICS in debug mode! (attempt to subtract with overflow)
```
In debug mode, Rust inserts checks for integer overflow and panics. In release mode (`--release`), it performs two's complement wrapping. Our `reduce_stock` function guards against this by checking `quantity > current_stock`.

---

## Compiler Errors
What happens when you reassign an immutable variable?

```rust
fn main() {
    let stock = 10;
    stock = 5;
}
```

Compiler output:
```text
error[E0384]: cannot assign twice to immutable variable `stock`
 --> src/main.rs:3:5
  |
2 |     let stock = 10;
  |         -----
  |         |
  |         first assignment to `stock`
  |         help: consider making this binding mutable: `mut stock`
3 |     stock = 5;
  |     ^^^^^^^^^ cannot assign twice to immutable variable
```

---

## Practice
1. Open `ministore/src/main.rs` and add a new immutable variable for tax percentage (e.g., `let tax_rate_percent: u32 = 5;`).
2. Write a function `calculate_tax(price_cents: u32, tax_rate_percent: u32) -> u32`.
3. Add a unit test verifying that a `$50.00` (`5000` cents) item with `5%` tax produces `250` cents in tax.
4. Run `cargo test` to verify your test passes.

---

## Checkpoint
- [x] Mastered the difference between `let`, `let mut`, and variable shadowing.
- [x] Understood scalar types and why financial calculations must avoid floating-point errors.
- [x] Used tuples and fixed-size stack arrays.
- [x] Verified unit tests with `cargo test`.

---

## What We Learned
- Default immutability forces developers to think carefully about state transitions.
- Shadowing cleanly handles data transformations without polluting scope with auxiliary names.
- Rust numeric types are explicit and prevent silent data loss or unexpected overflows.

---

## What's Next
In **Chapter 4: Control Flow**, we will explore `if / else` expressions (and why they return values in Rust), `loop`, `while`, `for`, and loop labels to implement shopping carts and inventory batch processing.
