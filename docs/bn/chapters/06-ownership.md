# অধ্যায় ৬: ওনারশিপ (Ownership)

## আপনি কী শিখবেন
- কম্পিউটার মেমোরি ব্যবস্থাপনার মূল দ্বন্দ্ব: ম্যানুয়াল বরাদ্দ বনাম গার্বেজ কালেকশন বনাম **রাস্টের ওনারশিপ মডেল**।
- রাস্ট কম্পাইলার দ্বারা প্রয়োগকৃত **ওনারশিপের তিনটি সোনালী নিয়ম**।
- মেমোরির বাস্তবিক রূপ: **স্ট্যাক বনাম হিপ (Stack vs. Heap)**।
- **মুভ সেমান্টিকস (Move Semantics)**: কেন হিপ-বরাদ্দকৃত ডাটা অন্য ভ্যারিয়েবলে দিলে পূর্বের ভ্যারিয়েবলটি বাতিল হয়ে যায়।
- অন্তর্নিহিত স্ট্যাক কপির (**`Copy` ট্রেইট**) সাথে স্পষ্ট ডিপ ক্লোনিংয়ের (**`Clone` ট্রেইট**) মৌলিক পার্থক্য।
- ফাংশনের মধ্যে ওনারশিপ আদান-প্রদান: ওনারশিপ গ্রহণ (Consuming) এবং ফেরত পাঠানো (Returning)।
- গার্বেজ কালেক্টর ছাড়াই মেমোরি স্বয়ংক্রিয়ভাবে মুক্ত করা: **স্কোপ এবং আরএআইআই (Scope & RAII / `Drop`)**।
- MiniStore-এ ওনারশিপের বাস্তব প্রয়োগ: `PendingOrder`-এর ওনারশিপ সম্পূর্ণ গ্রহণ করে `ConfirmedReceipt` তৈরি, যা টাইপ লেভেলেই একই অর্ডার বারবার প্রসেস হওয়ার ঝুঁকি সমূলে দূর করে।

---

## আমাদের কেন এটি প্রয়োজন?
প্রতিটি কম্পিউটার প্রোগ্রামকে মেমোরি ব্যবহার ও নিয়ন্ত্রণ করতে হয়। ঐতিহাসিকভাবে প্রোগ্রামিং ভাষাগুলো মূলত দুটি বিপরীত শিবিরে বিভক্ত ছিল:

১. **ম্যানুয়াল মেমোরি ব্যবস্থাপনা (C, C++)**:
প্রোগ্রামার নিজ দায়িত্বে মেমোরি বরাদ্দ করেন (`malloc`, `new`) এবং কাজ শেষে তা মুক্ত করেন (`free`, `delete`)।
- *সুবিধা*: সর্বোচ্চ গতি, কোনো অতিরিক্ত রানটাইম ওভারহেড নেই।
- *মারাত্মক ত্রুটি*: মানুষ ভুলপ্রবণ। মেমোরি মুক্ত করতে ভুলে গেলে তৈরি হয় **মেমোরি লিক (Memory Leak)**। একই মেমোরি দুবার মুক্ত করতে গেলে ঘটে **ডাবল-ফ্রি (Double Free)** বিপর্যয়। মেমোরি মুক্ত করার পর তা ব্যবহার করতে গেলে তৈরি হয় **ইউজ-আফটার-ফ্রি (Use-After-Free)** ও সেগমেন্টেশন ফল্ট। সি/সি++ কোডের প্রায় ৭০% গুরুতর সিকিউরিটি বাগ এই মেমোরি ত্রুটি থেকেই সৃষ্টি হয়।

২. **গার্বেজ কালেকশন (Java, Go, C#, Python, JavaScript)**:
অ্যাপ্লিকেশনের পেছনে একটি স্বয়ংক্রিয় ব্যাকগ্রাউন্ড ইঞ্জিন চলে যা মেমোরি স্ক্যান করে অব্যবহৃত অবজেক্ট মুছে দেয়।
- *সুবিধা*: ডাবল-ফ্রি বা ইউজ-আফটার-ফ্রি থেকে নিরাপত্তা দেয়।
- *মারাত্মক ত্রুটি*: যেকোনো মুহূর্তে অপ্রত্যাশিত স্টপ-দ্য-ওয়ার্ল্ড (Stop-The-World) লেটেন্সি পজ, ২ থেকে ৪ গুণ বেশি র‍্যামের অপচয় এবং রানটাইম ওভারহেড—যা হাই-পারফরম্যান্স বা লো-লেটেন্সি অ্যাপ্লিকেশনের জন্য অনুপযুক্ত।

**রাস্টের যুগান্তকারী সমাধান**: কোনো গার্বেজ কালেক্টর ছাড়াই শতভাগ মেমোরি সেফটি। রাস্ট সম্পূর্ণ বিষয়টি কম্পাইল টাইমে **ওনারশিপ (Ownership)** নিয়মের মাধ্যমে যাচাই করে। কোড কম্পাইল হওয়া মানেই নিশ্চিত হওয়া যে এতে কোনো ড্যাংলিং পয়েন্টার, ডাবল-ফ্রি বা ডেটা রেস নেই—এবং এর জন্য রানটাইমে ১ মিলিসেকেন্ডও নষ্ট হয় না।

---

## সমস্যাটি কী?
MiniStore-এর মতো একটি ই-কমার্স অ্যাপ্লিকেশনের অর্ডার চেকআউট ফ্লো বিবেচনা করুন:

```text
কাস্টমার কার্ট / পেন্ডিং অর্ডার ──► চেকআউট সম্পন্ন ──► নিশ্চিত রশিদ (Confirmed Receipt)
```

প্রচলিত ভাষায়:
- চেকআউট সম্পন্ন হওয়ার পরেও যদি `pending_order` ভ্যারিয়েবলটি মেমোরিতে সক্রিয় থাকে, তবে অন্য কোনো থ্রেড অসাবধানতাবশত এটিকে আবার সাবমিট করে **ডুপ্লিকেট চার্জ** কেটে নিতে পারে।
- ডেভেলপারদের বাধ্য হয়ে বাড়তি কপি করতে হয় অথবা `is_processed = true`-এর মতো ম্যানুয়াল ফ্ল্যাগ রাখতে হয়, যা ভুলে যাওয়ার ঝুঁকি থেকেই যায়।

রাস্টের ওনারশিপ নিয়মে, `finalize_order(order: PendingOrder)` ফাংশনে যখনই আপনি `pending_order` পাস করেন, এর ওনারশিপ **মুভ (Move)** হয়ে যায়। আগের ভ্যারিয়েবলটি স্বয়ংক্রিয়ভাবে অচল হয়ে যায়। এরপরে আগের ভ্যারিয়েবলটি ব্যবহারের চেষ্টা করলেই কম্পাইলার সাথে সাথে কোড আটকে দেয়!

---

## রাস্টের সমাধান (Rust Concept)

### ১. ওনারশিপের তিনটি সোনালী নিয়ম
১. **রাস্টে প্রতিটি মানের (Value) একজন ওনার (মালিক ভ্যারিয়েবল) থাকে।**
২. **এক সময়ে একটি মানের কেবল একজনই ওনার থাকতে পারে।**
৩. **ওনার যখন স্কোপের বাইরে চলে যায়, তখন সেই মানটি মেমোরি থেকে সম্পূর্ণ ড্রপ (Drop / মুছে ফেলা) হয়ে যায়।**

### ২. স্ট্যাক বনাম হিপ (Stack vs. Heap)
- **স্ট্যাক (Stack)**:
  - যেসব ডাটার আকার কম্পাইল টাইমে সুনির্দিষ্ট ও অপরিবর্তনীয় (যেমন `u32`, `i64`, `bool`, `[u32; 4]` এবং `OrderId(u64)`-এর মতো টাপল স্ট্রাক্ট), সেগুলো স্ট্যাকে জমা হয়।
  - গতি অবিশ্বাস্য রকম দ্রুত: স্ট্যাকে ডাটা রাখা ও তোলা কেবল সিপিইউ পয়েন্টারের সামান্য স্থানান্তর।
- **হিপ (Heap)**:
  - যেসব ডাটার আকার রানটাইমে পরিবর্তিত হতে পারে বা আগে থেকে নিশ্চিত নয় (যেমন `String`, `Vec<T>`), সেগুলো হিপ মেমরিতে বরাদ্দ হয়।
  - অপারেটিং সিস্টেম মেমোরিতে ফাঁকা জায়গা খুঁজে নেয় এবং তার একটি পয়েন্টার ফিরিয়ে দেয়।

```text
স্ট্যাক (ফিক্সড ২৪-বাইটের ডেসক্রিপ্টর)            হিপ (ডাইনামিক মেমোরি বাফার)
┌──────────────┬─────┬──────────┐            ┌───┬───┬───┬───┬───┬───┬───┐
│ Pointer (ptr)│ Len │ Capacity │ ─────────► │ H │ o │ p │ p │ e │ r │ ! │
└──────────────┴─────┴──────────┘            └───┴───┴───┴───┴───┴───┴───┘
```

স্ট্যাকে থাকা প্রতিটি `String` ভ্যারিয়েবলের ৩টি `usize` উপাদান থাকে (৬৪-বিট সিস্টেমে ২৪ বাইট):
১. হিপে থাকা বাফারের পয়েন্টার (`ptr`)।
২. বর্তমান দৈর্ঘ্য (`len`)।
৩. মোট ক্যাপাসিটি (`capacity`)।

### ৩. মুভ সেমান্টিকস: ডাবল-ফ্রি সমস্যার স্থায়ী সমাধান
একটি হিপ-মালিকানাধীন টাইপ অন্য ভ্যারিয়েবলে অ্যাসাইন করলে কী ঘটে?

```rust
let s1 = String::from("hello");
let s2 = s1; // ওনারশিপ s2-তে চলে গেল (MOVE)!
// println!("{s1}"); // কম্পাইলার এরর! borrow of moved value: `s1`
```

Python বা Java-তে `s2 = s1` কেবল রেফারেন্স কপি করে; দুটি ভ্যারিয়েবলই মেমোরির একই অবজেক্টকে নির্দেশ করে। রাস্ট যদি এমনটা করত, তবে `s1` এবং `s2` উভয় স্কোপের বাইরে যাওয়ার সময় একই মেমোরি ব্লক দুবার মুক্ত করার চেষ্টা করত—যা সি/সি++ এর কুখ্যাত **ডাবল-ফ্রি বাগ**।

C++ এ `std::string` স্বয়ংক্রিয়ভাবে পুরো মেমোরি ডিপ-কপি করে, যা লুপের ভেতর পারফরম্যান্স নষ্ট করে দেয়।

**রাস্টের অনন্য কৌশল**: রাস্ট কেবল স্ট্যাকের ২৪-বাইটের ডেসক্রিপ্টরটি `s1` থেকে `s2`-তে কপি করে এবং অবিলম্বে `s1`-কে **বাতিল (Invalidate)** ঘোষণা করে। এই প্রক্রিয়াটিকে বলা হয় **মুভ (Move)**। যেহেতু `s1` আর বৈধ নয়, তাই কেবল `s2` স্কোপ ছাড়ার সময় একবারই নিরাপদে মেমোরি মুক্ত হবে।

### ৪. `Copy` বনাম `Clone` ট্রেইট
- **`Copy` ট্রেইট**:
  পুরোপুরি স্ট্যাকে বসবাসকারী প্রিমিটিভ টাইপসমূহ (যেমন ইন্টিজার, ফ্লোট, বুলিয়ান, এবং কেবল Copy উপাদান নিয়ে গঠিত স্ট্রাক্ট) `Copy` ট্রেইট সমর্থন করে। এদের ক্ষেত্রে `let b = a;` করলে স্ট্যাকে স্বয়ংক্রিয় বিটওয়াইজ কপি হয়। আদি ভ্যারিয়েবল `a` সম্পূর্ণ কার্যকর থাকে।
- **`Clone` ট্রেইট**:
  হিপ মেমোরি পরিচালনাকারী টাইপগুলো (`String`, `Product`) কখনোই `Copy` হতে পারে না। আপনি যদি সচেতনভাবে হিপ মেমোরির সম্পূর্ণ নতুন ও স্বাধীন প্রতিলিপি চান, তবে আপনাকে স্পষ্টভাবে `.clone()` মেথড কল করতে হবে।

```rust
let id1 = OrderId(101);
let id2 = id1; // COPIED: id1 এবং id2 উভয়ই স্ট্যাকে জীবিত

let p1 = Product::new(1, String::from("Keyboard"), 8999, 5);
// let p2 = p1;        // MOVED: p1 বাতিল হয়ে যাবে
let p2 = p1.clone();   // CLONED: p1 ও p2 হিপে দুটি সম্পূর্ণ পৃথক বাফারের মালিক
```

### ৫. ফাংশনে ওনারশিপ হস্তান্তর
ফাংশনে ভ্যারিয়েবল পাস করলেও অ্যাসাইনমেন্টের মতো একই ওনারশিপ নিয়ম প্রযোজ্য:
- `Copy` টাইপ পাস করলে মান কপি হয়ে যায়।
- নন-`Copy` টাইপ পাস করলে ওনারশিপ ফাংশনের ভেতর **মুভ** হয়ে যায়। ফাংশন কল করার পর কলার আর সেই মানটি অ্যাক্সেস করতে পারে না!

```rust
fn print_customer(c: Customer) {
    println!("{}", c.name);
} // c এই স্কোপের শেষে ড্রপ হয়ে যাবে, মেমোরি মুক্ত হবে!

let cust = Customer::new(1, String::from("Ada"), String::from("a@b.com"), true);
print_customer(cust);
// println!("{}", cust.name); // এরর! cust-এর ওনারশিপ আগেই শেষ হয়ে গেছে
```

---

## অন্যান্য ভাষা থেকে আসলে যা জানা দরকার

| বৈশিষ্ট্য | C / C++ | Java / Go / C# | Python / JS | Rust |
| :--- | :--- | :--- | :--- | :--- |
| **মেমোরি মুক্তকরণ** | ম্যানুয়াল (`free`, `delete`) | স্বয়ংক্রিয় ব্যাকগ্রাউন্ড গার্বেজ কালেক্টর | রেফারেন্স কাউন্টিং ও জিসি | **কম্পাইল-টাইম আরএআইআই (`Drop`)** |
| **হিপ টাইপ অ্যাসাইনমেন্ট** | C++: ডিফল্ট ডিপ কপি (বা ম্যানুয়াল `std::move`) | শেয়ার্ড রেফারেন্স পয়েন্টার | শেয়ার্ড রেফারেন্স পয়েন্টার | **মুভ সেমান্টিকস (আগের ভ্যারিয়েবল বাতিল)** |
| **পারফরম্যান্স ক্ষতি** | কোনো ওভারহেড নেই | স্টপ-দ্য-ওয়ার্ল্ড জিসি পজ ও র‍্যামের অপচয় | ইন্টারপ্রেটার ও জিসি ওভারহেড | **জিরো রানটাইম ওভারহেড** |
| **ডাবল-ফ্রি নিরাপত্তা** | ঝুঁকিপূর্ণ ও প্রায়ই ক্র্যাশ হয় | জিসি সামাল দেয় | জিসি সামাল দেয় | **কম্পাইলার দ্বারা স্থায়ীভাবে অসম্ভব** |
| **ডিপ কপি** | কপি কনস্ট্রাক্টর | `clone()` / ম্যানুয়াল বিল্ডার | `copy.deepcopy()` | **স্পষ্ট `.clone()` মেথড** |

---

## ছোট উদাহরণ (Small Example)

```rust
fn main() {
    let original = String::from("Rust Engine");

    // ১. Move
    let moved = original;
    // println!("{original}"); // এরর: value used here after move

    // ২. Clone
    let cloned = moved.clone();
    println!("উভয়ই সক্রিয়: moved = '{moved}', cloned = '{cloned}'");
}
```

---

## MiniStore-এ প্রয়োগ
MiniStore-এ:
1. `OrderId` হলো স্ট্যাকের ওপর দ্রুত কাজ করা একটি `Copy` টাপল স্ট্রাক্ট।
2. `Product` এবং `Customer` হিপে তাদের নিজস্ব `String` ধারণ করে।
3. `finalize_order(order: PendingOrder) -> ConfirmedReceipt` ফাংশনটি পুরো `order`-এর ওনারশিপ নিজে গ্রহণ করে:
   - পেন্ডিং অর্ডারটি ব্যবহৃত ও ধ্বংস হয়ে যায়।
   - এর ভেতরের স্ট্রিংগুলো (`customer.name`, `product.name`) কোনো নতুন মেমোরি খরচ না করেই সরাসরি নতুন `ConfirmedReceipt`-এ স্থানান্তরিত হয়।
   - অ্যাপ্লিকেশন লেভেলে কোনো গ্রাহকের একই অর্ডার দুবার প্রসেস হওয়া কম্পাইলার লেভেলেই প্রতিরোধ করা সম্ভব হয়েছে।

---

## কোড (Code)

### `src/main.rs`
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

    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    pub fn reduce_stock(&mut self, quantity: u32) -> Result<u32, &'static str> {
        if quantity > self.stock {
            Err("Insufficient stock available")
        } else {
            self.stock -= quantity;
            Ok(self.stock)
        }
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
}

/// OrderId implements `Copy`: simple stack value, never moved
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderId(pub u64);

/// An unconfirmed shopping order owning its line items
#[derive(Debug, Clone, PartialEq)]
pub struct PendingOrder {
    pub order_id: OrderId,
    pub customer: Customer,
    pub product: Product,
    pub quantity: u32,
}

/// Finalized invoice/receipt created when a PendingOrder is processed.
/// By taking ownership of `PendingOrder`, the order is consumed and cannot be re-processed!
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmedReceipt {
    pub receipt_id: String,
    pub order_id: OrderId,
    pub customer_name: String,
    pub product_name: String,
    pub quantity: u32,
    pub total_cents: u32,
}

/// Consumes ownership of a `PendingOrder` by value (Move Semantics).
/// The caller cannot reuse `order` after passing it here.
pub fn finalize_order(order: PendingOrder) -> ConfirmedReceipt {
    let subtotal = order.product.price_cents * order.quantity;
    let discount = if order.customer.is_vip {
        (subtotal * 10) / 100
    } else {
        0
    };
    let total_cents = subtotal - discount;

    ConfirmedReceipt {
        receipt_id: format!("REC-{}-{}", order.order_id.0, order.customer.id),
        order_id: order.order_id,
        customer_name: order.customer.name, // ownership of `String` moves into receipt
        product_name: order.product.name,   // ownership of `String` moves into receipt
        quantity: order.quantity,
        total_cents,
    }
}

fn main() {
    println!("=== MiniStore: Ownership, Move Semantics & Memory ===");

    // 1. Stack Allocation & Copy Trait: Primitive scalar types and Copy structs
    let order_id_1 = OrderId(9001);
    let order_id_2 = order_id_1; // Copied! Both remain fully valid on the stack.
    println!(
        "Copy Demonstration: id_1 = {:?}, id_2 = {:?}",
        order_id_1, order_id_2
    );

    // 2. Heap Allocation: String owns its character buffer on the heap
    let customer_name = String::from("Grace Hopper");
    let customer = Customer::new(
        201,
        customer_name, // Ownership of heap buffer MOVES into `customer`
        String::from("grace@example.com"),
        true,
    );
    // Note: `customer_name` is no longer valid here! Its ownership moved into `customer`.
    println!(
        "Customer Created: {} (VIP: {})",
        customer.name, customer.is_vip
    );

    // 3. Move Semantics vs Explicit Clone
    let product_original = Product::new(
        501,
        String::from("4K Ultra-Wide Monitor"),
        49999, // $499.99
        8,
    );

    // Explicit deep clone: allocates a separate String buffer on the heap
    let product_for_order = product_original.clone();
    println!(
        "Original Product retained: {} (Stock: {})",
        product_original.name, product_original.stock
    );
    println!(
        "Cloned Product for Order: {} (Stock: {})",
        product_for_order.name, product_for_order.stock
    );

    // 4. Moving ownership into an order pipeline
    let pending_order = PendingOrder {
        order_id: order_id_1,
        customer: customer.clone(),
        product: product_for_order,
        quantity: 1,
    };

    println!(
        "\nPending Order #{} created for {}",
        pending_order.order_id.0, pending_order.customer.name
    );

    // 5. Transferring ownership into `finalize_order` (Consuming the order)
    // `pending_order` is MOVED into `finalize_order`. It cannot be used again!
    let receipt = finalize_order(pending_order);

    println!("\n--- Order Confirmed (Ownership Consumed) ---");
    println!("Receipt ID: {}", receipt.receipt_id);
    println!("Customer:   {}", receipt.customer_name);
    println!(
        "Product:    {} x {}",
        receipt.product_name, receipt.quantity
    );
    println!("Total Paid: ${:.2}", receipt.total_cents as f64 / 100.0);

    // 6. Demonstrating Scope & RAII (Resource Acquisition Is Initialization)
    {
        println!("\n--- Entering Temporary Promotion Scope ---");
        let promo_code = String::from("SPRING_CLEANUP_2026");
        println!("Promo code active: {promo_code}");
        // When this block ends, `promo_code` goes out of scope and its heap buffer is automatically dropped!
    }
    println!(
        "Exited promotion scope: heap memory was automatically freed without garbage collection!"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_trait_preserves_original() {
        let id_a = OrderId(101);
        let id_b = id_a; // Copy
        assert_eq!(id_a, id_b);
        assert_eq!(id_a.0, 101);
        assert_eq!(id_b.0, 101);
    }

    #[test]
    fn test_clone_creates_independent_heap_allocation() {
        let original = Product::new(1, String::from("Item A"), 1000, 10);
        let mut cloned = original.clone();

        // Mutating cloned does not mutate original
        cloned.reduce_stock(5).unwrap();
        assert_eq!(cloned.stock, 5);
        assert_eq!(original.stock, 10);
        assert_eq!(original.name, cloned.name);
    }

    #[test]
    fn test_finalize_order_consumes_and_transforms() {
        let customer = Customer::new(1, String::from("Alice"), String::from("a@test.com"), true);
        let product = Product::new(10, String::from("Desk Pad"), 2000, 5); // $20.00
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

## কোড পর্যালোচনা (Understanding The Code)

1. **জিরো-কপি ফিল্ড ট্রান্সফার**:
   ```rust
   customer_name: order.customer.name,
   product_name: order.product.name,
   ```
   যেহেতু `finalize_order` সম্পূর্ণ `order`-এর ওনারশিপ ধারণ করে, এটি অর্ডারকে ভেঙে এর ভেতরের হিপ পয়েন্টারগুলো সরাসরি `ConfirmedReceipt`-এ ট্রান্সফার করে দিতে পারে। কোনো বাড়তি মেমোরি অ্যালোকেশন বা স্ট্রিং কপি করতে হয় না!

2. **স্ট্রাক্টে স্ট্যাক বনাম হিপ**:
   - `OrderId`: `Copy` ট্রেইট থাকার কারণে এটি ৮-বাইটের ইন্টিজার বিটওয়াইজ কপি করে, কোনো মুভ হয় না।
   - `Product`: ভেতরে `name: String` থাকার কারণে এটি কখনোই `Copy` হতে পারে না। ফলে একে অ্যাসাইন করলে ওনারশিপ মুভ হয়ে যায় (যদি না স্পষ্ট `.clone()` করা হয়)।

---

## সাধারণ ভুলসমূহ (Common Mistakes)

### ১. মুভ হয়ে যাওয়া ভ্যারিয়েবল পুনরায় ব্যবহার করা
```rust
let name = String::from("Keyboard");
let p = Product::new(1, name, 5000, 2);
// println!("{name}"); // এরর! name-এর ওনারশিপ Product-এ চলে গেছে
```
একবার ওনারশিপ চলে গেলে বর্তমান স্কোপে সেই ভ্যারিয়েবলের অস্তিত্ব চিরতরে বিলীন হয়ে যায়।

### ২. যত্রতত্র ক্লোন (`.clone()`) করা
জাভা বা পাইথন থেকে আসা ডেভেলপাররা কম্পাইলার এরর এড়াতে সব জায়গায় `.clone()` বসিয়ে দেন। এতে মেমোরির অপচয় হয় এবং পারফরম্যান্স কমে যায়।
> [!TIP]
> পরবর্তী **অধ্যায় ৭: বরোয়িং এবং রেফারেন্স**-এ আমরা শিখব কীভাবে ওনারশিপ হস্তান্তর না করে বা ক্লোন না করে রেফারেন্স (`&T` এবং `&mut T`) দিয়ে ডাটা পড়া ও পরিবর্তন করতে হয়।

---

## কম্পাইলার এরর (Compiler Errors)
মুভ হওয়া ভ্যারিয়েবল ব্যবহারের চেষ্টা করলে কম্পাইলার কী বলে?

```rust
fn main() {
    let order = PendingOrder { ... };
    let receipt = finalize_order(order);
    println!("Processed order: {:?}", order); // এরর!
}
```

কম্পাইলার এরর মেসেজ:
```text
error[E0382]: borrow of moved value: `order`
  --> src/main.rs:160:39
   |
158|     let receipt = finalize_order(order);
   |                                  ----- value moved here
159|     println!("Processed order: {:?}", order);
   |                                       ^^^^^ value borrowed here after move
   |
   = note: this error has occurred because `order` has type `PendingOrder`, which does not implement the `Copy` trait
```
কম্পাইলার কোনো ব্যবহৃত বা বাতিল স্টেট ভুল করে পুনরায় এক্সেস করা অসম্ভব করে তোলে।

---

## অনুশীলন (Practice)
1. একটি ফাংশন `cancel_order(order: PendingOrder) -> String` তৈরি করুন যা অর্ডারের ওনারশিপ গ্রহণ করবে এবং অর্ডার বাতিলের একটি সফল মেসেজ রিটার্ন করবে।
2. `tests` মডিউলে একটি টেস্ট লিখে নিশ্চিত করুন যে `cancel_order` কল করলে কাঙ্ক্ষিত মেসেজ পাওয়া যাচ্ছে।
3. যে অর্ডারটি আপনি `cancel_order`-এ পাঠিয়েছেন, তাকে আবার `finalize_order`-এ পাঠানোর চেষ্টা করে কম্পাইলার এররটি পর্যবেক্ষণ করুন।
4. `cargo test` দিয়ে আপনার সমাধান যাচাই করুন।

---

## চেকবক্স (Checkpoint)
- [x] ওনারশিপের তিনটি সোনালী নিয়ম আয়ত্ত করেছি।
- [x] স্ট্যাক এবং হিপ মেমোরির কাজের পার্থক্য বুঝেছি।
- [x] মুভ সেমান্টিকস এবং ডাবল-ফ্রি থেকে সুরক্ষার কৌশল শিখেছি।
- [x] `Copy` (স্ট্যাক কপি) বনাম `Clone` (হিপ ডিপ কপি)-এর তফাত স্পষ্ট করেছি।
- [x] ফাংশনে ওনারশিপ দিয়ে ওয়ান-ওয়ে স্টেট ট্রানজিশন নিশ্চিত করেছি।

---

## আমরা কী শিখলাম
- গার্বেজ কালেকশনের কোনো রানটাইম পারফরম্যান্স ক্ষতি ছাড়াই রাস্ট কম্পাইল টাইমে নিখুঁত মেমোরি সেফটি দেয়।
- মুভ সেমান্টিকস স্বয়ংক্রিয়ভাবে ডাবল-ফ্রি বন্ধ করে এবং হিপ রিসোর্সের জিরো-কস্ট ট্রান্সফার সম্ভব করে।
- ওনারশিপের নিয়মের সাহায্যে ডুপ্লিকেট চেকআউটের মতো জটিল বিজনেস বাগ কোড লেভেলেই প্রতিরোধ করা যায়।

---

## পরবর্তী অধ্যায়ে কী আসছে
ফাংশনে ওনারশিপ দিয়ে দিলে আগের ভ্যারিয়েবলটি আর ব্যবহার করা যায় না। কিন্তু সবসময় কি ওনারশিপ বদলানো দরকার? যদি কেবল ডাটাটি একটু পড়ার বা সাময়িক বদলানোর প্রয়োজন হয়? **অধ্যায় ৭: বরোয়িং এবং রেফারেন্স (Borrowing and References)**-এ আমরা শিখব রেফারেন্স (`&` ও `&mut`), বরোয়িংয়ের নিয়ম এবং কীভাবে রাস্ট সম্পূর্ণ মেমোরি সুরক্ষা নিশ্চিত রেখে ডাটা রেস রোধ করে।
