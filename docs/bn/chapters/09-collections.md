# অধ্যায় ৯: কালেকশনস (Collections)

## আপনি কী শিখবেন
- কেন ফিক্সড আকারের স্ট্যাক অ্যারের বাইরে ডাইনামিক হিপ কালেকশন অপরিহার্য।
- রাস্টের স্ট্যান্ডার্ড কালেকশনের ত্রয়ী রূপ: **`Vec<T>`**, **`HashMap<K, V>`**, এবং **`HashSet<T>`**।
- **ভেক্টরের অভ্যন্তরীণ গঠন**: পয়েন্টার, দৈর্ঘ্য, ক্যাপাসিটি এবং দ্বিগুণ হারে মেমোরি রিলোকেশন কৌশল।
- **এন্ট্রি এপিআই (The Entry API)**: হাশম্যাপে কোনো অতিরিক্ত লুকআপ ছাড়াই সরাসরি ইন-প্লেস ডাটা হালনাগাদের অনন্য রাস্ট টেকনিক।
- **অনন্য মান ও সদস্যপদ নির্ধারণ**: `HashSet` দিয়ে $O(1)$ গতিতে ডুপ্লিকেট রোধ।
- বরো চেকার বনাম কালেকশন: রিলোকেশনের সময় ড্যাংলিং পয়েন্টার ও ইটারেটর ইনভ্যালিডেশনের চিরতরে অবসান।
- ভেক্টর থেকে স্বয়ংক্রিয় স্লাইস রূপান্তর (`&Vec<T>` -> `&[T]`)।
- MiniStore-এ কালেকশনের বাস্তব প্রয়োগ: ডাইনামিক মাল্টি-আইটেম শপিং কার্ট (`ShoppingCart`), $O(1)$ গতিতে SKU সার্চ ইনডেক্স, ডিপার্টমেন্টভিত্তিক পণ্য গণনা এবং কাস্টমার ট্যাগিং।

---

## আমাদের কেন এটি প্রয়োজন?
পূর্ববর্তী অধ্যায়গুলোতে আমরা একক প্রোডাক্ট এবং নির্দিষ্ট আকারের স্ট্যাক অ্যারে (যেমন `[u32; 4]`) নিয়ে কাজ করেছি।
কিন্তু বাস্তব জীবনের কোনো ই-কমার্স সিস্টেমে আগে থেকে বলা সম্ভব নয় একজন ক্রেতা তার কার্টে কতটি পণ্য রাখবেন, কিংবা স্টোরের ক্যাটালগে মোট কত হাজার পণ্য থাকবে:

১. একজন ক্রেতা ১টি পণ্যও কিনতে পারেন, আবার ৫০টি পণ্যও কিনতে পারেন। নির্দিষ্ট আকারের `[CartItem; 10]` অ্যারে ব্যবহার করলে ১০টির বেশি আইটেম রাখলেই প্রোগ্রাম ক্র্যাশ করবে।
২. হাজার হাজার পণ্যের অ্যারেতে কোনো পণ্য খুঁজতে গেলে প্রতিবার লুপ চালিয়ে খোঁজা ($O(N)$ সময়) অত্যন্ত ধীরগতির। আমাদের দরকার সেকেন্ডের ভগ্নাংশে তাৎক্ষণিক ($O(1)$) কি-ভ্যালু সার্চ।
৩. কাস্টমারের আগ্রহ বা ট্যাগ ট্র্যাক করতে (যেমন `"vip"`, `"early_adopter"`) আমাদের এমন ডাটা স্ট্রাকচার প্রয়োজন যা স্বয়ংক্রিয়ভাবে ডুপ্লিকেট আইটেম এড়িয়ে চলে।

```text
নির্দিষ্ট আকারের স্ট্যাক অ্যারে [T; 4]      ডাইনামিক হিপ ভেক্টর Vec<T>
┌────────┬────────┬────────┬────────┐    স্ট্যাক (২৪ বাইট)        হিপ মেমোরি (ডাইনামিক আকার)
│ item 0 │ item 1 │ item 2 │ item 3 │    ┌─────┬─────┬─────┐     ┌───────┬───────┬───────┬───
└────────┴────────┴────────┴────────┘    │ ptr │ len │ cap │ ──► │ val 0 │ val 1 │ val 2 │...
কম্পাইল টাইমে ফিক্সড                     └─────┴─────┴─────┘     └───────┴───────┴───────┴───
```

**রাস্টের যুগান্তকারী সমাধান**: স্ট্যান্ডার্ড লাইব্রেরি কালেকশনস (`std::collections`)। রাস্টের ওনারশিপ নিয়মের ওপর ভিত্তি করে তৈরি এই কালেকশনগুলো কোনো গার্বেজ কালেক্টর ছাড়াই ডাইনামিক হিপ মেমোরি ১০০% নিরাপদে পরিচালনা করে।

---

## সমস্যাটি কী?
অন্যান্য ভাষায় কালেকশন ব্যবহারের সমস্যাসমূহ:
- **C++**: `std::vector`-এ নতুন উপাদান যোগ করতে গিয়ে ক্যাপাসিটি ফুরিয়ে গেলে এটি মেমোরির অন্য ঠিকানায় সরে যায় (Reallocation)। এখন প্রোগ্রামের অন্য কোথাও যদি ভেক্টরের কোনো উপাদানের পয়েন্টার জমা থাকে (`int* p = &vec[0];`), তবে `vec.push_back()` কল করার সাথে সাথেই পয়েন্টারটি অকেজো হয়ে যায়—যা মারাত্মক **ড্যাংলিং পয়েন্টার ও মেমোরি করাপশন** তৈরি করে।
- **Java / Python**: জাভায় লুপ চলাকালীন কোনো লিস্টে উপাদান যোগ বা বিয়োগ করলে সাথে সাথে রানটাইমে `ConcurrentModificationException` ছুঁড়ে প্রোগ্রাম ক্র্যাশ করে।

MiniStore-এর প্রয়োজন:
- একটি ডাইনামিক শপিং কার্ট যেখানে ইচ্ছেমতো পণ্য যোগ, বিয়োগ বা পরিমাণ বাড়ানো যায়।
- সম্পূর্ণ প্রোডাক্ট লিস্ট স্ক্যান না করে সরাসরি SKU দিয়ে $O(1)$ সময়ে পণ্য খুঁজে বের করা।
- কম্পাইল টাইমের শতভাগ গ্যারান্টি যে কোনো ভেক্টরে ডাটা যোগ করার সময় অন্য কোনো রেফারেন্স ক্ষতিগ্রস্ত হবে না।

---

## রাস্টের সমাধান (Rust Concept)

### ১. ডাইনামিক ভেক্টর: `Vec<T>`
ভেক্টর হলো হিপ মেমোরিতে সংরক্ষিত একটি পরিবর্তনশীল ও সম্প্রসারণযোগ্য অ্যারে। `String`-এর মতোই স্ট্যাকে একটি `Vec<T>` ২৪ বাইট (৩টি ৬৪-বিট উপাদান) জায়গা নেয়:
১. হিপ বাফারের শুরুর পয়েন্টার (`ptr`)।
২. বর্তমান উপাদানের সংখ্যা (`len`)।
৩. মোট বরাদ্দকৃত মেমোরি ধারণক্ষমতা (`capacity`)।

```rust
// ১. ফাঁকা ভেক্টর তৈরি:
let mut items: Vec<CartItem> = Vec::new();

// ২. vec! ম্যাক্রো দিয়ে সরাসরি মানসহ ভেক্টর তৈরি:
let mut numbers = vec![10, 20, 30];

// ৩. নতুন উপাদান যোগ করা (হিপে স্বয়ংক্রিয়ভাবে বড় হয়):
numbers.push(40);
```

#### ভেক্টরের মেমোরি সম্প্রসারণ নীতি (Reallocation)
যখন ভেক্টরের উপাদান সংখ্যা ক্যাপাসিটির সমান হয়ে যায় (`len == capacity`), তখন `.push()` কল করলে ভেক্টরটি:
১. পূর্বের তুলনায় দ্বিগুণ মেমোরির নতুন হিপ ব্লক বরাদ্দ করে।
২. পুরাতন উপাদানগুলোকে নতুন মেমোরি ব্লকে স্থানান্তর (Move) করে।
৩. পুরাতন ব্লকটি স্বয়ংক্রিয়ভাবে মুছে ফেলে।
৪. নতুন পয়েন্টার ও ক্যাপাসিটি আপডেট করে।

যেহেতু `Vec<T>` স্বয়ংক্রিয়ভাবে স্লাইস `&[T]`-এ পরিণত হতে পারে (Deref Coercion), তাই অধ্যায় ৮-এ স্লাইসের জন্য লেখা সমস্ত ফাংশন সরাসরি ভেক্টরের সাথে কাজ করে!

### ২. হাশ ম্যাপ: `HashMap<K, V>`
`HashMap<K, V>` একটি হাশ অ্যালগরিদম (ডিফল্টভাবে ক্রিপ্টোগ্রাফিক্যালি নিরাপদ SipHash) ব্যবহার করে কি (Key) এবং ভ্যালু (Value)-এর সম্পর্ক সংরক্ষণ করে।

```rust
use std::collections::HashMap;

let mut catalog: HashMap<String, Product> = HashMap::new();
catalog.insert(product.sku.clone(), product);

// তাৎক্ষণিক O(1) লুকআপ:
if let Some(product) = catalog.get("TECH-KEY-001") {
    println!("Found: {}", product.name);
}
```

#### রাস্টের অনন্য এন্ট্রি এপিআই (The Entry API)
পাইথন বা জাভায় কোনো কি না থাকলে ডিফল্ট মান বসিয়ে মান আপডেট করতে সাধারণত দুবার হাশ টেবিল খুঁজতে হয়:
```python
# পাইথনে ২টি লুকআপ
if key not in counts:
    counts[key] = 0
counts[key] += 1
```

রাস্টে এটি **মাত্র একবার হাশ লুকআপ** করে অবিশ্বাস্য গতিতে সম্পন্ন করা যায়:
```rust
// হাশ টেবিলে মাত্র একটি সিঙ্গেল লুকআপ!
*counts.entry(department_name).or_insert(0) += 1;
```
- `.entry(key)` হাশম্যাপে কি-এর জায়গাটি চিহ্নিত করে।
- `.or_insert(default)` মান না থাকলে ডিফল্ট মান ইনসার্ট করে এবং উপস্থিত বা নতুন মানের একটি মিউটেবল রেফারেন্স `&mut V` ফিরিয়ে দেয়।
- শুরুর `*` দিয়ে ডিরেফারেন্স করে সরাসরি ইন-প্লেস মান বৃদ্ধি করা হয়!

### ৩. হাশ সেট: `HashSet<T>`
`HashSet<T>` মূলত এমন একটি কালেকশন যা শুধুমাত্র অনন্য (Unique) উপাদান সংরক্ষণ করে। কোনো ডুপ্লিকেট ডাটা এতে প্রবেশ করতে পারে না।

```rust
use std::collections::HashSet;

let mut tags = HashSet::new();
tags.insert(String::from("vip"));
tags.insert(String::from("vip")); // ডুপ্লিকেট: স্বয়ংক্রিয়ভাবে বাতিল

assert!(tags.contains("vip"));
```

### ৪. বরো চেকার বনাম কালেকশন মিউটেশন
ভেক্টরের কোনো উপাদানের রেফারেন্স ধরে রেখে যদি নতুন উপাদান যোগ করার চেষ্টা করা হয়, তবে কী ঘটবে?

```rust
let mut v = vec![1, 2, 3];
let first = &v[0]; // v-এর হিপ বাফারের একটি উপাদানের রেফারেন্স

// v.push(4); // কম্পাইলার এরর! cannot borrow `v` as mutable while borrowed as immutable
println!("First: {first}");
```

**কম্পাইলার কেন কোড আটকে দেয়?** যদি ৪ যোগ করার ফলে ভেক্টরের মেমোরি দ্বিগুণ করতে গিয়ে মেমোরি অন্য ঠিকানায় চলে যায়, তবে `first` একটি মৃত মেমোরির ঠিকানায় নির্দেশ করত (Dangling Pointer)। রাস্ট কম্পাইল টাইমে এটি পুরোপুরি অসম্ভব করে তুলে মেমোরি সুরক্ষিত রাখে।

---

## অন্যান্য ভাষার সাথে তুলনা

| বিষয় | C++ | Java | Python | Go | Rust |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **ডাইনামিক অ্যারে** | `std::vector<T>` | `ArrayList<T>` | `list` | `[]T` স্লাইস | `Vec<T>` |
| **হাশ ম্যাপ** | `std::unordered_map` | `HashMap<K, V>` | `dict` | `map[K]V` | `HashMap<K, V>` |
| **হাশ সেট** | `std::unordered_set` | `HashSet<T>` | `set` | `map[T]struct{}` দিয়ে বানাতে হয় | `HashSet<T>` |
| **রিলোকেশন নিরাপত্তা**| রিসাইজ হলে পয়েন্টার অচল হয়ে যায় (বাগ)। | সেফ (GC পয়েন্টার বাঁচিয়ে রাখে)। | সেফ (GC)। | সেফ (GC)। | **বরো চেকার দ্বারা কম্পাইল টাইমেই প্রতিরোধিত।** |
| **ইটারেশন মিউটেশন** | আনডিফাইন্ড বিহেভিয়ার বা ক্র্যাশ। | রানটাইমে `ConcurrentModificationException`। | ভুল ফলাফল বা লুপ স্কিপ। | অপ্রত্যাশিত আচরণ। | **কম্পাইল টাইমেই পুরোপুরি নিষিদ্ধ।** |

---

## ছোট উদাহরণ (Small Example)
নিচে `Vec`, `HashMap` (Entry API সহ) এবং `HashSet`-এর বাস্তব রূপ দেওয়া হলো:

```rust
use std::collections::{HashMap, HashSet};

fn main() {
    // ১. Vec
    let mut scores = vec![100, 95];
    scores.push(88);
    println!("মোট স্কোর: {}", scores.len());

    // ২. HashMap এবং Entry API
    let mut word_counts = HashMap::new();
    let words = ["apple", "banana", "apple", "cherry"];
    for &word in &words {
        *word_counts.entry(word).or_insert(0) += 1;
    }
    println!("Apple পাওয়া গেছে: {:?}", word_counts.get("apple")); // Some(2)

    // ৩. HashSet
    let mut unique_words = HashSet::new();
    for &word in &words {
        unique_words.insert(word);
    }
    println!("অনন্য শব্দের সংখ্যা: {}", unique_words.len()); // ৩টি
}
```

---

## MiniStore-এ বাস্তব প্রয়োগ
MiniStore অ্যাপ্লিকেশনে কালেকশনের ব্যবহার:
১. **ডাইনামিক শপিং কার্ট (`ShoppingCart`)**: `Vec<CartItem>` দিয়ে পরিচালিত কার্টে যত ইচ্ছা আইটেম যোগ করা যায়। একই প্রোডাক্ট বারবার যোগ করলে স্বয়ংক্রিয়ভাবে তার কোয়ান্টিটি বৃদ্ধি পায়।
২. **মাল্টি-আইটেম চেকআউট**: `PendingOrder` এখন সম্পূর্ণ আইটেম লিস্ট `items: Vec<CartItem>` গ্রহণ করে বহু-পণ্যের চেকআউট সমর্থন করে।
৩. **SKU ইনডেক্স দিয়ে তাৎক্ষণিক সার্চ**: `build_sku_index` মেথড `HashMap<String, Product>` তৈরি করে যা ওয়ান-শটে পণ্য খুঁজে দেয়।
৪. **ডিপার্টমেন্ট অনুযায়ী প্রোডাক্ট গণনা**: `count_products_by_department` এন্ট্রি এপিআই ব্যবহার করে বিভাগভিত্তিক পণ্যের হিসাব করে।
৫. **কাস্টমার ট্যাগিং**: `Customer` এখন `HashSet<String>` ব্যবহার করে যাতে একই কাস্টমারকে কোনো ট্যাগ দুবার দেওয়া না যায়।

---

## কোড (Code)
নিচে `ministore/src/main.rs`-এর সম্পূর্ণ ও কম্পাইলযোগ্য কোড দেওয়া হলো:

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub sku: String,
    pub name: String,
    pub price_cents: u32,
    pub stock: u32,
}

impl Product {
    pub fn new(id: u64, sku: String, name: String, price_cents: u32, stock: u32) -> Self {
        Self {
            id,
            sku,
            name,
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
    /// Zero heap allocation: returns a 16-byte fat pointer (ptr + len) directly into `sku`.
    pub fn department_code(&self) -> &str {
        match self.sku.find('-') {
            Some(idx) => &self.sku[..idx],
            None => &self.sku[..],
        }
    }

    /// Returns a slice of the product name truncated to `max_bytes` without reallocating,
    /// ensuring the slice boundary respects UTF-8 character boundaries.
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

    /// Adds a tag to customer's unique tag set.
    pub fn add_tag(&mut self, tag: &str) -> bool {
        self.tags.insert(tag.to_string())
    }

    /// Checks if customer has a specific tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    /// Borrows `&self` immutably: reads customer data to display membership badge.
    pub fn display_badge(&self) -> String {
        if self.is_vip {
            format!("[VIP Member] {}", self.name)
        } else {
            format!("[Standard Member] {}", self.name)
        }
    }

    /// Borrows `&mut self` mutably: grants VIP membership status in place.
    pub fn upgrade_to_vip(&mut self) {
        self.is_vip = true;
        self.tags.insert(String::from("vip"));
    }

    /// Borrows `&mut self` mutably: updates email address in place.
    pub fn update_email(&mut self, new_email: String) {
        self.email = new_email;
    }
}

/// OrderId implements `Copy`: 8-byte scalar on the stack, never moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(pub u64);

/// Represents a single line item in a shopping cart.
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

/// A dynamic, multi-item shopping cart backed by a heap-allocated `Vec<CartItem>`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShoppingCart {
    pub items: Vec<CartItem>,
}

impl ShoppingCart {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Adds a product to the cart. If already present, increments quantity in place.
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

    /// Removes an item from the cart by product ID.
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

/// An unconfirmed order owning a dynamic list of items.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingOrder {
    pub order_id: OrderId,
    pub customer: Customer,
    pub items: Vec<CartItem>,
}

/// Finalized invoice/receipt produced when checkout consumes `PendingOrder`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmedReceipt {
    pub receipt_id: String,
    pub order_id: OrderId,
    pub customer_name: String,
    pub item_count: u32,
    pub total_cents: u32,
}

// ============================================================================
// Collection Utility Functions (Vec, HashMap, HashSet & Entry API)
// ============================================================================

/// Groups products by department and counts them using `HashMap` and the Entry API.
pub fn count_products_by_department(products: &[Product]) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for product in products {
        let dept = product.department_code().to_string();
        // The Entry API: zero duplicate lookups!
        *counts.entry(dept).or_insert(0) += 1;
    }
    counts
}

/// Indexes an array or slice of products into a `HashMap<String, Product>` by SKU for O(1) lookups.
pub fn build_sku_index(products: &[Product]) -> HashMap<String, Product> {
    let mut map = HashMap::new();
    for product in products {
        map.insert(product.sku.clone(), product.clone());
    }
    map
}

/// Calculates discount for a multi-item subtotal based on VIP status.
pub fn calculate_discount(customer: &Customer, subtotal_cents: u32) -> u32 {
    if customer.is_vip || customer.has_tag("vip") {
        (subtotal_cents * 10) / 100 // 10% discount for VIPs
    } else {
        0
    }
}

/// Deduplicates user IDs using a `HashSet`.
pub fn collect_unique_customers(orders: &[PendingOrder]) -> HashSet<u64> {
    let mut unique_ids = HashSet::new();
    for order in orders {
        unique_ids.insert(order.customer.id);
    }
    unique_ids
}

/// Consumes ownership of `PendingOrder` by value (Move Semantics) to finalize multi-item checkout.
pub fn finalize_order(order: PendingOrder) -> ConfirmedReceipt {
    let subtotal: u32 = order.items.iter().map(|item| item.line_total()).sum();
    let discount = calculate_discount(&order.customer, subtotal);
    let total_cents = subtotal - discount;
    let item_count = order.items.iter().map(|item| item.quantity).sum();

    ConfirmedReceipt {
        receipt_id: format!("REC-{}-{}", order.order_id.0, order.customer.id),
        order_id: order.order_id,
        customer_name: order.customer.name,
        item_count,
        total_cents,
    }
}

fn main() {
    println!("=== MiniStore: Collections (Vec, HashMap, HashSet & Entry API) ===\n");

    // 1. Dynamic Heap Array: Vec<T> for Products & Cart Items
    println!("1. Dynamic Vectors (Vec<T>):");
    let mut catalog_vec: Vec<Product> = vec![
        Product::new(
            101,
            String::from("TECH-KEY-001"),
            String::from("Mechanical Keyboard"),
            12000,
            10,
        ),
        Product::new(
            102,
            String::from("TECH-MOU-002"),
            String::from("Wireless Gaming Mouse"),
            4500,
            25,
        ),
        Product::new(
            103,
            String::from("OFFC-CHR-003"),
            String::from("Ergonomic Desk Chair"),
            35000,
            5,
        ),
    ];

    // Demonstrating dynamic growth via push:
    catalog_vec.push(Product::new(
        104,
        String::from("OFFC-DSK-004"),
        String::from("Standing Desk Frame"),
        45000,
        4,
    ));

    println!("   Catalog count: {} products", catalog_vec.len());
    println!(
        "   Vector capacity: {} (Allocated on heap)",
        catalog_vec.capacity()
    );

    // 2. HashMap<K, V> with O(1) Fast Lookups & The Entry API
    println!("\n2. Fast Key-Value Lookups with HashMap & Entry API:");
    let sku_map = build_sku_index(&catalog_vec);
    if let Some(mouse) = sku_map.get("TECH-MOU-002") {
        println!("   Found product by SKU 'TECH-MOU-002': {}", mouse.name);
    }

    let dept_counts = count_products_by_department(&catalog_vec);
    println!("   Products grouped by department (via Entry API):");
    for (dept, count) in &dept_counts {
        println!("     - {dept}: {count} item(s)");
    }

    // 3. HashSet<T> for Unique Tags & Deduplication
    println!("\n3. Unique Elements with HashSet<T>:");
    let mut customer = Customer::new(
        501,
        String::from("Grace Hopper"),
        String::from("grace@navy.mil"),
        false,
    );
    customer.add_tag("newsletter");
    customer.add_tag("early_adopter");
    customer.add_tag("newsletter"); // Duplicate insertion is ignored

    println!("   Customer: {}", customer.name);
    println!("   Tags set: {:?}", customer.tags);
    println!(
        "   Has 'newsletter' tag: {}",
        customer.has_tag("newsletter")
    );
    customer.upgrade_to_vip();
    println!("   After VIP upgrade: {:?}", customer.tags);

    // 4. Multi-Item ShoppingCart backed by Vec<CartItem>
    println!("\n4. Shopping Cart Operations (Dynamic Items List):");
    let mut cart = ShoppingCart::new();
    cart.add_item(101, 1, 12000); // 1x Keyboard @ $120.00
    cart.add_item(102, 2, 4500); // 2x Mouse @ $45.00
    cart.add_item(101, 1, 12000); // 1 more Keyboard (increments quantity to 2)

    println!("   Total line items in cart: {}", cart.items.len());
    println!("   Total physical units:     {}", cart.total_units());
    println!(
        "   Cart subtotal:            ${:.2}",
        cart.subtotal_cents() as f64 / 100.0
    );

    // 5. Finalizing Multi-Item Order (Ownership Consumption)
    println!("\n5. Multi-Item Order Finalization:");
    let pending_order = PendingOrder {
        order_id: OrderId(8801),
        customer,
        items: cart.items, // Ownership of Vec<CartItem> moves into pending_order
    };

    let receipt = finalize_order(pending_order);
    println!("   Receipt ID:   {}", receipt.receipt_id);
    println!("   Customer:     {}", receipt.customer_name);
    println!("   Total Units:  {}", receipt.item_count);
    println!(
        "   Total Paid:   ${:.2} (10% VIP discount applied)",
        receipt.total_cents as f64 / 100.0
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_shopping_cart_add_and_aggregate() {
        let mut cart = ShoppingCart::new();
        assert!(cart.is_empty());

        cart.add_item(1, 2, 1000); // 2 x $10.00 = $20.00
        cart.add_item(2, 1, 2500); // 1 x $25.00 = $25.00
        cart.add_item(1, 1, 1000); // Adds 1 more to product 1 -> 3 x $10.00 = $30.00

        assert_eq!(cart.items.len(), 2);
        assert_eq!(cart.total_units(), 4);
        assert_eq!(cart.subtotal_cents(), 5500); // $30.00 + $25.00 = $55.00
    }

    #[test]
    fn test_vec_shopping_cart_remove_item() {
        let mut cart = ShoppingCart::new();
        cart.add_item(10, 2, 500);
        cart.add_item(20, 1, 1500);

        assert_eq!(cart.items.len(), 2);
        let removed = cart.remove_item(10);
        assert!(removed);
        assert_eq!(cart.items.len(), 1);
        assert_eq!(cart.items[0].product_id, 20);

        let not_found = cart.remove_item(999);
        assert!(!not_found);
    }

    #[test]
    fn test_hashmap_entry_api_department_counts() {
        let products = [
            Product::new(1, String::from("TECH-1"), String::from("P1"), 100, 5),
            Product::new(2, String::from("TECH-2"), String::from("P2"), 200, 5),
            Product::new(3, String::from("HOME-1"), String::from("P3"), 300, 5),
        ];

        let counts = count_products_by_department(&products);
        assert_eq!(counts.get("TECH"), Some(&2));
        assert_eq!(counts.get("HOME"), Some(&1));
        assert_eq!(counts.get("FOOD"), None);
    }

    #[test]
    fn test_hashset_customer_tags() {
        let mut customer = Customer::new(1, String::from("Alice"), String::from("a@a.com"), false);
        assert!(!customer.has_tag("vip"));

        customer.add_tag("beta_tester");
        customer.add_tag("beta_tester"); // Duplicate
        assert_eq!(customer.tags.len(), 1);
        assert!(customer.has_tag("beta_tester"));

        customer.upgrade_to_vip();
        assert!(customer.is_vip);
        assert!(customer.has_tag("vip"));
    }

    #[test]
    fn test_multi_item_order_finalize() {
        let mut customer = Customer::new(1, String::from("Bob"), String::from("b@b.com"), false);
        customer.upgrade_to_vip();

        let items = vec![
            CartItem::new(1, 2, 2000), // 2 * $20 = $40.00 (4000 cents)
            CartItem::new(2, 1, 6000), // 1 * $60 = $60.00 (6000 cents)
        ]; // Subtotal: 10000 cents ($100.00)

        let order = PendingOrder {
            order_id: OrderId(42),
            customer,
            items,
        };

        // 10% VIP discount on 10000 = 1000 -> Total: 9000 cents ($90.00)
        let receipt = finalize_order(order);
        assert_eq!(receipt.order_id, OrderId(42));
        assert_eq!(receipt.customer_name, "Bob");
        assert_eq!(receipt.item_count, 3);
        assert_eq!(receipt.total_cents, 9000);
    }
}
```

---

## কোড বিশ্লেষণ (Understanding The Code)

### ১. `ShoppingCart`-এ ইন-প্লেস কোয়ান্টিটি বৃদ্ধি
`add_item` মেথডে:
```rust
for item in &mut self.items {
    if item.product_id == product_id {
        item.quantity += quantity;
        return;
    }
}
self.items.push(CartItem::new(product_id, quantity, unit_price_cents));
```
আমরা `&mut self.items` দিয়ে প্রতিটি আইটেম মিউটেবলভাবে ধার নিই। যদি পণ্যটি কার্টে আগে থেকেই থাকে, তবে ইন-প্লেস এর পরিমাণ বৃদ্ধি করা হয়। পুরো ভেক্টর স্ক্যান করার পরও না পাওয়া গেলে নতুন `CartItem` পুশ করা হয়।

### ২. `entry().or_insert()`-এর জাদু
`count_products_by_department` মেথডে:
```rust
*counts.entry(dept).or_insert(0) += 1;
```
যদি ডিপার্টমেন্ট কি-টি ম্যাপে থাকে, তবে `or_insert(0)` পূর্ববর্তী কাউন্টের মিউটেবল রেফারেন্স `&mut u32` দেয়। আর না থাকলে `0` ইনসার্ট করে সেই নতুন রেফারেন্সটি দেয়। শুরুর `*` ডিরেফারেন্স করে এক নিমেষেই `+= 1` যোগ করে দেয়।

---

## সাধারণ ভুলসমূহ (Common Mistakes)

### ১. ইটারেশন চলাকালীন ভেক্টরে পুশ করার চেষ্টা করা
```rust
let mut v = vec![1, 2, 3];
for item in &v {
    if *item == 2 {
        v.push(10); // কম্পাইলার এরর! cannot borrow `v` as mutable while borrowed as immutable
    }
}
```
জাভায় এটি রানটাইমে `ConcurrentModificationException` দেয়, আর সি++ এ মেমোরি ধ্বংস করে। রাস্টে কম্পাইলার এই কোড বিল্ড হতেই দেবে না।

### ২. হাশম্যাপে অ্যান্টি-প্যাটার্ন (খোঁজা তারপর ইনসার্ট করা)
নিচের মতো কোড পরিহার করুন:
```rust
if !map.contains_key(&key) {
    map.insert(key.clone(), 0);
}
*map.get_mut(&key).unwrap() += 1; // অপ্রয়োজনীয় ডাবল হাশ গণনা!
```
সর্বোচ্চ পারফরম্যান্সের জন্য সবসময় `map.entry(key).or_insert(...)` ব্যবহার করুন।

---

## কম্পাইলার এরর বিশ্লেষণ (Compiler Errors)

### Error E0502: রেফারেন্স ধরে রেখে কালেকশন পরিবর্তন
মনে করুন আপনি লিখেছেন:

```rust
let mut cart = vec![CartItem::new(1, 2, 1000)];
let first_item = &cart[0]; // ভেক্টরের হিপ উপাদানের ইমিউটেবল রেফারেন্স
cart.push(CartItem::new(2, 1, 2500)); // ভেক্টর পরিবর্তনের চেষ্টা
println!("First item: {}", first_item.unit_price_cents);
```

কম্পাইলার সাথে সাথে কোড আটকে দেবে:

```text
error[E0502]: cannot borrow `cart` as mutable because it is also borrowed as immutable
  --> src/main.rs:190:5
   |
189|     let first_item = &cart[0];
   |                       ---- immutable borrow occurs here
190|     cart.push(CartItem::new(2, 1, 2500));
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
191|     println!("First item: {}", first_item.unit_price_cents);
   |                                ---------- immutable borrow later used here
```

**কেন এটি গুরুত্বপূর্ণ**: `push` করার ফলে ভেক্টর যদি হিপের জায়গা পরিবর্তন করে ফেলে, তবে `first_item` খালি মেমোরির দিকে নির্দেশ করত (Dangling Pointer)। বরো চেকার থাকায় এই বাগ প্রোগ্রামে ঢুকতেই পারে না।

---

## অনুশীলনী (Practice)
১. **কার্ট খালি করা**: `ShoppingCart`-এ একটি মেথড লিখুন `pub fn clear(&mut self)` যা কার্টের সমস্ত আইটেম মুছে ফেলবে।
২. **ডিপার্টমেন্ট ফিল্টার**: একটি ফাংশন লিখুন `filter_products_by_dept(products: &[Product], dept: &str) -> Vec<Product>` যা নির্দিষ্ট ডিপার্টমেন্টের পণ্যগুলো নতুন ভেক্টরে ফিল্টার করে দেবে।
৩. **টেস্টে প্রমাণ করুন**: `src/main.rs`-এ টেস্ট লিখে প্রমাণ করুন যে `clear` কার্ট খালি করে এবং `filter_products_by_dept` সঠিক পণ্য ফেরত দেয়।
৪. `cargo test` দিয়ে নিশ্চিত করুন।

---

## চেকপয়েন্ট (Checkpoint)
- [x] ডাইনামিক হিপ ভেক্টর `Vec<T>`-এর অভ্যন্তরীণ রূপ বুঝেছেন।
- [x] হাশম্যাপ `HashMap<K, V>` দিয়ে $O(1)$ গতিতে ডাটা কি-ভ্যালু সংরক্ষণ ও রিড করেছেন।
- [x] হাশম্যাপের এন্ট্রি এপিআই (`.entry().or_insert()`) আয়ত্ত করেছেন।
- [x] `HashSet<T>` ব্যবহার করে ডুপ্লিকেট ডাটা প্রতিরোধ ও ট্যাগিং শিখেছেন।
- [x] রেফারেন্স থাকা অবস্থায় কালেকশন মিউটেশন কেন নিষিদ্ধ তা প্রমাণসহ দেখেছেন।
- [x] MiniStore-কে একটি পূর্ণাঙ্গ মাল্টি-আইটেম ই-কমার্সে রূপান্তর করেছেন।

---

## আমরা কী শিখলাম
- `Vec<T>`, `HashMap<K, V>`, এবং `HashSet<T>` হলো বাস্তব জগতের জটিল ডাটা সামলানোর ভিত্তি।
- ওনারশিপ এবং RAII-এর কারণে প্রতিটি কালেকশন স্কোপ ছাড়ার সাথে সাথে তার সমস্ত হিপ মেমোরি কোনো লিকেজ ছাড়াই স্বয়ংক্রিয়ভাবে ক্লিনআপ হয়।
- MiniStore এখন সম্পূর্ণ গতিশীল কার্ট, ডিপার্টমেন্টভিত্তিক বিশ্লেষণ এবং বহু-পণ্যের অর্ডার সমর্থন করে।

---

## পরবর্তীতে কী আসছে
এর মাধ্যমেই সমাপ্ত হলো **পার্ট ১ — ওনারশিপ**!
পরবর্তী **পার্ট ২ — বিজনেস লজিক মডেলিং**-এর সূচনা হবে **অধ্যায় ১০: এনাম এবং প্যাটার্ন ম্যাচিং (Enums and Pattern Matching)** দিয়ে, যেখানে আমরা শিখব রাস্টের অ্যালজেব্রেইক ডাটা টাইপ দিয়ে কীভাবে বাস্তব ব্যবসার নানা স্ট্যাটাস ও ট্রানজ্যাকশন মডেল করা যায়।
