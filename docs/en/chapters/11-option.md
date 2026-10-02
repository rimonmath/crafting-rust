# Chapter 11: Option — The Billion-Dollar Mistake Solved

## What You'll Learn
- Why computer science pioneer Sir Tony Hoare called `null` references his **"billion-dollar mistake"**.
- How Rust eliminates `null` pointers entirely and replaces them with the **`Option<T>`** enum.
- The standard library definition: **`Some(T)`** vs **`None`**.
- The critical distinction between types that are always present (`T`) and types that might be absent (`Option<T>`).
- Rust's zero-cost memory optimization: the **Null Pointer Optimization (NPO)**.
- Safe extraction and pattern matching: `match`, `if let`, and why **`.unwrap()`** is dangerous in production.
- Ergonomic standard combinators: **`.unwrap_or()`**, **`.unwrap_or_else()`**, **`.map()`**, **`.and_then()`**, and **`.as_ref()`**.
- In-place mutation helpers: **`.take()`** and **`.replace()`**.
- Applying `Option<T>` to MiniStore:
  - Safe O(1) catalog search lookups (`find_by_sku`, `find_by_id`).
  - Optional customer profile fields (`phone: Option<String>`).
  - Safe cart item retrieval (`get_item`, `get_item_mut`).
  - Optional promotional coupons (`coupon: Option<Coupon>`) and dynamic discount calculations.

---

## Why Do We Need This?
In 1965, British computer scientist Sir Tony Hoare invented the `null` reference while designing the type system for the ALGOL W language. Decades later, at a software conference in 2009, Hoare delivered a famous public apology:

> *"I call it my billion-dollar mistake. It was the invention of the null reference in 1965... At that time, I was designing the first comprehensive type system for references in an object-oriented language. My goal was to ensure that all use of references should be absolutely safe, with checking performed automatically by the compiler. But I couldn't resist the temptation to put in a null reference, simply because it was so easy to implement. This has led to innumerable errors, vulnerabilities, and system crashes, which have probably caused a billion dollars of pain and damage in the last forty years."*

Why was `null` such an expensive mistake?
In languages like C, C++, Java, C#, Python, and JavaScript:
- **Every reference or object variable can secretly be null at any time.**
- If you declare a variable `Customer customer`, nothing in the type signature tells you whether `customer` points to a real customer in memory or `null`.
- If your code attempts to access `customer.email` when the value happens to be `null`, your application crashes immediately at runtime:
  - Java: `java.lang.NullPointerException`
  - JavaScript: `TypeError: Cannot read properties of null`
  - Python: `AttributeError: 'NoneType' object has no attribute 'email'`
  - C / C++: `Segmentation fault (core dumped)`

Developers spend countless hours writing defensive boilerplate checks:
```java
// Traditional defensive programming in Java / C#
if (customer != null) {
    if (customer.getAddress() != null) {
        if (customer.getAddress().getCity() != null) {
            // finally safe to use city...
        }
    }
}
```
If a developer forgets a single check anywhere in a codebase of hundreds of thousands of lines, the program compiles without complaint and crashes in production in front of a real customer.

**Rust's Solution**: **Rust has no `null`, no `nil`, and no null pointers in safe code.**

If a variable in Rust has type `String`, it is **100% guaranteed by the compiler to contain a valid string**. It can NEVER be null.

When absence of data is possible (for example, looking up a product that might not exist, or a customer who opted not to provide a phone number), Rust forces you to use the **`Option<T>`** type. You cannot use the inner value without explicitly handling the possibility that it is absent.

---

## The Problem
Consider how MiniStore interacts with data:

```text
1. Catalog Lookups:
   catalog.find_by_sku("TECH-KEY-001") ──► Product exists! Return product.
   catalog.find_by_sku("NON-EXISTENT")  ──► Missing! How should we signal this?

2. Customer Profiles:
   Customer Alice: Has phone "+1-555-0199"
   Customer Bob:   Declined to provide a phone number.

3. Checkout Discounts:
   Order 901: Customer enters coupon code "LAUNCH20" (20% off).
   Order 902: Customer checks out without any coupon.

4. Shopping Cart Lookups:
   cart.get_item(101) ──► Is product #101 already in the cart?
```

In languages with `null`:
- If `find_by_sku` returns `null` on a cache miss, any downstream function computing shipping costs or printing an invoice will crash if it forgets an `if (product != null)` check.
- If a customer has no phone number, developers often store an empty string `""` or a dummy value like `"N/A"`. But is `""` an absent phone number or an invalid phone number? Sentinel values cause subtle bugs.
- If an order has no coupon, storing `null` requires manual null checks before calculating the final invoice amount.

MiniStore needs:
1. A way to represent the presence or absence of a value explicitly in the type system.
2. A guarantee that callers cannot access an optional value without safely handling the missing case.
3. Zero runtime memory penalty compared to raw pointers.
4. Clean, readable methods to transform and extract optional values without repetitive nested `if` statements.

---

## Rust Concept

### 1. The `Option<T>` Definition
In Rust's standard library, `Option<T>` is simply an enum:

```rust
pub enum Option<T> {
    None,
    Some(T),
}
```

Notice:
- It has two variants:
  - `None`: Indicates that no value is present.
  - `Some(T)`: Wraps a value of type `T`.
- It is generic over type `T`. That means you can have `Option<u32>`, `Option<String>`, `Option<Product>`, or `Option<&Product>`.
- Both `Option`, `Some`, and `None` are included in the Rust **prelude**. You do not need to import them with `use std::option::Option;`—they are directly available in every Rust file!

### 2. The Type Separation: `T` vs `Option<T>`
In Rust, `String` and `Option<String>` are completely distinct types:

```rust
let name: String = String::from("Alice");
let maybe_phone: Option<String> = Some(String::from("+1-555-0199"));
let no_phone: Option<String> = None;
```

If you try to treat an `Option<String>` as a `String`:

```rust
fn print_length(s: String) {
    println!("Length: {}", s.len());
}

print_length(maybe_phone); // COMPILE ERROR!
```

The Rust compiler stops you instantly:
```text
error[E0308]: mismatched types
  --> src/main.rs
   |
   | print_length(maybe_phone);
   |              ^^^^^^^^^^^ expected `String`, found `Option<String>`
```
To access the inner `String`, you **must** unpack the `Option`. The compiler physically prevents you from forgetting!

### 3. Memory Layout: The Null Pointer Optimization (NPO)
You might wonder: *Does wrapping every reference in `Option<&T>` add extra memory overhead for a tag byte?*

In [Chapter 10](/en/chapters/10-enums-and-pattern-matching), we saw that general enums use a discriminant tag plus payload union. However, for references (`&T`, `&mut T`), pointers, and types like `Box<T>`, Rust implements the **Null Pointer Optimization (NPO)**:

```text
Standard Reference (&Product):
┌───────────────────────────────┐
│ Pointer Address (8 bytes)     │  Cannot be 0x0 (valid pointer)
└───────────────────────────────┘

Option<&Product>:
┌───────────────────────────────┐
│ Some(&p) = Pointer Address    │  Valid memory address
│ None     = 0x0000000000000000 │  All zeros represents None!
└───────────────────────────────┘
```

Because safe references in Rust can **never** be address `0x0`, the compiler uses address `0x0` internally to represent `None`.
Therefore:
- `std::mem::size_of::<&Product>() == 8 bytes`
- `std::mem::size_of::<Option<&Product>>() == 8 bytes`

You get 100% compile-time safety with **zero memory overhead and zero runtime performance penalty**.

### 4. Unpacking `Option`: The Ways to Handle Missing Data

#### A. Exhaustive Matching with `match`
The most fundamental way to handle an `Option` is pattern matching:

```rust
match catalog.find_by_sku("TECH-KEY-001") {
    Some(product) => println!("Product found: {}", product.name),
    None => println!("Product does not exist in catalog"),
}
```
Because `match` is exhaustive, the compiler refuses to build if you forget the `None` arm.

#### B. Single-Case Handling with `if let`
When you only care about the present case:

```rust
if let Some(item) = cart.get_item(101) {
    println!("Quantity in cart: {}", item.quantity);
}
```

#### C. Fallback Values with `.unwrap_or()` and `.unwrap_or_else()`
Instead of a verbose `match`, provide a default value if `None`:

```rust
// If catalog.product_price() returns None, fallback to 0
let price = catalog.product_price("UNKNOWN-SKU").unwrap_or(0);

// For expensive fallbacks, compute lazily with a closure:
let discount = order.coupon.as_ref()
    .map(|c| c.discount_percent)
    .unwrap_or_else(|| calculate_default_discount());
```

#### D. Transforming with `.map()`
If you want to apply a function or field extraction to the inner value *if it exists*, use `.map()`:

```rust
// find_by_sku returns Option<&Product>
// .map(|p| p.price_cents) transforms it into Option<u32>
let price: Option<u32> = catalog.find_by_sku("TECH-KEY-001").map(|p| p.price_cents);
```
If the product is `Some(p)`, `.map` returns `Some(p.price_cents)`.
If the product is `None`, `.map` returns `None` immediately without doing anything.

#### E. Chaining Operations with `.and_then()`
If your transformation function itself returns an `Option`, `.map()` would produce a nested `Option<Option<T>>`. To flatten it, use `.and_then()`:

```rust
// find_order -> Option<Order>
// order.customer_phone -> Option<String>
let area_code = find_order(id)
    .and_then(|order| order.customer.phone)
    .map(|phone| phone[..3].to_string());
```

#### F. Borrowing the Inner Value: `.as_ref()` and `.as_deref()`
If you have an `&Option<T>` and want an `Option<&T>` so you don't take ownership of the inner value:

```rust
let opt: Option<String> = Some(String::from("Hello"));

// opt.as_ref() gives Option<&String>
let len = opt.as_ref().map(|s| s.len());

// opt.as_deref() gives Option<&str> (coerces &String to &str)
let slice: Option<&str> = opt.as_deref();
```

#### G. Extracting and Resetting: `.take()`
`.take()` takes the value out of the `Option`, leaving a `None` in its place:

```rust
let mut coupon = Some(Coupon::new(String::from("SAVE10"), 10));

let extracted = coupon.take(); // extracted is Some(Coupon)
println!("{:?}", coupon);      // coupon is now None!
```
This is a game-changer when modifying struct fields: you can extract an owned value out of a mutable reference `&mut self` without cloning!

---

## Coming From Other Languages

| Concept | Rust | Java / C# | TypeScript / JavaScript | Python | Go |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Missing value** | `Option::None` | `null` | `null` / `undefined` | `None` | `nil` |
| **Present value** | `Option::Some(val)` | Direct value | Direct value | Direct value | Pointer / value |
| **Compiler enforcement** | **Mandatory** (`T` cannot be `null`) | Optional (`@Nullable`, `Optional<T>`) | Optional (`strictNullChecks`) | None (runtime duck typing) | None |
| **Memory layout** | Null Pointer Optimization (0-cost) | Extra object allocation (`Optional<T>`) | Dynamic type tag | Heap object (`NoneType`) | 8-byte pointer |
| **Missing value crash** | **Impossible** in safe code | `NullPointerException` | `TypeError: Cannot read property of undefined` | `AttributeError: 'NoneType'` | `panic: nil pointer dereference` |
| **Value transformation** | `opt.map(f)` | `opt.map(f)` | Optional chaining `val?.prop` | `f(val) if val else None` | `if val != nil { ... }` |

---

## Small Example
Let's see `Option<T>`, pattern matching, combinators, and `.take()` in a standalone, copy-pasteable script:

```rust
#[derive(Debug, PartialEq)]
struct UserProfile {
    username: String,
    phone: Option<String>,
}

fn main() {
    let alice = UserProfile {
        username: String::from("alice_dev"),
        phone: Some(String::from("+1-555-0149")),
    };

    let bob = UserProfile {
        username: String::from("bob_builder"),
        phone: None,
    };

    // 1. Pattern matching with match
    match &alice.phone {
        Some(phone) => println!("Alice's phone: {phone}"),
        None => println!("Alice has no phone on record"),
    }

    // 2. Using .as_deref() and .unwrap_or()
    println!("Bob's phone: {}", bob.phone.as_deref().unwrap_or("Unlisted"));

    // 3. Transforming with .map()
    let phone_len: Option<usize> = alice.phone.as_ref().map(|p| p.len());
    println!("Alice's phone number length: {:?}", phone_len);

    // 4. Modifying with .take()
    let mut mutable_bob_phone = Some(String::from("+1-555-9999"));
    let extracted = mutable_bob_phone.take();
    println!("Extracted phone: {:?}", extracted);
    println!("Remaining phone: {:?}", mutable_bob_phone); // None
}
```

---

## Apply To MiniStore
In MiniStore, we integrate `Option<T>` across the domain:

```text
┌─────────────────────────────────────────────────────────────┐
│                           Catalog                           │
├─────────────────────────────────────────────────────────────┤
│ - products: HashMap<String, Product>                        │
├─────────────────────────────────────────────────────────────┤
│ + find_by_sku(&self, sku: &str) -> Option<&Product>        │
│ + find_by_id(&self, id: u64) -> Option<&Product>           │
│ + product_price(&self, sku: &str) -> Option<u32>            │
│ + is_product_in_stock(&self, sku: &str) -> bool             │
└─────────────────────────────────────────────────────────────┘
                               │ returns
                               ▼
                        Option<&Product>
                         ├── Some(&Product) ──► Inspect stock, price, SKU
                         └── None           ──► Handled gracefully, zero crashes!

┌─────────────────────────────────────────────────────────────┐
│                          Customer                           │
├─────────────────────────────────────────────────────────────┤
│ - phone: Option<String>                                     │
│ + formatted_phone(&self) -> &str                            │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                            Order                            │
├─────────────────────────────────────────────────────────────┤
│ - coupon: Option<Coupon>                                    │
│ + apply_coupon(&mut self, coupon: Coupon)                   │
│ + remove_coupon(&mut self) -> Option<Coupon>  [.take()]    │
│ + total_cents(&self) -> u32 (applies optional coupon)       │
└─────────────────────────────────────────────────────────────┘
```

1. **`Catalog`**:
   - `find_by_sku(&self, sku: &str) -> Option<&Product>` borrows the product directly from the underlying `HashMap`.
   - `product_price(&self, sku: &str) -> Option<u32>` uses `.map(|p| p.price_cents)` to cleanly extract the price without unwrapping.
   - `is_product_in_stock(&self, sku: &str) -> bool` uses `.map(|p| p.is_in_stock()).unwrap_or(false)`.
2. **`Customer`**:
   - `phone: Option<String>` models optional customer phone numbers.
   - `formatted_phone(&self) -> &str` uses `self.phone.as_deref().unwrap_or("Unspecified")`.
3. **`ShoppingCart`**:
   - `get_item(&self, product_id: u64) -> Option<&CartItem>`
   - `get_item_mut(&mut self, product_id: u64) -> Option<&mut CartItem>`
   - `add_item` uses `get_item_mut`: if `Some(item)` is found, it increments `item.quantity`; if `None`, it pushes a new item.
4. **`Coupon` and `Order`**:
   - An order optionally has a coupon: `pub coupon: Option<Coupon>`.
   - Discount calculation inspects `self.coupon.as_ref()`.
   - `remove_coupon(&mut self) -> Option<Coupon>` uses `self.coupon.take()` to remove the coupon in place.

---

## Code
Here is the complete, production-grade MiniStore codebase for Chapter 11. You can find this snapshot in `examples/chapter-11/src/main.rs`:

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

    pub fn find_by_id(&self, id: u64) -> Option<&Product> {
        self.products.values().find(|product| product.id == id)
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
        let discount = calculate_discount(&self.customer, self.coupon.as_ref(), subtotal);
        subtotal.saturating_sub(discount) + self.payment.fee_cents()
    }
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
    println!("=== MiniStore: Option & Safe Error-Free Design (Part II) ===\n");

    // 1. Safe Lookups: Catalog returning Option<&Product>
    println!("1. Safe Catalog Lookups (Option<&T>):");
    let mut catalog = Catalog::new();

    let keyboard = Product::new(
        101,
        String::from("TECH-KEY-001"),
        String::from("Tenkeyless Mechanical Keyboard"),
        ProductCategory::Electronics,
        12000,
        15,
    );
    let mouse = Product::new(
        102,
        String::from("TECH-MOU-002"),
        String::from("Ergonomic Wireless Mouse"),
        ProductCategory::Electronics,
        4500,
        0, // Out of stock
    );

    catalog.add_product(keyboard.clone());
    catalog.add_product(mouse);

    // Look up an existing product
    match catalog.find_by_sku("TECH-KEY-001") {
        Some(product) => println!(
            "   Found SKU TECH-KEY-001: {} ({}, In stock: {})",
            product.name,
            product.formatted_price(),
            product.is_in_stock()
        ),
        None => println!("   SKU TECH-KEY-001 not found!"),
    }

    // Look up a non-existent product - No NullPointerException!
    match catalog.find_by_sku("NON-EXISTENT-SKU") {
        Some(product) => println!("   Found unexpected product: {}", product.name),
        None => println!("   Safely handled missing SKU 'NON-EXISTENT-SKU': returned None"),
    }

    // 2. Option Combinators: .map() and .unwrap_or()
    println!("\n2. Transforming with Combinators (.map, .unwrap_or):");
    let key_price = catalog.product_price("TECH-KEY-001").unwrap_or(0);
    let missing_price = catalog.product_price("MISSING-SKU").unwrap_or(0);
    let is_in_stock = catalog.is_product_in_stock("TECH-MOU-002");

    println!("   TECH-KEY-001 Price: ${:.2}", key_price as f64 / 100.0);
    println!(
        "   MISSING-SKU  Price: ${:.2} (defaulted)",
        missing_price as f64 / 100.0
    );
    println!("   TECH-MOU-002 In Stock: {is_in_stock}");

    // 3. Optional Fields: Customer Phone
    println!("\n3. Modeling Optional Fields (Customer Phone):");
    let customer_with_phone = Customer::new(
        301,
        String::from("Margaret Hamilton"),
        String::from("margaret@apollo.nasa.gov"),
        Some(String::from("+1-555-0199")),
        true,
    );
    let customer_no_phone = Customer::new(
        302,
        String::from("Grace Hopper"),
        String::from("grace@navy.mil"),
        None,
        false,
    );

    println!(
        "   Customer 1: {} | Phone: {}",
        customer_with_phone.name,
        customer_with_phone.formatted_phone()
    );
    println!(
        "   Customer 2: {} | Phone: {}",
        customer_no_phone.name,
        customer_no_phone.formatted_phone()
    );

    // 4. ShoppingCart Lookups: get_item
    println!("\n4. ShoppingCart Lookups returning Option<&CartItem>:");
    let mut cart = ShoppingCart::new();
    cart.add_item(keyboard.id, 2, keyboard.price_cents);

    if let Some(item) = cart.get_item(101) {
        println!(
            "   Found item in cart: Product ID {} x {} units = ${:.2}",
            item.product_id,
            item.quantity,
            item.line_total() as f64 / 100.0
        );
    }

    // 5. Orders with Optional Coupon Discounts & .take():
    println!("\n5. Orders with Optional Coupon Discounts & .take():");
    let mut order = Order::new(
        OrderId(901),
        customer_with_phone,
        cart.items,
        PaymentMethod::CreditCard {
            last_four: String::from("9876"),
        },
        None, // Created initially without a coupon
    );

    println!(
        "   Before Coupon Total: ${:.2} (Subtotal: ${:.2}, 10% VIP, +$1.50 Card Fee)",
        order.total_cents() as f64 / 100.0,
        order.subtotal_cents() as f64 / 100.0
    );

    // Apply coupon (e.g. 20% discount)
    order.apply_coupon(Coupon::new(String::from("LAUNCH20"), 20));
    println!(
        "   After Coupon Applied: Coupon is {:?}",
        order.coupon.as_ref().map(|c| &c.code)
    );
    println!(
        "   With Coupon Total:    ${:.2} (Subtotal: ${:.2}, 10% VIP + 20% Coupon = 30% discount)",
        order.total_cents() as f64 / 100.0,
        order.subtotal_cents() as f64 / 100.0
    );

    // Removing coupon using .take()
    let removed_coupon = order.remove_coupon();
    println!(
        "   Removed Coupon using .take(): {:?}",
        removed_coupon.map(|c| c.code)
    );
    println!("   Order coupon is now None: {}", order.coupon.is_none());
    println!(
        "   Total reverted to: ${:.2}",
        order.total_cents() as f64 / 100.0
    );

    // 6. Order Lifecycle Progression
    order.confirm(String::from("REC-901-HAMILTON")).unwrap();
    order.ship(String::from("TRK-USPS-774921")).unwrap();
    order.mark_delivered().unwrap();
    println!("\n   Final Order Status: {}", order.status.display_status());
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

        // Subtotal: 20000
        // Coupon 15%: 3000
        // Bank transfer fee: 0
        // Total: 17000 cents ($170.00)
        assert_eq!(order.total_cents(), 17000);

        // Remove coupon with .take()
        let extracted = order.remove_coupon();
        assert_eq!(
            extracted,
            Some(Coupon {
                code: String::from("SAVE15"),
                discount_percent: 15
            })
        );
        assert_eq!(order.coupon, None);

        // Total now reverts to full subtotal: 20000 cents
        assert_eq!(order.total_cents(), 20000);
    }
}
```

---

## Understanding The Code

### 1. Catalog Lookups Returning `Option<&Product>`
In `Catalog::find_by_sku`:
```rust
pub fn find_by_sku(&self, sku: &str) -> Option<&Product> {
    self.products.get(sku)
}
```
`HashMap::get` naturally returns `Option<&V>`. It borrows from `&self` without cloning any `Product` data. If the SKU exists in the table, the caller receives `Some(&product)`. If not, `None`. The caller is forced by the compiler to acknowledge the possibility of a missing product.

### 2. Concise Transformations with Combinators
Consider `product_price`:
```rust
pub fn product_price(&self, sku: &str) -> Option<u32> {
    self.find_by_sku(sku).map(|p| p.price_cents)
}
```
Without `.map()`, we would have had to write:
```rust
match self.find_by_sku(sku) {
    Some(p) => Some(p.price_cents),
    None => None,
}
```
`.map()` eliminates 4 lines of boilerplate. Furthermore, callers can chain `.unwrap_or(0)`:
```rust
let price = catalog.product_price("TECH-KEY-001").unwrap_or(0);
```

### 3. Borrowing with `as_deref()`
In `Customer::formatted_phone`:
```rust
pub fn formatted_phone(&self) -> &str {
    self.phone.as_deref().unwrap_or("Unspecified")
}
```
- `self.phone` is `Option<String>`.
- `self.phone.as_deref()` converts `&Option<String>` to `Option<&str>`.
- `.unwrap_or("Unspecified")` returns the `&str` slice if `Some`, or the static string slice `"Unspecified"` if `None`.
Both branches evaluate to `&str` without allocating a single byte on the heap!

### 4. Taking Ownership Out of a Reference with `.take()`
In `Order::remove_coupon`:
```rust
pub fn remove_coupon(&mut self) -> Option<Coupon> {
    self.coupon.take()
}
```
If you tried:
```rust
let old = self.coupon; // COMPILE ERROR: cannot move out of `self.coupon` behind `&mut self`
self.coupon = None;
```
Rust's borrow checker rejects this because leaving `self.coupon` momentarily uninitialized violates memory safety. `self.coupon.take()` atomically replaces the value with `None` and returns the old value wrapped in `Option<Coupon>`—completely avoiding expensive clones.

---

## Common Mistakes

### 1. Calling `.unwrap()` in Production Code
```rust
// ANTI-PATTERN: Will panic if the SKU is missing!
let product = catalog.find_by_sku("PROD-999").unwrap();
```
`unwrap()` is acceptable in quick prototypes or unit tests where failure indicates a broken test assertion. In production business logic, **never** unwrap unless you have mathematically proven it cannot be `None`. Use `match`, `if let`, `.unwrap_or()`, or `?` (which we will learn in Chapter 12).

### 2. Moving Out of an `Option` Behind a Shared Reference
```rust
fn print_customer_phone(customer: &Customer) {
    // ERROR: cannot move out of `customer.phone`
    match customer.phone {
        Some(p) => println!("{p}"),
        None => println!("None"),
    }
}
```
`customer` is borrowed (`&Customer`). Matching on `customer.phone` attempts to move the `String` out of the struct!
**Fix**: Match by reference (`match &customer.phone` or `customer.phone.as_ref()`):
```rust
match &customer.phone {
    Some(p) => println!("{p}"),
    None => println!("None"),
}
```

### 3. Using Magic Sentinel Values Instead of `Option`
```rust
// ANTI-PATTERN: Using "" or -1 to signal absence
struct BadCustomer {
    phone: String, // "" means no phone
}
```
Magic strings and negative numbers bypass the compiler. Other developers will forget to check `if phone == ""` and treat it as a valid number. Use `Option<String>`.

---

## Compiler Errors

### Error E0308: Mismatched Types (`Option<T>` vs `T`)
```rust
let price: Option<u32> = Some(1500);
let total = price + 100;
```
Compiler output:
```text
error[E0369]: cannot add `{integer}` to `Option<u32>`
 --> src/main.rs:2:19
  |
2 |     let total = price + 100;
  |                 ----- ^ --- {integer}
  |                 |
  |                 Option<u32>
```
**Why it happens**: Rust does not automatically coerce `Option<T>` to `T`. You cannot accidentally do arithmetic with a potentially missing number.
**Fix**: Safely unpack the option:
```rust
let total = price.unwrap_or(0) + 100;
```

---

## Practice
1. **Find Cheapest Product**:
   Write a method on `Catalog`:
   ```rust
   pub fn find_cheapest_product(&self) -> Option<&Product>
   ```
   If the catalog has no products, return `None`. If it has products, return `Some(&cheapest)`.
2. **Customer Area Code**:
   Write a function that extracts the first 3 characters of a customer's phone number as an area code:
   ```rust
   pub fn area_code(customer: &Customer) -> Option<&str>
   ```
   Use `.as_deref()` and safe slicing.
3. **Upgrade Coupon**:
   Add a method on `Order`:
   ```rust
   pub fn upgrade_coupon(&mut self, new_coupon: Coupon) -> Option<Coupon>
   ```
   Use `self.coupon.replace(new_coupon)` to insert the new coupon and return the previous coupon (if any).

---

## Checkpoint
Verify all 9 tests pass cleanly:
```bash
cargo test
```
Expected output:
```text
running 9 tests
test tests::test_cart_item_option_lookup ... ok
test tests::test_catalog_option_lookups ... ok
test tests::test_coupon_discount_and_take ... ok
test tests::test_customer_optional_phone ... ok
test tests::test_order_cancellation_prevention ... ok
test tests::test_order_status_valid_lifecycle ... ok
test tests::test_order_total_with_payment_fee ... ok
test tests::test_payment_method_fees_and_descriptions ... ok
test tests::test_product_category_tax_rates ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Check with `clippy`:
```bash
cargo clippy -- -D warnings
```

---

## What We Learned
- Why `null` references caused decades of runtime crashes, and how Rust's type system prevents them at compile time.
- The structure of `Option<T>`: `Some(T)` vs `None`.
- How the Null Pointer Optimization guarantees zero memory overhead for `Option<&T>`.
- The dangers of `.unwrap()` in production and how to use `.unwrap_or()`, `.map()`, `.and_then()`, and `.take()` instead.
- How MiniStore uses `Option` for catalog searching, cart queries, customer contact info, and promotional coupons.

---

## What's Next
Now that we can safely represent the absence of a value with `Option<T>`, what happens when an operation doesn't just return *nothing*, but actually **fails with an error** (like invalid inputs or payment failures)?
In [Chapter 12: Result and Error Handling](/en/chapters/12-result-and-error-handling), we will explore Rust's second superpower enum: **`Result<T, E>`** and the legendary **`?` operator**!
