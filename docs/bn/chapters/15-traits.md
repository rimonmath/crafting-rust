# অধ্যায় ১৫: ট্রেইটস (Traits)

## আপনি যা শিখবেন
- **ট্রেইটস (Traits)** কী এবং অবজেক্ট-ওরিয়েন্টেড ক্লাস ইনহেরিটেন্স (Class Inheritance) ছাড়াই রাস্ট কীভাবে পলিমরফিজম (Polymorphism) ও আচরণ বিনিময় বাস্তবায়ন করে।
- **`trait`** কিওয়ার্ড (`pub trait Taxable`) ব্যবহার করে একটি ট্রেইট ডিফাইন করার নিয়ম।
- **`impl Trait for Type`** সিনট্যাক্স দিয়ে স্ট্রাক্ট এবং এনামে ট্রেইট মেথড ইমপ্লিমেন্ট করা।
- পুনরাবৃত্তি রোধ করতে **ডিফল্ট মেথড ইমপ্লিমেন্টেশন (Default Method Implementations)** প্রদান ও প্রয়োজনমতো ওভাররাইড করা।
- **ট্রেইট বাউন্ডস (Trait Bounds)** (`fn process<T: Summarizable>(item: &T)`) দিয়ে জেনেরিক টাইপের ওপর শর্ত আরোপ করা।
- **`+` সিনট্যাক্স** (`T: Taxable + Summarizable`) দিয়ে একাধিক ট্রেইট বাউন্ড যুক্ত করা এবং জটিল সিগনেচার সাজাতে **`where` ক্লজ (where clauses)** ব্যবহার করা।
- কোনো ট্রেইট ইমপ্লিমেন্টকারী টাইপ রিটার্ন করার জন্য **`impl Trait`** সিনট্যাক্সের ব্যবহার।
- রাস্ট স্ট্যান্ডার্ড লাইব্রেরির অপরিহার্য ট্রেইটসমূহ:
  - **`std::fmt::Display`**: `{}` ফরম্যাটের মাধ্যমে ব্যবহারকারী-বান্ধব (User-facing) স্ট্রিং রূপান্তর।
  - **`std::fmt::Debug`**: `{:?}` ফরম্যাটের মাধ্যমে ডেভেলপার ডায়াগনস্টিক রূপান্তর।
  - **`std::error::Error`**: রাস্টের অফিসিয়াল স্ট্যান্ডার্ড এরর কন্ট্রাক্ট।
  - **`Clone`**, **`Copy`**, **`PartialEq`**, এবং **`Default`** ট্রেইট।
- স্বয়ংক্রিয়ভাবে ট্রেইট তৈরি করা (`#[derive(...)]`) বনাম ম্যানুয়াল ট্রেইট ইমপ্লিমেন্টেশন লেখা।
- MiniStore অ্যাপ্লিকেশনে ট্রেইটস সংহতকরণ:
  - ডিফল্ট ট্যাক্স হিসাব সহ **`Taxable`** ট্রেইট তৈরি।
  - প্রোডাক্ট, কাস্টমার এবং অর্ডারের সার্বজনীন রিপোর্টিংয়ের জন্য **`Summarizable`** ট্রেইট তৈরি।
  - `StoreError`-এর জন্য `std::fmt::Display` এবং `std::error::Error` ইমপ্লিমেন্ট করা।
  - `OrderId` এবং `OrderStatus`-এর জন্য `std::fmt::Display` ইমপ্লিমেন্ট করা।

---

## কেন আমাদের এটি প্রয়োজন?

অধ্যায় ১৪-তে আমরা **জেনেরিকস (Generics)** (`<T>`) শিখেছি, যার সাহায্যে আমরা `Page<T>`-এর মতো সার্বজনীন কন্টেইনার এবং `paginate<T>`-এর মতো জেনেরিক ফাংশন তৈরি করতে পেরেছি।

তবে সাধারণ বিশুদ্ধ জেনেরিকসের একটি বড় সীমাবদ্ধতা রয়েছে:
```rust
fn print_tax<T>(item: &T, price_cents: u32) {
    // COMPILE ERROR: Rust জানে না T-এর কোনো `tax_rate()` মেথড আছে কি না!
    let tax = item.tax_rate() * price_cents;
    println!("Tax: {}", tax);
}
```
যদি `T` মহাবিশ্বের *যেকোনো* টাইপ হতে পারে (একটি সংখ্যা, একটি স্ট্রিং, একটি কাস্টমার বা একটি প্রোডাক্ট), তবে রাস্ট কম্পাইলার `item`-এর ওপর কোনো মেথড কল করতে দেবে না। কারণ একটি সাধারণ পূর্ণসংখ্যার (`i32`) কোনো `.tax_rate()` মেথড থাকতে পারে না!

### ট্র্যাডিশনাল অবজেক্ট ওরিয়েন্টেড ভাষার সমস্যা (ক্লাস ইনহেরিটেন্স)
জাভা, সি++ বা সি#-এর মতো অবজেক্ট ওরিয়েন্টেড ভাষায় বিভিন্ন ক্লাসের অভিন্ন আচরণ প্রকাশের জন্য সাধারণত **ক্লাস ইনহেরিটেন্স (Class Inheritance)** ব্যবহার করা হয়:
```java
// Java / C# শৈলীর ক্লাস হায়ারার্কি
abstract class TaxableEntity {
    abstract int getTaxRate();
}

class Product extends TaxableEntity { ... }
class Service extends TaxableEntity { ... }
```
যদিও ইনহেরিটেন্স ব্যাপকভাবে ব্যবহৃত হয়, সফটওয়্যার আর্কিটেকচারে এর সুপরিচিত কিছু মারাত্মক দুর্বলতা রয়েছে:
১. **ফ্র্যাজাইল বেস ক্লাস প্রবলেম (Fragile Base Class Problem)**: মূল প্যারেন্ট ক্লাসে কোনো ছোট পরিবর্তন আনলে হায়ারার্কির অনেক নিচে থাকা সাবক্লাসগুলো অপ্রত্যাশিতভাবে ভেঙে যেতে পারে।
২. **অনমনীয় একক ইনহেরিটেন্স (Rigid Single Inheritance)**: বেশিরভাগ ওওপি ভাষায় একটি ক্লাস কেবল একটি প্যারেন্ট ক্লাস থেকেই ইনহেরিট করতে পারে। কিন্তু বাস্তবে একটি আইটেম একই সাথে `Taxable`, `Auditable`, `Shippable`, এবং `Discountable` হতে পারে!
৩. **অতিরিক্ত ডেটা ক্লাটার (Bloated State)**: সাবক্লাসগুলো প্যারেন্ট ক্লাসের এমন অনেক ইন্টারনাল ফিল্ড বহন করতে বাধ্য হয় যা তাদের আদতে কোনো কাজেই লাগে না। ডেটার বিন্যাস এবং আচরণ অহেতুক একে অপরের সাথে জড়িয়ে পড়ে।

### রাস্টের সমাধান: ট্রেইটস (Traits)
রাস্টে **কোনো ক্লাস নেই** এবং **কোনো ইনহেরিটেন্স নেই**।

এর পরিবর্তে রাস্ট সম্পূর্ণ স্পষ্টভাবে **ডেটা** (যা `struct` এবং `enum` দিয়ে গঠিত) এবং **আচরণ** (Behavior - যা `trait` দিয়ে গঠিত)-কে আলাদা রাখে:
- একটি **ট্রেইট (Trait)** হলো কিছু মেথডের বিমূর্ত রূপরেখা বা চুক্তি (Contract): একটি টাইপ আসলে কী *করতে পারে*।
- যেকোনো স্ট্রাক্ট বা এনাম যেকোনো সংখ্যক ট্রেইট ইমপ্লিমেন্ট করতে পারে।
- জেনেরিক ফাংশন লেখার সময় নির্দিষ্ট ট্রেইট শর্ত দিয়ে টাইপ সীমাবদ্ধ করা যায় (**ট্রেইট বাউন্ডস / Trait Bounds**)।

এটি ক্লাসিক সফটওয়্যার ইঞ্জিনিয়ারিং নীতিকে পূর্ণ সম্মান জানায়: **"Composition over Inheritance"** (ইনহেরিটেন্সের চেয়ে কম্পোজিশনকে অগ্রাধিকার দিন)।

---

## ট্রেইট সংজ্ঞায়িত এবং ইমপ্লিমেন্ট করা

একটি ট্রেইট ডিক্লেয়ার করতে `trait` কিওয়ার্ড ব্যবহার করা হয়। ট্রেইট ব্লকের ভেতরে আপনি মেথডের সিগনেচার ঘোষণা করেন:

```rust
pub trait Taxable {
    /// ট্যাক্সের শতকরা হার প্রদান করে (যেমন: ১৫% এর জন্য ১৫)।
    fn tax_rate(&self) -> u32;

    /// মূল মূল্যের ওপর ভিত্তি করে ট্যাক্সের পরিমাণ হিসাব করে।
    /// এটি একটি ডিফল্ট ইমপ্লিমেন্টেশন প্রদান করে!
    fn calculate_tax(&self, price_cents: u32) -> u32 {
        (price_cents * self.tax_rate()) / 100
    }
}
```

লক্ষ্য করুন:
১. `tax_rate(&self) -> u32`-এর শেষে কেবল সেমিকোলন `;` রয়েছে—যেকোনো টাইপ এই ট্রেইট ইমপ্লিমেন্ট করলে তাকে অবশ্যই এই মেথডের লজিক নিজে লিখতে হবে।
২. `calculate_tax(&self, price_cents: u32) -> u32`-এর একটি **ডিফল্ট ইমপ্লিমেন্টেশন বডি** রয়েছে! যারা এই ট্রেইট ব্যবহার করবে, তারা কোনো নতুন কোড না লিখেই সরাসরি এই হিসাবের সুবিধা উপভোগ করতে পারবে।

### কোনো টাইপের ওপর ট্রেইট ইমপ্লিমেন্ট করা
কোনো নির্দিষ্ট টাইপের জন্য ট্রেইট বাস্তবায়ন করতে `impl TraitName for TypeName` সিনট্যাক্স ব্যবহার করা হয়:

```rust
impl Taxable for ProductCategory {
    fn tax_rate(&self) -> u32 {
        self.default_tax_rate()
    }
    // `calculate_tax` স্বয়ংক্রিয়ভাবে ডিফল্ট ইমপ্লিমেন্টেশন থেকে পাওয়া যাবে!
}

impl Taxable for Product {
    fn tax_rate(&self) -> u32 {
        self.category.tax_rate()
    }
}
```

এখন যেকোনো `ProductCategory` বা `Product` সরাসরি `.tax_rate()` এবং `.calculate_tax()` কল করতে পারে:
```rust
let category = ProductCategory::Electronics;
println!("Tax on $100: ${:.2}", category.calculate_tax(10000) as f64 / 100.0);
```

---

## মানসিক মডেল: চুক্তি এবং সক্ষমতা (Contracts and Capabilities)

ট্রেইটকে কোনো বংশবৃক্ষ বা ফ্যামিলি ট্রি হিসেবে না দেখে একটি **কাজের যোগ্যতা সনদ** বা **সক্ষমতার ব্যাজ (Capability Badge)** হিসেবে ভাবুন:

```
           +-----------------------+
           |    Trait: Taxable     |
           |  fn tax_rate(&self)   |
           +-----------------------+
                       ▲
        ┌──────────────┴──────────────┐
        │                             │
+---------------+             +---------------+
|    Product    |             | ProductCategory|
+---------------+             +---------------+
```

যেকোনো টাইপ যদি এই চুক্তিতে স্বাক্ষর করে এবং `tax_rate` মেথড লিখে দেয়, সে-ই তার নামের পাশে `Taxable` ব্যাজ ধারণ করতে পারবে। একটি স্ট্রাক্ট যত খুশি তত ব্যাজ অর্জন করতে পারে:
```rust
impl Taxable for Product { ... }
impl Summarizable for Product { ... }
impl Display for Product { ... }
```

---

## অন্যান্য ভাষার সাথে তুলনা

| বৈশিষ্ট্য | রাস্ট (Rust) | জাভা / সি# (Java/C#) | গো (Go) | সি++ (C++) | টাইপস্ক্রিপ্ট (TypeScript) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **মূল ধারণা** | **Trait** | Interface | Interface | Pure Virtual Class / Concept | Interface |
| **বাস্তবায়ন** | স্পষ্ট (`impl Trait for Type`) | স্পষ্ট (`implements Interface`) | পরোক্ষ (Duck typing মেথড সিগনেচার দেখে) | ইনহেরিটেন্স (`: public Base`) | স্ট্রাকচারাল (পরোক্ষ শেইপ ম্যাচিং) |
| **ডিফল্ট মেথড** | হ্যাঁ (ট্রেইট সংজ্ঞাতেই দেওয়া যায়) | হ্যাঁ (ডিফল্ট ইন্টারফেস মেথড) | না | হ্যাঁ (ভার্চুয়াল বেস ক্লাস মেথড) | না (ইন্টারফেসে কেবল সিগনেচার থাকে) |
| **ডায়নামিক ডিসপ্যাচ** | অপশনাল (`&dyn Trait`) | ইন্টারফেসের জন্য ডিফল্ট | ইন্টারফেসের জন্য ডিফল্ট | ভার্চুয়াল মেথড টেবিল (`vtable`) | ডায়নামিক রানটাইম |
| **স্ট্যাটিক ডিসপ্যাচ** | **ডিফল্ট** (Monomorphized জিরো-কস্ট) | কনস্ট্রেইন্ট সহ জেনেরিক প্রয়োজন | মাঝারি | টেমপ্লেট ও কনসেপ্টস | কম্পাইল টাইমে ইরেজড |

---

## ট্রেইট বাউন্ডস: জেনেরিকস সীমাবদ্ধ করা (Trait Bounds)

এখন আমরা অধ্যায়ের শুরুতে যে সমস্যার মুখোমুখি হয়েছিলাম তার সমাধান করতে পারব! জেনেরিক ফাংশন লেখার সময় আমরা রাস্ট কম্পাইলারকে জানিয়ে দিতে পারি যে `T` কেবল যেকোনো টাইপ নয়, বরং এটি এমন একটি টাইপ যা একটি নির্দিষ্ট ট্রেইট মেনে চলে:

### ১. `impl Trait` সিনট্যাক্স (সহজ ও সাবলীল)
সহজ ফাংশনগুলোর ক্ষেত্রে আর্গুমেন্টে সরাসরি `item: &impl Summarizable` লেখা যায়:
```rust
pub fn print_summary(item: &impl Summarizable) {
    println!("{}", item.summary());
}
```
এটি ট্রেইট বাউন্ডের একটি সহজ এবং পঠনযোগ্য সিনট্যাক্টিক সুগার।

### ২. স্ট্যান্ডার্ড ট্রেইট বাউন্ড সিনট্যাক্স (`<T: Trait>`)
যখন একাধিক প্যারামিটারকে অবশ্যই হুবহু একই টাইপ শেয়ার করতে হয়:
```rust
pub fn compare_summaries<T: Summarizable>(a: &T, b: &T) {
    println!("A: {}", a.summary());
    println!("B: {}", b.summary());
}
```

### ৩. একাধিক ট্রেইট বাউন্ড (`+` সিনট্যাক্স)
যদি কোনো ফাংশনের জন্য একটি টাইপকে একাধিক ট্রেইটের অধিকারী হতে হয়, তখন `+` প্রতীক ব্যবহার করে তা যুক্ত করা হয়:
```rust
pub fn print_tax_and_summary<T: Taxable + Summarizable>(item: &T, price: u32) {
    println!("{}: Tax is ${}", item.summary(), item.calculate_tax(price));
}
```

### ৪. `where` ক্লজ দিয়ে পরিষ্কার কোড গঠন
যখন একটি জেনেরিক ফাংশনে একাধিক টাইপ প্যারামিটার এবং তাদের প্রতিটিতে একাধিক ট্রেইট বাউন্ড থাকে, তখন ফাংশনের নামের পাশে অ্যাঙ্গেল ব্র্যাকেটের ভেতরের কোড পড়া কঠিন হয়ে পড়ে। রাস্ট এজন্য `where` ক্লজ প্রদান করে:

```rust
// পড়তে কষ্টকর:
pub fn process<T: Taxable + Summarizable, U: Clone + Display>(t: &T, u: &U) { ... }

// `where` ক্লজ ব্যবহার করে সুন্দর ও ইডিওম্যাটিক বিন্যাস:
pub fn process<T, U>(t: &T, u: &U)
where
    T: Taxable + Summarizable,
    U: Clone + Display,
{
    // ...
}
```

---

## স্ট্যান্ডার্ড লাইব্রেরির অপরিহার্য ট্রেইটসমূহ

রাস্টের স্ট্যান্ডার্ড লাইব্রেরি পুরোপুরি ট্রেইটের ওপর প্রতিষ্ঠিত। ইডিওম্যাটিক রাস্ট লেখার জন্য এই ট্রেইটগুলো জানা আবশ্যক:

### ১. `std::fmt::Display` বনাম `std::fmt::Debug`
- **`Debug` (`{:?}`)**: মূলত ডেভেলপারদের জন্য কোডের ইন্টারনাল অবস্থা ডিবাগ করার জন্য ব্যবহৃত হয়। এটি প্রায় সবসময় স্বয়ংক্রিয়ভাবে ডেরাইভ করা যায়: `#[derive(Debug)]`।
- **`Display` (`{}`)**: ব্যবহারকারীদের (Human end-users) সামনে সুন্দরভাবে উপস্থাপনের জন্য ব্যবহৃত হয়। এটি স্বয়ংক্রিয়ভাবে ডেরাইভ করা যায় না, কারণ রাস্ট জানে না আপনার কাস্টমার বা অর্ডারের তথ্য কীভাবে প্রদর্শিত হওয়া উচিত; এটি আপনাকে ম্যানুয়ালি ইমপ্লিমেন্ট করতে হয়।

```rust
impl std::fmt::Display for OrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}
```
এখন:
```rust
let id = OrderId(901);
println!("Order ID: {}", id); // প্রিন্ট হবে: Order ID: #901
```

### ২. `std::error::Error`
রাস্টের সমস্ত স্ট্যান্ডার্ড এরর টাইপ `std::error::Error` ট্রেইট বাস্তবায়ন করে। একটি কাস্টম এররকে অফিসিয়াল স্ট্যান্ডার্ড এররে পরিণত করতে হলে তাকে প্রথমে `Display` এবং `Debug` ইমপ্লিমেন্ট করতে হয়:
```rust
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

// স্ট্যান্ডার্ড Error ট্রেইট বাস্তবায়ন
impl std::error::Error for StoreError {}
```

---

## MiniStore আর্কিটেকচার এবং কোড ইমপ্লিমেন্টেশন

MiniStore-এ ট্রেইট ইন্টিগ্রেশনের আর্কিটেকচার দেখে নেওয়া যাক:

```
ministore/
└── src/
    ├── traits.rs           # Taxable, Summarizable, format_tax_summary
    ├── error.rs            # Display + std::error::Error বাস্তবায়ন
    ├── models/
    │   ├── customer.rs     # OrderId-এ Display; Customer-এ Summarizable
    │   ├── order.rs        # OrderStatus-এ Display; Order-এ Summarizable
    │   └── product.rs      # Product-এ Taxable ও Summarizable
    ├── lib.rs              # ট্রেইট রি-এক্সপোর্ট এবং ২০টি ইউনিট টেস্ট
    └── main.rs             # ট্রেইটস, বাউন্ডস এবং Display ফরম্যাটের ডেমো
```

### ১. `src/traits.rs`
```rust
use crate::models::{Customer, Order, Product, ProductCategory};

/// বিক্রয় কর হিসাব করার আচরণ সংজ্ঞায়িত করে।
pub trait Taxable {
    /// ট্যাক্সের শতকরা হার প্রদান করে (যেমন: ১৫% এর জন্য ১৫)।
    fn tax_rate(&self) -> u32;

    /// সেন্ট মূল্যের ওপর ভিত্তি করে ট্যাক্সের পরিমাণ হিসাব করে।
    /// ডিফল্ট ইমপ্লিমেন্টেশন থাকায় ইমপ্লিমেন্টকারীকে কেবল `tax_rate` সরবরাহ করলেই চলে।
    fn calculate_tax(&self, price_cents: u32) -> u32 {
        (price_cents * self.tax_rate()) / 100
    }
}

/// মানুষের পাঠযোগ্য একক-লাইনের সামারি তৈরির আচরণ সংজ্ঞায়িত করে।
pub trait Summarizable {
    /// সত্ত্বাটির সংক্ষিপ্ত এক লাইনের বিবরণ প্রদান করে।
    fn summary(&self) -> String;
}

// ----------------------------------------------------------------------------
// MiniStore ডোমেন মডেলের জন্য ট্রেইট বাস্তবায়ন
// ----------------------------------------------------------------------------

impl Taxable for ProductCategory {
    fn tax_rate(&self) -> u32 {
        self.default_tax_rate()
    }
}

impl Taxable for Product {
    fn tax_rate(&self) -> u32 {
        self.category.tax_rate()
    }
}

impl Summarizable for Product {
    fn summary(&self) -> String {
        format!(
            "Product #{}: {} [{}] - ${:.2}",
            self.id,
            self.name,
            self.sku,
            self.price_cents as f64 / 100.0
        )
    }
}

impl Summarizable for Customer {
    fn summary(&self) -> String {
        format!("Customer #{}: {} <{}>", self.id, self.name, self.email)
    }
}

impl Summarizable for Order {
    fn summary(&self) -> String {
        format!(
            "Order #{}: {} item(s), Total: ${:.2} [{}]",
            self.order_id.0,
            self.items.len(),
            self.total_cents() as f64 / 100.0,
            self.status.display_status()
        )
    }
}

// ----------------------------------------------------------------------------
// ট্রেইট বাউন্ড সহ জেনেরিক ফাংশন
// ----------------------------------------------------------------------------

/// `Summarizable` বাস্তবায়নকারী যেকোনো আইটেমের সামারি প্রদান করে।
pub fn summarize_item<T: Summarizable>(item: &T) -> String {
    item.summary()
}

/// যে আইটেম একই সাথে `Taxable` এবং `Summarizable`, তার সম্পূর্ণ কর ও মূল্য সামারি তৈরি করে।
pub fn format_tax_summary<T>(item: &T, price_cents: u32) -> String
where
    T: Taxable + Summarizable,
{
    let tax = item.calculate_tax(price_cents);
    format!(
        "{} | Tax: ${:.2} ({}%)",
        item.summary(),
        tax as f64 / 100.0,
        item.tax_rate()
    )
}
```

### ২. স্ট্যান্ডার্ড `Display` বাস্তবায়ন

`src/error.rs`-এ:
```rust
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for StoreError {}
```

`src/models/customer.rs`-এ:
```rust
impl std::fmt::Display for OrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}
```

`src/models/order.rs`-এ:
```rust
impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_status())
    }
}
```

### ৩. `src/main.rs`-এ অ্যাপ্লিকেশন ডেমো
```rust
use ministore::{
    checkout, format_tax_summary, summarize_item, ApiResponse, Catalog, Coupon, Customer, OrderId,
    Page, PaymentMethod, Product, ProductCategory, ShoppingCart,
};

fn main() {
    println!("=== MiniStore: Traits & Shared Behavior (Part III) ===\n");

    let keyboard = Product::new(101, String::from("TECH-KEY-001"), String::from("Mechanical Keyboard"), ProductCategory::Electronics, 12000, 5);
    let customer = Customer::new(301, String::from("Margaret Hamilton"), String::from("margaret@apollo.nasa.gov"), Some(String::from("+1-555-0199")), true);

    // ট্রেইট বাউন্ডের ব্যবহার
    println!("Tax Summary: {}", format_tax_summary(&keyboard, keyboard.price_cents));
    println!("Customer Summary: {}", summarize_item(&customer));

    // Display ট্রেইট ব্যবহার: {order.order_id} প্রিন্ট করবে #901!
    // println!("Checkout Order {} created successfully!", order.order_id);
}
```

---

## সাধারণ কম্পাইলার এরর এবং সমাধানের উপায়

### ১. `error[E0599]: no method named ... found for type ... in the current scope`
**ভুল কোড**:
```rust
let product = Product::new(...);
product.summary(); // COMPILE ERROR
```
**কেন ঘটে**:
রাস্টে কোনো ট্রেইট দ্বারা সংজ্ঞায়িত মেথড কল করতে হলে **সেই ট্রেইটটি কারেন্ট ফাইলে স্কোপের মধ্যে থাকতে হবে** (`in scope`)!
**সমাধান**:
`use` স্টেটমেন্ট দিয়ে ট্রেইটটিকে স্কোপে আনুন:
```rust
use ministore::Summarizable; // এবার product.summary() নির্বিঘ্নে চলবে!
```

---

### ২. অরফান রুল বা অনাথ নিয়ম (`error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate`)
**ভুল কোড**:
```rust
// স্ট্যান্ডার্ড টাইপের (Vec<i32>) ওপর স্ট্যান্ডার্ড ট্রেইট (Display) বাস্তবায়নের চেষ্টা
impl std::fmt::Display for Vec<i32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { ... }
}
```
**কেন ঘটে**:
রাস্টের **অরফান রুল (Orphan Rule)** অনুসারে: আপনি কোনো টাইপের ওপর তখনই কোনো ট্রেইট ইমপ্লিমেন্ট করতে পারবেন, **যদি সেই ট্রেইট অথবা সেই টাইপটির অন্তত যেকোনো একটি আপনার বর্তমান ক্রেটে সংজ্ঞায়িত হয়ে থাকে**।
- ট্রেইট (`Display`) এবং টাইপ (`Vec`) দুটোই যদি বাইরের ক্রেট বা স্ট্যান্ডার্ড লাইব্রেরি থেকে আসে, তবে আপনি তাদের যুক্ত করতে পারবেন না। এটি বিভিন্ন লাইব্রেরির মধ্যে একই টাইপের জন্য বিপরীতমুখী বা দ্বৈত ইমপ্লিমেন্টেশনের সংঘাত রোধ করে।
**সমাধান**:
**নিউটাইপ প্যাটার্ন (Newtype Pattern)** ব্যবহার করুন (বাইরের টাইপটিকে একটি লোকাল টাপল স্ট্রাক্টে মুড়ে নিন):
```rust
struct IntList(Vec<i32>);

impl std::fmt::Display for IntList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { ... }
}
```

---

## ইডিওম্যাটিক রাস্ট বেস্ট প্র্যাকটিস

১. **একাধিক বাউন্ডের ক্ষেত্রে `where` ক্লজ ব্যবহার করুন**: একের অধিক ট্রেইট বাউন্ড থাকলে ফাংশন সিগনেচার পরিষ্কার রাখতে `where` ক্লজে বাউন্ডগুলো সরিয়ে নিন।
২. **যুক্তিসঙ্গত ডিফল্ট ইমপ্লিমেন্টেশন প্রদান করুন**: নিজস্ব ট্রেইট তৈরি করার সময় সাধারণ অ্যালগরিদমগুলোর ডিফল্ট মেথড (`calculate_tax`-এর মতো) লিখে দিন, যাতে ইমপ্লিমেন্টকারীদের কেবল মৌলিক ডেটা সরবরাহ করতে হয়।
৩. **স্ট্যান্ডার্ড ট্রেইট বাস্তবায়ন করুন**: আপনার ডোমেন মডেলগুলোতে স্ট্যান্ডার্ড ট্রেইটসমূহ (`Debug`, `Clone`, `PartialEq`, `Display`, `Default`) ডেরাইভ অথবা ম্যানুয়ালি ইমপ্লিমেন্ট করুন। এতে অন্যান্য রাস্ট ডেভেলপারদের জন্য আপনার কোড ব্যবহার করা সহজ ও স্বস্তিদায়ক হয়।
৪. **ট্রেইটকে নির্দিষ্ট ও ছোট রাখুন (Single Responsibility)**: একটি বিশাল ট্রেইটের বদলে ছোট ছোট সুনির্দিষ্ট ট্রেইট (`Taxable`, `Summarizable`) তৈরি করুন। ছোট ট্রেইট সহজে কম্পোজ এবং ইমপ্লিমেন্ট করা যায়।

---

## বাস্তবমুখী অনুশীলন (Exercises)

### অনুশীলন ১: `Discountable` ট্রেইট বাস্তবায়ন
১. `discount_rate(&self) -> u32` মেথড সহ একটি ট্রেইট `Discountable` ডিফাইন করুন।
২. একটি ডিফল্ট মেথড যোগ করুন `apply_discount(&self, price_cents: u32) -> u32`।
৩. `Coupon` এবং `Customer`-এর জন্য `Discountable` ইমপ্লিমেন্ট করুন (ভিআইপি কাস্টমাররা ফ্ল্যাট ১০% ডিসকাউন্ট পাবেন)।
৪. একটি জেনেরিক ফাংশন লিখুন `calculate_final_price<T: Discountable>(item: &T, base_price: u32) -> u32`।

### অনুশীলন ২: `Product`-এর জন্য `std::fmt::Display` বাস্তবায়ন
১. `src/models/product.rs`-এ `Product`-এর জন্য ম্যানুয়ালি `std::fmt::Display` বাস্তবায়ন করুন।
২. এটিকে `"[SKU] Title - $Price"` ফরম্যাটে সাজান।
৩. `src/main.rs`-এ ম্যানুয়াল ফিল্ড অ্যাক্সেসের পরিবর্তে `{product}` দিয়ে প্রোডাক্ট প্রিন্ট করুন।

---

## চেকপয়েন্ট (Checkpoint)

কম্পাইলার চেক চালিয়ে নিশ্চিত করুন যে ২০টি টেস্ট সফলভাবে পাস করছে:
```bash
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
cargo run
```

প্রত্যাশিত টেস্ট আউটপুট:
```text
running 20 tests
test tests::test_cart_item_option_lookup ... ok
test tests::test_catalog_option_lookups ... ok
test tests::test_checkout_error_propagation_and_success ... ok
test tests::test_coupon_discount_and_take ... ok
test tests::test_coupon_validation_error ... ok
test tests::test_customer_optional_phone ... ok
test tests::test_display_trait_implementations ... ok
test tests::test_generic_api_response_wrapper ... ok
test tests::test_generic_catalog_product_pagination ... ok
test tests::test_generic_page_map_transformation ... ok
test tests::test_generic_pagination_with_integers ... ok
test tests::test_generic_trait_bound_functions ... ok
test tests::test_order_cancellation_prevention ... ok
test tests::test_order_status_valid_lifecycle ... ok
test tests::test_order_total_with_payment_fee ... ok
test tests::test_payment_method_fees_and_descriptions ... ok
test tests::test_product_category_tax_rates ... ok
test tests::test_product_stock_reduction_error ... ok
test tests::test_summarizable_trait_on_domain_models ... ok
test tests::test_taxable_trait_and_default_method ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

প্রত্যাশিত বাইনারি আউটপুট:
```text
=== MiniStore: Traits & Shared Behavior (Part III) ===

1. Catalog initialized with 3 products.

2. Browsing Catalog with Generic Pagination (Page<Product>):
   Page 1 of 2 (Total Items: 3)
   - [TECH-KEY-001] Tenkeyless Mechanical Keyboard ($120.00)
   - [TECH-MOU-002] Ergonomic Wireless Mouse ($45.00)
   Has next page? true
   Transformed to Page<String>: ["Tenkeyless Mechanical Keyboard", "Ergonomic Wireless Mouse"]
   API Page 2 response: 1 product(s) returned out of 3 total.

3. Customer: Margaret Hamilton (+1-555-0199)

4. Shared Behaviors via Traits:
   Tax Summary: Product #101: Tenkeyless Mechanical Keyboard [TECH-KEY-001] - $120.00 | Tax: $18.00 (15%)
   Tax Summary: Product #102: Ergonomic Wireless Mouse [TECH-MOU-002] - $45.00 | Tax: $6.75 (15%)
   Customer Summary: Customer #301: Margaret Hamilton <margaret@apollo.nasa.gov>

5. Processing checkout through modular services...
   Checkout Order #901 created successfully!
   Subtotal: $210.00 | Total: $148.50

6. Order Lifecycle Transitions:
   Order confirmed: Confirmed (Receipt: REC-901-HAMILTON)
   Order shipped:   Shipped (Tracking: TRK-FEDEX-77189)
   Cancellation prevented -> Cannot perform action 'cancel' while order is in 'Shipped (Tracking: TRK-FEDEX-77189)' state
   Final Lifecycle State: Delivered to Customer
   Order Summary: Order #901: 2 item(s), Total: $148.50 [Delivered to Customer]
```

---

## আমরা যা শিখলাম
- নমনীয় ও মডুলার পলিমরফিজম অর্জনের জন্য কেন রাস্ট ক্লাস-ভিত্তিক ইনহেরিটেন্সের পরিবর্তে ট্রেইটকে বেছে নিয়েছে।
- কীভাবে রিকোয়ার্ড মেথড এবং ডিফল্ট মেথড সহ ট্রেইট ডিফাইন এবং ইমপ্লিমেন্ট করতে হয়।
- ট্রেইট বাউন্ডস (`<T: Trait>`), মাল্টিপল বাউন্ডস (`+`), এবং `where` ক্লজ দিয়ে জেনেরিক টাইপকে নিয়ন্ত্রণ করা।
- `Display`, `Debug`, এবং `std::error::Error`-এর মতো স্ট্যান্ডার্ড লাইব্রেরি ট্রেইটের অপরিহার্যতা।
- কীভাবে অরফান রুল ক্রেট বাউন্ডারি রক্ষা করে এবং পারস্পরিক দ্বন্দ্ব সৃষ্টিকারী ইমপ্লিমেন্টেশন প্রতিরোধ করে।

---

## পরবর্তী অধ্যায়
এখন আমরা জেনেরিক টাইপ সংজ্ঞায়িত করতে পারি এবং ট্রেইট দিয়ে তাদের আচরণকে সীমাবদ্ধ করতে পারি, যার ফলে আমাদের কোড অনেক বেশি শক্তিশালী ও সমৃদ্ধ হয়েছে।
তবে যখন কোনো ফাংশন বা স্ট্রাক্ট ওনড (Owned) ভ্যালুর পরিবর্তে রেফারেন্স ধারণ করে, তখন কম্পাইলারকে নিশ্চিত হতে হয় যে সেই রেফারেন্সগুলো তাদের মূল মেমরির চেয়ে বেশি সময় বেঁচে থাকছে না!
**অধ্যায় ১৬: লাইফটাইমস (Lifetimes)**-এ আমরা রাস্টের লাইফটাইম অ্যানোটেশন (`'a`) সম্পর্কে বিস্তারিত জানব এবং দেখব কীভাবে বরো চেকার রেফারেন্স বাউন্ডারির মেমরি নিরাপত্তা নিশ্চিত করে!
