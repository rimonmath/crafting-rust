# Chapter 4: Control Flow

## What You'll Learn
- How expressions differ fundamentally from statements in Rust.
- Using `if / else` as an **expression** that yields a value (and why Rust does not need a ternary operator).
- Unconditional looping with `loop`, returning values from loops using `break <value>`.
- Disambiguating nested loops with **loop labels** (e.g., `'outer: loop`).
- Conditional execution with `while` loops.
- Safe and idiomatic iteration using `for` loops over ranges and array slices.
- Implementing an e-commerce shopping cart checkout engine in MiniStore: calculating order subtotals, tiered promotional discounts, conditional shipping fees, and inventory packaging.

---

## Why Do We Need This?
Real software is not a linear sequence of instructions. It must make decisions and process repetitive datasets:
- Calculating tiered discounts based on cart values.
- Granting free shipping once an order subtotal exceeds a promotional threshold.
- Iterating through line items in a customer's shopping cart to compute totals.
- Packaging items into boxes and tracking dispatch queues.

In many languages (such as C, Java, or Python), control flow structures like `if` and `switch` are **statements**. They perform side effects (such as mutating a variable declared outside the statement):

```javascript
// JavaScript / Java: Imperative statement with mutable variable
let discountRate;
if (subtotal > 10000) {
    discountRate = 0.15;
} else {
    discountRate = 0.05;
}
```

Notice how `discountRate` had to be declared uninitialized or mutable. If a developer forgets an `else` branch or inadvertently reassigns it later, subtle state corruption occurs.

Rust takes a functional approach: **`if` is an expression**. It produces a value directly. This allows variables to remain completely immutable:

```rust
// Rust: Functional expression yielding an immutable binding
let discount_rate = if subtotal > 10000 { 15 } else { 5 };
```

---

## The Problem
When building MiniStore's order processing pipeline, we encounter three core design challenges:

1. **State Mutation During Evaluation**: Calculating order totals, discounts, and delivery fees without creating mutable intermediate variables that could be corrupted.
2. **Loop Exit Values**: Scanning through an array of items to locate a specific product (such as the highest-priced item) without maintaining manual index flags or uninitialized variables outside the loop.
3. **Array Indexing Out of Bounds**: In C-style `for (int i = 0; i <= count; i++)` loops, off-by-one errors frequently cause buffer overreads or panics. We need an iteration mechanism guaranteed to never exceed array boundaries.

---

## Rust Concept

### 1. Statements vs Expressions
Rust is primarily an **expression-based language**:
- **Statements**: Instructions that perform an action and do not return a value. Statements end with a semicolon (`;`). For example, `let x = 5;` is a statement.
- **Expressions**: Evaluate to a resultant value. Expressions do not end with a semicolon. If you add a semicolon to the end of an expression, you turn it into a statement that yields the **unit type** `()`.

```rust
// The block { ... } evaluates to 15 because the last line has NO semicolon
let total = {
    let base = 10;
    let tax = 5;
    base + tax // Tail expression: returns 15
};
```

### 2. `if / else` Expressions
Because `if` is an expression, both branches must evaluate to the **same concrete type**:

```rust
let shipping_fee = if order_total >= 10000 {
    0
} else {
    599
};
```

> [!IMPORTANT]
> **No Ternary Operator**: Rust has no `condition ? a : b` syntax. `if condition { a } else { b }` serves the exact same role with zero runtime overhead and greater readability.

If you omit the `else` branch, the `if` expression must evaluate to `()`. You cannot write `let x = if condition { 10 };` because if the condition is false, `x` would have no value.

### 3. Infinite Looping and Returning Values with `loop`
Rust provides `loop` for unconditional repetition:
```rust
loop {
    println!("Runs forever unless broken");
    break;
}
```
`loop` is also an expression! You can pass a value to `break`, and `loop` will return that value:

```rust
let mut counter = 0;
let result = loop {
    counter += 1;
    if counter == 10 {
        break counter * 2; // Returns 20 from the loop
    }
};
```

### 4. Loop Labels for Disambiguation
When nesting loops, `break` and `continue` apply to the innermost loop by default. To break or continue an outer loop, annotate it with a **loop label** (starting with a single quote `'`):

```rust
'outer: loop {
    'inner: loop {
        break 'outer; // Exits the outer loop directly!
    }
}
```

### 5. Conditional Looping with `while`
A `while` loop evaluates a condition before each iteration:

```rust
let mut queue = 3;
while queue > 0 {
    println!("Processing order #{queue}");
    queue -= 1;
}
```

### 6. Idiomatic Iteration with `for`
Rather than manually managing indices (`while i < items.len()`), Rust developers almost always use `for .. in`:

```rust
// Iterating through an array slice
for item in cart {
    println!("{}", item.0);
}

// Iterating over an exclusive numeric range 0..3 (0, 1, 2)
for i in 0..3 {
    println!("Step {i}");
}

// Iterating over an inclusive range 1..=5 (1, 2, 3, 4, 5)
for i in 1..=5 {
    println!("Count {i}");
}
```

`for` loops in Rust are:
1. **Safer**: Impossible to trigger out-of-bounds panics or infinite loops due to forgotten index increments.
2. **Faster**: The compiler eliminates runtime bounds checks because it mathematically proves the bounds upfront.

---

## Coming From Other Languages

| Concept | Python | Go | Java / C# / C++ | Rust |
| :--- | :--- | :--- | :--- | :--- |
| **`if` as Expression** | `a if cond else b` (ternary only) | Statements only | `cond ? a : b` (ternary only) | **Native `if / else` expression** |
| **Branch Type Rule** | Can return dynamic types | N/A (statement) | Ternary requires compatible type | **Strict static match across all arms** |
| **Break with Value** | Not supported | Not supported | Not supported | **`break <value>;` inside `loop`** |
| **Loop Labels** | Not supported | Labels with `break Label` | `label: for(...)` with `break label;` | **`'label: loop` with `break 'label;`** |
| **Array Iteration** | `for item in items:` | `for _, item := range items` | `for (T item : items)` | **`for item in items` (zero-cost iterator)** |

---

## Small Example: Expressions & Loop Values

```rust
fn main() {
    // 1. `if` expression assigning directly to an immutable variable
    let cart_value = 12000; // $120.00 in cents
    let discount_pct = if cart_value > 10000 { 15 } else { 5 };
    println!("Discount: {discount_pct}%");

    // 2. Returning a value from a `loop` expression
    let mut attempt = 0;
    let auth_token = loop {
        attempt += 1;
        if attempt == 3 {
            break "TOKEN_SECURE_XYZ_99";
        }
    };
    println!("Acquired token on attempt {attempt}: {auth_token}");
}
```

---

## Apply To MiniStore
In this chapter, MiniStore advances to **cart and order processing**:
1. Store a fixed catalog of cart items as tuples `(&str, u32, u32)` containing `(name, unit_price_cents, quantity)`.
2. Compute the subtotal using a safe `for` loop.
3. Determine a tiered volume discount (0%, 5%, 10%, or 15%) using an `if / else` expression.
4. Calculate delivery charges: free shipping on orders of $100 or more; otherwise, standard $5.99.
5. Use a `loop` expression with `break value` to identify the most expensive product in the cart.
6. Simulate packing items into shipping cartons (max 3 per carton) using nested loops and **loop labels**.
7. Dispatch the completed shipments with a `while` loop.

---

## Code

### `src/main.rs`
```rust
fn main() {
    println!("=== MiniStore: Cart & Order Processing ===");

    // Cart representation using fixed-size array of tuples:
    // (item_name: &str, unit_price_cents: u32, quantity: u32)
    let cart: [(&str, u32, u32); 4] = [
        ("Mechanical Keyboard", 8999, 1),
        ("USB-C Cable", 1299, 2),
        ("Mouse Pad", 1999, 1),
        ("Keycap Puller", 599, 3),
    ];

    println!("--- Cart Items ---");
    // 1. `for` loop: idiomatic iteration over array collections
    for item in cart {
        let (name, price_cents, qty) = item;
        let item_total = price_cents * qty;
        println!(
            "- {name} x {qty} @ ${:.2} each = ${:.2}",
            price_cents as f64 / 100.0,
            item_total as f64 / 100.0
        );
    }

    let subtotal_cents = calculate_subtotal(&cart);
    println!("\nCart Subtotal: ${:.2}", subtotal_cents as f64 / 100.0);

    // 2. `if / else` expression: assigns discount percentage based on subtotal tiers
    let discount_percent: u32 = calculate_tier_discount(subtotal_cents);
    let discount_amount_cents = (subtotal_cents * discount_percent) / 100;
    let discounted_subtotal = subtotal_cents - discount_amount_cents;

    println!(
        "Tier Discount: {discount_percent}% (-${:.2})",
        discount_amount_cents as f64 / 100.0
    );
    println!(
        "Discounted Subtotal: ${:.2}",
        discounted_subtotal as f64 / 100.0
    );

    // 3. `if` expression to compute shipping cost (free shipping over $100.00 = 10000 cents)
    let shipping_fee_cents: u32 = if discounted_subtotal >= 10000 {
        0
    } else {
        599 // $5.99 standard shipping
    };

    if shipping_fee_cents == 0 {
        println!("Shipping: FREE (Orders over $100 qualify for free shipping)");
    } else {
        println!("Shipping: ${:.2}", shipping_fee_cents as f64 / 100.0);
    }

    let total_order_cents = discounted_subtotal + shipping_fee_cents;
    println!(
        "Final Order Total: ${:.2}",
        total_order_cents as f64 / 100.0
    );

    // 4. `loop` with `break value`: identify the highest-value single item in cart
    let most_expensive_item = find_most_expensive_item(&cart);
    println!(
        "Highest Value Item: {} (${:.2})",
        most_expensive_item.0,
        most_expensive_item.1 as f64 / 100.0
    );

    // 5. Nested loop with loop label: batch packaging items into boxes
    println!("\n--- Packaging Simulation (Nested Loops with Labels) ---");
    let mut total_packed = 0;
    let total_items_count: u32 = 7; // 1 + 2 + 1 + 3

    'outer_box: loop {
        println!("Opened a new shipping box...");

        'pack_items: loop {
            total_packed += 1;
            println!("  Packed item {total_packed} of {total_items_count}");

            if total_packed >= total_items_count {
                println!("All items safely packed!");
                break 'outer_box; // breaks the outer loop directly!
            }

            // Each shipping box holds a maximum of 3 items
            if total_packed % 3 == 0 {
                println!("  Box full (3 items). Sealing box.");
                break 'pack_items; // breaks inner loop to start next box
            }
        }
    }

    // 6. `while` loop: inventory dispatch queue
    println!("\n--- Warehouse Dispatch Queue (while loop) ---");
    let mut orders_in_queue = 3;
    while orders_in_queue > 0 {
        println!("Dispatching order #{orders_in_queue} to carrier...");
        orders_in_queue -= 1;
    }
    println!("All pending orders dispatched!");
}

/// Calculates the subtotal price in cents for an array of cart items
pub fn calculate_subtotal(cart: &[(&str, u32, u32)]) -> u32 {
    let mut total = 0;
    for &(_, price_cents, quantity) in cart {
        total += price_cents * quantity;
    }
    total
}

/// Returns a discount percentage (0, 5, 10, or 15) using an `if / else` expression
pub fn calculate_tier_discount(subtotal_cents: u32) -> u32 {
    if subtotal_cents >= 15000 {
        15 // 15% discount for orders >= $150
    } else if subtotal_cents >= 10000 {
        10 // 10% discount for orders >= $100
    } else if subtotal_cents >= 5000 {
        5 // 5% discount for orders >= $50
    } else {
        0 // No discount
    }
}

/// Computes shipping fee: free if order meets threshold, standard otherwise
pub fn calculate_shipping_fee(discounted_subtotal_cents: u32) -> u32 {
    if discounted_subtotal_cents >= 10000 {
        0
    } else {
        599
    }
}

/// Scans the cart using a `loop` expression with `break value` to return the highest-priced item
pub fn find_most_expensive_item<'a>(cart: &[(&'a str, u32, u32)]) -> (&'a str, u32) {
    if cart.is_empty() {
        return ("None", 0);
    }

    let mut index = 0;
    let mut highest = cart[0];

    loop {
        if index >= cart.len() {
            break (highest.0, highest.1);
        }

        if cart[index].1 > highest.1 {
            highest = cart[index];
        }

        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_subtotal() {
        let sample_cart = [
            ("Item A", 1000, 2), // 2000
            ("Item B", 2500, 1), // 2500
        ];
        assert_eq!(calculate_subtotal(&sample_cart), 4500);
    }

    #[test]
    fn test_tier_discounts() {
        assert_eq!(calculate_tier_discount(16000), 15);
        assert_eq!(calculate_tier_discount(15000), 15);
        assert_eq!(calculate_tier_discount(12000), 10);
        assert_eq!(calculate_tier_discount(10000), 10);
        assert_eq!(calculate_tier_discount(7500), 5);
        assert_eq!(calculate_tier_discount(5000), 5);
        assert_eq!(calculate_tier_discount(4999), 0);
    }

    #[test]
    fn test_shipping_fee_threshold() {
        assert_eq!(calculate_shipping_fee(10000), 0);
        assert_eq!(calculate_shipping_fee(15000), 0);
        assert_eq!(calculate_shipping_fee(9999), 599);
        assert_eq!(calculate_shipping_fee(0), 599);
    }

    #[test]
    fn test_find_most_expensive_item() {
        let sample_cart = [("Cable", 500, 2), ("Monitor", 25000, 1), ("Mouse", 3000, 1)];
        let (name, price) = find_most_expensive_item(&sample_cart);
        assert_eq!(name, "Monitor");
        assert_eq!(price, 25000);
    }
}
```

---

## Understanding The Code

1. **`if / else` as Return Expressions**:
   ```rust
   pub fn calculate_tier_discount(subtotal_cents: u32) -> u32 {
       if subtotal_cents >= 15000 {
           15
       } else if subtotal_cents >= 10000 {
           10
       } else if subtotal_cents >= 5000 {
           5
       } else {
           0
       }
   }
   ```
   Notice there is no `return` keyword and no semicolons after the numbers. The entire `if / else if / else` construct is a single expression acting as the body of the function.

2. **Loop Exit with Value**:
   ```rust
   loop {
       if index >= cart.len() {
           break (highest.0, highest.1);
       }
       ...
   }
   ```
   `break (highest.0, highest.1);` halts the loop and supplies the tuple `(&str, u32)` as the resulting value of the `loop` expression.

3. **Loop Labels**:
   ```rust
   'outer_box: loop {
       'pack_items: loop {
           if total_packed >= total_items_count {
               break 'outer_box;
           }
       }
   }
   ```
   Labels enable clean early termination across nested boundaries without awkward boolean sentinel flags (like `bool should_exit = false;`).

---

## Common Mistakes

### 1. Incompatible Branch Types in `if / else`
Every branch in an `if / else` expression must return the identical type. The compiler cannot reconcile differing types:

```rust
// COMPILER ERROR: mismatched types
let fee = if is_vip {
    0 // integer
} else {
    "standard" // string slice!
};
```

### 2. Semicolons in Tail Expressions
Placing a semicolon on the last line turns it into a statement, returning `()` (the unit type):

```rust
fn get_discount() -> u32 {
    let subtotal = 10000;
    if subtotal > 5000 {
        10; // ERROR: semicolon turns 10 into (), but function returns u32!
    } else {
        0;
    }
}
```

---

## Compiler Errors
What happens if you use `if` without `else` to assign a variable?

```rust
fn main() {
    let condition = true;
    let score = if condition { 100 };
}
```

Compiler output:
```text
error[E0317]: `if` may be missing an `else` clause
 --> src/main.rs:3:17
  |
3 |     let score = if condition { 100 };
  |                 ^^^^^^^^^^^^^^^^^^^^ expected `()`, found integer
  |
  = note: `if` expressions without `else` have the type `()`
  = help: consider adding an `else` block that evaluates to the expected type
```
Because the compiler cannot know what value `score` should have when `condition` is `false`, any `if` without `else` must have type `()`.

---

## Practice
1. Modify `calculate_shipping_fee` to accept a boolean flag `is_express: bool`. If express delivery is chosen, add `1000` cents ($10.00) to the standard fee, but offer express shipping for only `500` cents if the order exceeds `$100.00`.
2. Write a unit test `test_express_shipping()` to verify both cases.
3. Run `cargo test` and verify that all tests pass.

---

## Checkpoint
- [x] Learned the difference between statements (semicolon) and expressions (value-producing).
- [x] Replaced ternary operators with idiomatic Rust `if / else` expressions.
- [x] Used `loop` with `break value` to return computation results.
- [x] Controlled nested iteration using loop labels (`'label:`).
- [x] Iterated over arrays cleanly with `for` loops.
- [x] Verified all unit tests with `cargo test`.

---

## What We Learned
- Rust's expression-oriented nature reduces state mutation and produces concise, robust code.
- Eliminating ternary operators in favor of `if / else` expressions simplifies language grammar while preserving strong type safety.
- `for` loops in Rust eliminate index bounds errors entirely.

---

## What's Next
In **Chapter 5: Structs: Modeling Real Things**, we will move beyond tuples and raw arrays to create custom domain types (`Product`, `CartItem`, `Order`) with named fields and methods.
