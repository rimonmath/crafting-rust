# ২০. কনকারেন্সি (Concurrency)

## আপনি যা শিখবেন
- **কনকারেন্সি (Concurrency)** কী এবং রাস্টে কীভাবে ডেটা রেস (Data Race) ছাড়াই **নির্ভয় কনকারেন্সি (Fearless Concurrency)** নিশ্চিত করা হয়।
- **`std::thread::spawn`** ব্যবহার করে নেটিভ ওএস থ্রেড (Native OS Threads) চালানো এবং **`JoinHandle<T>`** দিয়ে তাদের ফলাফল সংগ্রহ করা।
- **`move`** কীওয়ার্ড ব্যবহার করে থ্রেডের ভেতরে ভেরিয়েবলের ওনারশিপ নিরাপদে স্থানান্তর করা।
- মেসেজ পাসিং (Message Passing) এবং চ্যানেল (**`std::sync::mpsc`**):
  - মাল্টিপল প্রডিউসার, সিঙ্গেল কনজিউমার (Multiple Producer, Single Consumer) আর্কিটেকচার।
  - একাধিক প্রেরক হ্যান্ডেল তৈরির জন্য `tx.clone()` করা।
  - ব্লকিং (`rx.recv()`) এবং নন-ব্লকিং মেসেজ গ্রহণ।
- শেয়ার্ড-স্টেট কনকারেন্সি: **`Arc<T>`** এবং **`Mutex<T>`**:
  - কেন `Rc<T>` এবং `RefCell<T>` মাল্টি-থ্রেডে কাজ করতে পারে না।
  - অ্যাটমিক রেফারেন্স কাউন্টিং (**`Arc<T>`**)।
  - মিউচুয়াল এক্সক্লুশন (**`Mutex<T>`**) এবং RAII গার্ডের মাধ্যমে স্বয়ংক্রিয় আনলকিং।
  - **`Arc<Mutex<T>>`** ব্যবহার করে থ্রেড-সেফ শেয়ার্ড মিউটেবল স্টেট তৈরি।
- **`Send`** এবং **`Sync`** মার্কার ট্রেইট (Marker Traits):
  - কীভাবে রাস্টের টাইপ সিস্টেম কম্পাইল-টাইমেই ডেটা রেস অসম্ভব করে তোলে।
- সাধারণ কম্পাইলার এরর ও জটিলতা:
  - `error[E0277]: 'Rc<...>' cannot be sent between threads safely`
  - `error[E0382]: use of moved value in closure`
  - ডেডলক (Deadlock) ও মিউটেক্স পয়জনিং (Mutex Poisoning)।
- মিনিস্টোরে কনকারেন্সি বাস্তবায়ন:
  - `OrderNotificationChannel` দিয়ে ব্যাকগ্রাউন্ড অর্ডার নোটিফিকেশন।
  - `ConcurrentSalesTracker` (`Arc<Mutex<SalesMetrics>>`) দিয়ে একাধিক চেকআউট থ্রেডের মোট বিক্রয় ট্র্যাকিং।
  - `parallel_batch_valuation`-এর মাধ্যমে মাল্টি-কোর সিপিয়ুতে প্যারালাল ইনভেন্টরি ভ্যালুয়েশন।

---

## আমাদের এটি কেন প্রয়োজন?

এখন পর্যন্ত মিনিস্টোরের প্রতিটি কাজ একটি মাত্র প্রধান থ্রেডে (Main Thread) একের পর এক ধারাবাহিকভাবে (Sequential) সম্পন্ন হয়েছে:
1. পণ্য খোঁজা হয়েছে এক এক করে।
2. চেকআউট লেনদেন চলেছে একটির পর একটি।
3. সম্পূর্ণ ইনভেন্টরির হিসাব প্রধান থ্রেডেই লুপ চালিয়ে করা হয়েছে।

সিঙ্গেল-থ্রেডেড কোড বুঝতে সহজ হলেও আধুনিক কম্পিউটারে একাধিক কোর (Multi-core CPU) থাকে। যখন হাজার হাজার ক্রেতা একসাথে একটি ই-কমার্স স্টোরে কেনাকাটা করতে আসে, তখন ধারাবাহিক কার্যপদ্ধতি বিশাল বাধার সৃষ্টি করে:
- একটি বড় ইনভেন্টরি অডিট চলার সময়ে অন্য সব ক্রেতার চেকআউট আটকে থাকে।
- অর্ডার কনফার্মেশন ইমেইল বা ইনভয়েস জেনারেট হতে গিয়ে পুরো সিস্টেম ফ্রিজ হয়ে যায়।

### অন্যান্য ভাষায় কনকারেন্সির সমস্যা
সি (C) বা সি++ (C++) এ মাল্টি-থ্রেডিং প্রোগ্রামিংকে সবচেয়ে ঝুঁকিপূর্ণ ও জটিল মনে করা হয়:
- **ডেটা রেস (Data Race)**: দুটি থ্রেড একই সাথে মেমরির এক জায়গায় অ্যাক্সেস করছে এবং অন্তত একটি থ্রেড লিখছে (Write), কোনো সিনক্রোনাইজেশন ছাড়া।
- **রেস কন্ডিশন (Race Condition)**: থ্রেডগুলোর চলার অনির্দিষ্ট গতির কারণে ডেটা বিশৃঙ্খল হয়ে যাওয়া।
- **মেমরি করাপশন**: সেগমেন্টেশন ফল্ট বা পয়েন্টার জটিলতা।

### রাস্টের সমাধান: নির্ভয় কনকারেন্সি (Fearless Concurrency)
রাস্ট নিয়ে এসেছে **Fearless Concurrency**:
> যদি আপনার মাল্টি-থ্রেডেড রাস্ট কোড কম্পাইল হতে পারে, তবে এতে কোনো ডেটা রেস (Data Race) নেই—এটি গাণিতিকভাবে নিশ্চিত!

যে ওনারশিপ, বরোয়িং এবং লাইফটাইম নীতি দিয়ে রাস্ট সিঙ্গেল-থ্রেডে মেমরি সেফটি দেয়, সেই একই নীতি একাধিক থ্রেডের ক্ষেত্রেও মেমরি নিরাপত্তা ও নির্ভুলতা নিশ্চিত করে।

---

## নেটিভ থ্রেড: `std::thread`

রাস্ট তার থ্রেডগুলোকে সরাসরি অপারেটিং সিস্টেমের **নেটিভ থ্রেড** (1:1 threading) হিসেবে চালায়। প্রতিটি থ্রেডের নিজস্ব স্ট্যাক মেমরি থাকে এবং তারা সমান্তরালে চলতে পারে।

### ১. থ্রেড তৈরি করা (`thread::spawn`)
একটি নতুন থ্রেড চালু করতে `std::thread::spawn` ব্যবহার করা হয় এবং এর ভেতরে একটি ক্লোজার দেওয়া হয়:

```rust
use std::thread;
use std::time::Duration;

fn main() {
    thread::spawn(|| {
        for i in 1..=5 {
            println!("হ্যাল্লো স্পনড থ্রেড থেকে: {i}");
            thread::sleep(Duration::from_millis(1));
        }
    });

    for i in 1..=3 {
        println!("হ্যাল্লো মেইন থ্রেড থেকে: {i}");
        thread::sleep(Duration::from_millis(1));
    }
}
```

যদি প্রধান (main) থ্রেড শেষ হয়ে যায়, তবে ব্যাকগ্রাউন্ড থ্রেডগুলো কাজ শেষ না করলেও সাথে সাথে বন্ধ হয়ে যাবে।

### ২. `JoinHandle` দিয়ে অপেক্ষা করা
`thread::spawn` একটি `JoinHandle<T>` রিটার্ন করে। এই হ্যান্ডেলের ওপর `.join()` কল করলে প্রধান থ্রেড ব্যাকগ্রাউন্ড থ্রেডটির কাজ শেষ হওয়া পর্যন্ত অপেক্ষা করে এবং থ্রেডের রিটার্ন করা মান প্রদান করে:

```rust
let handle = thread::spawn(|| {
    let item_price = 12000;
    let qty = 2;
    (item_price * qty) + 500 // থ্রেড থেকে রিটার্ন করা মান
});

// `join()` থ্রেড শেষ হওয়া পর্যন্ত অপেক্ষা করবে:
let total = handle.join().expect("Worker thread panicked!");
assert_eq!(total, 24500);
```

### ৩. থ্রেডে `move` কীওয়ার্ড
যখন কোনো থ্রেড তার বাইরের লোকাল ভেরিয়েবল ব্যবহার করতে চায়, তখন রাস্ট বাধ্যতামূলকভাবে **`move`** কীওয়ার্ড দাবি করে। এর কারণ হলো—স্পন করা থ্রেডটি প্রধান থ্রেডের ভেরিয়েবলের চেয়ে বেশি সময় বেঁচে থাকতে পারে, ফলে রেফারেন্স দিয়ে ধার নিলে ড্যাংলিং রেফারেন্সের ঝুঁকি তৈরি হবে:

```rust
let customer_name = String::from("Margaret Hamilton");

// `move` ছাড়া কম্পাইলার এরর দেবে!
let handle = thread::spawn(move || {
    println!("গ্রাহক: {customer_name}");
});

handle.join().unwrap();
```

---

## মেসেজ পাসিং: `std::sync::mpsc` চ্যানেল

কনকারেন্ট প্রোগ্রামিংয়ের একটি জনপ্রিয় ও নিরাপদ ধারা হলো **মেসেজ পাসিং (Message Passing)**:
> "মেমরি শেয়ার করে যোগাযোগ করবেন না; বরং যোগাযোগের মাধ্যমে মেমরি শেয়ার করুন।"

রাস্টের স্ট্যান্ডার্ড লাইব্রেরিতে রয়েছে **`mpsc`**, যার অর্থ **Multiple Producer, Single Consumer** (একাধিক প্রেরক, একজন প্রাপক)।

### ১. চ্যানেল তৈরি ও বার্তা আদান-প্রদান
`mpsc::channel()` একটি চ্যানেল তৈরি করে এবং `(Sender<T>, Receiver<T>)` টাপল প্রদান করে:

```rust
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();

thread::spawn(move || {
    let notification = String::from("Order #901 Confirmed");
    tx.send(notification).unwrap();
    // notification ভেরিয়েবলটির ওনারশিপ চ্যানেলে চলে গেছে!
});

// rx.recv() মেসেজ না আসা পর্যন্ত অপেক্ষা করে:
let msg = rx.recv().expect("Sender disconnected");
println!("গৃহীত বার্তা: {msg}");
```

### ২. একাধিক প্রেরক (`tx.clone()`)
`mpsc` একাধিক প্রডিউসার সমর্থন করে, তাই আমরা `tx.clone()` করে একাধিক ওয়ার্কার থ্রেডে আলাদা আলাদা সেন্ডার পাঠিয়ে দিতে পারি:

```rust
let (tx, rx) = mpsc::channel();

let tx1 = tx.clone();
let tx2 = tx.clone();
drop(tx); // মূল tx ড্রপ করা হলো যাতে সব প্রেরক শেষ হলে rx লুপ শেষ হতে পারে

thread::spawn(move || {
    tx1.send("Order #901").unwrap();
});

thread::spawn(move || {
    tx2.send("Order #902").unwrap();
});

// সব প্রেরক ড্রপ হলে এই লুপ স্বয়ংক্রিয়ভাবে শেষ হবে:
for order_id in rx {
    println!("প্রসেসিং: {order_id}");
}
```

### ৩. মিনিস্টোরের নোটিফিকেশন চ্যানেল
`ministore/src/concurrency.rs`-এ আমরা এই প্যাটার্নটি বাস্তবায়ন করেছি:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderNotification {
    OrderPlaced { order_id: u64, amount_cents: u32 },
    OrderStatusUpdated { order_id: u64, status: String },
    AuditAlert { message: String },
}

pub struct OrderNotificationChannel {
    sender: std::sync::mpsc::Sender<OrderNotification>,
}

impl OrderNotificationChannel {
    pub fn new() -> (Self, std::sync::mpsc::Receiver<OrderNotification>) {
        let (sender, receiver) = std::sync::mpsc::channel();
        (Self { sender }, receiver)
    }

    pub fn clone_sender(&self) -> std::sync::mpsc::Sender<OrderNotification> {
        self.sender.clone()
    }

    pub fn send(&self, notification: OrderNotification) -> Result<(), std::sync::mpsc::SendError<OrderNotification>> {
        self.sender.send(notification)
    }
}
```

---

## শেয়ার্ড-স্টেট কনকারেন্সি: `Arc<T>` এবং `Mutex<T>`

কিছু সিস্টেমে একাধিক থ্রেডকে একই ডেটা একযোগে রিড এবং মিউটেট করতে হয় (যেমন মোট বিক্রয় খাতা বা লাইভ ইনভেন্টরি স্টক)।

### ১. কেন `Rc<T>` চলবে না?
১৯তম অধ্যায়ে আমরা `Rc<T>` শিখেছি। কিন্তু `Rc<T>` মাল্টি-থ্রেডে ব্যবহার করতে গেলে কম্পাইলার এরর দেয়:
```rust
// কম্পাইলার এরর! `Rc<T>` থ্রেড-সেফ নয় (`!Send`)
let rc = Rc::new(5);
thread::spawn(move || { println!("{rc}"); });
```
কারণ `Rc<T>` নন-অ্যাটমিক ইন্সট্রাকশন দিয়ে কাউন্টার বাড়ায়। একাধিক থ্রেড একসাথে কাউন্টার বাড়াতে গেলে মেমরি করাপশন হবে।

### ২. `Arc<T>`: অ্যাটমিক রেফারেন্স কাউন্টিং (Atomic Reference Counting)
`Arc<T>` হলো `Rc<T>` এর থ্রেড-সেফ রূপান্তর। এটি সিপিয়ুর হার্ডওয়্যার অ্যাটমিক ইন্সট্রাকশন ব্যবহার করে নিরাপদে একাধিক থ্রেডে রেফারেন্স শেয়ার করে:
```rust
use std::sync::Arc;
use std::thread;

let shared_data = Arc::new(vec![1, 2, 3]);
let data_clone = Arc::clone(&shared_data);

thread::spawn(move || {
    println!("থ্রেড ডেটা: {:?}", data_clone);
});
```

### ৩. মিউচুয়াল এক্সক্লুশন: `Mutex<T>`
`Arc<T>` ডেটা শেয়ার করে ঠিকই, কিন্তু শুধু **ইমিউটেবল (`&T`)** অ্যাক্সেস দেয়। একাধিক থ্রেডে ডেটা পরিবর্তন করতে আমাদের চাই **`Mutex<T>`** (Mutual Exclusion)।

একটি মিউটেক্স নিশ্চিত করে যে যেকোনো মুহূর্তে কেবল একটি থ্রেডই ডেটা অ্যাক্সেস করতে পারবে:
1. থ্রেড `.lock()` কল করে তালার চাবি চায়।
2. যদি অন্য কোনো থ্রেড তালা ধরে থাকে, তবে বর্তমান থ্রেড ঘুমিয়ে অপেক্ষা করে।
3. তালা মুক্ত হলে `.lock()` একটি `MutexGuard<T>` স্মার্ট পয়েন্টার দেয়।
4. `MutexGuard<T>` স্কোপের বাইরে গেলেই তার `Drop` ট্রেইট **স্বয়ংক্রিয়ভাবে তালা খুলে দেয়**!

```rust
use std::sync::Mutex;

let m = Mutex::new(10);

{
    let mut guard = m.lock().unwrap();
    *guard += 5;
} // <-- এখানে guard ড্রপ হয়ে স্বয়ংক্রিয়ভাবে মিউটেক্স আনলক হলো!

assert_eq!(*m.lock().unwrap(), 15);
```

### ৪. আদর্শ যুগল: `Arc<Mutex<T>>`
`Arc` এবং `Mutex` একসাথে জুড়ে দিলেই আমরা একাধিক থ্রেডের মাঝে শেয়ার্ড মিউটেবল স্টেট তৈরি করতে পারি:

```
              ┌───────────────────────────────┐
Thread 1 ────►│      Arc (Strong Count: 2)     │
              │              │                │
Thread 2 ────►│              ▼                │
              │    Mutex<SalesMetrics>        │
              │    (Exclusive lock guard)     │
              └───────────────────────────────┘
```

### মিনিস্টোর ইমপ্লিমেন্টেশন: `ConcurrentSalesTracker`
`ministore/src/concurrency.rs`:

```rust
use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SalesMetrics {
    pub total_revenue_cents: u64,
    pub transactions_count: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ConcurrentSalesTracker {
    inner: Arc<Mutex<SalesMetrics>>,
}

impl ConcurrentSalesTracker {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SalesMetrics::default())),
        }
    }

    pub fn record_sale(&self, amount_cents: u32) {
        let mut guard = self.inner.lock().expect("Mutex poisoned");
        guard.total_revenue_cents += amount_cents as u64;
        guard.transactions_count += 1;
        // guard ড্রপ হলে আনলক হয়ে যায়!
    }

    pub fn total_revenue(&self) -> u64 {
        self.inner.lock().expect("Mutex poisoned").total_revenue_cents
    }

    pub fn transactions_count(&self) -> u64 {
        self.inner.lock().expect("Mutex poisoned").transactions_count
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}
```

```rust
let tracker = ConcurrentSalesTracker::new();
let mut handles = Vec::new();

// ৮টি চেকআউট থ্রেড একযোগে বিক্রয় রেকর্ড করছে:
for _ in 0..8 {
    let tracker_clone = tracker.clone();
    let handle = std::thread::spawn(move || {
        for _ in 0..5 {
            tracker_clone.record_sale(1000); // $10 বিক্রয়
        }
    });
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

assert_eq!(tracker.transactions_count(), 40);
assert_eq!(tracker.total_revenue(), 40000);
```

---

## `Send` এবং `Sync` মার্কার ট্রেইট (Marker Traits)

রাস্টের টাইপ সিস্টেম দুটি বিল্ট-ইন মার্কার ট্রেইটের মাধ্যমে থ্রেড নিরাপত্তা নিয়ন্ত্রণ করে:

1. **`Send`**: বোঝায় যে এই টাইপটির ওনারশিপ নিরাপদে অন্য থ্রেডে স্থানান্তর করা যাবে।
   - রাস্টের প্রায় সব মৌলিক টাইপই `Send` (`String`, `Vec`, `Box`, `Arc`, `Mutex`)।
   - ব্যতিক্রম: `Rc<T>` (অ্যাটমিক নয়), র-পয়েন্টার (`*const T`)।
2. **`Sync`**: বোঝায় যে একাধিক থ্রেড নিরাপদে এর রেফারেন্স (`&T`) শেয়ার করতে পারবে।
   - অর্থাৎ: `T` যদি `Sync` হয়, তবে `&T` হলো `Send`।
   - `Arc<T>` এবং `Mutex<T>` হলো `Sync`।
   - `RefCell<T>` হলো `Send`, কিন্তু **`Sync` নয়** (কারণ এর রানটাইম কাউন্টার থ্রেড-সেফ নয়)।

---

## প্যারালাল ওয়ার্কলোড: `parallel_batch_valuation`

ইনভেন্টরির লক্ষ লক্ষ পণ্যের মোট মূল্য হিসাব করতে আমরা কাজকে কয়েকটি ভাগে (chunks) ভাগ করে একাধিক থ্রেডে সমান্তরালে হিসাব করি:

```rust
pub fn parallel_batch_valuation(items: Vec<(u32, u32)>, num_workers: usize) -> u64 {
    if items.is_empty() || num_workers == 0 {
        return 0;
    }

    let chunk_size = items.len().div_ceil(num_workers);
    let chunks: Vec<Vec<(u32, u32)>> = items.chunks(chunk_size).map(|c| c.to_vec()).collect();

    let mut handles = Vec::new();

    for chunk in chunks {
        let handle = std::thread::spawn(move || {
            chunk
                .into_iter()
                .map(|(price, qty)| (price as u64) * (qty as u64))
                .sum::<u64>()
        });
        handles.push(handle);
    }

    handles
        .into_iter()
        .map(|handle| handle.join().expect("Worker thread panicked"))
        .sum()
}
```

---

## সাধারণ এরর ও সম্ভাব্য জটিলতা

### ১. `error[E0277]: 'Rc<...>' cannot be sent between threads safely`
**সমাধান**: `Rc<T>` এর বদলে `Arc<T>` ব্যবহার করুন।

### ২. `error[E0382]: use of moved value in closure`
**সমাধান**: লুপের ভেতরে পাঠানোর আগে `Arc::clone(&data)` করে আলাদা হ্যান্ডেল তৈরি করুন।

### ৩. ডেডলক (Deadlock)
থ্রেড ১ ধরে আছে তালা ক এবং অপেক্ষা করছে তালা খ-এর জন্য; অন্যদিকে থ্রেড ২ ধরে আছে তালা খ এবং অপেক্ষা করছে তালা ক-এর জন্য। দুটি থ্রেড অনন্তকাল আটকে থাকবে।
**সমাধান**:
- সব থ্রেডে একই নির্দিষ্ট ক্রমে তালা নিন।
- কার্লি ব্র্যাকেট `{ ... }` দিয়ে লকের স্কোপ যতটা সম্ভব ছোট রাখুন যাতে দ্রুত তালা খুলে যায়।

---

## মিনিস্টোর আর্কিটেকচার এবং ৪৪টি টেস্ট ভ্যালিডেশন

```
ministore/
├── src/
│   ├── concurrency.rs      # OrderNotification, OrderNotificationChannel, ConcurrentSalesTracker, parallel_batch_valuation
│   ├── catalog.rs
│   ├── promotions.rs
│   ├── models/
│   ├── lib.rs              # ৪৪টি পাসিং টেস্ট
│   └── main.rs             # মাল্টি-থ্রেডিং, MPSC চ্যানেল এবং Arc<Mutex>-এর পূর্ণ প্রদর্শন
```

### টেস্ট রান
```bash
cargo test
```
```text
running 44 tests
test tests::test_thread_spawn_and_join ... ok
test tests::test_mpsc_channel_message_passing ... ok
test tests::test_mpsc_multiple_producers ... ok
test tests::test_arc_mutex_concurrent_sales_tracker ... ok
test tests::test_parallel_batch_valuation ... ok
...
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

---

## অনুশীলনী (Hands-On Exercises)

1. **টাইমআউট চ্যানেল রিসিভার**:
   `rx.recv_timeout(Duration::from_millis(500))` মেথডটি পরীক্ষা করুন। যদি ৫০০ মিলিসেকেন্ডের মধ্যে কোনো মেসেজ না আসে তবে একটি সতর্কবার্তা প্রিন্ট করুন।
2. **ডায়নামিক ওয়ার্কার পুল**:
   একটি চ্যানেল-ভিত্তিক ওয়ার্কার পুল ডিজাইন করুন যেখানে ৪টি থ্রেড একসাথে একটি শেয়ার্ড চ্যানেল থেকে কাজ নিয়ে প্রসেস করবে।
3. **প্যারালাল বনাম সিরিয়াল পারফরম্যান্স**:
   ১০ লক্ষ পণ্যের একটি ইনভেন্টরি তৈরি করে সাধারণ লুপ বনাম `parallel_batch_valuation`-এর সময় মেপে দেখুন (ক্লক পরিমাপের জন্য `std::time::Instant` ব্যবহার করুন)।

---

## অধ্যায় সারসংক্ষেপ ও কনকারেন্সি চিটশিট

| আদিম উপাদান | মডিউল | মূল কাজ | থ্রেড-সেফ? |
| :--- | :--- | :--- | :--- |
| **`thread::spawn`** | `std::thread` | নতুন ওএস থ্রেড চালু | হ্যাঁ |
| **`JoinHandle<T>`** | `std::thread` | থ্রেড শেষের অপেক্ষা ও ফলাফল | হ্যাঁ |
| **`mpsc::channel`** | `std::sync::mpsc` | বার্তা আদান-প্রদান চ্যানেল | হ্যাঁ |
| **`Rc<T>`** | `std::rc` | সিঙ্গেল-থ্রেড রেফারেন্স কাউন্টিং | **না (`!Send`)** |
| **`RefCell<T>`** | `std::cell` | সিঙ্গেল-থ্রেড মিউটেবিলিটি | **না (`!Sync`)** |
| **`Arc<T>`** | `std::sync` | থ্রেড-সেফ অ্যাটমিক রেফারেন্স | হ্যাঁ (`Send + Sync`) |
| **`Mutex<T>`** | `std::sync` | পারস্পরিক বর্জন তালা | হ্যাঁ (`Send + Sync`) |
| **`Arc<Mutex<T>>`** | Combined | থ্রেড-সেফ শেয়ার্ড মিউটেবল স্টেট | হ্যাঁ (`Send + Sync`) |

পরবর্তী অধ্যায়ে আমরা নেটিভ সিনক্রোনাস থ্রেড থেকে **অ্যাসিঙ্ক রাস্ট (Async Rust) ও টোকিও (Tokio)**-তে প্রবেশ করব!
