# Chapter 25: Declarative Macros

## What You'll Learn

- What **Metaprogramming** means in Rust and how macros write code that writes code at compile time.
- Why functions cannot solve every duplication problem (fixed arity, lack of syntax inspection, runtime overhead).
- The difference between C's textual `#define` preprocessor and Rust's **hygienic, AST-aware macro system**.
- The core syntax of **Declarative Macros (`macro_rules!`)**:
  - **Designators**: `$expr`, `$ident`, `$ty`, `$pat`, `$literal`, and `$tt`.
  - **Repetition patterns**: `$( ... ),*`, `$( ... ),+`, and trailing comma matching `$(,)?`.
  - **Compile-time pattern matching** with multiple matcher branches.
- Building domain DSLs (Domain-Specific Languages) for MiniStore:
  - `product!`: Concise instantiation with identifier shorthand for enums and default values.
  - `catalog!`: Initializing collections with comma-separated syntax.
  - `cart!`: Multi-syntax matching (tuples vs. named fields).
  - `calculate_total!`: Variadic recursion pattern (base case + inductive step).
  - `assert_in_stock!`: Custom diagnostic assertion macros with expressive panic messages.
- Understanding **Macro Hygiene** and why `$crate` is essential for exported macros (`#[macro_export]`).

---

## Why Do We Need Macros?

Throughout our journey building MiniStore, we've repeatedly constructed products, catalogs, and shopping carts in test suites, setup scripts, and CLI demos:

```rust
// Verbose boilerplate required for every product:
let p1 = Product::new(
    101,
    String::from("TECH-KEY-001"),
    String::from("Mechanical Keyboard"),
    ProductCategory::Electronics,
    15000,
    10,
);

let mut catalog = Catalog::new();
catalog.add_product(p1);
catalog.add_product(p2);
catalog.add_product(p3);

let mut cart = ShoppingCart::new();
cart.add_item(101, 2, 15000);
cart.add_item(102, 5, 1200);
```

While clean and explicit, this becomes tedious when authoring hundreds of unit tests, fixture seeds, or domain scenarios.

### Why Functions Aren't Enough

Could we solve this with regular Rust functions? Not entirely:

1. **Fixed Arity**: A function in Rust must accept a fixed number of parameters. You cannot write a function `catalog(p1, p2, p3)` that also accepts `catalog(p1)` or `catalog(p1, p2, p3, p4)` without wrapping them in an array or slice.
2. **Cannot Inspect Syntax**: A function cannot accept the bare identifier `Electronics` and convert it into `ProductCategory::Electronics`. A function requires the full type.
3. **Cannot Generate Code**: Functions execute at **runtime**. They cannot generate struct fields, implement traits across multiple types, or manipulate the abstract syntax tree.

### C Preprocessor vs. Rust Macros

In languages like C or C++, macros are powered by `#define`, which is simple, unhygienic text substitution:

```c
// Dangerous C macro:
#define SQUARE(x) x * x

int result = SQUARE(1 + 2); // Expands to: 1 + 2 * 1 + 2 = 5 (NOT 9!)
```

In Rust, macros operate on **Token Streams** and the **Abstract Syntax Tree (AST)**:

- They respect operator precedence.
- They are **hygienic**: local variables declared inside a macro do not leak into the caller's scope or accidentally shadow variables.
- They are completely type-checked after macro expansion.

---

## Declarative Macros Anatomy (`macro_rules!`)

Declarative macros are defined using the `macro_rules!` construct. They look like a compile-time `match` expression:

```rust
macro_rules! my_macro {
    ( matcher_pattern_1 ) => {
        expansion_code_1
    };
    ( matcher_pattern_2 ) => {
        expansion_code_2
    };
}
```

### 1. Macro Designators

When matching inputs, variables are prefixed with `$` followed by a **designator** indicating what syntactic element is expected:

| Designator     | Description                                                   | Example Match                     |
| :------------- | :------------------------------------------------------------ | :-------------------------------- |
| **`$expr`**    | Any valid Rust expression                                     | `42`, `1 + 2`, `foo()`            |
| **`$ident`**   | An identifier (variable, function, or enum variant name)      | `Electronics`, `my_var`           |
| **`$ty`**      | A Rust type                                                   | `u32`, `Vec<String>`, `Option<T>` |
| **`$pat`**     | A pattern used in `match` or `let`                            | `Some(x)`, `1..=10`               |
| **`$literal`** | A literal value                                               | `"hello"`, `100`, `true`          |
| **`$tt`**      | A single token tree (any bracketed group or individual token) | `(1, 2)`, `{ ... }`               |

### 2. Repetition Patterns

To accept a variable number of arguments (like `vec![1, 2, 3]`), Rust uses repetition operators:

- `$( ... ),*` — Zero or more matches, separated by commas.
- `$( ... ),+` — One or more matches, separated by commas.
- `$( ... )?` — Zero or one match (optional).
- `$(,)?` — Optional trailing comma.

```rust
macro_rules! sum_items {
    ( $( $val:expr ),* $(,)? ) => {
        {
            let mut total = 0;
            $(
                total += $val;
            )*
            total
        }
    };
}
```

During expansion, the block inside `$( ... )*` is repeated for each matched element!

---

## Practical Domain Macros in MiniStore

Let's examine the macros implemented in MiniStore's [`src/macros.rs`](crafting-rust/ministore/src/macros.rs).

### 1. The `product!` Macro: Flexible Construction & Enum Shorthand

The `product!` macro provides multiple compile-time branches:

- A named-parameter branch for self-documenting syntax.
- A positional branch that accepts the bare category identifier (`Electronics` instead of `ProductCategory::Electronics`).
- An overloaded branch that automatically assigns a default `stock = 1` if omitted.
- Automatic conversion of string literals into `String` via `.to_string()`.

```rust
#[macro_export]
macro_rules! product {
    // Branch 1: Named argument style
    (id: $id:expr, sku: $sku:expr, name: $name:expr, category: $cat:ident, price: $price:expr, stock: $stock:expr) => {
        $crate::models::Product::new(
            $id,
            $sku.to_string(),
            $name.to_string(),
            $crate::models::ProductCategory::$cat,
            $price,
            $stock,
        )
    };

    // Branch 2: Positional with category ident shorthand and stock
    ($id:expr, $sku:expr, $name:expr, $cat:ident, $price:expr, $stock:expr) => {
        $crate::models::Product::new(
            $id,
            $sku.to_string(),
            $name.to_string(),
            $crate::models::ProductCategory::$cat,
            $price,
            $stock,
        )
    };

    // Branch 3: Positional with default stock = 1
    ($id:expr, $sku:expr, $name:expr, $cat:ident, $price:expr) => {
        $crate::product!($id, $sku, $name, $cat, $price, 1)
    };
}
```

Notice how Branch 3 delegates recursively to Branch 2 using `$crate::product!(...)`!

### 2. The `catalog!` Macro: Variadic Collections

The `catalog!` macro allows populating a `Catalog` with zero, one, or dozens of products, supporting optional trailing commas:

```rust
#[macro_export]
macro_rules! catalog {
    ( $( $prod:expr ),* $(,)? ) => {
        {
            let mut catalog = $crate::catalog::Catalog::new();
            $(
                catalog.add_product($prod);
            )*
            catalog
        }
    };
}
```

Usage:

```rust
let catalog = catalog![
    product!(1, "SKU-001", "Laptop", Electronics, 120000, 5),
    product!(2, "SKU-002", "Pen", OfficeSupplies, 150), // default stock 1
];
```

### 3. The `cart!` Macro: Multi-Syntax Matching

The `cart!` macro demonstrates supporting multiple distinct calling syntaxes:

1. Concise tuple syntax: `(product_id, quantity, unit_price_cents)`
2. Named syntax: `(item: id, qty: count, price: cents)`

```rust
#[macro_export]
macro_rules! cart {
    // Empty cart
    () => {
        $crate::models::ShoppingCart::new()
    };

    // Tuple style: (product_id, quantity, unit_price_cents)
    ( $( ( $prod_id:expr, $qty:expr, $price:expr ) ),* $(,)? ) => {
        {
            let mut cart = $crate::models::ShoppingCart::new();
            $(
                cart.add_item($prod_id, $qty, $price);
            )*
            cart
        }
    };

    // Named style: (item: $id, qty: $qty, price: $price)
    ( $( ( item: $prod_id:expr, qty: $qty:expr, price: $price:expr ) ),* $(,)? ) => {
        {
            let mut cart = $crate::models::ShoppingCart::new();
            $(
                cart.add_item($prod_id, $qty, $price);
            )*
            cart
        }
    };
}
```

### 4. The `calculate_total!` Macro: Recursive Variadic Accumulation

Macros can also use recursive expansion (dividing arguments into a head and tail):

```rust
#[macro_export]
macro_rules! calculate_total {
    // Base case 1: empty arguments
    () => {
        0u32
    };

    // Base case 2: single argument
    ($head:expr) => {
        $head
    };

    // Inductive step: head + sum of remaining tail
    ($head:expr, $($tail:expr),+ $(,)?) => {
        $head + $crate::calculate_total!($($tail),+)
    };
}
```

When evaluated:

```rust
calculate_total!(100, 200, 300)
// Expands to:
100 + calculate_total!(200, 300)
// Expands to:
100 + (200 + 300) = 600
```

### 5. The `assert_in_stock!` Macro: Domain Assertions

Custom assertion macros allow embedding domain-specific diagnostics directly into assertions:

```rust
#[macro_export]
macro_rules! assert_in_stock {
    ($prod:expr, $required:expr) => {
        assert!(
            $prod.stock >= $required,
            "Stock assertion failed for SKU '{}' ({}): available {}, required {}",
            $prod.sku,
            $prod.name,
            $prod.stock,
            $required
        );
    };
}
```

---

## Macro Hygiene and `$crate`

When a macro is exported using `#[macro_export]`, it can be invoked from other modules, integration tests, or external crates that depend on `ministore`.

If our macro generated code like `Product::new(...)`, the caller would fail to compile unless they explicitly had `use ministore::Product;` in their local scope!

To prevent this, exported macros use **`$crate`**:

- `$crate` expands to the path of the defining crate (e.g., `crate` within the library itself, or `::ministore` when invoked from an external binary or test).
- Using `$crate::models::Product` guarantees that the macro will always locate the correct types, regardless of what the caller has imported.

---

## Testing Your Code

Run the full test suite across the workspace:

```bash
cargo test
```

Key test scenarios in `ministore/src/lib.rs`:

1. **`test_declarative_product_and_catalog_macros`**: Verifies named, positional, and default-stock branches of `product!`, as well as `catalog!` assembly.
2. **`test_declarative_cart_and_variadic_calculation_macros`**: Validates tuple and named syntax in `cart!`, variadic recursion in `calculate_total!`, and `assert_in_stock!`.
3. **Doc-tests in `src/macros.rs`**: All 5 markdown documentation code blocks compile and run as unit tests.

---

## Checkpoint & Summary

With Chapter 25 complete, you have unlocked the power of Rust metaprogramming:

- You understand how declarative macros operate on syntax tokens at compile time.
- You know how to use designators (`$expr`, `$ident`, `$ty`) and repetition patterns (`$( ... ),*`).
- You built practical domain DSLs (`product!`, `catalog!`, `cart!`, `calculate_total!`, and `assert_in_stock!`).
- You understand macro hygiene and the critical role of `$crate`.

In the next chapter, we will explore the final frontier of Rust: **Unsafe Rust**, understanding how Rust interacts with raw memory and why safe abstractions make Rust unique!
