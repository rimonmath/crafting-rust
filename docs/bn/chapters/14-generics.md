# অধ্যায় ১৪: জেনেরিকস (Generics)

## আপনি যা শিখবেন
- **জেনেরিকস (Generics)** কী এবং কোডের পুনরাবৃত্তি না ঘটিয়ে কীভাবে টাইপ-সেফ ও পুনর্ব্যবহারযোগ্য কোড লেখা যায়।
- টাইপ প্যারামিটার সিনট্যাক্স (`fn foo<T>(item: T)`) ব্যবহার করে **জেনেরিক ফাংশন (Generic Functions)** তৈরি ও কল করার নিয়ম।
- যেকোনো ধরণের ডাটা ধারণ করতে সক্ষম **জেনেরিক স্ট্রাক্ট (Generic Structs)** (যেমন: `Page<T>` বা `Pair<K, V>`) ডিজাইন করা।
- কীভাবে জেনেরিক টাইপ স্ট্যান্ডার্ড লাইব্রেরির **`Option<T>`** এবং **`Result<T, E>`** এনামগুলোকে শক্তিশালী করেছে এবং কীভাবে কাস্টম জেনেরিক এনাম যেমন **`ApiResponse<T>`** লিখতে হয়।
- `impl<T> StructName<T>` দিয়ে জেনেরিক টাইপের ওপর মেথড ইমপ্লিমেন্ট করা এবং মেথডে নতুন টাইপ প্যারামিটার যুক্ত করা (`fn map<U>(self, f: fn(T) -> U) -> Page<U>`)।
- রাস্ট কীভাবে কম্পাইল টাইমে **মনোমর্ফাইজেশন (Monomorphization)**-এর মাধ্যমে **জিরো-কস্ট অ্যাবস্ট্রাকশন (Zero-Cost Abstractions)** অর্জন করে: কোনো রানটাইম ওভারহেড বা অবজেক্ট বক্সিং ছাড়াই প্রতিটি নির্দিষ্ট টাইপের জন্য ডেডিকেটেড মেশিন কোড তৈরি করা।
- MiniStore অ্যাপ্লিকেশনে জেনেরিকস প্রয়োগ:
  - একটি সার্বজনীন পুনর্ব্যবহারযোগ্য **`Page<T>`** কন্টেইনার তৈরি করা।
  - একটি জেনেরিক **`paginate<T>`** ফাংশন লেখা যা যেকোনো কালেকশনকে পৃষ্ঠায় (Page) বিভক্ত করতে পারে।
  - ক্যাটালগে প্রোডাক্ট পেজিনেশন যুক্ত করা (`catalog.paginate(page, per_page)`)।
  - ই-কমার্স এপিআই রেসপন্সের জন্য একটি জেনেরিক **`ApiResponse<T>`** র‍্যাপার তৈরি করা।

---

## কেন আমাদের এটি প্রয়োজন?
MiniStore-এর মতো একটি বাস্তবধর্মী ই-কমার্স প্ল্যাটফর্মের সাধারণ কিছু চাহিদার কথা ভাবুন:
১. ক্রেতাদের বিভিন্ন পৃষ্ঠায় প্রোডাক্ট ব্রাউজ করতে হবে (`ProductPage`)।
২. ওয়্যারহাউস কর্মীদের বিভিন্ন পেজে বিভক্ত অর্ডারের তালিকা দেখতে হবে (`OrderPage`)।
৩. স্টোর অ্যাডমিনিস্ট্রেটরদের পেজ আকারে কাস্টমার অ্যাকাউন্ট অডিট করতে হবে (`CustomerPage`)।

### সমস্যা: কোডের পুনরাবৃত্তি (Code Duplication)
জেনেরিকস ছাড়া আমরা কীভাবে এটি লিখতাম? আমাদের প্রতিটি আলাদা টাইপের জন্য সম্পূর্ণ আলাদা স্ট্রাক্ট তৈরি করতে হতো:

```rust
// প্রোডাক্টের পেজ
pub struct ProductPage {
    pub items: Vec<Product>,
    pub page: usize,
    pub per_page: usize,
    pub total_items: usize,
}

// অর্ডারের পেজ
pub struct OrderPage {
    pub items: Vec<Order>,
    pub page: usize,
    pub per_page: usize,
    pub total_items: usize,
}

// কাস্টমারদের পেজ
pub struct CustomerPage {
    pub items: Vec<Customer>,
    pub page: usize,
    pub per_page: usize,
    pub total_items: usize,
}
```

সমস্যাটি লক্ষ্য করুন:
- পেজিনেশনের লজিক (`total_pages()`, `has_next()`, `has_previous()`, স্লাইসিং ইত্যাদি) তিনটি স্ট্রাক্টেই **হুবহু এক**।
- যদি আমাদের ১০টি আলাদা টাইপ পেজিনেট করতে হয়, তবে আমাদের ১০টি কপি-পেস্ট করা স্ট্রাক্ট এবং ১০ সেট অভিন্ন মেথড লিখতে ও মেইনটেইন করতে হবে!
- যদি `total_pages()` গণনায় কোনো বাগ পাওয়া যায়, তবে আমাদের ১০টি ভিন্ন ভিন্ন জায়গায় গিয়ে ম্যানুয়ালি কোড ঠিক করতে হবে।

### প্রচলিত ডায়নামিক সমাধান (এবং রাস্ট কেন তা বর্জন করেছে)
পাইথন বা জাভাস্ক্রিপ্টের মতো ডায়নামিক ভাষায় ফাংশন যেকোনো ডাটা গ্রহণ করতে পারে, কিন্তু সেখানে কম্পাইল টাইমে কোনো টাইপ সুরক্ষাই থাকে না।

জাভার পুরোনো সংস্করণ বা সি ল্যাঙ্গুয়েজে ডেভেলপাররা `void*` বা `Object`-এর মতো বেস পয়েন্টার ব্যবহার করতেন:
```java
// জাভার আদিম অবজেক্ট বক্সিং
public class Page {
    public Object[] items;
    // ...
}
```
এই পদ্ধতির দুটি মারাত্মক ত্রুটি রয়েছে:
১. **টাইপ সেফটি নষ্ট হওয়া**: আপনি ভুলবশত প্রোডাক্ট পেজের ভেতর একটি `Customer` অবজেক্ট ঢুকিয়ে দিলেও কম্পাইলার ধরতে পারবে না। প্রোগ্রাম যখন চলবে, তখন রানটাইমে `ClassCastException` দিয়ে ক্র্যাশ করবে।
২. **রানটাইম মেমরি ওভারহেড**: সাধারণ প্রিমিটিভ টাইপকে হিপে অবজেক্ট আকারে "বক্স" করতে হয়, ফলে পয়েন্টার ট্রাভার্সাল এবং ক্যাশ মিসের কারণে গতি কমে যায়।

**রাস্টের সমাধান**: **জেনেরিকস (Generics)।**
জেনেরিকসের মাধ্যমে আপনি একটিমাত্র টাইপ প্লেসহোল্ডার (যেমন: `T`) দিয়ে অ্যালগরিদম ও ডাটা স্ট্রাকচার লিখতে পারেন। কম্পাইলার সম্পূর্ণ কম্পাইল-টাইম টাইপ সুরক্ষা প্রদান করে এবং কোনো অবজেক্ট বক্সিং ছাড়াই **সর্বোচ্চ রানটাইম পারফরম্যান্স** নিশ্চিত করে।

---

## মূল রাস্ট কনসেপ্ট: জেনেরিকস

জেনেরিকস আমাদের নির্দিষ্ট কোনো কংক্রিট টাইপের (যেমন: `u32`, `String`, বা `Product`) পরিবর্তে একটি বিমূর্ত টাইপ প্যারামিটার ব্যবহার করার সুবিধা দেয়, যাকে কনভেনশন অনুযায়ী `T` (Type-এর সংক্ষিপ্ত রূপ) বলা হয়।

### ১. জেনেরিক ফাংশন (Generic Functions)
কোনো ফাংশনকে জেনেরিক করতে হলে ফাংশনের নামের ঠিক পরে এবং প্যারামিটার তালিকার আগে অ্যাঙ্গেল ব্র্যাকেটের ভেতর টাইপ প্যারামিটার `<T>` ঘোষণা করতে হয়:

```rust
// কংক্রিট ফাংশন: কেবল i32-এর জন্য কাজ করে
fn first_i32(list: &[i32]) -> Option<&i32> {
    list.first()
}

// জেনেরিক ফাংশন: যেকোনো টাইপ T-এর জন্য কাজ করে!
fn first_element<T>(list: &[T]) -> Option<&T> {
    list.first()
}

fn main() {
    let numbers = vec![10, 20, 30];
    let names = vec![String::from("Alice"), String::from("Bob")];

    // রাস্ট কম্পাইলার স্বয়ংক্রিয়ভাবে বুঝে নেয় T = i32
    let num = first_element(&numbers);

    // রাস্ট কম্পাইলার স্বয়ংক্রিয়ভাবে বুঝে নেয় T = String
    let name = first_element(&names);
}
```

লক্ষ্য করুন:
- `T` হলো কলার কর্তৃক প্রদত্ত যেকোনো টাইপ।
- কল করার সময় আপনাকে সাধারণত `first_element::<i32>(&numbers)` স্পষ্টভাবে লিখতে হয় না (যদিও টার্বোফিশ `::<T>` সিনট্যাক্স সমর্থিত); রাস্ট কম্পাইলার পাস করা আর্গুমেন্ট দেখেই নিজে থেকে `T`-এর টাইপ অনুমান করে নেয়।

---

## জেনেরিক স্ট্রাক্ট (Generic Structs)

সার্বজনীন কন্টেইনার তৈরি করতে স্ট্রাক্ট ডেফিনিশনে জেনেরিক টাইপ প্যারামিটার ব্যবহার করা হয়:

```rust
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: usize,
    pub per_page: usize,
    pub total_items: usize,
}
```

এখন, `Page<Product>`, `Page<Order>`, `Page<Customer>`, এমনকি `Page<i32>` সবাই একই সংজ্ঞা ভাগাভাগি করে:
```rust
let product_page: Page<Product> = Page {
    items: vec![/* products */],
    page: 1,
    per_page: 10,
    total_items: 45,
};

let number_page: Page<u32> = Page {
    items: vec![1, 2, 3],
    page: 1,
    per_page: 3,
    total_items: 3,
};
```

### একাধিক টাইপ প্যারামিটার
যদি কোনো স্ট্রাক্টে একাধিক প্লেসহোল্ডার টাইপের প্রয়োজন হয়, তবে কমা দিয়ে একাধিক টাইপ ঘোষণা করা যায়:

```rust
pub struct KeyValue<K, V> {
    pub key: K,
    pub value: V,
}

fn main() {
    // K হলো String, V হলো u32
    let item_price = KeyValue {
        key: String::from("TECH-KEY-001"),
        value: 12000,
    };

    // K হলো u64 (ID), V হলো bool (অ্যাক্টিভ স্ট্যাটাস)
    let user_status = KeyValue {
        key: 301,
        value: true,
    };
}
```

---

## জেনেরিক এনাম: আপনি যা ইতিমধ্যে ব্যবহার করেছেন!

জেনেরিকস যদি আপনার কাছে নতুন মনে হয়, তবে জেনে রাখুন আপনি আসলে অধ্যায় ১১ ও ১২ থেকেই এটি ব্যবহার করছেন!

স্ট্যান্ডার্ড লাইব্রেরির `Option` এবং `Result`-এর ডেফিনিশন লক্ষ্য করুন:
```rust
// Option-এর একটি জেনেরিক টাইপ প্যারামিটার রয়েছে: T
pub enum Option<T> {
    Some(T),
    None,
}

// Result-এর দুটি জেনেরিক টাইপ প্যারামিটার রয়েছে: T (সফলতা) এবং E (এরর)
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
```
`Option<T>` জেনেরিক হওয়ার কারণেই এটি আলাদা এনাম না বানিয়েই একটি ইন্টিজার (`Option<u32>`), একজন কাস্টমার (`Option<Customer>`), বা একটি কার্ট (`Option<ShoppingCart>`) সংরক্ষণ করতে পারে।

### কাস্টম জেনেরিক এনাম
MiniStore-এ এপিআই রেসপন্সের জন্য আমরা একটি জেনেরিক এনাম তৈরি করতে পারি:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiResponse<T> {
    Success { data: T, total: usize },
    Error { message: String },
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T, total: usize) -> Self {
        Self::Success { data, total }
    }

    pub fn err(message: String) -> Self {
        Self::Error { message }
    }
}
```

---

## জেনেরিক টাইপের ওপর মেথড (`impl<T>`)

কোনো জেনেরিক স্ট্রাক্টের জন্য মেথড ব্লক লেখার সময় `impl`-এর ঠিক পরেই `<T>` ঘোষণা করতে হয়, যাতে রাস্ট বুঝতে পারে যে `T` কোনো কংক্রিট টাইপ নয়, বরং জেনেরিক টাইপ:

```rust
impl<T> Page<T> {
    pub fn new(items: Vec<T>, page: usize, per_page: usize, total_items: usize) -> Self {
        Self { items, page, per_page, total_items }
    }

    pub fn total_pages(&self) -> usize {
        if self.per_page == 0 {
            0
        } else {
            self.total_items.div_ceil(self.per_page)
        }
    }

    pub fn has_next(&self) -> bool {
        self.page > 0 && self.page < self.total_pages()
    }

    pub fn has_previous(&self) -> bool {
        self.page > 1 && self.page <= self.total_pages() + 1
    }
}
```

### মেথডে নতুন টাইপ প্যারামিটার যুক্ত করা
কোনো জেনেরিক স্ট্রাক্ট `Page<T>`-এর মেথড তার নিজস্ব অতিরিক্ত জেনেরিক প্যারামিটার `U` ঘোষণা করতে পারে।

উদাহরণস্বরূপ, একটি রূপান্তর ফাংশন দিয়ে `Page<T>`-কে `Page<U>`-তে রূপান্তর করা:
```rust
impl<T> Page<T> {
    /// একটি ট্রান্সফর্ম ফাংশন দিয়ে Page<T>-কে Page<U>-তে রূপান্তর করে
    pub fn map<U>(self, transform: fn(T) -> U) -> Page<U> {
        let mapped_items: Vec<U> = self.items.into_iter().map(transform).collect();
        Page {
            items: mapped_items,
            page: self.page,
            per_page: self.per_page,
            total_items: self.total_items,
        }
    }
}
```
এখানে `T` স্ট্রাক্ট লেভেলে সংজ্ঞায়িত, আর `U` বিশেষভাবে `map` মেথডের জন্য ঘোষিত হয়েছে!

---

## মানসিক মডেল: মনোমর্ফাইজেশন (Zero-Cost Abstractions)

রাস্ট কীভাবে জেনেরিক কোড এত দ্রুততম গতিতে রান করে?

জাভা বা পাইথনের মতো ভাষায় জেনেরিক্সে রানটাইম পেনাল্টি দিতে হয়:
- **জাভা**: *Type Erasure* ব্যবহার করে। কম্পাইলের সময় `List<Integer>` এবং `List<String>` উভয়ই মুছে গিয়ে `List<Object>` হয়ে যায়। এর ফলে ডাটা বক্সিং এবং প্রতিবার কাস্টিং করতে হয়।
- **পাইথন**: প্রতিটি নির্দেশনায় ডায়নামিকভাবে টাইপ যাচাই করে, যার ফলে ইন্টারপ্রেটারের প্রচুর ওভারহেড হয়।

### রাস্ট কীভাবে জেনেরিকস কম্পাইল করে
রাস্ট **মনোমর্ফাইজেশন (Monomorphization)** নামক একটি বিশেষ পদ্ধতি ব্যবহার করে (অর্থাৎ জেনেরিক কোডকে মনো-মর্ফিক বা একক-রূপের নির্দিষ্ট কোডে রূপান্তর করা):

```
                       জেনেরিক সোর্স কোড:
                            Page<T>
                               │
                  কম্পাইল-টাইম মনোমর্ফাইজেশন
                               │
                ┌──────────────┴──────────────┐
                ▼                             ▼
   T = Product-এর জন্য তৈরি:       T = i32-এর জন্য তৈরি:
        struct Page_Product {          struct Page_i32 {
            items: Vec<Product>,           items: Vec<i32>,
            page: usize,                   page: usize,
            ...                            ...
        }                              }
```

১. কম্পাইল করার সময় `rustc` আপনার পুরো প্রজেক্ট স্ক্যান করে দেখে `Page<T>` কোন কোন নির্দিষ্ট টাইপের সাথে ব্যবহার করা হয়েছে।
২. ব্যবহৃত প্রতিটি কংক্রিট টাইপের জন্য (যেমন: `Page<Product>` এবং `Page<i32>`), কম্পাইলার পর্দার আড়ালে স্ট্রাক্ট এবং তার মেথডগুলোর সম্পূর্ণ আলাদা, ডেডিকেটেড কপি তৈরি করে।
৩. প্রতিটি মেথড কল সরাসরি হয় এবং LLVM কম্পাইলার কোড পুরোপুরি ইনলাইন (Inline) করে দিতে পারে।

### সুবিধা ও ট্রেডঅফ
- **রানটাইম পারফরম্যান্স**: **জিরো ওভারহেড (Zero Cost)**। জেনেরিক কোড ঠিক ততটুকুই দ্রুত চলে যতটুকু হাতে লেখা নির্দিষ্ট টাইপের কোড চলত। কোনো অবজেক্ট বক্সিং বা ভার্চুয়াল টেবিল লুকআপ নেই।
- **বাইনারি সাইজ**: যেহেতু কম্পাইলার প্রতিটি টাইপের জন্য কোডের কপি তৈরি করে, তাই কম্পাইল করা বাইনারি সাইজ সামান্য বৃদ্ধি পেতে পারে ("কোড ব্লোট")। তবে অবিশ্বাস্য গতি ও মেমরি নিরাপত্তার তুলনায় এই ট্রেডঅফ অত্যন্ত নগণ্য।

---

## বিভিন্ন ভাষায় জেনেরিকসের তুলনা

| ভাষা | কৌশল | টাইপ সেফটি | রানটাইম ওভারহেড | কোড ব্লোট |
| :--- | :--- | :--- | :--- | :--- |
| **রাস্ট (Rust)** | কম্পাইল টাইমে **মনোমর্ফাইজেশন** | কোড তৈরির আগেই শতভাগ পরীক্ষিত | **জিরো কস্ট** (সরাসরি কল, ইনলাইনিং) | বাইনারি আকারে সামান্য বৃদ্ধি |
| **সি++ (C++)** | টেমপ্লেট (ইনস্ট্যানশিয়েট করার সময় ডাক টাইপিং) | ইনস্ট্যানশিয়েট করার সময় চেক হয় | জিরো কস্ট | টেমপ্লেট ব্লোট বেশি |
| **জাভা (Java)** | টাইপ ইরেজার (`Object`-এ রূপান্তর) | কম্পাইল টাইমে চেক, কিন্তু রানটাইমে কাস্টিং | বেশি (বক্সিং, পয়েন্টার ক্যাশ মিস) | নেই (একক ক্লাস ফাইল) |
| **গো (Go 1.18+)** | GC-শেপ ডিকশনারি পাসিং | কম্পাইল টাইমে চেক | সামান্য থেকে মাঝারি (ডিকশনারি লুকআপ) | কম |
| **টাইপস্ক্রিপ্ট** | টাইপ ইরেজার (প্লেইন জেএস-এ কনভার্ট) | কেবল কম্পাইল টাইমে (`any` দিয়ে বাইপাস সম্ভব) | জেএস-এর সাধারণ ডায়নামিক খরচ | নেই |

---

## MiniStore-এ বাস্তবায়ন

MiniStore-এ আমরা জেনেরিক পেজিনেশন এবং এপিআই রেসপন্স মডিউল যুক্ত করেছি।

### ১. `src/models/page.rs`
`Page<T>`, সার্বজনীন `paginate<T>` স্লাইসিং ফাংশন এবং `ApiResponse<T>` সংজ্ঞায়িত:

```rust
/// MiniStore-এর জন্য সার্বজনীন জেনেরিক পেজিনেশন কন্টেইনার।
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: usize,
    pub per_page: usize,
    pub total_items: usize,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, page: usize, per_page: usize, total_items: usize) -> Self {
        Self {
            items,
            page,
            per_page,
            total_items,
        }
    }

    pub fn total_pages(&self) -> usize {
        if self.per_page == 0 {
            0
        } else {
            self.total_items.div_ceil(self.per_page)
        }
    }

    pub fn has_next(&self) -> bool {
        self.page > 0 && self.page < self.total_pages()
    }

    pub fn has_previous(&self) -> bool {
        self.page > 1 && self.page <= self.total_pages() + 1
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// ফাংশন পয়েন্টার ব্যবহার করে একটি `Page<T>`-কে `Page<U>`-তে রূপান্তর করে।
    pub fn map<U>(self, transform: fn(T) -> U) -> Page<U> {
        let mapped_items = self.items.into_iter().map(transform).collect();
        Page {
            items: mapped_items,
            page: self.page,
            per_page: self.per_page,
            total_items: self.total_items,
        }
    }
}

/// ওনড (Owned) ভেক্টরকে পৃষ্ঠায় বিভক্তকারী জেনেরিক ফাংশন।
pub fn paginate<T>(items: Vec<T>, page: usize, per_page: usize) -> Page<T> {
    let total_items = items.len();
    if per_page == 0 || page == 0 {
        return Page::new(Vec::new(), page, per_page, total_items);
    }

    let start_index = (page - 1) * per_page;
    if start_index >= total_items {
        return Page::new(Vec::new(), page, per_page, total_items);
    }

    let end_index = (start_index + per_page).min(total_items);
    let page_items: Vec<T> = items
        .into_iter()
        .skip(start_index)
        .take(end_index - start_index)
        .collect();

    Page::new(page_items, page, per_page, total_items)
}

/// জেনেরিক এপিআই রেসপন্স র‍্যাপার এনাম।
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiResponse<T> {
    Success { data: T, total: usize },
    Error { message: String },
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T, total: usize) -> Self {
        Self::Success { data, total }
    }

    pub fn err(message: String) -> Self {
        Self::Error { message }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    pub fn data(&self) -> Option<&T> {
        match self {
            Self::Success { data, .. } => Some(data),
            Self::Error { .. } => None,
        }
    }
}
```

### ২. `src/catalog.rs` আপডেট
ক্যাটালগে পেজিনেশন মেথড যোগ করা হয়েছে:

```rust
impl Catalog {
    // ...

    /// ক্যাটালগের সমস্ত প্রোডাক্ট আইডি অনুসারে সাজিয়ে ভেক্টর রিটার্ন করে।
    pub fn get_products(&self) -> Vec<Product> {
        let mut list: Vec<Product> = self.products.values().cloned().collect();
        list.sort_by_key(|p| p.id);
        list
    }

    /// জেনেরিক `Page<T>` ব্যবহার করে প্রোডাক্ট পেজিনেট করে।
    pub fn paginate(&self, page: usize, per_page: usize) -> crate::models::Page<Product> {
        crate::models::paginate(self.get_products(), page, per_page)
    }
}
```

### ৩. `src/main.rs`-এ জেনেরিকসের বাস্তব ডেমো

```rust
use ministore::{
    checkout, ApiResponse, Catalog, Coupon, Customer, OrderId, Page, PaymentMethod, Product,
    ProductCategory, ShoppingCart,
};

fn main() {
    println!("=== MiniStore: Generics & Monomorphization (Part III) ===\n");

    // ১. প্রোডাক্ট ক্যাটালগ শুরু করা
    let mut catalog = Catalog::new();
    catalog.add_product(Product::new(101, String::from("TECH-KEY-001"), String::from("Tenkeyless Mechanical Keyboard"), ProductCategory::Electronics, 12000, 5));
    catalog.add_product(Product::new(102, String::from("TECH-MOU-002"), String::from("Ergonomic Wireless Mouse"), ProductCategory::Electronics, 4500, 10));
    catalog.add_product(Product::new(103, String::from("TECH-MON-003"), String::from("27-inch 4K IPS Display"), ProductCategory::Electronics, 35000, 3));

    println!("1. Catalog initialized with {} products.", catalog.total_products());

    // ২. জেনেরিক পেজিনেশন ডেমো (Page<Product>)
    println!("\n2. Browsing Catalog with Generic Pagination (Page<Product>):");
    let product_page: Page<Product> = catalog.paginate(1, 2);
    println!(
        "   Page {} of {} (Total Items: {})",
        product_page.page,
        product_page.total_pages(),
        product_page.total_items
    );
    for item in &product_page.items {
        println!(
            "   - [{}] {} (${:.2})",
            item.sku, item.name, item.price_cents as f64 / 100.0
        );
    }
    println!("   Has next page? {}", product_page.has_next());

    // জেনেরিক ট্রান্সফরমেশন: Page<Product> -> Page<String>
    let name_page: Page<String> = product_page.map(|p| p.name);
    println!("   Transformed to Page<String>: {:?}", name_page.items);

    // জেনেরিক এপিআই রেসপন্স কনটেইনার
    let api_response = ApiResponse::ok(catalog.paginate(2, 2), catalog.total_products());
    if let ApiResponse::Success { data, total } = api_response {
        println!(
            "   API Page 2 response: {} product(s) returned out of {} total.",
            data.item_count(),
            total
        );
    }

    // ৩. কাস্টমার প্রোফাইল
    let customer = Customer::new(
        301,
        String::from("Margaret Hamilton"),
        String::from("margaret@apollo.nasa.gov"),
        Some(String::from("+1-555-0199")),
        true,
    );
    println!("\n3. Customer: {} ({})", customer.name, customer.formatted_phone());

    // ৪. শপিং কার্ট
    let mut cart = ShoppingCart::new();
    cart.add_item(101, 1, 12000);
    cart.add_item(102, 2, 4500);

    let coupon = Coupon::new(String::from("LAUNCH20"), 20);

    // ৫. চেকআউট
    println!("\n4. Processing checkout through modular services...");
    match checkout(
        OrderId(901),
        customer,
        &mut cart,
        &mut catalog,
        PaymentMethod::CreditCard { last_four: String::from("9876") },
        Some(coupon),
    ) {
        Ok(mut order) => {
            println!("   Checkout Order #{} created successfully!", order.order_id.0);
            println!(
                "   Subtotal: ${:.2} | Total: ${:.2}",
                order.subtotal_cents() as f64 / 100.0,
                order.total_cents() as f64 / 100.0
            );

            // ৬. অর্ডার লাইফসাইকেল ট্রানজিশন
            println!("\n5. Order Lifecycle Transitions:");
            order.confirm(String::from("REC-901-HAMILTON")).unwrap();
            println!("   Order confirmed: {}", order.status.display_status());

            order.ship(String::from("TRK-FEDEX-77189")).unwrap();
            println!("   Order shipped:   {}", order.status.display_status());

            match order.cancel(String::from("Buyer changed mind")) {
                Ok(()) => println!("   Order cancelled!"),
                Err(err) => println!("   Cancellation prevented -> {}", err.message()),
            }

            order.mark_delivered().unwrap();
            println!("   Final Lifecycle State: {}", order.status.display_status());
        }
        Err(err) => println!("   Checkout failed: {}", err.message()),
    }
}
```

---

## সাধারণ কম্পাইলার এরর এবং সমাধান

### ১. `error[E0412]: cannot find type in this scope`
**ভুল কোড**:
```rust
struct Page {
    items: Vec<T>, // কম্পাইলার জানে না `T` কী জিনিস!
}
```
**সমাধান**:
স্ট্রাক্ট নামের ঠিক পরে `<T>` ডিক্লেয়ার করতে হবে:
```rust
struct Page<T> {
    items: Vec<T>,
}
```

---

### ২. `error[E0107]: struct takes 1 generic argument but 0 generic arguments were supplied`
**ভুল কোড**:
```rust
fn print_page(p: Page) { /* ... */ }
```
**সমাধান**:
একটি জেনেরিক স্ট্রাক্ট নিজে কোনো কংক্রিট টাইপ নয়—এটি একটি টাইপের *ব্লুপ্রিন্ট*। আপনাকে কংক্রিট টাইপ উল্লেখ করতে হবে অথবা ফাংশনটিকেও জেনেরিক রাখতে হবে:
```rust
// কংক্রিট টাইপ উল্লেখ করা:
fn print_page(p: Page<Product>) { /* ... */ }

// অথবা ফাংশনকেও জেনেরিক রাখা:
fn print_page<T>(p: Page<T>) { /* ... */ }
```

---

### ৩. `impl` ব্লকে `<T>` ভুলে যাওয়া
**ভুল কোড**:
```rust
impl Page<T> { // Error: cannot find type `T` in this scope
    fn len(&self) -> usize { self.items.len() }
}
```
**সমাধান**:
সবসময় `impl<T> Page<T>` লিখতে হবে। প্রথম `<T>` টি মেথড ব্লকের জন্য জেনেরিক প্যারামিটার ঘোষণা করে, এবং দ্বিতীয় `Page<T>` নির্দেশ করে এটি কোন স্ট্রাক্টের ওপর প্রযোজ্য:
```rust
impl<T> Page<T> {
    fn len(&self) -> usize { self.items.len() }
}
```

---

## আইডিওম্যাটিক রাস্ট প্র্যাকটিস

১. **টাইপ প্লেসহোল্ডারের জন্য একক ক্যাপিটাল লেটার ব্যবহার করুন**: কনভেনশন অনুসারে জেনেরিক টাইপ সাধারণত একক ক্যাপিটাল লেটার হয় (`T` for Type, `E` for Error, `K` for Key, `V` for Value)। বড় নাম লাগলে CamelCase ব্যবহার করুন (`Item`, `Payload`)।
২. **অকারণে আগে থেকেই অতিরিক্ত জেনেরিক করবেন না**: যখন সত্যিই একাধিক ভিন্ন টাইপের জন্য একই ডাটা স্ট্রাকচার বা অ্যালগরিদমের প্রয়োজন হবে, তখনই কেবল জেনেরিকস ব্যবহার করুন।
৩. **কম্পাইলারের টাইপ ইনফারেন্সের ওপর ভরসা রাখুন**: কম্পাইলারকে নিজে থেকে টাইপ অনুমান করতে দিন। যেখানে কম্পাইলার একা সিদ্ধান্ত নিতে পারে না, কেবল সেখানেই টার্বোফিশ (`parse::<u32>()`) ব্যবহার করুন।
৪. **হাই-পারফরম্যান্স পাথে মনোমর্ফাইজেশনের পূর্ণ সুবিধা নিন**: জেনেরিক মেথডগুলো সরাসরি ইনলাইন হয়ে যাওয়ায় ডায়নামিক ডিসপ্যাচের কোনো ভার্চুয়াল মেথড টেবিল পেনাল্টি থাকে না।

---

## অনুশীলন (Hands-On Exercises)

### অনুশীলন ১: সার্বজনীন সার্চ ফিল্টার
একটি জেনেরিক ফাংশন লিখুন: `filter_by<T>(items: Vec<T>, predicate: fn(&T) -> bool) -> Vec<T>`, যা একটি ভেক্টর গ্রহণ করবে এবং প্রেডিকেট ফাংশন সত্য হলে কেবল সেই আইটেমগুলো নতুন ভেক্টরে রিটার্ন করবে। ইন্টিজার ভেক্টর এবং প্রোডাক্ট ভেক্টর উভয়ের সাথে এটি পরীক্ষা করুন।

### অনুশীলন ২: জেনেরিক `Cache<K, V>` স্ট্রাক্ট
`src/models/cache.rs`-এ `std::collections::HashMap<K, V>` ব্যবহার করে একটি জেনেরিক `Cache<K, V>` স্ট্রাক্ট তৈরি করুন:
- কনস্ট্রাক্টর `Cache::new()` যোগ করুন।
- `insert(&mut self, key: K, value: V)` যোগ করুন।
- `get(&self, key: &K) -> Option<&V>` যোগ করুন।
- কাস্টমার লুকআপ ও প্রোডাক্ট প্রাইস ক্যাশ করে টেস্ট করুন!

---

## চেকপয়েন্ট (Checkpoint)

টেস্ট স্যুট রান করে ১৬টি টেস্ট পাস করার বিষয়টি নিশ্চিত করুন:
```bash
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
cargo run
```

প্রত্যাশিত টেস্ট আউটপুট:
```text
running 16 tests
test tests::test_cart_item_option_lookup ... ok
test tests::test_catalog_option_lookups ... ok
test tests::test_checkout_error_propagation_and_success ... ok
test tests::test_coupon_discount_and_take ... ok
test tests::test_coupon_validation_error ... ok
test tests::test_customer_optional_phone ... ok
test tests::test_generic_api_response_wrapper ... ok
test tests::test_generic_catalog_product_pagination ... ok
test tests::test_generic_page_map_transformation ... ok
test tests::test_generic_pagination_with_integers ... ok
test tests::test_order_cancellation_prevention ... ok
test tests::test_order_status_valid_lifecycle ... ok
test tests::test_order_total_with_payment_fee ... ok
test tests::test_payment_method_fees_and_descriptions ... ok
test tests::test_product_category_tax_rates ... ok
test tests::test_product_stock_reduction_error ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

প্রত্যাশিত বাইনারি আউটপুট:
```text
=== MiniStore: Generics & Monomorphization (Part III) ===

1. Catalog initialized with 3 products.

2. Browsing Catalog with Generic Pagination (Page<Product>):
   Page 1 of 2 (Total Items: 3)
   - [TECH-KEY-001] Tenkeyless Mechanical Keyboard ($120.00)
   - [TECH-MOU-002] Ergonomic Wireless Mouse ($45.00)
   Has next page? true
   Transformed to Page<String>: ["Tenkeyless Mechanical Keyboard", "Ergonomic Wireless Mouse"]
   API Page 2 response: 1 product(s) returned out of 3 total.

3. Customer: Margaret Hamilton (+1-555-0199)

4. Processing checkout through modular services...
   Checkout Order #901 created successfully!
   Subtotal: $210.00 | Total: $148.50

5. Order Lifecycle Transitions:
   Order confirmed: Confirmed (Receipt: REC-901-HAMILTON)
   Order shipped:   Shipped (Tracking: TRK-FEDEX-77189)
   Cancellation prevented -> Cannot perform action 'cancel' while order is in 'Shipped (Tracking: TRK-FEDEX-77189)' state
   Final Lifecycle State: Delivered to Customer
```

---

## আমরা কী শিখলাম
- কম্পাইল-টাইম টাইপ সুরক্ষা অক্ষুণ্ণ রেখেই কোড পুনর্ব্যবহারের জন্য জেনেরিকস কেন অপরিহার্য।
- কীভাবে জেনেরিক ফাংশন (`fn func<T>(...)`), স্ট্রাক্ট (`struct Container<T>`) এবং এনাম (`enum Envelope<T>`) ঘোষণা করতে হয়।
- স্ট্যান্ডার্ড লাইব্রেরির `Option<T>` এবং `Result<T, E>` এনামগুলো কীভাবে জেনেরিকস ব্যবহার করে।
- **মনোমর্ফাইজেশন**-এর অভ্যন্তরীণ মেকানিজম: রাস্ট কীভাবে প্রতিটি কংক্রিট টাইপের জন্য জিরো-কস্ট মেশিন কোড তৈরি করে।
- `impl<T>` দিয়ে মেথড ইমপ্লিমেন্ট করা এবং মেথড লেভেলে নতুন টাইপ প্যারামিটার (`map<U>`) প্রয়োগ।
- MiniStore-এ সার্বজনীন জেনেরিক `Page<T>` এবং `ApiResponse<T>` বাস্তবায়ন।

---

## পরবর্তীতে কী আসছে
এই অধ্যায়ে আমরা শিখলাম একটি জেনেরিক প্যারামিটার `T` মহাবিশ্বের *যেকোনো* টাইপ হতে পারে।
কিন্তু যখন আমাদের প্রয়োজন হয় যে `T`-এর নির্দিষ্ট কিছু ক্ষমতা থাকতে হবে—যেমন প্রিন্ট করা যাবে, তুলনা করা যাবে বা ক্লোন করা যাবে?
**অধ্যায় ১৫: ট্রেইটস (Traits)**-এ আমরা আবিষ্কার করব রাস্টের শেয়ার্ড আচরণ নির্ধারণের শক্তিশালী ট্রেইট সিস্টেম এবং ট্রেইট বাউন্ডসের মাধ্যমে জেনেরিক টাইপকে সীমাবদ্ধ করার নিয়ম!
