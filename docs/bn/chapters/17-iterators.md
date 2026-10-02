# অধ্যায় ১৭: ইটারেটরস (Iterators)

## আপনি যা শিখবেন
- **`Iterator` ট্রেইট** কী এবং রাস্ট কীভাবে ডিক্লেয়ারেটিভ ও ফাংশনাল পদ্ধতিতে সিকোয়েন্স প্রসেসিং বাস্তবায়ন করে।
- ইটারেটর তৈরির তিনটি মৌলিক ধরন:
  - **`.iter()`**: ধার করা অপরিবর্তনীয় রেফারেন্স (`&T`) প্রদান করে।
  - **`.iter_mut()`**: ধার করা পরিবর্তনযোগ্য রেফারেন্স (`&mut T`) প্রদান করে।
  - **`.into_iter()`**: কালেকশনের ওনারশিপ গ্রহণ করে ওনড ভ্যালু (`T`) প্রদান করে।
- **`IntoIterator`** ট্রেইট বাস্তবায়ন করা যাতে কাস্টম স্ট্রাক্ট যেমন `ShoppingCart`, `Order` এবং `Catalog`-এর ওপর সরাসরি `for item in &cart` লুপ চালানো যায়।
- **অলস মূল্যায়ন (Lazy Evaluation)**: কেন কোনো কনজিউমার মেথড কল না করা পর্যন্ত ইটারেটর অ্যাডাপ্টার কোনো কাজই করে না এবং কীভাবে এটি কম্পাইলারকে লুপ ফিউশন ও বাউন্ডস চেক বাতিল করতে সাহায্য করে।
- **ইটারেটর অ্যাডাপ্টারসমূহ (Iterator Adapters)**:
  - **`.map()`** দিয়ে উপাদান রূপান্তর।
  - **`.filter()`** দিয়ে শর্তসাপেক্ষে উপাদান বাছাই।
  - **`.take()`** এবং **`.skip()`** দিয়ে সিকোয়েন্স স্লাইসিং।
  - **`.enumerate()`** দিয়ে ইনডেক্স ও উপাদান একসাথে পাওয়া।
- **কনজিউমার মেথডসমূহ (Consumer Methods)**:
  - **`.collect()`** এবং টার্বোফিশ (`::<Vec<_>>()`) দিয়ে ফলাফল সংগ্রহ।
  - **`.sum()`**, **`.count()`**, এবং **`.fold()`** দিয়ে মান একত্রীকরণ (Aggregation)।
  - **`.find()`**, **`.position()`**, **`.any()`**, এবং **`.all()`** দিয়ে দ্রুত অনুসন্ধান।
  - **`.for_each()`** দিয়ে ইন-প্লেস রূপান্তর।
- **কাস্টম ইটারেটর তৈরি করা**:
  - `type Item` এবং `fn next(&mut self) -> Option<Self::Item>` দিয়ে নিজস্ব ইটারেটর লেখা।
  - রিসিটের লাইন বাই লাইন রিপোর্ট তৈরির জন্য `CartReportIterator<'a>` তৈরি।
  - ক্রমান্বয়ে ডিসকাউন্টের ধাপ তৈরি করতে `DiscountTierIter` তৈরি।
- কেন রাস্টের ইটারেটর হলো একটি খাঁটি **জিরো-কস্ট অ্যাবস্ট্রাকশন (Zero-Cost Abstraction)** যা সি/সি++-এর ইনডেক্সড `for` লুপের চেয়েও বেশি দ্রুত চলতে পারে।
- MiniStore অ্যাপ্লিকেশনে ইটারেটর সংহতকরণ:
  - `Catalog`-এ ফাংশনাল কুয়েরি যুক্ত করা (`filter_by_category`, `products_in_price_range`, `total_inventory_valuation`)।
  - `ShoppingCart`-এ বাল্ক অপারেশন যোগ করা (`has_product`, `apply_promotional_discount`, `report_iter`)।
  - `&ShoppingCart`, `&mut ShoppingCart`, `ShoppingCart`, `&Order`, এবং `&Catalog`-এর জন্য `IntoIterator` বাস্তবায়ন।

---

## কেন আমাদের এটি প্রয়োজন?

প্রচলিত প্রোগ্রামিং ভাষাগুলোতে কোনো কালেকশনের উপাদানগুলো একে একে প্রক্রিয়াজাত করতে ইনডেক্সড `for` লুপ ব্যবহার করা হয়:

```c
// ট্র্যাডিশনাল C / Java ইনডেক্সড লুপ
for (int i = 0; i < items.length; i++) {
    process(items[i]);
}
```

এই পদ্ধতিটি পরিচিত হলেও এতে তিনটি মারাত্মক সমস্যা রয়েছে:
১. **অফ-বাই-ওয়ান এরর (Off-by-One Errors)**: ভুল করে `<`-এর জায়গায় `<=` লিখে ফেললে ইনডেক্স সীমার বাইরে চলে গিয়ে প্রোগ্রাম ক্র্যাশ করে।
২. **বাউন্ডস চেকিং ওভারহেড (Bounds Checking Overhead)**: মেমরি-নিরাপদ ভাষাগুলোতে প্রতিবার `items[i]` অ্যাক্সেস করার সময় সিপিইউকে পরীক্ষা করতে হয় `i < length` কি না। প্রতিবার এই চেক করার ফলে সিপিইউ ব্রাঞ্চ মিসপ্রেডিকশন ঘটে এবং কার্যক্ষমতা কমে যায়।
৩. **ইম্পারেটিভ বয়লারপ্লেট (Imperative Boilerplate)**: ডেটা ফিল্টার করা, রূপান্তর করা বা যোগফল বের করতে প্রোগ্রামারকে অস্থায়ী ভেরিয়েবল, নেস্টেড `if` শর্ত এবং ইনডেক্স কাউন্টার ম্যানুয়ালি ট্র্যাক করতে হয়।

### রাস্টের সমাধান: `Iterator` ট্রেইট
রাস্টে কালেকশনগুলো ইটারেটর সরবরাহ করে। এর ফলে প্রোগ্রামারকে ইনডেক্স কীভাবে বাড়বে তা না ভেবে শুধু বলতে হয় সে ডাটা নিয়ে **কী করতে চায়** (ফিল্টার, ম্যাপ, যোগ):

```rust
// ডিক্লেয়ারেটিভ, পরিচ্ছন্ন এবং অবিশ্বাস্য দ্রুতগতির কোড
let total: u32 = cart.items
    .iter()
    .filter(|item| item.quantity > 1)
    .map(|item| item.line_total())
    .sum();
```

যেহেতু রাস্ট কম্পাইলার ইটারেটরের শুরু ও শেষের সুনির্দিষ্ট সীমা আগে থেকেই জানে, তাই এটি লুপের ভেতর থেকে **বাউন্ডস চেকিং পুরোপুরি মুছে ফেলে** এবং সম্পূর্ণ লুপটিকে অত্যন্ত দ্রুতগতির অপ্টিমাইজড ভেক্টরাইজড বা SIMD মেশিন কোডে রূপান্তর করে!

---

## `Iterator` ট্রেইটের গঠন

রাস্টের স্ট্যান্ডার্ড লাইব্রেরির সমস্ত ইটারেটর `Iterator` ট্রেইট বাস্তবায়ন করে:

```rust
pub trait Iterator {
    type Item; // অ্যাসোসিয়েটেড টাইপ: যে ধরণের উপাদান এটি প্রদান করে

    fn next(&mut self) -> Option<Self::Item>;

    // ডজন ডজন ডিফল্ট মেথড সম্পূর্ণ বিনামূল্যে পাওয়া যায়!
    // map, filter, fold, sum, collect ইত্যাদি।
}
```

এই চুক্তিটি লক্ষ্য করুন:
১. আপনি নির্ধারণ করবেন `type Item`: ইটারেটরটি কোন ধরনের ডাটা প্রদান করবে।
২. আপনি বাস্তবায়ন করবেন `fn next(&mut self) -> Option<Self::Item>`:
   - যতক্ষণ উপাদান অবশিষ্ট থাকবে, ততক্ষণ এটি `Some(value)` রিটার্ন করবে।
   - উপাদান শেষ হয়ে গেলে এটি `None` রিটার্ন করবে।

আপনি একবার শুধু `next()` মেথডটি লিখে দিলেই রাস্টের স্ট্যান্ডার্ড লাইব্রেরি আপনাকে **৭০টিরও বেশি অ্যাডাপ্টার ও কনজিউমার মেথড সম্পূর্ণ বিনামূল্যে** উপহার দেয়!

---

## ইটারেটর তৈরির তিনটি মৌলিক ধরন

কালেকশনের (যেমন: `Vec<T>` বা `HashMap<K, V>`) ওপর কাজ করার জন্য তিনটি আদর্শ উপায়ে ইটারেটর তৈরি করা যায়:

| মেথড | প্রদত্ত উপাদান | ওনারশিপ প্রভাব | পরবর্তীতে কালেকশন ব্যবহারযোগ্য? |
| :--- | :--- | :--- | :--- |
| **`.iter()`** | `&T` | অপরিবর্তনীয় রেফারেন্স ধার দেয় | হ্যাঁ |
| **`.iter_mut()`** | `&mut T` | পরিবর্তনযোগ্য রেফারেন্স ধার দেয় | হ্যাঁ |
| **`.into_iter()`** | `T` | উপাদানগুলোকে মুভ/কনজিউম করে | না (কালেকশন বিলুপ্ত হয়) |

### ১. `.iter()` — অপরিবর্তনীয় বরোয়িং
যখন কেবল ডেটা পড়ার প্রয়োজন হয়:
```rust
for item in cart.items.iter() {
    println!("Item: {} x ${:.2}", item.quantity, item.unit_price_cents as f64 / 100.0);
}
// `cart` অক্ষত থাকে এবং পরবর্তীতে ব্যবহার করা যায়!
```

### ২. `.iter_mut()` — ইন-প্লেস রূপান্তর
যখন কোনো নতুন ভেক্টর তৈরি না করেই সরাসরি ভেতরের ডেটা পরিবর্তন করতে চান:
```rust
// কার্টের সমস্ত আইটেমে ১০% ডিসকাউন্ট প্রয়োগ
cart.items.iter_mut().for_each(|item| {
    item.unit_price_cents = (item.unit_price_cents * 90) / 100;
});
```

### ৩. `.into_iter()` — ওনারশিপ হস্তান্তর
যখন কালেকশনটির ভেতরের উপাদানগুলোকে অন্য কোথাও স্থানান্তরিত করতে চান:
```rust
let items: Vec<CartItem> = cart.into_iter().collect();
// `cart` এখানে কনজিউম হয়ে গেছে, তাই এটি আর ব্যবহার করা যাবে না!
```

---

## `IntoIterator` ট্রেইট এবং `for` লুপের আসল রহস্য

রাস্টে `for` লুপ আসলে `IntoIterator`-এর একটি সিনট্যাক্টিক সুগার:
```rust
for x in collection {
    // ...
}
```
কম্পাইলার স্বয়ংক্রিয়ভাবে এটিকে রূপান্তরিত করে:
```rust
let mut iter = collection.into_iter();
while let Some(x) = iter.next() {
    // ...
}
```

আমরা যদি আমাদের `&ShoppingCart`-এর জন্য `IntoIterator` ইমপ্লিমেন্ট করি, তবে ব্যবহারকারীরা সরাসরি কার্টের ওপর লুপ চালাতে পারবে:

```rust
impl<'a> IntoIterator for &'a ShoppingCart {
    type Item = &'a CartItem;
    type IntoIter = std::slice::Iter<'a, CartItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}
```

এখন সরাসরি ব্যবহার করা যাবে:
```rust
for item in &cart {
    println!("Product #{}", item.product_id);
}
```

---

## অলস মূল্যায়ন (Lazy Evaluation): ইটারেটর কীভাবে কাজ করে

> [!IMPORTANT]
> **রাস্টে ইটারেটর পুরোপুরি অলস (Lazy)!**
> কোনো অ্যাডাপ্টার যেমন `.map()` বা `.filter()` কল করলে নিজে থেকে কোনো কাজ শুরু হয় না।

নিচের কোডটি লক্ষ্য করুন:
```rust
let numbers = vec![1, 2, 3, 4, 5];
numbers.iter().map(|x| println!("Processing {}", x)); // কম্পাইলার ওয়ার্নিং দেবে: unused `Map`
```
আপনি যদি এটি কম্পাইল করেন, তবে পর্দায় কিছুই প্রিন্ট হবে না! কারণ কেউ এর আউটপুট চায়নি।

### কেন এই অলসতা একটি সুপারপাওয়ার?
অলস মূল্যায়নের কারণে আপনি একের পর এক অপারেশন চেইন করতে পারেন:
```rust
let total: u32 = catalog
    .products
    .values()
    .filter(|p| p.category == ProductCategory::Electronics)
    .filter(|p| p.is_in_stock())
    .map(|p| p.price_cents)
    .sum();
```
মাঝখানে কোনো অস্থায়ী ভেক্টর তৈরি না করেই রাস্ট পুরো পাইপলাইনটিকে একটিমাত্র একক লুপে ফিউজ করে নেয়। প্রতিটি উপাদান মেমরি থেকে সরাসরি সিপিইউ রেজিস্টারের মাধ্যমে ফিল্টার, ম্যাপ ও সাম হয়ে যায়!

---

## অপরিহার্য ইটারেটর অ্যাডাপ্টার এবং কনজিউমার

### ১. অ্যাডাপ্টারসমূহ (নতুন ইটারেটর তৈরি করে)
- **`.map(|x| ...)`**: প্রতিটি উপাদানের ওপর ফাংশন প্রয়োগ করে রূপান্তরিত করে।
- **`.filter(|x| ...)`**: কেবল সত্য হওয়া শর্তের উপাদানগুলোকে রাখে।
- **`.take(n)`**: প্রথম `n`টি উপাদান গ্রহণ করে।
- **`.skip(n)`**: প্রথম `n`টি উপাদান বাদ দিয়ে বাকিগুলো নেয়।
- **`.enumerate()`**: উপাদানের সাথে তার ইনডেক্স `(index, item)` টাপল আকারে দেয়।
- **`.zip(other)`**: দুটি ইটারেটরের উপাদানগুলোকে জোড়ায় জোড়ায় মেলায়।

### ২. কনজিউমারসমূহ (ইটারেটর শেষ পর্যন্ত চালিয়ে ফলাফল বের করে)
- **`.collect()`**: ফলাফলগুলোকে একটি নতুন কালেকশনে (যেমন: `Vec` বা `HashMap`) জড়ো করে।
- **`.sum()` / `.product()`**: গাণিতিক যোগফল বা গুণফল বের করে।
- **`.fold(initial, |acc, x| ...)`**: প্রাথমিক মান থেকে শুরু করে একটি একক মানে রিডিউস করে।
- **`.find(|x| ...)`**: প্রথম মিলে যাওয়া উপাদানের `Option<&T>` ফেরত দেয়।
- **`.any(|x| ...)` / `.all(|x| ...)`**: যেকোনো একটি বা সব উপাদান শর্ত পূরণ করে কি না যাচাই করে।
- **`.for_each(|x| ...)`**: প্রতিটি উপাদানের ওপর কোনো পার্শ্বপ্রতিক্রিয়া (যেমন মিউটেশন বা প্রিন্ট) চালায়।

---

## কাস্টম ইটারেটর তৈরি করা

নিজস্ব ডোমেন মডেলে `Iterator` বাস্তবায়ন করা অত্যন্ত সহজ এবং এটি রাস্টের সম্পূর্ণ ফাংশনাল টুলকিটের প্রবেশদ্বার খুলে দেয়।

### উদাহরণ ১: `DiscountTierIter`
একটি প্রোগ্রেসিভ ডিসকাউন্ট জেনারেটর যা নির্দিষ্ট ধাপে ডিসকাউন্ট রেট প্রদান করে (যেমন ৫%, ১০%, ১৫%):

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscountTierIter {
    current: u32,
    step: u32,
    max: u32,
}

impl DiscountTierIter {
    pub fn new(step: u32, max: u32) -> Self {
        Self { current: 0, step, max }
    }
}

impl Iterator for DiscountTierIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let next_val = self.current + self.step;
        if next_val <= self.max {
            self.current = next_val;
            Some(self.current)
        } else {
            None
        }
    }
}
```

এখন যেকোনো স্ট্যান্ডার্ড অ্যাডাপ্টার সরাসরি এই কাস্টম ইটারেটরে ব্যবহার করা যাবে:
```rust
let tiers: Vec<u32> = DiscountTierIter::new(5, 20).collect();
// [5, 10, 15, 20]

let sum_over_ten: u32 = DiscountTierIter::new(5, 25)
    .filter(|&rate| rate > 10)
    .sum();
// 15 + 20 + 25 = 60
```

### উদাহরণ ২: `CartReportIterator<'a>`
একটি স্টেটফুল লাইন-আইটেম রিপোর্ট জেনারেটর:
```rust
pub struct CartReportIterator<'a> {
    cart: &'a ShoppingCart,
    index: usize,
}

impl<'a> Iterator for CartReportIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.cart.items.len() {
            let item = &self.cart.items[self.index];
            self.index += 1;
            Some(format!(
                "Item #{}: Product #{} (Qty: {}) - ${:.2}",
                self.index,
                item.product_id,
                item.quantity,
                item.line_total() as f64 / 100.0
            ))
        } else {
            None
        }
    }
}
```

---

## জিরো-কস্ট অ্যাবস্ট্রাকশন: ইটারেটর বনাম `for` লুপ

সি++ এর জনক বিয়ারনে স্ট্রাউস্ট্রুপ জিরো-কস্ট অ্যাবস্ট্রাকশনের সংজ্ঞা দিয়েছিলেন:
> *"যা আপনি ব্যবহার করেন না, তার জন্য আপনাকে কোনো মূল্য দিতে হয় না। আর যা আপনি ব্যবহার করেন, হাতে কোড লিখেও আপনি এর চেয়ে ভালো কিছু বানাতে পারবেন না।"*

রাস্টের ইটারেটর হলো জিরো-কস্ট অ্যাবস্ট্রাকশনের উজ্জ্বলতম উদাহরণ:

```
ফাংশনাল রাস্ট কোড:
cart.items.iter().map(|i| i.line_total()).sum()
                       │
               LLVM অপ্টিমাইজেশন
                       │
কম্পাইল করা মেশিন কোড:
শূন্য বাউন্ডস চেক সহ সরাসরি SIMD রেজিস্টার ব্যবহার করে ভেক্টরাইজড যোগফল
```

বিভিন্ন বেঞ্চমার্কে দেখা গেছে, রাস্টের ইটারেটর কোড হাতে লেখা ইনডেক্সড `for` লুপের চেয়েও বেশি **দ্রুত** কাজ করে!

---

## অন্যান্য ভাষার সাথে তুলনা

| বৈশিষ্ট্য | রাস্ট (Rust) | জাভা স্ট্রিমস (Java Streams) | পাইথন (Python) | গো (Go) | জাভাস্ক্রিপ্ট (JavaScript) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **মডেল** | **`Iterator` Trait** | Stream API | Generators / `iter()` | সাধারণ লুপ | Iterators / Generators |
| **মূল্যায়ন** | **অলস (Lazy)** | অলস | অলস | ইম্পারেটিভ ইগার | ইগার মেথড (`.map`) / অলস জেনারেটর |
| **রানটাইম ওভারহেড** | **শূন্য** (র মেশিন কোডে ইনলাইনড) | অবজেক্ট বক্সিং, হিপ মেমরি, জিসি ওভারহেড | ইন্টারপ্রেটার ওভারহেড | নেই | চেইন করা মেথডে নতুন অ্যারে তৈরি |
| **ওনারশিপ সচেতনতা** | `.iter()`, `.iter_mut()`, `.into_iter()` | কেবল অবজেক্ট রেফারেন্স | ডায়নামিক রেফারেন্স | ভ্যালু / পয়েন্টার | অবজেক্ট রেফারেন্স |

---

## MiniStore আর্কিটেকচার এবং কোড ইমপ্লিমেন্টেশন

MiniStore-এ ইটারেটর কীভাবে সাজানো হয়েছে:

```
ministore/
└── src/
    ├── models/
    │   ├── cart.rs         # iter(), iter_mut(), has_product, IntoIterator, CartReportIterator, DiscountTierIter
    │   ├── order.rs        # iter(), total_quantity, IntoIterator
    │   └── mod.rs          # কাস্টম ইটারেটর রি-এক্সপোর্ট
    ├── catalog.rs          # filter_by_category, products_in_price_range, total_inventory_valuation
    ├── lib.rs              # রি-এক্সপোর্ট এবং ২৯টি পাস করা ইউনিট টেস্ট
    └── main.rs             # ক্যাটালগ কুয়েরি, কার্ট লুপ এবং ডিসকাউন্ট টায়ারের ডেমো
```

### ১. `src/catalog.rs`
```rust
impl Catalog {
    pub fn filter_by_category(&self, category: ProductCategory) -> Vec<&Product> {
        self.products
            .values()
            .filter(|p| p.category == category)
            .collect()
    }

    pub fn products_in_price_range(&self, min_cents: u32, max_cents: u32) -> Vec<&Product> {
        self.products
            .values()
            .filter(|p| p.price_cents >= min_cents && p.price_cents <= max_cents)
            .collect()
    }

    pub fn total_inventory_valuation(&self) -> u64 {
        self.products
            .values()
            .map(|p| (p.price_cents as u64) * (p.stock as u64))
            .sum()
    }
}
```

### ২. `src/models/cart.rs`
```rust
impl ShoppingCart {
    pub fn iter(&self) -> std::slice::Iter<'_, CartItem> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, CartItem> {
        self.items.iter_mut()
    }

    pub fn has_product(&self, product_id: u64) -> bool {
        self.items.iter().any(|item| item.product_id == product_id)
    }

    pub fn apply_promotional_discount(&mut self, percentage: u32) {
        self.items.iter_mut().for_each(|item| {
            let discount = (item.unit_price_cents * percentage) / 100;
            item.unit_price_cents = item.unit_price_cents.saturating_sub(discount);
        });
    }

    pub fn report_iter(&self) -> CartReportIterator<'_> {
        CartReportIterator::new(self)
    }
}
```

---

## সাধারণ কম্পাইলার এরর এবং সমাধানের উপায়

### ১. `.collect()`-এ টাইপ না বলা (`error[E0282]: type annotations needed`)
**ভুল কোড**:
```rust
let items = cart.items.iter().map(|item| item.product_id).collect(); // COMPILE ERROR
```
**কেন ঘটে**:
`.collect()` মেথডটি যেকোনো কালেকশন (`Vec`, `HashSet`, `LinkedList`) তৈরি করতে পারে। আপনি কোন কালেকশন চান তা কম্পাইলার বুঝতে পারে না।
**সমাধান**:
ভেরিয়েবলে টাইপ উল্লেখ করুন অথবা **টার্বোফিশ (`::<Vec<_>>`)** সিনট্যাক্স ব্যবহার করুন:
```rust
// পদ্ধতি ক: ভেরিয়েবলে টাইপ ডিক্লেয়ার করা
let items: Vec<u64> = cart.items.iter().map(|item| item.product_id).collect();

// পদ্ধতি খ: টার্বোফিশ সিনট্যাক্স
let items = cart.items.iter().map(|item| item.product_id).collect::<Vec<_>>();
```

---

### ২. লুপ চলার সময় কালেকশন পরিবর্তন করার চেষ্টা
**ভুল কোড**:
```rust
for item in &cart.items {
    if item.quantity == 0 {
        cart.remove_item(item.product_id); // COMPILE ERROR: cannot borrow cart as mutable
    }
}
```
**কেন ঘটে**:
`&cart.items` কার্টটিকে ইমিউটেবল হিসেবে ধার করে রেখেছে। লুপের ভেতরে `remove_item` কল করতে হলে মিউটেবল বরো প্রয়োজন। রাস্টের এলিয়াসিং নিয়ম একই সাথে রিড ও রাইট বরো নিষিদ্ধ করে।
**সমাধান**:
`Vec`-এর ইনবিল্ট `retain` ব্যবহার করুন:
```rust
cart.items.retain(|item| item.quantity > 0);
```

---

## ইডিওম্যাটিক রাস্ট বেস্ট প্র্যাকটিস

১. **ইনডেক্সড লুপের চেয়ে ইটারেটর চেইনিংকে অগ্রাধিকার দিন**: চেইন করা মেথড (`.filter().map().sum()`) অনেক বেশি পরিচ্ছন্ন, অফ-বাই-ওয়ান এরর মুক্ত এবং কম্পাইলারকে সেরা অ্যাসেম্বলি কোড তৈরি করতে সাহায্য করে।
২. **পার্শ্বপ্রতিক্রিয়ার উদ্দেশ্য থাকলে তবেই `.for_each()` ব্যবহার করুন**: নতুন মান হিসাব করার জন্য `.map()` ও কনজিউমার ব্যবহার করুন। কেবল মিউটেশন বা কনসোল প্রিন্টের মতো কাজের জন্য `.for_each()` ব্যবহার করা উচিত।
৩. **অহেতুক মাঝপথে কালেকশন তৈরি করবেন না**: পাইপলাইনের মাঝখানে দরকার ছাড়া `.collect::<Vec<_>>()` কল করবেন না। পুরো পাইপলাইনকে অলস থাকতে দিন, যাতে শেষ কনজিউমার একবারে লুপ ফিউজ করতে পারে।
৪. **কাস্টম কালেকশনে `IntoIterator` বাস্তবায়ন করুন**: আপনার কালেকশনের জন্য `IntoIterator` ইমপ্লিমেন্ট করলে অন্যান্য ডেভেলপাররা সরাসরি স্বাভাবিক `for` লুপ দিয়ে তা ব্রাউজ করতে পারবে।

---

## বাস্তবমুখী অনুশীলন (Exercises)

### অনুশীলন ১: কার্টের সবচেয়ে মূল্যবান আইটেম খুঁজে বের করা
১. `ShoppingCart`-এ একটি মেথড লিখুন: `pub fn most_expensive_item(&self) -> Option<&CartItem>`।
২. ইটারেটরের `.max_by_key(|item| item.unit_price_cents)` ব্যবহার করে এক লাইনে সমাধান করুন।

### অনুশীলন ২: এসকেইউ প্রিফিক্স সার্চ
১. `Catalog`-এ বাস্তবায়ন করুন: `pub fn search_by_sku_prefix(&self, prefix: &str) -> Vec<&Product>`।
২. `.values().filter(|p| p.sku.starts_with(prefix)).collect()` ব্যবহার করুন।

---

## চেকপয়েন্ট (Checkpoint)

কম্পাইলার চেক চালিয়ে নিশ্চিত করুন যে ২৯টি টেস্টই সফলভাবে পাস করেছে:
```bash
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
cargo run
```

প্রত্যাশিত টেস্ট আউটপুট:
```text
running 29 tests
test tests::test_cart_item_option_lookup ... ok
test tests::test_catalog_iterator_queries ... ok
test tests::test_catalog_option_lookups ... ok
test tests::test_checkout_error_propagation_and_success ... ok
test tests::test_coupon_discount_and_take ... ok
test tests::test_coupon_validation_error ... ok
test tests::test_custom_cart_report_iterator ... ok
test tests::test_custom_discount_tier_iter ... ok
test tests::test_customer_optional_phone ... ok
test tests::test_display_trait_implementations ... ok
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
test tests::test_zero_copy_order_receipt_and_traits ... ok

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

প্রত্যাশিত সিএলআই আউটপুট:
```text
=== MiniStore: Iterators & Functional Pipelines (Part III) ===

1. Catalog initialized with 3 products.

2. Catalog Iteration & Functional Queries:
   Found 3 Electronics product(s): ["Ergonomic Wireless Mouse", "27-inch 4K IPS Display", "Tenkeyless Mechanical Keyboard"]
   Products under $150: Ergonomic Wireless Mouse ($45.00), Tenkeyless Mechanical Keyboard ($120.00)
   Total Inventory Valuation: $2100.00

3. Browsing Catalog with Generic Pagination (Page<Product>):
   Page 1 of 2 (Total Items: 3)
   - [TECH-KEY-001] Tenkeyless Mechanical Keyboard ($120.00)
   - [TECH-MOU-002] Ergonomic Wireless Mouse ($45.00)
   Has next page? true
   Transformed to Page<String>: ["Tenkeyless Mechanical Keyboard", "Ergonomic Wireless Mouse"]
   API Page 2 response: 1 product(s) returned out of 3 total.

4. Customer: Margaret Hamilton (+1-555-0199)

5. Lifetimes & Reference Safety:
   Higher priced item: Tenkeyless Mechanical Keyboard ($120.00)
   Best contact info: +1-555-0199
   Store Policy ('static): MiniStore Guarantee: 30-Day Hassle-Free Returns & 1-Year Warranty

6. Shared Behaviors via Traits:
   Tax Summary: Product #101: Tenkeyless Mechanical Keyboard [TECH-KEY-001] - $120.00 | Tax: $18.00 (15%)
   Tax Summary: Product #102: Ergonomic Wireless Mouse [TECH-MOU-002] - $45.00 | Tax: $6.75 (15%)
   Customer Summary: Customer #301: Margaret Hamilton <margaret@apollo.nasa.gov>

7. Cart Iteration (IntoIterator & Custom CartReportIterator):
   Iterating over cart items directly:
   -> Product ID #101: Qty 1 @ $120.00 each
   -> Product ID #102: Qty 2 @ $45.00 each
   Custom Cart Line Item Reports:
      Item #1: Product #101 (Qty: 1) - $120.00
      Item #2: Product #102 (Qty: 2) - $90.00
   Progressive Discount Tiers available: [5, 10, 15, 20]%

8. Processing checkout through modular services...
   Checkout Order #901 created successfully!
   Subtotal: $210.00 | Total Units: 3 | Final Total: $148.50

9. Order Lifecycle Transitions:
   Order confirmed: Confirmed (Receipt: REC-901-HAMILTON)
   Order shipped:   Shipped (Tracking: TRK-FEDEX-77189)
   Cancellation prevented -> Cannot perform action 'cancel' while order is in 'Shipped (Tracking: TRK-FEDEX-77189)' state
   Final Lifecycle State: Delivered to Customer
   Order Summary: Order #901: 2 item(s), Total: $148.50 [Delivered to Customer]

10. Zero-Copy Receipt Borrowing Order & Slices (OrderReceipt<'a>):
   Display format: Receipt for Order #901 (Margaret Hamilton) - Total: $148.50
   Trait Summary:  Receipt: Order #901 for Margaret Hamilton | Total: $148.50

--- Printed Slip ---
=== RECEIPT: #901 ===
Customer: Margaret Hamilton
Items: 2
Total: $148.50
Status: Delivered to Customer
Note: VIP Client - Express courier delivery verified
===================
--------------------
```

---

## আমরা যা শিখলাম
- `Iterator` ট্রেইট কীভাবে `next()`-এর মাধ্যমে সিকোয়েন্সের উপাদান সরবরাহ করে।
- `.iter()`, `.iter_mut()`, এবং `.into_iter()`-এর মধ্যে ওনারশিপের পার্থক্য।
- কীভাবে `IntoIterator` কাস্টম ডোমেন কালেকশনকে সরাসরি `for` লুপের আওতায় নিয়ে আসে।
- কেন অলস মূল্যায়ন লুপ ফিউশন ও জিরো-কস্ট অ্যাবস্ট্রাকশন সম্ভব করে তোলে।
- বিশেষ প্রয়োজনের জন্য কীভাবে কাস্টম ইটারেটর (`DiscountTierIter`, `CartReportIterator`) তৈরি করতে হয়।

---

## পরবর্তী অধ্যায়
ইটারেটরের ক্ষমতা বহুগুণ বেড়ে যায় যখন এর সাথে যুক্ত হয় **ক্লোজার (Closures)**—বেনামী ফাংশন (Anonymous Functions) যা তাদের পারিপার্শ্বিক স্কোপ থেকে ভেরিয়েবল ক্যাপচার করতে পারে!
**অধ্যায় ১৮: ক্লোজারস (Closures)**-এ আমরা ক্লোজার সিনট্যাক্স, পরিবেশ ক্যাপচারের ধরন (`Fn`, `FnMut`, `FnOnce`) এবং কীভাবে ক্লোজার MiniStore-এর ডায়নামিক ডিসকাউন্ট ও ভ্যালিডেশন ইঞ্জিনকে চালিত করে তা শিখব!
