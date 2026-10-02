# অধ্যায় ১৬: লাইফটাইমস (Lifetimes)

## আপনি যা শিখবেন
- **লাইফটাইমস (Lifetimes)** কী এবং কোনো গার্বেজ কালেক্টর (GC) ছাড়াই রাস্ট কীভাবে রেফারেন্সের শতভাগ মেমরি নিরাপত্তা নিশ্চিত করে।
- কম্পাইল টাইমে **ড্যাংলিং রেফারেন্স (Dangling References)** শনাক্ত ও প্রতিরোধে **বরো চেকার (Borrow Checker)**-এর ভূমিকা।
- লাইফটাইম অ্যানোটেশন সিনট্যাক্স: **`'a`**, **`'b`**, এবং লাইফটাইম প্যারামিটার পড়ার নিয়ম।
- কেন লাইফটাইম অ্যানোটেশন **কোনো ভ্যালুর আসল আয়ুষ্কাল পরিবর্তন করে না**, বরং একাধিক রেফারেন্সের পারস্পরিক সম্পর্ক নির্দেশ করে।
- ফাংশনে লাইফটাইম অ্যানোটেশন: কখন এটি বাধ্যতামূলক এবং কীভাবে এটি রিটার্ন করা রেফারেন্সকে সুরক্ষিত রাখে।
- কম্পাইলারের **তিনটি লাইফটাইম এলিশন নিয়ম (Three Lifetime Elision Rules)** যার ফলে সাধারণ মেথডগুলোতে আমাদের ম্যানুয়ালি `'a` লিখতে হয় না।
- রেফারেন্স ধারণকারী স্ট্রাক্ট: কোনো হিপ ক্লোনিং বা মেমরি কপি ছাড়া উচ্চগতির **জিরো-কপি আর্কিটেকচার (Zero-Copy Structs)** যেমন `OrderReceipt<'a>` তৈরি করা।
- লাইফটাইম সমৃদ্ধ স্ট্রাক্টে মেথড ইমপ্লিমেন্টেশন (`impl<'a> OrderReceipt<'a>`) লেখার নিয়ম।
- বিশেষ সংরক্ষিত **`'static`** লাইফটাইম: স্ট্রিং লিটারাল, গ্লোবাল কনস্ট্যান্ট এবং `T: 'static` ট্রেইট বাউন্ড।
- সাধারণ কম্পাইলার এরর এবং সমাধানের কৌশল:
  - `error[E0106]: missing lifetime specifier`
  - `error[E0515]: cannot return reference to local variable`
  - `error[E0597]: borrowed value does not live long enough`
- MiniStore অ্যাপ্লিকেশনে লাইফটাইম সংহতকরণ:
  - ইনভয়েস এবং স্লিপ তৈরির জন্য জিরো-কপি `OrderReceipt<'a>` তৈরি।
  - দুটি ধার করা `Product` রেফারেন্সের দাম তুলনা করতে `find_higher_priced<'a>` তৈরি।
  - একাধিক অপশনাল রেফারেন্স ইনপুটের মধ্য থেকে যোগাযোগের সঠিক তথ্য বেছে নিতে `best_contact_info<'a>` লেখা।
  - কোনো মেমরি অ্যালোকেশন ছাড়াই স্টোরের রিটার্ন পলিসি জানাতে `store_policy() -> &'static str` যুক্ত করা।

---

## কেন আমাদের এটি প্রয়োজন?

অধ্যায় ৬ এবং ৭-এ আমরা **ওনারশিপ (Ownership)** এবং **বরোয়িং (Borrowing)** শিখেছি:
- কোনো ভেরিয়েবল যখন তার স্কোপের বাইরে চলে যায়, রাস্ট স্বয়ংক্রিয়ভাবে তার মেমরি মুক্ত (Drop) করে দেয়।
- ওনারশিপ হস্তান্তর না করেই ডাটা পড়তে বা পরিবর্তন করতে আমরা রেফারেন্স (`&T` বা `&mut T`) বরো করতে পারি।

সাধারণত বেশিরভাগ সময় রাস্ট কম্পাইলার নিজে থেকেই রেফারেন্সের বৈধতা বুঝে নিতে পারে:
```rust
fn print_length(s: &str) {
    println!("Length: {}", s.len());
}
```
এখানে `s` ধার করা হয়, ব্যবহার করা হয় এবং ফাংশন শেষ হওয়ার সাথে সাথে বরো শেষ হয়ে যায়। এখানে কোনো বিভ্রান্তি বা অস্পষ্টতা নেই।

কিন্তু নিচের সাধারণ দৃশ্যপটটি লক্ষ্য করুন:
```rust
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

আপনি যদি এই কোডটি কম্পাইল করার চেষ্টা করেন, রাস্ট **কম্পাইল করতে সরাসরি অস্বীকৃতি জানাবে**:
```text
error[E0106]: missing lifetime specifier
 --> src/main.rs:1:33
  |
1 | fn longest(x: &str, y: &str) -> &str {
  |               ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the
          signature does not say whether it is borrowed from `x` or `y`
```

### সমস্যা: রেফারেন্সের অনিশ্চিত উৎস এবং ড্যাংলিং পয়েন্টার
এরর মেসেজটি গভীরভাবে লক্ষ্য করুন: রাস্ট বুঝতে পারছে না যে ফেরত দেওয়া রেফারেন্সটি আসলে `x`-এর মেমরিকে নির্দেশ করছে নাকি `y`-এর মেমরিকে!
- কলার যদি এমন একটি `x` পাস করে যার আয়ু ১০ সেকেন্ড, কিন্তু `y`-এর আয়ু মাত্র ২ সেকেন্ড—তবে ফেরত আসা রেফারেন্সটির আয়ু কতক্ষণ হবে?
- কলার যদি রিটার্ন ভ্যালুটিকে এমন একটি ভেরিয়েবলে সংরক্ষণ করে যা `y` ধ্বংস হওয়ার পরেও বেঁচে থাকে, কিন্তু `longest` ফাংশনটি রানটাইমে `y`-কেই রিটার্ন করেছিল—তখন সেই রেফারেন্সটি পড়তে গেলে প্রোগ্রামটি মেমরির অস্তিত্বহীন ঠিকানায় আঘাত হানবে! একে বলা হয় **Use-after-free** বা **ড্যাংলিং পয়েন্টার (Dangling Pointer)** বাগ।

### অন্যান্য প্রোগ্রামিং ভাষা কীভাবে এটি মোকাবেলা করে?
১. **সি / সি++ (C / C++)**: রেফারেন্স বা পয়েন্টারের বৈধতার সম্পূর্ণ দায়ভার প্রোগ্রামারের কাঁধে ছেড়ে দেয়। কোনো ফাংশন যদি সাময়িক মেমরির বা ধ্বংস হয়ে যাওয়া অবজেক্টের পয়েন্টার ফেরত দেয়, তবে কোড কোনো এরর ছাড়াই কম্পাইল হয়; কিন্তু প্রোগ্রাম চলার সময় অপ্রত্যাশিতভাবে **Segmentation Fault** বা মেমরি করাপশন ঘটিয়ে ক্র্যাশ করে।
২. **জাভা / গো / সি# / পাইথন (Java / Go / C# / Python)**: সমস্ত অবজেক্টকে হিপে (Heap) তৈরি করে এবং ব্যাকগ্রাউন্ডে একটি **গার্বেজ কালেক্টর (GC)** চালায়। মেমরিতে যতক্ষণ একটি রেফারেন্সও বেঁচে থাকে, জিসি অবজেক্টটিকে মুছে ফেলে না। এটি নিরাপদ হলেও এর জন্য প্রচুর অতিরিক্ত মেমরি অপচয় হয়, ক্যাশ মিস বাড়ে এবং অ্যাপ্লিকেশন হঠাৎ ফ্রিজ (GC Pause) হয়ে পারফরম্যান্স হ্রাস পায়।
৩. **রাস্ট (Rust)**: রাস্ট কোনো গার্বেজ কালেক্টর ছাড়াই **শতভাগ মেমরি নিরাপত্তা** প্রদান করে। কম্পাইলারের ভেতরে থাকা **বরো চেকার (Borrow Checker)** কম্পাইল টাইমেই গাণিতিকভাবে প্রমাণ করে নেয় যে প্রতিটি রেফারেন্স যতটুক সময় বেঁচে থাকবে, তার পেছনের আসল ডাটাও ততটুক সময় অবশ্যই অক্ষত থাকবে।

আর এই প্রমাণ সম্পন্ন করার জন্যই রাস্ট আমাদের কাছে ফাংশন এবং স্ট্রাক্টের সীমানায় **লাইফটাইম অ্যানোটেশন (`'a`)** প্রত্যাশা করে।

---

## লাইফটাইম আসলে কী?

একটি **লাইফটাইম** হলো কোডের সেই নির্দিষ্ট পরিসর (Scope বা Span), যতক্ষণ পর্যন্ত একটি রেফারেন্স মেমরিতে সম্পূর্ণ বৈধ থাকে।

```rust
{
    let r;                // ---------+-- 'a
                          //          |
    {                     //          |
        let x = 5;        // -+-- 'b  |
        r = &x;           //  |       |
    }                     // -+       |
                          //          |
    println!("r: {}", r); // ---------+ // ERROR: ধার করা অবস্থায় `x` ড্রপ হয়ে গেছে!
}
```

উপরের চিত্রে:
- `r`-এর লাইফটাইম হলো `'a`।
- `x`-এর লাইফটাইম হলো `'b`।
- `'b` স্পষ্টভাবে `'a`-এর চেয়ে ছোট। ভেতরের স্কোপ শেষ হওয়ার সাথে সাথেই `x` মেমরি থেকে মুছে যায়।
- এর ফলে `r` এমন একটি মেমরির দিকে তাকিয়ে থাকে যা আর নেই! রাস্টের বরো চেকার কম্পাইল টাইমে এটি ধরে ফেলে এবং কোড আটকে দেয়।

### লাইফটাইম অ্যানোটেশনের সিনট্যাক্স
লাইফটাইম প্যারামিটার শুরু হয় একটি অ্যাপোস্ট্রফি `'` দিয়ে এবং প্রচলিত নিয়ম অনুযায়ী ছোট হাতের অক্ষর যেমন `'a`, `'b`, বা `'c` দিয়ে লেখা হয়:
- `&i32` — কোনো স্পষ্ট লাইফটাইম ছাড়া সাধারণ রেফারেন্স।
- `&'a i32` — একটি রেফারেন্স যার লাইফটাইম স্পষ্টভাবে `'a`।
- `&'a mut i32` — একটি মিউটেবল রেফারেন্স যার লাইফটাইম স্পষ্টভাবে `'a`।

> [!IMPORTANT]
> **লাইফটাইম অ্যানোটেশন কোনো ভ্যালুর আসল আয়ুষ্কাল পরিবর্তন করতে পারে না!**
> যেমন জেনেরিক প্যারামিটার `<T>` কোনো ডাটার অভ্যন্তরীণ গঠন বদলে দেয় না, তেমনি লাইফটাইম প্যারামিটার `<'a>` কেবল একাধিক রেফারেন্সের পারস্পরিক সম্পর্কের রূপরেখা কম্পাইলারকে বুঝিয়ে দেয়, যাতে কম্পাইলার তাদের নিরাপত্তা যাচাই করতে পারে।

---

## ফাংশনে লাইফটাইম অ্যানোটেশন

এবার আমরা পূর্বের ত্রুটিপূর্ণ `longest` ফাংশনটিকে সমাধান করতে পারি:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

### এই ফাংশন সিগনেচারের অর্থ কী?
১. `<'a>` একটি জেনেরিক লাইফটাইম প্যারামিটার ঘোষণা করে।
২. `x: &'a str` এবং `y: &'a str` নির্দেশ করে যে `x` এবং `y` উভয় রেফারেন্সই **অন্তত `'a` সময় পর্যন্ত** জীবিত থাকবে।
৩. `-> &'a str` নিশ্চয়তা দেয় যে ফাংশন থেকে ফেরত দেওয়া রেফারেন্সটিও **অন্তত `'a` সময় পর্যন্ত** বেঁচে থাকবে।

বাস্তবে কম্পাইলার `'a`-এর মান নির্ধারণ করবে `x` এবং `y`-এর লাইফটাইমের মধ্যে যেটি **তুলনামূলক ছোট বা সংকুচিত (Intersection)**, সেই পরিসর অনুযায়ী।

### MiniStore-এ ব্যবহারিক উদাহরণ: প্রোডাক্ট তুলনা
MiniStore-এ ধরুন আমাদের এমন একটি সহায়ক ফাংশন দরকার যা দুটি ধার করা প্রোডাক্ট রেফারেন্স গ্রহণ করে এবং যেটির ইউনিট প্রাইস বেশি, তার রেফারেন্স ফেরত দেয়:

```rust
pub fn find_higher_priced<'a>(p1: &'a Product, p2: &'a Product) -> &'a Product {
    if p1.price_cents >= p2.price_cents {
        p1
    } else {
        p2
    }
}
```

কম্পাইলার কীভাবে এটি প্রয়োগ করে দেখুন:
```rust
let keyboard = Product::new(101, ...); // বাইরের স্কোপে বেঁচে থাকে
let result;
{
    let mouse = Product::new(102, ...); // ভেতরের স্কোপে বেঁচে থাকে
    result = find_higher_priced(&keyboard, &mouse);
    println!("Higher: {}", result.name); // সম্পূর্ণ বৈধ: mouse এখনো বেঁচে আছে!
}
// println!("Higher: {}", result.name); // COMPILE ERROR: mouse ধ্বংস হয়ে গেছে!
```
বরো চেকার নিশ্চিত করে যে `result` ভেতরের স্কোপের বাইরে ব্যবহার করা যাবে না, কারণ এটি হয়তো `mouse`-এর দিকে তাকিয়ে ছিল যা স্কোপ শেষে বিলুপ্ত হয়ে গেছে!

---

## তিনটি লাইফটাইম এলিশন নিয়ম (Lifetime Elision Rules)

আপনার মনে প্রশ্ন জাগতে পারে: *আগের অধ্যায়গুলোতে শত শত মেথড ও ফাংশন লেখার সময় কেন আমাদের কখনো `'a` লিখতে হয়নি?*

রাস্টের প্রাক-১.০ সংস্করণে প্রতিটি রেফারেন্সে ডেভেলপারদের ম্যানুয়ালি লাইফটাইম লিখতে হতো। পরবর্তীতে রাস্ট টিম হাজার হাজার ফাংশন বিশ্লেষণ করে দেখে যে সাধারণ কোডে নির্দিষ্ট কিছু প্যাটার্ন বারবার ঘটে।

তাই কম্পাইলারের মধ্যে **তিনটি লাইফটাইম এলিশন নিয়ম** যুক্ত করা হয়, যা স্বয়ংক্রিয়ভাবে সাধারণ ফাংশনগুলোর লাইফটাইম অনুমান করে নেয়:

### নিয়ম ১: ইনপুটের প্রতিটি অনুক্ত লাইফটাইম একটি স্বতন্ত্র লাইফটাইম প্যারামিটার পায়
```rust
fn print(s: &str)               // রূপান্তরিত হয় -> fn print<'a>(s: &'a str)
fn pair(x: &str, y: &str)       // রূপান্তরিত হয় -> fn pair<'a, 'b>(x: &'a str, y: &'b str)
```

### নিয়ম ২: যদি ইনপুটে ঠিক একটিমাত্র লাইফটাইম থাকে, তবে সেই লাইফটাইমটি সমস্ত আউটপুটে বরাদ্দ হয়
```rust
fn first_word(s: &str) -> &str  // রূপান্তরিত হয় -> fn first_word<'a>(s: &'a str) -> &'a str
```
যেহেতু ইনপুটে একটাই রেফারেন্স আছে, ফেরত আসা রেফারেন্সটি অবশ্যই সেই ইনপুট থেকেই এসেছে!

### নিয়ম ৩: যদি ইনপুটে একাধিক প্যারামিটার থাকে, কিন্তু তাদের মধ্যে একটি `&self` বা `&mut self` হয়, তবে `self`-এর লাইফটাইম সমস্ত আউটপুটে বরাদ্দ হয়
```rust
impl Product {
    fn name(&self) -> &str      // রূপান্তরিত হয় -> fn name<'a>(&'a self) -> &'a str
}
```
এই ৩য় নিয়মের কারণেই স্ট্রাক্টের মেথডগুলোতে আমাদের কখনোই ম্যানুয়ালি লাইফটাইম লিখতে হয় না।

> [!NOTE]
> এই তিনটি নিয়ম প্রয়োগ করার পরেও যদি কোনো আউটপুট রেফারেন্সের উৎস দ্ব্যর্থহীনভাবে নিশ্চিত করা না যায় (যেমন আমাদের `find_higher_priced` ফাংশনে যেখানে দুটি ইনপুট ও একটি আউটপুট আছে), কেবল তখনই কম্পাইলার `error[E0106]` ছুঁড়ে দেয় এবং প্রোগ্রামারকে স্পষ্টভাবে `'a` উল্লেখ করতে বলে।

---

## রেফারেন্স ধারণকারী স্ট্রাক্ট: জিরো-কপি আর্কিটেকচার (Zero-Copy Architecture)

এখন পর্যন্ত MiniStore-এর সমস্ত মডেল (`Product`, `Customer`, `Order`) তাদের নিজস্ব ডেটা হিপে তৈরি `String` এবং `Vec` আকারে ধারণ (Own) করেছে।

লং-টার্ম ডেটার জন্য এটি স্বাভাবিক হলেও, হাই-পারফরম্যান্স সিস্টেমে (যেমন চেকআউটের সময় প্রতি সেকেন্ডে হাজার হাজার রিসিট প্রিন্ট করা) প্রতিটি রিসিটের জন্য কাস্টমারের নাম বা অর্ডারের বিবরণ ক্লোন করা চরম অপচয়।

ডাটা ক্লোন না করে **রেফারেন্স** ধারণ করার মাধ্যমে আমরা অর্জন করতে পারি **জিরো-কপি আর্কিটেকচার (Zero-Copy Architecture)**:

```rust
/// একটি জিরো-কপি রিসিট ভিউ যা কোনো ডেটা কপি না করে `Order`, কাস্টমারের নাম এবং ক্যাশিয়ার নোট ধার করে।
pub struct OrderReceipt<'a> {
    pub order: &'a Order,
    pub customer_name: &'a str,
    pub cashier_note: &'a str,
}
```

### স্ট্রাক্টে কেন `<'a>` দিতে হয়?
যদি আপনি `<'a>` বাদ দিয়ে লেখেন:
```rust
pub struct OrderReceipt {
    pub order: &Order, // COMPILE ERROR: missing lifetime specifier
}
```
রাস্ট এটি কম্পাইল করবে না, কারণ:
১. স্ট্রাক্টের ইনস্ট্যান্সটি মেমরির এক স্থানে থাকে।
২. স্ট্রাক্টের ভেতরের রেফারেন্সগুলো মেমরির *অন্য স্থানে* থাকা ডেটাকে নির্দেশ করে।
৩. লাইফটাইম প্যারামিটার `<'a>` কম্পাইলারকে নিশ্চিত করে: **একটি `OrderReceipt` কখনো তার পেছনের `Order` বা স্ট্রিংগুলোর চেয়ে বেশি সময় মেমরিতে টিকে থাকতে পারবে না!**

### লাইফটাইম স্ট্রাক্টে মেথড বাস্তবায়ন
লাইফটাইম যুক্ত স্ট্রাক্টে মেথড লিখতে `impl` কিওয়ার্ডের পরেও `'a` ঘোষণা করতে হয়:

```rust
impl<'a> OrderReceipt<'a> {
    pub fn new(order: &'a Order, customer_name: &'a str, cashier_note: &'a str) -> Self {
        Self {
            order,
            customer_name,
            cashier_note,
        }
    }

    /// লক্ষ্য করুন: &'a str রিটার্ন করায় এটি কাস্টমারের আসল স্ট্রিংয়ের সাথে সম্পর্কিত,
    /// সাময়িক `&self`-এর সাথে নয়!
    pub fn customer_name(&self) -> &'a str {
        self.customer_name
    }

    pub fn note(&self) -> &'a str {
        self.cashier_note
    }

    pub fn generate_slip(&self) -> String {
        format!(
            "=== RECEIPT: {} ===\nCustomer: {}\nItems: {}\nTotal: ${:.2}\nStatus: {}\nNote: {}\n===================",
            self.order.order_id,
            self.customer_name,
            self.order.items.len(),
            self.order.total_cents() as f64 / 100.0,
            self.order.status,
            self.cashier_note
        )
    }
}
```

---

## বিশেষ সংরক্ষিত `'static` লাইফটাইম

রাস্টে একটি বিশেষ সংরক্ষিত লাইফটাইমের নাম হলো **`'static`**।

একটি রেফারেন্সের লাইফটাইম `'static` হলে তার অর্থ হলো সেই রেফারেন্সটি **পুরো প্রোগ্রাম চলাকালীন সম্পূর্ণ সময়** মেমরিতে বেঁচে থাকে।

### ১. স্ট্রিং লিটারাল (String Literals)
সমস্ত হার্ডকোডেড স্ট্রিং লিটারাল (`"..."`) স্বভাবগতভাবেই `'static` লাইফটাইম ধারণ করে, কারণ তাদের কাঁচা বাইটগুলো সরাসরি কম্পাইল করা বাইনারির রিড-অনলি ডেটা সেগমেন্টে গেঁথে দেওয়া হয়:
```rust
pub fn store_policy() -> &'static str {
    "MiniStore Guarantee: 30-Day Hassle-Free Returns & 1-Year Warranty"
}
```
স্কোপ শেষ হয়ে ডাটা মুছে যাওয়ার কোনো ভয় ছাড়াই আপনি যেকোনো ফাংশন থেকে সরাসরি স্ট্রিং লিটারাল রিটার্ন করতে পারেন!

### ২. ট্রেইট বাউন্ড `T: 'static`
জেনেরিক কোড বা মাল্টি-থ্রেডিংয়ে প্রায়ই `T: 'static` বাউন্ড দেখা যায়। এর অর্থ এই নয় যে ভ্যালুটি চিরকাল বেঁচে থাকবে; এর অর্থ হলো টাইপ `T`-এর ভেতরে **এমন কোনো রেফারেন্স নেই যা স্বল্পস্থায়ী** (অর্থাৎ এটি সম্পূর্ণ ওনড ডেটা যেমন `String` বা `u32`, অথবা শুধুমাত্র `'static` রেফারেন্স ধারণ করে)।

---

## অন্যান্য ভাষার সাথে তুলনা

| মাত্রা | রাস্ট (Rust) | সি / সি++ (C / C++) | গো (Go) | জাভা / সি# (Java / C#) |
| :--- | :--- | :--- | :--- | :--- |
| **রেফারেন্স সুরক্ষা** | **কম্পাইল-টাইম নিশ্চয়তা** (বরো চেকার) | ম্যানুয়াল (ভুল হলে আনডিফাইন্ড আচরণ) | রানটাইম গার্বেজ কালেক্টর | রানটাইম গার্বেজ কালেক্টর |
| **ড্যাংলিং পয়েন্টার** | নিরাপদ কোডে **অসম্ভব** | অত্যন্ত সাধারণ গুরুতর বাগ | অসম্ভব (GC বাঁচিয়ে রাখে) | অসম্ভব (GC বাঁচিয়ে রাখে) |
| **রানটাইম খরচ** | **শূন্য রানটাইম ওভারহেড** (কম্পাইল টাইমে ইরেজড) | শূন্য রানটাইম খরচ | জিসি পজ ও মেমরি ওভারহেড | জিসি পজ, অবজেক্ট বক্সিং, ক্যাশ মিস |
| **সিনট্যাক্স লোড** | কেবল দ্ব্যর্থক সিগনেচারে স্পষ্ট `'a` | কোনো লাইফটাইম নেই (অনিরাপদ) | নেই (রানটাইম নিয়ন্ত্রিত) | নেই (রানটাইম নিয়ন্ত্রিত) |
| **জিরো-কপি ভিউ** | শতভাগ নিরাপদ ও ফার্স্ট-ক্লাস | দ্রুত কিন্তু বিপজ্জনক | স্লাইসের (`[]byte`) মাধ্যমে নিরাপদ | ByteBuffers / String views |

---

## MiniStore আর্কিটেকচার এবং কোড ইমপ্লিমেন্টেশন

MiniStore-এ লাইফটাইম কীভাবে যুক্ত করা হয়েছে তা দেখে নেওয়া যাক:

```
ministore/
└── src/
    ├── models/
    │   ├── receipt.rs      # OrderReceipt<'a>, find_higher_priced, best_contact_info, store_policy
    │   └── mod.rs          # রিসিট মডিউল এবং আইটেম রি-এক্সপোর্ট
    ├── lib.rs              # লাইফটাইম আইটেম রি-এক্সপোর্ট এবং ২৪টি ইউনিট টেস্ট
    └── main.rs             # জিরো-কপি রিসিট, স্লিপ তৈরি এবং লাইফটাইম ফাংশনের ডেমো
```

### ১. `src/models/receipt.rs`
```rust
use crate::models::{Order, OrderId, Product};
use crate::traits::Summarizable;

#[derive(Debug, Clone, PartialEq)]
pub struct OrderReceipt<'a> {
    pub order: &'a Order,
    pub customer_name: &'a str,
    pub cashier_note: &'a str,
}

impl<'a> OrderReceipt<'a> {
    pub fn new(order: &'a Order, customer_name: &'a str, cashier_note: &'a str) -> Self {
        Self {
            order,
            customer_name,
            cashier_note,
        }
    }

    pub fn customer_name(&self) -> &'a str {
        self.customer_name
    }

    pub fn note(&self) -> &'a str {
        self.cashier_note
    }

    pub fn order_id(&self) -> OrderId {
        self.order.order_id
    }

    pub fn generate_slip(&self) -> String {
        format!(
            "=== RECEIPT: {} ===\nCustomer: {}\nItems: {}\nTotal: ${:.2}\nStatus: {}\nNote: {}\n===================",
            self.order.order_id,
            self.customer_name,
            self.order.items.len(),
            self.order.total_cents() as f64 / 100.0,
            self.order.status,
            self.cashier_note
        )
    }
}

impl<'a> std::fmt::Display for OrderReceipt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Receipt for Order {} ({}) - Total: ${:.2}",
            self.order.order_id,
            self.customer_name,
            self.order.total_cents() as f64 / 100.0
        )
    }
}

impl<'a> Summarizable for OrderReceipt<'a> {
    fn summary(&self) -> String {
        format!(
            "Receipt: Order {} for {} | Total: ${:.2}",
            self.order.order_id,
            self.customer_name,
            self.order.total_cents() as f64 / 100.0
        )
    }
}

pub fn find_higher_priced<'a>(p1: &'a Product, p2: &'a Product) -> &'a Product {
    if p1.price_cents >= p2.price_cents {
        p1
    } else {
        p2
    }
}

pub fn best_contact_info<'a>(primary_phone: Option<&'a str>, fallback_email: &'a str) -> &'a str {
    match primary_phone {
        Some(phone) if !phone.trim().is_empty() => phone,
        _ => fallback_email,
    }
}

pub fn store_policy() -> &'static str {
    "MiniStore Guarantee: 30-Day Hassle-Free Returns & 1-Year Warranty"
}
```

---

## সাধারণ কম্পাইলার এরর এবং সমাধানের উপায়

### ১. `error[E0515]: cannot return reference to local variable`
**ভুল কোড**:
```rust
fn create_receipt_note() -> &str {
    let note = String::from("Customer paid in cash");
    &note // COMPILE ERROR: বর্তমান ফাংশনের লোকাল ভেরিয়েবলের রেফারেন্স ফেরত দেওয়া হচ্ছে
}
```
**কেন ঘটে**:
লোকাল ভেরিয়েবল `note` ফাংশন স্কোপ শেষ হওয়ার সাথে সাথে ধ্বংস হয়ে যায়। ফলে `&note` রিটার্ন করলে তা একটি মৃত মেমরির দিকে তাকিয়ে থাকবে!
**সমাধান**:
রেফারেন্সের বদলে **ওনড (Owned)** `String` রিটার্ন করুন, অথবা কলারের সরবরাহ করা রেফারেন্স ব্যবহার করুন:
```rust
fn create_receipt_note() -> String {
    String::from("Customer paid in cash")
}
```

---

### ২. `error[E0597]: borrowed value does not live long enough`
**ভুল কোড**:
```rust
let receipt;
{
    let local_order = Order::new(...);
    receipt = OrderReceipt::new(&local_order, "Alice", "Paid");
} // local_order এখানে ড্রপ হয়ে গেল!
println!("{}", receipt.order_id()); // COMPILE ERROR
```
**কেন ঘটে**:
`receipt`-এর ভেতরের রেফারেন্সটি `local_order`-এর সমান সময় বেঁচে থাকতে চায়। কিন্তু `local_order` ভেতরের স্কোপেই মারা গেছে।
**সমাধান**:
আসল ডেটাকে অন্তত স্ট্রাক্টটির সমান দীর্ঘায়ু দিন:
```rust
let local_order = Order::new(...);
let receipt = OrderReceipt::new(&local_order, "Alice", "Paid");
println!("{}", receipt.order_id());
```

---

## ইডিওম্যাটিক রাস্ট বেস্ট প্র্যাকটিস

১. **সম্ভব হলে লাইফটাইম এলিশনের ওপর ভরসা রাখুন**: কম্পাইলার যদি নিজে থেকেই এলিশন নিয়মের মাধ্যমে লাইফটাইম বুঝে নিতে পারে, তবে জোর করে `'a` লিখে কোড জটিল করবেন না।
২. **দীর্ঘজীবী ডোমেন মডেলের জন্য ওনড টাইপ ব্যবহার করুন**: যেসব স্ট্রাক্ট সিস্টেমে দীর্ঘ সময় টিকে থাকে (`Order` বা `Product`), সেগুলোতে ওনড টাইপ (`String`, `Vec`) ব্যবহার করাই শ্রেয়। জিরো-কপি রেফারেন্স স্ট্রাক্ট (`OrderReceipt<'a>`) মূলত স্বল্পস্থায়ী ভিউ, রিপোর্ট বা পার্সিংয়ের জন্য নিখুঁত।
৩. **লাইফটাইমের স্কোপ সংকীর্ণ রাখুন**: রেফারেন্স ধারণকারী স্ট্রাক্টকে গ্লোবাল স্টেট বা দীর্ঘমেয়াদী ক্যাশে আটকে রাখবেন না, এতে লাইফটাইম ম্যানেজমেন্ট জটিল হয়ে পড়ে।
৪. **একাধিক স্বাধীন লাইফটাইমের ক্ষেত্রে অর্থপূর্ণ নাম দিন**: কেবল একটি লাইফটাইম থাকলে `'a` মানসম্মত হলেও, স্ট্রাক্টে একাধিক স্বাধীন রেফারেন্স থাকলে `'order` এবং `'note`-এর মতো বর্ণনামূলক নাম দিলে কোডের উদ্দেশ্য পরিষ্কার হয়।

---

## বাস্তবমুখী অনুশীলন (Exercises)

### অনুশীলন ১: জিরো-কপি সার্চ ম্যাচ
১. `SearchMatch<'a>` নামের একটি স্ট্রাক্ট তৈরি করুন যা `product: &'a Product` এবং `matched_query: &'a str` ধার করে।
২. `format_result(&self) -> String` মেথড যোগ করুন যা `"[Matched 'query']: Product Name ($Price)"` আউটপুট দেয়।
৩. পরীক্ষা করে নিশ্চিত করুন যে একাধিক `SearchMatch` তৈরি করতে প্রোডাক্টের নামের জন্য কোনো নতুন হিপ অ্যালোকেশন হচ্ছে না।

### অনুশীলন ২: লাইফটাইম যুক্ত স্লাইসার ফাংশন
১. `first_matching_tag<'a>(tags: &'a [String], prefix: &str) -> Option<&'a str>` ফাংশনটি লিখুন।
২. এটি স্লাইসের ভেতর থেকে মিলে যাওয়া স্ট্রিংয়ের ধার করা `&str` স্লাইস ফেরত দেবে।
৩. যাচাই করুন যে ফেরত আসা স্লাইসটি আসল `Vec<String>`-এর সমান সময় পর্যন্ত বেঁচে থাকছে কি না।

---

## চেকপয়েন্ট (Checkpoint)

কম্পাইলার চেক চালিয়ে নিশ্চিত করুন যে ২৪টি টেস্টই সফলভাবে পাস করেছে:
```bash
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
cargo run
```

প্রত্যাশিত টেস্ট আউটপুট:
```text
running 24 tests
test tests::test_cart_item_option_lookup ... ok
test tests::test_coupon_validation_error ... ok
test tests::test_display_trait_implementations ... ok
test tests::test_customer_optional_phone ... ok
test tests::test_checkout_error_propagation_and_success ... ok
test tests::test_generic_api_response_wrapper ... ok
test tests::test_catalog_option_lookups ... ok
test tests::test_coupon_discount_and_take ... ok
test tests::test_generic_catalog_product_pagination ... ok
test tests::test_generic_page_map_transformation ... ok
test tests::test_generic_pagination_with_integers ... ok
test tests::test_generic_trait_bound_functions ... ok
test tests::test_lifetime_annotated_contact_resolution ... ok
test tests::test_lifetime_annotated_product_comparison ... ok
test tests::test_order_cancellation_prevention ... ok
test tests::test_order_status_valid_lifecycle ... ok
test tests::test_order_total_with_payment_fee ... ok
test tests::test_payment_method_fees_and_descriptions ... ok
test tests::test_product_category_tax_rates ... ok
test tests::test_product_stock_reduction_error ... ok
test tests::test_static_lifetime_policy ... ok
test tests::test_summarizable_trait_on_domain_models ... ok
test tests::test_taxable_trait_and_default_method ... ok
test tests::test_zero_copy_order_receipt_and_traits ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

প্রত্যাশিত সিএলআই আউটপুট:
```text
=== MiniStore: Lifetimes & Reference Safety (Part III) ===

1. Catalog initialized with 3 products.

2. Browsing Catalog with Generic Pagination (Page<Product>):
   Page 1 of 2 (Total Items: 3)
   - [TECH-KEY-001] Tenkeyless Mechanical Keyboard ($120.00)
   - [TECH-MOU-002] Ergonomic Wireless Mouse ($45.00)
   Has next page? true
   Transformed to Page<String>: ["Tenkeyless Mechanical Keyboard", "Ergonomic Wireless Mouse"]
   API Page 2 response: 1 product(s) returned out of 3 total.

3. Customer: Margaret Hamilton (+1-555-0199)

4. Lifetimes in Functions & Reference Safety:
   Higher priced item: Tenkeyless Mechanical Keyboard ($120.00)
   Best contact info: +1-555-0199
   Store Policy ('static): MiniStore Guarantee: 30-Day Hassle-Free Returns & 1-Year Warranty

5. Shared Behaviors via Traits:
   Tax Summary: Product #101: Tenkeyless Mechanical Keyboard [TECH-KEY-001] - $120.00 | Tax: $18.00 (15%)
   Tax Summary: Product #102: Ergonomic Wireless Mouse [TECH-MOU-002] - $45.00 | Tax: $6.75 (15%)
   Customer Summary: Customer #301: Margaret Hamilton <margaret@apollo.nasa.gov>

6. Processing checkout through modular services...
   Checkout Order #901 created successfully!
   Subtotal: $210.00 | Total: $148.50

7. Order Lifecycle Transitions:
   Order confirmed: Confirmed (Receipt: REC-901-HAMILTON)
   Order shipped:   Shipped (Tracking: TRK-FEDEX-77189)
   Cancellation prevented -> Cannot perform action 'cancel' while order is in 'Shipped (Tracking: TRK-FEDEX-77189)' state
   Final Lifecycle State: Delivered to Customer
   Order Summary: Order #901: 2 item(s), Total: $148.50 [Delivered to Customer]

8. Zero-Copy Receipt Borrowing Order & Slices (OrderReceipt<'a>):
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
- লাইফটাইম কেন বিদ্যমান: কোনো গার্বেজ কালেক্টরের ওভারহেড ছাড়াই কম্পাইল টাইমে রেফারেন্সের নিশ্চিত বৈধতা রক্ষা করা।
- জেনেরিক লাইফটাইম অ্যানোটেশন (`'a`) কীভাবে একাধিক রেফারেন্সের পারস্পরিক সম্পর্ক নির্ধারণ করে।
- কীভাবে তিনটি লাইফটাইম এলিশন রুল সাধারণ ফাংশনে বয়লারপ্লেট কোড কমায়।
- কীভাবে নিরাপদে রেফারেন্স ধারণকারী জিরো-কপি স্ট্রাক্ট (`OrderReceipt<'a>`) ডিজাইন করা যায়।
- স্ট্রিং লিটারাল এবং গ্লোবাল মেমরির জন্য `'static`-এর তাৎপর্য।

---

## পরবর্তী অধ্যায়
ওনারশিপ, জেনেরিকস, ট্রেইটস এবং লাইফটাইমস আয়ত্ত করার মাধ্যমে আমরা এখন আধুনিক রাস্টের সবচেয়ে কার্যকরী ও মার্জিত ফিচারের জন্য প্রস্তুত: **ইটারেটরস (Iterators)**!
**অধ্যায় ১৭: ইটারেটরস**-এ আমরা শিখব কীভাবে রাস্ট ফাংশনাল অ্যাডাপ্টার (`map`, `filter`, `fold`)-এর মাধ্যমে অলস মূল্যায়ন (Lazy Evaluation) ব্যবহার করে জিরো-কস্ট অ্যাবস্ট্রাকশনে কালেকশন প্রসেস করে!
