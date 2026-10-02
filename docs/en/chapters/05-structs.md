# Chapter 5: Structs: Modeling Real Things

## What You'll Learn
- How to define custom composite data types using **classic C-style structs** with named fields.
- Instantiation techniques: field initialization shorthand and **struct update syntax** (`..base`).
- Enforcing domain type safety using **Tuple Structs** (the Newtype pattern).
- Defining marker types with **Unit-like Structs**.
- Attaching behavior to data using `impl` blocks: distinguishing **methods** (`&self`, `&mut self`) from **associated functions** (`Self::new`).
- The role of compiler attribute macros: `#[derive(Debug, Clone, PartialEq)]`.
- Replacing raw tuples in MiniStore with robust, domain-driven entities: `Product`, `Customer`, `CartItem`, and `OrderSummary`.

---

## Why Do We Need This?
In Chapters 3 and 4, we tracked items in MiniStore using raw tuples:

```rust
let item: (&str, u32, u32) = ("Mechanical Keyboard", 8999, 2);
let total = item.1 * item.2; // What was item.1 again? Price? Quantity?
```

While tuples are lightweight, they quickly become unmaintainable in production systems:
1. **Zero Semantic Meaning**: Tuple elements are accessed by positional indices (`.0`, `.1`, `.2`). Swapping two integers (e.g., price cents vs stock quantity) produces silent business logic bugs without any compiler errors.
2. **Scattered Invariants**: Price validation, stock reduction, and discount rules are scattered throughout arbitrary helper functions rather than belonging directly to the domain concepts they operate on.
3. **Lack of Encapsulation**: Any piece of code can arbitrarily construct invalid tuples without constraint.

Structs solve this by giving semantic names to data fields and encapsulating behavior within `impl` blocks.

---

## The Problem
When modeling real e-commerce workflows, three software design issues inevitably arise:

1. **Primitive Obsession**: Passing plain integers (`u64`) for product IDs, customer IDs, and order IDs. An engineer can easily pass `customer_id` into a function expecting `order_id`, causing cross-customer data leakage.
2. **Uncontrolled Mutation**: Modifying inventory stock or pricing outside authorized domain boundaries.
3. **Duplicated Entity Definitions**: Cloning or configuring product variations (e.g., standard edition vs RGB edition) requires re-specifying identical fields over and over.

Rust provides three struct variants and `impl` blocks to resolve these problems cleanly.

---

## Rust Concept

### 1. Structs with Named Fields
A struct groups together related values with explicit field names:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub name: String,
    pub price_cents: u32,
    pub stock: u32,
}
```

> [!NOTE]
> We use owned `String` fields rather than string slices (`&str`). In Rust, storing a reference (`&str`) inside a struct requires explicit lifetime annotations (e.g., `struct Product<'a>`), because the struct must guarantee the referenced memory outlives the struct itself. Owned types like `String` simplify domain modeling by owning their data on the heap.

### 2. Field Init Shorthand & Struct Update Syntax
When variable names match struct field names, you can omit the redundant assignment:

```rust
let id = 1001;
let name = String::from("Keyboard");
// Instead of Product { id: id, name: name, ... }
let p = Product { id, name, price_cents: 8999, stock: 10 };
```

To create a new instance that borrows most of its values from an existing instance, use the **struct update syntax** (`..`):

```rust
let keyboard_rgb = Product {
    id: 1002,
    name: String::from("Keyboard (RGB Edition)"),
    ..keyboard.clone() // Copies price_cents and stock from `keyboard`
};
```

### 3. Tuple Structs (The Newtype Pattern)
Tuple structs have field types but no field names. They are invaluable for creating strong, zero-cost type abstractions:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomerId(pub u64);
```

Even though both wrap a `u64`, passing an `OrderId` to a function expecting a `CustomerId` is a compile-time error.

### 4. Unit-Like Structs
Structs without any fields behave like `()` (the unit type). They are often used as state markers or packaging descriptors:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandardPackaging;
```

### 5. `impl` Blocks: Methods vs Associated Functions
In Rust, data definitions (`struct`) and logic definitions (`impl`) are strictly decoupled:

```rust
impl Product {
    // Associated Function (Constructor): No `self` parameter
    pub fn new(id: u64, name: String, price_cents: u32, stock: u32) -> Self {
        Self { id, name, price_cents, stock }
    }

    // Method (Read-only inspection): Borrows `self` immutably
    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    // Method (Mutation): Borrows `self` mutably
    pub fn reduce_stock(&mut self, quantity: u32) -> Result<u32, &'static str> {
        if quantity > self.stock {
            Err("Insufficient stock available")
        } else {
            self.stock -= quantity;
            Ok(self.stock)
        }
    }
}
```

- **Associated Functions** (`Product::new(...)`): Called using double colons (`::`). They do not operate on an existing instance and typically serve as constructors.
- **Methods** (`p.is_in_stock()`): Called using dot notation (`.`). They receive `&self`, `&mut self`, or `self` as their first parameter.

---

## Coming From Other Languages

| Concept | Python | Go | Java / C# | C++ | Rust |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Data Grouping** | `class` / `@dataclass` | `type T struct` | `class` / `record` | `class` / `struct` | **`struct`** |
| **Inheritance** | Single & multiple inheritance | No inheritance (embedding) | Class inheritance (`extends`) | Multiple inheritance | **No inheritance (composition only)** |
| **Encapsulation of Behavior** | Methods inside `class` | Methods with receiver `(p *T)` | Methods inside `class` | Methods inside `class` | **Separate `impl` blocks** |
| **Receiver / `this`** | Explicit `self` | Explicit receiver | Implicit `this` pointer | Implicit `this` pointer | **Explicit `&self` / `&mut self`** |
| **Constructor Syntax** | `def __init__(self)` | Conventional factory `NewT()` | `public MyClass()` | `MyClass()` | **Associated function `Self::new()`** |
| **Strong Type Wrappers** | Custom classes | `type OrderId uint64` | Wrapper classes | Typedef / enum class | **Tuple structs (`struct OrderId(u64)`)** |

---

## Small Example: Modeling a User

```rust
#[derive(Debug)]
struct User {
    username: String,
    login_count: u64,
    active: bool,
}

impl User {
    fn new(username: String) -> Self {
        Self {
            username,
            login_count: 0,
            active: true,
        }
    }

    fn record_login(&mut self) {
        self.login_count += 1;
    }
}

fn main() {
    let mut admin = User::new(String::from("root"));
    admin.record_login();
    println!("User state: {:?}", admin);
}
```

---

## Apply To MiniStore
In MiniStore, we now transition to clean, object-oriented domain modeling without classes or inheritance:
1. Model **`Product`**: ID, name, price in cents, and available inventory stock. Includes `reduce_stock(&mut self, qty)` to prevent overselling.
2. Model **`Customer`**: ID, name, email address, and VIP loyalty status.
3. Model **`CartItem`**: Encapsulating a `Product` along with ordered `quantity`, providing a `total_price_cents(&self)` method.
4. Model **`OrderId`**: A type-safe tuple struct wrapper preventing identifier confusion.
5. Model **`OrderSummary`**: An immutable audit record computing subtotals, customer-tier discounts, conditional shipping fees, and final charges.

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
    /// Associated constructor function
    pub fn new(id: u64, name: String, price_cents: u32, stock: u32) -> Self {
        Self {
            id,
            name,
            price_cents,
            stock,
        }
    }

    /// Method taking an immutable reference &self to check stock availability
    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    /// Method taking a mutable reference &mut self to decrement stock on purchase
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

#[derive(Debug, Clone, PartialEq)]
pub struct CartItem {
    pub product: Product,
    pub quantity: u32,
}

impl CartItem {
    pub fn new(product: Product, quantity: u32) -> Self {
        Self { product, quantity }
    }

    /// Computes total price for this line item
    pub fn total_price_cents(&self) -> u32 {
        self.product.price_cents * self.quantity
    }
}

/// Tuple struct: Type-safe order identifier (Newtype Pattern)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderId(pub u64);

/// Unit-like struct: Marker for orders with standard packaging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandardPackaging;

#[derive(Debug, Clone, PartialEq)]
pub struct OrderSummary {
    pub order_id: OrderId,
    pub customer_id: u64,
    pub total_items: u32,
    pub subtotal_cents: u32,
    pub discount_cents: u32,
    pub shipping_fee_cents: u32,
    pub final_total_cents: u32,
}

impl OrderSummary {
    pub fn build(
        order_id: OrderId,
        customer: &Customer,
        cart_items: &[CartItem],
    ) -> Result<Self, &'static str> {
        if cart_items.is_empty() {
            return Err("Cart cannot be empty to build order summary");
        }

        let mut subtotal_cents = 0;
        let mut total_items = 0;

        for item in cart_items {
            subtotal_cents += item.total_price_cents();
            total_items += item.quantity;
        }

        // VIP customers receive an automatic 10% discount; others get tiered discount
        let discount_percent = if customer.is_vip {
            10
        } else if subtotal_cents >= 15000 {
            15
        } else if subtotal_cents >= 10000 {
            10
        } else if subtotal_cents >= 5000 {
            5
        } else {
            0
        };

        let discount_cents = (subtotal_cents * discount_percent) / 100;
        let discounted_subtotal = subtotal_cents - discount_cents;

        // Free shipping on discounted orders >= $100.00 (10000 cents)
        let shipping_fee_cents = if discounted_subtotal >= 10000 { 0 } else { 599 };

        let final_total_cents = discounted_subtotal + shipping_fee_cents;

        Ok(Self {
            order_id,
            customer_id: customer.id,
            total_items,
            subtotal_cents,
            discount_cents,
            shipping_fee_cents,
            final_total_cents,
        })
    }
}

fn main() {
    println!("=== MiniStore: Domain Models with Structs ===");

    // 1. Initializing customer domain struct
    let customer = Customer::new(
        101,
        String::from("Ada Lovelace"),
        String::from("ada@example.com"),
        true, // VIP member
    );
    println!(
        "Customer: {} ({}) | VIP: {}",
        customer.name, customer.email, customer.is_vip
    );

    // 2. Initializing products using associated constructor function
    let mut keyboard = Product::new(1001, String::from("Mechanical Keyboard"), 8999, 10);
    let mut mouse = Product::new(1002, String::from("Ergonomic Mouse"), 4999, 15);

    // 3. Demonstrating Struct Update Syntax
    // Create a special edition keyboard that shares price and stock with base keyboard
    let keyboard_rgb = Product {
        id: 1003,
        name: String::from("Mechanical Keyboard (RGB Edition)"),
        price_cents: 10999,
        ..keyboard.clone()
    };
    println!(
        "\nCatalog Item 1: {:?} (In Stock: {})",
        keyboard,
        keyboard.is_in_stock()
    );
    println!(
        "Catalog Item 2: {:?} (In Stock: {})",
        mouse,
        mouse.is_in_stock()
    );
    println!("Catalog Item 3 (from update syntax): {:?}", keyboard_rgb);

    // 4. Assembling CartItems
    let cart = [
        CartItem::new(keyboard.clone(), 1),
        CartItem::new(mouse.clone(), 2),
    ];

    println!("\n--- Customer Cart ---");
    for item in &cart {
        println!(
            "- {} x {} @ ${:.2} = ${:.2}",
            item.product.name,
            item.quantity,
            item.product.price_cents as f64 / 100.0,
            item.total_price_cents() as f64 / 100.0
        );
    }

    // 5. Building order summary
    let order_id = OrderId(5001);
    let summary = OrderSummary::build(order_id, &customer, &cart)
        .expect("Order summary build should succeed");

    println!("\n--- Order Summary ---");
    println!("Order ID: #{}", summary.order_id.0);
    println!("Total Items: {}", summary.total_items);
    println!("Subtotal: ${:.2}", summary.subtotal_cents as f64 / 100.0);
    println!("Discount: -${:.2}", summary.discount_cents as f64 / 100.0);
    println!(
        "Shipping: {}",
        if summary.shipping_fee_cents == 0 {
            "FREE".to_string()
        } else {
            format!("${:.2}", summary.shipping_fee_cents as f64 / 100.0)
        }
    );
    println!(
        "Final Total: ${:.2}",
        summary.final_total_cents as f64 / 100.0
    );

    // 6. Mutating inventory using &mut self method
    println!("\n--- Updating Inventory ---");
    keyboard.reduce_stock(1).expect("Stock should decrement");
    mouse.reduce_stock(2).expect("Stock should decrement");
    println!("Remaining Keyboard Stock: {}", keyboard.stock);
    println!("Remaining Mouse Stock: {}", mouse.stock);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_creation_and_stock_reduction() {
        let mut p = Product::new(1, String::from("Item"), 1000, 5);
        assert!(p.is_in_stock());
        assert_eq!(p.reduce_stock(2), Ok(3));
        assert_eq!(p.stock, 3);
        assert_eq!(p.reduce_stock(4), Err("Insufficient stock available"));
        assert_eq!(p.stock, 3); // stock unchanged on error
    }

    #[test]
    fn test_cart_item_total() {
        let p = Product::new(1, String::from("USB Hub"), 2500, 10);
        let item = CartItem::new(p, 3);
        assert_eq!(item.total_price_cents(), 7500);
    }

    #[test]
    fn test_struct_update_syntax() {
        let base = Product::new(10, String::from("Base"), 500, 20);
        let updated = Product {
            id: 11,
            name: String::from("Variant"),
            ..base
        };
        assert_eq!(updated.id, 11);
        assert_eq!(updated.name, "Variant");
        assert_eq!(updated.price_cents, 500);
        assert_eq!(updated.stock, 20);
    }

    #[test]
    fn test_order_summary_vip_discount() {
        let vip = Customer::new(1, String::from("VIP"), String::from("vip@test.com"), true);
        let p = Product::new(10, String::from("Item"), 10000, 5); // $100.00
        let items = [CartItem::new(p, 1)];

        let summary = OrderSummary::build(OrderId(1), &vip, &items).unwrap();
        // 10% VIP discount on 10000 = 1000 -> discounted subtotal = 9000 ($90.00)
        // Under $100 -> shipping fee = 599
        // Final total = 9000 + 599 = 9599
        assert_eq!(summary.subtotal_cents, 10000);
        assert_eq!(summary.discount_cents, 1000);
        assert_eq!(summary.shipping_fee_cents, 599);
        assert_eq!(summary.final_total_cents, 9599);
    }

    #[test]
    fn test_order_summary_empty_cart() {
        let customer = Customer::new(2, String::from("User"), String::from("u@test.com"), false);
        let empty_items: [CartItem; 0] = [];
        let res = OrderSummary::build(OrderId(2), &customer, &empty_items);
        assert!(res.is_err());
    }
}
```

---

## Understanding The Code

1. **Associated Constructor Pattern**:
   ```rust
   pub fn new(id: u64, name: String, price_cents: u32, stock: u32) -> Self
   ```
   `Self` (capitalized) is an alias for the type defined in the `impl` block (`Product`). Using `Self` avoids hardcoding the type name and keeps refactoring effortless.

2. **`&self` vs `&mut self`**:
   - `is_in_stock(&self)` only reads the `stock` field. It cannot inadvertently mutate state.
   - `reduce_stock(&mut self, quantity: u32)` explicitly demands mutable access. If a caller holds an immutable binding (`let keyboard = ...`), attempting to invoke `keyboard.reduce_stock(1)` will fail to compile.

3. **Composite Domain Types**:
   ```rust
   pub struct CartItem {
       pub product: Product,
       pub quantity: u32,
   }
   ```
   Rather than building a brittle parallel array of product IDs and quantities, `CartItem` composes `Product` directly.

---

## Common Mistakes

### 1. Attempting to Mutate an Individual Field of an Immutable Struct
In Rust, mutability is a property of the **binding**, not individual fields:

```rust
let p = Product::new(1, String::from("Chair"), 4500, 10);
// p.stock = 5; // ERROR: cannot assign to field `stock` of immutable binding `p`
```
To allow mutating fields or invoking `&mut self` methods, mark the binding itself as mutable: `let mut p = ...;`.

### 2. Confusing Associated Functions (`::`) with Methods (`.`)
Calling an associated function via dot notation or a method via double-colon syntax without self leads to compilation errors:

```rust
let p = Product.new(...); // ERROR: syntax error
let p = Product::new(...); // CORRECT

Product::is_in_stock(); // ERROR: this associated function takes 1 argument but 0 arguments were supplied
p.is_in_stock(); // CORRECT: compiler automatically borrows `&p`
```

---

## Compiler Errors
What happens when you call a `&mut self` method on an immutable binding?

```rust
fn main() {
    let mut keyboard = Product::new(1, String::from("K"), 1000, 5);
    // Suppose keyboard was declared as immutable:
    let keyboard = keyboard; // shadowed into immutable
    keyboard.reduce_stock(1);
}
```

Compiler output:
```text
error[E0596]: cannot borrow `keyboard` as mutable, as it is not declared as mutable
 --> src/main.rs:5:5
  |
4 |     let keyboard = keyboard;
  |         -------- help: consider changing this to be mutable: `mut keyboard`
5 |     keyboard.reduce_stock(1);
  |     ^^^^^^^^ cannot borrow as mutable
```
Rust's compiler catches unauthorized state mutation at compile time before the program can ever run.

---

## Practice
1. Add a method `pub fn restock(&mut self, additional_units: u32)` to `Product` that increments `self.stock`.
2. Add a method `pub fn apply_discount_cents(&mut self, discount_cents: u32)` that decreases `self.price_cents` while ensuring the price cannot drop below `0`.
3. Add a unit test verifying that restocking a product with 5 units increases its inventory count to 15.
4. Run `cargo test` to ensure all tests pass.

---

## Checkpoint
- [x] Defined structs with named fields, tuple structs, and unit-like structs.
- [x] Used field initialization shorthand and struct update syntax (`..`).
- [x] Separated state declaration (`struct`) from behavioral logic (`impl`).
- [x] Implemented methods with `&self` and `&mut self`.
- [x] Implemented constructor associated functions returning `Self`.
- [x] Modeled clean, testable domain entities for MiniStore.

---

## What We Learned
- Rust achieves object-oriented encapsulation without classes, inheritance, or hidden `this` pointers.
- Decoupling data and behavior keeps structures transparent, inspectable, and simple.
- Tuple structs enforce strong domain typing at zero runtime cost.

---

## What's Next
This completes **Part 0: Starting the Journey**! In **Part I: Ownership (Chapter 6: Ownership)**, we explore the revolutionary memory management model that makes Rust unique: stack vs heap, ownership rules, move semantics, and how Rust eliminates garbage collection without manual memory leaks.
