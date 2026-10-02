# ২২. পারসিস্টেন্স ও ফাইল আই/ও (Persistence in Rust)

## আপনি যা শিখবেন
- কেন **মেমরিতে থাকা অ্যাপ্লিকেশনের স্টেট ক্ষণস্থায়ী (ephemeral)** এবং কেন বাস্তব অ্যাপ্লিকেশনে পারসিস্টেন্ট স্টোরেজ প্রয়োজন।
- স্ট্যান্ডার্ড লাইব্রেরির ফাইল আই/ও ফান্ডামেন্টালস: **`std::fs`**, **`std::path::{Path, PathBuf}`**, এবং **`std::io`**।
- **সিরিয়ালাইজেশন (Serialization)** ও **ডিসিরিয়ালাইজেশন (Deserialization)** কী এবং কীভাবে **সার্ডে (Serde)** ফ্রেমওয়ার্ক জিরো-ওভারহেড কনভার্সন নিশ্চিত করে।
- জটিল স্ট্রাক্ট ও এনামে **`Serialize`** এবং **`Deserialize`** ডেরাইভ করা।
- জেনেরিক হেল্পারে ডাইনামিকালি সাইজড স্লাইস (`&[Order]`) সাপোর্ট করার জন্য শিথিলকৃত **`?Sized`** ট্রেইট বাউন্ডের ব্যবহার।
- টেম্পোরারি ফাইল এবং অ্যাটমিক ফাইলরিনেম (`fs::rename`) ব্যবহারের মাধ্যমে **ক্র্যাশ-প্রুফ অ্যাটমিক রাইট (Atomic Writes)** ডিজাইন করা।
- লো-লেভেল আই/ও এবং জেএসন পার্সিং এররগুলোকে ইউজার-ফ্রেন্ডলি ডোমেন এররে রূপান্তর (`StoreError::IoError` এবং `StoreError::SerializationError`)।
- মিনিস্টোরে পারসিস্টেন্স ইঞ্জিন তৈরি:
  - `catalog.json` ফাইল থেকে ক্যাটালগ সেভ এবং লোড করা।
  - `customers.json` ফাইলে গ্রাহকদের ডেটা সংরক্ষণ করা।
  - `orders.json` ফাইলে অর্ডারের লেনদেন হিস্ট্রি পারসিস্ট করা।
  - `StoreSnapshot` দিয়ে নির্দিষ্ট সময়ের সম্পূর্ণ ব্যাকআপ স্ন্যাপশট তৈরি এবং কোল্ড স্টার্টে সম্পূর্ণ স্টেট রিস্টোর করা।

---

## আমাদের এটি কেন প্রয়োজন?

১ম থেকে ২১তম অধ্যায় পর্যন্ত মিনিস্টোর মৌলিক স্ট্রাক্ট থেকে শুরু করে একটি উচ্চ ক্ষমতাসম্পন্ন কনকারেন্ট ও অ্যাসিঙ্ক্রোনাস অ্যাপ্লিকেশনে রূপ নিয়েছে। কিন্তু এ পর্যন্ত আমাদের প্রোডাক্ট, কার্ট, অর্ডার এবং সেলস মেট্রিক্সের একটি মারাত্মক সীমাবদ্ধতা ছিল:

> **সমস্ত তথ্য কেবল ভোলাটাইল র‍্যামে (RAM) সংরক্ষিত ছিল।**

যখনই প্রোগ্রাম বন্ধ হয়, টার্মিনাল উইন্ডো কেটে দেওয়া হয়, কিংবা সার্ভার রিস্টার্ট হয়:
- ক্যাটালগে থাকা প্রতিটি প্রোডাক্ট মুছে যায়।
- কাস্টমারদের সম্পন্ন করা প্রতিটি অর্ডার হারিয়ে যায়।
- সেলস অডিটের সমস্ত হিসেব শূন্য হয়ে যায়।

```
অস্থায়ী র‍্যাম মেমরি (Ephemeral):
[অ্যাপ্লিকেশন চলছে] ──► ক্যাটালগ, অর্ডার, কাস্টমার র‍্যামে উপস্থিত
        │
[প্রসেস বন্ধ হলো]   ──► ওএস সমস্ত র‍্যাম ফাঁকা করে দিল
        │
[নতুন স্টার্টআপ]    ──► খালি মেমরি! পূর্বের সব ডেটা চিরতরে হারিয়ে গেল!
```

বাস্তব জগতের একটি স্টোর পরিচালনা করতে ডেটাকে প্রসেস রিস্টার্টের পরেও টিকিয়ে রাখতে হয়। জটিল ডেটাবেস কিংবা ওয়েব সার্ভারের দিকে যাওয়ার আগে সফটওয়্যার ইঞ্জিনিয়ারিংয়ের সবচেয়ে মৌলিক ভিত্তি হলো **ফাইলসিস্টেম পারসিস্টেন্স (Filesystem Persistence)**: অর্থাৎ হার্ডডিস্ক বা এসএসডিতে বিশ্বজনীন ও মানব-পাঠযোগ্য ফরম্যাট যেমন **JSON (JavaScript Object Notation)**-এ ডেটা সংরক্ষণ করা।

```
স্থায়ী ফাইল স্টোরেজ (Persistent):
[অ্যাপ্লিকেশন চলছে] ──► মেমরির স্ট্রাক্টকে JSON টেক্সটে রূপান্তর
        │
[অ্যাটমিক রাইট]     ──► ডিস্কে নিরাপদে সংরক্ষণ (যেমন data/catalog.json)
        │
[প্রসেস বন্ধ হলো]   ──► ফাইল ডিস্কেই অক্ষত থেকে যায়
        │
[পরবর্তী স্টার্টআপ] ──► ডিস্ক থেকে JSON পড়ে পুনরায় স্ট্রংলি-টাইপড রাস্ট স্ট্রাক্টে লোড
```

---

## রাস্টে পারসিস্টেন্সের মূলনীতি ও গঠন

### ১. `std::fs` এবং `std::path` দিয়ে ফাইলসিস্টেম আই/ও

রাস্টের স্ট্যান্ডার্ড লাইব্রেরি ক্রস-প্ল্যাটফর্ম ফাইল ম্যানিপুলেশনের জন্য শক্তিশালী টুল সরবরাহ করে:

- **`std::path::Path`**: ফাইলসিস্টেম পাথের একটি আনসাইজড স্লাইস ধার করে রাখা ভিউ (যেমন `&str`)।
- **`std::path::PathBuf`**: হিপ মেমরিতে থাকা ওউনড এবং মিউটেবল পাথ (যেমন `String`)।
- **`std::fs::read_to_string(path)`**: একটি ফাইলের সমস্ত টেক্সট একবারে পড়ে ওউনড `String`-এ নিয়ে আসে।
- **`std::fs::write(path, bytes)`**: নির্দিষ্ট পাথে বাইট স্লাইস লেখে (প্রয়োজনে ফাইল তৈরি করে বা পুরোনোটা মুছে নতুন করে লেখে)।
- **`std::fs::create_dir_all(path)`**: প্যারেন্ট ডিরেক্টরিসহ সম্পূর্ণ ফোল্ডার পাথ তৈরি করে।
- **`std::fs::rename(from, to)`**: অত্যন্ত দ্রুত ও অ্যাটমিক পদ্ধতিতে একটি ফাইলকে অন্য নামে বা নতুন পাথে স্থানান্তর করে।

```rust
use std::fs;
use std::path::PathBuf;

let dir = PathBuf::from("data");
fs::create_dir_all(&dir).expect("ডিরেক্টরি তৈরি ব্যর্থ");

let file_path = dir.join("note.txt");
fs::write(&file_path, b"MiniStore Persistent Note").expect("রাইট ব্যর্থ");

let contents = fs::read_to_string(&file_path).expect("রিড ব্যর্থ");
println!("ফাইল থেকে পড়া হলো: {contents}");
```

---

### ২. সার্ডে (Serde) কী?

**Serde** (শব্দটি এসেছে **Ser**ialize / **De**serialize থেকে) হলো রাস্টের সবচেয়ে জনপ্রিয় ও স্ট্যান্ডার্ড ডাটা রূপান্তর ফ্রেমওয়ার্ক। এর দুটি প্রধান কাজ:

1. **সিরিয়ালাইজেশন (Serialization)**: মেমরিতে থাকা রাস্ট ডেটা স্ট্রাকচারকে (যেমন স্ট্রাক্ট বা এনাম) পোর্টেবল ডাটা ফরম্যাটে (যেমন JSON, YAML, TOML বা বাইনারি MessagePack) রূপান্তর করা।
2. **ডিসিরিয়ালাইজেশন (Deserialization)**: কোনো এক্সটার্নাল ডাটা ফাইল বা নেটওয়ার্ক পে-লোড পড়ে পুনরায় স্ট্রংলি-টাইপড রাস্ট ডেটা স্ট্রাকচারে ফিরিয়ে আনা।

অন্যান্য ভাষার মতো সার্ডে কোনো ধীরগতির রানটাইম রিফ্লেকশন (Runtime Reflection) ব্যবহার করে না। বরং কম্পাইল-টাইমে রাস্টের ট্রেইট সিস্টেম ও প্রসিডিউরাল ডেরাইভ ম্যাক্রো (`#[derive(Serialize, Deserialize)]`) ব্যবহার করে শূন্য-ওভারহেডের অত্যন্ত দ্রুত মেশিন কোড তৈরি করে।

```
                       সিরিয়ালাইজেশন
রাস্ট স্ট্রাক্ট / এনাম ──────────────────► JSON স্ট্রিং / ফাইল
                       ◄──────────────────
                      ডিসিরিয়ালাইজেশন
```

আপনার `Cargo.toml`-এ সার্ডে যোগ করুন:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

### ৩. `Serialize` এবং `Deserialize` ডেরাইভ করা

যেকোনো স্ট্রাক্ট বা এনামের উপরে `#[derive(Serialize, Deserialize)]` লিখে দিলেই কম্পাইলার স্বয়ংক্রিয়ভাবে সার্ডের ট্রেইটগুলো ইমপ্লিমেন্ট করে ফেলে:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ProductCategory {
    Electronics,
    OfficeSupplies,
    Furniture,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: u64,
    pub sku: String,
    pub name: String,
    pub category: ProductCategory,
    pub price_cents: u32,
    pub stock: u32,
}
```

একটি প্রোডাক্টকে ফরম্যাটেড JSON টেক্সটে রূপান্তর:
```rust
let product = Product {
    id: 101,
    sku: String::from("TECH-KEY-001"),
    name: String::from("Mechanical Keyboard"),
    category: ProductCategory::Electronics,
    price_cents: 12000,
    stock: 10,
};

let json_string = serde_json::to_string_pretty(&product).unwrap();
println!("{json_string}");
```

আউটপুট:
```json
{
  "id": 101,
  "sku": "TECH-KEY-001",
  "name": "Mechanical Keyboard",
  "category": "Electronics",
  "price_cents": 12000,
  "stock": 10
}
```

JSON টেক্সট থেকে পুনরায় রাস্ট স্ট্রাক্টে ফিরিয়ে আনা:
```rust
let restored: Product = serde_json::from_str(&json_string).unwrap();
assert_eq!(product, restored);
```

---

### ৪. ডাইনামিকালি সাইজড টাইপস ও `?Sized` ট্রেইট বাউন্ড

ফাইল সেভ করার জন্য একটি জেনেরিক হেল্পার মেথডের কথা ভাবুন:

```rust
pub fn save_json<T: Serialize>(&self, filename: &str, data: &T) -> Result<PathBuf, StoreError>
```

রাস্টে প্রতিটি জেনেরিক টাইপ প্যারামিটার `T`-এর পেছনে একটি গোপন ট্রেইট বাউন্ড থাকে: `T: Sized`। এর মানে হলো কম্পাইল টাইমে `T`-এর সাইজ জানা থাকতে হবে।

কিন্তু যখন আপনি কোনো স্লাইস পাস করতে চান (যেমন `&[Order]` বা `&[Customer]`), তখন আসল টাইপ `[Order]` হলো একটি **ডাইনামিকালি সাইজড টাইপ (DST)**। রানটাইমে স্লাইসের লেন্থ যাই হোক না কেন, কম্পাইল টাইমে তার সাইজ নিশ্চিত নয়! আপনি যদি `&[Order]` পাঠান, তাহলে কম্পাইলার এরর দেবে: `error[E0277]: the size for values of type [Order] cannot be known at compilation time`।

এক্ষেত্রে সাইজড টাইপ (যেমন `&Catalog`) এবং আনসাইজড স্লাইস (যেমন `&[Order]`) উভয়কেই অনুমতি দিতে আমরা `?Sized` বাউন্ড ব্যবহার করে `Sized` সীমাবদ্ধতা শিথিল করি:

```rust
pub fn save_json<T: Serialize + ?Sized>(
    &self,
    filename: &str,
    data: &T,
) -> Result<PathBuf, StoreError> {
    // এখন &Catalog এবং &[Order] উভয়ই এখানে পাস করা যাবে!
}
```

---

### ৫. ক্র্যাশ সুরক্ষার জন্য অ্যাটমিক রাইট (Atomic Writes)

ফাইলে লেখার সময় যদি কম্পিউটার হঠাৎ বন্ধ হয়ে যায় বা বিদ্যুৎ চলে যায়, তখন কী ঘটবে?
আপনি যদি সরাসরি মূল `catalog.json` ফাইলে লিখতে থাকেন, তবে ডিস্কে একটি অর্ধ-লিখিত, ভাঙা ফাইল থেকে যাবে যা পরবর্তীতে আর কখনোই ডিসিরিয়ালাইজ করা যাবে না!

এই ডেটা করাপশন রোধ করতে প্রোডাকশন সিস্টেমে **অ্যাটমিক রিনেম সেম্যান্টিকস (Atomic Rename)** ব্যবহার করা হয়:
1. মেমরিতে ডেটাকে সুন্দর JSON টেক্সটে রূপান্তর করা হয়।
2. প্রথমে একটি অস্থায়ী ফাইলে লেখা হয় (`catalog.json.tmp`)।
3. লেখা শতভাগ সফল হলে `std::fs::rename` দিয়ে চোখের পলকে মূল `catalog.json` ফাইলটিকে প্রতিস্থাপন করা হয়।

POSIX এবং উইন্ডোজ ফাইলসিস্টেমে একই ডিরেক্টরির ভেতর ফাইল রিনেম করা একটি অ্যাটমিক মেটাডেটা অপারেশন। ফলে ডিস্কের ফাইলটি হয় শতভাগ পুরোনো সংস্করণে থাকবে, অথবা শতভাগ নতুন সংস্করণে রূপান্তরিত হবে—মাঝামাঝি কোনো ভাঙা বা করাপ্ট অবস্থা তৈরি হতে পারে না!

```
[মেমরিতে রূপান্তর]
        │
[টেম্প ফাইলে রাইট]  ──► catalog.json.tmp (সম্পূর্ণ লেখা হলো)
        │
[অ্যাটমিক রিনেম]   ──► catalog.json (এক নিমিষে মূল ফাইলে রূপান্তর)
```

---

## মিনিস্টোরে পারসিস্টেন্স ইমপ্লিমেন্টেশন

### ১. ডোমেন এরর এক্সটেনশন

`ministore/src/error.rs`-এ আই/ও এবং জেএসন পার্সিং এরর হ্যান্ডেল করার জন্য আমরা দুটি নতুন ভ্যারিয়্যান্ট যোগ করেছি:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum StoreError {
    // ... পূর্ববর্তী ভ্যারিয়্যান্টসমূহ ...
    IoError { path: String, message: String },
    SerializationError { message: String },
}
```

### ২. `StorePersistence` সার্ভিস

`ministore/src/persistence.rs`-এ তৈরি করা হয়েছে স্টোরেজ সার্ভিস:

```rust
use crate::catalog::Catalog;
use crate::error::StoreError;
use crate::models::{Customer, Order};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreSnapshot {
    pub catalog: Catalog,
    pub customers: Vec<Customer>,
    pub orders: Vec<Order>,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct StorePersistence {
    base_dir: PathBuf,
}

impl StorePersistence {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    pub fn save_json<T: Serialize + ?Sized>(
        &self,
        filename: &str,
        data: &T,
    ) -> Result<PathBuf, StoreError> {
        let dir = &self.base_dir;
        fs::create_dir_all(dir).map_err(|e| StoreError::IoError {
            path: dir.display().to_string(),
            message: format!("ফোল্ডার তৈরিতে ব্যর্থ: {e}"),
        })?;

        let target_path = dir.join(filename);
        let temp_path = dir.join(format!("{filename}.tmp"));

        let json_text = serde_json::to_string_pretty(data).map_err(|e| {
            StoreError::SerializationError {
                message: format!("{filename} সিরিয়ালাইজ করতে ব্যর্থ: {e}"),
            }
        })?;

        fs::write(&temp_path, json_text.as_bytes()).map_err(|e| StoreError::IoError {
            path: temp_path.display().to_string(),
            message: format!("টেম্পোরারি ফাইলে লিখতে ব্যর্থ: {e}"),
        })?;

        fs::rename(&temp_path, &target_path).map_err(|e| StoreError::IoError {
            path: target_path.display().to_string(),
            message: format!("ফাইল রিনেম ব্যর্থ: {e}"),
        })?;

        Ok(target_path)
    }

    pub fn load_json<T: serde::de::DeserializeOwned>(
        &self,
        filename: &str,
    ) -> Result<T, StoreError> {
        let file_path = self.base_dir.join(filename);
        let content = fs::read_to_string(&file_path).map_err(|e| StoreError::IoError {
            path: file_path.display().to_string(),
            message: format!("ফাইল পড়তে ব্যর্থ: {e}"),
        })?;

        serde_json::from_str(&content).map_err(|e| StoreError::SerializationError {
            message: format!("{filename} ডিসিরিয়ালাইজ করতে ব্যর্থ: {e}"),
        })
    }

    pub fn save_catalog(&self, catalog: &Catalog) -> Result<PathBuf, StoreError> {
        self.save_json("catalog.json", catalog)
    }

    pub fn load_catalog(&self) -> Result<Catalog, StoreError> {
        self.load_json("catalog.json")
    }

    pub fn save_orders(&self, orders: &[Order]) -> Result<PathBuf, StoreError> {
        self.save_json("orders.json", orders)
    }

    pub fn load_orders(&self) -> Result<Vec<Order>, StoreError> {
        self.load_json("orders.json")
    }

    pub fn export_snapshot(&self, snapshot: &StoreSnapshot) -> Result<PathBuf, StoreError> {
        self.save_json("snapshot.json", snapshot)
    }

    pub fn import_snapshot(&self) -> Result<StoreSnapshot, StoreError> {
        self.load_json("snapshot.json")
    }
}
```

---

## কোড পরীক্ষা ও ভ্যালিডেশন

মিনিস্টোরের সমস্ত পারসিস্টেন্স ফিচার `src/lib.rs`-এ টেস্ট কেসের মাধ্যমে যাচাইকৃত:

```bash
cargo test
```

প্রধান টেস্ট কেসসমূহ:
1. **`test_product_and_category_json_roundtrip`**: প্রোডাক্ট ও ক্যাটাগরি সফলভাবে JSON-এ রূপান্তর এবং পুনরায় রিস্টোর হওয়া নিশ্চিত করে।
2. **`test_order_and_customer_json_roundtrip`**: জটিল নেস্টেড মডেল (যেমন `HashSet<String>` এবং `OrderStatus`) রাউন্ডট্রিপ ভ্যালিডেট করে।
3. **`test_store_persistence_catalog_file_io`**: টেম্পোরারি ডিরেক্টরিতে ক্যাটালগ সেভ এবং লোড করে ডেটার নিখুঁত সমতা পরীক্ষা করে।
4. **`test_store_persistence_snapshot_export_import`**: পুরো স্টোরের স্ন্যাপশট এক্সপোর্ট ও ইমপোর্ট করার ক্ষমতা যাচাই করে।
5. **`test_store_persistence_errors`**: ফাইল অনুপস্থিত থাকলে `StoreError::IoError` এবং করাপ্ট JSON থাকলে `StoreError::SerializationError` সঠিকভাবে ফেরত আসে কিনা তা নিশ্চিত করে।

---

## সারসংক্ষেপ ও চেকপয়েন্ট

২২তম অধ্যায় সম্পন্ন করার মাধ্যমে মিনিস্টোরের ডেটা এখন মেমরির অস্থায়িত্ব কাটিয়ে দীর্ঘস্থায়ী সুরক্ষার রূপ নিয়েছে। আপনি শিখেছেন:
- স্ট্যান্ডার্ড লাইব্রেরি ফাইল আই/ও (`std::fs`, `std::path::PathBuf`)।
- `serde` ও `serde_json` দিয়ে শক্তিশালী টাইপ কনভার্সন।
- আনসাইজড স্লাইস সমর্থনের জন্য `?Sized` ট্রেইট বাউন্ড।
- টেম্পোরারি ফাইল ও রিনেমের মাধ্যমে ক্র্যাশ-সেফ অ্যাটমিক রাইট।
- পুরো স্টোরের রিস্টোরের জন্য পয়েন্ট-ইন-টাইম ব্যাকআপ স্ন্যাপশট।

পরবর্তী অধ্যায়ে আমরা মিনিস্টোরকে এক্সটার্নাল ক্লায়েন্টদের সাথে সংযুক্ত করতে আধুনিক ওয়েব ফ্রেমওয়ার্ক অ্যাক্সাম (Axum) ব্যবহার করে একটি **রাস্ট ওয়েব এপিআই (Web API)** তৈরি করব!
