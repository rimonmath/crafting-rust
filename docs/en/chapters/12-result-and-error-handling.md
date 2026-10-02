# Chapter 12: Result and Error Handling

## What You'll Learn
- The difference between **recoverable errors** (expected failures) and **unrecoverable errors** (`panic!`).
- How Rust models fallible operations using the standard library **`Result<T, E>`** enum (`Ok(T)` vs `Err(E)`).
- Why Rust completely rejected exceptions (`try / catch / throw`): explicit function signatures, no hidden control flow, and zero runtime cost on the happy path.
- The **`?` operator**: ergonomic error propagation and early returns without repetitive boilerplate.
- Converting between `Option<T>` and `Result<T, E>` using **`.ok_or()`** and **`.ok_or_else()`**.
- Designing custom domain errors using enums (**`StoreError`**) with rich, structured diagnostic fields.
- Transforming and mapping results using **`.map()`**, **`.map_err()`**, and **`.and_then()`**.
- Applying robust error handling to MiniStore:
  - Replacing unstructured string errors (`&'static str`) with typed domain errors.
  - Enforcing inventory stock bounds (`StoreError::InsufficientStock`).
  - Preventing invalid order state transitions (`StoreError::InvalidStateTransition`).
  - Validating promotional coupons (`StoreError::InvalidCoupon`).
  - Building a transactional multi-step **`checkout`** function with the `?` operator.

---

## Why Do We Need This?
In any real-world software, operations can fail:
- A user tries to buy 5 units of an item, but the warehouse only has 2 left in stock.
- A courier tracking number is submitted for an order that hasn't been paid for yet.
- A shopper types an expired or malformed discount code.
- A customer submits an order with an empty shopping cart.

How do traditional programming languages handle errors?

### 1. Traditional Exceptions (`throw / catch`)
Languages like Java, C++, C#, and Python use exceptions:
```java
// Java / C# style
public Order checkout(Cart cart) {
    if (cart.isEmpty()) {
        throw new EmptyCartException();
    }
    // ...
}
```
While exceptions look convenient at first glance, they carry major architectural problems:
- **Invisible Control Flow**: Looking at `checkout(cart)`, there is no way to know what exceptions can be thrown without reading the entire internal implementation and all its dependencies. Any function call can suddenly abort and unwind the stack.
- **Hidden Bugs**: Developers forget to catch exceptions because the type signature doesn't enforce handling them (especially unchecked runtime exceptions). The program crashes at runtime.
- **Heavy Runtime Overhead**: Setting up stack unwinding tables and capturing stack traces degrades performance even when exceptions are not thrown.

### 2. C-Style Error Codes (`-1` or `NULL`)
Languages like C and Go often use sentinel return values (`-1`, `NULL`, or returning `(value, err)` tuples):
```c
// C style
int result = reduce_stock(product_id, 5);
if (result == -1) {
    // what went wrong? Insufficient stock? Database disconnected?
}
```
Sentinel numbers lose all diagnostic details, and developers frequently forget to check return codes, allowing corrupted state to silently propagate through the system.

**Rust's Philosophy**: **Errors are just values.**

Rust has no exceptions. Instead, a function that can fail returns a `Result<T, E>`.
- The possibility of failure is **explicitly declared** in the function signature.
- The caller **cannot access the success value `T`** without explicitly acknowledging that an error `E` might have occurred.
- Errors are regular enum values: they can be matched, inspected, transformed, stored, or propagated with the `?` operator.

---

## The Problem
In our previous chapters, MiniStore used temporary string slices for error messages:
```rust
// Early prototype in Chapter 10:
pub fn reduce_stock(&mut self, quantity: u32) -> Result<u32, &'static str> {
    if quantity > self.stock {
        Err("Insufficient stock available")
    } else {
        self.stock -= quantity;
        Ok(self.stock)
    }
}
```

While this was a useful stepping stone, unstructured string errors have serious limitations in production:
1. **No Programmatic Inspection**: If downstream code needs to know *how many* units are actually available (e.g. to display "Only 2 left in stock!"), it cannot parse numbers out of an English sentence reliably.
2. **String Typos**: A caller trying to check `if err == "Insufficient stock available"` will break if someone edits the message for better grammar.
3. **No Centralized Error Architecture**: Order state errors, payment errors, and inventory errors are all mixed together as plain strings.
4. **Boilerplate Propagation**: Without the `?` operator, combining multiple fallible operations (checking cart -> looking up product in catalog -> verifying stock -> validating coupon -> creating order) requires deep, unreadable pyramids of nested `match` statements.

MiniStore needs:
- A strongly-typed enum (`StoreError`) representing all business failure modes with structured payloads.
- An ergonomic way to chain and propagate errors cleanly using the `?` operator.
- Methods to convert `Option` lookups into descriptive domain errors.

---

## Rust Concept

### 1. The `Result<T, E>` Definition
In Rust's standard library, `Result<T, E>` is an enum with two variants:

```rust
pub enum Result<T, E> {
    Ok(T),  // Represents success, containing value T
    Err(E), // Represents failure, containing error E
}
```

Both `Result`, `Ok`, and `Err` are in the standard prelude—no imports required.

Just like `Option<T>` forces you to handle `None`, `Result<T, E>` forces you to handle `Err`. If a function returns `Result<Order, StoreError>`, you cannot use the `Order` without checking whether it is `Ok(order)` or `Err(error)`.

### 2. Recoverable vs Unrecoverable Errors
Rust makes a clear distinction:
- **Unrecoverable Errors (`panic!`)**: Bugs in the code that cannot be reasonably handled at runtime (e.g., array index out of bounds, broken mathematical invariants). The program immediately aborts or unwinds the stack.
- **Recoverable Errors (`Result<T, E>`)**: Situations where failure is expected as part of normal business operation (e.g., user enters invalid coupon, item out of stock, file not found). These should **never** panic; they must return `Result`.

```text
┌─────────────────────────────────────────────────────────────┐
│                       Error Handling                        │
├──────────────────────────────┬──────────────────────────────┤
│    Recoverable (Result)      │   Unrecoverable (panic!)     │
├──────────────────────────────┼──────────────────────────────┤
│ - Out of stock               │ - Array index out of bounds  │
│ - Invalid coupon code        │ - Assertion failure          │
│ - Payment declined           │ - Memory corruption check    │
│ - Missing product in catalog │ - Impossible internal state  │
│ ──► Handled with Result<T,E> │ ──► Program terminates       │
└──────────────────────────────┴──────────────────────────────┘
```

### 3. Custom Domain Error Enums
Instead of returning vague strings, idiomatic Rust defines domain error enums:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum StoreError {
    InsufficientStock { available: u32, requested: u32 },
    InvalidStateTransition { current: String, action: String },
    ProductNotFound { identifier: String },
    InvalidCoupon { code: String, reason: String },
    EmptyCart,
}
```

Notice the benefits:
- Each variant carries **precisely the data needed** to diagnose or resolve the issue.
- Callers can match on specific error variants programmatically:
  ```rust
  match product.reduce_stock(qty) {
      Ok(remaining) => println!("Purchased! Left: {remaining}"),
      Err(StoreError::InsufficientStock { available, requested }) => {
          println!("Sorry! You wanted {requested}, but we only have {available}.");
      }
      Err(e) => println!("Other error: {}", e.message()),
  }
  ```

### 4. The `?` Operator: Clean Error Propagation
In languages without exceptions, propagating errors can feel tedious if done manually:

```rust
// WITHOUT the ? operator: Verbose and repetitive!
let product = match catalog.find_by_id_mut(item.product_id) {
    Some(p) => p,
    None => return Err(StoreError::ProductNotFound { identifier: "ID".into() }),
};

match product.reduce_stock(item.quantity) {
    Ok(remaining) => remaining,
    Err(err) => return Err(err), // early return error
};
```

Rust solves this with the **`?` operator**:

```rust
// WITH the ? operator: Clean, concise, and safe!
let product = catalog
    .find_by_id_mut(item.product_id)
    .ok_or_else(|| StoreError::ProductNotFound { identifier: "ID".into() })?;

product.reduce_stock(item.quantity)?;
```

How `?` works:
1. If the expression evaluates to `Ok(value)`, the `?` operator unwraps the inner `value` and execution continues.
2. If the expression evaluates to `Err(error)`, the function **immediately returns** `Err(error)` to the caller.

> [!NOTE]
> The `?` operator can only be used in functions whose return type is compatible with the error being returned (e.g. returning `Result<T, E>`).

### 5. Bridging `Option` and `Result`
Often, an operation returns `Option<T>` (e.g., looking up an item in a map), but our business function needs to return `Result<T, StoreError>`. Rust provides seamless conversion methods:

- **`.ok_or(err)`**: Converts `Some(v)` into `Ok(v)`, and `None` into `Err(err)`.
- **`.ok_or_else(|| err)`**: Same as `.ok_or()`, but lazily creates the error using a closure (ideal when building the error string involves allocation).
- **`.ok()`**: Converts `Result<T, E>` into `Option<T>`, discarding the error detail if you only care about presence.

```rust
// HashMap::get returns Option<&Product>
// .ok_or_else converts None into StoreError::ProductNotFound
let product = catalog.find_by_sku("TECH-01")
    .ok_or_else(|| StoreError::ProductNotFound { identifier: "TECH-01".into() })?;
```

---

## Coming From Other Languages

| Concept | Rust | Java / C# | Python | Go | C++ |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Error paradigm** | Return `Result<T, E>` | `throw / catch` Exceptions | `raise / except` Exceptions | Multiple returns `(val, err)` | `throw / catch` or `std::expected` |
| **Type safety** | **Enforced at compile time** | Partial (Checked exceptions bypassed) | None (Runtime duck typing) | Manual `if err != nil` convention | Unenforced exceptions |
| **Early exit syntax** | `?` operator | Automatic stack unwinding | Automatic stack unwinding | Repetitive `if err != nil { return nil, err }` | Automatic stack unwinding |
| **Performance overhead** | **Zero cost** on success path | Heavy (stack capture, unwinding) | Heavy (frame inspection) | Low (pointer return) | Heavy unwinding tables |
| **Error diagnostics** | Typed enum variants with fields | Exception classes with inheritance | Exception classes | String formatting `errors.New` | `std::exception` / error codes |

---

## Small Example
Here is a self-contained, copy-pasteable script demonstrating custom errors, `Result<T, E>`, and the `?` operator:

```rust
#[derive(Debug, PartialEq)]
enum BankError {
    InsufficientFunds { balance: u32, attempt: u32 },
    DailyLimitExceeded,
}

struct BankAccount {
    balance_cents: u32,
}

impl BankAccount {
    fn withdraw(&mut self, amount: u32) -> Result<u32, BankError> {
        if amount > 50000 {
            return Err(BankError::DailyLimitExceeded);
        }
        if amount > self.balance_cents {
            return Err(BankError::InsufficientFunds {
                balance: self.balance_cents,
                attempt: amount,
            });
        }
        self.balance_cents -= amount;
        Ok(self.balance_cents)
    }
}

// Function using the ? operator to chain multiple transactions
fn process_transfer(from: &mut BankAccount, to: &mut BankAccount, amount: u32) -> Result<(), BankError> {
    from.withdraw(amount)?; // Early returns if withdraw fails!
    to.balance_cents += amount;
    Ok(())
}

fn main() {
    let mut account_a = BankAccount { balance_cents: 10000 };
    let mut account_b = BankAccount { balance_cents: 5000 };

    // 1. Successful transfer
    match process_transfer(&mut account_a, &mut account_b, 4000) {
        Ok(()) => println!("Transfer successful! Account A balance: ${:.2}", account_a.balance_cents as f64 / 100.0),
        Err(e) => println!("Transfer failed: {:?}", e),
    }

    // 2. Failed transfer (insufficient funds)
    match process_transfer(&mut account_a, &mut account_b, 8000) {
        Ok(()) => println!("Transfer successful!"),
        Err(BankError::InsufficientFunds { balance, attempt }) => {
            println!("Declined: Tried to transfer ${:.2}, but balance is only ${:.2}", attempt as f64 / 100.0, balance as f64 / 100.0);
        }
        Err(BankError::DailyLimitExceeded) => println!("Declined: Exceeded limit"),
    }
}
```

---

## Apply To MiniStore
In MiniStore, we integrate `Result<T, StoreError>` across the entire business lifecycle:

```text
┌─────────────────────────────────────────────────────────────┐
│                         StoreError                          │
├─────────────────────────────────────────────────────────────┤
│ + InsufficientStock { available: u32, requested: u32 }      │
│ + InvalidStateTransition { current: String, action: String }│
│ + ProductNotFound { identifier: String }                    │
│ + InvalidPayment { reason: String }                         │
│ + InvalidCoupon { code: String, reason: String }            │
│ + EmptyCart                                                 │
└─────────────────────────────────────────────────────────────┘
                               ▲
                               │ returned by
   ┌───────────────────────────┴───────────────────────────┐
   │                                                       │
┌──────────────┐     ┌──────────────┐             ┌─────────────────┐
│   Product    │     │    Order     │             │    checkout     │
├──────────────┤     ├──────────────┤             ├─────────────────┤
│ reduce_stock │     │ confirm()    │             │ Validates:      │
│              │     │ ship()       │             │ 1. Cart non-empty│
│              │     │ deliver()    │             │ 2. Coupon valid │
│              │     │ cancel()     │             │ 3. Catalog item │
│              │     │              │             │ 4. Deducts stock│
└──────────────┘     └──────────────┘             └─────────────────┘
                                                           │
                                                           ▼ (using ?)
                                              Result<Order, StoreError>
```

1. **`StoreError`**: A comprehensive enum with structured diagnostic data and a `.message()` method for user presentation.
2. **`Product::reduce_stock`**: Returns `Result<u32, StoreError>`, ensuring callers know exactly how many units were requested vs available.
3. **`Coupon::validate`**: Returns `Result<(), StoreError>`, rejecting empty codes or impossible discount percentages (> 100%).
4. **`Order` Transitions**: `confirm`, `ship`, `mark_delivered`, and `cancel` return `Result<(), StoreError>` instead of raw strings.
5. **Transactional `checkout`**:
   - Accepts customer, shopping cart, catalog, payment method, and optional coupon.
   - Uses `?` to sequentially validate the cart, validate the coupon, look up each product in the catalog with `.ok_or_else(...)?`, and deduct stock with `product.reduce_stock(...)?`.
   - If any step fails, the function returns immediately with the exact `StoreError`.

---

## Code
Here is the complete, production-ready MiniStore codebase for Chapter 12. You can find this snapshot in `examples/chapter-12/src/main.rs`:

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum ProductCategory {
    Electronics,
    OfficeSupplies,
    Furniture,
    Custom(String),
}

impl ProductCategory {
    pub fn default_tax_rate(&self) -> u32 {
        match self {
            Self::Electronics => 15,
            Self::OfficeSupplies => 5,
            Self::Furniture => 10,
            Self::Custom(_) => 8,
        }
    }
}

/// Domain errors that can occur during MiniStore business operations.
#[derive(Debug, Clone, PartialEq)]
pub enum StoreError {
    InsufficientStock { available: u32, requested: u32 },
    InvalidStateTransition { current: String, action: String },
    ProductNotFound { identifier: String },
    InvalidPayment { reason: String },
    InvalidCoupon { code: String, reason: String },
    EmptyCart,
}

impl StoreError {
    /// User-friendly descriptive error message.
    pub fn message(&self) -> String {
        match self {
            Self::InsufficientStock {
                available,
                requested,
            } => {
                format!("Insufficient stock: requested {requested}, but only {available} available")
            }
            Self::InvalidStateTransition { current, action } => {
                format!("Cannot perform action '{action}' while order is in '{current}' state")
            }
            Self::ProductNotFound { identifier } => {
                format!("Product '{identifier}' was not found in catalog")
            }
            Self::InvalidPayment { reason } => {
                format!("Payment processing failed: {reason}")
            }
            Self::InvalidCoupon { code, reason } => {
                format!("Coupon '{code}' is invalid: {reason}")
            }
            Self::EmptyCart => String::from("Cannot checkout with an empty shopping cart"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub sku: String,
    pub name: String,
    pub category: ProductCategory,
    pub price_cents: u32,
    pub stock: u32,
}

impl Product {
    pub fn new(
        id: u64,
        sku: String,
        name: String,
        category: ProductCategory,
        price_cents: u32,
        stock: u32,
    ) -> Self {
        Self {
            id,
            sku,
            name,
            category,
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
    pub fn department_code(&self) -> &str {
        match self.sku.find('-') {
            Some(idx) => &self.sku[..idx],
            None => &self.sku[..],
        }
    }

    /// Returns a slice of the product name truncated to `max_bytes` safely.
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
    pub fn reduce_stock(&mut self, quantity: u32) -> Result<u32, StoreError> {
        if quantity > self.stock {
            Err(StoreError::InsufficientStock {
                available: self.stock,
                requested: quantity,
            })
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
    pub phone: Option<String>,
    pub is_vip: bool,
    pub tags: HashSet<String>,
}

impl Customer {
    pub fn new(id: u64, name: String, email: String, phone: Option<String>, is_vip: bool) -> Self {
        Self {
            id,
            name,
            email,
            phone,
            is_vip,
            tags: HashSet::new(),
        }
    }

    pub fn add_tag(&mut self, tag: &str) -> bool {
        self.tags.insert(tag.to_string())
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    pub fn display_badge(&self) -> String {
        if self.is_vip {
            format!("[VIP Member] {}", self.name)
        } else {
            format!("[Standard Member] {}", self.name)
        }
    }

    pub fn upgrade_to_vip(&mut self) {
        self.is_vip = true;
        self.tags.insert(String::from("vip"));
    }

    pub fn update_email(&mut self, new_email: String) {
        self.email = new_email;
    }

    pub fn formatted_phone(&self) -> &str {
        self.phone.as_deref().unwrap_or("Unspecified")
    }
}

/// A discount coupon that can optionally be applied to an order.
#[derive(Debug, Clone, PartialEq)]
pub struct Coupon {
    pub code: String,
    pub discount_percent: u32,
}

impl Coupon {
    pub fn new(code: String, discount_percent: u32) -> Self {
        Self {
            code,
            discount_percent,
        }
    }

    /// Validates that coupon has a non-empty code and a realistic discount percentage.
    pub fn validate(&self) -> Result<(), StoreError> {
        if self.code.trim().is_empty() {
            Err(StoreError::InvalidCoupon {
                code: self.code.clone(),
                reason: String::from("Coupon code cannot be empty"),
            })
        } else if self.discount_percent == 0 || self.discount_percent > 100 {
            Err(StoreError::InvalidCoupon {
                code: self.code.clone(),
                reason: format!(
                    "Discount percentage {} must be between 1 and 100",
                    self.discount_percent
                ),
            })
        } else {
            Ok(())
        }
    }
}

/// Catalog of products indexed by SKU for O(1) lookups returning `Option<&Product>`.
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

    pub fn total_products(&self) -> usize {
        self.products.len()
    }
}

/// OrderId implements `Copy`: 8-byte scalar on the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(pub u64);

/// Represents payment instruments accepted by MiniStore.
#[derive(Debug, Clone, PartialEq)]
pub enum PaymentMethod {
    CreditCard { last_four: String },
    BankTransfer { reference: String },
    CashOnDelivery,
}

impl PaymentMethod {
    /// Returns transaction or handling fee in cents based on payment method.
    pub fn fee_cents(&self) -> u32 {
        match self {
            Self::CreditCard { .. } => 150, // $1.50 processing fee
            Self::BankTransfer { .. } => 0, // Free
            Self::CashOnDelivery => 300,    // $3.00 handling fee
        }
    }

    /// User-friendly description of payment channel.
    pub fn description(&self) -> String {
        match self {
            Self::CreditCard { last_four } => format!("Credit Card (ending in {last_four})"),
            Self::BankTransfer { reference } => format!("Bank Transfer (Ref: {reference})"),
            Self::CashOnDelivery => String::from("Cash on Delivery"),
        }
    }
}

/// Explicit lifecycle states for an Order. Enums prevent impossible states!
#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Confirmed { receipt_id: String },
    Shipped { tracking_number: String },
    Delivered,
    Cancelled { reason: String },
}

impl OrderStatus {
    /// Formats state for display using pattern matching.
    pub fn display_status(&self) -> String {
        match self {
            Self::Pending => String::from("Awaiting Confirmation"),
            Self::Confirmed { receipt_id } => format!("Confirmed (Receipt: {receipt_id})"),
            Self::Shipped { tracking_number } => format!("Shipped (Tracking: {tracking_number})"),
            Self::Delivered => String::from("Delivered to Customer"),
            Self::Cancelled { reason } => format!("Cancelled (Reason: {reason})"),
        }
    }

    /// Orders can only be cancelled while still Pending or Confirmed.
    pub fn can_cancel(&self) -> bool {
        match self {
            Self::Pending | Self::Confirmed { .. } => true,
            Self::Shipped { .. } | Self::Delivered | Self::Cancelled { .. } => false,
        }
    }

    /// Checks if order is in a final, immutable terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Delivered | Self::Cancelled { .. })
    }
}

/// Represents a single line item in a shopping cart or order.
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

/// A dynamic shopping cart backed by `Vec<CartItem>`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShoppingCart {
    pub items: Vec<CartItem>,
}

impl ShoppingCart {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn get_item(&self, product_id: u64) -> Option<&CartItem> {
        self.items.iter().find(|item| item.product_id == product_id)
    }

    pub fn get_item_mut(&mut self, product_id: u64) -> Option<&mut CartItem> {
        self.items
            .iter_mut()
            .find(|item| item.product_id == product_id)
    }

    pub fn add_item(&mut self, product_id: u64, quantity: u32, unit_price_cents: u32) {
        if let Some(item) = self.get_item_mut(product_id) {
            item.quantity += quantity;
        } else {
            self.items
                .push(CartItem::new(product_id, quantity, unit_price_cents));
        }
    }

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

/// Full Order domain model with state-machine lifecycle and optional coupon discount.
#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub order_id: OrderId,
    pub customer: Customer,
    pub items: Vec<CartItem>,
    pub payment: PaymentMethod,
    pub status: OrderStatus,
    pub coupon: Option<Coupon>,
}

impl Order {
    pub fn new(
        order_id: OrderId,
        customer: Customer,
        items: Vec<CartItem>,
        payment: PaymentMethod,
        coupon: Option<Coupon>,
    ) -> Self {
        Self {
            order_id,
            customer,
            items,
            payment,
            status: OrderStatus::Pending,
            coupon,
        }
    }

    pub fn apply_coupon(&mut self, coupon: Coupon) {
        self.coupon = Some(coupon);
    }

    pub fn remove_coupon(&mut self) -> Option<Coupon> {
        self.coupon.take()
    }

    /// Transitions Pending -> Confirmed { receipt_id }.
    pub fn confirm(&mut self, receipt_id: String) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Pending => {
                self.status = OrderStatus::Confirmed { receipt_id };
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                current: self.status.display_status(),
                action: String::from("confirm"),
            }),
        }
    }

    /// Transitions Confirmed -> Shipped { tracking_number }.
    pub fn ship(&mut self, tracking_number: String) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Confirmed { .. } => {
                self.status = OrderStatus::Shipped { tracking_number };
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                current: self.status.display_status(),
                action: String::from("ship"),
            }),
        }
    }

    /// Transitions to Delivered.
    pub fn mark_delivered(&mut self) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Shipped { .. } => {
                self.status = OrderStatus::Delivered;
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                current: self.status.display_status(),
                action: String::from("deliver"),
            }),
        }
    }

    /// Cancels order if current state permits.
    pub fn cancel(&mut self, reason: String) -> Result<(), StoreError> {
        if self.status.can_cancel() {
            self.status = OrderStatus::Cancelled { reason };
            Ok(())
        } else {
            Err(StoreError::InvalidStateTransition {
                current: self.status.display_status(),
                action: String::from("cancel"),
            })
        }
    }

    pub fn subtotal_cents(&self) -> u32 {
        self.items.iter().map(|item| item.line_total()).sum()
    }

    pub fn total_cents(&self) -> u32 {
        let subtotal = self.subtotal_cents();
        let discount = calculate_discount(&self.customer, self.coupon.as_ref(), subtotal);
        subtotal.saturating_sub(discount) + self.payment.fee_cents()
    }
}

/// Processes a shopping cart into a confirmed order, validating stock and coupon with `?`.
pub fn checkout(
    order_id: OrderId,
    customer: Customer,
    cart: &mut ShoppingCart,
    catalog: &mut Catalog,
    payment: PaymentMethod,
    coupon: Option<Coupon>,
) -> Result<Order, StoreError> {
    if cart.is_empty() {
        return Err(StoreError::EmptyCart);
    }

    if let Some(c) = &coupon {
        c.validate()?;
    }

    // Check inventory and deduct stock for every cart item using `?` error propagation
    for item in &cart.items {
        let product =
            catalog
                .find_by_id_mut(item.product_id)
                .ok_or_else(|| StoreError::ProductNotFound {
                    identifier: format!("ID #{}", item.product_id),
                })?;

        product.reduce_stock(item.quantity)?;
    }

    let items = std::mem::take(&mut cart.items);
    Ok(Order::new(order_id, customer, items, payment, coupon))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculates discount based on customer VIP status and optional Coupon.
pub fn calculate_discount(
    customer: &Customer,
    coupon: Option<&Coupon>,
    subtotal_cents: u32,
) -> u32 {
    let vip_discount = if customer.is_vip || customer.has_tag("vip") {
        (subtotal_cents * 10) / 100 // 10% discount for VIPs
    } else {
        0
    };

    let coupon_discount = coupon
        .map(|c| (subtotal_cents * c.discount_percent) / 100)
        .unwrap_or(0);

    vip_discount + coupon_discount
}

/// Groups products by department and counts them using `HashMap` and the Entry API.
pub fn count_products_by_department(products: &[Product]) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for product in products {
        let dept = product.department_code().to_string();
        *counts.entry(dept).or_insert(0) += 1;
    }
    counts
}

/// Evaluates dispatch message based on OrderStatus using exhaustive pattern matching.
pub fn order_dispatch_advisory(status: &OrderStatus) -> &'static str {
    match status {
        OrderStatus::Pending => "Hold in warehouse: waiting for customer payment.",
        OrderStatus::Confirmed { .. } => "Ready to pick and pack at fulfillment center.",
        OrderStatus::Shipped { .. } => "In transit with logistics courier.",
        OrderStatus::Delivered => "Package successfully delivered to recipient.",
        OrderStatus::Cancelled { .. } => "Halted: restock inventory items immediately.",
    }
}

fn main() {
    println!("=== MiniStore: Result & Robust Error Handling (Part II) ===\n");

    // 1. Initializing Catalog with Products
    let mut catalog = Catalog::new();
    let keyboard = Product::new(
        101,
        String::from("TECH-KEY-001"),
        String::from("Tenkeyless Mechanical Keyboard"),
        ProductCategory::Electronics,
        12000,
        2, // Only 2 in stock!
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

    // 2. Handling Recoverable Errors with match on Result<T, StoreError>
    println!("1. Handling Recoverable Errors (Stock Reduction):");
    let mut desk_pad = Product::new(
        103,
        String::from("OFF-PAD-003"),
        String::from("Leather Desk Mat"),
        ProductCategory::OfficeSupplies,
        2500,
        3,
    );

    // Attempting to buy 5 units when only 3 exist
    match desk_pad.reduce_stock(5) {
        Ok(remaining) => println!("   Stock reduced successfully! Remaining: {remaining}"),
        Err(err) => println!("   Error handled gracefully -> {}", err.message()),
    }

    // 3. Coupon Validation returning Result<(), StoreError>
    println!("\n2. Domain Validation with Custom Errors:");
    let invalid_coupon = Coupon::new(String::from(""), 120);
    match invalid_coupon.validate() {
        Ok(()) => println!("   Coupon is valid!"),
        Err(err) => println!("   Coupon rejected -> {}", err.message()),
    }

    let valid_coupon = Coupon::new(String::from("SUMMER15"), 15);
    if let Err(err) = valid_coupon.validate() {
        println!("   Unexpected coupon error: {}", err.message());
    } else {
        println!("   Coupon 'SUMMER15' (15%) validated successfully!");
    }

    // 4. End-to-End Checkout with '?' Operator Error Propagation
    println!("\n3. Checkout Workflow & Error Propagation (?):");
    let customer = Customer::new(
        301,
        String::from("Margaret Hamilton"),
        String::from("margaret@apollo.nasa.gov"),
        Some(String::from("+1-555-0199")),
        true,
    );

    // Scenario A: Attempt checkout with empty cart
    let mut empty_cart = ShoppingCart::new();
    match checkout(
        OrderId(901),
        customer.clone(),
        &mut empty_cart,
        &mut catalog,
        PaymentMethod::CashOnDelivery,
        None,
    ) {
        Ok(_) => println!("   Checkout succeeded unexpectedly!"),
        Err(err) => println!("   Empty cart rejected -> {}", err.message()),
    }

    // Scenario B: Attempt checkout with more units than stock available (Keyboard has only 2)
    let mut greedy_cart = ShoppingCart::new();
    greedy_cart.add_item(101, 5, 12000); // Wants 5 units
    match checkout(
        OrderId(902),
        customer.clone(),
        &mut greedy_cart,
        &mut catalog,
        PaymentMethod::CreditCard {
            last_four: String::from("4242"),
        },
        None,
    ) {
        Ok(_) => println!("   Checkout succeeded unexpectedly!"),
        Err(err) => println!("   Excessive quantity rejected -> {}", err.message()),
    }

    // Scenario C: Successful checkout with valid quantities and coupon
    let mut valid_cart = ShoppingCart::new();
    valid_cart.add_item(101, 1, 12000); // 1 keyboard
    valid_cart.add_item(102, 2, 4500); // 2 mice

    match checkout(
        OrderId(903),
        customer,
        &mut valid_cart,
        &mut catalog,
        PaymentMethod::CreditCard {
            last_four: String::from("9876"),
        },
        Some(valid_coupon),
    ) {
        Ok(mut order) => {
            println!(
                "   Checkout Order #{} created! Total: ${:.2} (Subtotal: ${:.2})",
                order.order_id.0,
                order.total_cents() as f64 / 100.0,
                order.subtotal_cents() as f64 / 100.0
            );

            // 5. Order State Machine Enforcing Transitions with StoreError
            println!("\n4. Order Lifecycle Transitions:");
            order.confirm(String::from("REC-903-HAMILTON")).unwrap();
            println!("   Order confirmed: {}", order.status.display_status());

            // Attempting invalid transition: cannot ship before courier tracking is set
            order.ship(String::from("TRK-FEDEX-88319")).unwrap();
            println!("   Order shipped:   {}", order.status.display_status());

            // Attempting illegal cancellation once shipped
            match order.cancel(String::from("Buyer changed mind")) {
                Ok(()) => println!("   Order cancelled!"),
                Err(err) => println!("   Cancellation prevented -> {}", err.message()),
            }

            order.mark_delivered().unwrap();
            println!("   Order delivered: {}", order.status.display_status());
        }
        Err(err) => println!("   Checkout failed: {}", err.message()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_method_fees_and_descriptions() {
        let card = PaymentMethod::CreditCard {
            last_four: String::from("1234"),
        };
        let transfer = PaymentMethod::BankTransfer {
            reference: String::from("TX-99"),
        };
        let cod = PaymentMethod::CashOnDelivery;

        assert_eq!(card.fee_cents(), 150);
        assert_eq!(transfer.fee_cents(), 0);
        assert_eq!(cod.fee_cents(), 300);

        assert_eq!(card.description(), "Credit Card (ending in 1234)");
        assert_eq!(transfer.description(), "Bank Transfer (Ref: TX-99)");
        assert_eq!(cod.description(), "Cash on Delivery");
    }

    #[test]
    fn test_order_status_valid_lifecycle() {
        let customer = Customer::new(
            1,
            String::from("Alice"),
            String::from("a@a.com"),
            None,
            false,
        );
        let items = vec![CartItem::new(10, 1, 5000)];
        let mut order = Order::new(
            OrderId(100),
            customer,
            items,
            PaymentMethod::CashOnDelivery,
            None,
        );

        // Starts pending
        assert_eq!(order.status, OrderStatus::Pending);
        assert!(order.status.can_cancel());
        assert!(!order.status.is_terminal());

        // Cannot ship while pending
        let ship_err = order.ship(String::from("TRK-1")).unwrap_err();
        assert_eq!(
            ship_err,
            StoreError::InvalidStateTransition {
                current: String::from("Awaiting Confirmation"),
                action: String::from("ship"),
            }
        );

        // Confirm
        assert!(order.confirm(String::from("REC-100")).is_ok());
        assert_eq!(
            order.status,
            OrderStatus::Confirmed {
                receipt_id: String::from("REC-100")
            }
        );
        assert!(order.status.can_cancel());

        // Ship
        assert!(order.ship(String::from("TRK-100")).is_ok());
        assert_eq!(
            order.status,
            OrderStatus::Shipped {
                tracking_number: String::from("TRK-100")
            }
        );
        assert!(!order.status.can_cancel());

        // Deliver
        assert!(order.mark_delivered().is_ok());
        assert_eq!(order.status, OrderStatus::Delivered);
        assert!(order.status.is_terminal());
    }

    #[test]
    fn test_order_cancellation_prevention() {
        let customer = Customer::new(2, String::from("Bob"), String::from("b@b.com"), None, false);
        let items = vec![CartItem::new(20, 2, 2500)];
        let mut order = Order::new(
            OrderId(200),
            customer,
            items,
            PaymentMethod::CashOnDelivery,
            None,
        );

        // Cancel while pending succeeds
        assert!(order.cancel(String::from("Out of stock")).is_ok());
        assert_eq!(
            order.status,
            OrderStatus::Cancelled {
                reason: String::from("Out of stock")
            }
        );
        assert!(order.status.is_terminal());

        // Cannot confirm a cancelled order
        let confirm_err = order.confirm(String::from("REC-200")).unwrap_err();
        assert_eq!(
            confirm_err,
            StoreError::InvalidStateTransition {
                current: String::from("Cancelled (Reason: Out of stock)"),
                action: String::from("confirm"),
            }
        );
    }

    #[test]
    fn test_product_category_tax_rates() {
        assert_eq!(ProductCategory::Electronics.default_tax_rate(), 15);
        assert_eq!(ProductCategory::OfficeSupplies.default_tax_rate(), 5);
        assert_eq!(ProductCategory::Furniture.default_tax_rate(), 10);
        assert_eq!(
            ProductCategory::Custom(String::from("Handmade")).default_tax_rate(),
            8
        );
    }

    #[test]
    fn test_order_total_with_payment_fee() {
        let mut customer = Customer::new(
            3,
            String::from("Carol"),
            String::from("c@c.com"),
            None,
            false,
        );
        customer.upgrade_to_vip(); // 10% discount

        let items = vec![CartItem::new(1, 1, 10000)]; // $100.00 subtotal
        let order = Order::new(
            OrderId(300),
            customer,
            items,
            PaymentMethod::CreditCard {
                last_four: String::from("1111"),
            }, // $1.50 (150 cents) fee
            None,
        );

        // Subtotal: 10000 cents
        // VIP discount: 1000 cents
        // Card fee: 150 cents
        // Total: 10000 - 1000 + 150 = 9150 cents ($91.50)
        assert_eq!(order.total_cents(), 9150);
    }

    #[test]
    fn test_catalog_option_lookups() {
        let mut catalog = Catalog::new();
        let prod = Product::new(
            50,
            String::from("OFF-DESK-01"),
            String::from("Standing Desk"),
            ProductCategory::Furniture,
            35000,
            5,
        );
        catalog.add_product(prod);

        // Existing lookups return Some(&Product)
        assert!(catalog.find_by_sku("OFF-DESK-01").is_some());
        assert_eq!(catalog.find_by_sku("OFF-DESK-01").unwrap().id, 50);
        assert_eq!(
            catalog.find_by_id(50).map(|p| p.sku.as_str()),
            Some("OFF-DESK-01")
        );
        assert_eq!(catalog.product_price("OFF-DESK-01"), Some(35000));
        assert!(catalog.is_product_in_stock("OFF-DESK-01"));

        // Non-existent lookups return None
        assert_eq!(catalog.find_by_sku("UNKNOWN-SKU"), None);
        assert_eq!(catalog.find_by_id(999), None);
        assert_eq!(catalog.product_price("UNKNOWN-SKU"), None);
        assert!(!catalog.is_product_in_stock("UNKNOWN-SKU"));
    }

    #[test]
    fn test_customer_optional_phone() {
        let with_phone = Customer::new(
            1,
            String::from("Alice"),
            String::from("alice@ex.com"),
            Some(String::from("+1-202-555-0143")),
            false,
        );
        let no_phone = Customer::new(
            2,
            String::from("Bob"),
            String::from("bob@ex.com"),
            None,
            false,
        );

        assert_eq!(with_phone.formatted_phone(), "+1-202-555-0143");
        assert_eq!(no_phone.formatted_phone(), "Unspecified");
        assert!(with_phone.phone.is_some());
        assert!(no_phone.phone.is_none());
    }

    #[test]
    fn test_cart_item_option_lookup() {
        let mut cart = ShoppingCart::new();
        cart.add_item(10, 2, 1500);

        assert!(cart.get_item(10).is_some());
        assert_eq!(cart.get_item(10).unwrap().quantity, 2);
        assert!(cart.get_item(99).is_none());

        // Increment existing item via add_item
        cart.add_item(10, 3, 1500);
        assert_eq!(cart.get_item(10).unwrap().quantity, 5);
    }

    #[test]
    fn test_coupon_discount_and_take() {
        let customer = Customer::new(
            10,
            String::from("Dave"),
            String::from("d@d.com"),
            None,
            false,
        );
        let items = vec![CartItem::new(1, 1, 20000)]; // $200.00
        let coupon = Coupon::new(String::from("SAVE15"), 15); // 15% discount = $30.00 (3000 cents)

        let mut order = Order::new(
            OrderId(500),
            customer,
            items,
            PaymentMethod::BankTransfer {
                reference: String::from("REF1"),
            },
            Some(coupon),
        );

        assert_eq!(order.total_cents(), 17000);

        let extracted = order.remove_coupon();
        assert_eq!(
            extracted,
            Some(Coupon {
                code: String::from("SAVE15"),
                discount_percent: 15
            })
        );
        assert_eq!(order.coupon, None);
        assert_eq!(order.total_cents(), 20000);
    }

    #[test]
    fn test_product_stock_reduction_error() {
        let mut product = Product::new(
            1,
            String::from("SKU-1"),
            String::from("Book"),
            ProductCategory::OfficeSupplies,
            1000,
            5,
        );

        // Success
        assert_eq!(product.reduce_stock(3), Ok(2));
        assert_eq!(product.stock, 2);

        // Insufficient stock error
        let err = product.reduce_stock(5).unwrap_err();
        assert_eq!(
            err,
            StoreError::InsufficientStock {
                available: 2,
                requested: 5,
            }
        );
        assert_eq!(
            err.message(),
            "Insufficient stock: requested 5, but only 2 available"
        );
    }

    #[test]
    fn test_coupon_validation_error() {
        let empty_coupon = Coupon::new(String::from("  "), 10);
        assert_eq!(
            empty_coupon.validate(),
            Err(StoreError::InvalidCoupon {
                code: String::from("  "),
                reason: String::from("Coupon code cannot be empty"),
            })
        );

        let excessive_coupon = Coupon::new(String::from("MAX150"), 150);
        assert_eq!(
            excessive_coupon.validate(),
            Err(StoreError::InvalidCoupon {
                code: String::from("MAX150"),
                reason: String::from("Discount percentage 150 must be between 1 and 100"),
            })
        );

        let valid = Coupon::new(String::from("DISC25"), 25);
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_checkout_error_propagation_and_success() {
        let mut catalog = Catalog::new();
        catalog.add_product(Product::new(
            10,
            String::from("PEN-01"),
            String::from("Gel Pen"),
            ProductCategory::OfficeSupplies,
            200,
            4,
        ));

        let customer = Customer::new(5, String::from("Eve"), String::from("e@e.com"), None, false);

        // 1. Empty cart error
        let mut empty_cart = ShoppingCart::new();
        let res = checkout(
            OrderId(1),
            customer.clone(),
            &mut empty_cart,
            &mut catalog,
            PaymentMethod::CashOnDelivery,
            None,
        );
        assert_eq!(res.unwrap_err(), StoreError::EmptyCart);

        // 2. Product not in catalog
        let mut ghost_cart = ShoppingCart::new();
        ghost_cart.add_item(999, 1, 500);
        let res = checkout(
            OrderId(2),
            customer.clone(),
            &mut ghost_cart,
            &mut catalog,
            PaymentMethod::CashOnDelivery,
            None,
        );
        assert_eq!(
            res.unwrap_err(),
            StoreError::ProductNotFound {
                identifier: String::from("ID #999")
            }
        );

        // 3. Insufficient stock error
        let mut big_cart = ShoppingCart::new();
        big_cart.add_item(10, 10, 200); // Catalog only has 4!
        let res = checkout(
            OrderId(3),
            customer.clone(),
            &mut big_cart,
            &mut catalog,
            PaymentMethod::CashOnDelivery,
            None,
        );
        assert_eq!(
            res.unwrap_err(),
            StoreError::InsufficientStock {
                available: 4,
                requested: 10,
            }
        );

        // 4. Successful checkout
        let mut valid_cart = ShoppingCart::new();
        valid_cart.add_item(10, 3, 200);
        let res = checkout(
            OrderId(4),
            customer,
            &mut valid_cart,
            &mut catalog,
            PaymentMethod::CashOnDelivery,
            None,
        );
        assert!(res.is_ok());
        let order = res.unwrap();
        assert_eq!(order.order_id, OrderId(4));
        assert_eq!(catalog.find_by_id(10).unwrap().stock, 1); // 4 - 3 = 1 remaining!
        assert!(valid_cart.is_empty()); // Items moved into order
    }
}
```

---

## Understanding The Code

### 1. The `StoreError` Domain Enum
Instead of scattered strings or generic error codes:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum StoreError {
    InsufficientStock { available: u32, requested: u32 },
    InvalidStateTransition { current: String, action: String },
    ProductNotFound { identifier: String },
    InvalidPayment { reason: String },
    InvalidCoupon { code: String, reason: String },
    EmptyCart,
}
```
Every variant is a self-contained data model for that specific category of failure. For example, `InsufficientStock` holds both `available` and `requested`, allowing user interfaces to say: *"Requested 5 units, but only 2 remain"*.

### 2. Error Propagation in `checkout`
Look closely at the `checkout` function:
```rust
pub fn checkout(...) -> Result<Order, StoreError> {
    if cart.is_empty() {
        return Err(StoreError::EmptyCart);
    }

    if let Some(c) = &coupon {
        c.validate()?;
    }

    for item in &cart.items {
        let product = catalog
            .find_by_id_mut(item.product_id)
            .ok_or_else(|| StoreError::ProductNotFound {
                identifier: format!("ID #{}", item.product_id),
            })?;

        product.reduce_stock(item.quantity)?;
    }

    let items = std::mem::take(&mut cart.items);
    Ok(Order::new(order_id, customer, items, payment, coupon))
}
```
Notice how clean the control flow is:
- `c.validate()?`: If the coupon is invalid, `checkout` returns the `Err` immediately.
- `.ok_or_else(...)?`: If `find_by_id_mut` returns `None`, it is converted into `StoreError::ProductNotFound` and immediately returned.
- `product.reduce_stock(item.quantity)?`: If stock is insufficient, the error is propagated up without any manual `match` boilerplate.

### 3. Taking Cart Items with `std::mem::take`
```rust
let items = std::mem::take(&mut cart.items);
```
`std::mem::take` replaces `cart.items` with an empty `Vec::new()`, taking ownership of the existing items without needing to clone them. The shopping cart is left clean and empty for the next shopping session!

---

## Common Mistakes

### 1. Using `panic!` or `.unwrap()` for Expected Errors
```rust
// ANTI-PATTERN: Panics if warehouse is out of stock!
pub fn buy(product: &mut Product, qty: u32) {
    if qty > product.stock {
        panic!("Out of stock!"); // Crashes the whole web server!
    }
}
```
A shopper requesting more units than available is a **normal, expected business condition**, not an internal software bug. Use `Result<T, StoreError>`.

### 2. Discarding Meaningful Error Information
```rust
// ANTI-PATTERN: Forgetting why the lookup failed
if catalog.find_by_sku(sku).is_none() {
    return Err("Error"); // What error? Which SKU was missing?
}
```
Always attach diagnostic context (such as the requested SKU or product ID) to error variants so loggers and users can diagnose the problem immediately.

### 3. Using `?` in Functions That Do Not Return `Result`
```rust
fn print_stock(product: &mut Product) {
    product.reduce_stock(5)?; // COMPILE ERROR: cannot use ? in a function returning ()
}
```
The `?` operator performs an early `return Err(...)`. If the enclosing function does not return a compatible `Result`, the compiler rejects it.

---

## Compiler Errors

### Error E0277: The `?` Operator Can Only Be Used in Functions That Return `Result`
```rust
fn update_inventory(product: &mut Product) {
    product.reduce_stock(1)?;
}
```
Compiler output:
```text
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option`
 --> src/main.rs:2:28
  |
1 | fn update_inventory(product: &mut Product) {
  | ------------------------------------------ this function should return `Result` or `Option`
2 |     product.reduce_stock(1)?;
  |                            ^ cannot use the `?` operator in a function that returns `()`
```
**Fix**: Update the function's return type to `Result<(), StoreError>`, or handle the error locally with `match` or `if let Err(e) = ...`.

---

## Practice
1. **Payment Amount Validation**:
   Add a method to `PaymentMethod`:
   ```rust
   pub fn validate_amount(&self, amount_cents: u32) -> Result<(), StoreError>
   ```
   If payment is `CashOnDelivery` and amount exceeds $500.00 (50,000 cents), return `Err(StoreError::InvalidPayment { reason: "Cash on delivery limit is $500.00".into() })`.
2. **Catalog Product Lookup with `?`**:
   Write a helper function:
   ```rust
   pub fn verify_product_in_stock(catalog: &Catalog, sku: &str) -> Result<(), StoreError>
   ```
   Combine `.find_by_sku()` with `.ok_or_else()` and `.is_in_stock()` to return `Ok(())` or an appropriate `StoreError`.
3. **Cart Stock Pre-flight Check**:
   Write a method on `ShoppingCart`:
   ```rust
   pub fn verify_all_in_stock(&self, catalog: &Catalog) -> Result<(), StoreError>
   ```
   Iterate over items and return the first out-of-stock error without mutating the catalog.

---

## Checkpoint
Verify all 12 tests pass cleanly:
```bash
cargo test
```
Expected output:
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

Run clippy:
```bash
cargo clippy -- -D warnings
```

---

## What We Learned
- Why Rust treats errors as regular values using `Result<T, E>` rather than invisible exceptions.
- The fundamental difference between expected recoverable failures (`Result`) and code defects (`panic!`).
- How the `?` operator enables clean, readable error propagation without nested pyramids of code.
- Designing custom domain error enums (`StoreError`) with structured diagnostic fields.
- Converting between `Option` and `Result` using `.ok_or()` and `.ok_or_else()`.
- Building end-to-end transactional workflows like `checkout` that safely validate inventory and coupon states.

---

## What's Next
Now that MiniStore has a robust, error-handling domain model, our single `main.rs` file has grown to over 700 lines!
In **Chapter 13: Modules, Packages and Project Structure**, we will learn how Rust organizes large codebases into modules (`mod`), separate files, packages, and crates!
