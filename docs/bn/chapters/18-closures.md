# অধ্যায় ১৮: ক্লোজারস (Closures)

## আপনি যা শিখবেন
- **ক্লোজারস (Closures)** কী এবং সাধারণ ফাংশন (`fn`)-এর সাথে এদের পার্থক্য কোথায়।
- ক্লোজার সিনট্যাক্স এবং রাস্টের শক্তিশালী **টাইপ ইনফারেন্স (Type Inference)** (`|x| x + 1`)।
- ক্লোজার কীভাবে তার পারিপার্শ্বিক স্কোপ থেকে ভেরিয়েবল ক্যাপচার করে:
  - **অপরিবর্তনীয় রেফারেন্স ধার নেওয়া** (`&T`)।
  - **পরিবর্তনযোগ্য রেফারেন্স ধার নেওয়া** (`&mut T`)।
  - **সম্পূর্ণ ওনারশিপ গ্রহণ করা** (`T`) **`move`** কিওয়ার্ডের মাধ্যমে।
- তিনটি স্ট্যান্ডার্ড ক্লোজার ট্রেইট এবং তাদের ইনহেরিটেন্স হায়ারার্কি:
  - **`FnOnce`**: অন্তত একবার কল করা যায়; ক্যাপচার করা ভেরিয়েবলকে কনজিউম বা ধ্বংস করতে পারে।
  - **`FnMut`**: বারবার কল করা যায়; ক্যাপচার করা স্টেট ইন-প্লেস মিউটেট (পরিবর্তন) করতে পারে।
  - **`Fn`**: বারবার এবং একই সাথে কনকারেন্টলি কল করা যায়; ক্যাপচার করা ভেরিয়েবলকে কেবল রিড-অনলি হিসেবে ধার করে।
- জেনেরিক ট্রেইট বাউন্ড দিয়ে ক্লোজার আর্গুমেন্ট গ্রহণকারী ফাংশন লেখা (`F: Fn(&Product) -> bool` বা `F: FnMut(&CartItem) -> u32`)।
- **`impl Fn`** এবং `move` ব্যবহার করে ফ্যাক্টরি ফাংশন থেকে ডায়নামিক ক্লোজার রিটার্ন করার নিয়ম।
- কেন কম্পাইলার প্রতিটি ক্লোজারের জন্য একটি **অনন্য, নামহীন কংক্রিট টাইপ (Unique Unnameable Type)** তৈরি করে।
- সাধারণ কম্পাইলার এরর এবং সমাধানের উপায়:
  - `error[E0507]: cannot move out of captured variable`
  - `error[E0382]: use of moved value in closure`
  - `error[E0308]: distinct uses of impl Trait result in different opaque types`
- MiniStore অ্যাপ্লিকেশনে ক্লোজার সংহতকরণ:
  - `Catalog::find_products`-এ প্রেডিকেট ক্লোজার দিয়ে ডায়নামিক প্রোডাক্ট সার্চ।
  - `src/promotions.rs`-এ ক্লোজার ডিসকাউন্ট ফ্যাক্টরি তৈরি (`make_percentage_discount`, `make_threshold_discount`, `make_vip_discount`)।
  - `ShoppingCart`-এ `FnMut` ব্যবহার করে স্টেটফুল ব্যাচ ডিসকাউন্ট প্রয়োগ।
  - `DiscountAuditor` দিয়ে ডিসকাউন্ট হিসাব সংরক্ষণ।

---

## কেন আমাদের এটি প্রয়োজন?

অধ্যায় ৩-এ আমরা `fn` কিওয়ার্ড দিয়ে সাধারণ ফাংশন লিখতে শিখেছিলাম:
```rust
fn is_discount_eligible(subtotal_cents: u32) -> bool {
    subtotal_cents >= 10000
}
```

সাধারণ ফাংশনের একটি বড় সীমাবদ্ধতা রয়েছে: **যেখানে ফাংশনটি কল করা হচ্ছে, সেই চারপাশের পরিবেশের (Scope) লোকাল ভেরিয়েবলগুলো এটি নিজে থেকে দেখতে বা ব্যবহার করতে পারে না।**

MiniStore-এর বাস্তব একটি সমস্যা বিবেচনা করুন:
একজন ক্রেতা সার্চ বক্সে তার বাজেট লিখে দিলেন (যেমন: সর্বোচ্চ $১০০), এবং আমরা সেই গতিশীলভাবে নির্ধারিত বাজেটের ওপর ভিত্তি করে ক্যাটালগের প্রোডাক্ট ফিল্টার করতে চাই:

```rust
let max_budget = 10000; // ব্যবহারকারীর দেওয়া লোকাল ভেরিয়েবল

// আমরা এখানে কোনো সাধারণ `fn` ফাংশন পাস করতে পারব না, কারণ সাধারণ ফাংশন
// `max_budget` ভেরিয়েবলটি দেখতে পায় না!
```

### সমাধান: ক্লোজারস (Closures)
একটি **ক্লোজার** হলো একটি বেনামী ফাংশন (Anonymous Function), যা কোনো ভেরিয়েবলে সংরক্ষণ করা যায়, অন্য ফাংশনে আর্গুমেন্ট হিসেবে পাঠানো যায় এবং সবচেয়ে গুরুত্বপূর্ণ—**যে স্কোপে এটি তৈরি হয়েছে, সেই স্কোপের ভেরিয়েবলগুলোকে ক্যাপচার (Capture) করতে পারে**:

```rust
let max_budget = 10000;

// ক্লোজার `|p|` সরাসরি চারপাশের স্কোপ থেকে `max_budget` ধার করে নেয়!
let affordable_products = catalog.find_products(|p| p.price_cents <= max_budget);
```

---

## ক্লোজার সিনট্যাক্স এবং টাইপ ইনফারেন্স

ক্লোজারে প্যারামিটার সংজ্ঞায়িত করতে বন্ধনীর `(...)` পরিবর্তে দুটি উলম্ব পাইপ চিহ্ন (`|...|`) ব্যবহার করা হয়:

```rust
// ১. সাধারণ ফাংশন
fn add_one_fn(x: u32) -> u32 { x + 1 }

// ২. সম্পূর্ণ টাইপ অ্যানোটেশন সহ ক্লোজার
let add_one_v1 = |x: u32| -> u32 { x + 1 };

// ৩. অনুমিত (Inferred) প্যারামিটার ও রিটার্ন টাইপ সহ ক্লোজার
let add_one_v2 = |x| x + 1;

// ৪. একাধিক স্টেটমেন্টের ব্লক বডি সহ ক্লোজার
let calculate_tax = |price: u32| {
    let rate = 15;
    (price * rate) / 100
};
```

### টাইপ লকিং (Type Locking)
সাধারণ ফাংশনের মতো ক্লোজারে সচরাচর টাইপ লিখতে হয় না, কারণ প্রথমবার যখন ক্লোজারটি কল করা হয়, কম্পাইলার নিজেই তার টাইপ অনুমান করে নেয়।

তবে একবার টাইপ নির্ধারিত হয়ে গেলে, **সেই টাইপটি লক হয়ে যায়**:
```rust
let print_item = |x| println!("{}", x);

print_item(String::from("Keyboard")); // টাইপ `String` হিসেবে লক হয়ে গেল
// print_item(101); // COMPILE ERROR: প্রত্যাশিত টাইপ `String`, কিন্তু পাওয়া গেছে পূর্ণসংখ্যা!
```

---

## ক্লোজার কীভাবে পরিবেশ ক্যাপচার করে

যখন কোনো ক্লোজার তার বাইরের স্কোপের ভেরিয়েবল ব্যবহার করে, তখন এটি তিনটি ভিন্ন উপায়ে সেই ভেরিয়েবলকে ক্যাপচার করতে পারে:

### ১. অপরিবর্তনীয় রেফারেন্স ধার নিয়ে (`&T`)
ক্লোজার যদি ভেরিয়েবলটি কেবল পড়ে, তবে এটি একটি ইমিউটেবল রেফারেন্স ধার নেয়:
```rust
let store_name = String::from("MiniStore");
let greet = || println!("Welcome to {}", store_name);

greet();
println!("Store: {}", store_name); // সম্পূর্ণ বৈধ: store_name কেবল রিড করা হয়েছে
```

### ২. পরিবর্তনযোগ্য রেফারেন্স ধার নিয়ে (`&mut T`)
ক্লোজার যদি ভেরিয়েবলটির মান পরিবর্তন করে, তবে এটি একটি মিউটেবল রেফারেন্স ধার নেয়:
```rust
let mut total_discount = 0;
let mut record_discount = |amount| {
    total_discount += amount; // ক্যাপচার করা ভেরিয়েবল মিউটেট করছে
};

record_discount(500);
record_discount(1000);
println!("Total discount: ${:.2}", total_discount as f64 / 100.0); // $15.00
```
লক্ষ্য করুন: ক্লোজার ভেরিয়েবলটিকে অবশ্যই `let mut` হিসেবে ঘোষণা করতে হবে, কারণ এটি নিজে স্টেট পরিবর্তন করছে।

### ৩. সম্পূর্ণ ওনারশিপ গ্রহণ করে (`move`)
ক্লোজারকে যদি ক্যাপচার করা ভ্যালুর সম্পূর্ণ **ওনারশিপ** দিতে চান, তবে ক্লোজারের শুরুতে **`move`** কিওয়ার্ড যুক্ত করুন:
```rust
let greeting = String::from("Welcome");
let print_welcome = move || {
    println!("{}", greeting); // `greeting` ক্লোজারের ভেতরে MOVED হয়ে গেল!
};

print_welcome();
// println!("{}", greeting); // COMPILE ERROR: move হওয়ার পর ভ্যালু আর ব্যবহার করা যাবে না!
```

ফাংশন থেকে ক্লোজার রিটার্ন করার সময় বা ব্যাকগ্রাউন্ড থ্রেডে ক্লোজার পাঠানোর সময় `move` কিওয়ার্ড অত্যন্ত অপরিহার্য।

---

## তিনটি ক্লোজার ট্রেইট: `FnOnce`, `FnMut`, এবং `Fn`

ক্লোজার তার ক্যাপচার করা ডেটার সাথে কেমন আচরণ করে, তার ওপর ভিত্তি করে রাস্ট স্বয়ংক্রিয়ভাবে এক বা একাধিক ট্রেইট বরাদ্দ করে:

```
          ┌────────────┐
          │   FnOnce   │ (অন্তত একবার কল করা যায়; ক্যাপচার করা ভ্যালু কনজিউম করতে পারে)
          └─────▲──────┘
                │
          ┌─────┴──────┐
          │   FnMut    │ (বারবার কল করা যায়; ক্যাপচার করা ভ্যালু মিউটেট করতে পারে)
          └─────▲──────┘
                │
          ┌─────┴──────┐
          │     Fn     │ (বারবার কল করা যায়; ক্যাপচার করা ভ্যালু শুধু রিড করে)
          └────────────┘
```

### ১. `FnOnce`
- সমস্ত ক্লোজার এই ট্রেইট বাস্তবায়ন করে।
- অন্তত একবার কল করা যায়।
- কোনো ক্লোজার যদি ক্যাপচার করা ভ্যালুকে মুভ বা ড্রপ করে দেয়, তবে এটি শুধুমাত্র `FnOnce` বাস্তবায়ন করে এবং দ্বিতীয়বার কল করা যায় না।

### ২. `FnMut`
- এটি `FnOnce` থেকে ইনহেরিট করে।
- যেসব ক্লোজার ক্যাপচার করা ভ্যালু মিউটেট করে কিন্তু পুরোপুরি ধ্বংস করে না, তারা `FnMut`।
- একাধিকবার ক্রমান্বয়ে কল করা যায়।

### ৩. `Fn`
- এটি `FnMut` (এবং সেই সূত্রে `FnOnce`) থেকে ইনহেরিট করে।
- যেসব ক্লোজার ক্যাপচার করা ভ্যালুকে কেবল রিড-অনলি ধার করে বা কোনো পরিবেশ ক্যাপচারই করে না, তারা `Fn`।
- একই সাথে একাধিক থ্রেড থেকে নিরাপদভাবে বারবার কল করা যায়।

---

## ফাংশনে ক্লোজার আর্গুমেন্ট গ্রহণ করা

কোনো ফাংশনে ক্লোজার আর্গুমেন্ট হিসেবে গ্রহণ করতে হলে জেনেরিক ট্রেইট বাউন্ড ব্যবহার করতে হয়:

```rust
impl Catalog {
    /// যেকোনো ক্লোজার গ্রহণ করে যা একটি `&Product` নিয়ে `bool` রিটার্ন করে।
    pub fn find_products<P>(&self, predicate: P) -> Vec<&Product>
    where
        P: Fn(&Product) -> bool,
    {
        self.products.values().filter(|p| predicate(p)).collect()
    }
}
```

স্টেটফুল অপারেশনের ক্ষেত্রে `FnMut` গ্রহণ করা হয়:
```rust
impl ShoppingCart {
    /// একটি মিউটেবল ক্লোজার গ্রহণ করে যা আইটেম ভেদে ডিসকাউন্টের হিসাব ট্র্যাক করতে পারে।
    pub fn apply_custom_discount<F>(&mut self, mut discount_calc: F)
    where
        F: FnMut(&CartItem) -> u32,
    {
        for item in &mut self.items {
            let discount = discount_calc(item);
            item.unit_price_cents = item.unit_price_cents.saturating_sub(discount);
        }
    }
}
```

---

## ক্লোজার রিটার্ন করা: `move` সহ ফ্যাক্টরি ফাংশন

MiniStore-এ বিভিন্ন ধরণের প্রমোশন ক্যাম্পেইন (যেমন: ফ্ল্যাশ সেল পার্সেন্টেজ বা ভিআইপি বেনিফিট) রানটাইমে নির্ধারিত হয়।

আমরা **ক্লোজার ফ্যাক্টরি (Closure Factories)** তৈরি করতে পারি যা `impl Fn` এবং `move` ব্যবহার করে কাস্টমাইজড ডিসকাউন্ট ক্লোজার রিটার্ন করে:

```rust
/// একটি নির্দিষ্ট শতকরা হারের ডিসকাউন্ট ক্লোজার তৈরি করে।
pub fn make_percentage_discount(percentage: u32) -> impl Fn(u32) -> u32 {
    move |amount_cents| (amount_cents * percentage) / 100
}

/// একটি নির্দিষ্ট সীমার ডিসকাউন্ট ক্লোজার তৈরি করে ($১৫০ বা তদূর্ধ্ব অর্ডারে $২০ ছাড়)।
pub fn make_threshold_discount(min_cents: u32, discount_cents: u32) -> impl Fn(u32) -> u32 {
    move |amount_cents| {
        if amount_cents >= min_cents {
            discount_cents
        } else {
            0
        }
    }
}
```

এখন রানটাইমে যেকোনো প্রচারমূলক ডিসকাউন্ট তৈরি করা অত্যন্ত সহজ:
```rust
let flash_sale = make_percentage_discount(15);
println!("Discount on $100: ${:.2}", flash_sale(10000) as f64 / 100.0); // $15.00
```

---

## কেন প্রতিটি ক্লোজারের একটি অনন্য টাইপ থাকে

রাস্টের অত্যন্ত গুরুত্বপূর্ণ একটি বৈশিষ্ট্য: **কম্পাইলার প্রতিটি ক্লোজারের জন্য সম্পূর্ণ আলাদা, নামহীন অনন্য কংক্রিট টাইপ তৈরি করে**।

এমনকি দুটি ক্লোজারের প্যারামিটার ও রিটার্ন টাইপ হুবহু এক হলেও:
```rust
let f1 = |x: u32| x + 1;
let f2 = |x: u32| x + 2;

// COMPILE ERROR: mismatched types!
// let closures = [f1, f2];
```
কম্পাইলার `f1`-এর জন্য একটি বেনামী স্ট্রাক্ট তৈরি করে এবং `f2`-এর জন্য আরেকটি ভিন্ন বেনামী স্ট্রাক্ট তৈরি করে।
যেহেতু অ্যারে `[T; N]`-এর প্রতিটি উপাদানের টাইপ হুবহু এক হতে হয়, তাই ভিন্ন দুটি ক্লোজার সরাসরি সাধারণ অ্যারেতে রাখা যায় না!

একাধিক ক্লোজার একসাথে সংগ্রহ করতে হলে:
১. হয়তো একই ফ্যাক্টরি ফাংশন থেকে তৈরি করতে হবে (`make_percentage_discount`),
২. অথবা ফাংশন পয়েন্টার `fn(u32) -> u32` ব্যবহার করতে হবে (যদি কোনো পরিবেশ ক্যাপচার না থাকে),
৩. অথবা ট্রেইট অবজেক্ট (`Box<dyn Fn>` বা `&dyn Fn`) ব্যবহার করতে হবে।

---

## অন্যান্য ভাষার সাথে তুলনা

| বৈশিষ্ট্য | রাস্ট (Rust) | জাভাস্ক্রিপ্ট (JavaScript) | পাইথন (Python) | জাভা (Java) | সি++ (C++) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **সিনট্যাক্স** | `\|x\| x + 1` | `x => x + 1` | `lambda x: x + 1` | `x -> x + 1` | `[=](int x) { return x + 1; }` |
| **ক্যাপচার মোড** | `&T`, `&mut T`, `move` | শেয়ার্ড রেফারেন্স (GC) | শেয়ার্ড রেফারেন্স (GC) | কেবল Effectively final ভ্যালু | `[&]`, `[=]`, `[move]` |
| **ট্রেইট / টাইপ** | `Fn`, `FnMut`, `FnOnce` | `Function` অবজেক্ট | ফাংশন অবজেক্ট | ফাংশনাল ইন্টারফেস (`Function<T, R>`) | `std::function` / Lambda auto |
| **রানটাইম খরচ** | **শূন্য ওভারহেড** (ইনলাইনড স্ট্রাক্ট) | হিপ অবজেক্ট + জিসি ওভারহেড | ইন্টারপ্রেটার ওভারহেড | জেভিএম বক্সিং + জিসি ওভারহেড | ইনলাইনড স্ট্রাক্ট (জিরো-কস্ট) |

---

## MiniStore আর্কিটেকচার এবং কোড ইমপ্লিমেন্টেশন

MiniStore-এ ক্লোজার কীভাবে সাজানো হয়েছে:

```
ministore/
└── src/
    ├── promotions.rs       # ক্লোজার ফ্যাক্টরি: make_percentage_discount, make_threshold_discount, make_vip_discount
    ├── models/
    │   ├── cart.rs         # apply_custom_discount(FnMut)
    │   └── mod.rs
    ├── catalog.rs          # find_products(Fn)
    ├── lib.rs              # রি-এক্সপোর্ট এবং ৩৪টি ইউনিট টেস্ট
    └── main.rs             # ডায়নামিক প্রমোশন, বাজেট ফিল্টারিং এবং স্টেটফুল কার্ট ডিসকাউন্টের ডেমো
```

### ১. `src/promotions.rs`
```rust
use crate::models::Customer;

pub fn make_percentage_discount(percentage: u32) -> impl Fn(u32) -> u32 {
    move |amount_cents| (amount_cents * percentage) / 100
}

pub fn make_threshold_discount(min_cents: u32, discount_cents: u32) -> impl Fn(u32) -> u32 {
    move |amount_cents| {
        if amount_cents >= min_cents {
            discount_cents
        } else {
            0
        }
    }
}

pub fn make_vip_discount(vip_rate: u32) -> impl Fn(&Customer, u32) -> u32 {
    move |customer, amount_cents| {
        if customer.is_vip || customer.has_tag("vip") {
            (amount_cents * vip_rate) / 100
        } else {
            0
        }
    }
}

pub fn calculate_total_promotions<F>(amount_cents: u32, rules: &[F]) -> u32
where
    F: Fn(u32) -> u32,
{
    rules.iter().map(|rule| rule(amount_cents)).sum()
}

pub fn find_best_promotion<F>(amount_cents: u32, rules: &[F]) -> u32
where
    F: Fn(u32) -> u32,
{
    rules.iter().map(|rule| rule(amount_cents)).max().unwrap_or(0)
}

pub struct DiscountAuditor {
    pub total_discount_given: u32,
    pub operations_count: u32,
}

impl DiscountAuditor {
    pub fn new() -> Self {
        Self {
            total_discount_given: 0,
            operations_count: 0,
        }
    }

    pub fn record(&mut self, discount_cents: u32) {
        self.total_discount_given += discount_cents;
        self.operations_count += 1;
    }
}
```

---

## সাধারণ কম্পাইলার এরর এবং সমাধানের উপায়

### ১. `error[E0507]: cannot move out of captured variable in Fn/FnMut closure`
**ভুল কোড**:
```rust
let name = String::from("VIP Voucher");
let consume_voucher = || {
    let owned = name; // COMPILE ERROR: বারবার কল হতে পারে এমন ক্লোজার থেকে ভ্যালু মুভ করা যাবে না!
};
```
**কেন ঘটে**:
যেহেতু ক্লোজারটি একাধিকবার কল হতে পারে, প্রথমবার `name` মুভ হয়ে গেলে দ্বিতীয়বার কল করার সময় আর কোনো ডাটা থাকবে না।
**সমাধান**:
ভেতরে ওনড ডেটার প্রয়োজন হলে `.clone()` করুন, অথবা ক্লোজারটি যদি সত্যিই একবারই চলে তবে `FnOnce` ব্যবহার করুন।

---

### ২. `error[E0382]: borrow of moved value`
**ভুল কোড**:
```rust
let threshold = 5000;
let promo = move |x| x >= threshold;
println!("{}", threshold); // COMPILE ERROR: move হওয়ার পর ভ্যালু ব্যবহারের চেষ্টা
```
**কেন ঘটে**:
`move` কিওয়ার্ড সমস্ত ক্যাপচার করা ভেরিয়েবলকে ক্লোজারে স্থানান্তর করে।
**সমাধান**:
বাইরের স্কোপে ভ্যালুটির প্রয়োজন থাকলে মুভ করার আগে ক্লোন করে নিন।

---

## ইডিওম্যাটিক রাস্ট বেস্ট প্র্যাকটিস

১. **কম সীমাবদ্ধ ট্রেইট ব্যবহার করুন**: ক্লোজার গ্রহণকারী ফাংশন লেখার সময় যতটুকু সম্ভব `Fn` অগ্রাধিকার দিন। এতে কলার সর্বোচ্চ নমনীয়তার সাথে একাধিকবার ক্লোজারটি চালাতে পারবে।
২. **কনফিগারেবল লজিকের জন্য `move` সহ ফ্যাক্টরি ফাংশন ব্যবহার করুন**: ডজন ডজন কনফিগারেশন প্যারামিটার সরাসরি ফাংশনে না পাঠিয়ে হাইয়ার-অর্ডার ফ্যাক্টরি (`make_percentage_discount`) দিয়ে লজিক ক্যাপসুল তৈরি করুন।
৩. **ক্লোজারকে ছোট ও সুনির্দিষ্ট রাখুন**: ক্লোজার ১-৫ লাইনের মধ্যে সীমাবদ্ধ থাকলে সবচেয়ে ভালো পঠনযোগ্য হয়।
৪. **অহেতুক টাইপ অ্যানোটেশন পরিহার করুন**: কম্পাইলারের টাইপ ইনফারেন্সের সুবিধা নিন।

---

## বাস্তবমুখী অনুশীলন (Exercises)

### অনুশীলন ১: শর্তসাপেক্ষ বাল্ক ডিসকাউন্ট
১. `ShoppingCart`-এ একটি মেথড লিখুন `discount_if<P>(cart: &mut ShoppingCart, predicate: P, discount_cents: u32)` যেখানে `P: Fn(&CartItem) -> bool`।
২. শুধুমাত্র সেইসব কার্ট আইটেমে ছাড় দিন যা ক্লোজারের শর্ত পূরণ করে (যেমন: পরিমাণ >= ৩)।

### অনুশীলন ২: ক্যাটাগরি সারচার্জ ফ্যাক্টরি
১. একটি ক্লোজার ফ্যাক্টরি তৈরি করুন `make_category_surcharge(target_category: ProductCategory, fee_cents: u32) -> impl Fn(&Product) -> u32`।
২. প্রোডাক্টটি যদি নির্দিষ্ট ক্যাটাগরির হয় তবে `fee_cents` যোগ হবে, অন্যথায় ০।

---

## চেকপয়েন্ট (Checkpoint)

কম্পাইলার চেক চালিয়ে নিশ্চিত করুন যে ৩৪টি টেস্টই সফলভাবে পাস করেছে:
```bash
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
cargo run
```

প্রত্যাশিত টেস্ট আউটপুট:
```text
running 34 tests
test tests::test_cart_item_option_lookup ... ok
test tests::test_catalog_iterator_queries ... ok
test tests::test_catalog_option_lookups ... ok
test tests::test_checkout_error_propagation_and_success ... ok
test tests::test_closure_dynamic_filtering_in_catalog ... ok
test tests::test_closure_factories_with_move ... ok
test tests::test_coupon_discount_and_take ... ok
test tests::test_coupon_validation_error ... ok
test tests::test_custom_cart_report_iterator ... ok
test tests::test_custom_discount_tier_iter ... ok
test tests::test_customer_optional_phone ... ok
test tests::test_display_trait_implementations ... ok
test tests::test_evaluate_promotions_and_auditor ... ok
test tests::test_fn_mut_stateful_cart_discount ... ok
test tests::test_generic_api_response_wrapper ... ok
test tests::test_generic_catalog_product_pagination ... ok
test tests::test_generic_page_map_transformation ... ok
test tests::test_generic_pagination_with_integers ... ok
test tests::test_generic_trait_bound_functions ... ok
test tests::test_lifetime_annotated_contact_resolution ... ok
test tests::test_lifetime_annotated_product_comparison ... ok
test tests::test_order_cancellation_prevention ... ok
test tests::test_order_iterator_methods ... ok
test tests::test_order_status_valid_lifecycle ... ok
test tests::test_order_total_with_payment_fee ... ok
test tests::test_payment_method_fees_and_descriptions ... ok
test tests::test_product_category_tax_rates ... ok
test tests::test_product_stock_reduction_error ... ok
test tests::test_shopping_cart_iterators_and_into_iterator ... ok
test tests::test_static_lifetime_policy ... ok
test tests::test_summarizable_trait_on_domain_models ... ok
test tests::test_taxable_trait_and_default_method ... ok
test tests::test_vip_discount_closure ... ok
test tests::test_zero_copy_order_receipt_and_traits ... ok

test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

প্রত্যাশিত সিএলআই আউটপুট:
```text
=== MiniStore: Closures & Dynamic Policies (Part III) ===

1. Catalog initialized with 3 products.

2. Catalog Iteration & Functional Queries:
   Found 3 Electronics product(s): ["Tenkeyless Mechanical Keyboard", "Ergonomic Wireless Mouse", "27-inch 4K IPS Display"]
   Products under $150: Tenkeyless Mechanical Keyboard ($120.00), Ergonomic Wireless Mouse ($45.00)
   Total Inventory Valuation: $2100.00

3. Dynamic Product Search via Closures:
   Products within budget ($100.00): ["Ergonomic Wireless Mouse"]

4. Browsing Catalog with Generic Pagination (Page<Product>):
   Page 1 of 2 (Total Items: 3)
   - [TECH-KEY-001] Tenkeyless Mechanical Keyboard ($120.00)
   - [TECH-MOU-002] Ergonomic Wireless Mouse ($45.00)
   Has next page? true
   Transformed to Page<String>: ["Tenkeyless Mechanical Keyboard", "Ergonomic Wireless Mouse"]
   API Page 2 response: 1 product(s) returned out of 3 total.

5. Customer: Margaret Hamilton (+1-555-0199)

6. Dynamic Promotion Factories via Closures:
   Simulating discounts on sample subtotal ($210.00):
   - 10% General Sale:   $21.00
   - $20 over $150:      $20.00
   - VIP Member Benefit: $21.00

7. Lifetimes & Reference Safety:
   Higher priced item: Tenkeyless Mechanical Keyboard ($120.00)
   Best contact info: +1-555-0199
   Store Policy ('static): MiniStore Guarantee: 30-Day Hassle-Free Returns & 1-Year Warranty

8. Shared Behaviors via Traits:
   Tax Summary: Product #101: Tenkeyless Mechanical Keyboard [TECH-KEY-001] - $120.00 | Tax: $18.00 (15%)
   Tax Summary: Product #102: Ergonomic Wireless Mouse [TECH-MOU-002] - $45.00 | Tax: $6.75 (15%)
   Customer Summary: Customer #301: Margaret Hamilton <margaret@apollo.nasa.gov>

9. Cart Iteration & Stateful Closures (FnMut):
   Iterating over cart items directly:
   -> Product ID #101: Qty 1 @ $120.00 each
   -> Product ID #102: Qty 2 @ $45.00 each
   Custom Cart Line Item Reports:
      Item #1: Product #101 (Qty: 1) - $120.00
      Item #2: Product #102 (Qty: 2) - $90.00
   Progressive Discount Tiers available: [5, 10, 15, 20]%
   Stateful FnMut applied $5.00 discount on multi-quantity items!

10. Processing checkout through modular services...
   Checkout Order #901 created successfully!
   Subtotal: $200.00 | Total Units: 3 | Final Total: $141.50

11. Order Lifecycle Transitions:
   Order confirmed: Confirmed (Receipt: REC-901-HAMILTON)
   Order shipped:   Shipped (Tracking: TRK-FEDEX-77189)
   Cancellation prevented -> Cannot perform action 'cancel' while order is in 'Shipped (Tracking: TRK-FEDEX-77189)' state
   Final Lifecycle State: Delivered to Customer
   Order Summary: Order #901: 2 item(s), Total: $141.50 [Delivered to Customer]

12. Zero-Copy Receipt Borrowing Order & Slices (OrderReceipt<'a>):
   Display format: Receipt for Order #901 (Margaret Hamilton) - Total: $141.50
   Trait Summary:  Receipt: Order #901 for Margaret Hamilton | Total: $141.50

--- Printed Slip ---
=== RECEIPT: #901 ===
Customer: Margaret Hamilton
Items: 2
Total: $141.50
Status: Delivered to Customer
Note: VIP Client - Express courier delivery verified
===================
--------------------
```

---

## আমরা যা শিখলাম
- ক্লোজার কীভাবে তার পারিপার্শ্বিক স্কোপ থেকে ভেরিয়েবল ধার (`&T`, `&mut T`) বা মুভ (`move`) করে।
- `Fn`, `FnMut`, এবং `FnOnce`-এর মধ্যকার কার্যকরী পার্থক্য।
- কীভাবে জেনেরিক ট্রেইট বাউন্ড দিয়ে ফাংশনে ক্লোজার আর্গুমেন্ট গ্রহণ করা যায়।
- কীভাবে `impl Fn` দিয়ে ডায়নামিক ফ্যাক্টরি ফাংশন তৈরি করা যায়।
- কেন প্রতিটি ক্লোজারের জন্য কম্পাইলার সম্পূর্ণ স্বতন্ত্র টাইপ তৈরি করে।

---

## পরবর্তী অধ্যায়
জেনেরিকস, ট্রেইটস, লাইফটাইমস, ইটারেটরস এবং ক্লোজারস আয়ত্ত করার পর আমাদের এমন কিছু পরিস্থিতির মুখোমুখি হতে হয় যেখানে ডেটাকে সরাসরি হিপে রাখতে হয়, একাধিক ওনারের মাঝে শেয়ার করতে হয়, অথবা ইমিউটেবল রেফারেন্সের ভেতর থেকে ডেটা পরিবর্তন করতে হয়!
**অধ্যায় ১৯: স্মার্ট পয়েন্টারস (Smart Pointers)**-এ আমরা `Box<T>`, `Rc<T>`, `RefCell<T>` এবং ইন্টারিয়র মিউটেবিলিটি (Interior Mutability) সম্পর্কে বিস্তারিত জানব!
