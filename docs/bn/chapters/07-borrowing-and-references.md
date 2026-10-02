# অধ্যায় ৭: বরোয়িং এবং রেফারেন্স (Borrowing and References)

## আপনি কী শিখবেন
- প্রতিটি অপারেশনে ওনারশিপ হস্তান্তর (Move) কেন অদক্ষ এবং ব্যবহারিক কোডিংয়ের জন্য ঝামেলাপূর্ণ।
- **ধার নেওয়া বা বরোয়িং (Borrowing)** ধারণা এবং কীভাবে এটি অ্যাক্সেসকে ওনারশিপ থেকে পৃথক করে।
- **শেয়ার্ড রেফারেন্স (`&T`)**: শুধুমাত্র পড়ার অনুমতি (Read-only), যার মাধ্যমে একসাথে একাধিক পাঠক ডাটা পড়তে পারে।
- **মিউটেবল রেফারেন্স (`&mut T`)**: একক পরিবর্তনকারী অধিকার (Exclusive Read-Write), যার মাধ্যমে মেমোরিতে সরাসরি ডাটা সংশোধন করা যায়।
- রাস্টের মেমোরি সুরক্ষার সোনালী নীতি: **অ্যালিয়াসিং বনাম পরিবর্তনশীলতা (Aliasing XOR Mutability)**—একাধিক পাঠক অথবা কেবল একজন লেখক, কখনোই দুটি একসাথে নয়।
- কম্পাইলার কীভাবে কোনো গার্বেজ কালেক্টর বা রানটাইম ওভারহেড ছাড়াই **ড্যাংলিং রেফারেন্স (Dangling References)** সম্পূর্ণ প্রতিরোধ করে।
- **নন-লেক্সিক্যাল লাইফটাইম (Non-Lexical Lifetimes - NLL)** কীভাবে রেফারেন্সের মেয়াদ নির্ধারণ করে।
- মেথড রিসিভারের তুলনামূলক বিশ্লেষণ: `self` (ওনারশিপ গ্রহণ) বনাম `&self` (মান পরিদর্শন) বনাম `&mut self` (সরাসরি পরিবর্তন)।
- MiniStore অ্যাপ্লিকেশনে বরোয়িংয়ের বাস্তব প্রয়োগ: অপ্রয়োজনীয় ক্লোন বা মুভ ছাড়াই অর্ডারের প্রিভিউ প্রদর্শন, ইনভেন্টরি স্টক বৃদ্ধি এবং গ্রাহক প্রোফাইল আপডেট।

---

## আমাদের কেন এটি প্রয়োজন?
[অধ্যায় ৬: ওনারশিপে](/bn/chapters/06-ownership) আমরা দেখেছি রাস্টের মেমোরি সুরক্ষার মূল চাবিকাঠি হলো **মুভ সেমান্টিকস (Move Semantics)**। একটি ভ্যারিয়েবল ফাংশনে বাই-ভ্যালু (by value) পাস করলে তার ওনারশিপ সেই ফাংশনে চলে যায় এবং আগের স্কোপে ভ্যারিয়েবলটি পুরোপুরি অচল হয়ে যায়:

```rust
fn print_product(product: Product) {
    println!("Product: {}", product.name);
} // এখানে `product` স্কোপের বাইরে যায় এবং হিপ থেকে মুছে যায়!

let keyboard = Product::new(1, String::from("Keychron K2"), 8500, 10);
print_product(keyboard);
// println!("{}", keyboard.name); // কম্পাইলার এরর! borrow of moved value: `keyboard`
```

এখন প্রশ্ন হলো: প্রিন্ট করার পরেও যদি আমরা `keyboard` অবজেক্টটি ব্যবহার করতে চাই, তবে শুধুমাত্র ওনারশিপ দিয়ে কীভাবে সমাধান করা যেত?

১. **অতিরিক্ত ক্লোনিং (`.clone()`)**:
   ```rust
   print_product(keyboard.clone()); // শুধুমাত্র নাম পড়ার জন্য নতুন হিপ মেমোরি তৈরি!
   ```
   *সমস্যা*: শুধুমাত্র ডাটা দেখার জন্য সম্পূর্ণ হিপ মেমোরি কপি করা র‍্যামের অপচয় এবং পারফরম্যান্সের অপমৃত্যু।

২. **টাপল দিয়ে ওনারশিপ ফিরিয়ে আনা**:
   ```rust
   fn print_product(product: Product) -> Product {
       println!("Product: {}", product.name);
       product
   }
   let keyboard = print_product(keyboard); // ওনারশিপ বারবার আদান-প্রদান করা
   ```
   *সমস্যা*: জটিল, বিশৃঙ্খল এবং বড় অ্যাপ্লিকেশনে যেখানে ১০টি ভিন্ন জায়গায় ডাটা পড়তে হয়, সেখানে এটি কার্যকর করা অসম্ভব।

**রাস্টের চমৎকার সমাধান**: **রেফারেন্স এবং বরোয়িং (Borrowing & References)**। ডাটার ওনারশিপ হস্তান্তর না করে, আপনি একটি *রেফারেন্স* (`&`) তৈরি করে সাময়িকভাবে কাউকে ডাটা ব্যবহারের অধিকার "ধার" (Borrow) দিতে পারেন। ফাংশনের কাজ শেষ হলে ধার নেওয়ার মেয়াদ শেষ হয়ে যায়, আর মূল মালিক ভ্যারিয়েবল আগের মতোই ডাটার ওপর পূর্ণ নিয়ন্ত্রণ বজায় রাখে।

---

## সমস্যাটি কী?
MiniStore-এর মতো একটি ই-কমার্স অ্যাপ্লিকেশনের স্টক ও শপিং কার্টের কথা ভাবুন:

```text
দোকানের ক্যাটালগে থাকা প্রোডাক্ট ──► স্টোরফ্রন্টে প্রদর্শন (Read)
                               ──► কার্টের মূল্য গণনা (Read)
                               ──► ইনভেন্টরি রিস্টক করা (Write in place)
                               ──► চেকআউটে কনফার্ম করা (Consume)
```

অন্যান্য প্রচলিত ভাষায় কী ঘটে?
- **C / C++**: পয়েন্টার দিয়ে সরাসরি মান পরিবর্তন ও শেয়ার করা যায়, কিন্তু কম্পাইল টাইমে কোনো সুরক্ষা নেই। ভুলবশত অবজেক্ট ডিলিট হয়ে যাওয়ার পরও পয়েন্টার সক্রিয় থাকলে তৈরি হয় **ড্যাংলিং পয়েন্টার** যা সফটওয়্যার ক্র্যাশ করায়।
- **Java / Python / Go**: প্রতিটি অবজেক্টই একটি শেয়ার্ড রেফারেন্স। যেকোনো মেথড বা থ্রেড অজান্তেই আপনার ডাটা পরিবর্তন করে দিতে পারে। ফলে তৈরি হয় রেস কন্ডিশন ও **ইটারেটর ইনভ্যালিডেশন**।

MiniStore-এর যা প্রয়োজন:
১. প্রোডাক্ট বা কাস্টমার রেকর্ড নষ্ট বা ক্লোন না করে অর্ডারের প্রিভিউ দেখানো এবং ডিসকাউন্ট গণনা করা।
২. নিরাপদে একই মেমোরি ব্লকে সরাসরি স্টক আপডেট এবং কাস্টমার প্রোফাইল সংশোধন করা।
৩. গ্যারান্টি থাকা যে কেউ যখন রশিদ তৈরি করতে ডাটা পড়ছে, ঠিক তখনই অন্য কেউ স্টকের পরিমাণ পরিবর্তন করতে পারবে না।

---

## রাস্টের সমাধান (Rust Concept)

### ১. রেফারেন্স এবং ধার নেওয়া (Borrowing)
একটি রেফারেন্স মূলত একটি সুরক্ষিত পয়েন্টারের মতো: এটি মেমোরিতে মূল ডাটা কোথায় সংরক্ষিত আছে তার ঠিকানা ধারণ করে। সি/সি++ এর অনিরাপদ পয়েন্টারের মতো নয়, রাস্টের রেফারেন্স কম্পাইল টাইমে **সর্বদা কার্যকর এবং নির্দিষ্ট টাইপের ডাটাকে নির্দেশ করার নিশ্চয়তা দেয়**। রেফারেন্স তৈরি করার এই কাজটিকে বলা হয় **বরোয়িং বা ধার নেওয়া**।

```text
ভ্যারিয়েবল `product` (স্ট্যাকে থাকা মূল মালিক):
┌──────────────┬──────────────────┬─────────────┬───────┐
│ id: 501      │ name: (ptr,len)  │ price: 8500 │ stock │
└──────────────┴─────────┬────────┴─────────────┴───────┘
                         │
রেফারেন্স `&product`     ▼
┌────────────────────────┐
│ Pointer to `product`   │
└────────────────────────┘
```

রেফারেন্স যখন তার স্কোপের বাইরে চলে যায়, তখন কিন্তু মূল ডাটা মুছে যায় না; কারণ রেফারেন্স ডাটার মালিক নয়, সে কেবল সাময়িক ব্যবহারকারী।

### ২. শেয়ার্ড রেফারেন্স (`&T` - Shared References)
একটি সাধারণ রেফারেন্স (`&T`) শুধুমাত্র **পড়ার অধিকার (Read-only)** দেয়। একই সময়ে আপনি যত ইচ্ছা ততগুলো শেয়ার্ড রেফারেন্স তৈরি করতে পারেন।

```rust
let p = Product::new(1, String::from("Mouse"), 2500, 10);
let r1 = &p;
let r2 = &p;
let r3 = &p;
println!("Name: {}, Price: {}", r1.name, r2.price_cents); // সম্পূর্ণ নিরাপদ!
```

যেহেতু সকলেই কেবল রিডার (পাঠক), কেউই ডাটা পরিবর্তন করতে পারছে না, তাই কোনো পাঠকের পাঠেই ডাটা বিকৃত হওয়ার সম্ভাবনা থাকে না।

### ৩. মিউটেবল রেফারেন্স (`&mut T` - Mutable References)
একটি মিউটেবল রেফারেন্স (`&mut T`) **একক পরিবর্তনকারী অধিকার (Exclusive Read-Write)** দেয়। এর মাধ্যমে ওনারশিপ মুভ না করেই মেমোরির মূল মান পরিবর্তন করা সম্ভব হয়।

মিউটেবল রেফারেন্স তৈরির শর্ত:
১. মূল ভ্যারিয়েবলটি অবশ্যই `mut` দিয়ে ঘোষিত হতে হবে।
২. রেফারেন্স নেওয়ার সময় `&mut` ব্যবহার করতে হবে।

```rust
let mut p = Product::new(1, String::from("Mouse"), 2500, 10);
let p_mut = &mut p;
p_mut.stock += 5; // রেফারেন্সের মাধ্যমে মূল `p` পরিবর্তন হলো
```

### ৪. রাস্টের সোনালী নীতি: অ্যালিয়াসিং বনাম পরিবর্তনশীলতা (Aliasing XOR Mutability)
এটি রাস্টের সবচেয়ে গুরুত্বপূর্ণ নিয়ম:

> **যেকোনো মুহূর্তে আপনি কেবল যেকোনো একটি পেতে পারেন:**
> - **যত ইচ্ছা ততগুলো ইমিউটেবল রেফারেন্স (`&T`), অথবা**
> - **ঠিক একটি মাত্র মিউটেবল রেফারেন্স (`&mut T`)।**
> 
> **একই সাথে দুটি কখনোই সম্ভব নয়।**

```text
   ┌────────────────────────────────────────────────────────┐
   │             রাস্টের বরো চেকার ম্যাট্রিক্স               │
   ├──────────────────────────┬─────────────────────────────┤
   │ একাধিক শেয়ার্ড (&T)?     │ অনুমোদিত (বহু পাঠক)        │
   │ একটি মিউটেবল (&mut T)?   │ অনুমোদিত (একক লেখক)         │
   │ শেয়ার্ড + মিউটেবল একত্রে? │ নিষিদ্ধ (কম্পাইলার এরর!)   │
   │ একাধিক মিউটেবল একত্রে?   │ নিষিদ্ধ (কম্পাইলার এরর!)   │
   └──────────────────────────┴─────────────────────────────┘
```

কেন রাস্ট এটি নিষিদ্ধ করে?
কেউ যখন ডাটা পড়ছে, ঠিক তখনই অন্য কেউ যদি তা বদলাতে শুরু করে, তবে রিডাররা আংশিক পরিবর্তিত বা ভুল ডাটা দেখতে পাবে। থ্রেডিংয়ের ক্ষেত্রে এটি ডেটা রেস ঘটায়। কম্পাইল টাইমে **Aliasing XOR Mutability** বাধ্য করার মাধ্যমে রাস্ট রানটাইম ক্র্যাশ ও রেস কন্ডিশন সমূলে উৎপাটন করে।

### ৫. নন-লেক্সিক্যাল লাইফটাইম (Non-Lexical Lifetimes - NLL)
আগের ভার্সনের রাস্টে একটি রেফারেন্স কার্লি ব্র্যাকেট `}` শেষ না হওয়া পর্যন্ত সক্রিয় থাকত। আধুনিক রাস্টে কম্পাইলার **Non-Lexical Lifetimes (NLL)** ব্যবহার করে:
একটি রেফারেন্সের মেয়াদ ঠিক সেখানেই শেষ হয়ে যায়, **যেখানে কোডে শেষবারের মতো এটিকে ব্যবহার করা হয়েছে**।

```rust
let mut product = Product::new(1, String::from("Desk"), 15000, 2);

let r1 = &product;
println!("Reading product: {}", r1.name); // `r1`-এর ব্যবহার এখানেই শেষ!
// -> ইমিউটেবল ধারের মেয়াদ ঠিক এখানেই শেষ হয়ে গেল

let r2 = &mut product; // সম্পূর্ণ বৈধ! কারণ `r1` আর সক্রিয় নেই।
r2.stock += 1;
```

---

## অন্যান্য ভাষার সাথে তুলনা

| বিষয় | C / C++ | Java / Go / Python / C# | Rust |
| :--- | :--- | :--- | :--- |
| **ফাংশনে ডাটা পাস** | ভ্যালু বা পয়েন্টার (`*p`)। কম্পাইলার সুরক্ষার ট্র্যাক রাখে না। | ডিফল্টভাবে রেফারেন্স। ডাটা যেকেউ যেকোনো জায়গা থেকে বদলাতে পারে। | বাই ভ্যালু (`T`), শেয়ার্ড রেফারেন্স (`&T`), অথবা মিউটেবল রেফারেন্স (`&mut T`)। |
| **ড্যাংলিং পয়েন্টার** | নিত্যদিনের বাগ। মেমোরি মোছার পরও পয়েন্টার এক্সেস করে ক্র্যাশ। | অসম্ভব (কারণ মেমোরি কখনো ডিলিট না করে গার্বেজ কালেক্টরের হাতে থাকে)। | অসম্ভব (কম্পাইলার কম্পাইল টাইমে প্রমাণ করে যে রেফারেন্সের মেয়াদ মূল ডাটার চেয়ে দীর্ঘ নয়)। |
| **ডেটা রেস (Data Race)** | রানটাইম বিপর্যয়। ম্যানুয়াল লক দিতে হয়। | রানটাইম বাগ। রেস ডিটেক্টর দিয়ে খুঁজতে হয়। | বরো চেকার দ্বারা কম্পাইল টাইমেই প্রতিরোধিত। |
| **অ্যালিয়াসিং ও পরিবর্তন** | যেকোনো জায়গায় অনুমতিপ্রাপ্ত। ইটারেটর লুপের ভেতর আইটেম মুছলে ক্র্যাশ করে। | অনুমতিপ্রাপ্ত। রানটাইমে `ConcurrentModificationException` দেয়। | কম্পাইল টাইমে **Aliasing XOR Mutability** দ্বারা সম্পূর্ণ নিষিদ্ধ। |
| **মেমোরি রিলিজ** | ম্যানুয়াল (`delete`, `free`)। | ব্যাকগ্রাউন্ড গার্বেজ কালেক্টর (GC)। | ওনার স্কোপ ছাড়ার সাথে সাথে স্বয়ংক্রিয় RAII। |

---

## ছোট উদাহরণ (Small Example)
এখানে `&T`, `&mut T` এবং বরো চেকার কীভাবে কাজ করে তার একটি বাস্তব উদাহরণ দেওয়া হলো:

```rust
fn main() {
    let mut greeting = String::from("Hello");

    // ১. একাধিক শেয়ার্ড রেফারেন্স
    let r1 = &greeting;
    let r2 = &greeting;
    println!("{r1} and {r2}"); // উভয়ই একসাথে শান্তিতে পড়ছে

    // ২. মিউটেবল রেফারেন্স (r1 এবং r2 এর কাজ শেষ হওয়ার পর)
    let r_mut = &mut greeting;
    r_mut.push_str(", Rust!");
    println!("{r_mut}"); // greeting এখন পরিবর্তিত হয়ে "Hello, Rust!"

    // ৩. শেয়ার্ড ও মিউটেবল একত্রে মেলানোর চেষ্টা:
    // let ref_read = &greeting;
    // let ref_write = &mut greeting; // কম্পাইলার আটকে দেবে!
    // println!("{ref_read}");
}
```

---

## MiniStore-এ বাস্তব প্রয়োগ
MiniStore-এ আমরা কীভাবে বরোয়িং কাজে লাগাচ্ছি:
১. **অর্ডার প্রিভিউ (Order Preview)**: কেনার আগে কাস্টমার কার্টের মোট খরচ এবং তার ভিআইপি ডিসকাউন্ট দেখতে চায়। আমরা `calculate_line_total`, `calculate_discount` এবং `print_order_preview` ফাংশনে `&Product` এবং `&Customer` পাঠাই। ফলে প্রোডাক্ট বা কাস্টমার অবজেক্ট ধ্বংস বা মুভ হয় না, তারা আগের মতোই কার্যকর থাকে।
২. **ইনভেন্টরি রিস্টক ও প্রোফাইল আপডেট**: যখন গুদামে নতুন পণ্য আসে বা কাস্টমার ভিআইপি মর্যাদা পায়, আমরা `&mut self` দিয়ে মেথড কল করি (`restock` এবং `upgrade_to_vip`)। কোনো মেমোরি পুনঃবরাদ্দ ছাড়াই ইন-প্লেস মান পরিবর্তিত হয়।
৩. **অর্ডার সম্পন্নকরণ (Finalize Order)**: যখন গ্রাহক চেকআউট চূড়ান্ত করেন, তখন আমরা `finalize_order(order: PendingOrder)` ফাংশনে পুরো ওনারশিপ মুভ করে দিই—যাতে অর্ডারটি আর কখনো পুনরায় প্রসেস না করা যায়।

---

## কোড (Code)
নিচে `ministore/src/main.rs`-এর সম্পূর্ণ ও কম্পাইলযোগ্য কোড দেওয়া হলো:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub name: String,
    pub price_cents: u32,
    pub stock: u32,
}

impl Product {
    pub fn new(id: u64, name: String, price_cents: u32, stock: u32) -> Self {
        Self {
            id,
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
// Borrowing Demonstration Functions: Shared (`&T`) & Mutable (`&mut T`)
// ============================================================================

/// Borrows `&Product` immutably: calculates subtotal without taking ownership.
/// The caller retains full ownership of `product`!
pub fn calculate_line_total(product: &Product, quantity: u32) -> u32 {
    product.price_cents * quantity
}

/// Borrows `&Customer` immutably: calculates discount based on VIP status.
/// The caller retains full ownership of `customer`!
pub fn calculate_discount(customer: &Customer, subtotal_cents: u32) -> u32 {
    if customer.is_vip {
        (subtotal_cents * 10) / 100 // 10% discount for VIPs
    } else {
        0
    }
}

/// Borrows both `&Customer` and `&Product` immutably to render a preview.
/// Neither value is moved or cloned.
pub fn print_order_preview(customer: &Customer, product: &Product, quantity: u32) {
    let subtotal = calculate_line_total(product, quantity);
    let discount = calculate_discount(customer, subtotal);
    let final_total = subtotal - discount;

    println!("--- Order Preview (Borrowed Read-Only) ---");
    println!("Customer: {}", customer.display_badge());
    println!(
        "Item:     {} x {} @ {}",
        product.name,
        quantity,
        product.formatted_price()
    );
    println!("Subtotal: ${:.2}", subtotal as f64 / 100.0);
    println!("Discount: ${:.2}", discount as f64 / 100.0);
    println!("Estimate: ${:.2}", final_total as f64 / 100.0);
}

/// Consumes ownership of `PendingOrder` by value (Move Semantics).
/// Contrast this with the borrowed functions above: once passed here, `order` cannot be used again!
pub fn finalize_order(order: PendingOrder) -> ConfirmedReceipt {
    let subtotal = calculate_line_total(&order.product, order.quantity);
    let discount = calculate_discount(&order.customer, subtotal);
    let total_cents = subtotal - discount;

    ConfirmedReceipt {
        receipt_id: format!("REC-{}-{}", order.order_id.0, order.customer.id),
        order_id: order.order_id,
        customer_name: order.customer.name, // Ownership of heap String moves to receipt
        product_name: order.product.name,   // Ownership of heap String moves to receipt
        quantity: order.quantity,
        total_cents,
    }
}

fn main() {
    println!("=== MiniStore: Borrowing, References & Aliasing XOR Mutability ===\n");

    // 1. Shared References (&T): Multiple Readers Without Moving Ownership
    let customer = Customer::new(
        101,
        String::from("Ada Lovelace"),
        String::from("ada@analytical.org"),
        false,
    );

    let mut product = Product::new(501, String::from("Mechanical Keyboard"), 12000, 15);

    println!("1. Shared References (&T) in Action:");
    // Both references borrow simultaneously and peacefully:
    let ref1 = &product;
    let ref2 = &product;
    println!(
        "   Simultaneous shared borrows: ref1: {}, ref2 price: {}",
        ref1.name,
        ref2.formatted_price()
    );

    // Call functions borrowing `&customer` and `&product`:
    print_order_preview(&customer, &product, 2);

    // Notice: `customer` and `product` were NOT moved! We can continue using them:
    println!(
        "\n   Original product still valid after preview: {} (Stock: {})",
        product.name, product.stock
    );

    // 2. Mutable References (&mut T): Exclusive In-Place Modification
    println!("\n2. Mutable References (&mut T) in Action:");
    // Borrow mutably to restock:
    product.restock(5);
    println!("   Restocked product: new stock = {}", product.stock);

    // Update price through a mutable borrow:
    let product_mut_ref = &mut product;
    product_mut_ref.update_price(11500); // On sale for $115.00!
    println!(
        "   Price updated via mutable reference: {}",
        product_mut_ref.formatted_price()
    );

    // 3. Non-Lexical Lifetimes (NLL) and The Golden Rule (Aliasing XOR Mutability):
    // Once `product_mut_ref` is no longer used, we can take a shared borrow again:
    let shared_after_mutation = &product;
    println!(
        "   New shared borrow after mutation completed: {} @ {}",
        shared_after_mutation.name,
        shared_after_mutation.formatted_price()
    );

    // 4. Modifying Customer via Mutable Reference:
    let mut customer_mutable = customer;
    println!("\n3. Mutating Customer Profile In-Place:");
    println!("   Before upgrade: {}", customer_mutable.display_badge());
    customer_mutable.upgrade_to_vip();
    customer_mutable.update_email(String::from("ada.lovelace@computing.org"));
    println!(
        "   After upgrade:  {} ({})",
        customer_mutable.display_badge(),
        customer_mutable.email
    );

    // 5. Finalizing an Order (Contrasting Borrowing with Moving Ownership):
    println!("\n4. Finalizing Order (Moving Ownership):");
    let pending_order = PendingOrder {
        order_id: OrderId(7001),
        customer: customer_mutable,
        product,
        quantity: 2,
    };

    // `finalize_order` takes ownership of `pending_order`:
    let receipt = finalize_order(pending_order);
    println!("   Receipt issued: {}", receipt.receipt_id);
    println!("   Customer:       {}", receipt.customer_name);
    println!(
        "   Product:        {} x {}",
        receipt.product_name, receipt.quantity
    );
    println!(
        "   Total Paid:     ${:.2}",
        receipt.total_cents as f64 / 100.0
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_borrowing_allows_multiple_readers() {
        let product = Product::new(1, String::from("USB-C Cable"), 1500, 50);

        let ref_a = &product;
        let ref_b = &product;

        // Multiple shared borrows can read simultaneously
        assert_eq!(ref_a.id, ref_b.id);
        assert_eq!(ref_a.price_cents, 1500);
        assert_eq!(ref_b.formatted_price(), "$15.00");
        assert!(ref_a.is_in_stock());
    }

    #[test]
    fn test_preview_does_not_consume_values() {
        let customer = Customer::new(1, String::from("Bob"), String::from("bob@test.com"), true);
        let product = Product::new(2, String::from("Mousepad"), 2000, 10);

        // Pass by shared reference:
        let total = calculate_line_total(&product, 3);
        let discount = calculate_discount(&customer, total);

        assert_eq!(total, 6000);
        assert_eq!(discount, 600); // 10% VIP discount on 6000

        // Customer and Product are NOT moved, still fully accessible!
        assert_eq!(customer.name, "Bob");
        assert_eq!(product.name, "Mousepad");
        assert_eq!(product.stock, 10);
    }

    #[test]
    fn test_mutable_borrowing_updates_in_place() {
        let mut product = Product::new(3, String::from("Ergo Chair"), 35000, 5);

        // Mutate via method borrowing &mut self
        product.restock(10);
        assert_eq!(product.stock, 15);

        product.reduce_stock(3).expect("Reduction should succeed");
        assert_eq!(product.stock, 12);

        // Mutate price via explicit &mut borrow
        let ref_mut = &mut product;
        ref_mut.update_price(32000);
        assert_eq!(ref_mut.price_cents, 32000);

        // Original variable reflects the in-place mutations
        assert_eq!(product.stock, 12);
        assert_eq!(product.price_cents, 32000);
    }

    #[test]
    fn test_customer_mutations_via_ref() {
        let mut customer = Customer::new(
            10,
            String::from("Carol"),
            String::from("carol@old.com"),
            false,
        );

        assert_eq!(customer.display_badge(), "[Standard Member] Carol");
        assert!(!customer.is_vip);

        customer.upgrade_to_vip();
        customer.update_email(String::from("carol@new.com"));

        assert_eq!(customer.display_badge(), "[VIP Member] Carol");
        assert!(customer.is_vip);
        assert_eq!(customer.email, "carol@new.com");
    }

    #[test]
    fn test_finalize_order_consumes_and_calculates_vip_discount() {
        let customer = Customer::new(1, String::from("Alice"), String::from("a@test.com"), true);
        let product = Product::new(10, String::from("Desk Pad"), 2000, 5);
        let order = PendingOrder {
            order_id: OrderId(77),
            customer,
            product,
            quantity: 2, // 2 * $20.00 = $40.00 ($4000 cents)
        };

        // VIP discount: 10% off $4000 = $400 -> $3600 cents ($36.00)
        let receipt = finalize_order(order);
        assert_eq!(receipt.order_id, OrderId(77));
        assert_eq!(receipt.customer_name, "Alice");
        assert_eq!(receipt.product_name, "Desk Pad");
        assert_eq!(receipt.quantity, 2);
        assert_eq!(receipt.total_cents, 3600);
    }
}
```

---

## কোড বিশ্লেষণ (Understanding The Code)

### ১. মেথড রিসিভারের তিন রূপ: `self` বনাম `&self` বনাম `&mut self`
`Product` স্ট্রাক্টের মেথডগুলো লক্ষ্য করুন:

```rust
impl Product {
    // ১. &self: প্রোডাক্টটি কেবল পড়ার জন্য ধার নেয়
    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    // ২. &mut self: প্রোডাক্টটি সরাসরি পরিবর্তন করার জন্য একক অধিকার নিয়ে ধার নেয়
    pub fn restock(&mut self, additional_units: u32) {
        self.stock += additional_units;
    }
}
```
- **`&self`**: ইনস্ট্যান্সটিকে ইমিউটেবলভাবে ধার নেয়। মেথডটি যেকোনো ফিল্ড পড়তে পারে কিন্তু পরিবর্তন করতে পারে না। কলারের ওনারশিপ অক্ষুণ্ণ থাকে।
- **`&mut self`**: ইনস্ট্যান্সটিকে মিউটেবলভাবে ধার নেয়। মেথডটির একক কর্তৃত্ব থাকে এবং সরাসরি মান পরিবর্তন করতে পারে। মেথডের কাজ শেষ হলে কলারের কাছে ডাটা ফেরত আসে।
- **`self`**: মান হিসেবে সম্পূর্ণ ওনারশিপ গ্রহণ করে (যেমন `finalize_order`)। মেথডটি মানটিকে গ্রাস করে ফেলে, ফলে কলার আর এটিকে ব্যবহার করতে পারে না।

### ২. অটো-ডিরেফারেন্সিং (The Dot Operator)
লক্ষ্য করুন `calculate_line_total(product: &Product, quantity: u32)` ফাংশনে আমরা লিখেছি:
```rust
product.price_cents * quantity
```
যদিও `product` একটি রেফারেন্স `&Product`, তবুও আমাদের `(*product).price_cents` লেখার প্রয়োজন হয়নি। রাস্টের ডট অপারেটর `.` স্বয়ংক্রিয়ভাবে রেফারেন্স অনুসরণ করে মূল মানটি খুঁজে নেয় (যাকে বলা হয় **auto-dereferencing**)।

---

## সাধারণ ভুলসমূহ (Common Mistakes)

### ১. শেয়ার্ড রেফারেন্স দিয়ে মান পরিবর্তনের চেষ্টা করা
```rust
fn apply_sale(product: &Product) {
    product.price_cents = 1000; // কম্পাইলার এরর! cannot assign to `product.price_cents`
}
```
শেয়ার্ড রেফারেন্স `&T` সম্পূর্ণরূপে অপরিবর্তনীয় (Immutable)। পরিবর্তন করতে চাইলে প্যারামিটারে অবশ্যই `&mut Product` লিখতে হবে এবং কলার ভ্যারিয়েবলটিও `mut` হতে হবে।

### ২. শেয়ার্ড রেফারেন্স জীবিত থাকা অবস্থায় ডাটা পরিবর্তন করা
```rust
let mut product = Product::new(1, String::from("Pen"), 200, 10);
let r1 = &product; // ইমিউটেবল ধার শুরু হলো
product.restock(5); // কম্পাইলার এরর! cannot borrow `product` as mutable
println!("{}", r1.stock); // r1 এখনও পাঠরত অবস্থায় আছে!
```
যেহেতু শেষ লাইনে `r1` পড়া হচ্ছে, তাই `product.restock(5)` এর সাথে রিডারের সংঘর্ষ ঘটে। ডাটার বিশুদ্ধতা রক্ষায় কম্পাইলার সাথে সাথে কোড আটকে দেয়।

---

## কম্পাইলার এরর বিশ্লেষণ (Compiler Errors)

### Error E0502: ইমিউটেবল ধার চলাকালীন মিউটেবল ধার নেওয়ার চেষ্টা
মনে করুন আপনি লিখেছেন:

```rust
let mut product = Product::new(1, String::from("Webcam"), 4500, 5);
let name_ref = &product.name;
product.restock(10);
println!("Product: {name_ref}");
```

রাস্ট কম্পাইলার তীব্র আপত্তি জানিয়ে বলবে:

```text
error[E0502]: cannot borrow `product` as mutable because it is also borrowed as immutable
  --> src/main.rs:185:5
   |
184|     let name_ref = &product.name;
   |                    ------------- immutable borrow occurs here
185|     product.restock(10);
   |     ^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
186|     println!("Product: {name_ref}");
   |                        ---------- immutable borrow later used here
```

**কেন এটি গুরুত্বপূর্ণ**: সি++ এ কোনো অবজেক্টের একটি ফিল্ডের রেফারেন্স ধরে রেখে অবজেক্ট মডিফাই করলে মেমোরি রিলোকেশন ঘটে পয়েন্টার নষ্ট হয়ে যেতে পারে (ক্র্যাশ/সিকিউরিটি রিস্ক)। রাস্টে বরো চেকারের কারণে এই ভুল প্রোগ্রাম রান করার সুযোগই পায় না।

---

## অনুশীলনী (Practice)
১. **ট্যাক্স যোগ করা**: একটি ফাংশন লিখুন `add_tax(product: &mut Product, tax_percentage: u32)`, যা কোনো ওনারশিপ পরিবর্তন না করে ইন-প্লেস প্রোডাক্টের দাম বাড়াবে।
২. **স্টক খালি চেক করা**: একটি ফাংশন লিখুন `is_any_out_of_stock(p1: &Product, p2: &Product) -> bool`, যা দুটি শেয়ার্ড রেফারেন্স নেবে এবং যেকোনো একটির স্টক শূন্য হলে `true` ফিরিয়ে দেবে।
৩. **টেস্টে প্রমাণ করুন**: `src/main.rs`-এর টেস্ট মডিউলে টেস্ট যোগ করে নিশ্চিত করুন যে `add_tax` ওনারশিপ মুভ না করেই সঠিকভাবে কাজ করছে।
৪. `cargo test` রান করে আপনার সমাধান যাচাই করুন।

---

## চেকপয়েন্ট (Checkpoint)
- [x] ওনারশিপ হস্তান্তর (Move) এবং ধার নেওয়ার (Borrowing) পার্থক্য উপলব্ধি করেছেন।
- [x] শেয়ার্ড রেফারেন্স (`&T`) এবং একাধিক পাঠকের নিরাপদে পড়ার নিয়ম বুঝেছেন।
- [x] মিউটেবল রেফারেন্স (`&mut T`) এবং ইন-প্লেস ডাটা পরিবর্তনের নিয়ম আয়ত্ত করেছেন।
- [x] অ্যালিয়াসিং বনাম পরিবর্তনশীলতার (Aliasing XOR Mutability) সার্বজনীন নিয়ম বুঝেছেন।
- [x] নন-লেক্সিক্যাল লাইফটাইম (NLL) কীভাবে ধারের মেয়াদ শেষ করে তা জেনেছেন।
- [x] মেথডে `&self` এবং `&mut self` ব্যবহার করে MiniStore-এর পরিচ্ছন্ন এপিআই তৈরি করেছেন।

---

## আমরা কী শিখলাম
- বরোয়িং অপ্রয়োজনীয় ক্লোনিং ও মেমোরি রূপান্তর পরিহার করে অ্যাপ্লিকেশনকে চরম গতিশীল ও দক্ষ করে তোলে।
- বরো চেকার নিশ্চিত করে যে কেউ পড়ার সময় ডাটা পরিবর্তিত হতে পারবে না—ফলে রেস কন্ডিশন কম্পাইল টাইমে চিরতরে নির্বাসিত হয়।
- MiniStore এখন নিরাপদে পণ্যের প্রিভিউ দেখতে পারে, ইনভেন্টরি আপডেট করতে পারে এবং চেকআউটে চূড়ান্তভাবে অর্ডার কনসিউম করতে পারে।

---

## পরবর্তীতে কী আসছে
আমরা একক স্ট্রাক্ট ধার নেওয়া শিখলাম। কিন্তু বড় কালেকশন বা স্ট্রিংয়ের কোনো নির্দিষ্ট অংশকে কোনো কপি ছাড়াই কীভাবে ধার নেওয়া যায়? **অধ্যায় ৮: স্ট্রিং, স্লাইস এবং ওনারশিপের ব্যবহারিক প্রয়োগ (Strings, Slices and Ownership in Practice)**-এ আমরা দেখব স্ট্রিং স্লাইস (`&str`), অ্যারে স্লাইস (`&[T]`) এবং মেমোরির জিরো-কপি ভিউ।
