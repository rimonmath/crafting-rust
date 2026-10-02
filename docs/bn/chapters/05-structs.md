# অধ্যায় ৫: স্ট্রাক্ট: বাস্তব জিনিসের মডেলিং (Structs: Modeling Real Things)

## আপনি কী শিখবেন
- ফিল্ডের নামসহ **ক্লাসিক সি-স্টাইল স্ট্রাক্ট (Structs with named fields)** তৈরি ও ব্যবহার।
- স্ট্রাক্ট তৈরি করার দ্রুত কৌশল: ফিল্ড ইনিশিয়ালাইজেশন শর্টহ্যান্ড এবং **স্ট্রাক্ট আপডেট সিনট্যাক্স** (`..base`)।
- নিউটাইপ প্যাটার্ন (Newtype Pattern) এবং **টাপল স্ট্রাক্ট (Tuple Structs)** দিয়ে টাইপ-সেফটি নিশ্চিত করা।
- কোনো ফিল্ড ছাড়া মার্কার হিসেবে **ইউনিট-লাইক স্ট্রাক্ট (Unit-like Structs)**।
- `impl` ব্লকের মাধ্যমে ডাটার সাথে আচরণ (Behavior) যুক্ত করা: **মেথড** (`&self`, `&mut self`) এবং **অ্যাসোসিয়েটেড ফাংশন** (`Self::new`)।
- কম্পাইলার অ্যাট্রিবিউট ম্যাক্রো: `#[derive(Debug, Clone, PartialEq)]`-এর ভূমিকা।
- MiniStore-এর কাঁচা টাপল বাদ দিয়ে ডোমেন-ভিত্তিক বাস্তব মডেল তৈরি: `Product`, `Customer`, `CartItem` এবং `OrderSummary`।

---

## আমাদের কেন এটি প্রয়োজন?
অধ্যায় ৩ এবং ৪-এ আমরা MiniStore-এর পণ্যের ডাটা কাঁচা টাপলে রেখেছিলাম:

```rust
let item: (&str, u32, u32) = ("Mechanical Keyboard", 8999, 2);
let total = item.1 * item.2; // item.1 কোনটা ছিল? পণ্যের দাম নাকি সংখ্যা?
```

শুরুর দিকে টাপল সুবিধাজনক মনে হলেও বাস্তব প্রোডাকশন সিস্টেমে এটি দ্রুত বিপর্যয় ডেকে আনে:
1. **কোনো নাম বা অর্থ নেই (Zero Semantic Meaning)**: টাপলের মান পড়তে হয় ইনডেক্স (`.0`, `.1`, `.2`) দিয়ে। অসাবধানতাবশত দাম এবং স্টকের সংখ্যা অদলবদল হয়ে গেলে কম্পাইলার কোনো এরর দেয় না, অথচ বিজনেসে মারাত্মক ক্ষতি হয়ে যায়।
2. **লজিক ছড়িয়ে-ছিটিয়ে থাকা**: দাম যাচাই, স্টক কমানো বা ভ্যাট হিসাব করার মতো লজিকগুলো বিভিন্ন হেল্পার ফাংশনে ছড়িয়ে থাকে; ডোমেন এনটিটির নিজস্ব অংশ হয়ে ওঠে না।
3. **এনক্যাপসুলেশনের অভাব**: কোডের যেকোনো অংশ থেকে ভুল বা অসম্ভব মান দিয়ে অবৈধ টাপল তৈরি করে ফেলা যায়।

স্ট্রাক্ট প্রতিটি ফিল্ডকে নির্দিষ্ট নাম দিয়ে এবং `impl` ব্লকের মাধ্যমে ডাটার সাথে মেথড যুক্ত করে এই সমস্যার নিখুঁত সমাধান দেয়।

---

## সমস্যাটি কী?
একটি বাস্তব ই-কমার্স অ্যাপ্লিকেশন মডেল করার সময় তিনটি বড় চ্যালেঞ্জ দেখা দেয়:

1. **প্রিমিটিভ অবসেশন (Primitive Obsession)**: প্রোডাক্ট আইডি, কাস্টমার আইডি এবং অর্ডার আইডি—সবগুলোর জন্যই সাধারণ পূর্ণসংখ্যা (`u64`) ব্যবহার করা। একজন প্রোগ্রামার অসাবধানতাবশত `order_id`-এর জায়গায় `customer_id` পাঠিয়ে দিলে ডেটাবেজ ও স্টেট ওলটপালট হয়ে যায়।
2. **অনিয়ন্ত্রিত মিউটেশন**: ডোমেন ভ্যালিডেশন ছাড়াই যে কেউ স্টকের সংখ্যা নেগেটিভ করে ফেলতে পারে।
3. **একই কোডের পুনরাবৃত্তি**: একই পণ্যের একটি ভ্যারিয়েন্ট তৈরি করতে (যেমন স্ট্যান্ডার্ড কিবোর্ড বনাম আরজিবি কিবোর্ড) সবগুলো ফিল্ড বারবার টাইপ করা ক্লান্তিকর।

রাস্টের তিন ধরণের স্ট্রাক্ট এবং `impl` ব্লক এই সমস্ত সমস্যার সহজ ও শক্তিশালী সমাধান প্রদান করে।

---

## রাস্টের সমাধান (Rust Concept)

### ১. নামযুক্ত ফিল্ডসহ স্ট্রাক্ট (Named Field Structs)
স্ট্রাক্ট বিভিন্ন টাইপের ডাটাকে অর্থপূর্ণ নামে একসাথে দলভুক্ত করে:

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
> এখানে আমরা স্ট্রিং স্লাইস (`&str`)-এর বদলে ওনড (Owned) `String` ব্যবহার করেছি। রাস্টে স্ট্রাক্টের ভেতর রেফারেন্স (`&str`) রাখতে হলে লাইফটাইম অ্যানোটেশন (যেমন `struct Product<'a>`) দেওয়া বাধ্যতামূলক হয়, যাতে মেমোরি সুরক্ষার নিশ্চয়তা থাকে। কিন্তু ডোমেন মডেলে নিজস্ব মেমোরি ধারণকারী `String` ব্যবহার করা অনেক বেশি সহজ ও বাস্তবসম্মত।

### ২. ফিল্ড শর্টহ্যান্ড ও স্ট্রাক্ট আপডেট সিনট্যাক্স
যদি ভ্যারিয়েবলের নাম আর স্ট্রাক্টের ফিল্ডের নাম একই হয়, তবে বাড়তি কোড লিখতে হয় না:

```rust
let id = 1001;
let name = String::from("Keyboard");
// Product { id: id, name: name, ... } লেখার প্রয়োজন নেই
let p = Product { id, name, price_cents: 8999, stock: 10 };
```

একটি স্ট্রাক্টের বেশিরভাগ মান অপরিবর্তিত রেখে নতুন ইনস্ট্যান্স তৈরি করতে **স্ট্রাক্ট আপডেট সিনট্যাক্স** (`..`) ব্যবহার করা যায়:

```rust
let keyboard_rgb = Product {
    id: 1002,
    name: String::from("Keyboard (RGB Edition)"),
    ..keyboard.clone() // keyboard থেকে price_cents এবং stock স্বয়ংক্রিয়ভাবে নিয়ে নেবে
};
```

### ৩. টাপল স্ট্রাক্ট ও নিউটাইপ প্যাটার্ন (Tuple Structs)
টাপল স্ট্রাক্টে ফিল্ডের নাম থাকে না, শুধু টাইপ থাকে। এটি প্রিমিটিভ অবসেশন দূর করে শক্তিশালী টাইপ-সেফটি নিশ্চিত করে:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomerId(pub u64);
```

দুটিই ভেতরে `u64` সংখ্যা হলেও, কম্পাইলার কখনোই একটির জায়গায় অন্যটির ব্যবহার মেনে নেবে না (জিরো রানটাইম কস্টে কম্পাইল-টাইম গ্যারান্টি)।

### ৪. ইউনিট-লাইক স্ট্রাক্ট (Unit-like Structs)
কোনো ফিল্ড ছাড়া স্ট্রাক্ট ইউনিট টাইপ `()`-এর মতো আচরণ করে। কোনো অবস্থার মার্কার বা স্টেট ফ্ল্যাগ হিসেবে এটি ব্যবহৃত হয়:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandardPackaging;
```

### ৫. `impl` ব্লক: মেথড বনাম অ্যাসোসিয়েটেড ফাংশন
রাস্টে ডাটার গঠন (`struct`) এবং কার্যপ্রণালী (`impl`) আলাদা ব্লকে সংজ্ঞায়িত হয়:

```rust
impl Product {
    // অ্যাসোসিয়েটেড ফাংশন (কনস্ট্রাক্টর): কোনো self প্যারামিটার থাকে না
    pub fn new(id: u64, name: String, price_cents: u32, stock: u32) -> Self {
        Self { id, name, price_cents, stock }
    }

    // মেথড (রিড-অনলি): self-কে ইমিউটেবল রেফারেন্স হিসেবে ধার নেয়
    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    // মেথড (মিউটেশন): self-কে মিউটেবল রেফারেন্স হিসেবে গ্রহণ করে
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

- **অ্যাসোসিয়েটেড ফাংশন** (`Product::new(...)`): ডাবল কোলন (`::`) দিয়ে কল করা হয়। এরা ইনস্ট্যান্স ছাড়া সরাসরি টাইপের ওপর কাজ করে (সাধারণত কনস্ট্রাক্টর হিসেবে কাজ করে)।
- **মেথড** (`p.is_in_stock()`): ডট সিনট্যাক্স (`.`) দিয়ে কোনো ইনস্ট্যান্সের ওপর কল করা হয়। এদের প্রথম প্যারামিটার হিসেবে `&self`, `&mut self`, বা `self` থাকে।

---

## অন্যান্য ভাষা থেকে আসলে যা জানা দরকার

| ধারণা | Python | Go | Java / C# | C++ | Rust |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **ডাটা মডেলিং** | `class` / `@dataclass` | `type T struct` | `class` / `record` | `class` / `struct` | **`struct`** |
| **উত্তরাধিকার (Inheritance)** | সিঙ্গেল ও মাল্টিপল | কোনো ইনহেরিট্যান্স নেই | ক্লাস ইনহেরিট্যান্স (`extends`) | মাল্টিপল ইনহেরিট্যান্স | **ইনহেরিট্যান্স নেই (শুধুমাত্র কম্পোজিশন)** |
| **আচরণের সংজ্ঞা** | ক্লাসের ভেতরে | রিসিভার মেথড `(p *T)` | ক্লাসের ভেতরে | ক্লাসের ভেতরে | **পৃথক `impl` ব্লক** |
| **রিসিভার / `this`** | স্পষ্ট `self` | স্পষ্ট রিসিভার ভ্যারিয়েবল | অদৃশ্য `this` পয়েন্টার | অদৃশ্য `this` পয়েন্টার | **স্পষ্ট `&self` / `&mut self`** |
| **কনস্ট্রাক্টর** | `def __init__(self)` | প্রচলিত ফ্যাক্টরি `NewT()` | `public MyClass()` | `MyClass()` | **অ্যাসোসিয়েটেড ফাংশন `Self::new()`** |
| **কঠোর টাইপ র‍্যাপার** | কাস্টম ক্লাস | `type OrderId uint64` | র‍্যাপার ক্লাস | Typedef / enum class | **টাপল স্ট্রাক্ট (`struct OrderId(u64)`)** |

---

## ছোট উদাহরণ (Small Example)

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
    println!("ব্যবহারকারীর তথ্য: {:?}", admin);
}
```

---

## MiniStore-এ প্রয়োগ
MiniStore-এ এবার আমরা ক্লাস বা ইনহেরিট্যান্সের জটিলতা ছাড়াই নিখুঁত অবজেক্ট-ওরিয়েন্টেড ডোমেন মডেল তৈরি করছি:
1. **`Product`**: পণ্যের আইডি, নাম, সেন্টে দাম এবং ইনভেন্টরি স্টক। অতিরিক্ত বিক্রি রোধ করতে এতে রয়েছে `reduce_stock(&mut self, qty)` মেথড।
2. **`Customer`**: আইডি, নাম, ইমেইল এবং ভিআইপি লয়্যালটি স্ট্যাটাস।
3. **`CartItem`**: একটি `Product` এবং তার ক্রয়ের সংখ্যা (`quantity`) নিয়ে গঠিত কম্পোজিট টাইপ, যার রয়েছে `total_price_cents(&self)` মেথড।
4. **`OrderId`**: টাইপ-সেফ টাপল স্ট্রাক্ট যা কাস্টমার আইডির সাথে অর্ডার আইডির বিভ্রান্তি দূর করে।
5. **`OrderSummary`**: অর্ডারের সাবটোটাল, কাস্টমার ভিআইপি ছাড়, ডেলিভারি চার্জ এবং চূড়ান্ত বিল গণনাকারী অপরিবর্তনশীল রিপোর্ট।

---

## কোড (Code)

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

## কোড পর্যালোচনা (Understanding The Code)

1. **অ্যাসোসিয়েটেড কনস্ট্রাক্টর প্যাটার্ন**:
   ```rust
   pub fn new(id: u64, name: String, price_cents: u32, stock: u32) -> Self
   ```
   এখানে `Self` (বড়হাতের 'S') হলো `impl` ব্লকে সংজ্ঞায়িত টাইপটির (`Product`) সংক্ষিপ্ত রূপ। এটি কোডকে পরিষ্কার রাখে এবং টাইপের নাম পরিবর্তন করা সহজ করে তোলে।

2. **`&self` বনাম `&mut self`**:
   - `is_in_stock(&self)` কেবল `stock` ফিল্ডের মান পরীক্ষা করে। এটি কোনো স্টেট পরিবর্তন করতে পারে না।
   - `reduce_stock(&mut self, quantity: u32)` স্পষ্টভাবে মিউটেবল এক্সেস দাবি করে। যদি কোনো ভ্যারিয়েবল ইমিউটেবল (`let keyboard = ...`) হিসেবে থাকে, তবে কম্পাইলার `keyboard.reduce_stock(1)` কল করতে দেবে না।

3. **কম্পোজিট ডোমেন টাইপস**:
   ```rust
   pub struct CartItem {
       pub product: Product,
       pub quantity: u32,
   }
   ```
   আলাদা আলাদা প্যারালাল অ্যারে না বানিয়ে, `CartItem` সরাসরি `Product` অবজেক্টটিকে নিজের মধ্যে ধারণ করে (কম্পোজিশন)।

---

## সাধারণ ভুলসমূহ (Common Mistakes)

### ১. ইমিউটেবল স্ট্রাক্টের নির্দিষ্ট ফিল্ড পরিবর্তন করার চেষ্টা
রাস্টে মিউটেবিলিটি নির্ধারিত হয় পুরো ভ্যারিয়েবলটির বাইন্ডিং দ্বারা, ফিল্ডের একক বৈশিষ্ট্যে নয়:

```rust
let p = Product::new(1, String::from("Chair"), 4500, 10);
// p.stock = 5; // এরর! ইমিউটেবল ভ্যারিয়েবলের ফিল্ডে মান বসানো নিষেধ
```
কোনো ফিল্ড পরিবর্তন করতে বা `&mut self` মেথড কল করতে হলে ভ্যারিয়েবলটিকে শুরুতেই মিউটেবল করতে হবে: `let mut p = ...;`।

### ২. অ্যাসোসিয়েটেড ফাংশন (`::`) এবং মেথডের (`.`) মধ্যে গোলমাল
কনস্ট্রাক্টরের মতো অ্যাসোসিয়েটেড ফাংশন ডট দিয়ে কল করা যায় না:

```rust
let p = Product.new(...); // ভুল! সিনট্যাক্স এরর
let p = Product::new(...); // সঠিক

Product::is_in_stock(); // ভুল! ইনস্ট্যান্স দরকার
p.is_in_stock(); // সঠিক: কম্পাইলার নিজে থেকেই &p ধার করে
```

---

## কম্পাইলার এরর (Compiler Errors)
ইমিউটেবল ভ্যারিয়েবলের ওপর `&mut self` মেথড চালালে কম্পাইলার কী সতর্কবার্তা দেয়?

```rust
fn main() {
    let mut keyboard = Product::new(1, String::from("K"), 1000, 5);
    let keyboard = keyboard; // ইমিউটেবল ভ্যারিয়েবলে শ্যাডো করা হলো
    keyboard.reduce_stock(1);
}
```

কম্পাইলার আউটপুট:
```text
error[E0596]: cannot borrow `keyboard` as mutable, as it is not declared as mutable
 --> src/main.rs:5:5
  |
4 |     let keyboard = keyboard;
  |         -------- help: consider changing this to be mutable: `mut keyboard`
5 |     keyboard.reduce_stock(1);
  |     ^^^^^^^^ cannot borrow as mutable
```
রাস্টের কম্পাইলার রানটাইমে কোনো বাগ তৈরি হওয়ার আগেই কোড লেভেলে অননুমোদিত স্টেট পরিবর্তন আটকে দেয়।

---

## অনুশীলন (Practice)
1. `Product` স্ট্রাক্টে একটি নতুন মেথড `pub fn restock(&mut self, additional_units: u32)` যোগ করুন যা বিদ্যমান স্টকের সাথে নতুন ইউনিট যুক্ত করবে।
2. `Product`-এ `pub fn apply_discount_cents(&mut self, discount_cents: u32)` যোগ করুন যা পণ্যের দাম কমাবে, তবে দাম যেন কখনো শূন্যের নিচে না নামে তা নিশ্চিত করবে।
3. একটি নতুন টেস্ট লিখে যাচাই করুন যে ৫টি ইউনিট রিস্টক করলে ইনভেন্টরি স্টক বেড়ে ১৫ হচ্ছে।
4. `cargo test` দিয়ে সব টেস্ট সফলভাবে পাস হয়েছে কিনা নিশ্চিত করুন।

---

## চেকবক্স (Checkpoint)
- [x] নামযুক্ত স্ট্রাক্ট, টাপল স্ট্রাক্ট এবং ইউনিট-লাইক স্ট্রাক্ট সংজ্ঞায়িত করেছি।
- [x] ফিল্ড ইনিশিয়ালাইজেশন শর্টহ্যান্ড এবং স্ট্রাক্ট আপডেট সিনট্যাক্স (`..`) ব্যবহার করেছি।
- [x] ডাটার সংজ্ঞা (`struct`) এবং আচরণের সংজ্ঞা (`impl`) আলাদা রেখেছি।
- [x] `&self` এবং `&mut self` দিয়ে মেথড বাস্তবায়ন করেছি।
- [x] `Self` রিটার্নকারী অ্যাসোসিয়েটেড কনস্ট্রাক্টর ফাংশন লিখেছি।
- [x] MiniStore-এর জন্য শক্তিশালী ও টেস্টেবল ডোমেন মডেল তৈরি করেছি।

---

## আমরা কী শিখলাম
- ইনহেরিট্যান্স বা ক্লাসের জটিলতা ছাড়াই রাস্টে পরিষ্কার অবজেক্ট-ওরিয়েন্টেড ডিজাইন করা সম্ভব।
- ডাটা এবং মেথড আলাদা থাকায় কোড অনেক বেশি স্বচ্ছ, মডুলার ও সহজে পরিবর্তনযোগ্য হয়।
- টাপল স্ট্রাক্ট খুব চমৎকারভাবে প্রিমিটিভ অবসেশন দূর করে নিখুঁত টাইপ-সেফটি উপহার দেয়।

---

## পরবর্তী অধ্যায়ে কী আসছে
এর মধ্য দিয়ে সম্পন্ন হলো আমাদের **পার্ট ০: যাত্রা শুরু (Part 0: Starting the Journey)**! পরবর্তী **পার্ট ১: ওনারশিপ (অধ্যায় ৬: ওনারশিপ)**-এ আমরা উন্মোচন করব রাস্টের সবচেয়ে বিপ্লবী মেমোরি ম্যানেজমেন্ট মডেল: স্ট্যাক বনাম হিপ, ওনারশিপের নিয়মাবলী, মুভ সেমান্টিকস এবং কীভাবে রাস্ট কোনো গার্বেজ কালেক্টর ছাড়াই শতভাগ মেমোরি লিকমুক্ত সফটওয়্যার তৈরি নিশ্চিত করে।
