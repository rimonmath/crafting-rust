# অধ্যায় ১৩: মডিউল, প্যাকেজ এবং প্রজেক্ট স্ট্রাকচার

## আপনি যা শিখবেন
- **প্যাকেজ (Package)**, **ক্রেট (Crate)** (বাইনারি ক্রেট বনাম লাইব্রেরি ক্রেট) এবং **মডিউল (Module)**-এর মধ্যকার পার্থক্য।
- কীভাবে কার্গো একক বাইনারি, একক লাইব্রেরি অথবা **ডুয়াল-টার্গেট ক্রেট** (`src/lib.rs` + `src/main.rs`) আর্কিটেকচার ম্যানেজ করে।
- **`mod` কিওয়ার্ড**: ইনলাইন মডিউল ডিক্লেয়ারেশন, আলাদা ফাইলে মডিউল (`foo.rs`), এবং ডিরেক্টরিতে সাবমডিউল (`foo/mod.rs`) সাজানো।
- রাস্টের কঠোর প্রাইভেসি রুলস: **বাই ডিফল্ট প্রাইভেট (Private by default)**, **`pub`** কিওয়ার্ড, স্ট্রাক্টের ফিল্ড প্রাইভেসি এবং গ্র্যানুলার ভিজিবিলিটি যেমন **`pub(crate)`**।
- পাথ রেজোলিউশন (Path Resolution): অ্যাবসলিউট পাথ (**`crate::`**) এবং রিলেটিভ পাথ (**`self::`**, **`super::`**) ব্যবহার করে মডিউল ট্রি নেভিগেট করা।
- **`use`** কিওয়ার্ড দিয়ে আইটেম স্কোপে আনা, নেস্টেড পাথ, **`as`** দিয়ে নাম পরিবর্তন (Renaming) এবং গ্লোব ইমপোর্ট (`*`) ব্যবহারের নিয়ম।
- **`pub use` দিয়ে রি-এক্সপোর্ট (Re-exporting)**: পরিষ্কার এবং মার্জিত পাবলিক ফ্যাসাড (Public Facade Pattern) তৈরি করা, যাতে ব্যবহারকারীকে ভেতরের জটিল ফোল্ডার স্ট্রাকচার জানতে না হয়।
- MiniStore-কে মনোলিথিক ৭০০+ লাইনের `main.rs` ফাইল থেকে একটি অত্যন্ত প্রফেশনাল ও মডুলার আর্কিটেকচারে রূপান্তর করা:
  - `src/error.rs`: কেন্দ্রীয় ডোমেইন এরর ডেফিনিশন (`StoreError`)।
  - `src/models/`: ডোমেইন এন্টিটিসমূহ আলাদা সাবমডিউলে বিভক্ত (`product.rs`, `customer.rs`, `cart.rs`, `order.rs`)।
  - `src/catalog.rs`: প্রোডাক্ট ক্যাটালগ এবং স্টক কোয়েরি সার্ভিস।
  - `src/checkout.rs`: ট্রানজ্যাকশনাল অর্ডার প্লেসমেন্ট এবং ডিসকাউন্ট ক্যালকুলেশন লজিক।
  - `src/lib.rs`: লাইব্রেরি ক্রেট রুট, পাবলিক ফ্যাসাড রি-এক্সপোর্ট এবং ইন্টিগ্রেশন ইউনিট টেস্ট।
  - `src/main.rs`: এক্সিকিউটেবল সিএলআই বাইনারি, যা `ministore` লাইব্রেরিকে ক্লায়েন্ট হিসেবে ব্যবহার করে।

---

## কেন আমাদের এটি প্রয়োজন?
অধ্যায় ১২ পর্যন্ত আমাদের সম্পূর্ণ MiniStore অ্যাপ্লিকেশনটি একটিমাত্র ফাইল `src/main.rs`-এর ভেতরেই ছিল।

শেখার শুরুতে বা প্রোটোটাইপিংয়ের জন্য একটিমাত্র ফাইলে কোড রাখা সহজ মনে হলেও, বাস্তব সফটওয়্যার ইঞ্জিনিয়ারিংয়ে কখনোই একটিমাত্র ফাইলে পুরো কোডবেস রাখা সম্ভব নয়:
1. **কগনিটিভ ওভারলোড (Cognitive Overload)**: একটি ফাইল যখন ৮০০ বা ২,০০০ লাইনের বেশি হয়ে যায়, তখন বিভিন্ন স্ট্রাক্ট, মেথড এবং লজিক খুঁজে পাওয়া অত্যন্ত দুর্বিষহ হয়ে ওঠে।
2. **মার্জ কনফ্লিক্ট (Merge Conflicts)**: একটি ডেভেলপমেন্ট টিমে যখন একাধিক ইঞ্জিনিয়ার একই সাথে `main.rs`-এ কোড পুশ করবেন, তখন অনবরত গিট কনফ্লিক্ট তৈরি হবে।
3. **এনক্যাপসুলেশনের অভাব (Lack of Encapsulation)**: একই ফাইলে সব কোড থাকলে যেকোনো ফাংশন যেকোনো স্ট্রাক্টের প্রাইভেট ফিল্ড সরাসরি পরিবর্তন করে ফেলতে পারে। ডেটার সঠিকতা রক্ষার বাউন্ডারি ভেঙে যায়।
4. **পুনর্ব্যবহারযোগ্যতা এবং টেস্টেবিলিটি (Reusability & Testability)**: শুধুমাত্র বাইনারি ক্রেট (`src/main.rs`) এক্সটার্নাল ইন্টিগ্রেশন টেস্ট বা বাইরের অন্য প্রজেক্টে ইমপোর্ট করা যায় না। কেবল লাইব্রেরি ক্রেটই (`src/lib.rs`) অন্য জায়গায় ইমপোর্ট করা সম্ভব।

রাস্টের কম্পাইলার-এনফোর্সড মডিউল সিস্টেম কোডকে লজিক্যাল ইউনিটে ভাগ করতে, অ্যাক্সেস কন্ট্রোল নিখুঁত রাখতে এবং পরিচ্ছন্ন পাবলিক এপিআই (API) তৈরি করতে সহায়তা করে।

---

## রাস্ট মডিউল সিস্টেমের স্তরবিন্যাস (Hierarchy)
রাস্ট কীভাবে কোড বিন্যস্ত করে তা বোঝার জন্য নিচের ডায়াগ্রামটি লক্ষ্য করুন:

```
+-----------------------------------------------------------+
|                        Package                            |
|  (Cargo.toml: মেটাডাটা, ডিপেন্ডেন্সি এবং ক্রেট সংজ্ঞায়িত)   |
|                                                           |
|   +-----------------------+   +-----------------------+   |
|   |     Library Crate     |   |     Binary Crate      |   |
|   |      src/lib.rs       |   |      src/main.rs      |   |
|   |                       |   |                       |   |
|   |   +---------------+   |   |   use my_crate::*;    |   |
|   |   |    Modules    |   |   |                       |   |
|   |   |  models/      |   |   |   fn main() { ... }   |   |
|   |   |  checkout.rs  |   |   +-----------------------+   |
|   |   |  catalog.rs   |   |                               |
|   |   +---------------+   |                               |
|   +-----------------------+                               |
+-----------------------------------------------------------+
```

### ১. প্যাকেজ (Package)
একটি **প্যাকেজ** হলো কার্গোর (Cargo) একটি ফিচার যা এক বা একাধিক ক্রেট তৈরি, টেস্ট এবং শেয়ার করার ব্যবস্থা করে। প্যাকেজের রুটে একটি `Cargo.toml` ফাইল থাকে।
- একটি প্যাকেজে **অবশ্যই অন্তত একটি ক্রেট** থাকতে হবে (হতে পারে লাইব্রেরি অথবা বাইনারি)।
- একটি প্যাকেজে **সর্বোচ্চ একটি লাইব্রেরি ক্রেট** (`src/lib.rs`) থাকতে পারে।
- একটি প্যাকেজে **শূন্য, এক বা একাধিক বাইনারি ক্রেট** থাকতে পারে (`src/main.rs`, অথবা `src/bin/*.rs`)।

### ২. ক্রেট (Crate)
রাস্ট কম্পাইলার (`rustc`) একবারে যতটুকু কোড ইউনিট হিসেবে বিবেচনা করে, তাকে **ক্রেট** বলে।
ক্রেট দুই প্রকার:
- **বাইনারি ক্রেট (Binary Crate)**: এটি একটি এক্সিকিউটেবল প্রোগ্রাম যার একটি `fn main()` এন্ট্রি পয়েন্ট থাকে। কম্পাইল করলে এটি একটি রানযোগ্য মেশিন ফাইল তৈরি করে (যেমন: `target/debug/ministore`)।
- **লাইব্রেরি ক্রেট (Library Crate)**: এটি পুনরায় ব্যবহারযোগ্য কার্যক্ষমতার সংগ্রহ যেখানে কোনো `main()` ফাংশন থাকে না। এর মূল ফাইল হলো `src/lib.rs`।

> [!TIP]
> **ডুয়াল-টার্গেট প্যাটার্ন (The Dual-Crate Pattern)**: রাস্টে সিএলআই টুল বা মাইক্রোসার্ভিস তৈরির জন্য সবচেয়ে আইডিওম্যাটিক প্যাটার্ন হলো সমস্ত মূল বিজনেস লজিক লাইব্রেরি ক্রেটে (`src/lib.rs`) রাখা এবং `src/main.rs`-কে কেবল একটি পাতলা র‍্যাপার (Thin Wrapper) হিসেবে রাখা, যা শুধু ইনপুট নেয় ও লাইব্রেরির মেথড কল করে। এর ফলে সমস্ত বিজনেস লজিক সহজেই ইউনিট টেস্ট ও রিইউজ করা যায়!

### ৩. মডিউল (Module)
একটি ক্রেটের *ভেতরে* কোডের রিড্যাবিলিটি, নেমস্পেস আলাদা রাখা এবং প্রাইভেসি নিয়ন্ত্রণের জন্য **মডিউল** ব্যবহার করা হয়।
- মডিউল **প্রাইভেসি** নিশ্চিত করে: স্পষ্টভাবে `pub` না লিখলে মডিউলের ভেতরের যেকোনো আইটেম বাইরের কোডের জন্য লুকায়িত (প্রাইভেট) থাকে।
- প্রতিটি মডিউল মিলে একটি ট্রি (Tree) গঠন করে, যার গোড়া হলো ক্রেট রুট (`src/lib.rs` বা `src/main.rs`)।

---

## মূল রাস্ট কনসেপ্ট: মডিউল ডিক্লেয়ার ও অর্গানাইজ করা

জাভা বা নোড.জেএস (Node.js)-এর মতো ল্যাঙ্গুয়েজে ফোল্ডারে একটি ফাইল বানালেই সেটি নিজে থেকেই ইমপোর্টযোগ্য মডিউল হয়ে যায়। **কিন্তু রাস্টে ফাইল সিস্টেম নিজে থেকে মডিউল নির্ধারণ করে না।**

রাস্টে প্যারেন্ট ফাইলে **`mod` কিওয়ার্ড লিখে স্পষ্টভাবে মডিউল ঘোষণা করতে হয়**।

### কম্পাইলার কীভাবে মডিউল খুঁজে পায়?
আপনি যখন `src/lib.rs`-এ `mod models;` লিখবেন, কম্পাইলার নিচের যেকোনো একটি স্থানে কোডটি খুঁজবে:
1. ইনলাইন ব্লক:
   ```rust
   mod models {
       // ভেতরের আইটেমসমূহ সরাসরি এখানে
   }
   ```
2. একই নামের একটি আলাদা ফাইল:
   `src/models.rs`
3. একই নামের ডিরেক্টরির ভেতর `mod.rs` ফাইল:
   `src/models/mod.rs`

### মডিউল লেআউট স্টাইল (মডার্ন রাস্ট বনাম ২০১৫ স্টাইল)
রাস্ট ২০১৮ এডিশন থেকে দুটি স্টাইলই অনুমোদিত:

| স্টাইল | ডিরেক্টরি লেআউট | ব্যাখ্যা |
| :--- | :--- | :--- |
| **ডিরেক্টরি + `mod.rs`** | `src/models/mod.rs`<br>`src/models/product.rs` | সুস্পষ্ট ও পরিষ্কার; `mod.rs` ফাইলটি `models` সাবমডিউলের রুট হিসেবে কাজ করে। |
| **সিবলিং ফাইল + ফোল্ডার** | `src/models.rs`<br>`src/models/product.rs` | রাস্ট ২০১৮ থেকে সমর্থিত; কোড এডিটরে অনেকগুলো `mod.rs` ট্যাব খোলা থাকার ঝামেলা এড়ায়। |

আমাদের MiniStore-এ আমরা `src/models/mod.rs` স্টাইল ব্যবহার করেছি, কারণ এটি ডোমেইন মডেলগুলোকে একটি স্বয়ংসম্পূর্ণ সাবপ্যাকেজ হিসেবে চমৎকারভাবে উপস্থাপন করে।

---

## প্রাইভেসি এবং ভিজিবিলিটি: বাই ডিফল্ট প্রাইভেট (Private by Default)

রাস্টে **সবকিছু ডিফল্টভাবে সম্পূর্ণ প্রাইভেট**।
- ফাংশন, স্ট্রাক্ট, এনাম, ট্রেইট, কনস্ট্যান্ট ইত্যাদি কেবল সেই মডিউলে এবং তার ভেতরের চাইল্ড মডিউলগুলোতে দৃশ্যমান থাকে।
- প্যারেন্ট মডিউল তার চাইল্ড মডিউলের প্রাইভেট আইটেম **দেখতে পারে না**।
- চাইল্ড মডিউল তার পূর্বপুরুষ (Ancestor) মডিউলের যেকোনো প্রাইভেট আইটেম **দেখতে পারে**।

```rust
mod warehouse {
    // প্রাইভেট ফাংশন: কেবল warehouse-এর ভেতরের কোড এটি কল করতে পারে
    fn secret_code() -> u32 {
        42
    }

    // পাবলিক ফাংশন: warehouse-এর বাইরে থেকেও কল করা যাবে
    pub fn open_doors() {
        // একই স্কোপের প্রাইভেট আইটেম এক্সেস করা বৈধ:
        let code = secret_code();
        println!("Opening doors with code {}", code);
    }
}

fn main() {
    warehouse::open_doors(); // সঠিক: ফাংশনটি pub
    // warehouse::secret_code(); // কম্পাইলার এরর: `secret_code` প্রাইভেট!
}
```

### স্ট্রাক্ট বনাম এনাম ভিজিবিলিটির সূক্ষ্ম পার্থক্য
রাস্টে `struct` এবং `enum`-এর ভিজিবিলিটির মধ্যে একটি অত্যন্ত গুরুত্বপূর্ণ পার্থক্য রয়েছে:

#### ১. স্ট্রাক্ট: ফিল্ডসমূহ ডিফল্টভাবে প্রাইভেট
একটি স্ট্রাক্টকে `pub struct Product` লিখলেও তার ভেতরের ফিল্ডগুলো **সম্পূর্ণ প্রাইভেট** থাকে, যদি না প্রতিটি ফিল্ডের আগে আলাদা করে `pub` লেখা হয়:
```rust
pub struct Product {
    pub id: u64,          // পাবলিক ফিল্ড: বাইরের মডিউল থেকে রিড/রাইট করা যাবে
    pub title: String,    // পাবলিক
    cost_price: u32,      // প্রাইভেট: কেবল এই মডিউলের কোড এটি এক্সেস করতে পারবে!
}
```
যদি কোনো স্ট্রাক্টে *একটিও* প্রাইভেট ফিল্ড থাকে:
- বাইরের কোনো মডিউল সরাসরি স্ট্রাক্ট লিটারেল দিয়ে অবজেক্ট তৈরি করতে পারবে না (`Product { id: 1, title: ..., cost_price: ... }` এরর দেবে)।
- আপনাকে অবশ্যই একটি পাবলিক কনস্ট্রাক্টর মেথড (যেমন: `pub fn new(...) -> Self`) প্রদান করতে হবে! এটি ডেটা এনক্যাপসুলেশন নিশ্চিত করে।

#### ২. এনাম: ভ্যারিয়েন্টগুলো নিজে থেকেই পাবলিক হয়
কোনো এনামকে যদি `pub enum OrderStatus` ডিক্লেয়ার করা হয়, তবে তার **সবকটি ভ্যারিয়েন্ট এবং ভেতরের পেলোড ডাটা স্বয়ংক্রিয়ভাবে পাবলিক** হয়ে যায়:
```rust
pub enum OrderStatus {
    Pending,
    Confirmed { receipt_id: String }, // ভ্যারিয়েন্ট এবং receipt_id উভয়ই পাবলিক!
    Cancelled,
}
```
কেন এমন নিয়ম? কারণ এনাম হলো নির্দিষ্ট কিছু সম্ভাব্য অবস্থার সমষ্টি যা কলারকে `match` দিয়ে হ্যান্ডেল করতে হয়। ভ্যারিয়েন্ট লুকিয়ে রাখলে প্যাটার্ন ম্যাচিং ভেঙে যাবে।

### উন্নত ভিজিবিলিটি স্পেসিফায়ার
সাধারণ `pub` ছাড়াও রাস্টে সূক্ষ্ম ভিজিবিলিটি কন্ট্রোল রয়েছে:
- `pub`: বিশ্বব্যাপী উন্মুক্ত (লাইব্রেরি ব্যবহারকারী অন্য প্রজেক্ট থেকেও দেখা যাবে)।
- `pub(crate)`: কেবল **বর্তমান ক্রেটের** যেকোনো মডিউলে দৃশ্যমান, কিন্তু বাইরের প্রজেক্টের কাছে সম্পূর্ণ লুকায়িত।
- `pub(super)`: কেবল সরাসরি **প্যারেন্ট মডিউলে** দৃশ্যমান।
- `pub(in path)`: কোনো নির্দিষ্ট পূর্বপুরুষ মডিউল পাথের মধ্যে দৃশ্যমান।

```rust
pub(crate) fn internal_database_sync() {
    // ministore ক্রেটের ভেতরের যেকোনো মডিউল দেখতে পাবে,
    // কিন্তু বাইরের ডিপেন্ডেন্সি হিসেবে ব্যবহারকারীরা দেখতে পাবে না!
}
```

---

## পাথ (Paths) এবং `use` কিওয়ার্ড

কোনো মডিউলের ফাংশন বা টাইপ কল করতে তার পাথ ব্যবহার করা হয়। পাথ দুই ধরনের:

### ১. অ্যাবসলিউট পাথ (Absolute Paths)
ক্রেটের গোড়া থেকে শুরু হয় এবং শুরুতেই `crate` কিওয়ার্ড থাকে:
```rust
let product = crate::models::product::Product::new(...);
```
অ্যাবসলিউট পাথ সম্পূর্ণ স্পষ্ট এবং কোড অন্য কোনো সাবমডিউলে সরিয়ে নিলেও এর অর্থের কোনো পরিবর্তন হয় না।

### ২. রিলেটিভ পাথ (Relative Paths)
বর্তমান মডিউল থেকে শুরু হয় এবং নিচের কিওয়ার্ড ব্যবহার করে:
- `self`: বর্তমান মডিউল স্কোপ বোঝায়।
- `super`: ফাইল সিস্টেমের `..`-এর মতো সরাসরি **প্যারেন্ট মডিউলকে** বোঝায়।

```rust
// src/models/cart.rs-এর ভেতরে:
use super::product::Product; // এক ধাপ ওপরে models-এ গিয়ে product-এ প্রবেশ করে
```

### `use` দিয়ে শর্টকাট তৈরি
কোডের সর্বত্র `crate::models::order::OrderStatus` লেখা বিরক্তিকর। তাই `use` দিয়ে লোকাল স্কোপে শর্টকাট আনা হয়:

```rust
use crate::models::order::OrderStatus;

let status = OrderStatus::Pending; // সংক্ষেপ ও পাঠযোগ্য!
```

#### নেস্টেড পাথ এবং নাম পরিবর্তন
একই মডিউল থেকে একাধিক আইটেম এক লাইনে আনতে কার্লি ব্র্যাকেট ব্যবহার করা যায়:
```rust
use crate::models::product::{Product, ProductCategory};
use crate::models::order::{Order, OrderStatus, PaymentMethod};
```

নামের সংঘাত এড়াতে `as` কিওয়ার্ড ব্যবহার করা যায়:
```rust
use std::fmt::Result as FmtResult;
use std::io::Result as IoResult;
```

---

## `pub use` দিয়ে রি-এক্সপোর্ট: ফ্যাসাড প্যাটার্ন (The Facade Pattern)

আমাদের MiniStore-এর ফাইল বিন্যাস চিন্তা করুন:
- `Product` স্ট্রাক্টটি আছে `crate::models::product::Product`-এ।
- `ShoppingCart` স্ট্রাক্টটি আছে `crate::models::cart::ShoppingCart`-এ।
- `Catalog` আছে `crate::catalog::Catalog`-এ।
- `checkout` ফাংশনটি আছে `crate::checkout::checkout`-এ।

যদি বাইরের কোনো ডেভেলপার আমাদের লাইব্রেরি ব্যবহার করতে গিয়ে দেখে তাকে এত গভীরে ঢুকে টাইপ খুঁজতে হচ্ছে, তবে তা অত্যন্ত জটিল মনে হবে।

### `pub use` কীভাবে কাজ করে?
`pub` এবং `use` একসাথে লিখে আমরা ভেতরের কোনো মডিউলের আইটেমকে ওপরের লেভেলে **রি-এক্সপোর্ট** করতে পারি:

```rust
// src/models/mod.rs-এর ভেতরে:
pub mod product;
pub mod customer;
pub mod cart;
pub mod order;

// প্রধান টাইপগুলোকে সরাসরি models-এর আন্ডারে রি-এক্সপোর্ট:
pub use product::{Product, ProductCategory};
pub use customer::{Customer, OrderId};
pub use cart::{CartItem, ShoppingCart};
pub use order::{Coupon, Order, OrderStatus, PaymentMethod};
```

এবং `src/lib.rs`-এ:
```rust
// src/lib.rs (ক্রেট রুট):
pub mod catalog;
pub mod checkout;
pub mod error;
pub mod models;

// ব্যবহারকারীর জন্য ফ্ল্যাট পাবলিক ফ্যাসাড তৈরি:
pub use catalog::Catalog;
pub use checkout::{calculate_discount, checkout};
pub use error::StoreError;
pub use models::{
    CartItem, Coupon, Customer, Order, OrderId, OrderStatus, PaymentMethod,
    Product, ProductCategory, ShoppingCart,
};
```

এখন যে কেউ (এমনকি আমাদের নিজস্ব `src/main.rs`) খুব সহজেই সরাসরি লিখতে পারে:
```rust
use ministore::{Catalog, Customer, Order, Product, checkout};
```
এটিই হলো **ফ্যাসাড প্যাটার্ন (Facade Pattern)**। ভবিষ্যতে আপনি ভেতরের ফাইল বা ফোল্ডার পরিবর্তন করলেও বাইরের ব্যবহারকারীর কোনো কোড ভাঙবে না!

---

## অন্যান্য ভাষার সাথে মানসিক তুলনা

| কনসেপ্ট | রাস্ট (Rust) | পাইথন (Python) | জাভাস্ক্রিপ্ট / টাইপস্ক্রিপ্ট | গো (Go) | জাভা / সি# |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **কম্পাইলেশন ইউনিট** | ক্রেট (`lib.rs` / `main.rs`) | স্ক্রিপ্ট বা মডিউল (`.py`) | প্যাকেজ (`package.json`) | প্যাকেজ ডিরেক্টরি | প্রজেক্ট / অ্যাসেম্বলি |
| **ফাইল থেকে মডিউল** | স্পষ্ট `mod foo;` ঘোষণা প্রয়োজন | অটোমেটিক (`import foo`) | অটোমেটিক (`import foo from './foo'`) | অটোমেটিক (এক ফোল্ডারের সব `.go` ফাইল একই প্যাকেজ) | অটোমেটিক (`package com.example`) |
| **ডিফল্ট প্রাইভেসি** | **প্রাইভেট** | পাবলিক (কনভেনশন অনুযায়ী `_foo`) | পাবলিক (ক্লাসে `#` প্রাইভেট ফিল্ড) | ছোট হাতের হলে প্রাইভেট; বড় হাতের হলে পাবলিক | প্যাকেজ-প্রাইভেট (জাভা) বা ইন্টারনাল (সি#) |
| **ফিল্ড প্রাইভেসি** | ডিফল্টভাবে প্রাইভেট, `pub` দিতে হয় | পাবলিক | পাবলিক / প্রাইভেট ক্লাস ফিল্ড | নামের প্রথম অক্ষর দিয়ে নির্ধারিত | ফিল্ডের আগে private/public |
| **রি-এক্সপোর্ট** | `pub use foo::Bar;` | `__init__.py`-এ `from foo import Bar` | `export { Bar } from './foo'` | প্যারেন্ট প্যাকেজে এলিয়াস এক্সপোর্ট | রি-ডিক্লেয়ারেশন বা ইনহেরিটেন্স |

---

## কোড উদাহরণ

### উদাহরণ ১: বেসিক মডিউল ডিক্লেয়ারেশন এবং এনক্যাপসুলেশন
```rust
mod storage {
    pub struct Box {
        pub capacity: u32,
        secret_label: String, // প্রাইভেট ফিল্ড
    }

    impl Box {
        pub fn new(capacity: u32, secret_label: String) -> Self {
            Self { capacity, secret_label }
        }

        pub fn reveal_label(&self) -> &str {
            &self.secret_label
        }
    }
}

fn main() {
    let b = storage::Box::new(50, String::from("Fragile Glass"));
    println!("Capacity: {}, Label: {}", b.capacity, b.reveal_label());
    
    // b.secret_label; // কম্পাইলার এরর: secret_label একটি প্রাইভেট ফিল্ড!
}
```

### উদাহরণ ২: প্যারেন্ট স্কোপ এক্সেস করতে `super` কিওয়ার্ড
```rust
mod network {
    fn ping() -> &'static str {
        "pong"
    }

    pub mod client {
        pub fn test_connection() {
            // super দিয়ে প্যারেন্ট মডিউল network-এর আইটেম এক্সেস করা হচ্ছে
            let response = super::ping();
            println!("Connection test: {}", response);
        }
    }
}

fn main() {
    network::client::test_connection();
}
```

---

## MiniStore আর্কিটেকচার এবং মডুলার রিফ্যাক্টরিং

আসুন `ministore/`-এর নতুন ফাইল ট্রি দেখে নিই:

```
ministore/
├── Cargo.toml
└── src/
    ├── error.rs            # সেন্ট্রাল এরর এনাম (StoreError)
    ├── catalog.rs          # Catalog স্ট্রাক্ট ও স্টক কোয়েরি মেথডস
    ├── checkout.rs         # চেকআউট ট্রানজ্যাকশন ও ডিসকাউন্ট লজিক
    ├── models/             # ডোমেইন মডেলস সাবমডিউল
    │   ├── mod.rs          # সাবমডিউল ডিক্লেয়ারেশন ও রি-এক্সপোর্ট
    │   ├── product.rs      # Product ও ProductCategory
    │   ├── customer.rs     # Customer ও OrderId
    │   ├── cart.rs         # CartItem ও ShoppingCart
    │   └── order.rs        # Order, OrderStatus, PaymentMethod, Coupon
    ├── lib.rs              # লাইব্রেরি রুট: ফ্যাসাড এক্সপোর্ট ও ইন্টিগ্রেশন টেস্ট
    └── main.rs             # বাইনারি এন্ট্রি পয়েন্ট (সিএলআই অ্যাপ্লিকেশন)
```

নিচে প্রতিটি ফাইলের সম্পূর্ণ কোড ও ব্যাখ্যা দেওয়া হলো।

---

### ১. `src/error.rs`
সেন্ট্রাল এরর মডিউল যেখানে `StoreError` সংজ্ঞায়িত। লক্ষ্য করুন সব ভ্যারিয়েন্ট ও মেথড `pub` করা হয়েছে।

```rust
/// MiniStore অপারেশনের জন্য ডোমেইন এরর এনাম।
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    ProductNotFound { sku: String },
    InsufficientStock { sku: String, requested: u32, available: u32 },
    EmptyCart,
    InvalidCoupon { code: String, reason: String },
    InvalidStateTransition { action: String, current_state: String },
    OrderAlreadyExists { order_id: u64 },
}

impl StoreError {
    pub fn message(&self) -> String {
        match self {
            StoreError::ProductNotFound { sku } => {
                format!("Product with SKU '{}' does not exist in the catalog.", sku)
            }
            StoreError::InsufficientStock { sku, requested, available } => {
                format!(
                    "Insufficient stock for SKU '{}': requested {}, but only {} available.",
                    sku, requested, available
                )
            }
            StoreError::EmptyCart => {
                String::from("Cannot checkout with an empty shopping cart.")
            }
            StoreError::InvalidCoupon { code, reason } => {
                format!("Invalid coupon code '{}': {}.", code, reason)
            }
            StoreError::InvalidStateTransition { action, current_state } => {
                format!(
                    "Cannot perform action '{}' while order is in '{}' state.",
                    action, current_state
                )
            }
            StoreError::OrderAlreadyExists { order_id } => {
                format!("An order with ID #{} has already been processed.", order_id)
            }
        }
    }
}
```

---

### ২. `src/models/product.rs`
`ProductCategory` এবং `Product` সংজ্ঞায়িত। স্টক কমানোর জন্য এটি `StoreError` ইমপোর্ট করে `use crate::error::StoreError;` দিয়ে।

```rust
use crate::error::StoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductCategory {
    Electronics,
    Clothing,
    Books,
    Home,
}

impl ProductCategory {
    pub fn tax_rate(&self) -> f64 {
        match self {
            ProductCategory::Electronics => 0.15,
            ProductCategory::Clothing => 0.05,
            ProductCategory::Books => 0.00,
            ProductCategory::Home => 0.08,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProductCategory::Electronics => "Consumer Electronics",
            ProductCategory::Clothing => "Apparel & Garments",
            ProductCategory::Books => "Books & Educational Material",
            ProductCategory::Home => "Home & Living Essentials",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub sku: String,
    pub title: String,
    pub category: ProductCategory,
    pub price_cents: u32,
    pub stock: u32,
}

impl Product {
    pub fn new(
        id: u64,
        sku: String,
        title: String,
        category: ProductCategory,
        price_cents: u32,
        stock: u32,
    ) -> Self {
        Self {
            id,
            sku,
            title,
            category,
            price_cents,
            stock,
        }
    }

    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    pub fn reduce_stock(&mut self, quantity: u32) -> Result<(), StoreError> {
        if self.stock >= quantity {
            self.stock -= quantity;
            Ok(())
        } else {
            Err(StoreError::InsufficientStock {
                sku: self.sku.clone(),
                requested: quantity,
                available: self.stock,
            })
        }
    }

    pub fn restore_stock(&mut self, quantity: u32) {
        self.stock += quantity;
    }
}
```

---

### ৩. `src/models/customer.rs`
`Customer` এবং `OrderId` সংজ্ঞায়িত।

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderId(pub u64);

#[derive(Debug, Clone, PartialEq)]
pub struct Customer {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
}

impl Customer {
    pub fn new(id: u64, name: String, email: String, phone: Option<String>) -> Self {
        Self { id, name, email, phone }
    }

    pub fn contact_info(&self) -> String {
        match &self.phone {
            Some(phone) => format!("{} ({})", self.name, phone),
            None => format!("{} <{}>", self.name, self.email),
        }
    }
}
```

---

### ৪. `src/models/cart.rs`
শপিং কার্ট ও কার্ট আইটেম ইমপ্লিমেন্টেশন।

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct CartItem {
    pub product_id: u64,
    pub quantity: u32,
    pub unit_price_cents: u32,
}

impl CartItem {
    pub fn new(product_id: u64, quantity: u32, unit_price_cents: u32) -> Self {
        Self { product_id, quantity, unit_price_cents }
    }

    pub fn line_total_cents(&self) -> u32 {
        self.quantity * self.unit_price_cents
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShoppingCart {
    pub items: Vec<CartItem>,
}

impl ShoppingCart {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(&mut self, product_id: u64, quantity: u32, unit_price_cents: u32) {
        if let Some(existing) = self.items.iter_mut().find(|item| item.product_id == product_id) {
            existing.quantity += quantity;
        } else {
            self.items.push(CartItem::new(product_id, quantity, unit_price_cents));
        }
    }

    pub fn remove_item(&mut self, product_id: u64) -> Option<CartItem> {
        if let Some(pos) = self.items.iter().position(|item| item.product_id == product_id) {
            Some(self.items.remove(pos))
        } else {
            None
        }
    }

    pub fn subtotal_cents(&self) -> u32 {
        self.items.iter().map(|item| item.line_total_cents()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}
```

---

### ৫. `src/models/order.rs`
`PaymentMethod`, `OrderStatus`, `Coupon`, এবং `Order`। এটি তার সিবলিং সাবমডিউল থেকে `CartItem` ও `OrderId` ইমপোর্ট করে `super::` পাথের মাধ্যমে।

```rust
use crate::error::StoreError;
use super::cart::CartItem;
use super::customer::OrderId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentMethod {
    CreditCard,
    CashOnDelivery,
    MobileBanking { provider_code: u8 },
}

impl PaymentMethod {
    pub fn processing_fee_cents(&self) -> u32 {
        match self {
            PaymentMethod::CreditCard => 150,
            PaymentMethod::CashOnDelivery => 0,
            PaymentMethod::MobileBanking { .. } => 50,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            PaymentMethod::CreditCard => "Credit Card",
            PaymentMethod::CashOnDelivery => "Cash on Delivery",
            PaymentMethod::MobileBanking { provider_code } => match provider_code {
                1 => "bKash",
                2 => "Nagad",
                3 => "Rocket",
                _ => "Other Mobile Banking",
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Confirmed { receipt_id: String },
    Shipped { tracking_number: String },
    Delivered,
    Cancelled { reason: String },
}

impl OrderStatus {
    pub fn display_status(&self) -> String {
        match self {
            OrderStatus::Pending => String::from("Pending Verification"),
            OrderStatus::Confirmed { receipt_id } => {
                format!("Confirmed (Receipt: {})", receipt_id)
            }
            OrderStatus::Shipped { tracking_number } => {
                format!("Shipped (Tracking: {})", tracking_number)
            }
            OrderStatus::Delivered => String::from("Delivered to Customer"),
            OrderStatus::Cancelled { reason } => format!("Cancelled: {}", reason),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Coupon {
    pub code: String,
    pub discount_percent: u8,
}

impl Coupon {
    pub fn new(code: String, discount_percent: u8) -> Self {
        Self { code, discount_percent }
    }

    pub fn validate(&self) -> Result<(), StoreError> {
        if self.discount_percent == 0 || self.discount_percent > 100 {
            Err(StoreError::InvalidCoupon {
                code: self.code.clone(),
                reason: format!("discount percentage {} must be between 1 and 100", self.discount_percent),
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub order_id: OrderId,
    pub customer_id: u64,
    pub items: Vec<CartItem>,
    pub discount_cents: u32,
    pub payment_method: PaymentMethod,
    pub status: OrderStatus,
}

impl Order {
    pub fn new(
        order_id: OrderId,
        customer_id: u64,
        items: Vec<CartItem>,
        discount_cents: u32,
        payment_method: PaymentMethod,
    ) -> Self {
        Self {
            order_id,
            customer_id,
            items,
            discount_cents,
            payment_method,
            status: OrderStatus::Pending,
        }
    }

    pub fn subtotal_cents(&self) -> u32 {
        self.items.iter().map(|item| item.line_total_cents()).sum()
    }

    pub fn total_cents(&self) -> u32 {
        let subtotal = self.subtotal_cents();
        let discounted = subtotal.saturating_sub(self.discount_cents);
        discounted + self.payment_method.processing_fee_cents()
    }

    pub fn confirm(&mut self, receipt_id: String) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Pending => {
                self.status = OrderStatus::Confirmed { receipt_id };
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                action: String::from("confirm"),
                current_state: self.status.display_status(),
            }),
        }
    }

    pub fn ship(&mut self, tracking_number: String) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Confirmed { .. } => {
                self.status = OrderStatus::Shipped { tracking_number };
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                action: String::from("ship"),
                current_state: self.status.display_status(),
            }),
        }
    }

    pub fn mark_delivered(&mut self) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Shipped { .. } => {
                self.status = OrderStatus::Delivered;
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                action: String::from("deliver"),
                current_state: self.status.display_status(),
            }),
        }
    }

    pub fn cancel(&mut self, reason: String) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Pending | OrderStatus::Confirmed { .. } => {
                self.status = OrderStatus::Cancelled { reason };
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                action: String::from("cancel"),
                current_state: self.status.display_status(),
            }),
        }
    }
}
```

---

### ৬. `src/models/mod.rs`
`models` সাবমডিউলের এন্ট্রি পয়েন্ট। এটি ভেতরের সাবমডিউল ঘোষণা করে এবং প্রয়োজনীয় টাইপগুলো রি-এক্সপোর্ট করে।

```rust
pub mod cart;
pub mod customer;
pub mod order;
pub mod product;

pub use cart::{CartItem, ShoppingCart};
pub use customer::{Customer, OrderId};
pub use order::{Coupon, Order, OrderStatus, PaymentMethod};
pub use product::{Product, ProductCategory};
```

---

### ৭. `src/catalog.rs`
ইনভেন্টরি ক্যাটালগ ম্যানেজমেন্ট।

```rust
use std::collections::HashMap;
use crate::models::Product;

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

    pub fn find_by_sku_mut(&mut self, sku: &str) -> Option<&mut Product> {
        self.products.get_mut(sku)
    }

    pub fn find_by_id(&self, id: u64) -> Option<&Product> {
        self.products.values().find(|product| product.id == id)
    }

    pub fn find_by_id_mut(&mut self, id: u64) -> Option<&mut Product> {
        self.products.values_mut().find(|product| product.id == id)
    }

    pub fn product_price(&self, sku: &str) -> Option<u32> {
        self.find_by_sku(sku).map(|p| p.price_cents)
    }

    pub fn is_product_in_stock(&self, sku: &str) -> bool {
        self.find_by_sku(sku)
            .map(|p| p.is_in_stock())
            .unwrap_or(false)
    }

    pub fn len(&self) -> usize {
        self.products.len()
    }

    pub fn is_empty(&self) -> bool {
        self.products.is_empty()
    }
}
```

---

### ৮. `src/checkout.rs`
অর্ডার প্লেসমেন্ট এবং ডিসকাউন্ট ক্যালকুলেশন কোঅর্ডিনেট করে।

```rust
use std::collections::HashMap;
use crate::catalog::Catalog;
use crate::error::StoreError;
use crate::models::{
    CartItem, Coupon, Customer, Order, OrderId, PaymentMethod, ProductCategory, ShoppingCart,
};

pub fn calculate_discount(subtotal_cents: u32, coupon: Option<&Coupon>) -> u32 {
    match coupon {
        Some(c) => (subtotal_cents * c.discount_percent as u32) / 100,
        None => 0,
    }
}

pub fn count_products_by_department(
    items: &[CartItem],
    catalog: &Catalog,
) -> HashMap<ProductCategory, u32> {
    let mut counts: HashMap<ProductCategory, u32> = HashMap::new();

    for item in items {
        if let Some(product) = catalog.find_by_id(item.product_id) {
            let counter = counts.entry(product.category).or_insert(0);
            *counter += item.quantity;
        }
    }

    counts
}

pub fn order_dispatch_advisory(status: &crate::models::OrderStatus) -> &'static str {
    match status {
        crate::models::OrderStatus::Pending => "Await payment confirmation before dispatch.",
        crate::models::OrderStatus::Confirmed { .. } => "Approved for immediate packaging and carrier assignment.",
        crate::models::OrderStatus::Shipped { .. } => "Carrier transit in progress; track coordinates.",
        crate::models::OrderStatus::Delivered => "Package received by customer; archive manifest.",
        crate::models::OrderStatus::Cancelled { .. } => "Halt fulfillment; reverse reserved warehouse inventory.",
    }
}

pub fn checkout(
    order_id: OrderId,
    customer: &Customer,
    cart: &mut ShoppingCart,
    catalog: &mut Catalog,
    payment_method: PaymentMethod,
    mut coupon: Option<Coupon>,
) -> Result<Order, StoreError> {
    if cart.is_empty() {
        return Err(StoreError::EmptyCart);
    }

    if let Some(ref c) = coupon {
        c.validate()?;
    }

    // ১. স্টক চেক করা (মিউটেশন শুরু করার আগেই)
    for item in &cart.items {
        let product = catalog
            .find_by_id(item.product_id)
            .ok_or_else(|| StoreError::ProductNotFound {
                sku: format!("ID-{}", item.product_id),
            })?;

        if product.stock < item.quantity {
            return Err(StoreError::InsufficientStock {
                sku: product.sku.clone(),
                requested: item.quantity,
                available: product.stock,
            });
        }
    }

    // ২. ক্যাটালগ থেকে স্টক হ্রাস করা
    for item in &cart.items {
        let product = catalog.find_by_id_mut(item.product_id).unwrap();
        product.reduce_stock(item.quantity)?;
    }

    // ৩. ডিসকাউন্ট ও অর্ডার তৈরি
    let subtotal = cart.subtotal_cents();
    let discount = calculate_discount(subtotal, coupon.as_ref());
    let active_coupon = coupon.take();
    let _ = active_coupon;

    let items = std::mem::take(&mut cart.items);

    let order = Order::new(
        order_id,
        customer.id,
        items,
        discount,
        payment_method,
    );

    Ok(order)
}
```

---

### ৯. `src/lib.rs`
আমাদের লাইব্রেরি ক্রেট রুট। এটি সাবমডিউলসমূহ ডিক্লেয়ার করে, `pub use` দিয়ে ফ্যাসাড তৈরি করে এবং পূর্ণাঙ্গ টেস্ট স্যুট ধারণ করে।

```rust
pub mod catalog;
pub mod checkout;
pub mod error;
pub mod models;

// পাবলিক ফ্যাসাড রি-এক্সপোর্ট
pub use catalog::Catalog;
pub use checkout::{
    calculate_discount, checkout, count_products_by_department, order_dispatch_advisory,
};
pub use error::StoreError;
pub use models::{
    CartItem, Coupon, Customer, Order, OrderId, OrderStatus, PaymentMethod, Product,
    ProductCategory, ShoppingCart,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_category_tax_rates() {
        assert_eq!(ProductCategory::Electronics.tax_rate(), 0.15);
        assert_eq!(ProductCategory::Clothing.tax_rate(), 0.05);
        assert_eq!(ProductCategory::Books.tax_rate(), 0.00);
        assert_eq!(ProductCategory::Home.tax_rate(), 0.08);
    }

    #[test]
    fn test_payment_method_fees_and_descriptions() {
        let cc = PaymentMethod::CreditCard;
        let cod = PaymentMethod::CashOnDelivery;
        let bkash = PaymentMethod::MobileBanking { provider_code: 1 };

        assert_eq!(cc.processing_fee_cents(), 150);
        assert_eq!(cod.processing_fee_cents(), 0);
        assert_eq!(bkash.processing_fee_cents(), 50);

        assert_eq!(cc.display_name(), "Credit Card");
        assert_eq!(bkash.display_name(), "bKash");
    }

    #[test]
    fn test_order_status_valid_lifecycle() {
        let mut order = Order::new(
            OrderId(100),
            1,
            vec![CartItem::new(10, 1, 5000)],
            0,
            PaymentMethod::CashOnDelivery,
        );

        assert_eq!(order.status, OrderStatus::Pending);

        assert!(order.confirm(String::from("REC-100")).is_ok());
        assert_eq!(
            order.status,
            OrderStatus::Confirmed { receipt_id: String::from("REC-100") }
        );

        assert!(order.ship(String::from("TRK-XYZ-99")).is_ok());
        assert_eq!(
            order.status,
            OrderStatus::Shipped { tracking_number: String::from("TRK-XYZ-99") }
        );

        assert!(order.mark_delivered().is_ok());
        assert_eq!(order.status, OrderStatus::Delivered);
    }

    #[test]
    fn test_order_cancellation_prevention() {
        let mut order = Order::new(
            OrderId(200),
            1,
            vec![],
            0,
            PaymentMethod::CreditCard,
        );
        order.confirm(String::from("REC-200")).unwrap();
        order.ship(String::from("TRK-200")).unwrap();

        let cancel_result = order.cancel(String::from("Customer changed mind"));
        assert!(cancel_result.is_err());
        match cancel_result {
            Err(StoreError::InvalidStateTransition { action, current_state }) => {
                assert_eq!(action, "cancel");
                assert!(current_state.contains("Shipped"));
            }
            _ => panic!("Expected InvalidStateTransition error"),
        }
    }

    #[test]
    fn test_coupon_discount_and_take() {
        let mut coupon = Some(Coupon::new(String::from("SAVE15"), 15));
        let discount = calculate_discount(10000, coupon.as_ref());
        assert_eq!(discount, 1500);

        let taken = coupon.take();
        assert!(taken.is_some());
        assert!(coupon.is_none());
    }

    #[test]
    fn test_coupon_validation_error() {
        let bad_coupon = Coupon::new(String::from("BAD"), 150);
        let result = bad_coupon.validate();
        assert!(result.is_err());
        match result {
            Err(StoreError::InvalidCoupon { code, reason }) => {
                assert_eq!(code, "BAD");
                assert!(reason.contains("between 1 and 100"));
            }
            _ => panic!("Expected InvalidCoupon error"),
        }
    }

    #[test]
    fn test_customer_optional_phone() {
        let alice = Customer::new(
            1,
            String::from("Alice"),
            String::from("alice@example.com"),
            Some(String::from("+1-555-0100")),
        );
        let bob = Customer::new(
            2,
            String::from("Bob"),
            String::from("bob@example.com"),
            None,
        );

        assert_eq!(alice.contact_info(), "Alice (+1-555-0100)");
        assert_eq!(bob.contact_info(), "Bob <bob@example.com>");
    }

    #[test]
    fn test_catalog_option_lookups() {
        let mut catalog = Catalog::new();
        let product = Product::new(
            42,
            String::from("SKU-BOOK"),
            String::from("Rust Book"),
            ProductCategory::Books,
            3500,
            10,
        );
        catalog.add_product(product);

        assert!(catalog.find_by_sku("SKU-BOOK").is_some());
        assert!(catalog.find_by_sku("NON-EXISTENT").is_none());
        assert_eq!(catalog.product_price("SKU-BOOK"), Some(3500));
        assert_eq!(catalog.product_price("NON-EXISTENT"), None);
        assert!(catalog.is_product_in_stock("SKU-BOOK"));
        assert!(!catalog.is_product_in_stock("NON-EXISTENT"));
    }

    #[test]
    fn test_cart_item_option_lookup() {
        let mut cart = ShoppingCart::new();
        cart.add_item(10, 2, 1500);
        cart.add_item(20, 1, 3000);

        let removed = cart.remove_item(10);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().product_id, 10);

        let non_existent = cart.remove_item(99);
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_product_stock_reduction_error() {
        let mut product = Product::new(
            1,
            String::from("SKU-1"),
            String::from("Item"),
            ProductCategory::Electronics,
            1000,
            5,
        );
        assert!(product.reduce_stock(3).is_ok());
        assert_eq!(product.stock, 2);

        let err = product.reduce_stock(5);
        assert!(err.is_err());
        assert_eq!(
            err,
            Err(StoreError::InsufficientStock {
                sku: String::from("SKU-1"),
                requested: 5,
                available: 2,
            })
        );
    }

    #[test]
    fn test_checkout_error_propagation_and_success() {
        let mut catalog = Catalog::new();
        catalog.add_product(Product::new(
            1,
            String::from("SKU-LAPTOP"),
            String::from("Laptop"),
            ProductCategory::Electronics,
            100000,
            2,
        ));

        let customer = Customer::new(
            1,
            String::from("Carol"),
            String::from("carol@example.com"),
            None,
        );

        let mut empty_cart = ShoppingCart::new();
        let empty_checkout = checkout(
            OrderId(1),
            &customer,
            &mut empty_cart,
            &mut catalog,
            PaymentMethod::CashOnDelivery,
            None,
        );
        assert_eq!(empty_checkout, Err(StoreError::EmptyCart));

        let mut cart = ShoppingCart::new();
        cart.add_item(1, 1, 100000);
        let coupon = Some(Coupon::new(String::from("SAVE10"), 10));

        let order_res = checkout(
            OrderId(2),
            &customer,
            &mut cart,
            &mut catalog,
            PaymentMethod::CreditCard,
            coupon,
        );

        assert!(order_res.is_ok());
        let order = order_res.unwrap();
        assert_eq!(order.subtotal_cents(), 100000);
        assert_eq!(order.discount_cents, 10000);
        assert_eq!(order.total_cents(), 90000 + 150);
        assert_eq!(catalog.find_by_sku("SKU-LAPTOP").unwrap().stock, 1);
        assert!(cart.is_empty());
    }

    #[test]
    fn test_order_total_with_payment_fee() {
        let order = Order::new(
            OrderId(50),
            1,
            vec![CartItem::new(1, 2, 2000)],
            500,
            PaymentMethod::MobileBanking { provider_code: 1 },
        );
        assert_eq!(order.total_cents(), 3500 + 50);
    }
}
```

---

### ১০. `src/main.rs`
বাইনারি এন্ট্রি পয়েন্ট। এটি যে কত পরিচ্ছন্ন তা লক্ষ্য করুন, কারণ এটি `ministore` লাইব্রেরিকে সাধারণ ক্লায়েন্ট হিসেবে ব্যবহার করছে!

```rust
use ministore::{
    checkout, Catalog, Coupon, Customer, OrderId, PaymentMethod, Product, ProductCategory,
    ShoppingCart,
};

fn main() {
    println!("=== MiniStore: Modules & Clean Project Architecture (Part II) ===\n");

    // ১. ক্যাটালগ শুরু করা
    let mut catalog = Catalog::new();
    let keyboard = Product::new(
        101,
        String::from("TECH-KEY-001"),
        String::from("Tenkeyless Mechanical Keyboard"),
        ProductCategory::Electronics,
        12000,
        5,
    );
    let mouse = Product::new(
        102,
        String::from("TECH-MOU-002"),
        String::from("Ergonomic Wireless Mouse"),
        ProductCategory::Electronics,
        4500,
        10,
    );

    catalog.add_product(keyboard);
    catalog.add_product(mouse);
    println!("1. Catalog initialized with {} products.", catalog.len());

    // ২. কাস্টমার প্রোফাইল তৈরি
    let customer = Customer::new(
        1,
        String::from("Margaret Hamilton"),
        String::from("margaret@apollo.org"),
        Some(String::from("+1-555-0199")),
    );
    println!("2. Customer: {}", customer.contact_info());

    // ৩. শপিং কার্ট তৈরি
    let mut cart = ShoppingCart::new();
    cart.add_item(101, 1, 12000); // ১টি কীবোর্ড
    cart.add_item(102, 2, 4500);  // ২টি মাউস

    let coupon = Coupon::new(String::from("LAUNCH20"), 20);

    // ৪. মডুলার সার্ভিস দিয়ে চেকআউট প্রক্রিয়া সম্পন্ন করা
    println!("\n3. Processing checkout through modular services...");
    match checkout(
        OrderId(901),
        &customer,
        &mut cart,
        &mut catalog,
        PaymentMethod::CreditCard,
        Some(coupon),
    ) {
        Ok(mut order) => {
            println!(
                "   Checkout Order #{} created successfully!",
                order.order_id.0
            );
            println!(
                "   Subtotal: ${:.2} | Total: ${:.2}",
                order.subtotal_cents() as f64 / 100.0,
                order.total_cents() as f64 / 100.0
            );

            // ৫. অর্ডার লাইফসাইকেল ট্রানজিশন
            println!("\n4. Order Lifecycle Transitions:");
            order.confirm(String::from("REC-901-HAMILTON")).unwrap();
            println!("   Order confirmed: {}", order.status.display_status());

            order.ship(String::from("TRK-FEDEX-77189")).unwrap();
            println!("   Order shipped:   {}", order.status.display_status());

            // অবৈধ ট্রানজিশন প্রতিরোধ
            if let Err(err) = order.cancel(String::from("Customer changed mind")) {
                println!("   Cancellation prevented -> {}", err.message());
            }

            order.mark_delivered().unwrap();
            println!(
                "   Final Lifecycle State: {}",
                order.status.display_status()
            );
        }
        Err(err) => println!("   Checkout failed: {}", err.message()),
    }
}
```

---

## সাধারণ কম্পাইলার এরর এবং সমাধানের উপায়

### ১. `error[E0583]: file not found for module`
**ভুল কোড**:
```rust
// src/lib.rs
mod analytics;
```
**কম্পাইলার মেসেজ**:
```text
error[E0583]: file not found for module `analytics`
 --> src/lib.rs:5:1
  |
5 | mod analytics;
  | ^^^^^^^^^^^^^^
  |
  = help: to create the module `analytics`, create file "src/analytics.rs" or "src/analytics/mod.rs"
```
**সমাধান**:
মনে রাখবেন, রাস্টে `mod analytics;` লিখলে কম্পাইলার আশা করে আপনি `src/analytics.rs` ফাইল অথবা `src/analytics/mod.rs` ফোল্ডার তৈরি করেছেন। ফাইলটি ফিজিক্যালি তৈরি করে দিলেই এরর সমাধান হবে!

---

### ২. `error[E0616]: field of struct is private`
**ভুল কোড**:
```rust
mod models {
    pub struct Product {
        price_cents: u32, // pub দেওয়া হয়নি!
    }
}

fn main() {
    let p = models::Product { price_cents: 1000 };
}
```
**কম্পাইলার মেসেজ**:
```text
error[E0616]: field `price_cents` of struct `Product` is private
```
**সমাধান**:
ফিল্ডের আগে `pub price_cents: u32;` যোগ করুন অথবা একই মডিউলে একটি পাবলিক কনস্ট্রাক্টর মেথড `pub fn new(...) -> Self` সরবরাহ করুন।

---

### ৩. `error[E0432]: unresolved import`
**ভুল কোড**:
```rust
// src/models/order.rs-এ
use crate::cart::CartItem; // ভুল: cart রুট লেভেলে নেই, models-এর ভেতর আছে!
```
**কম্পাইলার মেসেজ**:
```text
error[E0432]: unresolved import `crate::cart`
 --> src/models/order.rs:2:5
  |
2 | use crate::cart::CartItem;
  |     ^^^^^^^^^^^ could not find `cart` in the crate root
```
**সমাধান**:
সঠিক পাথ ব্যবহার করুন: `use crate::models::cart::CartItem;` অথবা রিলেটিভ পাথ `use super::cart::CartItem;`।

---

## আইডিওম্যাটিক রাস্ট প্র্যাকটিস

1. **`src/lib.rs` + `src/main.rs` যৌথভাবে ব্যবহার করুন**: সমস্ত বিজনেস লজিক লাইব্রেরি ক্রেটে রাখুন এবং `main.rs`-কে পাতলা রাখুন।
2. **ফ্যাসাডের জন্য `pub use` ব্যবহার করুন**: ভেতরের অতিরিক্ত নেস্টিং যেমন `crate::internal::db::connection::Pool` বাইরের ব্যবহারকারীর কাছে সরাসরি এক্সপোজ করবেন না।
3. **ডেটার শুদ্ধতা রক্ষায় ফিল্ড প্রাইভেট রাখুন**: স্টক বা ডিসকাউন্টের মতো স্পর্শকাতর ফিল্ড প্রাইভেট রাখুন এবং মেথডের মাধ্যমে ইনভ্যারিয়েন্ট যাচাই করুন।
4. **সিবলিং ইমপোর্ট এক লাইনে আনুন**: `use crate::models::{Product, Customer, Order};` ব্যবহার করুন।
5. **অ্যাপ্লিকেশনে ওয়াইল্ডকার্ড (`*`) এড়িয়ে চলুন**: কেবল টেস্ট মডিউলে `use super::*;` ব্যবহার করা নিরাপদ।

---

## অনুশীলন (Hands-On Exercises)

### অনুশীলন ১: নতুন `ShippingAddress` মডেল যুক্ত করা
1. `src/models/`-এ `address.rs` নামের একটি নতুন ফাইল তৈরি করুন।
2. `ShippingAddress` নামের একটি স্ট্রাক্ট লিখুন যার ফিল্ড হবে: `street: String`, `city: String`, `postal_code: String`, এবং `country: String`।
3. `src/models/mod.rs`-এ `pub mod address;` ঘোষণা করুন এবং `ShippingAddress` রি-এক্সপোর্ট করুন।
4. `Order` স্ট্রাক্টে `pub shipping_address: Option<ShippingAddress>` যোগ করুন।
5. `cargo test` ও `cargo check` রান করে যাচাই করুন।

### অনুশীলন ২: একটি ডেডিকেটেড `Pricing` মডিউল তৈরি করা
1. `src/pricing.rs` ফাইল তৈরি করুন।
2. `checkout.rs` থেকে `calculate_discount` ফাংশনটি `pricing.rs`-এ সরিয়ে নিন।
3. প্রতিটি ক্যাটাগরির ট্যাক্স রেট ব্যবহার করে একটি নতুন ফাংশন লিখুন: `pub fn calculate_sales_tax(subtotal: u32, category: ProductCategory) -> u32`।
4. `src/lib.rs` থেকে ফাংশন দুটি রি-এক্সপোর্ট করুন।

---

## চেকপয়েন্ট (Checkpoint)

কম্পাইলার চেক ও ইউনিট টেস্ট চালান:
```bash
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
cargo run
```

প্রত্যাশিত টেস্ট আউটপুট:
```text
running 12 tests
test tests::test_cart_item_option_lookup ... ok
test tests::test_catalog_option_lookups ... ok
test tests::test_checkout_error_propagation_and_success ... ok
test tests::test_coupon_discount_and_take ... ok
test tests::test_coupon_validation_error ... ok
test tests::test_customer_optional_phone ... ok
test tests::test_order_cancellation_prevention ... ok
test tests::test_order_status_valid_lifecycle ... ok
test tests::test_order_total_with_payment_fee ... ok
test tests::test_payment_method_fees_and_descriptions ... ok
test tests::test_product_category_tax_rates ... ok
test tests::test_product_stock_reduction_error ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

প্রত্যাশিত বাইনারি আউটপুট (`cargo run`):
```text
=== MiniStore: Modules & Clean Project Architecture (Part II) ===

1. Catalog initialized with 2 products.
2. Customer: Margaret Hamilton (+1-555-0199)

3. Processing checkout through modular services...
   Checkout Order #901 created successfully!
   Subtotal: $210.00 | Total: $148.50

4. Order Lifecycle Transitions:
   Order confirmed: Confirmed (Receipt: REC-901-HAMILTON)
   Order shipped:   Shipped (Tracking: TRK-FEDEX-77189)
   Cancellation prevented -> Cannot perform action 'cancel' while order is in 'Shipped (Tracking: TRK-FEDEX-77189)' state
   Final Lifecycle State: Delivered to Customer
```

---

## আমরা কী শিখলাম
- কীভাবে কার্গো প্যাকেজ বাইনারি ক্রেট এবং লাইব্রেরি ক্রেট ধারণ করে।
- কীভাবে `mod` কিওয়ার্ড দিয়ে মডিউল ডিক্লেয়ার করতে হয় এবং রাস্ট কীভাবে ফাইল সিস্টেমের সাথে তা ম্যাপ করে।
- "বাই ডিফল্ট প্রাইভেট" নীতি এবং `pub`, `pub(crate)` এবং `pub(super)` কীভাবে কার্যকর এনক্যাপসুলেশন নিশ্চিত করে।
- স্ট্রাক্ট ফিল্ড প্রাইভেসি কীভাবে ডেটার শুদ্ধতা বজায় রাখে।
- `crate::` এবং `super::` ব্যবহার করে মডিউল ট্রি নেভিগেশন।
- `pub use` ফ্যাসাড প্যাটার্ন ব্যবহার করে কীভাবে ক্লিন এবং ইউজার-ফ্রেন্ডলি পাবলিক এপিআই তৈরি করা যায়।
- সফলভাবে সম্পন্ন হলো **পার্ট ২: বিজনেস লজিক মডেলিং** একটি প্রোডাকশন-রেডি, মডুলার MiniStore প্রজেক্টের মাধ্যমে!

---

## পরবর্তীতে কী আসছে
অভিনন্দন! আপনি সফলভাবে **পার্ট ২: বিজনেস লজিক মডেলিং** সম্পন্ন করেছেন!

**পার্ট ৩: অ্যাডভান্সড আইডিওম্যাটিক রাস্ট**-এ আমরা শুরু করব **অধ্যায় ১৪: জেনেরিকস (Generics)** নিয়ে! আমরা শিখব কীভাবে টাইপ সেফটি ও জিরো-কস্ট পারফরম্যান্স অক্ষুণ্ণ রেখেই অত্যন্ত নমনীয় এবং পুনর্ব্যবহারযোগ্য অ্যালগরিদম ও ডেটা স্ট্রাকচার লিখতে হয়!
