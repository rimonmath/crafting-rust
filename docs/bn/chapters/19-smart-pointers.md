# ১৯. স্মার্ট পয়েন্টারস (Smart Pointers)

## আপনি যা শিখবেন
- **স্মার্ট পয়েন্টার (Smart Pointers)** কী এবং সাধারণ রেফারেন্সের (`&T`, `&mut T`) সাথে এদের পার্থক্য কী।
- স্মার্ট পয়েন্টারের মূল ভিত্তি: **`Deref`** এবং **`Drop`** ট্রেইট কীভাবে কাজ করে।
- **`Box<T>`**:
  - স্ট্যাকের পরিবর্তে হিপে (heap) মেমরি বরাদ্দ করা।
  - কম্পাইল-টাইমে সাইজ অজানা থাকা রিকার্সিভ ডেটা স্ট্রাকচার (Recursive Data Structures) তৈরি করা।
  - ট্রেইট অবজেক্ট (`Box<dyn Trait>` এবং `Box<dyn Fn(u32) -> u32>`) ব্যবহার করে ১৮তম অধ্যায়ের ভিন্ন ভিন্ন টাইপের ক্লোজারকে একটি একক ভেক্টরে (`Vec`) সংরক্ষণ করার সমাধান!
- **`Rc<T>` (রেফারেন্স কাউন্টিং - Reference Counting)**:
  - সিঙ্গেল-থ্রেডেড প্রোগ্রামে একই ডেটার একাধিক ওনারশিপ (Multiple Ownership) পরিচালনা।
  - `Rc::clone`-এর মাধ্যমে পয়েন্টার কপি এবং `Rc::strong_count`-এর মাধ্যমে সক্রিয় ওনার সংখ্যা পর্যবেক্ষণ।
- **`RefCell<T>` এবং ইন্টেরিয়র মিউটেবিলিটি (Interior Mutability Pattern)**:
  - একটি ইমিউটেবল রেফারেন্সের (`&T`) পেছনে থেকেও ডেটা মিউটেট (পরিবর্তন) করা।
  - বরো রুলসকে কম্পাইল-টাইমের বদলে রানটাইমে যাচাই করা (`.borrow()` এবং `.borrow_mut()`)।
  - রানটাইম বরো প্যানিক (`BorrowMutError`) কেন হয় এবং কীভাবে তা প্রতিরোধ করবেন।
- **`Rc<RefCell<T>>`-এর সংমিশ্রণ**:
  - একাধিক ওনারের মধ্যে শেয়ার্ড মিউটেবল স্টেট (Shared Mutable State) অর্জন।
- কাস্টম স্মার্ট পয়েন্টার তৈরি: **`Deref`**, **`DerefMut`**, এবং **`Drop`**:
  - ডিরেফ কোয়ের্সন (Deref Coercion)-এর মাধ্যমে মোড়ানো ডেটার মেথড কল করা।
  - স্কোপ শেষ হলে স্বয়ংক্রিয় ক্লিনআপ কোড চালানো।
- মিনিস্টোরে স্মার্ট পয়েন্টারের বাস্তব প্রজেক্ট ইন্টিগ্রেশন:
  - `CategoryNode` এবং `Box<CategoryNode>` দিয়ে রিকার্সিভ ক্যাটাগরি ট্রি।
  - `PromotionPipeline` দিয়ে ভিন্ন ভিন্ন ডিসকাউন্ট ক্লোজারের পাইপলাইন।
  - `Rc<Customer>` দিয়ে একাধিক সেশনের মাঝে গ্রাহকের প্রোফাইল শেয়ারিং।
  - `SharedAuditor` (`Rc<RefCell<DiscountAuditor>>`) দিয়ে গ্লোবাল ডিসকাউন্ট অডিট ট্র্যাকিং।
  - `StoreSession<T>` দিয়ে কাস্টম ডিরেফ ও ড্রপ মেকানিজম।

---

## আমাদের এটি কেন প্রয়োজন?

পূর্ববর্তী অধ্যায়গুলোতে আমরা রাস্টের কঠোর কম্পাইল-টাইম ওনারশিপ এবং বরোয়িং নীতিগুলো আয়ত্ত করেছি:
1. **প্রতিটি ভ্যালুর ঠিক একজন ওনার (Owner) থাকবে।**
2. **একই সময়ে যেকোনো সংখ্যক ইমিউটেবল রেফারেন্স (`&T`) অথবা মাত্র একটি মিউটেবল রেফারেন্স (`&mut T`) থাকতে পারবে।**
3. **কম্পাইল করার সময়েই রাস্টের প্রতিটি ভেরিয়েবলের সঠিক মেমরি সাইজ জানা থাকতে হবে।**

এই নিয়মগুলো মেমরি সেফটি নিশ্চিত করে, কিন্তু বাস্তব সফটওয়্যার আর্কিটেকচারে এমন কিছু ক্ষেত্র আসে যেখানে সাধারণ স্ট্যাক ভ্যালু ও রেফারেন্স দিয়ে কাজ করা সীমাবদ্ধ হয়ে পড়ে:

### সিনারিও ১: ভিন্ন ভিন্ন ক্লোজারের সংকলন তৈরি করা
১৮তম অধ্যায়ে আমরা দেখেছি যে প্রতিটি ক্লোজারের জন্য কম্পাইলার সম্পূর্ণ ইউনিক একটি অনামা টাইপ তৈরি করে। ফলে একটি সাধারণ ভেক্টরে (`Vec<F>`) একাধিক ডিসকাউন্ট নিয়ম একসাথে সংরক্ষণ করা অসম্ভব ছিল:
```rust
// ১৮তম অধ্যায়ে:
let rule_pct = make_percentage_discount(10);
let rule_thresh = make_threshold_discount(10000, 1500);

// কম্পাইলার এরর! দুটি ক্লোজারের টাইপ ভিন্ন:
// let rules = vec![rule_pct, rule_thresh];
```
কীভাবে আমরা একটি একক পাইপলাইনে ভিন্ন ভিন্ন ডিসকাউন্ট স্ট্র্যাটেজি একসাথে রাখতে পারি?

### সিনারিও ২: রিকার্সিভ ডেটা স্ট্রাকচার (Recursive Data Structures)
আমাদের মিনিস্টোরে একটি ক্যাটাগরি হায়ারার্কি প্রয়োজন: "Electronics" ক্যাটাগরির ভেতরে থাকবে "Computers", যার ভেতরে আবার থাকতে পারে "Laptops"। যদি একটি স্ট্রাক্ট সরাসরি নিজের ভেতরে নিজের চাইল্ড ধারণ করতে চায়, তবে কম্পাইলার এর সাইজ নির্ধারণ করতে পারে না (ইনফিনিট সাইজ এরর):
```rust
// কম্পাইলার এরর: recursive type has infinite size
struct CategoryNode {
    name: String,
    subcategories: Vec<CategoryNode>, // ইনডিরেকশন ছাড়া সাইজ অজানা!
}
```

### সিনারিও ৩: শেয়ার্ড রিড-অনলি ওনারশিপ (Shared Ownership)
একটি গ্রাহকের প্রোফাইল (`Customer`) একই সাথে সক্রিয় শপিং কার্ট, চেকআউট লেনদেন এবং অডিট লগে থাকা দরকার। প্রতিবার সম্পূর্ণ `Customer` স্ট্রাক্টটি ডিপ-ক্লন করা মানে অহেতুক হিপ মেমরি নষ্ট করা। আমাদের এমন একটি ব্যবস্থা দরকার যেখানে একাধিক অংশ একই হিপ মেমরির মালিকানা শেয়ার করতে পারবে।

### সিনারিও ৪: শেয়ার্ড মিউটেবল স্টেট (Shared Mutable State)
একাধিক ডিসকাউন্ট যাচাইকারী সিস্টেমের একটি সাধারণ অডিট কাউন্টার (`DiscountAuditor`) আপডেট করা দরকার। কিন্তু রাস্টের সাধারণ নীতি অনুযায়ী একাধিক পয়েন্টার থাকলে মিউটেট করা নিষিদ্ধ। কীভাবে সিঙ্গেল-থ্রেডে নিরাপদে শেয়ার্ড স্টেট মিউটেট করা যাবে?

স্মার্ট পয়েন্টারস এই চারটি সমস্যারই নিখুঁত সমাধান দেয়।

---

## স্মার্ট পয়েন্টার কী? (What is a Smart Pointer?)

সাধারণত **পয়েন্টার** হলো এমন একটি ভেরিয়েবল যা মেমরির কোনো অ্যাড্রেস ধারণ করে। রাস্টে সাধারণ রেফারেন্সগুলো (`&T`, `&mut T`) হলো নন-ওনিং পয়েন্টার, যা কেবল ডেটা ধার (borrow) করে।

**স্মার্ট পয়েন্টার (Smart Pointer)** হলো এমন একটি ডেটা স্ট্রাকচার যা কেবল মেমরি অ্যাড্রেস রাখে না, বরং এর সাথে অতিরিক্ত মেটাডেটা, বিশেষ ক্ষমতা এবং নিজস্ব ওনারশিপ পরিচালনা করে। সাধারণত স্মার্ট পয়েন্টার তার নির্দেশিত ডেটার ওনার হয়।

রাস্টের স্ট্যান্ডার্ড লাইব্রেরির স্মার্ট পয়েন্টারগুলো দুটি মৌলিক ট্রেইটের উপর ভিত্তি করে গড়ে উঠেছে:
- **`Deref`** (এবং `DerefMut`): পয়েন্টারকে সাধারণ রেফারেন্সের মতো আচরণ করতে দেয়, যার ফলে `*ptr` অপারেটর এবং স্বয়ংক্রিয় **ডিরেফ কোয়ের্সন (Deref Coercion)** কাজ করে।
- **`Drop`**: পয়েন্টার যখন স্কোপের বাইরে চলে যায়, তখন স্বয়ংক্রিয়ভাবে ক্লিনআপ কোড এক্সিকিউট করে মেমরি ও রিসোর্স মুক্ত করে (RAII নীতি)।

| টাইপ | মেমরি অবস্থান | ওনারশিপ | মিউটেবিলিটি যাচাই | ব্যবহারের ক্ষেত্র |
| :--- | :--- | :--- | :--- | :--- |
| `&T` | স্ট্যাক বা হিপ | ধার নেওয়া (Borrow) | কম্পাইল-টাইম (ইমিউটেবল) | স্বল্পমেয়াদী রিড অ্যাক্সেস |
| `&mut T` | স্ট্যাক বা হিপ | ধার নেওয়া (Borrow) | কম্পাইল-টাইম (এক্সক্লুসিভ মিউটেবল) | একক মিউটেশন অ্যাক্সেস |
| `Box<T>` | হিপ (Heap) | একক মালিক (Single) | কম্পাইল-টাইম | নির্দিষ্ট সাইজ, রিকার্সন, ট্রেইট অবজেক্ট |
| `Rc<T>` | হিপ (Heap) | একাধিক মালিক (Ref-counted) | কম্পাইল-টাইম (ইমিউটেবল) | সিঙ্গেল-থ্রেডে শেয়ার্ড রিড |
| `RefCell<T>` | স্ট্যাক বা হিপ | একক মালিক | **রানটাইম** (ইন্টেরিয়র মিউটেবিলিটি) | ইমিউটেবল রেফারেন্স দিয়ে মিউটেট করা |
| `Rc<RefCell<T>>` | হিপ (Heap) | একাধিক মালিক | **রানটাইম** | একাধিক মালিকের মধ্যে শেয়ার্ড মিউটেশন |

---

## `Box<T>`: হিপ অ্যালোকেশন এবং ডায়নামিক সাইজিং

`Box<T>` হলো রাস্টের সবচেয়ে সরল ও মৌলিক স্মার্ট পয়েন্টার। এটি ডেটাকে হিপে জমা রাখে, আর স্ট্যাকে কেবল একটি নির্দিষ্ট সাইজের পয়েন্টার (৬৪-বিট সিস্টেমে মাত্র ৮ বাইট) সংরক্ষণ করে:

```rust
// ৪২ সংখ্যাটি হিপে সংরক্ষিত হবে; `boxed_val` স্ট্যাকে থাকবে
let boxed_val: Box<u32> = Box::new(42);
assert_eq!(*boxed_val, 42); // ডিরেফারেন্স করে মান পাওয়া যায়
```

### ১. রিকার্সিভ ডেটা স্ট্রাকচার (Recursive Data Structures)
রাস্টে প্রতিটি স্ট্রাক্টের সাইজ কম্পাইল-টাইমে জানা থাকা বাধ্যতামূলক। রিকার্সিভ স্ট্রাক্ট (যে স্ট্রাক্ট নিজের ভেতরে নিজেকে ধারণ করে) সরাসরি সংজ্ঞায়িত করা যায় না, কারণ তার গভীরতা অসীম হতে পারে।

`Box<T>` ব্যবহারের মাধ্যমে আমরা স্ট্যাকের ভেতরে একটি নির্দিষ্ট সাইজের পয়েন্টার রাখি, আর চাইল্ড এলিমেন্টগুলোকে হিপে স্থানান্তরিত করি:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryNode {
    pub name: String,
    pub subcategories: Vec<Box<CategoryNode>>,
}

impl CategoryNode {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            subcategories: Vec::new(),
        }
    }

    pub fn add_subcategory(&mut self, child: CategoryNode) {
        self.subcategories.push(Box::new(child));
    }

    pub fn total_categories(&self) -> usize {
        1 + self.subcategories.iter().map(|s| s.total_categories()).sum::<usize>()
    }
}
```

```
Stack: CategoryNode ("Electronics")
  ├── name: "Electronics"
  └── subcategories: Vec<Box<CategoryNode>>
                             │
                             ▼ (Heap)
                    CategoryNode ("Computers")
                      └── subcategories: Vec<Box<CategoryNode>>
                                                 │
                                                 ▼ (Heap)
                                        CategoryNode ("Laptops")
```

### ২. ট্রেইট অবজেক্টস এবং ডায়নামিক ডিসপ্যাচ (`Box<dyn Trait>`)
১৮তম অধ্যায়ে আমরা দেখেছি যে দুটি ভিন্ন ক্লোজারকে একটি সাধারণ ভেক্টরে রাখা যায় না। কিন্তু `Box<dyn Fn(u32) -> u32>` ট্রেইট অবজেক্ট ব্যবহার করে আমরা যেকোনো ক্লোজারকে হিপে রেখে একটি সাধারণ কালেকশনে রাখতে পারি:

```rust
pub type BoxedDiscount = Box<dyn Fn(u32) -> u32>;

pub struct PromotionPipeline {
    rules: Vec<BoxedDiscount>,
}

impl PromotionPipeline {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule<F>(&mut self, rule: F)
    where
        F: Fn(u32) -> u32 + 'static,
    {
        self.rules.push(Box::new(rule));
    }

    pub fn apply_all(&self, amount_cents: u32) -> u32 {
        self.rules.iter().map(|rule| rule(amount_cents)).sum()
    }
}
```

এখন আমরা একই পাইপলাইনে শতাংশের ছাড় এবং থ্রেশহোল্ড ছাড় দুটোই একসাথে যুক্ত করতে পারি:
```rust
let mut pipeline = PromotionPipeline::new();
pipeline.add_rule(make_percentage_discount(10));
pipeline.add_rule(make_threshold_discount(15000, 2000));

// সফলভাবে কাজ করছে! দুটি ক্লোজারই হিপে ট্রেইট অবজেক্ট হিসেবে বক্স করা হয়েছে।
assert_eq!(pipeline.apply_all(20000), 4000);
```

---

## `Deref` ট্রেইট এবং ডিরেফ কোয়ের্সন (Deref Coercion)

`std::ops::Deref` ট্রেইট ব্যবহার করে ডিরেফারেন্স অপারেটরের (`*`) আচরণ কাস্টমাইজ করা যায়:

```rust
pub trait Deref {
    type Target: ?Sized;
    fn deref(&self) -> &Self::Target;
}
```

যখন আপনি `*my_box` লেখেন, তখন রাস্ট ব্যাকগ্রাউন্ডে সেটিকে রূপান্তর করে:
```rust
*(my_box.deref())
```

### ডিরেফ কোয়ের্সন (Deref Coercion)
রাস্টে **Deref Coercion** একটি স্বয়ংক্রিয় কম্পাইল-টাইম সুবিধা। যদি কোনো টাইপ `Deref` ইমপ্লিমেন্ট করে, তবে ফাংশন কলে সেই টাইপের রেফারেন্স স্বয়ংক্রিয়ভাবে তার টার্গেট টাইপের রেফারেন্সে রূপান্তরিত হয় (`&StoreSession<T>` স্বয়ংক্রিয়ভাবে `&T` হয়ে যায়)।

### কাস্টম স্মার্ট পয়েন্টার: `StoreSession<T>`
```rust
use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, Eq)]
pub struct StoreSession<T> {
    pub session_id: String,
    pub inner: T,
}

impl<T> Deref for StoreSession<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for StoreSession<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
```

`StoreSession<Customer>` ডিরেফ ইমপ্লিমেন্ট করার কারণে আমরা সেশনের ওপর সরাসরি `Customer`-এর মেথডগুলো কল করতে পারি:
```rust
let mut session = StoreSession::new("SESS-01", customer);

// ডিরেফ কোয়ের্সন: Customer-এর display_badge সরাসরি সেশনের ওপর কাজ করছে!
println!("{}", session.display_badge());

// DerefMut: Customer-এর upgrade_to_vip সরাসরি ভেতরের অবজেক্টকে পরিবর্তন করছে!
session.upgrade_to_vip();
```

---

## `Drop` ট্রেইট: ডিটারমিনিস্টিক ক্লিনআপ (Deterministic Cleanup)

সি বা সি++ এর মতো ভাষায় মেমরি ফ্রি করার জন্য ম্যানুয়ালি `free()` বা `delete` ডাকতে হয়। জাভাস্ক্রিপ্ট বা পাইথনে গারবেজ কালেক্টর অনিয়মিত সময়ে মেমরি পরিষ্কার করে।

রাস্ট ব্যবহার করে **RAII** (Resource Acquisition Is Initialization) নীতি। যখন কোনো ভ্যালু স্কোপের বাইরে যায়, তখন রাস্ট স্বয়ংক্রিয়ভাবে `Drop` ট্রেইটের `drop` মেথড চালায়:

```rust
pub trait Drop {
    fn drop(&mut self);
}
```

### `StoreSession<T>`-এ `Drop` বাস্তবায়ন
```rust
use std::cell::RefCell;
use std::rc::Rc;

pub struct StoreSession<T> {
    pub session_id: String,
    pub inner: T,
    drop_flag: Option<Rc<RefCell<bool>>>,
}

impl<T> Drop for StoreSession<T> {
    fn drop(&mut self) {
        println!("Cleaning up session [{}]...", self.session_id);
        if let Some(ref flag) = self.drop_flag {
            *flag.borrow_mut() = false; // সেশন নিষ্ক্রিয় করা হলো
        }
    }
}
```

```rust
let active_flag = Rc::new(RefCell::new(false));
{
    let session = StoreSession::with_drop_flag("SESS-01", customer, Rc::clone(&active_flag));
    assert!(*active_flag.borrow()); // সেশন এখন সক্রিয়
} // <-- এখানে স্কোপ শেষ হলো; স্বয়ংক্রিয়ভাবে Drop::drop চলল!

assert!(!*active_flag.borrow()); // প্রমাণিত: স্কোপ ছাড়ার সাথে সাথে সেশন নিষ্ক্রিয় হয়েছে!
```

> [!NOTE]
> কোনো ভ্যালুকে স্কোপ শেষ হওয়ার আগেই ম্যানুয়ালি ড্রপ করতে চাইলে সরাসরি `session.drop()` ডাকা যায় না (ডাবল-ফ্রি এরর প্রতিরোধে কম্পাইলার এটি নিষিদ্ধ করে)। এর বদলে `std::mem::drop(session)` ব্যবহার করতে হয়।

---

## `Rc<T>`: রেফারেন্স কাউন্টিং এবং মাল্টিপল ওনারশিপ

সাধারণত রাস্টে যেকোনো ভ্যালুর ঠিক একজনই মালিক থাকে। কিন্তু গ্রাফ ডেটা স্ট্রাকচার বা শেয়ার্ড সেশনে একাধিক সত্ত্বাকে একই অবজেক্ট শেয়ার করতে হতে পারে।

`Rc<T>` (Reference Counted) সিঙ্গেল-থ্রেডেড প্রোগ্রামে একই হিপ ডেটার একাধিক ওনারশিপ প্রদান করে:

```rust
use std::rc::Rc;

let customer = Rc::new(Customer::new(
    301,
    String::from("Margaret Hamilton"),
    String::from("margaret@apollo.nasa.gov"),
    None,
    true,
));

// শুরুতে ওনার সংখ্যা ১
assert_eq!(Rc::strong_count(&customer), 1);

// ক্লোন করলে ডেটা কপি হয় না, কেবল পয়েন্টার কপি হয় এবং কাউন্টার ১ বাড়ে:
let session_ref = Rc::clone(&customer);
let audit_ref = Rc::clone(&customer);

// ওনার সংখ্যা এখন ৩
assert_eq!(Rc::strong_count(&customer), 3);

// দুটি পয়েন্টারই একই মেমরি থেকে পড়ছে:
assert_eq!(session_ref.name, "Margaret Hamilton");
assert_eq!(audit_ref.name, "Margaret Hamilton");

// একটি রেফারেন্স ড্রপ করলে কাউন্টার কমে ২ হয়:
drop(audit_ref);
assert_eq!(Rc::strong_count(&customer), 2);
```

```
               ┌─────────────┐
session_ref ──►│             │
               │   Heap:     │
customer ─────►│ Customer    │  (Strong Count: 2)
               │             │
               └─────────────┘
```

> [!IMPORTANT]
> `Rc<T>` শুধুমাত্র **সিঙ্গেল-থ্রেড** (single-threaded) প্রোগ্রামের জন্য প্রযোজ্য। মাল্টিপল থ্রেডের মধ্যে ডেটা শেয়ার করতে চাইলে থ্রেড-সেফ অ্যাটমিক রেফারেন্স কাউন্টিং `Arc<T>` ব্যবহার করতে হবে, যা আমরা ২০তম অধ্যায়ে (Concurrency) শিখব।

---

## `RefCell<T>` এবং ইন্টেরিয়র মিউটেবিলিটি প্যাটার্ন (Interior Mutability)

`Rc<T>` একাধিক মালিক তৈরি করে ঠিকই, কিন্তু ডেটায় **কেবলমাত্র ইমিউটেবল অ্যাক্সেস (`&T`)** দেয়। কিন্তু একাধিক মালিকের যদি ডেটা পরিবর্তন করার প্রয়োজন হয়?

**ইন্টেরিয়র মিউটেবিলিটি (Interior Mutability)** হলো রাস্টের একটি শক্তিশালী ডিজাইন প্যাটার্ন, যার সাহায্যে বাইরে থেকে ইমিউটেবল রেফারেন্স থাকা সত্ত্বেও ভেতরের ডেটা মিউটেট করা সম্ভব হয়।

`RefCell<T>` এই প্যাটার্নটি বাস্তবায়ন করে বরোয়িং রুলসকে **কম্পাইল-টাইম থেকে রানটাইমে** স্থানান্তর করার মাধ্যমে:

| নিয়ম | কম্পাইল-টাইম (`&T` / `&mut T`) | রানটাইম (`RefCell<T>`) |
| :--- | :--- | :--- |
| **ইমিউটেবল বরো** | `&T` (অসীম সংখ্যক) | `.borrow()` -> `Ref<T>` (অসীম সংখ্যক) |
| **মিউটেবল বরো** | `&mut T` (ঠিক ১টি) | `.borrow_mut()` -> `RefMut<T>` (ঠিক ১টি) |
| **নিয়ম ভঙ্গ হলে** | কম্পাইলেশন এরর | **রানটাইম প্যানিক** (`already borrowed`) |

### উদাহরণ: মৌলিক `RefCell<T>`
```rust
use std::cell::RefCell;

let cell = RefCell::new(100);

// ইমিউটেবল রেফারেন্সের মধ্য দিয়ে ভেতরের ডেটা মিউটেট করা হচ্ছে!
{
    let mut mut_borrow = cell.borrow_mut();
    *mut_borrow += 50;
} // mut_borrow স্কোপের বাইরে গিয়ে ড্রপ হলো

assert_eq!(*cell.borrow(), 150);
```

### রানটাইম প্যানিক সতর্কতা
যদি একটি বরো সক্রিয় থাকা অবস্থাতেই আপনি `.borrow_mut()` ডাকেন:
```rust
let cell = RefCell::new(42);

let b1 = cell.borrow();
let mut b2 = cell.borrow_mut(); // PANIC: already borrowed: BorrowMutError!
```
রানটাইমে প্যানিক এড়াতে আপনি `.try_borrow()` এবং `.try_borrow_mut()` ব্যবহার করতে পারেন, যা প্যানিকের বদলে `Result` প্রদান করে।

---

## `Rc<T>` এবং `RefCell<T>`-এর সংমিশ্রণ

`Rc<T>` এবং `RefCell<T>` একসাথে জুড়ে দিলে আমরা সিঙ্গেল-থ্রেডেড প্রোগ্রামের সবচেয়ে শক্তিশালী আর্কিটেকচার তৈরি করতে পারি: **একাধিক মালিকের মাঝে শেয়ার্ড মিউটেবল স্টেট!**

```
              ┌──────────────────────────┐
handle_a ────►│   Rc (Strong Count: 2)   │
              │             │            │
handle_b ────►│             ▼            │
              │    RefCell<DiscountAuditor>
              │    (Runtime borrow checked)
              └──────────────────────────┘
```

### মিনিস্টোর ইমপ্লিমেন্টেশন: `SharedAuditor`
`ministore/src/promotions.rs`-এ আমরা `SharedAuditor` তৈরি করেছি:

```rust
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Default)]
pub struct SharedAuditor {
    inner: Rc<RefCell<DiscountAuditor>>,
}

impl SharedAuditor {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(DiscountAuditor::new())),
        }
    }

    /// &self (ইমিউটেবল) রেফারেন্স থাকা সত্ত্বেও interior mutability দিয়ে ডেটা পরিবর্তন!
    pub fn record(&self, discount_cents: u32) {
        self.inner.borrow_mut().record(discount_cents);
    }

    pub fn total_discount_given(&self) -> u32 {
        self.inner.borrow().total_discount_given
    }

    pub fn operations_count(&self) -> u32 {
        self.inner.borrow().operations_count
    }

    pub fn strong_count(&self) -> usize {
        Rc::strong_count(&self.inner)
    }
}
```

```rust
let auditor1 = SharedAuditor::new();
let auditor2 = auditor1.clone(); // একই খাতার দ্বিতীয় ওনার

// auditor2 দিয়ে ইমিউটেবল &auditor2 রেফারেন্সের মাধ্যমে রেকর্ড করা হলো
auditor2.record(1500);

// দুটি হ্যান্ডেলই তাৎক্ষণিকভাবে হালনাগাদ ডেটা দেখতে পাচ্ছে:
assert_eq!(auditor1.operations_count(), 1);
assert_eq!(auditor1.total_discount_given(), 1500);
assert_eq!(auditor2.total_discount_given(), 1500);
```

---

## সাধারণ কম্পাইলার ও রানটাইম এরর এবং সমাধান

### ১. `error[E0072]: recursive type has infinite size`
```rust
struct CategoryNode {
    subcategories: Vec<CategoryNode>, // Box ছাড়া সরাসরি রিকার্সন
}
```
**কারণ**: সরাসরি নিজেকে ধারণ করা কোনো স্ট্রাক্টের জন্য স্ট্যাকে কত বাইট মেমরি লাগবে তা কম্পাইলার অনুমান করতে পারে না।
**সমাধান**: `Box` দিয়ে ইনডিরেকশন তৈরি করুন:
```rust
struct CategoryNode {
    subcategories: Vec<Box<CategoryNode>>,
}
```

### ২. `error[E0596]: cannot borrow data in an 'Rc' as mutable`
```rust
let customer = Rc::new(Customer::new(...));
customer.upgrade_to_vip(); // এরর! Rc শুধুমাত্র ইমিউটেবল ধার দেয়।
```
**কারণ**: `Rc<T>` একাধিক ওনার থাকার কারণে সরাসরি মিউটেবল এক্সেস দেয় না।
**সমাধান**: ডেটাকে `RefCell<T>` দিয়ে মুড়িয়ে নিন (`Rc<RefCell<Customer>>`)।

### ৩. `already borrowed: BorrowMutError` (রানটাইম প্যানিক)
```rust
let cell = RefCell::new(vec![1, 2, 3]);
let reader = cell.borrow();
let mut writer = cell.borrow_mut(); // রানটাইমে প্যানিক!
```
**কারণ**: ইমিউটেবল ধার চলমান থাকা অবস্থাতেই আপনি মিউটেবল ধার নেওয়ার চেষ্টা করেছেন।
**সমাধান**: কার্লি ব্র্যাকেট `{ ... }` দিয়ে বোরোয়িং স্কোপ সীমিত রাখুন অথবা `drop(reader)` করুন।

---

## মিনিস্টোর আর্কিটেকচার এবং টেস্ট ভ্যালিডেশন

এই অধ্যায়ে আমরা মিনিস্টোরের কোর সার্ভিসে স্মার্ট পয়েন্টারগুলো সংযুক্ত করেছি:

```
ministore/
├── src/
│   ├── catalog.rs          # CategoryNode রিকার্সিভ ট্রি (Box<CategoryNode>)
│   ├── promotions.rs       # PromotionPipeline (Vec<Box<dyn Fn>>), SharedAuditor (Rc<RefCell>)
│   ├── models/
│   │   ├── session.rs      # StoreSession<T> (Deref, DerefMut, Drop বাস্তবায়ন)
│   │   ├── customer.rs     # Rc<Customer> শেয়ার্ড প্রোফাইল
│   │   └── mod.rs          # StoreSession রি-এক্সপোর্ট
│   ├── lib.rs              # ৩৯টি পাসিং টেস্ট
│   └── main.rs             # স্মার্ট পয়েন্টার ওয়ার্কফ্লোর পূর্ণ প্রদর্শন
```

### ৩৯টি টেস্ট যাচাইকরণ
```bash
cargo test
```
```text
running 39 tests
test tests::test_boxed_dynamic_dispatch_promotion_pipeline ... ok
test tests::test_recursive_category_tree_with_box ... ok
test tests::test_rc_multiple_ownership_customer ... ok
test tests::test_refcell_interior_mutability_shared_auditor ... ok
test tests::test_custom_smart_pointer_deref_and_drop ... ok
...
test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

## অনুশীলনী (Hands-On Exercises)

1. **ক্যাটাগরি ট্রির গভীরতা নির্ণয়**:
   `CategoryNode`-এ একটি মেথড `depth(&self) -> usize` যুক্ত করুন। যদি কোনো চাইল্ড না থাকে তবে ডেপথ ১, অন্যথায় `1 + max(children depths)`। এটি ৪ লেভেলের একটি ট্রিতে টেস্ট করুন।
2. **`try_borrow` দিয়ে প্যানিক প্রতিরোধ**:
   `SharedAuditor`-এ একটি মেথড `safe_record(&self, discount: u32) -> Result<(), &'static str>` লিখুন যা `.borrow_mut()`-এর বদলে `.try_borrow_mut()` ব্যবহার করবে এবং লক থাকলে এরর প্রদান করবে।
3. **সার্কুলার রেফারেন্স পরীক্ষা**:
   দুটি `Rc<RefCell<Node>>` একে অপরকে রেফারেন্স করলে কীভাবে মেমরি লিক (memory leak) হয় এবং `std::rc::Weak<T>` দিয়ে কীভাবে তা সমাধান করা যায় তা অনুসন্ধান করুন।

---

## অধ্যায় সারসংক্ষেপ ও পয়েন্টার চিটশিট

| পয়েন্টার | বর্ণনা | কখন ব্যবহার করবেন | মেমরি ওভারহেড |
| :--- | :--- | :--- | :--- |
| **`&T` / `&mut T`** | সাধারণ রেফারেন্স | সাধারণ স্বল্পমেয়াদী ধার | কোনো ওভারহেড নেই (শুধু মেমরি অ্যাড্রেস) |
| **`Box<T>`** | স্বতন্ত্র হিপ পয়েন্টার | নির্দিষ্ট সাইজ, রিকার্সন, ট্রেইট অবজেক্ট | স্ট্যাকে ১টি পয়েন্টার + হিপ ডেটা |
| **`Rc<T>`** | রেফারেন্স কাউন্টিং পয়েন্টার | সিঙ্গেল-থ্রেডে একাধিক রিড ওনার | ২টি usize কাউন্টার + হিপ ডেটা |
| **`RefCell<T>`** | ইন্টেরিয়র মিউটেবিলিটি কন্টেইনার | ইমিউটেবল রেফারেন্সের পেছনে মিউটেশন | ১টি isize বরো কাউন্টার |
| **`Rc<RefCell<T>>`** | শেয়ার্ড মিউটেবল কন্টেইনার | সিঙ্গেল-থ্রেডে একাধিক ওনারের মধ্যে মিউটেশন | কাউন্টারসমূহ + হিপ ডেটা |

পরবর্তী অধ্যায়ে আমরা সিঙ্গেল-থ্রেডেড স্মার্ট পয়েন্টার থেকে মাল্টি-থ্রেডেড **কনকারেন্সি (Concurrency)**-তে প্রবেশ করব, যেখানে শিখব থ্রেড, মেসেজ পাসিং চ্যানেল এবং থ্রেড-সেফ স্মার্ট পয়েন্টার (`Arc<T>` ও `Mutex<T>`)।
