# Chapter 10: Enums and Pattern Matching

## What You'll Learn
- How **Algebraic Data Types (ADTs)** and Sum Types allow modeling real-world domain states without invalid permutations.
- The three forms of Enum variants: **Unit variants**, **Tuple variants**, and **Struct variants**.
- The internal memory layout of an enum: **Discriminant Tag + Payload Union**.
- **Exhaustive Pattern Matching with `match`**: why the compiler forces you to handle every possible case.
- Advanced pattern matching features: destructuring, field binding, match guards (`if condition`), and wildcards (`_`).
- Ergonomic single-case matching with **`if let`**.
- Adding behavior directly to enums via **`impl` blocks** and methods.
- Applying enums to MiniStore: building an impossible-state-free `OrderStatus` state machine, flexible `PaymentMethod` instruments with fees, and `ProductCategory` classification.

---

## Why Do We Need This?
In [Part I: Ownership](/en/chapters/06-ownership), we mastered memory management, references, slices, and collections.
Now in **Part II: Modeling Business Logic**, our mission shifts from memory mechanics to modeling business domains correctly.

In software, entities have distinct states. Consider an order in MiniStore:
- A new order is **Pending**.
- When paid, it becomes **Confirmed** and has a `receipt_id: String`.
- When shipped, it has a courier `tracking_number: String`.
- If cancelled, it has a `reason: String`.

How would traditional programming languages model this?
Frequently, developers use a struct full of loose booleans and optional fields:

```rust
// ANTI-PATTERN: Full of impossible states!
struct BadOrder {
    is_pending: bool,
    is_confirmed: bool,
    receipt_id: String,
    is_shipped: bool,
    tracking_number: String,
    is_cancelled: bool,
    cancellation_reason: String,
}
```

What happens if a bug sets `is_cancelled = true` AND `is_shipped = true` at the same time? Or `is_shipped = true` but `tracking_number` is empty?
The software enters an **illegal, contradictory state**. Defensive developers write dozens of runtime validation checks (`if is_shipped && is_cancelled { throw ... }`), but humans inevitably miss edge cases.

**Rust's Solution**: **Enums and Pattern Matching**. With an Enum, a value is **strictly ONE variant at any time**. You cannot have an order that is both `Shipped` and `Cancelled`. The type system makes illegal states completely unrepresentable at compile time!

---

## The Problem
Consider an order lifecycle in MiniStore:

```text
               ┌─────────────┐
               │   Pending   │
               └──────┬──────┘
                      │ confirm(receipt_id)
                      ▼
               ┌─────────────┐
               │  Confirmed  │ ──────► cancel(reason) ──► Cancelled
               └──────┬──────┘
                      │ ship(tracking_number)
                      ▼
               ┌─────────────┐
               │   Shipped   │ ──────► Cannot be cancelled!
               └──────┬──────┘
                      │ deliver()
                      ▼
               ┌─────────────┐
               │  Delivered  │ (Terminal State)
               └─────────────┘
```

In languages without algebraic data types:
- **C**: Enums are merely named integer constants (`0, 1, 2`). They cannot hold data payloads like tracking numbers or refund reasons.
- **Java / C#**: Enums can hold fixed properties, but every variant must share the exact same fields. You cannot have `OrderStatus.Shipped` hold a tracking string while `OrderStatus.Delivered` holds nothing, without resorting to complex class hierarchies or nullable fields.
- **Python / Go**: Dynamic typing or interface types (`any` / `interface{}`) allow carrying data, but the compiler does not enforce exhaustive checking when new states are added.

MiniStore needs:
1. To ensure each order state holds exactly the data it needs, and nothing more.
2. A state machine where state transitions (e.g. shipping only confirmed orders) are rigorously enforced.
3. A compiler guarantee that whenever we inspect order state or payment methods, every possible case is explicitly handled.

---

## Rust Concept

### 1. Three Forms of Enum Variants
An enum can combine unit, tuple, and struct variants within a single type:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    // 1. Unit variant: holds no extra data
    Pending,
    Delivered,

    // 2. Struct variant: holds named fields
    Confirmed { receipt_id: String },
    Shipped { tracking_number: String },
    Cancelled { reason: String },
}
```

### 2. Memory Layout: Discriminant + Payload
How does the CPU store an enum in memory?
An enum is represented as a **Tagged Union**:
1. **Discriminant (Tag)**: An integer (usually 1 byte on modern compilers) storing which variant is active (0 for `Pending`, 1 for `Confirmed`, etc.).
2. **Payload Union**: A memory region sized to hold the *largest* variant's data.

```text
Memory layout of OrderStatus:
┌──────────────┬──────────────────────────────────────────────────┐
│ Discriminant │ Payload (Sized to largest variant: String 24B)   │
│ Tag (1 byte) │ e.g. receipt_id / tracking_number / reason       │
└──────────────┴──────────────────────────────────────────────────┘
```
Because the variants share the same payload space (only one is active at a time), an enum is compact and cache-friendly.

### 3. Exhaustive Pattern Matching with `match`
The `match` keyword evaluates an expression and branches execution based on patterns.
The golden rule of `match` in Rust: **It must be exhaustive.**

```rust
fn advisory(status: &OrderStatus) -> &'static str {
    match status {
        OrderStatus::Pending => "Awaiting payment.",
        OrderStatus::Confirmed { .. } => "Ready to pack.",
        OrderStatus::Shipped { .. } => "In transit.",
        OrderStatus::Delivered => "Completed.",
        OrderStatus::Cancelled { .. } => "Halted.",
    }
}
```

If you forget even a single variant, or if you add a new variant (like `Refunded`) in the future, the compiler **refuses to compile** until you handle it! This makes refactoring in Rust extraordinarily safe.

### 4. Pattern Syntax: Destructuring and Match Guards
You can extract values directly from variants during matching:

```rust
match &payment {
    PaymentMethod::CreditCard { last_four } => {
        println!("Charged card ending in {last_four}");
    }
    PaymentMethod::BankTransfer { reference } => {
        println!("Received wire transfer: {reference}");
    }
    PaymentMethod::CashOnDelivery => {
        println!("Collect cash on doorstep");
    }
}
```

You can also add **Match Guards** (`if condition`) to add conditional logic:
```rust
match status {
    OrderStatus::Cancelled { reason } if reason.contains("fraud") => {
        alert_security_team();
    }
    OrderStatus::Cancelled { reason } => {
        restock_inventory();
    }
    _ => {} // Wildcard matches everything else
}
```

### 5. Concise Matching with `if let`
When you only care about one specific variant and want to ignore all others, use `if let`:

```rust
if let OrderStatus::Delivered = order.status {
    println!("Order has arrived!");
}
```

### 6. Methods on Enums
Just like structs, enums can have `impl` blocks with methods taking `&self`, `&mut self`, or `self`:

```rust
impl OrderStatus {
    pub fn can_cancel(&self) -> bool {
        match self {
            Self::Pending | Self::Confirmed { .. } => true,
            Self::Shipped { .. } | Self::Delivered | Self::Cancelled { .. } => false,
        }
    }
}
```

---

## Coming From Other Languages

| Concept | C | Java / C# | Python | TypeScript | Rust |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Enum Payloads** | None (integer only). | Fixed class fields across all variants. | Loose attributes. | Discriminated unions (`type = 'a'`). | **True ADT (Tagged Union) with arbitrary per-variant payloads.** |
| **Exhaustiveness** | Ignored by compiler. | Warning or runtime default. | Handled at runtime (`match-case`). | Compiler checks if `never` type is used. | **Strict compile-time enforcement.** |
| **Memory Efficiency** | 4-byte int. | Full heap objects per variant. | Full dynamic heap dicts. | JavaScript object overhead. | **Tag + union (zero heap allocation for value payloads).** |
| **State Transitions** | Loose booleans or integer flags. | State pattern with multiple classes. | Runtime status strings. | Union types. | **Enum state machine with compile-time transition verification.** |

---

## Small Example
Here is a small example showing data-carrying enums and pattern matching:

```rust
enum WebEvent {
    PageLoad,
    KeyPress(char),
    Click { x: i64, y: i64 },
}

fn inspect(event: WebEvent) {
    match event {
        WebEvent::PageLoad => println!("Page loaded"),
        WebEvent::KeyPress(c) => println!("Key pressed: {c}"),
        WebEvent::Click { x, y } => println!("Clicked at ({x}, {y})"),
    }
}

fn main() {
    inspect(WebEvent::KeyPress('q'));
    inspect(WebEvent::Click { x: 100, y: 250 });
}
```

---

## Apply To MiniStore
In MiniStore:
1. **`OrderStatus` State Machine**: Tracks `Pending`, `Confirmed { receipt_id }`, `Shipped { tracking_number }`, `Delivered`, and `Cancelled { reason }`.
2. **Transition Rules**:
   - `confirm()` only works on `Pending`.
   - `ship()` only works on `Confirmed`.
   - `cancel()` only works on `Pending` or `Confirmed`. Once `Shipped` or `Delivered`, cancellation is mathematically rejected by the type system!
3. **`PaymentMethod` Enum**: `CreditCard { last_four }`, `BankTransfer { reference }`, and `CashOnDelivery` with per-method handling fees.
4. **`ProductCategory`**: Classifies catalog items and calculates department-specific tax rates.

---

## Code
Below is the complete, runnable code for `ministore/src/main.rs`:

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

/// Full Order domain model with state-machine lifecycle enforcement via Enums.
#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub order_id: OrderId,
    pub customer: Customer,
    pub items: Vec<CartItem>,
    pub payment: PaymentMethod,
    pub status: OrderStatus,
}

impl Order {
    pub fn new(
        order_id: OrderId,
        customer: Customer,
        items: Vec<CartItem>,
        payment: PaymentMethod,
    ) -> Self {
        Self {
            order_id,
            customer,
            items,
            payment,
            status: OrderStatus::Pending,
        }
    }

    /// Transitions Pending -> Confirmed { receipt_id }.
    pub fn confirm(&mut self, receipt_id: String) -> Result<(), &'static str> {
        match &self.status {
            OrderStatus::Pending => {
                self.status = OrderStatus::Confirmed { receipt_id };
                Ok(())
            }
            OrderStatus::Confirmed { .. } => Err("Order is already confirmed"),
            OrderStatus::Shipped { .. } => Err("Cannot confirm an order that is already shipped"),
            OrderStatus::Delivered => Err("Cannot confirm a delivered order"),
            OrderStatus::Cancelled { .. } => Err("Cannot confirm a cancelled order"),
        }
    }

    /// Transitions Confirmed -> Shipped { tracking_number }.
    pub fn ship(&mut self, tracking_number: String) -> Result<(), &'static str> {
        match &self.status {
            OrderStatus::Confirmed { .. } => {
                self.status = OrderStatus::Shipped { tracking_number };
                Ok(())
            }
            OrderStatus::Pending => Err("Order must be confirmed before shipping"),
            OrderStatus::Shipped { .. } => Err("Order is already shipped"),
            OrderStatus::Delivered => Err("Order is already delivered"),
            OrderStatus::Cancelled { .. } => Err("Cannot ship a cancelled order"),
        }
    }

    /// Transitions to Delivered.
    pub fn mark_delivered(&mut self) -> Result<(), &'static str> {
        match &self.status {
            OrderStatus::Shipped { .. } => {
                self.status = OrderStatus::Delivered;
                Ok(())
            }
            OrderStatus::Pending | OrderStatus::Confirmed { .. } => {
                Err("Order must be shipped before delivery")
            }
            OrderStatus::Delivered => Err("Order is already marked delivered"),
            OrderStatus::Cancelled { .. } => Err("Cannot deliver a cancelled order"),
        }
    }

    /// Cancels order if current state permits.
    pub fn cancel(&mut self, reason: String) -> Result<(), &'static str> {
        if self.status.can_cancel() {
            self.status = OrderStatus::Cancelled { reason };
            Ok(())
        } else {
            Err("Order cannot be cancelled in its current state")
        }
    }

    pub fn subtotal_cents(&self) -> u32 {
        self.items.iter().map(|item| item.line_total()).sum()
    }

    pub fn total_cents(&self) -> u32 {
        let subtotal = self.subtotal_cents();
        let discount = calculate_discount(&self.customer, subtotal);
        subtotal - discount + self.payment.fee_cents()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculates discount based on customer VIP status or tags.
pub fn calculate_discount(customer: &Customer, subtotal_cents: u32) -> u32 {
    if customer.is_vip || customer.has_tag("vip") {
        (subtotal_cents * 10) / 100 // 10% discount for VIPs
    } else {
        0
    }
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
    println!("=== MiniStore: Enums & Pattern Matching (Part II) ===\n");

    // 1. Enums with Data Payloads (Tagged Unions)
    println!("1. Modeling Payments with Data-Carrying Enums:");
    let card_payment = PaymentMethod::CreditCard {
        last_four: String::from("4242"),
    };
    let cod_payment = PaymentMethod::CashOnDelivery;

    println!(
        "   Method: {} (Fee: ${:.2})",
        card_payment.description(),
        card_payment.fee_cents() as f64 / 100.0
    );
    println!(
        "   Method: {} (Fee: ${:.2})",
        cod_payment.description(),
        cod_payment.fee_cents() as f64 / 100.0
    );

    // 2. Products with Category Enums
    let keyboard = Product::new(
        101,
        String::from("TECH-KEY-001"),
        String::from("Tenkeyless Mechanical Keyboard"),
        ProductCategory::Electronics,
        12000,
        15,
    );
    println!("\n2. Product Category Classification:");
    println!(
        "   Product: {} | Category: {:?} | Tax Rate: {}%",
        keyboard.name,
        keyboard.category,
        keyboard.category.default_tax_rate()
    );

    // 3. Order Lifecycle State Machine
    println!("\n3. Order State Machine & Transitions:");
    let mut customer = Customer::new(
        301,
        String::from("Margaret Hamilton"),
        String::from("margaret@apollo.nasa.gov"),
        true,
    );
    customer.add_tag("pioneer");

    let mut cart = ShoppingCart::new();
    cart.add_item(keyboard.id, 2, keyboard.price_cents);

    let mut order = Order::new(
        OrderId(901),
        customer,
        cart.items,
        PaymentMethod::CreditCard {
            last_four: String::from("9876"),
        },
    );

    println!("   Initial Status: {}", order.status.display_status());
    println!(
        "   Advisory:       {}",
        order_dispatch_advisory(&order.status)
    );
    println!("   Can Cancel?     {}", order.status.can_cancel());

    // Transition 1: Confirm order
    order.confirm(String::from("REC-901- Hamilton")).unwrap();
    println!("\n   After Confirm:  {}", order.status.display_status());
    println!(
        "   Advisory:       {}",
        order_dispatch_advisory(&order.status)
    );

    // Transition 2: Ship order
    order.ship(String::from("TRK-USPS-774921")).unwrap();
    println!("\n   After Shipping: {}", order.status.display_status());
    println!(
        "   Advisory:       {}",
        order_dispatch_advisory(&order.status)
    );
    println!("   Can Cancel?     {}", order.status.can_cancel());

    // Attempting invalid transition: cannot cancel once shipped!
    let cancel_result = order.cancel(String::from("Customer changed mind"));
    println!(
        "   Attempt Cancel: Failed as expected -> {:?}",
        cancel_result.unwrap_err()
    );

    // Transition 3: Mark delivered
    order.mark_delivered().unwrap();
    println!("\n   After Delivery: {}", order.status.display_status());
    println!("   Is Terminal?    {}", order.status.is_terminal());

    // 4. Pattern Matching with 'if let'
    if let OrderStatus::Delivered = order.status {
        println!("\n4. 'if let' Pattern Match: Order was safely delivered!");
    }

    println!(
        "\nTotal Paid: ${:.2} (Subtotal: ${:.2}, 10% VIP Discount, + ${:.2} Card Fee)",
        order.total_cents() as f64 / 100.0,
        order.subtotal_cents() as f64 / 100.0,
        order.payment.fee_cents() as f64 / 100.0
    );
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
        let customer = Customer::new(1, String::from("Alice"), String::from("a@a.com"), false);
        let items = vec![CartItem::new(10, 1, 5000)];
        let mut order = Order::new(OrderId(100), customer, items, PaymentMethod::CashOnDelivery);

        // Starts pending
        assert_eq!(order.status, OrderStatus::Pending);
        assert!(order.status.can_cancel());
        assert!(!order.status.is_terminal());

        // Cannot ship while pending
        assert!(order.ship(String::from("TRK-1")).is_err());

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
        let customer = Customer::new(2, String::from("Bob"), String::from("b@b.com"), false);
        let items = vec![CartItem::new(20, 2, 2500)];
        let mut order = Order::new(OrderId(200), customer, items, PaymentMethod::CashOnDelivery);

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
        assert!(order.confirm(String::from("REC-200")).is_err());
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
        let mut customer = Customer::new(3, String::from("Carol"), String::from("c@c.com"), false);
        customer.upgrade_to_vip(); // 10% discount

        let items = vec![CartItem::new(1, 1, 10000)]; // $100.00 subtotal
        let order = Order::new(
            OrderId(300),
            customer,
            items,
            PaymentMethod::CreditCard {
                last_four: String::from("1111"),
            }, // $1.50 (150 cents) fee
        );

        // Subtotal: 10000 cents
        // VIP discount: 1000 cents
        // Card fee: 150 cents
        // Total: 10000 - 1000 + 150 = 9150 cents ($91.50)
        assert_eq!(order.total_cents(), 9150);
    }
}
```

---

## Understanding The Code

### 1. State Transitions Enforced by `match`
Look at `Order::ship`:
```rust
match &self.status {
    OrderStatus::Confirmed { .. } => {
        self.status = OrderStatus::Shipped { tracking_number };
        Ok(())
    }
    OrderStatus::Pending => Err("Order must be confirmed before shipping"),
    OrderStatus::Shipped { .. } => Err("Order is already shipped"),
    OrderStatus::Delivered => Err("Order is already delivered"),
    OrderStatus::Cancelled { .. } => Err("Cannot ship a cancelled order"),
}
```
Only a `Confirmed` order can transition to `Shipped`. Attempting to ship a `Pending`, `Delivered`, or `Cancelled` order returns a descriptive error. State transitions are deterministic and protected against human error.

### 2. Multi-Case Matching (`|` Operator)
In `can_cancel`:
```rust
match self {
    Self::Pending | Self::Confirmed { .. } => true,
    Self::Shipped { .. } | Self::Delivered | Self::Cancelled { .. } => false,
}
```
Rust allows combining multiple patterns on a single arm using the pipe `|` operator, keeping match expressions concise without losing exhaustiveness.

---

## Common Mistakes

### 1. Forgetting a Variant in `match`
```rust
match payment {
    PaymentMethod::CreditCard { .. } => 150,
    PaymentMethod::CashOnDelivery => 300,
    // COMPILER ERROR: non-exhaustive patterns: `PaymentMethod::BankTransfer { .. }` not covered
}
```
Rust forces you to handle every variant. If you add a new payment method (like `Crypto` or `PayPal`), the compiler immediately identifies every single match expression in your codebase that needs updating.

### 2. Overusing `_` Wildcards Carelessly
While `_ => ...` satisfies exhaustiveness, using it prematurely can cause bugs: when you add a new variant, the wildcard will silently swallow it without prompting you to write specific handling code. Use specific variant patterns whenever possible.

---

## Compiler Errors

### Error E0004: Non-Exhaustive Pattern Match
Suppose you write:

```rust
let status = OrderStatus::Pending;
match status {
    OrderStatus::Pending => println!("Pending"),
    OrderStatus::Confirmed { .. } => println!("Confirmed"),
}
```

The Rust compiler halts compilation:

```text
error[E0004]: non-exhaustive patterns: `OrderStatus::Shipped { .. }`, `OrderStatus::Delivered` and `OrderStatus::Cancelled { .. }` not covered
  --> src/main.rs:210:11
   |
210|     match status {
   |           ^^^^^^ patterns `OrderStatus::Shipped { .. }`, `OrderStatus::Delivered` and `OrderStatus::Cancelled { .. }` not covered
```

This compiler guarantee prevents unhandled state bugs from ever reaching production.

---

## Practice
1. **Add a Variant**: Add a new variant `Refunded { refund_receipt_id: String }` to `OrderStatus`.
2. **Handle the Variant**: Update `display_status`, `can_cancel`, `is_terminal`, and `order_dispatch_advisory` to handle `Refunded`.
3. **Refund Method**: Implement a method `refund(&mut self, refund_receipt_id: String) -> Result<(), &'static str>` on `Order` that only permits refunding orders that have been `Delivered` or `Cancelled`.
4. Run `cargo test` to verify your implementation.

---

## Checkpoint
- [x] Replaced loose boolean flags with impossible-state-free Algebraic Data Types (Enums).
- [x] Defined unit variants, tuple variants, and struct variants with custom data payloads.
- [x] Understood the Tagged Union memory layout (Discriminant + Payload).
- [x] Mastered exhaustive pattern matching with `match`, field destructuring, and match guards.
- [x] Used `if let` for concise single-variant handling.
- [x] Built a robust, verified Order lifecycle state machine in MiniStore.

---

## What We Learned
- Enums in Rust are algebraic sum types, capable of carrying heterogeneous payloads with zero heap allocation.
- The compiler enforces exhaustive pattern matching, eliminating unhandled state vulnerabilities at compile time.
- MiniStore now models orders, payment methods, and product categories using rigorous, type-safe state machines.

---

## What's Next
What if a function might not find what you are looking for (like looking up an item in a map)? In traditional languages, this returns `null`, causing the dreaded NullPointerException. In [Chapter 11: Option](/en/chapters/11-option), we will explore Rust's safe alternative: the `Option<T>` enum.
