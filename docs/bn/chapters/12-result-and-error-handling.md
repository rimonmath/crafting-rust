# অধ্যায় ১২: Result এবং এরর হ্যান্ডলিং

## আপনি যা শিখবেন
- **রিকভারেবল এরর** (প্রত্যাশিত ত্রুটি) এবং **আনরিকভারেবল এরর** (`panic!`)-এর মধ্যে মৌলিক পার্থক্য।
- ব্যর্থ হতে পারে এমন অপারেশন প্রকাশে রাস্ট স্ট্যান্ডার্ড লাইব্রেরির **`Result<T, E>`** এনাম (`Ok(T)` বনাম `Err(E)`)-এর ভূমিকা।
- রাস্ট কেন ট্র্যাডিশনাল এক্সেপশন (`try / catch / throw`) সম্পূর্ণ বর্জন করেছে: স্পষ্ট ফাংশন সিগনেচার, অদৃশ্য কন্ট্রোল ফ্লো না থাকা এবং সফল পাথে শূন্য রানটাইম খরচ।
- **`?` অপারেটর**: বয়লারপ্লেট কোড ছাড়াই নিরবচ্ছিন্ন এরর প্রপাগেশন (Error Propagation) ও আগাম রিটার্ন।
- **`.ok_or()`** এবং **`.ok_or_else()`** ব্যবহার করে `Option<T>` থেকে `Result<T, E>`-তে রূপান্তর।
- কাঠামোগত ফিল্ডসহ কাস্টম ডোমেইন এরর এনাম (**`StoreError`**) ডিজাইন।
- **`.map()`**, **`.map_err()`** এবং **`.and_then()`** দিয়ে রেজাল্ট রূপান্তর।
- MiniStore-এ বাস্তব এরর হ্যান্ডলিং প্রয়োগ:
  - সাধারণ স্ট্রিং এরর (`&'static str`)-এর বদলে টাইপ-সেফ ডোমেইন এরর ব্যবহার।
  - ইনভেন্টরি স্টক নিয়ন্ত্রণ (`StoreError::InsufficientStock`)।
  - অর্ডারের অবৈধ স্টেট ট্রানজিশন প্রতিরোধ (`StoreError::InvalidStateTransition`)।
  - ডিসকাউন্ট কুপন যাচাই (`StoreError::InvalidCoupon`)।
  - `?` অপারেটর দিয়ে ট্রানজ্যাকশনাল মাল্টি-স্টেপ **`checkout`** ফাংশন তৈরি।

---

## কেন এটি আমাদের প্রয়োজন?
বাস্তব জীবনের যেকোনো সফটওয়্যারে বিভিন্ন কারণে অপারেশন ব্যর্থ হতে পারে:
- কোনো কাস্টমার ৫টি আইটেম কিনতে চাইলেন, কিন্তু গুদামে কেবল ২টি পণ্য অবশিষ্ট আছে।
- কোনো অর্ডারের টাকা পরিশোধ করার আগেই কুরিয়ার ট্র্যাকিং নম্বর যোগ করার চেষ্টা করা হলো।
- কোনো ক্রেতা মেয়াদোত্তীর্ণ বা ভুল ডিসকাউন্ট কোড ইনপুট দিলেন।
- ফাঁকা শপিং কার্ট নিয়ে কেউ চেকআউট করার চেষ্টা করলেন।

প্রচলিত প্রোগ্রামিং ভাষাগুলো কীভাবে এরর হ্যান্ডেল করে?

### ১. ট্র্যাডিশনাল এক্সেপশন (`throw / catch`)
Java, C++, C#, এবং Python-এর মতো ভাষাগুলোতে এক্সেপশন ব্যবহৃত হয়:
```java
// জাভা / সি# স্টাইল
public Order checkout(Cart cart) {
    if (cart.isEmpty()) {
        throw new EmptyCartException();
    }
    // ...
}
```
এক্সেপশন দেখতে সুবিধাজনক মনে হলেও সফটওয়্যার আর্কিটেকচারে এটি বড় ধরনের সমস্যা তৈরি করে:
- **অদৃশ্য কন্ট্রোল ফ্লো**: ফাংশনের সিগনেচার দেখে বোঝার উপায় নেই যে ভেতর থেকে কী এক্সেপশন থ্রো হতে পারে। পুরো কোড এবং তার সব ডিপেন্ডেন্সি না পড়ে এটি জানা অসম্ভব। যেকোনো সময় প্রোগ্রাম অপ্রত্যাশিতভাবে থমকে যেতে পারে।
- **লুক্কায়িত বাগ**: ডেভেলপাররা এক্সেপশন ক্যাচ করতে ভুলে যান কারণ টাইপ সিস্টেম ক্যাচ করতে বাধ্য করে না (বিশেষ করে আনচেকড রানটাইম এক্সেপশন)। ফলে প্রোডাকশনে সফটওয়্যার ক্র্যাশ করে।
- **ভারী রানটাইম খরচ**: স্ট্যাক আনওয়াইন্ডিং এবং স্ট্যাক ট্রেস সংরক্ষণ করার কারণে এক্সেপশন থ্রো না হলেও পারফরম্যান্সে নেতিবাচক প্রভাব পড়ে।

### ২. সি-স্টাইল এরর কোড (`-1` বা `NULL`)
C এবং Go ভাষায় রিটার্ন কোড বা `(value, err)` টিউপল ফেরত দেওয়া হয়:
```c
// সি স্টাইল
int result = reduce_stock(product_id, 5);
if (result == -1) {
    // সমস্যা কী? স্টক শেষ? নাকি ডাটাবেজ বিচ্ছিন্ন?
}
```
ম্যাজিক সংখ্যায় কোনো ডায়াগনস্টিক বিবরণ থাকে না এবং ডেভেলপাররা প্রায়ই রিটার্ন কোড চেক করতে ভুলে যান, যার ফলে নষ্ট ডাটা পুরো সিস্টেমে ছড়িয়ে পড়ে।

**রাস্টের দর্শন**: **এরর কোনো ব্যতিক্রম নয়, এরর হলো সাধারণ মান (Errors are Values)।**

রাস্টে কোনো এক্সেপশন নেই। কোনো ফাংশন ব্যর্থ হতে পারলে সেটি `Result<T, E>` রিটার্ন করে।
- ব্যর্থতার সম্ভাবনা ফাংশন সিগনেচারে **সুস্পষ্টভাবে ঘোষিত** থাকে।
- এরর `E` সামলানো ছাড়া কলার সফল মান `T`-কে ব্যবহারই করতে পারবে না (কম্পাইলার বাধ্য করে)।
- এরর হলো একটি সাধারণ এনাম ভ্যালু: একে প্যাটার্ন ম্যাচ করা যায়, ইনস্পেক্ট করা যায়, পরিবর্তন করা যায় এবং `?` অপারেটর দিয়ে সহজে প্রপাগেট করা যায়।

---

## সমস্যাটি কী?
পূর্ববর্তী অধ্যায়গুলোতে MiniStore সাধারণ স্ট্রিং স্লাইস দিয়ে সাময়িক এরর মেসেজ প্রকাশ করেছিল:
```rust
// অধ্যায় ১০-এর প্রাথমিক কোড:
pub fn reduce_stock(&mut self, quantity: u32) -> Result<u32, &'static str> {
    if quantity > self.stock {
        Err("Insufficient stock available")
    } else {
        self.stock -= quantity;
        Ok(self.stock)
    }
}
```

শেখার জন্য এটি সহজ হলেও প্রোডাকশনে সাধারণ স্ট্রিং এররের মারাত্মক সীমাবদ্ধতা রয়েছে:
১. **প্রোগ্রামেটিক বিশ্লেষণের অভাব**: কলার যদি জানতে চায় ঠিক *কতগুলো* ইউনিট অবশিষ্ট আছে (যেমন: অ্যাপে "মাত্র ২টি পণ্য অবশিষ্ট আছে!" দেখাতে), তবে ইংরেজি বাক্য পার্স করে সেই সংখ্যা পাওয়া অসম্ভব।
২. **বানান ভুলের ঝুঁকি**: কেউ যদি `if err == "Insufficient stock available"` দিয়ে চেক করে এবং ভবিষ্যতে ব্যাকএন্ডে বাক্যটি একটু সুন্দর করা হয়, তবে পুরো ফিচার ভেঙে যাবে।
৩. **কেন্দ্রীভূত এরর আর্কিটেকচারের অভাব**: ইনভেন্টরি, অর্ডার ট্রানজিশন, কার্ট এবং পেমেন্টের এররগুলো সাধারণ স্ট্রিংয়ের ভিড়ে তালগোল পাকিয়ে যায়।
৪. **বয়লারপ্লেটের বিস্তার**: `?` অপারেটর ছাড়া একাধিক ধাপের অপারেশন (কার্ট চেক -> ক্যাটালগে প্রোডাক্ট খোঁজা -> স্টক যাচাই -> কুপন ভ্যালিডেশন -> অর্ডার তৈরি) লিখতে গেলে নেস্টেড `match`-এর বিশ্রী পিরামিড তৈরি হয়।

MiniStore-এর প্রয়োজন:
- একটি স্ট্রংলি-টাইপড এনাম (`StoreError`) যা প্রতিটি ব্যর্থতার সুনির্দিষ্ট কারণ ও ডাটা বহন করবে।
- `?` অপারেটরের মাধ্যমে সহজে এরর প্রপাগেট করার সুশৃঙ্খল উপায়।
- `Option` লুকআপকে সরাসরি ডোমেইন এররে রূপান্তর করার সহজ কৌশল।

---

## রাস্টের মূল ধারণা

### ১. `Result<T, E>`-এর সংজ্ঞা
রাস্ট স্ট্যান্ডার্ড লাইব্রেরিতে `Result<T, E>` একটি দুই-ভ্যারিয়েন্টবিশিষ্ট এনাম:

```rust
pub enum Result<T, E> {
    Ok(T),  // অপারেশন সফল, সাথে পেলোড মান T
    Err(E), // অপারেশন ব্যর্থ, সাথে এরর মান E
}
```

`Result`, `Ok`, এবং `Err` রাস্টের প্রিভিউড (prelude)-এ স্বয়ংক্রিয়ভাবে অন্তর্ভুক্ত।

ঠিক যেমন `Option<T>` আপনাকে `None` হ্যান্ডেল করতে বাধ্য করে, তেমনি `Result<T, E>` আপনাকে `Err` হ্যান্ডেল করতে বাধ্য করে। কোনো ফাংশন `Result<Order, StoreError>` রিটার্ন করলে আপনি এরর চেক না করে ভেতরের `Order` কোনোভাবেই ব্যবহার করতে পারবেন না।

### ২. রিকভারেবল বনাম আনরিকভারেবল এরর
রাস্ট দুটি ক্ষেত্রে স্পষ্ট সীমারেখা টানে:
- **আনরিকভারেবল এরর (`panic!`)**: কোডের অভ্যন্তরীণ মারাত্মক বাগ বা অনাকাঙ্ক্ষিত অসামঞ্জস্য (যেমন: ভেক্টরের সীমার বাইরে ইনডেক্স করা, ভাঙা ম্যাথমেটিক্যাল ইনভ্যারিয়েন্ট)। এক্ষেত্রে প্রোগ্রাম তৎক্ষণাৎ বন্ধ হয়ে যায়।
- **রিকভারেবল এরর (`Result<T, E>`)**: স্বাভাবিক ব্যবসায়িক লজিকে যেসব ব্যর্থতা স্বাভাবিকভাবেই ঘটতে পারে (যেমন: স্টক শেষ, ভুল কুপন কোড, ফাইল খুঁজে না পাওয়া)। এগুলো কখনোই প্যানিক ঘটাবে না; এগুলো `Result` দিয়ে কলারের কাছে ফেরত দিতে হবে।

```text
┌─────────────────────────────────────────────────────────────┐
│                       এরর হ্যান্ডলিং                         │
├──────────────────────────────┬──────────────────────────────┤
│    রিকভারেবল (Result)        │    আনরিকভারেবল (panic!)      │
├──────────────────────────────┼──────────────────────────────┤
│ - স্টক অপর্যাপ্ত              │ - আউট-অব-বাউন্ডস ইনডেক্সিং   │
│ - ভুল কুপন কোড               │ - অ্যাসAssertion ফেইলিউর     │
│ - পেমেন্ট প্রত্যাখ্যান       │ - মেমরি অখণ্ডতা নষ্ট হওয়া   │
│ - ক্যাটালগে প্রোডাক্ট নেই    │ - অভ্যন্তরীণ অসম্ভব অবস্থা   │
│ ──► Result<T,E> দিয়ে হ্যান্ডেল│ ──► প্রোগ্রাম সমাপ্ত হয়      │
└──────────────────────────────┴──────────────────────────────┘
```

### ৩. কাস্টম ডোমেইন এরর এনাম
অস্পষ্ট স্ট্রিংয়ের পরিবর্তে রাস্টে ডোমেইন এরর এনাম ব্যবহার করা আদর্শ:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum StoreError {
    InsufficientStock { available: u32, requested: u32 },
    InvalidStateTransition { current: String, action: String },
    ProductNotFound { identifier: String },
    InvalidCoupon { code: String, reason: String },
    EmptyCart,
}
```

এর সুবিধাগুলো দেখুন:
- প্রতিটি ভ্যারিয়েন্ট সমস্যা সমাধানের জন্য **ঠিক যতটুকু ডাটা প্রয়োজন** তা বহন করে।
- কলার প্রোগ্রাম্যাটিকভাবে নির্দিষ্ট এররে প্যাটার্ন ম্যাচ করতে পারে:
  ```rust
  match product.reduce_stock(qty) {
      Ok(remaining) => println!("সফল! অবশিষ্ট: {remaining}"),
      Err(StoreError::InsufficientStock { available, requested }) => {
          println!("দুঃখিত! আপনি চেয়েছেন {requested}টি, কিন্তু আছে মাত্র {available}টি।");
      }
      Err(e) => println!("অন্যান্য সমস্যা: {}", e.message()),
  }
  ```

### ৪. `?` অপারেটর: পরিচ্ছন্ন এরর প্রপাগেশন
এক্সেপশন ছাড়া অন্যান্য ভাষায় এরর উপরে পাঠাতে গেলে প্রচুর পুনরাবৃত্তিমূলক কোড লিখতে হয়:

```rust
// ? অপারেটর ছাড়া: ক্লান্তিকর ও দীর্ঘ কোড!
let product = match catalog.find_by_id_mut(item.product_id) {
    Some(p) => p,
    None => return Err(StoreError::ProductNotFound { identifier: "ID".into() }),
};

match product.reduce_stock(item.quantity) {
    Ok(remaining) => remaining,
    Err(err) => return Err(err), // এরর রিটার্ন
};
```

রাস্ট এই সমস্যার সমাধান করেছে অত্যন্ত চমৎকার **`?` অপারেটর** দিয়ে:

```rust
// ? অপারেটর সহ: সংক্ষিপ্ত, স্পষ্ট এবং নিরাপদ!
let product = catalog
    .find_by_id_mut(item.product_id)
    .ok_or_else(|| StoreError::ProductNotFound { identifier: "ID".into() })?;

product.reduce_stock(item.quantity)?;
```

`?` কীভাবে কাজ করে:
১. এক্সপ্রেশনটি যদি `Ok(মান)` হয়, তবে `?` ভেতর থেকে সেই `মান` বের করে কোডের পরবর্তী লাইনগুলো চালাতে দেয়।
২. আর যদি এক্সপ্রেশনটি `Err(এরর)` হয়, তবে ফাংশনটি আর সামনে না এগিয়ে **তৎক্ষণাৎ কলারের কাছে `Err(এরর)` রিটার্ন করে বের হয়ে যায়**।

> [!NOTE]
> `?` অপারেটর কেবল এমন ফাংশনের ভেতরে ব্যবহার করা যায় যার রিটার্ন টাইপ সামঞ্জস্যপূর্ণ `Result` বা `Option`।

### ৫. `Option` এবং `Result`-এর সেতুবন্ধন
প্রায়শই কোনো অপারেশন `Option<T>` রিটার্ন করে (যেমন ম্যাপে খোঁজা), কিন্তু আমাদের বিজনেজ ফাংশন চায় `Result<T, StoreError>`। রাস্টে এদের রূপান্তরের চমৎকার মেথড রয়েছে:

- **`.ok_or(err)`**: `Some(v)`-কে `Ok(v)`-তে এবং `None`-কে `Err(err)`-এ রূপান্তর করে।
- **`.ok_or_else(|| err)`**: `.ok_or()`-এর মতোই, কিন্তু ক্লোজার দিয়ে লেজিভাবে এরর তৈরি করে (স্ট্রিং ফরম্যাটিং থাকলে মেমরি অপচয় রোধ করে)।
- **`.ok()`**: `Result<T, E>`-কে `Option<T>`-তে রূপান্তর করে এরর ফেলে দেয় (যখন কেবল মান থাকা বা না থাকা গুরুত্বপূর্ণ)।

```rust
let product = catalog.find_by_sku("TECH-01")
    .ok_or_else(|| StoreError::ProductNotFound { identifier: "TECH-01".into() })?;
```

---

## অন্যান্য ভাষার সাথে তুলনা

| ধারণা | Rust | Java / C# | Python | Go | C++ |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **এরর মডেল** | রিটার্ন `Result<T, E>` | `throw / catch` এক্সেপশন | `raise / except` এক্সেপশন | একাধিক রিটার্ন `(val, err)` | `throw / catch` বা `std::expected` |
| **টাইপ নিরাপত্তা** | **কম্পাইল টাইমে বাধ্যতামূলক** | আংশিক (রানটাইম এক্সেপশন লুকানো) | নেই (রানটাইম ডাক টাইপিং) | কনভেনশন (`if err != nil`) | অপ্রয়োগিত এক্সেপশন |
| **আগে বের হওয়ার সিনট্যাক্স** | `?` অপারেটর | স্ট্যাক আনওয়াইন্ডিং | স্ট্যাক আনওয়াইন্ডিং | পুনরাবৃত্তিমূলক `if err != nil` | স্ট্যাক আনওয়াইন্ডিং |
| **পারফরম্যান্স খরচ** | সফল পাথে **০ খরচ** | ভারী (স্ট্যাক ট্রেস সংগ্রহ) | ভারী (ফ্রেম বিশ্লেষণ) | কম (পয়েন্টার রিটার্ন) | ভারী আনওয়াইন্ডিং টেবিল |
| **এরর ডায়াগনস্টিকস** | ডাটা সমৃদ্ধ টাইপড এনাম | এক্সেপশন ক্লাস হায়ারার্কি | এক্সেপশন ক্লাস | সাধারণ স্ট্রিং `errors.New` | `std::exception` / এরর কোড |

---

## ছোট উদাহরণ
নিচের স্বয়ংসম্পূর্ণ কোডটি চালিয়ে কাস্টম এরর, `Result<T, E>`, এবং `?` অপারেটরের কার্যকারিতা দেখুন:

```rust
#[derive(Debug, PartialEq)]
enum BankError {
    InsufficientFunds { balance: u32, attempt: u32 },
    DailyLimitExceeded,
}

struct BankAccount {
    balance_cents: u32,
}

impl BankAccount {
    fn withdraw(&mut self, amount: u32) -> Result<u32, BankError> {
        if amount > 50000 {
            return Err(BankError::DailyLimitExceeded);
        }
        if amount > self.balance_cents {
            return Err(BankError::InsufficientFunds {
                balance: self.balance_cents,
                attempt: amount,
            });
        }
        self.balance_cents -= amount;
        Ok(self.balance_cents)
    }
}

// ? অপারেটর ব্যবহার করে লেনদেন পরিচালনা
fn process_transfer(from: &mut BankAccount, to: &mut BankAccount, amount: u32) -> Result<(), BankError> {
    from.withdraw(amount)?; // উত্তোলন ব্যর্থ হলে এখান থেকেই সরাসরি এরর রিটার্ন হবে!
    to.balance_cents += amount;
    Ok(())
}

fn main() {
    let mut account_a = BankAccount { balance_cents: 10000 };
    let mut account_b = BankAccount { balance_cents: 5000 };

    // ১. সফল লেনদেন
    match process_transfer(&mut account_a, &mut account_b, 4000) {
        Ok(()) => println!("লেনদেন সফল! একাউন্ট এ ব্যালেন্স: ${:.2}", account_a.balance_cents as f64 / 100.0),
        Err(e) => println!("লেনদেন ব্যর্থ: {:?}", e),
    }

    // ২. ব্যর্থ লেনদেন (অপর্যাপ্ত ব্যালেন্স)
    match process_transfer(&mut account_a, &mut account_b, 8000) {
        Ok(()) => println!("লেনদেন সফল!"),
        Err(BankError::InsufficientFunds { balance, attempt }) => {
            println!("প্রত্যাখ্যাত: ${:.2} ট্রান্সফার করতে চেয়েছেন, কিন্তু ব্যালেন্স আছে মাত্র ${:.2}", attempt as f64 / 100.0, balance as f64 / 100.0);
        }
        Err(BankError::DailyLimitExceeded) => println!("প্রত্যাখ্যাত: দৈনিক সীমা অতিক্রম করেছে"),
    }
}
```

---

## MiniStore-এ প্রয়োগ
MiniStore অ্যাপ্লিকেশনে আমরা পুরো ব্যবসায়িক লজিকে `Result<T, StoreError>` সমন্বিত করেছি:

```text
┌─────────────────────────────────────────────────────────────┐
│                         StoreError                          │
├─────────────────────────────────────────────────────────────┤
│ + InsufficientStock { available: u32, requested: u32 }      │
│ + InvalidStateTransition { current: String, action: String }│
│ + ProductNotFound { identifier: String }                    │
│ + InvalidPayment { reason: String }                         │
│ + InvalidCoupon { code: String, reason: String }            │
│ + EmptyCart                                                 │
└─────────────────────────────────────────────────────────────┘
                               ▲
                               │ রিটার্ন করে
   ┌───────────────────────────┴───────────────────────────┐
   │                                                       │
┌──────────────┐     ┌──────────────┐             ┌─────────────────┐
│   Product    │     │    Order     │             │    checkout     │
├──────────────┤     ├──────────────┤             ├─────────────────┤
│ reduce_stock │     │ confirm()    │             │ যাচাই করে:      │
│              │     │ ship()       │             │ ১. কার্ট ফাঁকা নয়│
│              │     │ deliver()    │             │ ২. কুপন বৈধ     │
│              │     │ cancel()     │             │ ৩. ক্যাটালগ আইটেম│
│              │     │              │             │ ৪. স্টক কমায়    │
└──────────────┘     └──────────────┘             └─────────────────┘
                                                           │
                                                           ▼ (? ব্যবহার করে)
                                              Result<Order, StoreError>
```

১. **`StoreError`**: ব্যবহারকারীকে দেখানোর জন্য `.message()` মেথডসহ একটি স্বয়ংসম্পূর্ণ ডোমেইন এরর এনাম।
২. **`Product::reduce_stock`**: `Result<u32, StoreError>` রিটার্ন করে, যা স্টক এবং চাহিদার পরিমাণ স্পষ্ট ধারণ করে।
৩. **`Coupon::validate`**: `Result<(), StoreError>` রিটার্ন করে, ফাঁকা কোড বা ১০০%-এর বেশি ডিসকাউন্ট বাতিল করে।
৪. **`Order` ট্রানজিশন**: `confirm`, `ship`, `mark_delivered`, এবং `cancel` এখন কাঁচা স্ট্রিংয়ের বদলে `Result<(), StoreError>` রিটার্ন করে।
৫. **ট্রানজ্যাকশনাল `checkout` ফাংশন**:
   - কার্ট, ক্যাটালগ, কাস্টমার এবং কুপন গ্রহণ করে।
   - `?` অপারেটর ব্যবহার করে ধাপে ধাপে যাচাই করে এবং স্টক কমায়।
   - যেকোনো একটি ধাপে সমস্যা দেখা দিলে তাৎক্ষণিকভাবে নির্দিষ্ট `StoreError` রিটার্ন করে।

---

## কোড
অধ্যায় ১২-এর সম্পূর্ণ এবং টেস্টেড MiniStore কোড নিচে দেওয়া হলো। স্ন্যাপশটটি `examples/chapter-12/src/main.rs`-এ সংরক্ষিত রয়েছে:

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

/// Domain errors that can occur during MiniStore business operations.
#[derive(Debug, Clone, PartialEq)]
pub enum StoreError {
    InsufficientStock { available: u32, requested: u32 },
    InvalidStateTransition { current: String, action: String },
    ProductNotFound { identifier: String },
    InvalidPayment { reason: String },
    InvalidCoupon { code: String, reason: String },
    EmptyCart,
}

impl StoreError {
    /// User-friendly descriptive error message.
    pub fn message(&self) -> String {
        match self {
            Self::InsufficientStock {
                available,
                requested,
            } => {
                format!("Insufficient stock: requested {requested}, but only {available} available")
            }
            Self::InvalidStateTransition { current, action } => {
                format!("Cannot perform action '{action}' while order is in '{current}' state")
            }
            Self::ProductNotFound { identifier } => {
                format!("Product '{identifier}' was not found in catalog")
            }
            Self::InvalidPayment { reason } => {
                format!("Payment processing failed: {reason}")
            }
            Self::InvalidCoupon { code, reason } => {
                format!("Coupon '{code}' is invalid: {reason}")
            }
            Self::EmptyCart => String::from("Cannot checkout with an empty shopping cart"),
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
    pub fn reduce_stock(&mut self, quantity: u32) -> Result<u32, StoreError> {
        if quantity > self.stock {
            Err(StoreError::InsufficientStock {
                available: self.stock,
                requested: quantity,
            })
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

    /// Validates that coupon has a non-empty code and a realistic discount percentage.
    pub fn validate(&self) -> Result<(), StoreError> {
        if self.code.trim().is_empty() {
            Err(StoreError::InvalidCoupon {
                code: self.code.clone(),
                reason: String::from("Coupon code cannot be empty"),
            })
        } else if self.discount_percent == 0 || self.discount_percent > 100 {
            Err(StoreError::InvalidCoupon {
                code: self.code.clone(),
                reason: format!(
                    "Discount percentage {} must be between 1 and 100",
                    self.discount_percent
                ),
            })
        } else {
            Ok(())
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
    pub fn confirm(&mut self, receipt_id: String) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Pending => {
                self.status = OrderStatus::Confirmed { receipt_id };
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                current: self.status.display_status(),
                action: String::from("confirm"),
            }),
        }
    }

    /// Transitions Confirmed -> Shipped { tracking_number }.
    pub fn ship(&mut self, tracking_number: String) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Confirmed { .. } => {
                self.status = OrderStatus::Shipped { tracking_number };
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                current: self.status.display_status(),
                action: String::from("ship"),
            }),
        }
    }

    /// Transitions to Delivered.
    pub fn mark_delivered(&mut self) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Shipped { .. } => {
                self.status = OrderStatus::Delivered;
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                current: self.status.display_status(),
                action: String::from("deliver"),
            }),
        }
    }

    /// Cancels order if current state permits.
    pub fn cancel(&mut self, reason: String) -> Result<(), StoreError> {
        if self.status.can_cancel() {
            self.status = OrderStatus::Cancelled { reason };
            Ok(())
        } else {
            Err(StoreError::InvalidStateTransition {
                current: self.status.display_status(),
                action: String::from("cancel"),
            })
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

/// Processes a shopping cart into a confirmed order, validating stock and coupon with `?`.
pub fn checkout(
    order_id: OrderId,
    customer: Customer,
    cart: &mut ShoppingCart,
    catalog: &mut Catalog,
    payment: PaymentMethod,
    coupon: Option<Coupon>,
) -> Result<Order, StoreError> {
    if cart.is_empty() {
        return Err(StoreError::EmptyCart);
    }

    if let Some(c) = &coupon {
        c.validate()?;
    }

    // Check inventory and deduct stock for every cart item using `?` error propagation
    for item in &cart.items {
        let product =
            catalog
                .find_by_id_mut(item.product_id)
                .ok_or_else(|| StoreError::ProductNotFound {
                    identifier: format!("ID #{}", item.product_id),
                })?;

        product.reduce_stock(item.quantity)?;
    }

    let items = std::mem::take(&mut cart.items);
    Ok(Order::new(order_id, customer, items, payment, coupon))
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
    println!("=== MiniStore: Result & Robust Error Handling (Part II) ===\n");

    // 1. Initializing Catalog with Products
    let mut catalog = Catalog::new();
    let keyboard = Product::new(
        101,
        String::from("TECH-KEY-001"),
        String::from("Tenkeyless Mechanical Keyboard"),
        ProductCategory::Electronics,
        12000,
        2, // Only 2 in stock!
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

    // 2. Handling Recoverable Errors with match on Result<T, StoreError>
    println!("1. Handling Recoverable Errors (Stock Reduction):");
    let mut desk_pad = Product::new(
        103,
        String::from("OFF-PAD-003"),
        String::from("Leather Desk Mat"),
        ProductCategory::OfficeSupplies,
        2500,
        3,
    );

    // Attempting to buy 5 units when only 3 exist
    match desk_pad.reduce_stock(5) {
        Ok(remaining) => println!("   Stock reduced successfully! Remaining: {remaining}"),
        Err(err) => println!("   Error handled gracefully -> {}", err.message()),
    }

    // 3. Coupon Validation returning Result<(), StoreError>
    println!("\n2. Domain Validation with Custom Errors:");
    let invalid_coupon = Coupon::new(String::from(""), 120);
    match invalid_coupon.validate() {
        Ok(()) => println!("   Coupon is valid!"),
        Err(err) => println!("   Coupon rejected -> {}", err.message()),
    }

    let valid_coupon = Coupon::new(String::from("SUMMER15"), 15);
    if let Err(err) = valid_coupon.validate() {
        println!("   Unexpected coupon error: {}", err.message());
    } else {
        println!("   Coupon 'SUMMER15' (15%) validated successfully!");
    }

    // 4. End-to-End Checkout with '?' Operator Error Propagation
    println!("\n3. Checkout Workflow & Error Propagation (?):");
    let customer = Customer::new(
        301,
        String::from("Margaret Hamilton"),
        String::from("margaret@apollo.nasa.gov"),
        Some(String::from("+1-555-0199")),
        true,
    );

    // Scenario A: Attempt checkout with empty cart
    let mut empty_cart = ShoppingCart::new();
    match checkout(
        OrderId(901),
        customer.clone(),
        &mut empty_cart,
        &mut catalog,
        PaymentMethod::CashOnDelivery,
        None,
    ) {
        Ok(_) => println!("   Checkout succeeded unexpectedly!"),
        Err(err) => println!("   Empty cart rejected -> {}", err.message()),
    }

    // Scenario B: Attempt checkout with more units than stock available (Keyboard has only 2)
    let mut greedy_cart = ShoppingCart::new();
    greedy_cart.add_item(101, 5, 12000); // Wants 5 units
    match checkout(
        OrderId(902),
        customer.clone(),
        &mut greedy_cart,
        &mut catalog,
        PaymentMethod::CreditCard {
            last_four: String::from("4242"),
        },
        None,
    ) {
        Ok(_) => println!("   Checkout succeeded unexpectedly!"),
        Err(err) => println!("   Excessive quantity rejected -> {}", err.message()),
    }

    // Scenario C: Successful checkout with valid quantities and coupon
    let mut valid_cart = ShoppingCart::new();
    valid_cart.add_item(101, 1, 12000); // 1 keyboard
    valid_cart.add_item(102, 2, 4500); // 2 mice

    match checkout(
        OrderId(903),
        customer,
        &mut valid_cart,
        &mut catalog,
        PaymentMethod::CreditCard {
            last_four: String::from("9876"),
        },
        Some(valid_coupon),
    ) {
        Ok(mut order) => {
            println!(
                "   Checkout Order #{} created! Total: ${:.2} (Subtotal: ${:.2})",
                order.order_id.0,
                order.total_cents() as f64 / 100.0,
                order.subtotal_cents() as f64 / 100.0
            );

            // 5. Order State Machine Enforcing Transitions with StoreError
            println!("\n4. Order Lifecycle Transitions:");
            order.confirm(String::from("REC-903-HAMILTON")).unwrap();
            println!("   Order confirmed: {}", order.status.display_status());

            // Attempting invalid transition: cannot ship before courier tracking is set
            order.ship(String::from("TRK-FEDEX-88319")).unwrap();
            println!("   Order shipped:   {}", order.status.display_status());

            // Attempting illegal cancellation once shipped
            match order.cancel(String::from("Buyer changed mind")) {
                Ok(()) => println!("   Order cancelled!"),
                Err(err) => println!("   Cancellation prevented -> {}", err.message()),
            }

            order.mark_delivered().unwrap();
            println!("   Order delivered: {}", order.status.display_status());
        }
        Err(err) => println!("   Checkout failed: {}", err.message()),
    }
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
        let ship_err = order.ship(String::from("TRK-1")).unwrap_err();
        assert_eq!(
            ship_err,
            StoreError::InvalidStateTransition {
                current: String::from("Awaiting Confirmation"),
                action: String::from("ship"),
            }
        );

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
        let confirm_err = order.confirm(String::from("REC-200")).unwrap_err();
        assert_eq!(
            confirm_err,
            StoreError::InvalidStateTransition {
                current: String::from("Cancelled (Reason: Out of stock)"),
                action: String::from("confirm"),
            }
        );
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

        assert_eq!(order.total_cents(), 17000);

        let extracted = order.remove_coupon();
        assert_eq!(
            extracted,
            Some(Coupon {
                code: String::from("SAVE15"),
                discount_percent: 15
            })
        );
        assert_eq!(order.coupon, None);
        assert_eq!(order.total_cents(), 20000);
    }

    #[test]
    fn test_product_stock_reduction_error() {
        let mut product = Product::new(
            1,
            String::from("SKU-1"),
            String::from("Book"),
            ProductCategory::OfficeSupplies,
            1000,
            5,
        );

        // Success
        assert_eq!(product.reduce_stock(3), Ok(2));
        assert_eq!(product.stock, 2);

        // Insufficient stock error
        let err = product.reduce_stock(5).unwrap_err();
        assert_eq!(
            err,
            StoreError::InsufficientStock {
                available: 2,
                requested: 5,
            }
        );
        assert_eq!(
            err.message(),
            "Insufficient stock: requested 5, but only 2 available"
        );
    }

    #[test]
    fn test_coupon_validation_error() {
        let empty_coupon = Coupon::new(String::from("  "), 10);
        assert_eq!(
            empty_coupon.validate(),
            Err(StoreError::InvalidCoupon {
                code: String::from("  "),
                reason: String::from("Coupon code cannot be empty"),
            })
        );

        let excessive_coupon = Coupon::new(String::from("MAX150"), 150);
        assert_eq!(
            excessive_coupon.validate(),
            Err(StoreError::InvalidCoupon {
                code: String::from("MAX150"),
                reason: String::from("Discount percentage 150 must be between 1 and 100"),
            })
        );

        let valid = Coupon::new(String::from("DISC25"), 25);
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_checkout_error_propagation_and_success() {
        let mut catalog = Catalog::new();
        catalog.add_product(Product::new(
            10,
            String::from("PEN-01"),
            String::from("Gel Pen"),
            ProductCategory::OfficeSupplies,
            200,
            4,
        ));

        let customer = Customer::new(5, String::from("Eve"), String::from("e@e.com"), None, false);

        // 1. Empty cart error
        let mut empty_cart = ShoppingCart::new();
        let res = checkout(
            OrderId(1),
            customer.clone(),
            &mut empty_cart,
            &mut catalog,
            PaymentMethod::CashOnDelivery,
            None,
        );
        assert_eq!(res.unwrap_err(), StoreError::EmptyCart);

        // 2. Product not in catalog
        let mut ghost_cart = ShoppingCart::new();
        ghost_cart.add_item(999, 1, 500);
        let res = checkout(
            OrderId(2),
            customer.clone(),
            &mut ghost_cart,
            &mut catalog,
            PaymentMethod::CashOnDelivery,
            None,
        );
        assert_eq!(
            res.unwrap_err(),
            StoreError::ProductNotFound {
                identifier: String::from("ID #999")
            }
        );

        // 3. Insufficient stock error
        let mut big_cart = ShoppingCart::new();
        big_cart.add_item(10, 10, 200); // Catalog only has 4!
        let res = checkout(
            OrderId(3),
            customer.clone(),
            &mut big_cart,
            &mut catalog,
            PaymentMethod::CashOnDelivery,
            None,
        );
        assert_eq!(
            res.unwrap_err(),
            StoreError::InsufficientStock {
                available: 4,
                requested: 10,
            }
        );

        // 4. Successful checkout
        let mut valid_cart = ShoppingCart::new();
        valid_cart.add_item(10, 3, 200);
        let res = checkout(
            OrderId(4),
            customer,
            &mut valid_cart,
            &mut catalog,
            PaymentMethod::CashOnDelivery,
            None,
        );
        assert!(res.is_ok());
        let order = res.unwrap();
        assert_eq!(order.order_id, OrderId(4));
        assert_eq!(catalog.find_by_id(10).unwrap().stock, 1); // 4 - 3 = 1 remaining!
        assert!(valid_cart.is_empty()); // Items moved into order
    }
}
```

---

## কোড ব্যাখ্যা

### ১. `StoreError` ডোমেইন এনাম
সাধারণ স্ট্রিং মেসেজের বদলে সুনির্দিষ্ট স্ট্রাকচার্ড এনাম:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum StoreError {
    InsufficientStock { available: u32, requested: u32 },
    InvalidStateTransition { current: String, action: String },
    ProductNotFound { identifier: String },
    InvalidPayment { reason: String },
    InvalidCoupon { code: String, reason: String },
    EmptyCart,
}
```
প্রতিটি ভ্যারিয়েন্ট তার নিজস্ব ডায়াগনস্টিক তথ্য ধারণ করে। যেমন: `InsufficientStock` একাধারে `available` এবং `requested` দুটি তথ্যই রাখে, যাতে ইউজার ইন্টারফেসে সুন্দরভাবে দেখানো যায়: *"আপনি ৫টি চেয়েছেন, কিন্তু মাত্র ২টি আছে"*, যা সাধারণ স্ট্রিং দিয়ে করা অসম্ভব ছিল।

### ২. `checkout` ফাংশনে এরর প্রপাগেশন
`checkout` ফাংশনটির ফ্লো খেয়াল করুন:
```rust
pub fn checkout(...) -> Result<Order, StoreError> {
    if cart.is_empty() {
        return Err(StoreError::EmptyCart);
    }

    if let Some(c) = &coupon {
        c.validate()?;
    }

    for item in &cart.items {
        let product = catalog
            .find_by_id_mut(item.product_id)
            .ok_or_else(|| StoreError::ProductNotFound {
                identifier: format!("ID #{}", item.product_id),
            })?;

        product.reduce_stock(item.quantity)?;
    }

    let items = std::mem::take(&mut cart.items);
    Ok(Order::new(order_id, customer, items, payment, coupon))
}
```
লক্ষ্য করুন কোডটি কতটা মার্জিত ও পরিষ্কার:
- `c.validate()?`: কুপন ভুল হলে সাথে সাথে এরর ফেরত পাঠায়।
- `.ok_or_else(...)?`: ক্যাটালগে প্রোডাক্ট না পেলে `Option`-এর `None` সাথে সাথে `StoreError::ProductNotFound`-এ পরিণত হয়ে ফেরত যায়।
- `product.reduce_stock(item.quantity)?`: স্টক কম থাকলে স্বয়ংক্রিয়ভাবে সেই এরর উপরে চলে যায়।

### ৩. `std::mem::take` দিয়ে মালিকানা স্থানান্তর
```rust
let items = std::mem::take(&mut cart.items);
```
`std::mem::take` মেথডটি `cart.items`-এর স্থানটিতে একটি খালি ভেক্টর `Vec::new()` বসিয়ে দেয় এবং কোনো ক্লোন ছাড়াই কার্ট আইটেমগুলোর পূর্ণ মালিকানা নিয়ে নেয়। ফলে কার্টটি পরবর্তী শপিং সেশনের জন্য স্বয়ংক্রিয়ভাবে খালি হয়ে যায়!

---

## সাধারণ ভুলসমূহ

### ১. ব্যবসায়িক এররে `panic!` বা `.unwrap()` ব্যবহার করা
```rust
// ক্ষতিকর অভ্যাস: স্টক শেষ হলে পুরো সার্ভার প্যানিক ক্র্যাশ করবে!
pub fn buy(product: &mut Product, qty: u32) {
    if qty > product.stock {
        panic!("স্টক শেষ!"); // লাইভ ওয়েব সার্ভার বন্ধ হয়ে যাবে!
    }
}
```
ক্রেতা মজুতের চেয়ে বেশি পণ্য কিনতে চাওয়া একটি **স্বাভাবিক ব্যবসায়িক ঘটনা**, কোডের মারাত্মক ত্রুটি নয়। সর্বদা `Result<T, StoreError>` রিটার্ন করুন।

### ২. এররের বিস্তারিত তথ্য হারিয়ে ফেলা
```rust
// ক্ষতিকর অভ্যাস: কেন ব্যর্থ হলো তা প্রকাশ না করা
if catalog.find_by_sku(sku).is_none() {
    return Err("Error"); // কী এরর? কোন SKU পাওয়া যায়নি?
}
```
এরর ভ্যারিয়েন্টের সাথে সর্বদা প্রয়োজনীয় ডায়াগনস্টিক তথ্য (যেমন ইনপুট SKU বা আইডি) যুক্ত করুন যাতে ডেভেলপার ও ব্যবহারকারী দ্রুত কারণটি শনাক্ত করতে পারেন।

### ৩. `Result` রিটার্ন করে না এমন ফাংশনে `?` ব্যবহার করা
```rust
fn print_stock(product: &mut Product) {
    product.reduce_stock(5)?; // কম্পাইল এরর: () রিটার্ন করা ফাংশনে ? ব্যবহার নিষিদ্ধ
}
```
`?` অপারেটর মূলত আগাম `return Err(...)` এক্সিকিউট করে। যে ফাংশন `Result` রিটার্ন করে না, সেখানে কম্পাইলার এটি নিষিদ্ধ করে।

---

## কম্পাইলার এররসমূহ

### Error E0277: `Result` ছাড়া অন্য ফাংশনে `?` ব্যবহারের চেষ্টা
```rust
fn update_inventory(product: &mut Product) {
    product.reduce_stock(1)?;
}
```
কম্পাইলার আউটপুট:
```text
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option`
 --> src/main.rs:2:28
  |
1 | fn update_inventory(product: &mut Product) {
  | ------------------------------------------ this function should return `Result` or `Option`
2 |     product.reduce_stock(1)?;
  |                            ^ cannot use the `?` operator in a function that returns `()`
```
**সমাধান**: ফাংশনটির রিটার্ন টাইপ পরিবর্তন করে `Result<(), StoreError>` করুন, অথবা `match` / `if let` দিয়ে লোকালি এরর হ্যান্ডেল করুন।

---

## অনুশীলন
১. **পেমেন্টের পরিমাণ যাচাই**:
   `PaymentMethod`-এ একটি মেথড যোগ করুন:
   ```rust
   pub fn validate_amount(&self, amount_cents: u32) -> Result<(), StoreError>
   ```
   পেমেন্ট যদি `CashOnDelivery` হয় এবং টাকার পরিমাণ $500.00 (৫০,০০০ সেন্ট)-এর বেশি হয়, তবে `Err(StoreError::InvalidPayment { reason: "ক্যাশ অন ডেলিভারির সর্বোচ্চ সীমা $500.00".into() })` রিটার্ন করুন।
২. **`?` দিয়ে ক্যাটালগ প্রোডাক্ট যাচাই**:
   একটি হেল্পার ফাংশন লিখুন:
   ```rust
   pub fn verify_product_in_stock(catalog: &Catalog, sku: &str) -> Result<(), StoreError>
   ```
   `.find_by_sku()`, `.ok_or_else()`, এবং `.is_in_stock()` সমন্বয় করে স্টক নিশ্চিত করুন।
৩. **কার্টের সামগ্রিক স্টক প্রি-ফ্লাইট চেক**:
   `ShoppingCart`-এ একটি মেথড লিখুন:
   ```rust
   pub fn verify_all_in_stock(&self, catalog: &Catalog) -> Result<(), StoreError>
   ```
   ক্যাটালগ মিউটেট না করে সব আইটেম স্টকে আছে কিনা লুপ চালিয়ে যাচাই করুন; ঘাটতি থাকলে প্রথম যে আইটেমে সমস্যা হয়েছে তার এরর ফেরত দিন।

---

## চেকপয়েন্ট
১২টি টেস্টই সফলভাবে পাস করছে কিনা পরীক্ষা করুন:
```bash
cargo test
```
প্রত্যাশিত আউটপুট:
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

`clippy` যাচাই করুন:
```bash
cargo clippy -- -D warnings
```

---

## আমরা কী শিখলাম
- অদৃশ্য এক্সেপশনের পরিবর্তে রাস্ট কেন এররকে সাধারণ মান হিসেবে `Result<T, E>` দিয়ে প্রকাশ করে।
- স্বাভাবিক রিকভারেবল এরর (`Result`) এবং কোডের গুরুতর ত্রুটি (`panic!`)-এর মধ্যকার তফাৎ।
- কীভাবে `?` অপারেটর কোডের গভীর পিরামিড তৈরি না করেই অত্যন্ত মার্জিতভাবে এরর উপরে পৌঁছে দেয়।
- সুনির্দিষ্ট ডায়াগনস্টিক ডাটা সমৃদ্ধ কাস্টম ডোমেইন এরর এনাম (`StoreError`) ডিজাইন।
- `.ok_or()` এবং `.ok_or_else()` দিয়ে `Option` থেকে `Result`-এ সহজ রূপান্তর।
- ইনভেন্টরি ও কুপন নিরাপদ রেখে সম্পূর্ণ ট্রানজ্যাকশনাল `checkout` ফাংশন বাস্তবায়ন।

---

## পরবর্তীতে কী আসছে
MiniStore এখন শক্তপোক্ত এরর হ্যান্ডলিং সমৃদ্ধ একটি চমৎকার ডোমেইন মডেলে পরিণত হয়েছে। তবে আমাদের একমাত্র `main.rs` ফাইলটি এখন ৭০০ লাইনেরও বেশি বড় হয়ে গেছে!
**অধ্যায় ১৩: মডিউল, প্যাকেজ এবং প্রজেক্ট স্ট্রাকচার**-এ আমরা শিখব কীভাবে বড় রাস্ট প্রজেক্টকে বিভিন্ন মডিউল (`mod`), আলাদা ফাইল, প্যাকেজ এবং ক্রেটে পরিচ্ছন্নভাবে ভাগ করতে হয়!
