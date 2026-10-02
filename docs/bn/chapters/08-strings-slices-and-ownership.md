# অধ্যায় ৮: স্ট্রিং, স্লাইস এবং ওনারশিপের ব্যবহারিক প্রয়োগ (Strings, Slices and Ownership in Practice)

## আপনি কী শিখবেন
- ওনড (Owned) কালেকশন এবং **স্লাইস (Slices)**-এর মৌলিক পার্থক্য: মেমোরির ধারাবাহিক অংশকে কোনো কপি বা নতুন বরাদ্দ ছাড়াই ধার নেওয়া (Zero-copy borrowing)।
- স্লাইসের অভ্যন্তরীণ রূপ: **১৬-বাইটের ফ্যাট পয়েন্টার (Fat Pointer)** (`ptr` + `len`)।
- **`String` বনাম `&str`**: হিপে সংরক্ষিত ডাইনামিক বাফার বনাম ধার নেওয়া স্ট্রিং স্লাইস।
- **ডিরেফ কোয়ার্সন (Deref Coercion)**: কেন দক্ষ রাস্ট ডেভেলপাররা ফাংশন প্যারামিটারে `&String`-এর বদলে `&str` ব্যবহার করেন।
- **অ্যারে স্লাইস (`&[T]`) এবং মিউটেবল স্লাইস (`&mut [T]`)**: ব্যাচ অপারেশনের জন্য অ্যারে বা ভেক্টরের নির্দিষ্ট অংশ ধার নিয়ে ইন-প্লেস পরিবর্তন।
- **UTF-8 সুরক্ষা এবং ক্যারেক্টার বাউন্ডারি**: কেন রাস্টে সরাসরি ইনডেক্সিং (`s[0]`) নিষিদ্ধ এবং কীভাবে নিরাপদে স্ট্রিং স্লাইস কাটতে হয়।
- কালেকশনের ডাটা স্লাইস করার সময় **বরো চেকার (Borrow Checker)** কীভাবে মেমোরি নষ্ট বা ইনভ্যালিডেশন রোধ করে।
- MiniStore-এ স্লাইসের বাস্তব প্রয়োগ: প্রোডাক্ট SKU থেকে ডিপার্টমেন্ট কোড পৃথকীকরণ (`"TECH-KEY-001"` -> `"TECH"`), কুপন কোড যাচাইকরণ, ব্যাচ ডিসকাউন্ট এবং ক্যাটালগ ইনভেন্টরির হিসাব।

---

## আমাদের কেন এটি প্রয়োজন?
[অধ্যায় ৭: বরোয়িং এবং রেফারেন্সে](/bn/chapters/07-borrowing-and-references) আমরা দেখেছি কীভাবে সম্পূর্ণ একটি অবজেক্ট বা স্ট্রাক্টকে ওনারশিপ না হারিয়ে `&T` অথবা `&mut T` দিয়ে ধার নেওয়া যায়। কিন্তু বাস্তব সফটওয়্যারে আমরা সবসময় সম্পূর্ণ ডাটা নিয়ে কাজ করি না।

MiniStore-এর সাধারণ কিছু কাজের কথা চিন্তা করুন:
১. প্রতিটি প্রোডাক্টের একটি SKU থাকে, যেমন: `"TECH-KEY-001"`। প্রায়ই আমাদের কেবল ডিপার্টমেন্ট কোডটি (`"TECH"`) দরকার হয়। এখন যদি `"TECH"` বের করার জন্য প্রতিবার হিপে নতুন একটি `String` তৈরি করতে হয়, তবে সার্ভার প্রতিদিন লক্ষ লক্ষ অপ্রয়োজনীয় ছোট ছোট হিপ মেমোরি বরাদ্দ ও মুক্ত করতেই ব্যস্ত থাকবে।
২. কাস্টমার যখন একটি ডিসকাউন্ট কোড ইনপুট দেয়, তখন আমাদের ভ্যালিডেশন ফাংশনটি কি কেবল হিপের `String` গ্রহণ করবে, নাকি কোডের ভেতর থাকা স্ট্যাটিক টেক্সট (`"SAVE10"`) ও সহজে গ্রহণ করতে পারবে?
৩. ইনভেন্টরির ২০টি পণ্যের মধ্যে প্রথম ৫টির উপর যদি ছাড় দিতে হয়, তবে কি সম্পূর্ণ অ্যারে কপি করতে হবে, নাকি সরাসরি ওই ৫টি উপাদানের একটি ভিউ ধার নেওয়া যায়?

```text
সম্পূর্ণ হিপ স্ট্রিং (স্ট্যাকে ২৪ বাইট):
┌──────────────┬─────┬──────────┐
│ ptr          │ len │ capacity │ ──► ['T','E','C','H','-','K','E','Y','-','0','0','1'] (Heap)
└──────────────┴─────┴──────────┘      ▲
                                       │
স্ট্রিং স্লাইস &str (স্ট্যাকে ১৬ বাইট): │
┌──────────────┬─────┐                 │
│ ptr          │ len │ ────────────────┘ (len = 4 বাইট: "TECH")
└──────────────┴─────┘
```

**রাস্টের যুগান্তকারী সমাধান**: **স্লাইস (`&str` এবং `&[T]`)**। স্লাইস মেমোরির নিজস্ব মালিক নয়; এটি একটি **ফ্যাট পয়েন্টার (Fat Pointer)** যা মেমোরির যেকোনো ধারাবাহিক ডাটার একটি নির্দিষ্ট অংশের দিকে নির্দেশ করে। এর ফলে মেমোরির কোনো নতুন কপি বা বরাদ্দ ছাড়াই (Zero-allocation) অত্যন্ত দ্রুত গতিতে কাজ করা যায়।

---

## সমস্যাটি কী?
অন্যান্য ভাষায় স্লাইসিংয়ের সীমাবদ্ধতা:
- **C**: স্ট্রিং মানেই নাল-টার্মিনেটেড ক্যারেক্টার পয়েন্টার (`char*`)। মূল স্ট্রিং না কেটে সাবস্ট্রিং বানাতে হলে `malloc` ও `strncpy` দিয়ে মেমোরি কপি করতে হয়। একটু ভুল হলেই ঘটে বাফার ওভারফ্লো এবং হ্যাকিংয়ের সুযোগ তৈরি হয়।
- **Java**: জাভায় স্ট্রিং ইমিউটেবল অবজেক্ট। অতীতে `substring()` মেথড মেমোরি শেয়ার করত, ফলে ছোট একটি সাবস্ট্রিং বিশাল একটি স্ট্রিংকে মেমোরি লিকের মতো আটকে রাখত। এটি ঠেকাতে আধুনিক জাভা প্রতিবার নতুন ক্যারেক্টার অ্যারে কপি করে, যা গার্বেজ কালেক্টরের ওপর চাপ বাড়ায়।
- **Python / Go**: পাইথনে যেকোনো স্লাইস (`s[0:4]`) নতুন স্ট্রিং অবজেক্ট তৈরি করে। গো-তে স্লাইস থাকলেও কনকারেন্ট পরিবর্তনের বিরুদ্ধে কম্পাইল টাইমে কোনো মালিকানা সুরক্ষা থাকে না।

MiniStore-এর প্রয়োজন:
- কোনো মেমোরি কপি ছাড়াই টেক্সট ও অ্যারের অংশবিশেষ দ্রুত পরিদর্শন করা।
- স্ট্রিং লিটারেল (`"SAVE10"`) এবং ব্যবহারকারীর ইনপুট (`String`) উভয়কেই একই ফাংশনে গ্রহণ করার সুবিধা।
- শতভাগ গ্যারান্টি যে একটি স্লাইস যখন ডাটা পড়ছে, তখন মূল মেমোরি বাফারটি মেমোরি থেকে মুছে বা স্থানান্তরিত হতে পারবে না।

---

## রাস্টের সমাধান (Rust Concept)

### ১. স্লাইস কী?
স্লাইস হলো একটি দ্বি-শব্দবিশিষ্ট (Two-word) অবজেক্ট, যাকে প্রোগ্রামিংয়ের ভাষায় **ফ্যাট পয়েন্টার (Fat Pointer)** বলা হয়:
১. মেমোরিতে ডাটার শুরুর ঠিকানার নির্দেশক (`ptr`)।
২. স্লাইসটিতে মোট কতটি উপাদান আছে তার দৈর্ঘ্য (`len`)।

৬৪-বিট কম্পিউটারে একটি স্লাইসের আকার সবসময় ঠিক **১৬ বাইট** (পয়েন্টার ৮ বাইট + দৈর্ঘ্য ৮ বাইট)—তা সে ৪ বাইটের ডাটা নির্দেশ করুক বা ৪ গিগাবাইটের ডাটা নির্দেশ করুক!

### ২. `String` বনাম `&str`
রাস্টে ওনড স্ট্রিং এবং ধার নেওয়া স্ট্রিং স্লাইসের মধ্যে পরিষ্কার পার্থক্য রয়েছে:

| বৈশিষ্ট্য | `String` | `&str` (স্ট্রিং স্লাইস) |
| :--- | :--- | :--- |
| **ওনারশিপ** | হিপ বাফারের নিজস্ব মালিক | মেমোরিতে থাকা UTF-8 বাফারের ধার নেওয়া ভিউ |
| **স্ট্যাকে আকার** | ২৪ বাইট (`ptr`, `len`, `capacity`) | ১৬ বাইট (`ptr`, `len`) |
| **পরিবর্তনশীলতা** | বড় করা, ছোট করা বা ডাটা যোগ করা যায় | শুধুই পাঠযোগ্য (ইমিউটেবল ভিউ) |
| **মেমোরি বরাদ্দ** | অপারেটিং সিস্টেম থেকে ডাইনামিক হিপ মেমোরি নেয় | শূন্য বরাদ্দ; বিদ্যমান মেমোরির দিকে নির্দেশ করে |
| **কোথায় থাকে** | সর্বদা হিপ মেমোরিতে থাকে | হিপ, স্ট্যাক বা বাইনারির স্ট্যাটিক টেক্সট (`&'static str`) হতে পারে |

```rust
// ১. কম্পাইল করা বাইনারির রিড-অনলি মেমোরিতে থাকা স্ট্যাটিক লিটারেল:
let greeting: &'static str = "Hello, MiniStore!";

// ২. হিপে বরাদ্দকৃত ওনড স্ট্রিং:
let mut dynamic_name = String::from("Wireless Mouse");

// ৩. dynamic_name থেকে ধার নেওয়া স্ট্রিং স্লাইস:
let word: &str = &dynamic_name[0..8]; // "Wireless"
```

### ৩. ডিরেফ কোয়ার্সন (Deref Coercion): `&str`-এর ম্যাজিক
ফাংশন লেখার সময় কেন `&String`-এর পরিবর্তে সবসময় `&str` লেখা উচিত?

```rust
// পরিহার করুন: কলারকে বাধ্য হয়ে &String পাঠাতে হয়
fn validate_code_bad(code: &String) -> bool { ... }

// আদর্শ রাস্ট কোড!
fn validate_code_good(code: &str) -> bool { ... }
```

যেহেতু `String` টাইপটি `Deref` ট্রেইট ইমপ্লিমেন্ট করে, তাই রাস্ট কম্পাইলার স্বয়ংক্রিয়ভাবে `&String`-কে `&str`-এ রূপান্তর করে নেয়। এই বৈশিষ্ট্যকে বলা হয় **Deref Coercion**।

এর ফলে প্যারামিটারে `code: &str` দিলে:
- কলার হিপের `String` থাকলে `&my_string` পাঠাতে পারে।
- কলার স্ট্রিং লিটারেল থাকলে সরাসরি `"SUMMER25"` পাঠাতে পারে।
- কলার কোনো বড় স্ট্রিংয়ের অংশবিশেষ `&my_string[0..4]` পাঠাতে পারে।
একটিমাত্র ফাংশন সিগনেচার দিয়ে সব ধরনের টেক্সট জিরো-ওভারহেডে পরিচালনা করা যায়!

### ৪. UTF-8 সুরক্ষা: কেন `s[0]` নিষিদ্ধ?
অন্যান্য ভাষায় আপনি `s[0]` লিখে প্রথম অক্ষরটি নিয়ে নেন। কিন্তু রাস্টে:

```rust
let s = String::from("hello");
// let c = s[0]; // কম্পাইলার এরর! `String` cannot be indexed by `{integer}`
```

**কারণ কী?** রাস্টের স্ট্রিং সম্পূর্ণ নিখুঁত **UTF-8 এনকোডিং** মেনে চলে:
- ইংরেজি অক্ষর (A-Z, 0-9) নেয় ১ বাইট।
- ল্যাটিন, গ্রীক বা আরবি অক্ষর নেয় ২ বাইট।
- বাংলা, হিন্দি, চীনা অক্ষর নেয় ৩ বাইট।
- ইমোজি নেয় ৪ বাইট।

রাস্ট যদি `s[0]` অনুমতি দিত, তবে বাংলা শব্দ `"বই"`-এর ক্ষেত্রে (যেখানে `'ব'` অক্ষরটি ৩ বাইট: `0xE0, 0xA6, 0xAC`) প্রথম ইনডেক্স কল করলে আপনি পেতেন অর্ধেক ভাঙা বাইট `0xE0`, যা কোনো অর্থপূর্ণ বর্ণ নয়!

স্ট্রিং স্লাইস করার সময়:
```rust
let s = "Hello";
let slice = &s[0..2]; // "He"
```
এখানে `0..2` হলো **বাইটের সংখ্যা**, অক্ষরের সংখ্যা নয়। যদি কোনো ৩-বাইটের বাংলা অক্ষরের মাঝখানে (যেমন ১ বা ২ নম্বর বাইটে) স্লাইস কাটার চেষ্টা করেন, তবে রাস্ট রানটাইমে প্যানিক (Panic) করবে যাতে ভুল ডাটা মেমোরিতে না ছড়ায়। নিরাপদ স্লাইসের জন্য `.is_char_boundary()` বা `.chars()` ব্যবহার করা হয়।

### ৫. অ্যারে স্লাইস (`&[T]` এবং `&mut [T]`)
স্ট্রিং স্লাইসের মতোই অ্যারের একটি অংশকে ধার নেওয়া যায়:

```rust
let numbers: [u32; 5] = [10, 20, 30, 40, 50]; // স্ট্যাকে ২০ বাইটের অ্যারে

// ইমিউটেবল স্লাইস:
let first_three: &[u32] = &numbers[0..3]; // [10, 20, 30]

// মিউটেবল স্লাইস (ইন-প্লেস পরিবর্তন):
let mut prices = [1000, 2000, 3000];
let first_two: &mut [u32] = &mut prices[0..2];
first_two[0] = 800; // prices এখন [800, 2000, 3000]
```

### ৬. স্লাইসের বরোয়িং সুরক্ষা (Borrow Invalidation)
স্লাইস একটি ধার বা রেফারেন্স। তাই এতে **Aliasing XOR Mutability** প্রযোজ্য:

```rust
let mut s = String::from("hello world");
let word = &s[0..5]; // s-এর ইমিউটেবল স্লাইস ধার নেওয়া হলো

// s.clear(); // কম্পাইলার এরর! স্লাইস বেঁচে থাকা অবস্থায় s পরিবর্তন নিষিদ্ধ!
println!("The word is: {word}");
```

রাস্ট যদি `s.clear()` করতে দিত, তবে হিপের মেমোরি রিলিজ হয়ে যেত এবং `word` একটি মৃত মেমোরির দিকে নির্দেশ করত (Dangling Pointer)। রাস্টের বরো চেকার কম্পাইল টাইমে এটি পুরোপুরি অসম্ভব করে তোলে।

---

## অন্যান্য ভাষার সাথে তুলনা

| বিষয় | C / C++ | Java | Python | Go | Rust |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **স্ট্রিং স্লাইস** | `std::string_view` (C++17)। অনিরাপদ, সহজেই ড্যাংলিং মেমোরি তৈরি করে। | নেই। `substring()` পুরো নতুন বাফার কপি করে। | প্রতিটি স্লাইসে নতুন স্ট্রিং কপি তৈরি হয়। | স্লাইস হেডার `(ptr, len)`। কনকারেন্ট মিউটেশন সুরক্ষা নেই। | `&str`: ১৬ বাইটের সুরক্ষিত ফ্যাট পয়েন্টার। |
| **মেমোরি খরচ** | শূন্য (অনিরাপদ)। | অবজেক্ট বরাদ্দ + মেমোরি কপি। | অবজেক্ট বরাদ্দ + মেমোরি কপি। | শূন্য বরাদ্দ। | **শূন্য বরাদ্দ + শতভাগ মেমোরি সুরক্ষা**। |
| **অ্যারে সাব-রেঞ্জ** | পয়েন্টার পাটিগণিত (`ptr + offset`)। কোনো বাউন্ডারি চেক নেই। | `Arrays.copyOfRange` (নতুন অ্যারে তৈরি করে)। | নতুন লিস্ট কপি (`list[1:3]`)। | স্লাইস `slice[1:3]`। | `&[T]` / `&mut [T]` কম্পাইল টাইম বরো ট্র্যাকিং সহ। |
| **এনকোডিং** | যেকোনো বাইট। | UTF-16 (`char` ২ বাইট)। | ইউনিকোড কোডপয়েন্ট। | সাধারণ বাইট সিকোয়েন্স। | কঠোরভাবে পরীক্ষিত UTF-8। ভুল কাটলে কম্পাইলার/রানটাইম গার্ড। |

---

## ছোট উদাহরণ (Small Example)
নিচে স্ট্রিং এবং অ্যারে স্লাইসের একটি সংক্ষিপ্ত উদাহরণ দেওয়া হলো:

```rust
fn print_prefix(text: &str, byte_count: usize) {
    if text.is_char_boundary(byte_count) {
        println!("Prefix: {}", &text[..byte_count]);
    } else {
        println!("Invalid UTF-8 character boundary!");
    }
}

fn sum_slice(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for &n in numbers {
        sum += n;
    }
    sum
}

fn main() {
    let message = String::from("MiniStore Inventory");
    print_prefix(&message, 9); // "MiniStore"

    let data = [10, 20, 30, 40, 50];
    let total = sum_slice(&data[1..4]); // [20, 30, 40] যোগ করে = ৯০
    println!("Total: {total}");
}
```

---

## MiniStore-এ বাস্তব প্রয়োগ
MiniStore-এ স্লাইস যেভাবে কাজ করছে:
১. **SKU থেকে ডিপার্টমেন্ট বের করা**: প্রোডাক্টের SKU ফরম্যাট `DEPT-ITEM-ID` (যেমন `"TECH-KEY-001"`)। `department_code(&self) -> &str` কোনো নতুন হিপ বরাদ্দ ছাড়াই সরাসরি `"TECH"` ভিউ প্রদান করে।
২. **নিরাপদ নাম ছাঁটাই (Safe Truncation)**: মোবাইল ডিসপ্লেতে দেখানোর জন্য বড় পণ্যের নাম ছোট করতে হয়। `truncated_name(&self, max_bytes: usize) -> &str` কোনো বাংলা বা ইউনিকোড অক্ষর ভেঙে ফেলা ছাড়াই নিরাপদ স্লাইস তৈরি করে।
৩. **ইউনিভার্সাল কুপন ভ্যালিডেশন**: `validate_coupon(code: &str)` স্ট্যাটিক লিটারেল (`"SAVE10"`) এবং ডাইনামিক `String` উভয়কেই অনায়াসে গ্রহণ করে।
৪. **ব্যাচ প্রাইস ডিসকাউন্ট**: `apply_batch_markdown(prices: &mut [u32], discount_cents: u32)` অ্যারের নির্দিষ্ট অংশে ইন-প্লেস ডিসকাউন্ট দেয়।
৫. **ইনভেন্টরি সামারি**: `summarize_inventory(catalog: &[Product])` যেকোনো প্রোডাক্ট স্লাইসের মোট স্টক ও মূল্য হিসাব করে।

---

## কোড (Code)
নিচে `ministore/src/main.rs`-এর সম্পূর্ণ ও কম্পাইলযোগ্য কোড দেওয়া হলো:

```rust
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
            // Find the nearest valid UTF-8 character boundary at or below max_bytes
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
    }

    /// Borrows `&mut self` mutably: updates email address in place.
    pub fn update_email(&mut self, new_email: String) {
        self.email = new_email;
    }
}

/// OrderId implements `Copy`: 8-byte scalar on the stack, never moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderId(pub u64);

/// An unconfirmed order owning its items.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingOrder {
    pub order_id: OrderId,
    pub customer: Customer,
    pub product: Product,
    pub quantity: u32,
}

/// Finalized invoice/receipt produced when checkout consumes `PendingOrder`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmedReceipt {
    pub receipt_id: String,
    pub order_id: OrderId,
    pub customer_name: String,
    pub product_name: String,
    pub quantity: u32,
    pub total_cents: u32,
}

// ============================================================================
// String & Array Slice Functions: Zero-Copy Views Into Memory
// ============================================================================

/// Borrows `&Product` immutably: calculates subtotal without taking ownership.
pub fn calculate_line_total(product: &Product, quantity: u32) -> u32 {
    product.price_cents * quantity
}

/// Borrows `&Customer` immutably: calculates discount based on VIP status.
pub fn calculate_discount(customer: &Customer, subtotal_cents: u32) -> u32 {
    if customer.is_vip {
        (subtotal_cents * 10) / 100 // 10% discount for VIPs
    } else {
        0
    }
}

/// Borrows an array slice of prices `&[u32]`.
/// Works seamlessly with fixed stack arrays `[u32; N]` or slices without copying data.
pub fn calculate_batch_total(prices: &[u32]) -> u32 {
    let mut total = 0;
    for &price in prices {
        total += price;
    }
    total
}

/// Borrows a mutable slice `&mut [u32]` to apply markdown discounts in-place.
pub fn apply_batch_markdown(prices: &mut [u32], markdown_cents: u32) {
    for price in prices.iter_mut() {
        if *price > markdown_cents {
            *price -= markdown_cents;
        } else {
            *price = 0;
        }
    }
}

/// Calculates inventory totals using a slice of Products `&[Product]`.
/// Returns `(total_units, total_valuation_cents)`.
pub fn summarize_inventory(catalog: &[Product]) -> (u32, u32) {
    let mut total_units = 0;
    let mut total_cents = 0;
    for product in catalog {
        total_units += product.stock;
        total_cents += product.price_cents * product.stock;
    }
    (total_units, total_cents)
}

/// Accepts any string slice `&str` (literal, heap slice, or &String via Deref coercion).
/// Validates promo codes and returns discount percentage.
pub fn validate_coupon(code: &str) -> Result<u32, &'static str> {
    let trimmed = code.trim();
    if trimmed.eq_ignore_ascii_case("SAVE10") {
        Ok(10)
    } else if trimmed.eq_ignore_ascii_case("SUMMER25") {
        Ok(25)
    } else if trimmed.is_empty() {
        Err("Coupon code cannot be empty")
    } else {
        Err("Invalid coupon code")
    }
}

/// Formats a single receipt line item using string slices `&str` to avoid unnecessary String copies.
pub fn format_line_summary(sku: &str, name: &str, price_cents: u32) -> String {
    format!("[{sku}] {name} - ${:.2}", price_cents as f64 / 100.0)
}

/// Borrows both `&Customer` and `&Product` immutably to render a preview.
pub fn print_order_preview(customer: &Customer, product: &Product, quantity: u32) {
    let subtotal = calculate_line_total(product, quantity);
    let discount = calculate_discount(customer, subtotal);
    let final_total = subtotal - discount;

    println!("--- Order Preview (Borrowed Read-Only) ---");
    println!("Customer: {}", customer.display_badge());
    println!(
        "Item:     {}",
        format_line_summary(&product.sku, &product.name, product.price_cents)
    );
    println!("Dept:     {}", product.department_code());
    println!("Quantity: {}", quantity);
    println!("Subtotal: ${:.2}", subtotal as f64 / 100.0);
    println!("Discount: ${:.2}", discount as f64 / 100.0);
    println!("Estimate: ${:.2}", final_total as f64 / 100.0);
}

/// Consumes ownership of `PendingOrder` by value (Move Semantics).
pub fn finalize_order(order: PendingOrder) -> ConfirmedReceipt {
    let subtotal = calculate_line_total(&order.product, order.quantity);
    let discount = calculate_discount(&order.customer, subtotal);
    let total_cents = subtotal - discount;

    ConfirmedReceipt {
        receipt_id: format!("REC-{}-{}", order.order_id.0, order.customer.id),
        order_id: order.order_id,
        customer_name: order.customer.name,
        product_name: order.product.name,
        quantity: order.quantity,
        total_cents,
    }
}

fn main() {
    println!("=== MiniStore: Strings, Slices & Zero-Copy Views ===\n");

    // 1. String Slices (&str) vs Owned Strings (String)
    let sample_sku = String::from("TECH-KEY-001");
    // Slicing without allocating: 16-byte fat pointer (ptr + len)
    let dept_slice: &str = &sample_sku[0..4];
    let id_slice: &str = &sample_sku[5..];
    println!("1. String Slice Decomposition (Zero Allocations):");
    println!("   Full SKU:    {sample_sku} (Heap String: 24 bytes on stack)");
    println!("   Dept Slice:  {dept_slice} (Slice view: 16 bytes on stack)");
    println!("   ID Slice:    {id_slice}\n");

    // 2. Deref Coercion: &String automatically coerces to &str
    println!("2. Universal &str Function Parameters & Deref Coercion:");
    let coupon_literal = "  SAVE10  "; // &'static str
    let coupon_heap = String::from("SUMMER25"); // String

    println!(
        "   Validating literal: {:?}",
        validate_coupon(coupon_literal)
    );
    println!(
        "   Validating heap string (&String coerces to &str): {:?}",
        validate_coupon(&coupon_heap)
    );

    // 3. Products with SKU Slices and Safe Truncation
    let keyboard = Product::new(
        501,
        String::from("TECH-KEY-001"),
        String::from("RGB Mechanical Gaming Keyboard with Hot-Swap Switches"),
        12000,
        15,
    );

    println!("\n3. Product Slice Methods:");
    println!("   Product SKU:       {}", keyboard.sku);
    println!("   Department:        {}", keyboard.department_code());
    println!(
        "   Short Title (25B): {}...",
        keyboard.truncated_name(25)
    );
    println!(
        "   Matches 'TECH':    {}",
        keyboard.matches_sku_prefix("TECH")
    );

    // 4. Array Slices (&[T]) & Mutable Slices (&mut [T])
    println!("\n4. Array Slices (&[T]) for Batch Pricing:");
    let mut clearance_prices = [2500, 4500, 12000, 8000]; // Stack array [u32; 4]
    println!(
        "   Initial prices:    {:?} (Total = ${:.2})",
        clearance_prices,
        calculate_batch_total(&clearance_prices) as f64 / 100.0
    );

    // Pass a subslice `&mut clearance_prices[0..2]` to apply markdown only to first two items
    apply_batch_markdown(&mut clearance_prices[0..2], 500);
    println!(
        "   After $5 markdown on first 2 items: {:?}",
        clearance_prices
    );

    // 5. Catalog Slices (&[Product])
    let catalog = [
        keyboard.clone(),
        Product::new(
            502,
            String::from("OFFC-CHAIR-002"),
            String::from("Ergonomic Mesh Chair"),
            35000,
            8,
        ),
        Product::new(
            503,
            String::from("TECH-MOU-003"),
            String::from("Wireless Laser Mouse"),
            4500,
            25,
        ),
    ];

    let (total_units, valuation) = summarize_inventory(&catalog);
    println!("\n5. Catalog Summary (Calculated via &[Product] slice):");
    println!("   Total Units:     {total_units}");
    println!("   Total Valuation: ${:.2}", valuation as f64 / 100.0);

    // 6. Preview and Checkout
    let customer = Customer::new(
        101,
        String::from("Ada Lovelace"),
        String::from("ada@analytical.org"),
        true,
    );

    println!("\n6. Order Preview & Checkout:");
    print_order_preview(&customer, &keyboard, 2);

    let pending_order = PendingOrder {
        order_id: OrderId(9001),
        customer,
        product: keyboard,
        quantity: 2,
    };

    let receipt = finalize_order(pending_order);
    println!("\n--- Checkout Confirmed ---");
    println!("Receipt: {}", receipt.receipt_id);
    println!("Item:    {} x {}", receipt.product_name, receipt.quantity);
    println!("Paid:    ${:.2}", receipt.total_cents as f64 / 100.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_slice_department_code() {
        let p = Product::new(
            1,
            String::from("ELEC-MON-001"),
            String::from("4K Monitor"),
            40000,
            5,
        );
        assert_eq!(p.department_code(), "ELEC");
        assert!(p.matches_sku_prefix("ELEC"));
        assert!(!p.matches_sku_prefix("FURN"));
    }

    #[test]
    fn test_truncated_name_ascii_and_boundaries() {
        let p = Product::new(
            2,
            String::from("BOOK-RUST-01"),
            String::from("Rust Programming in Depth"),
            5000,
            20,
        );
        assert_eq!(p.truncated_name(4), "Rust");
        assert_eq!(p.truncated_name(100), "Rust Programming in Depth");

        // UTF-8 multi-byte test (Bengali character: 'ব' is 3 bytes: 0xE0, 0xA6, 0xAC)
        let p_utf8 = Product::new(
            3,
            String::from("BOOK-BN-02"),
            String::from("বই"),
            1000,
            10,
        );
        // Slicing at 2 bytes would slice inside 'ব'; truncated_name snaps safely to 0
        assert_eq!(p_utf8.truncated_name(2), "");
        assert_eq!(p_utf8.truncated_name(3), "ব");
    }

    #[test]
    fn test_validate_coupon_deref_coercion() {
        // String literal (&'static str)
        assert_eq!(validate_coupon("SAVE10"), Ok(10));
        assert_eq!(validate_coupon("  save10  "), Ok(10));

        // Owned String coerced via &coupon
        let dynamic_code = String::from("SUMMER25");
        assert_eq!(validate_coupon(&dynamic_code), Ok(25));

        assert!(validate_coupon("UNKNOWN").is_err());
        assert!(validate_coupon("   ").is_err());
    }

    #[test]
    fn test_array_slices_and_mutable_markdown() {
        let mut prices = [1000, 2500, 5000];

        // Slice calculation:
        assert_eq!(calculate_batch_total(&prices), 8500);
        assert_eq!(calculate_batch_total(&prices[1..3]), 7500);

        // Mutable slice markdown on first two elements only:
        apply_batch_markdown(&mut prices[0..2], 500);
        assert_eq!(prices, [500, 2000, 5000]);
    }

    #[test]
    fn test_catalog_inventory_summary() {
        let items = [
            Product::new(1, String::from("A-1"), String::from("Item 1"), 100, 10),
            Product::new(2, String::from("A-2"), String::from("Item 2"), 200, 5),
        ];

        let (units, valuation) = summarize_inventory(&items);
        assert_eq!(units, 15);
        assert_eq!(valuation, (100 * 10) + (200 * 5)); // 1000 + 1000 = 2000
    }
}
```

---

## কোড বিশ্লেষণ (Understanding The Code)

### ১. রেঞ্জ স্লাইসিং দিয়ে শূন্য-বরাদ্দে সাবস্ট্রিং
`department_code` মেথডটিতে:
```rust
match self.sku.find('-') {
    Some(idx) => &self.sku[..idx],
    None => &self.sku[..],
}
```
`&self.sku[..idx]` একটি ১৬-বাইটের স্লাইস তৈরি করে যা মূল `self.sku`-এর হিপ পয়েন্টারের দিকে নির্দেশ করে যার দৈর্ঘ্য `idx`। কোনো নতুন স্ট্রিং তৈরি করা হয়নি। রিটার্ন করা `&str`-এর লাইফটাইম স্বয়ংক্রিয়ভাবে `&self`-এর সাথে যুক্ত থাকে।

### ২. `&mut [T]` দিয়ে অ্যারের সাব-রেঞ্জে ইন-প্লেস পরিবর্তন
`apply_batch_markdown(&mut clearance_prices[0..2], 500)` লাইনে:
আমরা সম্পূর্ণ অ্যারে পাস না করে কেবল ইনডেক্স ০ এবং ১-এর মিউটেবল স্লাইস পাঠিয়েছি। ফলে বাকি ইনডেক্সগুলোর ওপর কোনো প্রভাব পড়েনি। স্লাইস মেমোরির নির্দিষ্ট সীমারেখায় সুনির্দিষ্টভাবে কাজ করার সর্বোচ্চ নিয়ন্ত্রণ দেয়।

---

## সাধারণ ভুলসমূহ (Common Mistakes)

### ১. মাল্টি-বাইট UTF-8 অক্ষরের মাঝখানে স্লাইস কাটা
```rust
let text = "বাংলা"; // প্রতিটি বাংলা বর্ণ ৩ বাইট নেয়
let bad_slice = &text[0..2]; // RUNTIME PANIC! byte index 2 is not a char boundary; it is inside 'ব'
```
ইংরেজি ছাড়া অন্য ভাষার স্ট্রিং কাটতে হলে সবসময় `.is_char_boundary(idx)` অথবা `.chars()` ইটারেটর ব্যবহার করতে হবে।

### ২. `fn foo(s: &str)`-এর জায়গায় `fn foo(s: &String)` লেখা
প্যারামিটারে `&String` লিখলে কলারের কাছে কেবল স্ট্যাটিক টেক্সট `"hello"` থাকলেও তাকে বাধ্য হয়ে হিপে `String` তৈরি করতে হয়। সবসময় প্যারামিটারে `&str` অগ্রাধিকার দিন।

---

## কম্পাইলার এরর বিশ্লেষণ (Compiler Errors)

### Error E0502: স্লাইস রেফারেন্স ধরে রেখে মূল স্ট্রিং পরিবর্তন করা
ধরুন আপনি লিখেছেন:

```rust
let mut sku = String::from("TECH-KEY-001");
let dept = &sku[0..4]; // sku-এর ইমিউটেবল স্লাইস
sku.push_str("-DISCOUNT"); // sku-তে নতুন টেক্সট যোগ করার চেষ্টা
println!("Dept: {dept}"); // এখানে স্লাইস ব্যবহার করা হয়েছে
```

রাস্ট কম্পাইলার তীব্র সতর্কবার্তা দিয়ে বলবে:

```text
error[E0502]: cannot borrow `sku` as mutable because it is also borrowed as immutable
  --> src/main.rs:188:5
   |
187|     let dept = &sku[0..4];
   |                 --- immutable borrow occurs here
188|     sku.push_str("-DISCOUNT");
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
189|     println!("Dept: {dept}");
   |                     ------ immutable borrow later used here
```

**কেন এটি আপনাকে সুরক্ষা দেয়**: `push_str`-এর কারণে যদি হিপ বাফারের ক্যাপাসিটি ফুরিয়ে যায়, তবে অপারেটিং সিস্টেম মেমোরির অন্য কোনো ফাঁকা জায়গায় পুরো বাফার সরিয়ে নিয়ে যেতে পারে। রাস্ট যদি এই মিউটেশন অনুমোদন করত, তবে `dept` পুরানো খালি মেমোরির ঠিকানায় (Dangling Pointer) নির্দেশ করে বসে থাকত। বরো চেকার এই বিপর্যয় কম্পাইল টাইমে চিরতরে প্রতিরোধ করে।

---

## অনুশীলনী (Practice)
১. **আইটেম আইডি বের করা**: `Product`-এ একটি মেথড যোগ করুন `pub fn item_id(&self) -> &str`, যা প্রথম হাইফেনের পরের অংশটি ফিরিয়ে দেবে (যেমন `"TECH-KEY-001"` থেকে `"KEY-001"` পাবে)।
২. **দামি পণ্য গোনা**: একটি ফাংশন লিখুন `filter_high_value(prices: &[u32], threshold_cents: u32) -> usize`, যা প্রাইস স্লাইস থেকে নির্দিষ্ট সীমার বেশি দামের পণ্যের সংখ্যা গণনা করবে।
৩. **টেস্টে প্রমাণ করুন**: `src/main.rs`-এ টেস্ট লিখে নিশ্চিত করুন যে `item_id` এবং `filter_high_value` সঠিকভাবে কাজ করছে।
৪. `cargo test` দিয়ে সমাধান নিশ্চিত করুন।

---

## চেকপয়েন্ট (Checkpoint)
- [x] ওনড `String` এবং ধার নেওয়া `&str`-এর মৌলিক পার্থক্য বুঝেছেন।
- [x] স্লাইসের ১৬-বাইটের ফ্যাট পয়েন্টার (`ptr` + `len`) রূপ উপলব্ধি করেছেন।
- [x] ডিরেফ কোয়ার্সন (Deref Coercion) এবং কেন প্যারামিটারে `&str` সেরা তা আয়ত্ত করেছেন।
- [x] অ্যারে স্লাইস (`&[T]`) এবং মিউটেবল স্লাইস (`&mut [T]`) ব্যবহার করেছেন।
- [x] UTF-8 বাইট ইনডেক্সিং এবং ক্যারেক্টার বাউন্ডারির নিরাপত্তা নিশ্চিত করেছেন।
- [x] মেমোরি রিলোকেশনের সময় বরো চেকার কীভাবে স্লাইস ইনভ্যালিডেশন আটকায় তা প্রত্যক্ষ করেছেন।

---

## আমরা কী শিখলাম
- স্লাইস কোনো মেমোরি কপি ছাড়াই মেমোরির যেকোনো ধারাবাহিক অংশের উপর জিরো-অ্যালোকেশন ভিউ প্রদান করে।
- `&str` এবং `&[T]` যেকোনো সাধারণ ফাংশনকে সব ধরণের স্ট্যাটিক, হিপ বা সাব-রেঞ্জের সাথে সর্বজনীনভাবে কাজ করার সুযোগ দেয়।
- রাস্ট গ্যারান্টি দেয় যে স্লাইস যতদিন জীবিত আছে, মূল ডাটা ততদিন মেমোরি থেকে মুছে যেতে বা স্থানান্তরিত হতে পারবে না।

---

## পরবর্তীতে কী আসছে
আমরা একক ডাটা ও স্লাইস শিখলাম। কিন্তু ডাইনামিক আকারের পণ্যের তালিকা কীভাবে পরিচালনা করব? **অধ্যায় ৯: কালেকশনস (Collections)**-এ আমরা রাস্টের স্ট্যান্ডার্ড লাইব্রেরির শক্তিশালী হিপ কালেকশনগুলো শিখব: `Vec<T>`, `HashMap<K, V>`, এবং `HashSet<T>`।
