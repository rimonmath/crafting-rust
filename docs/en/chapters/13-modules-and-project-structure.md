# Chapter 13: Modules, Packages, and Project Structure

## What You'll Learn
- The difference between **Packages**, **Crates** (Binary crates vs. Library crates), and **Modules**.
- How Cargo organizes projects with single binaries, standalone libraries, or **dual-target crates** (`src/lib.rs` + `src/main.rs`).
- The **`mod` keyword**: declaring modules inline, in separate files (`foo.rs`), and in subdirectories (`foo/mod.rs`).
- Rust's strict privacy rules: **private by default**, the **`pub`** keyword, struct field privacy, and granular visibility like **`pub(crate)`**.
- Path resolution: navigating the module tree using absolute paths (**`crate::`**), relative paths (**`self::`**, **`super::`**).
- Importing items with **`use`**, nested paths, renaming with **`as`**, and when (or when not) to use glob imports (`*`).
- **Re-exporting with `pub use`**: designing clean, ergonomic public facades so consumers don't have to navigate deep internal folder structures.
- Refactoring MiniStore from a monolithic 700+ line `main.rs` file into a clean, maintainable modular architecture:
  - `src/error.rs`: Central domain error definitions (`StoreError`).
  - `src/models/`: Domain entities split across dedicated submodules (`product.rs`, `customer.rs`, `cart.rs`, `order.rs`).
  - `src/catalog.rs`: Product catalog and inventory management service.
  - `src/checkout.rs`: Transactional order placement and discount calculation logic.
  - `src/lib.rs`: The library crate root, public facade re-exports, and integration unit tests.
  - `src/main.rs`: The executable CLI binary consuming `ministore` as a client crate.

---

## Why Do We Need This?
Up to Chapter 12, our MiniStore application lived entirely inside a single file: `src/main.rs`. 

While having everything in one file is great when learning basic syntax, real-world systems cannot survive in a single file:
1. **Cognitive Overload**: When a single file reaches 800 to 2,000+ lines, finding functions, structs, and implementations becomes painful.
2. **Merge Conflicts**: In a software engineering team, multiple developers editing `main.rs` concurrently will produce constant Git merge conflicts.
3. **Lack of Encapsulation**: When all structs and functions sit in the same file, every function can see and touch every internal field of every struct. There are no boundaries to protect invariants.
4. **Reusability and Testing**: A binary crate (`src/main.rs`) cannot be imported into external integration tests, benchmarks, or third-party crates. Only library crates (`src/lib.rs`) can be reused.

Rust provides a powerful, compiler-enforced module system that lets you divide code into logical units, control visibility with precision, and design clean, intuitive public APIs.

---

## The Rust Module System Hierarchy
To understand how Rust organizes code, think of a Russian nesting doll or an organizational chart. Rust organizes code at three distinct levels:

```
+-----------------------------------------------------------+
|                        Package                            |
|  (Cargo.toml: defines metadata, dependencies, & crates)   |
|                                                           |
|   +-----------------------+   +-----------------------+   |
|   |     Library Crate     |   |     Binary Crate      |   |
|   |      src/lib.rs       |   |      src/main.rs      |   |
|   |                       |   |                       |   |
|   |   +---------------+   |   |   use my_crate::*;    |   |
|   |   |    Modules    |   |   |                       |   |
|   |   |  models/      |   |   |   fn main() { ... }   |   |
|   |   |  checkout.rs  |   |   +-----------------------+   |
|   |   |  catalog.rs   |   |                               |
|   |   +---------------+   |                               |
|   +-----------------------+                               |
+-----------------------------------------------------------+
```

### 1. Packages
A **Package** is a Cargo feature that lets you build, test, and share one or more crates. A package is defined by a `Cargo.toml` file at its root.
- A package **must contain at least one crate** (either a library or a binary).
- A package can contain **at most one library crate** (`src/lib.rs`).
- A package can contain **zero, one, or multiple binary crates** (`src/main.rs`, or multiple files in `src/bin/*.rs`).

### 2. Crates
A **Crate** is the smallest amount of code that the Rust compiler (`rustc`) considers at a time.
There are two varieties of crates:
- **Binary Crate**: An executable program that has a `fn main()` entry point. When compiled, it produces an executable machine binary (e.g., `target/debug/ministore`).
- **Library Crate**: A collection of reusable functionality without a `main()` function. It defines functionality meant to be shared across projects or consumed by binary crates. Its root is `src/lib.rs`.

> [!TIP]
> **The Dual-Crate Pattern**: The most idiomatic pattern for CLI tools and services in Rust is to put **all core business logic in a library crate** (`src/lib.rs`), and keep `src/main.rs` as a thin wrapper that merely parses CLI arguments, calls library functions, and handles exits. This makes all your business logic fully testable and reusable!

### 3. Modules
A **Module** lets you organize code *within* a crate for readability, namespace separation, and privacy control.
- Modules control **privacy**: code inside a module is private from outside code unless explicitly marked with `pub`.
- Modules form a tree hierarchy rooted at the crate root (`src/lib.rs` or `src/main.rs`).

---

## Core Rust Concept: Declaring and Organizing Modules

In Rust, the file system does **not** automatically dictate your module structure (unlike languages like Java or Node.js where creating a file immediately creates an importable module).

In Rust, **you must explicitly declare a module in its parent using the `mod` keyword**.

### How the Compiler Finds Modules
When you write `mod models;` in `src/lib.rs`, the compiler searches for the module code in one of two locations:
1. An inline block:
   ```rust
   mod models {
       // items defined directly here
   }
   ```
2. A separate file with the same name:
   `src/models.rs`
3. A directory with a `mod.rs` file:
   `src/models/mod.rs`

### Module Layout Styles (Modern Rust vs. 2015 Style)
Rust 2018 introduced a cleaner module layout, but both are supported:

| Style | Directory Layout | Notes |
| :--- | :--- | :--- |
| **Directory + `mod.rs`** | `src/models/mod.rs`<br>`src/models/product.rs` | Explicit and clear; `mod.rs` acts as the root of the `models` submodule. |
| **Sibling File + Folder** | `src/models.rs`<br>`src/models/product.rs` | Supported since Rust 2018; avoids having multiple tabs named `mod.rs` open in your IDE. |

In MiniStore, we use the `src/models/mod.rs` structure because it visually and logically encapsulates the `models` folder as a dedicated sub-package.

---

## Privacy and Visibility: Private by Default

In Rust, **everything is private by default**.
- Functions, structs, enums, traits, type aliases, and constants are visible only to the module they are defined in, and to any child modules nested inside it.
- Parent modules **cannot** see private items inside their child modules.
- Child modules **can** see private items inside their ancestor modules.

```rust
mod warehouse {
    // Private function: only code inside `warehouse` can call this.
    fn secret_code() -> u32 {
        42
    }

    // Public function: visible outside `warehouse`.
    pub fn open_doors() {
        // Child can see private items in the same scope:
        let code = secret_code();
        println!("Opening doors with code {}", code);
    }
}

fn main() {
    warehouse::open_doors(); // OK: public
    // warehouse::secret_code(); // COMPILE ERROR: `secret_code` is private!
}
```

### Struct and Enum Visibility Nuance
Notice a very important difference between `struct` and `enum` visibility in Rust:

#### 1. Structs: Fields are Private by Default
Even if a struct is marked `pub`, its fields remain **strictly private** unless each field is explicitly marked `pub`:
```rust
pub struct Product {
    pub id: u64,          // Public: outside modules can read and write this field
    pub title: String,    // Public
    cost_price: u32,      // PRIVATE: only code inside this module can access this!
}
```
If a struct has *even one* private field:
- Outside modules **cannot** construct it using struct literal syntax (`Product { id: 1, title: ..., cost_price: ... }`).
- You **must** provide a public constructor method (like `pub fn new(...) -> Self`)! This guarantees data encapsulation and invariant preservation.

#### 2. Enums: Variants Inherit Enum Visibility
If an enum is marked `pub`, **all of its variants and their payload data are automatically public**:
```rust
pub enum OrderStatus {
    Pending,
    Confirmed { receipt_id: String }, // Variant and receipt_id are both public!
    Cancelled,
}
```
Why? Because an enum represents an exhaustive set of possibilities that callers must be able to match on using `match`. Hiding individual variants would break exhaustive pattern matching.

### Granular Visibility Modifiers
Beyond simple `pub`, Rust offers fine-grained visibility specifiers:
- `pub`: Visible to the entire world (including external crates consuming your library).
- `pub(crate)`: Visible anywhere inside the **current crate**, but completely hidden from external users of the crate.
- `pub(super)`: Visible only to the **parent module**.
- `pub(in path)`: Visible only within a specific ancestor module path.

```rust
pub(crate) fn internal_database_sync() {
    // Visible to any module within ministore crate,
    // but users importing ministore as a dependency cannot see it!
}
```

---

## Paths and the `use` Keyword

To call a function or use a type in Rust, you reference its path. There are two kinds of paths:

### 1. Absolute Paths
An absolute path starts from the crate root using the literal keyword `crate`:
```rust
let product = crate::models::product::Product::new(...);
```
Absolute paths are unambiguous and don't change meaning if you move the calling code into another sub-module.

### 2. Relative Paths
A relative path starts from the current module and uses:
- `self`: Refers to the current module scope (rarely required except in disambiguation).
- `super`: Refers to the **parent module** (like `..` in a file system path).

```rust
// Inside src/models/cart.rs:
use super::product::Product; // Goes up to `models`, then into `product`
```

### Bringing Paths into Scope with `use`
Typing full paths like `crate::models::order::OrderStatus` everywhere is tedious. The `use` keyword creates a local alias or shortcut:

```rust
use crate::models::order::OrderStatus;

let status = OrderStatus::Pending; // Clean and readable!
```

#### Nested Paths and Renaming
You can group multiple imports from the same module using curly braces:
```rust
// Instead of multiple lines:
use crate::models::product::{Product, ProductCategory};
use crate::models::order::{Order, OrderStatus, PaymentMethod};
```

You can resolve naming conflicts using the `as` keyword:
```rust
use std::fmt::Result as FmtResult;
use std::io::Result as IoResult;
```

---

## Re-Exporting with `pub use`: The Facade Pattern

Consider how our files are structured inside MiniStore:
- Struct `Product` is located in `crate::models::product::Product`
- Struct `ShoppingCart` is located in `crate::models::cart::ShoppingCart`
- Struct `Catalog` is located in `crate::catalog::Catalog`
- Function `checkout` is located in `crate::checkout::checkout`

If an external developer uses our crate, requiring them to know that `Product` is inside `models::product` while `Catalog` is in `catalog` is messy and leaks internal implementation details.

### What is `pub use`?
By combining `pub` and `use`, you **re-export** an item. You take an item defined deep in an internal private module and expose it at a higher, more convenient level:

```rust
// Inside src/models/mod.rs:
pub mod product;
pub mod customer;
pub mod cart;
pub mod order;

// Re-export key types directly under `models`:
pub use product::{Product, ProductCategory};
pub use customer::{Customer, OrderId};
pub use cart::{CartItem, ShoppingCart};
pub use order::{Coupon, Order, OrderStatus, PaymentMethod};
```

And in `src/lib.rs`:
```rust
// Inside src/lib.rs (crate root):
pub mod catalog;
pub mod checkout;
pub mod error;
pub mod models;

// Create a flat, convenient top-level facade:
pub use catalog::Catalog;
pub use checkout::{calculate_discount, checkout};
pub use error::StoreError;
pub use models::{
    CartItem, Coupon, Customer, Order, OrderId, OrderStatus, PaymentMethod,
    Product, ProductCategory, ShoppingCart,
};
```

Now, anyone consuming our crate (including our own `src/main.rs`) can write:
```rust
use ministore::{Catalog, Customer, Order, Product, checkout};
```
This is the **Facade Pattern**. Internal refactorings (such as moving `Product` from `models/product.rs` to `models/item.rs`) will not break any external consumer code!

---

## Mental Bridge: Comparison With Other Languages

| Concept | Rust | Python | JavaScript / TypeScript | Go | Java / C# |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Compilation Unit** | Crate (`lib.rs` / `main.rs`) | Script or Module (`.py`) | Package (`package.json`) | Package directory | Project / Assembly |
| **File to Module** | Explicit `mod foo;` declaration required | Automatic (`import foo`) | Automatic (`import foo from './foo'`) | Automatic (all `.go` in folder share package) | Automatic (`package com.example`) |
| **Default Privacy** | **Private** | Public (convention `_prefix` for private) | Public (export needed, `#` for private fields) | Private if lowercase; Public if Capitalized | Package-private (Java) or Internal (C#) |
| **Struct Field Privacy** | Private by default, must specify `pub` | Public by convention | Public (or private class fields) | Capitalization determines field visibility | Private / protected / public per field |
| **Re-exporting** | `pub use foo::Bar;` | `from foo import Bar` in `__init__.py` | `export { Bar } from './foo'` | Aliased exports in root package | Re-declaring or inheritance |

---

## Step-by-Step Code Examples

### Example 1: Basic Module Declaration and Hierarchy
```rust
mod storage {
    pub struct Box {
        pub capacity: u32,
        secret_label: String, // Private field
    }

    impl Box {
        pub fn new(capacity: u32, secret_label: String) -> Self {
            Self { capacity, secret_label }
        }

        pub fn reveal_label(&self) -> &str {
            &self.secret_label
        }
    }
}

fn main() {
    let b = storage::Box::new(50, String::from("Fragile Glass"));
    println!("Capacity: {}, Label: {}", b.capacity, b.reveal_label());
    
    // b.secret_label; // COMPILE ERROR: secret_label is private!
}
```

### Example 2: The `super` Keyword for Upward Traversal
```rust
mod network {
    fn ping() -> &'static str {
        "pong"
    }

    pub mod client {
        pub fn test_connection() {
            // Traverse up to `network` module using `super`
            let response = super::ping();
            println!("Connection test: {}", response);
        }
    }
}

fn main() {
    network::client::test_connection();
}
```

---

## MiniStore Architecture & Modular Refactoring

Let us inspect the new file tree of `ministore/`:

```
ministore/
├── Cargo.toml
└── src/
    ├── error.rs            # Central error enum (StoreError)
    ├── catalog.rs          # Catalog struct & stock query methods
    ├── checkout.rs         # Checkout transaction & discount logic
    ├── models/             # Domain models submodule
    │   ├── mod.rs          # Submodule declarations & re-exports
    │   ├── product.rs      # Product & ProductCategory
    │   ├── customer.rs     # Customer & OrderId
    │   ├── cart.rs         # CartItem & ShoppingCart
    │   └── order.rs        # Order, OrderStatus, PaymentMethod, Coupon
    ├── lib.rs              # Library root: facade exports & integration tests
    └── main.rs             # Binary entry point (CLI application)
```

Let's examine each file and understand how the module system binds them together.

---

### 1. `src/error.rs`
The error module defines `StoreError`. Notice that all variants and methods needed across the crate are marked `pub`.

```rust
/// Comprehensive domain error enum for MiniStore operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    ProductNotFound { sku: String },
    InsufficientStock { sku: String, requested: u32, available: u32 },
    EmptyCart,
    InvalidCoupon { code: String, reason: String },
    InvalidStateTransition { action: String, current_state: String },
    OrderAlreadyExists { order_id: u64 },
}

impl StoreError {
    pub fn message(&self) -> String {
        match self {
            StoreError::ProductNotFound { sku } => {
                format!("Product with SKU '{}' does not exist in the catalog.", sku)
            }
            StoreError::InsufficientStock { sku, requested, available } => {
                format!(
                    "Insufficient stock for SKU '{}': requested {}, but only {} available.",
                    sku, requested, available
                )
            }
            StoreError::EmptyCart => {
                String::from("Cannot checkout with an empty shopping cart.")
            }
            StoreError::InvalidCoupon { code, reason } => {
                format!("Invalid coupon code '{}': {}.", code, reason)
            }
            StoreError::InvalidStateTransition { action, current_state } => {
                format!(
                    "Cannot perform action '{}' while order is in '{}' state.",
                    action, current_state
                )
            }
            StoreError::OrderAlreadyExists { order_id } => {
                format!("An order with ID #{} has already been processed.", order_id)
            }
        }
    }
}
```

---

### 2. `src/models/product.rs`
Contains `ProductCategory` and `Product`. It needs `StoreError` when reducing stock, so it imports it using `use crate::error::StoreError;`.

```rust
use crate::error::StoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductCategory {
    Electronics,
    Clothing,
    Books,
    Home,
}

impl ProductCategory {
    pub fn tax_rate(&self) -> f64 {
        match self {
            ProductCategory::Electronics => 0.15,
            ProductCategory::Clothing => 0.05,
            ProductCategory::Books => 0.00,
            ProductCategory::Home => 0.08,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProductCategory::Electronics => "Consumer Electronics",
            ProductCategory::Clothing => "Apparel & Garments",
            ProductCategory::Books => "Books & Educational Material",
            ProductCategory::Home => "Home & Living Essentials",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub sku: String,
    pub title: String,
    pub category: ProductCategory,
    pub price_cents: u32,
    pub stock: u32,
}

impl Product {
    pub fn new(
        id: u64,
        sku: String,
        title: String,
        category: ProductCategory,
        price_cents: u32,
        stock: u32,
    ) -> Self {
        Self {
            id,
            sku,
            title,
            category,
            price_cents,
            stock,
        }
    }

    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    pub fn reduce_stock(&mut self, quantity: u32) -> Result<(), StoreError> {
        if self.stock >= quantity {
            self.stock -= quantity;
            Ok(())
        } else {
            Err(StoreError::InsufficientStock {
                sku: self.sku.clone(),
                requested: quantity,
                available: self.stock,
            })
        }
    }

    pub fn restore_stock(&mut self, quantity: u32) {
        self.stock += quantity;
    }
}
```

---

### 3. `src/models/customer.rs`
Defines `Customer` and `OrderId`.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderId(pub u64);

#[derive(Debug, Clone, PartialEq)]
pub struct Customer {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
}

impl Customer {
    pub fn new(id: u64, name: String, email: String, phone: Option<String>) -> Self {
        Self { id, name, email, phone }
    }

    pub fn contact_info(&self) -> String {
        match &self.phone {
            Some(phone) => format!("{} ({})", self.name, phone),
            None => format!("{} <{}>", self.name, self.email),
        }
    }
}
```

---

### 4. `src/models/cart.rs`
Implements the shopping cart.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct CartItem {
    pub product_id: u64,
    pub quantity: u32,
    pub unit_price_cents: u32,
}

impl CartItem {
    pub fn new(product_id: u64, quantity: u32, unit_price_cents: u32) -> Self {
        Self { product_id, quantity, unit_price_cents }
    }

    pub fn line_total_cents(&self) -> u32 {
        self.quantity * self.unit_price_cents
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShoppingCart {
    pub items: Vec<CartItem>,
}

impl ShoppingCart {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(&mut self, product_id: u64, quantity: u32, unit_price_cents: u32) {
        if let Some(existing) = self.items.iter_mut().find(|item| item.product_id == product_id) {
            existing.quantity += quantity;
        } else {
            self.items.push(CartItem::new(product_id, quantity, unit_price_cents));
        }
    }

    pub fn remove_item(&mut self, product_id: u64) -> Option<CartItem> {
        if let Some(pos) = self.items.iter().position(|item| item.product_id == product_id) {
            Some(self.items.remove(pos))
        } else {
            None
        }
    }

    pub fn subtotal_cents(&self) -> u32 {
        self.items.iter().map(|item| item.line_total_cents()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}
```

---

### 5. `src/models/order.rs`
Contains `PaymentMethod`, `OrderStatus`, `Coupon`, and `Order`. It imports `CartItem` and `Customer` from its sibling submodules using `super::` or `crate::models::` paths.

```rust
use crate::error::StoreError;
use super::cart::CartItem;
use super::customer::OrderId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentMethod {
    CreditCard,
    CashOnDelivery,
    MobileBanking { provider_code: u8 },
}

impl PaymentMethod {
    pub fn processing_fee_cents(&self) -> u32 {
        match self {
            PaymentMethod::CreditCard => 150,
            PaymentMethod::CashOnDelivery => 0,
            PaymentMethod::MobileBanking { .. } => 50,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            PaymentMethod::CreditCard => "Credit Card",
            PaymentMethod::CashOnDelivery => "Cash on Delivery",
            PaymentMethod::MobileBanking { provider_code } => match provider_code {
                1 => "bKash",
                2 => "Nagad",
                3 => "Rocket",
                _ => "Other Mobile Banking",
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Confirmed { receipt_id: String },
    Shipped { tracking_number: String },
    Delivered,
    Cancelled { reason: String },
}

impl OrderStatus {
    pub fn display_status(&self) -> String {
        match self {
            OrderStatus::Pending => String::from("Pending Verification"),
            OrderStatus::Confirmed { receipt_id } => {
                format!("Confirmed (Receipt: {})", receipt_id)
            }
            OrderStatus::Shipped { tracking_number } => {
                format!("Shipped (Tracking: {})", tracking_number)
            }
            OrderStatus::Delivered => String::from("Delivered to Customer"),
            OrderStatus::Cancelled { reason } => format!("Cancelled: {}", reason),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Coupon {
    pub code: String,
    pub discount_percent: u8,
}

impl Coupon {
    pub fn new(code: String, discount_percent: u8) -> Self {
        Self { code, discount_percent }
    }

    pub fn validate(&self) -> Result<(), StoreError> {
        if self.discount_percent == 0 || self.discount_percent > 100 {
            Err(StoreError::InvalidCoupon {
                code: self.code.clone(),
                reason: format!("discount percentage {} must be between 1 and 100", self.discount_percent),
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub order_id: OrderId,
    pub customer_id: u64,
    pub items: Vec<CartItem>,
    pub discount_cents: u32,
    pub payment_method: PaymentMethod,
    pub status: OrderStatus,
}

impl Order {
    pub fn new(
        order_id: OrderId,
        customer_id: u64,
        items: Vec<CartItem>,
        discount_cents: u32,
        payment_method: PaymentMethod,
    ) -> Self {
        Self {
            order_id,
            customer_id,
            items,
            discount_cents,
            payment_method,
            status: OrderStatus::Pending,
        }
    }

    pub fn subtotal_cents(&self) -> u32 {
        self.items.iter().map(|item| item.line_total_cents()).sum()
    }

    pub fn total_cents(&self) -> u32 {
        let subtotal = self.subtotal_cents();
        let discounted = subtotal.saturating_sub(self.discount_cents);
        discounted + self.payment_method.processing_fee_cents()
    }

    pub fn confirm(&mut self, receipt_id: String) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Pending => {
                self.status = OrderStatus::Confirmed { receipt_id };
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                action: String::from("confirm"),
                current_state: self.status.display_status(),
            }),
        }
    }

    pub fn ship(&mut self, tracking_number: String) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Confirmed { .. } => {
                self.status = OrderStatus::Shipped { tracking_number };
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                action: String::from("ship"),
                current_state: self.status.display_status(),
            }),
        }
    }

    pub fn mark_delivered(&mut self) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Shipped { .. } => {
                self.status = OrderStatus::Delivered;
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                action: String::from("deliver"),
                current_state: self.status.display_status(),
            }),
        }
    }

    pub fn cancel(&mut self, reason: String) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Pending | OrderStatus::Confirmed { .. } => {
                self.status = OrderStatus::Cancelled { reason };
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                action: String::from("cancel"),
                current_state: self.status.display_status(),
            }),
        }
    }
}
```

---

### 6. `src/models/mod.rs`
The entry point of the `models` submodule. It declares the submodules and re-exports their types.

```rust
pub mod cart;
pub mod customer;
pub mod order;
pub mod product;

pub use cart::{CartItem, ShoppingCart};
pub use customer::{Customer, OrderId};
pub use order::{Coupon, Order, OrderStatus, PaymentMethod};
pub use product::{Product, ProductCategory};
```

---

### 7. `src/catalog.rs`
Manages product inventory.

```rust
use std::collections::HashMap;
use crate::models::Product;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Catalog {
    products: HashMap<String, Product>,
}

impl Catalog {
    pub fn new() -> Self {
        Self {
            products: HashMap::new(),
        }
    }

    pub fn add_product(&mut self, product: Product) {
        self.products.insert(product.sku.clone(), product);
    }

    pub fn find_by_sku(&self, sku: &str) -> Option<&Product> {
        self.products.get(sku)
    }

    pub fn find_by_sku_mut(&mut self, sku: &str) -> Option<&mut Product> {
        self.products.get_mut(sku)
    }

    pub fn find_by_id(&self, id: u64) -> Option<&Product> {
        self.products.values().find(|product| product.id == id)
    }

    pub fn find_by_id_mut(&mut self, id: u64) -> Option<&mut Product> {
        self.products.values_mut().find(|product| product.id == id)
    }

    pub fn product_price(&self, sku: &str) -> Option<u32> {
        self.find_by_sku(sku).map(|p| p.price_cents)
    }

    pub fn is_product_in_stock(&self, sku: &str) -> bool {
        self.find_by_sku(sku)
            .map(|p| p.is_in_stock())
            .unwrap_or(false)
    }

    pub fn len(&self) -> usize {
        self.products.len()
    }

    pub fn is_empty(&self) -> bool {
        self.products.is_empty()
    }
}
```

---

### 8. `src/checkout.rs`
Coordinates order placement and discount calculation.

```rust
use std::collections::HashMap;
use crate::catalog::Catalog;
use crate::error::StoreError;
use crate::models::{
    CartItem, Coupon, Customer, Order, OrderId, PaymentMethod, ProductCategory, ShoppingCart,
};

pub fn calculate_discount(subtotal_cents: u32, coupon: Option<&Coupon>) -> u32 {
    match coupon {
        Some(c) => (subtotal_cents * c.discount_percent as u32) / 100,
        None => 0,
    }
}

pub fn count_products_by_department(
    items: &[CartItem],
    catalog: &Catalog,
) -> HashMap<ProductCategory, u32> {
    let mut counts: HashMap<ProductCategory, u32> = HashMap::new();

    for item in items {
        if let Some(product) = catalog.find_by_id(item.product_id) {
            let counter = counts.entry(product.category).or_insert(0);
            *counter += item.quantity;
        }
    }

    counts
}

pub fn order_dispatch_advisory(status: &crate::models::OrderStatus) -> &'static str {
    match status {
        crate::models::OrderStatus::Pending => "Await payment confirmation before dispatch.",
        crate::models::OrderStatus::Confirmed { .. } => "Approved for immediate packaging and carrier assignment.",
        crate::models::OrderStatus::Shipped { .. } => "Carrier transit in progress; track coordinates.",
        crate::models::OrderStatus::Delivered => "Package received by customer; archive manifest.",
        crate::models::OrderStatus::Cancelled { .. } => "Halt fulfillment; reverse reserved warehouse inventory.",
    }
}

pub fn checkout(
    order_id: OrderId,
    customer: &Customer,
    cart: &mut ShoppingCart,
    catalog: &mut Catalog,
    payment_method: PaymentMethod,
    mut coupon: Option<Coupon>,
) -> Result<Order, StoreError> {
    if cart.is_empty() {
        return Err(StoreError::EmptyCart);
    }

    if let Some(ref c) = coupon {
        c.validate()?;
    }

    // 1. Validate all stock availability before mutating
    for item in &cart.items {
        let product = catalog
            .find_by_id(item.product_id)
            .ok_or_else(|| StoreError::ProductNotFound {
                sku: format!("ID-{}", item.product_id),
            })?;

        if product.stock < item.quantity {
            return Err(StoreError::InsufficientStock {
                sku: product.sku.clone(),
                requested: item.quantity,
                available: product.stock,
            });
        }
    }

    // 2. Reduce stock in catalog
    for item in &cart.items {
        let product = catalog.find_by_id_mut(item.product_id).unwrap();
        product.reduce_stock(item.quantity)?;
    }

    // 3. Calculate discounts & create order
    let subtotal = cart.subtotal_cents();
    let discount = calculate_discount(subtotal, coupon.as_ref());
    let active_coupon = coupon.take();
    let _ = active_coupon;

    let items = std::mem::take(&mut cart.items);

    let order = Order::new(
        order_id,
        customer.id,
        items,
        discount,
        payment_method,
    );

    Ok(order)
}
```

---

### 9. `src/lib.rs`
The crate root of our library crate. It declares the modules, creates the public facade using `pub use`, and hosts unit tests.

```rust
pub mod catalog;
pub mod checkout;
pub mod error;
pub mod models;

// Public facade re-exports
pub use catalog::Catalog;
pub use checkout::{
    calculate_discount, checkout, count_products_by_department, order_dispatch_advisory,
};
pub use error::StoreError;
pub use models::{
    CartItem, Coupon, Customer, Order, OrderId, OrderStatus, PaymentMethod, Product,
    ProductCategory, ShoppingCart,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_category_tax_rates() {
        assert_eq!(ProductCategory::Electronics.tax_rate(), 0.15);
        assert_eq!(ProductCategory::Clothing.tax_rate(), 0.05);
        assert_eq!(ProductCategory::Books.tax_rate(), 0.00);
        assert_eq!(ProductCategory::Home.tax_rate(), 0.08);
    }

    #[test]
    fn test_payment_method_fees_and_descriptions() {
        let cc = PaymentMethod::CreditCard;
        let cod = PaymentMethod::CashOnDelivery;
        let bkash = PaymentMethod::MobileBanking { provider_code: 1 };

        assert_eq!(cc.processing_fee_cents(), 150);
        assert_eq!(cod.processing_fee_cents(), 0);
        assert_eq!(bkash.processing_fee_cents(), 50);

        assert_eq!(cc.display_name(), "Credit Card");
        assert_eq!(bkash.display_name(), "bKash");
    }

    #[test]
    fn test_order_status_valid_lifecycle() {
        let mut order = Order::new(
            OrderId(100),
            1,
            vec![CartItem::new(10, 1, 5000)],
            0,
            PaymentMethod::CashOnDelivery,
        );

        assert_eq!(order.status, OrderStatus::Pending);

        assert!(order.confirm(String::from("REC-100")).is_ok());
        assert_eq!(
            order.status,
            OrderStatus::Confirmed { receipt_id: String::from("REC-100") }
        );

        assert!(order.ship(String::from("TRK-XYZ-99")).is_ok());
        assert_eq!(
            order.status,
            OrderStatus::Shipped { tracking_number: String::from("TRK-XYZ-99") }
        );

        assert!(order.mark_delivered().is_ok());
        assert_eq!(order.status, OrderStatus::Delivered);
    }

    #[test]
    fn test_order_cancellation_prevention() {
        let mut order = Order::new(
            OrderId(200),
            1,
            vec![],
            0,
            PaymentMethod::CreditCard,
        );
        order.confirm(String::from("REC-200")).unwrap();
        order.ship(String::from("TRK-200")).unwrap();

        let cancel_result = order.cancel(String::from("Customer changed mind"));
        assert!(cancel_result.is_err());
        match cancel_result {
            Err(StoreError::InvalidStateTransition { action, current_state }) => {
                assert_eq!(action, "cancel");
                assert!(current_state.contains("Shipped"));
            }
            _ => panic!("Expected InvalidStateTransition error"),
        }
    }

    #[test]
    fn test_coupon_discount_and_take() {
        let mut coupon = Some(Coupon::new(String::from("SAVE15"), 15));
        let discount = calculate_discount(10000, coupon.as_ref());
        assert_eq!(discount, 1500);

        let taken = coupon.take();
        assert!(taken.is_some());
        assert!(coupon.is_none());
    }

    #[test]
    fn test_coupon_validation_error() {
        let bad_coupon = Coupon::new(String::from("BAD"), 150);
        let result = bad_coupon.validate();
        assert!(result.is_err());
        match result {
            Err(StoreError::InvalidCoupon { code, reason }) => {
                assert_eq!(code, "BAD");
                assert!(reason.contains("between 1 and 100"));
            }
            _ => panic!("Expected InvalidCoupon error"),
        }
    }

    #[test]
    fn test_customer_optional_phone() {
        let alice = Customer::new(
            1,
            String::from("Alice"),
            String::from("alice@example.com"),
            Some(String::from("+1-555-0100")),
        );
        let bob = Customer::new(
            2,
            String::from("Bob"),
            String::from("bob@example.com"),
            None,
        );

        assert_eq!(alice.contact_info(), "Alice (+1-555-0100)");
        assert_eq!(bob.contact_info(), "Bob <bob@example.com>");
    }

    #[test]
    fn test_catalog_option_lookups() {
        let mut catalog = Catalog::new();
        let product = Product::new(
            42,
            String::from("SKU-BOOK"),
            String::from("Rust Book"),
            ProductCategory::Books,
            3500,
            10,
        );
        catalog.add_product(product);

        assert!(catalog.find_by_sku("SKU-BOOK").is_some());
        assert!(catalog.find_by_sku("NON-EXISTENT").is_none());
        assert_eq!(catalog.product_price("SKU-BOOK"), Some(3500));
        assert_eq!(catalog.product_price("NON-EXISTENT"), None);
        assert!(catalog.is_product_in_stock("SKU-BOOK"));
        assert!(!catalog.is_product_in_stock("NON-EXISTENT"));
    }

    #[test]
    fn test_cart_item_option_lookup() {
        let mut cart = ShoppingCart::new();
        cart.add_item(10, 2, 1500);
        cart.add_item(20, 1, 3000);

        let removed = cart.remove_item(10);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().product_id, 10);

        let non_existent = cart.remove_item(99);
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_product_stock_reduction_error() {
        let mut product = Product::new(
            1,
            String::from("SKU-1"),
            String::from("Item"),
            ProductCategory::Electronics,
            1000,
            5,
        );
        assert!(product.reduce_stock(3).is_ok());
        assert_eq!(product.stock, 2);

        let err = product.reduce_stock(5);
        assert!(err.is_err());
        assert_eq!(
            err,
            Err(StoreError::InsufficientStock {
                sku: String::from("SKU-1"),
                requested: 5,
                available: 2,
            })
        );
    }

    #[test]
    fn test_checkout_error_propagation_and_success() {
        let mut catalog = Catalog::new();
        catalog.add_product(Product::new(
            1,
            String::from("SKU-LAPTOP"),
            String::from("Laptop"),
            ProductCategory::Electronics,
            100000,
            2,
        ));

        let customer = Customer::new(
            1,
            String::from("Carol"),
            String::from("carol@example.com"),
            None,
        );

        let mut empty_cart = ShoppingCart::new();
        let empty_checkout = checkout(
            OrderId(1),
            &customer,
            &mut empty_cart,
            &mut catalog,
            PaymentMethod::CashOnDelivery,
            None,
        );
        assert_eq!(empty_checkout, Err(StoreError::EmptyCart));

        let mut cart = ShoppingCart::new();
        cart.add_item(1, 1, 100000);
        let coupon = Some(Coupon::new(String::from("SAVE10"), 10));

        let order_res = checkout(
            OrderId(2),
            &customer,
            &mut cart,
            &mut catalog,
            PaymentMethod::CreditCard,
            coupon,
        );

        assert!(order_res.is_ok());
        let order = order_res.unwrap();
        assert_eq!(order.subtotal_cents(), 100000);
        assert_eq!(order.discount_cents, 10000);
        assert_eq!(order.total_cents(), 90000 + 150);
        assert_eq!(catalog.find_by_sku("SKU-LAPTOP").unwrap().stock, 1);
        assert!(cart.is_empty());
    }

    #[test]
    fn test_order_total_with_payment_fee() {
        let order = Order::new(
            OrderId(50),
            1,
            vec![CartItem::new(1, 2, 2000)],
            500,
            PaymentMethod::MobileBanking { provider_code: 1 },
        );
        assert_eq!(order.total_cents(), 3500 + 50);
    }
}
```

---

### 10. `src/main.rs`
The binary entry point. Notice how clean and readable it is because it simply consumes `ministore` like any other crate!

```rust
use ministore::{
    checkout, Catalog, Coupon, Customer, OrderId, PaymentMethod, Product, ProductCategory,
    ShoppingCart,
};

fn main() {
    println!("=== MiniStore: Modules & Clean Project Architecture (Part II) ===\n");

    // 1. Initializing Catalog with Products
    let mut catalog = Catalog::new();
    let keyboard = Product::new(
        101,
        String::from("TECH-KEY-001"),
        String::from("Tenkeyless Mechanical Keyboard"),
        ProductCategory::Electronics,
        12000,
        5,
    );
    let mouse = Product::new(
        102,
        String::from("TECH-MOU-002"),
        String::from("Ergonomic Wireless Mouse"),
        ProductCategory::Electronics,
        4500,
        10,
    );

    catalog.add_product(keyboard);
    catalog.add_product(mouse);
    println!("1. Catalog initialized with {} products.", catalog.len());

    // 2. Customer Profile setup
    let customer = Customer::new(
        1,
        String::from("Margaret Hamilton"),
        String::from("margaret@apollo.org"),
        Some(String::from("+1-555-0199")),
    );
    println!("2. Customer: {}", customer.contact_info());

    // 3. Shopping Cart setup
    let mut cart = ShoppingCart::new();
    cart.add_item(101, 1, 12000); // 1 keyboard
    cart.add_item(102, 2, 4500);  // 2 mice

    let coupon = Coupon::new(String::from("LAUNCH20"), 20);

    // 4. Processing Checkout Transaction
    println!("\n3. Processing checkout through modular services...");
    match checkout(
        OrderId(901),
        &customer,
        &mut cart,
        &mut catalog,
        PaymentMethod::CreditCard,
        Some(coupon),
    ) {
        Ok(mut order) => {
            println!(
                "   Checkout Order #{} created successfully!",
                order.order_id.0
            );
            println!(
                "   Subtotal: ${:.2} | Total: ${:.2}",
                order.subtotal_cents() as f64 / 100.0,
                order.total_cents() as f64 / 100.0
            );

            // 5. Order State Machine Transitions
            println!("\n4. Order Lifecycle Transitions:");
            order.confirm(String::from("REC-901-HAMILTON")).unwrap();
            println!("   Order confirmed: {}", order.status.display_status());

            order.ship(String::from("TRK-FEDEX-77189")).unwrap();
            println!("   Order shipped:   {}", order.status.display_status());

            // Attempting illegal state transition
            if let Err(err) = order.cancel(String::from("Customer changed mind")) {
                println!("   Cancellation prevented -> {}", err.message());
            }

            order.mark_delivered().unwrap();
            println!(
                "   Final Lifecycle State: {}",
                order.status.display_status()
            );
        }
        Err(err) => println!("   Checkout failed: {}", err.message()),
    }
}
```

---

## Common Compiler Errors & How to Fix Them

### 1. `error[E0583]: file not found for module`
**The Mistake**:
```rust
// src/lib.rs
mod analytics;
```
**Compiler Output**:
```text
error[E0583]: file not found for module `analytics`
 --> src/lib.rs:5:1
  |
5 | mod analytics;
  | ^^^^^^^^^^^^^^
  |
  = help: to create the module `analytics`, create file "src/analytics.rs" or "src/analytics/mod.rs"
```
**The Fix**:
Remember: in Rust, writing `mod analytics;` tells the compiler to search for `src/analytics.rs` or `src/analytics/mod.rs`. You must create the corresponding file or folder!

---

### 2. `error[E0603]: struct is private` / `field is private`
**The Mistake**:
```rust
mod models {
    pub struct Product {
        price_cents: u32, // Not marked pub!
    }
}

fn main() {
    let p = models::Product { price_cents: 1000 };
}
```
**Compiler Output**:
```text
error[E0616]: field `price_cents` of struct `Product` is private
```
**The Fix**:
Mark the field `pub price_cents: u32;` or provide a constructor `pub fn new(...) -> Self` in the same module.

---

### 3. `error[E0432]: unresolved import`
**The Mistake**:
```rust
// In src/models/order.rs
use crate::cart::CartItem; // WRONG: cart is inside models!
```
**Compiler Output**:
```text
error[E0432]: unresolved import `crate::cart`
 --> src/models/order.rs:2:5
  |
2 | use crate::cart::CartItem;
  |     ^^^^^^^^^^^ could not find `cart` in the crate root
```
**The Fix**:
Use the correct path: `use crate::models::cart::CartItem;` or relative path `use super::cart::CartItem;`.

---

## Idiomatic Rust Best Practices

1. **Favor `src/lib.rs` + `src/main.rs`**: Put all logic in the library crate. Keep `main.rs` as tiny as possible. This makes integration testing seamless.
2. **Use `pub use` for Facades**: Do not expose internal module nesting like `crate::internal::db::connection::Pool`. Re-export key types at the top level or module level (`pub use internal::db::connection::Pool;`).
3. **Keep Struct Fields Private When Invariants Must Be Protected**: If a field has validation rules (like `stock` which must not drop below zero, or `discount_percent` which cannot exceed 100), keep the field private and expose getter and mutator methods.
4. **Group Sibling Imports**: Use `use crate::models::{Product, Customer, Order};` rather than individual import lines.
5. **Avoid Glob Imports (`*`) in Application Code**: Glob imports clutter your local scope and create naming collisions. Only use `*` inside test modules (`use super::*;`) or when importing a well-curated prelude.

---

## Hands-On Exercises

### Exercise 1: Add a New `ShippingAddress` Model
1. In `src/models/`, create a new file `address.rs`.
2. Define a struct `ShippingAddress` with fields: `street: String`, `city: String`, `postal_code: String`, and `country: String`.
3. In `src/models/mod.rs`, declare `pub mod address;` and re-export `ShippingAddress`.
4. Update `Order` to include an optional `shipping_address: Option<ShippingAddress>`.
5. Verify with `cargo test` and `cargo check`.

### Exercise 2: Create a Dedicated `Pricing` Module
1. Create `src/pricing.rs`.
2. Move `calculate_discount` from `checkout.rs` into `pricing.rs`.
3. Add a new function `calculate_sales_tax(subtotal: u32, category: ProductCategory) -> u32` using each category's tax rate.
4. Re-export the functions from `src/lib.rs`.

---

## Checkpoint

Run the compiler checks and unit tests:
```bash
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
cargo run
```

Expected test output:
```text
running 12 tests
test tests::test_cart_item_option_lookup ... ok
test tests::test_catalog_option_lookups ... ok
test tests::test_checkout_error_propagation_and_success ... ok
test tests::test_coupon_discount_and_take ... ok
test tests::test_coupon_validation_error ... ok
test tests::test_customer_optional_phone ... ok
test tests::test_order_cancellation_prevention ... ok
test tests::test_order_status_valid_lifecycle ... ok
test tests::test_order_total_with_payment_fee ... ok
test tests::test_payment_method_fees_and_descriptions ... ok
test tests::test_product_category_tax_rates ... ok
test tests::test_product_stock_reduction_error ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Expected binary output (`cargo run`):
```text
=== MiniStore: Modules & Clean Project Architecture (Part II) ===

1. Catalog initialized with 2 products.
2. Customer: Margaret Hamilton (+1-555-0199)

3. Processing checkout through modular services...
   Checkout Order #901 created successfully!
   Subtotal: $210.00 | Total: $148.50

4. Order Lifecycle Transitions:
   Order confirmed: Confirmed (Receipt: REC-901-HAMILTON)
   Order shipped:   Shipped (Tracking: TRK-FEDEX-77189)
   Cancellation prevented -> Cannot perform action 'cancel' while order is in 'Shipped (Tracking: TRK-FEDEX-77189)' state
   Final Lifecycle State: Delivered to Customer
```

---

## What We Learned
- How Cargo packages hold binary crates and library crates.
- How the `mod` keyword declares submodules and how Rust maps modules to files and directories.
- The principle of "private by default" and how `pub`, `pub(crate)`, and `pub(super)` enforce encapsulation.
- How struct field privacy enforces invariant validation and encapsulation.
- How to navigate the module tree using `crate::` and `super::`.
- How the Facade pattern with `pub use` creates clean, user-friendly APIs while keeping internal organization modular.
- Successfully completing **Part II: Modeling Business Logic** with a production-grade, modular MiniStore codebase!

---

## What's Next
Congratulations! You have officially completed **Part II: Modeling Business Logic**!

In **Part III: Advanced Idiomatic Rust**, we kick off with **Chapter 14: Generics**! We will learn how to write flexible, reusable algorithms and data structures without sacrificing type safety or zero-cost performance!
