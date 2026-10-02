# অধ্যায় ১০: এনাম এবং প্যাটার্ন ম্যাচিং (Enums and Pattern Matching)

## আপনি কী শিখবেন
- কীভাবে **অ্যালজেব্রেইক ডাটা টাইপ (Algebraic Data Types - ADTs)** এবং সাম টাইপ (Sum Types) দিয়ে কোনো অবৈধ অবস্থা (Impossible States) ছাড়াই বাস্তব ডোমেইন মডেল তৈরি করা যায়।
- এনাম ভ্যারিয়েন্টের তিনটি রূপ: **ইউনিট ভ্যারিয়েন্ট**, **টাপল ভ্যারিয়েন্ট**, এবং **স্ট্রাক্ট ভ্যারিয়েন্ট**।
- এনামের মেমোরি গঠন: **ডিসক্রিমিন্যান্ট ট্যাগ (Discriminant Tag) + পে-লোড ইউনিয়ন (Payload Union)**।
- **`match` দিয়ে সর্বাঙ্গীন প্যাটার্ন ম্যাচিং (Exhaustive Pattern Matching)**: কেন কম্পাইলার আপনাকে প্রতিটি সম্ভাব্য কেস হ্যান্ডেল করতে বাধ্য করে।
- প্যাটার্ন ম্যাচিংয়ের আধুনিক সুবিধাসমূহ: ডিসট্রাকচারিং, ফিল্ড বাইন্ডিং, ম্যাচ গার্ড (`if condition`), এবং ওয়াইল্ডকার্ড (`_`)।
- **`if let`** দিয়ে সহজ ও পরিচ্ছন্ন একক শর্ত মেলানো।
- **`impl` ব্লক** দিয়ে সরাসরি এনামে মেথড ও ফাংশন যুক্ত করা।
- MiniStore-এ এনামের বাস্তব প্রয়োগ: ত্রুটিহীন `OrderStatus` স্টেট মেশিন, বিভিন্ন `PaymentMethod` এবং তাদের ফি গণনা, ও `ProductCategory` শ্রেণিবিভাগ।

---

## আমাদের কেন এটি প্রয়োজন?
[পার্ট ১: ওনারশিপে](/bn/chapters/06-ownership) আমরা মেমোরি ব্যবস্থাপনা, রেফারেন্স, স্লাইস ও কালেকশন আয়ত্ত করেছি।
এখন **পার্ট ২: বিজনেস লজিক মডেলিং**-এ আমাদের মূল লক্ষ্য মেমোরি থেকে সফটওয়্যারের ব্যবসায়িক নীতি (Business Domain Rules) সঠিকভাবে রূপায়ণে স্থানান্তরিত হচ্ছে।

যেকোনো সফটওয়্যারে বিভিন্ন এন্টিটির সুনির্দিষ্ট অবস্থা (State) থাকে। MiniStore-এর একটি অর্ডারের কথা ভাবুন:
- একটি নতুন অর্ডার প্রথমে থাকে **Pending** (অপেক্ষমাণ)।
- পেমেন্ট পাওয়ার পর এটি হয় **Confirmed**, যার সাথে একটি `receipt_id: String` যুক্ত থাকে।
- কুরিয়ারে বুকিং দিলে এটি হয় **Shipped**, যার সাথে একটি `tracking_number: String` থাকে।
- কাস্টমার বা দোকান বাতিল করলে এটি হয় **Cancelled**, যার সাথে একটি `reason: String` থাকে।

প্রচলিত প্রোগ্রামিং ভাষায় ডেভেলপাররা কীভাবে এটি মডেল করেন?
সাধারণত একটি স্ট্রাক্টের ভেতর একগুচ্ছ বুলিয়ান ফ্ল্যাগ ও ফাঁকা ফিল্ড নিয়ে:

```rust
// খারাপ ডিজাইন (Anti-Pattern): স্ববিরোধী অবৈধ অবস্থা তৈরির ঝুঁকি!
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

এখন কোনো বাগের কারণে যদি অসাবধানতাবশত একই সাথে `is_cancelled = true` এবং `is_shipped = true` হয়ে যায়, তখন কী ঘটবে? কিংবা `is_shipped = true` কিন্তু `tracking_number` ফাঁকা স্ট্রিং?
সফটওয়্যারটি একটি **স্ববিরোধী ও অবৈধ অবস্থায় (Impossible State)** প্রবেশ করবে। ডেভেলপারদের তখন ডজন ডজন ম্যানুয়াল `if-else` চেক লিখতে হয়, যা মানুষ হিসেবে কোনো না কোনো সময় ভুলে যাওয়া খুবই স্বাভাবিক।

**রাস্টের যুগান্তকারী সমাধান**: **এনাম (Enums) এবং প্যাটার্ন ম্যাচিং (Pattern Matching)**। রাস্টে এনামের একটি মান যেকোনো মুহূর্তে **কঠোরভাবে কেবল একটিমাত্র ভ্যারিয়েন্টেই থাকতে পারে**। একটি অর্ডার একই সাথে `Shipped` এবং `Cancelled` হওয়া টাইপ লেভেলেই সম্পূর্ণ অসম্ভব! অবৈধ অবস্থা কোডেই লেখা যায় না।

---

## সমস্যাটি কী?
MiniStore-এর অর্ডারের জীবনচক্র (Order Lifecycle) লক্ষ্য করুন:

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
               │   Shipped   │ ──────► আর বাতিল করা সম্ভব নয়!
               └──────┬──────┘
                      │ deliver()
                      ▼
               ┌─────────────┐
               │  Delivered  │ (চূড়ান্ত সমাপ্ত অবস্থা)
               └─────────────┘
```

অন্যান্য ভাষায়:
- **C**: এনাম শুধুমাত্র ইনটিজার সংখ্যা (`0, 1, 2`) ধারণ করে। এতে ট্র্যাকিং নম্বর বা কারণের মতো কোনো ডাটা রাখা যায় না।
- **Java / C#**: এনামে ফিল্ড যোগ করা যায়, কিন্তু সব ভ্যারিয়েন্টকেই হুবহু একই ফিল্ড বহন করতে হয়। `Shipped`-এ ট্র্যাকিং নম্বর থাকবে কিন্তু `Delivered`-এ কিছু থাকবে না—এমন বৈচিত্র্য পেতে ক্লাস হায়ারার্কি বা নাল-ভ্যালু দিয়ে জটিলতা তৈরি করতে হয়।
- **Python / Go**: কোনো কম্পাইল-টাইম এক্সহস্টিভ (Exhaustive) নিশ্চয়তা নেই; নতুন কোনো অবস্থা যোগ করলে পুরোনো কোড সতর্কবার্তা ছাড়াই রানটাইমে ক্র্যাশ করতে পারে।

MiniStore-এর প্রয়োজন:
১. প্রতিটি অবস্থা কেবল তার প্রয়োজনীয় ডাটুকুই বহন করবে, অতিরিক্ত কোনো অপ্রয়োজনীয় বা নাল ফিল্ড থাকবে না।
২. একটি কঠোর স্টেট মেশিন যাতে শুধুমাত্র কনফার্ম হওয়া অর্ডারই শিপ করা যায় এবং শিপ হওয়া অর্ডার বাতিল করা না যায়।
৩. কম্পাইলারের শতভাগ গ্যারান্টি যে আমরা যখনই অর্ডারের অবস্থা যাচাই করব, তখন যেন কোনো একটি কেসও মিস না হয়।

---

## রাস্টের সমাধান (Rust Concept)

### ১. এনাম ভ্যারিয়েন্টের তিন রূপ
একটি এনামেই আপনি ডাটা ছাড়া (Unit), সাধারণ ডাটা (Tuple) এবং নামযুক্ত ডাটা (Struct) একত্রে সংজ্ঞায়িত করতে পারেন:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    // ১. ইউনিট ভ্যারিয়েন্ট: কোনো বাড়তি ডাটা নেই
    Pending,
    Delivered,

    // ২. স্ট্রাক্ট ভ্যারিয়েন্ট: নামযুক্ত ফিল্ড ধারণ করে
    Confirmed { receipt_id: String },
    Shipped { tracking_number: String },
    Cancelled { reason: String },
}
```

### ২. মেমোরি বিন্যাস: ডিসক্রিমিন্যান্ট ট্যাগ + পে-লোড
সিপিইউ মেমোরিতে এনাম কীভাবে সংরক্ষিত হয়?
রাস্টে এনাম একটি **ট্যাগড ইউনিয়ন (Tagged Union)** হিসেবে কাজ করে:
১. **ডিসক্রিমিন্যান্ট ট্যাগ (Discriminant)**: সাধারণত ১ বাইটের একটি ছোট ইনটিজার, যা নির্দেশ করে বর্তমানে কোন ভ্যারিয়েন্টটি সক্রিয় আছে (`Pending` = 0, `Confirmed` = 1, ইত্যাদি)।
২. **পে-লোড ইউনিয়ন (Payload Union)**: মেমোরির একটি অংশ, যার আকার নির্ধারিত হয় এনামের সবচেয়ে বড় ভ্যারিয়েন্টটির ওপর ভিত্তি করে।

```text
OrderStatus-এর মেমোরি বিন্যাস:
┌──────────────┬──────────────────────────────────────────────────┐
│ ডিসক্রিমিন্যান্ট │ পে-লোড ইউনিয়ন (সবচেয়ে বড় ভ্যারিয়েন্ট String ২৪B)│
│ ট্যাগ (১ বাইট) │ যেমন: receipt_id / tracking_number / reason      │
└──────────────┴──────────────────────────────────────────────────┘
```
যেহেতু যেকোনো মুহূর্তে কেবল একটি ভ্যারিয়েন্টই সক্রিয় থাকে, তাই সব ভ্যারিয়েন্ট একই মেমোরি পে-লোড শেয়ার করে। ফলে এনাম চরম কমপ্যাক্ট এবং ক্যাশ-বান্ধব।

### ৩. `match` দিয়ে এক্সহস্টিভ প্যাটার্ন ম্যাচিং
রাস্টে `match` স্টেটমেন্টের প্রধান নীতি: **এটি অবশ্যই সর্বাঙ্গীন (Exhaustive) হতে হবে।**

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

ভুলবশত একটি ভ্যারিয়েন্ট বাদ পড়লে, কিংবা ভবিষ্যতে আপনি নতুন কোনো অবস্থা (যেমন `Refunded`) যোগ করলে—কম্পাইলার সাথে সাথে পুরো কোড আটকে দিয়ে বলে দেবে ঠিক কোন কোন ফাইলে `match` আপডেট করতে হবে! এতে প্রজেক্ট রিফ্যাক্টরিং হয়ে ওঠে নির্ভুল।

### ৪. প্যাটার্ন ম্যাচিংয়ের বৈশিষ্ট্য: ডিসট্রাকচারিং ও ম্যাচ গার্ড
ম্যাচিং করার সময়ই আপনি ভ্যারিয়েন্টের ভেতর থাকা ডাটা সরাসরি বের করে নিতে পারেন:

```rust
match &payment {
    PaymentMethod::CreditCard { last_four } => {
        println!("কার্ডের শেষ চার ডিজিট: {last_four}");
    }
    PaymentMethod::BankTransfer { reference } => {
        println!("ব্যাংক রেফারেন্স: {reference}");
    }
    PaymentMethod::CashOnDelivery => {
        println!("ক্যাশ অন ডেলিভারি");
    }
}
```

শর্ত যুক্ত করার জন্য **ম্যাচ গার্ড (Match Guards)** ব্যবহার করা যায়:
```rust
match status {
    OrderStatus::Cancelled { reason } if reason.contains("fraud") => {
        alert_security_team();
    }
    OrderStatus::Cancelled { reason } => {
        restock_inventory();
    }
    _ => {} // অন্য সব ক্ষেত্রে কিছু করার দরকার নেই
}
```

### ৫. `if let` দিয়ে সহজ ম্যাচিং
যখন আপনার কেবল একটি নির্দিষ্ট ভ্যারিয়েন্ট দরকার এবং বাকি সব ভ্যারিয়েন্ট উপেক্ষা করতে চান, তখন `if let` ব্যবহার করা হয়:

```rust
if let OrderStatus::Delivered = order.status {
    println!("অর্ডার সফলভাবে পৌঁছে গেছে!");
}
```

### ৬. এনামে মেথড যোগ করা
স্ট্রাক্টের মতো এনামেও আপনি `impl` ব্লক লিখে `&self`, `&mut self`, বা `self` মেথড লিখতে পারেন:

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

## অন্যান্য ভাষার সাথে তুলনা

| বিষয় | C | Java / C# | Python | TypeScript | Rust |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **এনামে ডাটা পে-লোড** | নেই (শুধুই সংখ্যা)। | সব ভ্যারিয়েন্টে একই ফিক্সড ক্লাস ফিল্ড। | সাধারণ অবজেক্ট অ্যাট্রিবিউট। | ডিসক্রিমিনেটেড ইউনিয়ন (`type = 'a'`)। | **যেকোনো ভ্যারিয়েন্টে ভিন্ন ভিন্ন ডাটা সরাসরি পে-লোড হিসেবে রাখা যায়।** |
| **সব কেস হ্যান্ডেলিং** | কম্পাইলার গ্রাহ্য করে না। | ওয়ার্নিং বা রানটাইমে ডিফল্ট এরর। | রানটাইমে ধরা পড়ে। | `never` টাইপ দিয়ে চেক করা যায়। | **কম্পাইল টাইমে কঠোর এক্সহস্টিভ চেক বাধ্যতামূলক।** |
| **মেমোরি দক্ষতা** | ৪ বাইটের ইনটিজার। | প্রতি ভ্যারিয়েন্টে ভারী হিপ অবজেক্ট। | ডাইনামিক পাইথন ডিকশনারি অবজেক্ট। | জাভাস্ক্রিপ্ট অবজেক্ট ওভারহেড। | **ট্যাগ + ইউনিয়ন (ভ্যালু টাইপে কোনো হিপ বরাদ্দ নেই)।** |
| **স্টেট ট্রানজিশন** | আলগা বুলিয়ান ফ্ল্যাগ। | একাধিক ক্লাস দিয়ে স্টেট প্যাটার্ন। | রানটাইম স্ট্রিং স্ট্যাটাস। | ইউনিয়ন টাইপ। | **এনাম স্টেট মেশিন যাতে অবৈধ ট্রানজিশন কম্পাইলই হয় না।** |

---

## ছোট উদাহরণ (Small Example)
নিচে ডাটা-ধারণকারী এনাম এবং প্যাটার্ন ম্যাচিংয়ের একটি বাস্তব উদাহরণ দেওয়া হলো:

```rust
enum WebEvent {
    PageLoad,
    KeyPress(char),
    Click { x: i64, y: i64 },
}

fn inspect(event: WebEvent) {
    match event {
        WebEvent::PageLoad => println!("পৃষ্ঠা লোড হয়েছে"),
        WebEvent::KeyPress(c) => println!("কী প্রেস করা হয়েছে: {c}"),
        WebEvent::Click { x, y } => println!("ক্লিক করা হয়েছে ({x}, {y}) স্থানে"),
    }
}

fn main() {
    inspect(WebEvent::KeyPress('q'));
    inspect(WebEvent::Click { x: 100, y: 250 });
}
```

---

## MiniStore-এ বাস্তব প্রয়োগ
MiniStore অ্যাপ্লিকেশনে এনামের ব্যবহার:
১. **`OrderStatus` স্টেট মেশিন**: `Pending`, `Confirmed { receipt_id }`, `Shipped { tracking_number }`, `Delivered`, এবং `Cancelled { reason }` ট্র্যাক করে।
২. **কঠোর ট্রানজিশন নিয়ম**:
   - `confirm()` কেবল `Pending` অর্ডারের ক্ষেত্রে কাজ করবে।
   - `ship()` কেবল `Confirmed` অর্ডারের ক্ষেত্রে কাজ করবে।
   - `cancel()` কেবল `Pending` বা `Confirmed` থাকা অবস্থায় অনুমোদিত। একবার `Shipped` হয়ে গেলে অর্ডার বাতিল করা টাইপ লেভেলেই প্রতিরোধিত!
৩. **`PaymentMethod` এনাম**: ক্রেডিট কার্ড, ব্যাংক ট্রান্সফার এবং ক্যাশ অন ডেলিভারি পরিচালনা ও লেনদেন ফি নির্ধারণ করে।
৪. **`ProductCategory`**: পণ্য শ্রেণিবদ্ধকরণ এবং বিভাগভিত্তিক ট্যাক্স রেট নির্ধারণ করে।

---

## কোড (Code)
নিচে `ministore/src/main.rs`-এর সম্পূর্ণ ও কম্পাইলযোগ্য কোড দেওয়া হলো:

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

## কোড বিশ্লেষণ (Understanding The Code)

### ১. `match` দিয়ে স্টেট ট্রানজিশন পরিচালনা
`Order::ship` মেথডটিতে লক্ষ্য করুন:
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
শুধুমাত্র `Confirmed` অবস্থায় থাকা অর্ডারই `Shipped`-এ যেতে পারে। `Pending`, `Delivered`, বা `Cancelled` অবস্থায় শিপ করার চেষ্টা করলে কম্পাইলার দ্বারা চালিত কোড উপযুক্ত এরর ফিরিয়ে দেবে। ট্রানজিশনগুলো পুরোপুরি সুনির্দিষ্ট।

### ২. পাইপ (`|`) অপারেটর দিয়ে একাধিক কেস মেলানো
`can_cancel` মেথডে:
```rust
match self {
    Self::Pending | Self::Confirmed { .. } => true,
    Self::Shipped { .. } | Self::Delivered | Self::Cancelled { .. } => false,
}
```
রাস্টে একাধিক ভ্যারিয়েন্টকে পাইপ `|` অপারেটর দিয়ে এক লাইনে হ্যান্ডেল করা যায়। ফলে কোড হয় সংক্ষিপ্ত এবং এক্সহস্টিভ।

---

## সাধারণ ভুলসমূহ (Common Mistakes)

### ১. `match`-এ কোনো একটি ভ্যারিয়েন্ট ভুলে যাওয়া
```rust
match payment {
    PaymentMethod::CreditCard { .. } => 150,
    PaymentMethod::CashOnDelivery => 300,
    // কম্পাইলার এরর! non-exhaustive patterns: `PaymentMethod::BankTransfer { .. }` not covered
}
```
রাস্ট কম্পাইলার কোনো ভ্যারিয়েন্ট বাদ পড়তে দেয় না। নতুন কোনো পেমেন্ট গেটওয়ে যুক্ত করলে কম্পাইলার সাথে সাথে আপনাকে জানিয়ে দেবে কোডের কোথায় কোথায় পরিবর্তন আনতে হবে।

### ২. অসাবধানতাবশত ওয়াইল্ডকার্ড `_` ব্যবহার করা
`_ => ...` সাময়িকভাবে কাজ চালালেও, এর অতিরিক্ত ব্যবহারে বিপদ হতে পারে: ভবিষ্যতে নতুন ভ্যারিয়েন্ট যোগ করলে ওয়াইল্ডকার্ড নীরবে তা গ্রহণ করে ফেলবে, ফলে আপনি নির্দিষ্ট বিজনেস লজিক লিখতে ভুলে যেতে পারেন। যেখানে সম্ভব সুনির্দিষ্ট ভ্যারিয়েন্ট প্যাটার্ন লিখুন।

---

## কম্পাইলার এরর বিশ্লেষণ (Compiler Errors)

### Error E0004: নন-এক্সহস্টিভ প্যাটার্ন ম্যাচ
মনে করুন আপনি লিখেছেন:

```rust
let status = OrderStatus::Pending;
match status {
    OrderStatus::Pending => println!("Pending"),
    OrderStatus::Confirmed { .. } => println!("Confirmed"),
}
```

রাস্ট কম্পাইলার তীব্র আপত্তি জানাবে:

```text
error[E0004]: non-exhaustive patterns: `OrderStatus::Shipped { .. }`, `OrderStatus::Delivered` and `OrderStatus::Cancelled { .. }` not covered
  --> src/main.rs:210:11
   |
210|     match status {
   |           ^^^^^^ patterns `OrderStatus::Shipped { .. }`, `OrderStatus::Delivered` and `OrderStatus::Cancelled { .. }` not covered
```

কম্পাইলারের এই কঠোর নিয়মের কারণেই রাস্ট অ্যাপ্লিকেশনে হ্যান্ডেল না করা স্টেট বা অপ্রত্যাশিত ক্র্যাশ প্রোডাকশনে যাওয়ার সুযোগ পায় না।

---

## অনুশীলনী (Practice)
১. **নতুন ভ্যারিয়েন্ট যোগ**: `OrderStatus`-এ একটি নতুন ভ্যারিয়েন্ট যোগ করুন `Refunded { refund_receipt_id: String }`।
২. **প্যাটার্ন আপডেট**: `display_status`, `can_cancel`, `is_terminal`, এবং `order_dispatch_advisory`-তে `Refunded` যুক্ত করুন।
৩. **রিফান্ড মেথড**: `Order`-এ একটি মেথড লিখুন `pub fn refund(&mut self, refund_receipt_id: String) -> Result<(), &'static str>`, যা কেবল `Delivered` বা `Cancelled` অর্ডারের ক্ষেত্রে রিফান্ড অনুমোদন করবে।
৪. `cargo test` দিয়ে সমাধান যাচাই করুন।

---

## চেকপয়েন্ট (Checkpoint)
- [x] আলগা বুলিয়ান ফ্ল্যাগের বদলে টাইপ-লেভেল অ্যালজেব্রেইক ডাটা টাইপ (Enums) ব্যবহার করেছেন।
- [x] ইউনিট, টাপল ও স্ট্রাক্ট ভ্যারিয়েন্ট দিয়ে কাস্টম পে-লোড সংরক্ষণ শিখেছেন।
- [x] ট্যাগড ইউনিয়নের মেমোরি গঠন (ট্যাগ + পে-লোড ইউনিয়ন) উপলব্ধি করেছেন।
- [x] `match` দিয়ে এক্সহস্টিভ প্যাটার্ন ম্যাচিং ও ডিসট্রাকচারিং আয়ত্ত করেছেন।
- [x] `if let` দিয়ে পরিচ্ছন্ন একক ভ্যারিয়েন্ট ম্যাচিং করেছেন।
- [x] MiniStore-এ সম্পূর্ণ ত্রুটিহীন অর্ডার স্টেট মেশিন কার্যকর করেছেন।

---

## আমরা কী শিখলাম
- রাস্টের এনাম কেবল সাধারণ সংখ্যার তালিকা নয়; এটি একটি পাওয়ারফুল অ্যালজেব্রেইক সাম টাইপ যা ডাটা বহন করতে পারে।
- কম্পাইলারের এক্সহস্টিভ প্যাটার্ন ম্যাচিং নিশ্চিত করে যে কোনো অবাস্তব বা অপ্রত্যাশিত অবস্থা সিস্টেমকে ক্ষতিগ্রস্ত করতে পারবে না।
- MiniStore এখন অর্ডার, পেমেন্ট চ্যানেল এবং ট্যাক্স ক্যাটাগরি সুনির্দিষ্ট টাইপ-সেফ স্টেট মেশিনের মাধ্যমে পরিচালনা করে।

---

## পরবর্তীতে কী আসছে
যদি কোনো ফাংশনে প্রত্যাশিত মান পাওয়া না যায় (যেমন হাশম্যাপে কি না পাওয়া)? প্রচলিত ভাষায় একে `null` দেওয়া হয় যা রানটাইমে মারাত্মক NullPointerException ঘটায়। [অধ্যায় ১১: Option](/bn/chapters/11-option)-এ আমরা শিখব রাস্টের নাল-মুক্ত নিরাপদ সমাধান: `Option<T>` এনাম।
