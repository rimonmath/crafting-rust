# অধ্যায় ১১: Option — বিলিয়ন ডলারের ভুলের সমাধান

## আপনি যা শিখবেন
- কম্পিউটার সায়েন্স পথিকৃৎ স্যার টনি হোর কেন `null` রেফারেন্সকে তাঁর **"বিলিয়ন ডলারের ভুল" (Billion-Dollar Mistake)** বলেছিলেন।
- কীভাবে রাস্ট `null` পয়েন্টার সম্পূর্ণ বর্জন করে **`Option<T>`** এনাম দিয়ে নিরাপদ মডেলিং নিশ্চিত করে।
- স্ট্যান্ডার্ড লাইব্রেরি সংজ্ঞা: **`Some(T)`** বনাম **`None`**।
- নিশ্চিত মান (`T`) এবং অনুপস্থিত হতে পারে এমন মান (`Option<T>`)-এর মধ্যে টাইপ সিস্টেমের স্পষ্ট পার্থক্য।
- রাস্টের জিরো-কস্ট মেমরি অপ্টিমাইজেশন: **Null Pointer Optimization (NPO)**।
- নিরাপদে মান বের করা ও প্যাটার্ন ম্যাচিং: `match`, `if let`, এবং প্রোডাকশন কোডে **`.unwrap()`** ব্যবহারের ঝুঁকি।
- অত্যন্ত দরকারি স্ট্যান্ডার্ড কম্বিনেটর: **`.unwrap_or()`**, **`.unwrap_or_else()`**, **`.map()`**, **`.and_then()`**, এবং **`.as_ref()`**।
- মেমরিতে সরাসরি মান অদলবদল: **`.take()`** এবং **`.replace()`**।
- MiniStore-এ `Option<T>`-এর বাস্তব প্রয়োগ:
  - ক্যাটালগে নিরাপদ O(1) লুকআপ (`find_by_sku`, `find_by_id`)।
  - গ্রাহকের ঐচ্ছিক প্রোফাইল ফিল্ড (`phone: Option<String>`)।
  - কার্ট আইটেমের নিরাপদ অনুসন্ধান (`get_item`, `get_item_mut`)।
  - ঐচ্ছিক ডিসকাউন্ট কুপন (`coupon: Option<Coupon>`) এবং গতিশীল মোট মূল্য হিসাব।

---

## কেন এটি আমাদের প্রয়োজন?
১৯৬৫ সালে ব্রিটিশ কম্পিউটার বিজ্ঞানী স্যার টনি হোর (Sir Tony Hoare) ALGOL W ভাষার টাইপ সিস্টেম ডিজাইন করার সময় `null` রেফারেন্স উদ্ভাবন করেন। এর কয়েক দশক পর, ২০০৯ সালের একটি আন্তর্জাতিক কনফারেন্সে তিনি প্রকাশ্যে দুঃখ প্রকাশ করে বলেন:

> *"আমি একে আমার বিলিয়ন ডলারের ভুল বলে ডাকি। ১৯৬৫ সালে নাল (null) রেফারেন্সের আবিষ্কার ছিল এটি... আমি তখন অবজেক্ট-ওরিয়েন্টেড রেফারেন্সের প্রথম সমন্বিত টাইপ সিস্টেম ডিজাইন করছিলাম। আমার লক্ষ্য ছিল সব রেফারেন্স যেন কম্পাইলার স্বয়ংক্রিয়ভাবে শতভাগ নিরাপদে যাচাই করতে পারে। কিন্তু নাল রেফারেন্স যোগ করার লোভ আমি সামলাতে পারিনি, কারণ এটি ইমপ্লিমেন্ট করা ছিল ভীষণ সহজ। কিন্তু পরবর্তী চল্লিশ বছরে এর কারণে অগণিত ত্রুটি, সফটওয়্যার ক্র্যাশ এবং নিরাপত্তা দুর্বলতা তৈরি হয়েছে, যা সম্ভবত শতকোটি ডলারের ক্ষয়ক্ষতি ও ভোগান্তি ডেকে এনেছে।"*

কেন `null` এত মারাত্মক ভুল ছিল?
C, C++, Java, C#, Python, এবং JavaScript-এর মতো ভাষাগুলোতে:
- **যেকোনো অবজেক্ট বা রেফারেন্স ভ্যারিয়েবল গোপনে যেকোনো মুহূর্তে `null` হতে পারে।**
- আপনি যদি `Customer customer` টাইপ লিখেন, টাইপ দেখে বোঝার কোনো উপায় নেই যে এটি মেমরির বাস্তব কোনো কাস্টমারকে নির্দেশ করছে নাকি `null`।
- কোড চলাকালীন ভুলবশত `null` অবস্থায় `customer.email` এক্সেস করার চেষ্টা করলেই পুরো সার্ভিস তৎক্ষণাৎ ক্র্যাশ করে:
  - Java: `java.lang.NullPointerException`
  - JavaScript: `TypeError: Cannot read properties of null`
  - Python: `AttributeError: 'NoneType' object has no attribute 'email'`
  - C / C++: `Segmentation fault (core dumped)`

ডেভেলপাররা সারাজীবন এমন আত্মরক্ষামূলক বয়লারপ্লেট কোড লিখে কাটান:
```java
// জাভা বা সি#-এ প্রচলিত ডিফেন্সিভ চেকিং
if (customer != null) {
    if (customer.getAddress() != null) {
        if (customer.getAddress().getCity() != null) {
            // অবশেষে সিটি ব্যবহার করা নিরাপদ...
        }
    }
}
```
হাজার হাজার লাইনের প্রজেক্টে ডেভেলপার যদি একটি মাত্র চেক ভুলে যান, প্রোগ্রামটি কোনো কম্পাইল এরর ছাড়াই সফলভাবে বিল্ড হবে এবং মাঝরাতে প্রোডাকশনে আসল ইউজারের সামনে ক্র্যাশ করবে।

**রাস্টের সমাধান**: **নিরাপদ রাস্টে কোনো `null`, `nil` বা নাল পয়েন্টারের অস্তিত্বই নেই।**

রাস্টে কোনো ভ্যারিয়েবলের টাইপ যদি `String` হয়, কম্পাইলার শতভাগ গ্যারান্টি দেয় যে এতে সর্বদা একটি বৈধ স্ট্রিং মান বিদ্যমান—এটি কখনোই নাল হতে পারে না।

আর যখন কোনো ডাটা অনুপস্থিত থাকা সম্ভব (যেমন: ক্যাটালগে প্রোডাক্ট না পাওয়া যাওয়া, বা ইউজার ফোন নম্বর না দেওয়া), রাস্ট আপনাকে বাধ্যতামূলকভাবে **`Option<T>`** টাইপ ব্যবহার করতে বাধ্য করে। অনুপস্থিতির সম্ভাবনা নিরাপদভাবে হ্যান্ডেল না করে আপনি ভেতরকার আসল মান ব্যবহার করতেই পারবেন না।

---

## সমস্যাটি কী?
MiniStore অ্যাপ্লিকেশনের ডাটা লেনদেনের কথা ভাবুন:

```text
১. ক্যাটালগ অনুসন্ধান:
   catalog.find_by_sku("TECH-KEY-001") ──► প্রোডাক্ট পাওয়া গেছে! প্রোডাক্ট রিটার্ন করুন।
   catalog.find_by_sku("NON-EXISTENT")  ──► প্রোডাক্ট নেই! কীভাবে তা জানাবেন?

২. কাস্টমার প্রোফাইল:
   কাস্টমার অ্যালিস: ফোন নম্বর "+1-555-0199" দিয়েছেন।
   কাস্টমার বব:   ফোন নম্বর দিতে চাননি।

৩. চেকআউট ডিসকাউন্ট কুপন:
   অর্ডার ৯০১: গ্রাহক প্রোমো কোড "LAUNCH20" (২০% ডিসকাউন্ট) দিয়েছেন।
   অর্ডার ৯০২: গ্রাহক কোনো কুপন ছাড়াই সরাসরি অর্ডার করছেন।

৪. শপিং কার্ট লুকআপ:
   cart.get_item(101) ──► এই প্রোডাক্ট কি ইতিমধ্যে কার্টে আছে?
```

যেসব ভাষায় `null` রয়েছে:
- যদি `find_by_sku` প্রোডাক্ট না পেলে `null` রিটার্ন করে, তবে ট্যাক্স হিসাবের সময় কেউ `if (product != null)` চেক না করলেই পুরো স্টোর ক্র্যাশ করবে।
- কাস্টমারের ফোন নম্বর না থাকলে ডেভেলপাররা প্রায়ই ফাঁকা স্ট্রিং `""` বা `"N/A"` ব্যবহার করেন। কিন্তু ফাঁকা স্ট্রিং কি অনুপস্থিত ফোন নম্বর নাকি ভুল ফরম্যাটের নম্বর? এমন জাদুকরী সেন্টিনেল ভ্যালু ভবিষ্যতে মারাত্মক বাগ তৈরি করে।
- অর্ডারে কোনো কুপন না থাকলে `null` হ্যান্ডেল করার বাড়তি ঝামেলা তৈরি হয়।

MiniStore-এর প্রয়োজন:
১. টাইপ সিস্টেমের মাধ্যমেই মানের উপস্থিতি বা অনুপস্থিতি সুস্পষ্ট করা।
২. অনুপস্থিত ক্ষেত্র হ্যান্ডেল না করে ভেতরের মানে প্রবেশের কোনো সুযোগ না থাকা (কম্পাইল-টাইম নিরাপত্তা)।
৩. মেমরিতে র-পয়েন্টারের চেয়ে এক বাইটও বাড়তি অপচয় না হওয়া।
৪. নেস্টেড `if-else` পরিহার করে পরিচ্ছন্ন ও পঠনযোগ্য কোড লেখা।

---

## রাস্টের মূল ধারণা

### ১. `Option<T>`-এর সংজ্ঞা
রাস্ট স্ট্যান্ডার্ড লাইব্রেরিতে `Option<T>` আসলে একটি অত্যন্ত চমৎকার এনাম (Enum):

```rust
pub enum Option<T> {
    None,
    Some(T),
}
```

খেয়াল করুন:
- এর দুটি ভ্যারিয়েন্ট রয়েছে:
  - `None`: নির্দেশ করে কোনো মান উপস্থিত নেই।
  - `Some(T)`: এর ভেতরে `T` টাইপের একটি মান সংরক্ষিত থাকে।
- এটি জেনেরিক টাইপ `T` গ্রহণ করে। তার মানে আপনি `Option<u32>`, `Option<String>`, `Option<Product>`, বা `Option<&Product>` তৈরি করতে পারেন।
- `Option`, `Some`, এবং `None` রাস্টের **Prelude**-এ স্বয়ংক্রিয়ভাবে অন্তর্ভুক্ত। ফলে আলাদা করে `use std::option::Option;` ইমপোর্ট করার কোনো প্রয়োজন নেই—যেকোনো ফাইলে সরাসরি এগুলো ব্যবহার করা যায়!

### ২. টাইপের স্বাতন্ত্র্য: `T` বনাম `Option<T>`
রাস্টে সাধারণ `String` এবং `Option<String>` দুটি সম্পূর্ণ আলাদা টাইপ:

```rust
let name: String = String::from("Alice");
let maybe_phone: Option<String> = Some(String::from("+1-555-0199"));
let no_phone: Option<String> = None;
```

আপনি যদি কোনো `Option<String>`-কে সরাসরি `String` হিসেবে চালাতে চান:

```rust
fn print_length(s: String) {
    println!("Length: {}", s.len());
}

print_length(maybe_phone); // কম্পাইল এরর!
```

রাস্ট কম্পাইলার মুহূর্তেই এটি আটকে দেবে:
```text
error[E0308]: mismatched types
  --> src/main.rs
   |
   | print_length(maybe_phone);
   |              ^^^^^^^^^^^ expected `String`, found `Option<String>`
```
ভেতরের `String` পেতে হলে আপনাকে অবশ্যই `Option` আনপ্যাক (Unpack) করতে হবে। ভুলে যাওয়ার কোনো সুযোগই নেই!

### ৩. মেমরি বিন্যাস: Null Pointer Optimization (NPO)
আপনার মনে প্রশ্ন জাগতে পারে: *প্রতিটি রেফারেন্সকে `Option<&T>` দিয়ে মুড়ে দিলে কি ট্যাগ বাইটের কারণে বাড়তি মেমরি খরচ হবে?*

[অধ্যায় ১০](/bn/chapters/10-enums-and-pattern-matching)-এ আমরা দেখেছি যে সাধারণ এনামে ডিসক্রিমিন্যান্ট ট্যাগ এবং পেলোড থাকে। কিন্তু রেফারেন্স (`&T`, `&mut T`), র-পয়েন্টার এবং `Box<T>`-এর মতো টাইপগুলোর জন্য রাস্ট কম্পাইলার **Null Pointer Optimization (NPO)** প্রয়োগ করে:

```text
সাধারণ রেফারেন্স (&Product):
┌───────────────────────────────┐
│ Pointer Address (8 bytes)     │  কখনোই 0x0 হতে পারে না (বৈধ ঠিকানা)
└───────────────────────────────┘

Option<&Product>:
┌───────────────────────────────┐
│ Some(&p) = Pointer Address    │  বৈধ মেমরি ঠিকানা
│ None     = 0x0000000000000000 │  সব শূন্য বিট নির্দেশ করে None!
└───────────────────────────────┘
```

যেহেতু রাস্টে নিরাপদ রেফারেন্স কখনোই `0x0` ঠিকানায় থাকতে পারে না, তাই কম্পাইলার অভ্যন্তরীণভাবে `0x0` বিট প্যাটার্নটিকে `None` হিসেবে গণ্য করে।
ফলে:
- `std::mem::size_of::<&Product>() == 8 bytes`
- `std::mem::size_of::<Option<&Product>>() == 8 bytes`

আপনি শতভাগ কম্পাইল-টাইম নিরাপত্তা পাচ্ছেন **একদম শূন্য মেমরি খরচে এবং শূন্য রানটাইম পারফরম্যান্স পেনাল্টিতে**!

### ৪. `Option` হ্যান্ডেল করার বিভিন্ন উপায়

#### ক. `match` দিয়ে সব সম্ভাবনার যাচাই
`Option` হ্যান্ডেল করার সবচেয়ে নির্ভরযোগ্য উপায় হলো প্যাটার্ন ম্যাচিং:

```rust
match catalog.find_by_sku("TECH-KEY-001") {
    Some(product) => println!("প্রোডাক্ট পাওয়া গেছে: {}", product.name),
    None => println!("ক্যাটালগে প্রোডাক্টটি নেই"),
}
```
যেহেতু `match` সম্পূর্ণ একজস্টিভ (Exhaustive), আপনি `None` আর্মটি বাদ দিলে কোড কম্পাইলই হবে না।

#### খ. `if let` দিয়ে সংক্ষিপ্ত যাচাই
যখন আপনি কেবল মান উপস্থিত থাকার ক্ষেত্রটি নিয়ে আগ্রহী:

```rust
if let Some(item) = cart.get_item(101) {
    println!("কার্টে পরিমাণ: {}", item.quantity);
}
```

#### গ. ডিফল্ট মান নির্ধারণ: `.unwrap_or()` এবং `.unwrap_or_else()`
লম্বা `match` না লিখে ডিফল্ট মান বসাতে পারেন:

```rust
// find_by_sku যদি None রিটার্ন করে তবে ডিফল্ট হিসেবে 0 রিটার্ন করবে
let price = catalog.product_price("UNKNOWN-SKU").unwrap_or(0);

// জটিল বা ব্যয়বহুল হিসাবের ক্ষেত্রে ক্লোজার দিয়ে লেজি ক্যালকুলেশন:
let discount = order.coupon.as_ref()
    .map(|c| c.discount_percent)
    .unwrap_or_else(|| calculate_default_discount());
```

#### ঘ. মান রূপান্তর: `.map()`
ভেতরের মান উপস্থিত থাকলে তাতে কোনো অপারেশন চালাতে `.map()` ব্যবহার করা হয়:

```rust
// find_by_sku রিটার্ন করে Option<&Product>
// .map(|p| p.price_cents) একে Option<u32>-তে রূপান্তর করে
let price: Option<u32> = catalog.find_by_sku("TECH-KEY-001").map(|p| p.price_cents);
```
মান উপস্থিত থাকলে এটি `Some(দাম)` দেবে; আর `None` থাকলে সরাসরি `None` হয়ে যাবে।

#### ঙ. চেইনিং অপারেশন: `.and_then()`
যদি আপনার রূপান্তর ফাংশনটিও আরেকটি `Option` রিটার্ন করে, তবে `.map()` ব্যবহার করলে `Option<Option<T>>` তৈরি হয়। এটিকে সমতল (flatten) করতে `.and_then()` ব্যবহার করুন:

```rust
let area_code = find_order(id)
    .and_then(|order| order.customer.phone)
    .map(|phone| phone[..3].to_string());
```

#### চ. রেফারেন্স ধার নেওয়া: `.as_ref()` এবং `.as_deref()`
যদি আপনার কাছে `&Option<T>` থাকে এবং মালিকানা না হারিয়ে ভেতরের রেফারেন্স চান:

```rust
let opt: Option<String> = Some(String::from("Hello"));

// opt.as_ref() দেয় Option<&String>
let len = opt.as_ref().map(|s| s.len());

// opt.as_deref() দেয় Option<&str> (&String থেকে &str-এ কোয়ের্সন)
let slice: Option<&str> = opt.as_deref();
```

#### ছ. মান বের করে আনা ও ফাঁকা করা: `.take()`
`.take()` মেথডটি `Option`-এর ভেতর থেকে মান বের করে নিয়ে আসে এবং সেই জায়গায় `None` বসিয়ে দেয়:

```rust
let mut coupon = Some(Coupon::new(String::from("SAVE10"), 10));

let extracted = coupon.take(); // extracted হবে Some(Coupon)
println!("{:?}", coupon);      // coupon এখন হয়ে গেছে None!
```
মিউটেবল রেফারেন্স `&mut self`-এর ভেতর থাকা কোনো মান ক্লোন না করেই বের করে আনার ক্ষেত্রে এটি অত্যন্ত শক্তিশালী একটি কৌশল।

---

## অন্যান্য ভাষার সাথে তুলনা

| ধারণা | Rust | Java / C# | TypeScript / JavaScript | Python | Go |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **অনুপস্থিত মান** | `Option::None` | `null` | `null` / `undefined` | `None` | `nil` |
| **উপস্থিত মান** | `Option::Some(val)` | সরাসরি মান | সরাসরি মান | সরাসরি মান | পয়েন্টার / মান |
| **কম্পাইলার বাধ্যবাধকতা** | **বাধ্যতামূলক** (`T` কখনো `null` নয়) | ঐচ্ছিক (`@Nullable`, `Optional<T>`) | ঐচ্ছিক (`strictNullChecks`) | নেই (রানটাইম ডাক টাইপিং) | নেই |
| **মেমরি বিন্যাস** | Null Pointer Optimization (০-খরচ) | বাড়তি অবজেক্ট বরাদ্দ (`Optional<T>`) | ডায়নামিক টাইপ ট্যাগ | হিপ অবজেক্ট (`NoneType`) | ৮-বাইট পয়েন্টার |
| **নাল ক্র্যাশ ঝুঁকি** | নিরাপদ কোডে **অসম্ভব** | `NullPointerException` | `TypeError: Cannot read property` | `AttributeError: 'NoneType'` | `panic: nil pointer dereference` |
| **মান রূপান্তর** | `opt.map(f)` | `opt.map(f)` | অপশনাল চেইনিং `val?.prop` | `f(val) if val else None` | `if val != nil { ... }` |

---

## ছোট উদাহরণ
নিচের স্বয়ংসম্পূর্ণ কোডটি চালিয়ে `Option<T>`, প্যাটার্ন ম্যাচিং, কম্বিনেটর এবং `.take()` পর্যবেক্ষণ করুন:

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

    // ১. match দিয়ে প্যাটার্ন ম্যাচিং
    match &alice.phone {
        Some(phone) => println!("অ্যালিসের ফোন: {phone}"),
        None => println!("অ্যালিসের কোনো ফোন নম্বর নেই"),
    }

    // ২. .as_deref() এবং .unwrap_or() ব্যবহার
    println!("ববের ফোন: {}", bob.phone.as_deref().unwrap_or("তালিকাভুক্ত নয়"));

    // ৩. .map() দিয়ে রূপান্তর
    let phone_len: Option<usize> = alice.phone.as_ref().map(|p| p.len());
    println!("অ্যালিসের ফোন নম্বরের দৈর্ঘ্য: {:?}", phone_len);

    // ৪. .take() দিয়ে মান সরানো
    let mut mutable_bob_phone = Some(String::from("+1-555-9999"));
    let extracted = mutable_bob_phone.take();
    println!("বের করা ফোন: {:?}", extracted);
    println!("অবশিষ্ট ফোন: {:?}", mutable_bob_phone); // None
}
```

---

## MiniStore-এ প্রয়োগ
MiniStore-এ আমরা পুরো ডোমেইনে `Option<T>` সমন্বিত করেছি:

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
                               │ রিটার্ন করে
                               ▼
                        Option<&Product>
                         ├── Some(&Product) ──► স্টক ও দাম নিরাপদে দেখুন
                         └── None           ──► কোনো ক্র্যাশ ছাড়াই সামলান!

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
│ + total_cents(&self) -> u32 (ঐচ্ছিক কুপন ডিসকাউন্ট)         │
└─────────────────────────────────────────────────────────────┘
```

১. **`Catalog`**:
   - `find_by_sku(&self, sku: &str) -> Option<&Product>` কোনো ক্লোন ছাড়াই অভ্যন্তরীণ `HashMap` থেকে রেফারেন্স রিটার্ন করে।
   - `product_price(&self, sku: &str) -> Option<u32>` মেথডটি `.map(|p| p.price_cents)` ব্যবহার করে সুন্দরভাবে দাম বের করে।
   - `is_product_in_stock(&self, sku: &str) -> bool` মেথডটি `.map(|p| p.is_in_stock()).unwrap_or(false)` ব্যবহার করে স্টক স্ট্যাটাস দেয়।
২. **`Customer`**:
   - `phone: Option<String>` কাস্টমারের ঐচ্ছিক ফোন নম্বর ধারণ করে।
   - `formatted_phone(&self) -> &str` মেথডটি `self.phone.as_deref().unwrap_or("Unspecified")` দিয়ে ফরম্যাট করে।
৩. **`ShoppingCart`**:
   - `get_item(&self, product_id: u64) -> Option<&CartItem>`
   - `get_item_mut(&mut self, product_id: u64) -> Option<&mut CartItem>`
   - `add_item` এখন `get_item_mut` ব্যবহার করে: `Some(item)` পেলে কোয়ান্টিটি বাড়ায়, আর `None` হলে নতুন আইটেম যুক্ত করে।
৪. **`Coupon` এবং `Order`**:
   - অর্ডারে ডিসকাউন্ট কুপন ঐচ্ছিক: `pub coupon: Option<Coupon>`।
   - ডিসকাউন্ট হিসাবে `self.coupon.as_ref()` ব্যবহার করা হয়।
   - `remove_coupon(&mut self) -> Option<Coupon>` মেথডটি `self.coupon.take()` দিয়ে মান রিমুভ করে।

---

## কোড
অধ্যায় ১১-এর সম্পূর্ণ এবং প্রোডাকশন-রেডি MiniStore কোড নিচে দেওয়া হলো। স্ন্যাপশটটি `examples/chapter-11/src/main.rs`-এ সংরক্ষিত রয়েছে:

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

## কোড ব্যাখ্যা

### ১. ক্যাটালগ অনুসন্ধান ও `Option<&Product>`
`Catalog::find_by_sku` মেথডে:
```rust
pub fn find_by_sku(&self, sku: &str) -> Option<&Product> {
    self.products.get(sku)
}
```
`HashMap::get` স্বাভাবিকভাবেই `Option<&V>` রিটার্ন করে। এটি কোনো ক্লোনিং ছাড়াই মূল ডাটা থেকে `&self`-এর মাধ্যমে রেফারেন্স ধার দেয়। ক্যাটালগে SKU থাকলে কলার পাবে `Some(&product)`, আর না থাকলে `None`। ফলে কলার মিসিং প্রোডাক্টের সম্ভাবনা হ্যান্ডেল করতে বাধ্য হয়।

### ২. কম্বিনেটর দিয়ে সংক্ষিপ্ত রূপান্তর
`product_price` মেথডটি লক্ষ্য করুন:
```rust
pub fn product_price(&self, sku: &str) -> Option<u32> {
    self.find_by_sku(sku).map(|p| p.price_cents)
}
```
`.map()` ছাড়া আমাদের লিখতে হতো:
```rust
match self.find_by_sku(sku) {
    Some(p) => Some(p.price_cents),
    None => None,
}
```
`.map()` অনর্থক ৪ লাইনের কোড কমিয়ে এক লাইনে নিয়ে আসে। এরপর কলার সরাসরি `.unwrap_or(0)` চেইনিং করতে পারে:
```rust
let price = catalog.product_price("TECH-KEY-001").unwrap_or(0);
```

### ৩. `as_deref()` দিয়ে রেফারেন্স ধার নেওয়া
`Customer::formatted_phone` মেথডে:
```rust
pub fn formatted_phone(&self) -> &str {
    self.phone.as_deref().unwrap_or("Unspecified")
}
```
- `self.phone` হলো `Option<String>`।
- `self.phone.as_deref()` একে `Option<&str>`-এ রূপান্তর করে।
- `.unwrap_or("Unspecified")` মান থাকলে ভেতরের `&str` স্লাইস দেয়, আর `None` হলে স্ট্যাটিক স্লাইস `"Unspecified"` দেয়।
হিপ মেমরিতে এক বাইটও বরাদ্দ না করে চমৎকারভাবে দুটি ক্ষেত্রই `&str` রিটার্ন করে!

### ৪. `.take()` দিয়ে রেফারেন্স থেকে মান বের করা
`Order::remove_coupon` মেথডে:
```rust
pub fn remove_coupon(&mut self) -> Option<Coupon> {
    self.coupon.take()
}
```
আপনি যদি সরাসরি এমন লিখতেন:
```rust
let old = self.coupon; // কম্পাইল এরর: &mut self-এর ভেতর থেকে সরাসরি সরানো যায় না
self.coupon = None;
```
রাস্টের বরো চেকার এটি মানবে না, কারণ ক্ষণিকের জন্য হলেও মানটি আন-ইনিশিয়ালাইজড থেকে যায়। `self.coupon.take()` সেখানে তৎক্ষণাৎ `None` বসিয়ে দেয় এবং পূর্ববর্তী মানটি মালিকানাসহ কলারের কাছে পাঠিয়ে দেয়—কোনো ক্লোনিং ছাড়াই!

---

## সাধারণ ভুলসমূহ

### ১. প্রোডাকশন কোডে `.unwrap()` কল করা
```rust
// ক্ষতিকর অভ্যাস: SKU না থাকলে পুরো সিস্টেম প্যানিক ক্র্যাশ করবে!
let product = catalog.find_by_sku("PROD-999").unwrap();
```
`.unwrap()` কেবল প্রোটোটাইপ বা ইউনিট টেস্টে গ্রহণযোগ্য যেখানে ফেইলিউর মানে টেস্ট ফেইল। কিন্তু প্রোডাকশন বিজনেস লজিকে কখনোই সরাসরি আনর‍্যাপ করবেন না। এর বদলে `match`, `if let`, `.unwrap_or()`, বা `?` ব্যবহার করুন।

### ২. শেয়ার্ড রেফারেন্সের ভেতর থেকে মালিকানা সরানোর চেষ্টা
```rust
fn print_customer_phone(customer: &Customer) {
    // এরর: customer.phone থেকে মালিকানা সরানো সম্ভব নয়
    match customer.phone {
        Some(p) => println!("{p}"),
        None => println!("None"),
    }
}
```
`customer` এখানে ধার নেওয়া (`&Customer`)। সরাসরি `customer.phone`-এ ম্যাচ করলে ভেতরের `String`-এর ওনারশিপ নেওয়ার চেষ্টা করা হয়।
**সমাধান**: রেফারেন্স ধরে ম্যাচ করুন (`match &customer.phone` অথবা `customer.phone.as_ref()`):
```rust
match &customer.phone {
    Some(p) => println!("{p}"),
    None => println!("None"),
}
```

### ৩. `Option`-এর বদলে ফাঁকা স্ট্রিং বা সেন্টিনেল ভ্যালু ব্যবহার
```rust
// ক্ষতিকর অভ্যাস: ফাঁকা স্ট্রিং বা -1 দিয়ে অনুপস্থিতি বোঝানো
struct BadCustomer {
    phone: String, // "" মানে ফোন নম্বর নেই
}
```
জাদুকরী স্ট্রিং বা নেগেটিভ সংখ্যা ব্যবহার করলে কম্পাইলারের নিরাপত্তা পাওয়া যায় না। অন্য কোনো ডেভেলপার `if phone == ""` চেক করতে ভুলে গেলে ভুল নম্বর হিসেবে সেভ হয়ে যাবে। সর্বদা `Option<String>` ব্যবহার করুন।

---

## কম্পাইলার এররসমূহ

### Error E0308: Mismatched Types (`Option<T>` বনাম `T`)
```rust
let price: Option<u32> = Some(1500);
let total = price + 100;
```
কম্পাইলার আউটপুট:
```text
error[E0369]: cannot add `{integer}` to `Option<u32>`
 --> src/main.rs:2:19
  |
2 |     let total = price + 100;
  |                 ----- ^ --- {integer}
  |                 |
  |                 Option<u32>
```
**কেন এটি ঘটে**: রাস্ট স্বয়ংক্রিয়ভাবে `Option<T>`-কে `T`-তে পরিবর্তন করে না। অনুপস্থিত থাকতে পারে এমন কোনো সংখ্যার সাথে আপনি ভুলবশত যোগ-বিয়োগ করতে পারবেন না।
**সমাধান**: নিরাপদভাবে অপশনটি আনপ্যাক করুন:
```rust
let total = price.unwrap_or(0) + 100;
```

---

## অনুশীলন
১. **সবচেয়ে কম দামি প্রোডাক্ট খোঁজা**:
   `Catalog`-এ একটি মেথড যোগ করুন:
   ```rust
   pub fn find_cheapest_product(&self) -> Option<&Product>
   ```
   ক্যাটালগে কোনো প্রোডাক্ট না থাকলে `None` দিন। থাকলে সবচেয়ে কম মূল্যের প্রোডাক্টের `Some(&cheapest)` দিন।
২. **কাস্টমার এরিয়া কোড বের করা**:
   একটি ফাংশন লিখুন যা কাস্টমারের ফোন নম্বরের প্রথম ৩টি অক্ষর এরিয়া কোড হিসেবে রিটার্ন করে:
   ```rust
   pub fn area_code(customer: &Customer) -> Option<&str>
   ```
   `.as_deref()` এবং নিরাপদ স্লাইসিং ব্যবহার করুন।
৩. **কুপন আপগ্রেড করা**:
   `Order`-এ একটি মেথড লিখুন:
   ```rust
   pub fn upgrade_coupon(&mut self, new_coupon: Coupon) -> Option<Coupon>
   ```
   `self.coupon.replace(new_coupon)` ব্যবহার করে নতুন কুপন বসান এবং পূর্বের কুপনটি (যদি থাকে) রিটার্ন করুন।

---

## চেকপয়েন্ট
সবগুলো ৯টি ইউনিট টেস্ট সফলভাবে পাস করছে কিনা পরীক্ষা করুন:
```bash
cargo test
```
প্রত্যাশিত আউটপুট:
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

`clippy` দিয়ে কোড যাচাই করুন:
```bash
cargo clippy -- -D warnings
```

---

## আমরা কী শিখলাম
- কেন `null` রেফারেন্স কয়েক দশক ধরে সফটওয়্যার ক্র্যাশের প্রধান কারণ ছিল এবং রাস্টের টাইপ সিস্টেম কীভাবে কম্পাইল-টাইমেই এটি নির্মূল করেছে।
- `Option<T>`-এর অভ্যন্তরীণ রূপ: `Some(T)` বনাম `None`।
- কীভাবে Null Pointer Optimization শূন্য মেমরি খরচে `Option<&T>` নিশ্চিত করে।
- প্রোডাকশনে `.unwrap()` ব্যবহারের বিপদ এবং এর বদলে `.unwrap_or()`, `.map()`, `.and_then()` ও `.take()` ব্যবহারের নিয়ম।
- কীভাবে MiniStore ক্যাটালগ অনুসন্ধান, কার্ট আইটেম, কাস্টমার তথ্য এবং কুপন ব্যবস্থাপনায় `Option` ব্যবহার করেছে।

---

## পরবর্তী অধ্যায়
এখন আমরা জেনেছি কীভাবে `Option<T>` দিয়ে কোনো মানের অনুপস্থিতি নিরাপদে প্রকাশ করতে হয়। কিন্তু কোনো অপারেশন যদি কেবল অনুপস্থিত না হয়ে **সুনির্দিষ্ট কোনো কারণে ব্যর্থ হয়** (যেমন: ইনভ্যালিড ইনপুট বা পেমেন্ট গেটওয়ে ফেইলিউর)?
[অধ্যায় ১২: Result এবং এরর হ্যান্ডলিং](/bn/chapters/12-result-and-error-handling)-এ আমরা শিখব রাস্টের দ্বিতীয় সুপারপাওয়ার এনাম: **`Result<T, E>`** এবং সেই বিখ্যাত **`?` অপারেটর**!
