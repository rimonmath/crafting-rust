# অধ্যায় ৩: রাস্টের মৌলিক ভিত্তি

## আপনি কী শিখবেন
- রাস্টে ভ্যারিয়েবল কীভাবে কাজ করে: ডিফল্ট ইমিউটেবিলিটি বনাম স্পষ্ট মিউটেবিলিটি (`mut`)।
- **ভ্যারিয়েবল শ্যাডোয়িং (Variable Shadowing)** কী এবং মিউটেবিলিটির সাথে এর মৌলিক পার্থক্য।
- রাস্টের স্কেলার টাইপস: ইন্টিজার, ফ্লোট, বুলিয়ান এবং ৪-বাইটের ইউনিকোড ক্যারেক্টার।
- কেন MiniStore-এর মতো বাণিজ্যিক সিস্টেমে টাকার হিসাব ফ্লোটের বদলে ইন্টিজার সেন্টে করা আবশ্যক।
- কম্পাউন্ড ডাটা টাইপ: **টাপল (Tuple)** এবং স্ট্যাকে সংরক্ষিত **অ্যারে (Array)**।
- ইনভেন্টরি স্টক আপডেট এবং ডিসকাউন্ট ক্যালকুলেশন লজিক বাস্তবায়ন ও টেস্ট করা।

---

## আমাদের কেন এটি প্রয়োজন?
যেকোনো বাস্তব অ্যাপ্লিকেশনের মূল কাজ ডাটা প্রসেস করা। ডাইনামিক টাইপড ভাষায় (যেমন JavaScript বা Python) ভ্যারিয়েবলের টাইপ হঠাৎ বদলে গিয়ে প্রোডাকশনে নীরব বিপর্যয় ডেকে আনে:

```javascript
// JavaScript: রানটাইমে জটিল বাগ
let price = "49.99";
let quantity = 2;
let total = price + quantity; // "49.992" — যোগের বদলে স্ট্রিং জোড়া লেগে গেল!
```

অন্যদিকে C বা C++ এর মতো প্রাচীন ভাষায় অস্পষ্ট টাইপ কনভার্সন এবং ইন্টিজার ওভারফ্লোর কারণে গুরুতর সিকিউরিটি বাগ তৈরি হয়।

রাস্টের স্ট্যাটিক টাইপ সিস্টেম খুবই নিখুঁত: টাইপ ইনফারেন্সের মাধ্যমে কম্পাইলার নিজেই সঠিক টাইপ ধরে নিতে পারে, কিন্তু কোনো ইমপ্লিসিট (অদৃশ্য) কনভার্সন অনুমোদন করে না। আপনি যদি একটি `u32` সংখ্যাকে `f64` হিসেবে ব্যবহার করতে চান, তবে আপনাকে স্পষ্টভাবে `as` দিয়ে তা রূপান্তর করতে হবে।

---

## সমস্যাটি কী?
ই-কমার্স সফটওয়্যার তৈরির সময় দুটি অতি সাধারণ সমস্যা দেখা দেয়:

1. **অনাকাঙ্ক্ষিত স্টেট পরিবর্তন (Unintended Mutation)**: অবজেক্ট বা ভ্যারিয়েবলের মান কোনো ফাংশনের ভেতর অজান্তে পরিবর্তন হয়ে যাওয়া।
2. **টাইপ রূপান্তরের অস্বস্তি (Type Transformation)**: ব্যবহারকারীর ইনপুট (যেমন HTTP কোয়েরি প্যারামিটারে আসা `"15"` স্ট্রিং) থেকে ইন্টিজার `15` সংখ্যায় রূপান্তর করা। প্রচলিত ভাষায় এর জন্য `discount_str`, `discount_num` ইত্যাদির মতো একাধিক অপ্রয়োজনীয় অস্থায়ী ভ্যারিয়েবলের নাম দিতে হয়।

রাস্ট **ডিফল্ট ইমিউটেবিলিটি** এবং **ভ্যারিয়েবল শ্যাডোয়িং**-এর মাধ্যমে এ দুটি সমস্যার দুর্দান্ত সমাধান দেয়।

---

## রাস্টের সমাধান (Rust Concept)

### ১. ইমিউটেবিলিটি ও `mut`
রাস্টে `let` দিয়ে ঘোষিত প্রতিটি ভ্যারিয়েবল ডিফল্টভাবে অপরিবর্তনশীল (Immutable):
```rust
let price = 1000;
// price = 1200; // কম্পাইলার এরর! ইমিউটেবল ভ্যারিয়েবলে দ্বিতীয়বার মান বসানো নিষেধ
```

বিজনেস লজিকে যখন কোনো ভ্যারিয়েবলের মান সত্যি বদলানো প্রয়োজন, তখন স্পষ্টভাবে তা ঘোষণা করতে হয়:
```rust
let mut stock = 10;
stock -= 1; // অনুমোদিত
```

### ২. ভ্যারিয়েবল শ্যাডোয়িং (Shadowing)
শ্যাডোয়িংয়ের মাধ্যমে একই স্কোপে `let` কীওয়ার্ড ব্যবহার করে একই নামের ভ্যারিয়েবল পুনরায় ডিক্লেয়ার করা যায়:

```rust
let discount = "15";              // টাইপ: &str (স্ট্রিং স্লাইস)
let discount: u32 = discount.parse().unwrap(); // টাইপ এখন u32 ইন্টিজার!
```

**শ্যাডোয়িং আর মিউটেবিলিটি এক জিনিস নয়**:
- শ্যাডোয়িং আগের ভ্যারিয়েবলটিকে ঢেকে দিয়ে সম্পূর্ণ নতুন একটি ভ্যারিয়েবল মেমরিতে তৈরি করে।
- এটি ভ্যারিয়েবলের **টাইপ পরিবর্তনের** সুযোগ দেয়—অস্থায়ী আজেবাজে নামের প্রয়োজন পড়ে না।
- নতুন ভ্যারিয়েবলটি আবার সম্পূর্ণ ইমিউটেবল হিসেবে সুরক্ষিত থাকে (যদি না আবার `mut` দেওয়া হয়)।

### ৩. স্কেলার টাইপস (Scalar Types)

| বিভাগ | টাইপসমূহ | বিবরণ |
| :--- | :--- | :--- |
| **আনসাইন্ড ইন্টিজার** | `u8`, `u16`, `u32`, `u64`, `u128`, `usize` | শুধু ধনাত্মক পূর্ণসংখ্যা। `usize` মেশিনের আর্কিটেকচার অনুযায়ী নির্ধারিত হয় (যেমন ৬৪-বিট)। |
| **সাইন্ড ইন্টিজার** | `i8`, `i16`, `i32`, `i64`, `i128`, `isize` | ধনাত্মক ও ঋণাত্মক সংখ্যা (Two's complement)। |
| **ফ্লোটিং পয়েন্ট** | `f32`, `f64` | দশমিক সংখ্যা (IEEE 754)। ডিফল্ট হলো `f64`। |
| **বুলিয়ান** | `bool` | `true` অথবা `false` (মেমরিতে ১ বাইট)। |
| **ক্যারেক্টার** | `char` | ৪-বাইটের ইউনিকোড স্কেলার ভ্যালু (ইমোজি ও যেকোনো ভাষার অক্ষর ধারণ করতে সক্ষম)। |

> [!IMPORTANT]
> **MiniStore-এ টাকার হিসাব**: আমরা পণ্যের দাম ফ্লোটের বদলে `u32` ইন্টিজারে সেন্ট আকারে রাখব (যেমন `$89.99` = `8999` সেন্ট)। ফ্লোটিং পয়েন্টে `0.1 + 0.2 == 0.30000000000000004` এর মতো ভগ্নাংশের গাণিতিক ভুল হয়, যা আর্থিক সফটওয়্যারে কঠোরভাবে নিষিদ্ধ।

### ৪. কম্পাউন্ড টাইপস (Compound Types)

```rust
// টাপল: নির্দিষ্ট দৈর্ঘ্য, বিভিন্ন টাইপের ডাটার সমন্বয়
let product_tuple: (u64, &str, u32) = (1001, "Keyboard", 8999);
let id = product_tuple.0; // ডট ইনডেক্স দিয়ে অ্যাক্সেস

// অ্যারে: নির্দিষ্ট দৈর্ঘ্য, একই টাইপের ডাটা, স্ট্যাকে বরাদ্দকৃত
let daily_sales: [u32; 3] = [5, 8, 2];
let day_one = daily_sales[0];
```

---

## অন্যান্য ভাষা থেকে আসলে যা জানা দরকার

| বৈশিষ্ট্য | Python / JS | Go | Java / C# | Rust |
| :--- | :--- | :--- | :--- | :--- |
| **ডিফল্ট পরিবর্তনশীলতা** | সবসময় পরিবর্তনশীল | সবসময় পরিবর্তনশীল | ডিফল্ট পরিবর্তনশীল (`final` দিতে হয়) | **ডিফল্টভাবে অপরিবর্তনশীল (Immutable)** |
| **ভ্যারিয়েবল শ্যাডোয়িং** | মান বদলে যায় | একই ব্লকে পুনরায় ডিক্লেয়ার অবৈধ | একই ব্লকে অবৈধ | **ফার্স্ট-ক্লাস ফিচার** (`let x = ...`) |
| **ক্যারেক্টার টাইপ** | ১-অক্ষরের স্ট্রিং / UTF-16 | `rune` (`int32`-এর অ্যালিয়াস) | `char` (২-বাইটের UTF-16) | `char` (৪-বাইটের সম্পূর্ণ ইউনিকোড) |
| **ফিক্সড অ্যারে** | ডায়নামিক `list` / `Array` | `[N]T` (ভ্যালু টাইপ) | `T[]` (হিপে অবজেক্ট) | `[T; N]` (স্ট্যাকে সংরক্ষিত) |

---

## ছোট উদাহরণ: শ্যাডোয়িং বনাম মিউটেশন

```rust
fn main() {
    // মিউটেশন: মান বদলালেও টাইপ কখনো বদলানো যাবে না
    let mut stock = 20;
    stock = 15; // ঠিক আছে: একই টাইপ
    // stock = "out of stock"; // এরর! টাইপ অমিল!

    // শ্যাডোয়িং: টাইপ রূপান্তর সম্ভব এবং ইমিউটেবিলিটি অক্ষুণ্ণ থাকে
    let raw_input = "  42  ";
    let raw_input: u32 = raw_input.trim().parse().unwrap();
    println!("রূপান্তরিত সংখ্যা: {}", raw_input);
}
```

---

## MiniStore-এ প্রয়োগ
MiniStore-এ পণ্যের ইনভেন্টরি ও মূল্যের হিসাব পরিচালনার জন্য আমরা এগুলো প্রয়োগ করব:
1. পণ্যের পরিচয়: আইডি (`u64`), নাম (`&str`), সেন্টে মূল্য (`u32`)।
2. লাইভ ইনভেন্টরি স্টক: পণ্য বিক্রি হলে স্টক কমানোর জন্য `mut stock_quantity: u32`।
3. ব্যবহারকারীর ডিসকাউন্ট স্ট্রিং থেকে ইন্টিজারে রূপান্তর (শ্যাডোয়িং)।
4. বিগত ৩ দিনের বিক্রির হিসাব রাখার জন্য একটি ফিক্সড সাইজ অ্যারে।

---

## কোড

### `src/main.rs`
```rust
fn main() {
    println!("=== MiniStore: Inventory & Pricing ===");

    // 1. Immutable scalar values (explicit types)
    let product_id: u64 = 1001;
    let product_name: &str = "Mechanical Keyboard";
    let base_price_cents: u32 = 8999; // $89.99

    // 2. Mutable variable (stock changes as sales occur)
    let mut stock_quantity: u32 = 25;
    println!("Product #{product_id}: {product_name}");
    println!("Price: ${:.2}", base_price_cents as f64 / 100.0);
    println!("Initial Stock: {stock_quantity}");

    // Simulate customer purchases 2 units using helper function
    let units_sold: u32 = 2;
    match reduce_stock(stock_quantity, units_sold) {
        Ok(new_stock) => {
            stock_quantity = new_stock;
            println!("Units Sold: {units_sold}");
            println!("Remaining Stock: {stock_quantity}");
        }
        Err(err) => println!("Failed to sell units: {err}"),
    }

    // 3. Variable Shadowing (transforming a discount representation)
    let discount = "10"; // user entered string "10"%
    let discount: u32 = discount.parse().unwrap_or(0); // parsed into integer 10
    let final_price_cents = calculate_discounted_price(base_price_cents, discount);
    let discount_amount = base_price_cents - final_price_cents;
    println!(
        "Discount: {discount}% (-${:.2})",
        discount_amount as f64 / 100.0
    );
    println!("Final Price: ${:.2}", final_price_cents as f64 / 100.0);

    // 4. Compound Types: Tuples and Arrays
    let item_summary: (u64, &str, u32) = (product_id, product_name, final_price_cents);
    println!("Item Summary Tuple: {:?}", item_summary);

    let last_three_days_sales: [u32; 3] = [5, 8, 2];
    let total_recent_sales: u32 =
        last_three_days_sales[0] + last_three_days_sales[1] + last_three_days_sales[2];
    println!("Total Sales (Last 3 Days): {total_recent_sales}");
}

/// Applies a percentage discount in integer arithmetic to prevent floating-point inaccuracies
fn calculate_discounted_price(price_cents: u32, discount_percent: u32) -> u32 {
    let discount_amount = (price_cents * discount_percent) / 100;
    price_cents - discount_amount
}

/// Safely decrements stock, ensuring we do not underflow
fn reduce_stock(current_stock: u32, quantity: u32) -> Result<u32, &'static str> {
    if quantity > current_stock {
        Err("Insufficient stock")
    } else {
        Ok(current_stock - quantity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discount_calculation() {
        let original_price = 10000; // $100.00
        let discounted = calculate_discounted_price(original_price, 20); // 20% off
        assert_eq!(discounted, 8000); // $80.00
    }

    #[test]
    fn test_stock_reduction_success() {
        let stock = 10;
        let updated = reduce_stock(stock, 3).expect("Should succeed");
        assert_eq!(updated, 7);
    }

    #[test]
    fn test_stock_reduction_insufficient() {
        let stock = 2;
        let result = reduce_stock(stock, 5);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient stock");
    }

    #[test]
    fn test_shadowing_type_change() {
        let input = "42";
        let input: u32 = input.parse().unwrap();
        assert_eq!(input, 42);
    }
}
```

---

## কোডটি বুঝে নেওয়া যাক

1. **সেন্টে আর্থিক হিসাব**:
   ```rust
   let discount_amount = (price_cents * discount_percent) / 100;
   ```
   রাস্টে পূর্ণসংখ্যার ভাগফল সরাসরি শূন্যের দিকে রাউন্ড হয়। সেন্টে হিসাব করায় ভগ্নাংশের অনিশ্চয়তা সম্পূর্ণ দূর হয়েছে।

2. **পার্সিংয়ের জন্য শ্যাডোয়িং**:
   ```rust
   let discount = "10";
   let discount: u32 = discount.parse().unwrap_or(0);
   ```
   এখানে `discount` শুরুতে স্ট্রিং স্লাইস থাকলেও শ্যাডোয়িংয়ের ফলে তা একটি পূর্ণসংখ্যায় রূপান্তরিত হয়। পুরনো স্ট্রিংটি এই নামে আর অ্যাক্সেস করা সম্ভব নয়, ফলে ভুল ডেটা ব্যবহারের সুযোগ নেই।

3. **টাপল ও ডিবাগ প্রিন্টিং**:
   - `item_summary.0`, `item_summary.1`: টাপলের ডাটা জিরো-ইনডেক্সড ডট নোটেশন দিয়ে পড়া যায়।
   - `{:?}`: টাপল বা অ্যারের মতো কম্পাউন্ড ডাটা প্রিন্ট করতে এই ফরম্যাট স্পেসিফায়ার ব্যবহৃত হয়।

---

## সাধারণ ভুলসমূহ

### ১. মিউটেবল ভ্যারিয়েবলের টাইপ বদলানোর চেষ্টা করা
```rust
let mut price = 100;
price = "120"; // এরর!
```
`mut` ব্যবহারের ফলে আপনি ভ্যারিয়েবলের **মান** পরিবর্তন করতে পারবেন, কিন্তু তার **টাইপ** সবসময় নির্দিষ্ট থাকবে। টাইপ বদলাতে হলে শ্যাডোয়িং (`let price = ...`) করতে হবে।

### ২. আনসাইন্ড সংখ্যায় নীরব ওভারফ্লোর প্রত্যাশা করা
C ভাষায় আনসাইন্ড সংখ্যা শূন্যের নিচে নামলে তা স্বয়ংক্রিয়ভাবে ঘুরে বড় সংখ্যায় চলে যায়। কিন্তু রাস্টে:
```rust
let stock: u32 = 0;
// let next = stock - 1; // ডিবাগ মোডে তৎক্ষণাৎ প্যানিক (Panic) করবে!
```
ডিবাগ মোডে রাস্ট সাথে সাথে এরর দেয় যাতে কোনো অবাস্তব মান ইনভেন্টরিতে প্রবেশ করতে না পারে। আমাদের `reduce_stock` ফাংশনটি আগেই `quantity > current_stock` যাচাই করে এই ঝুঁকি রোধ করেছে।

---

## কম্পাইলার এরর ও ডায়াগনস্টিক
ইমিউটেবল ভ্যারিয়েবলে নতুন মান দিতে গেলে কম্পাইলার যা বলে:

```text
error[E0384]: cannot assign twice to immutable variable `stock`
 --> src/main.rs:3:5
  |
2 |     let stock = 10;
  |         -----
  |         |
  |         first assignment to `stock`
  |         help: consider making this binding mutable: `mut stock`
3 |     stock = 5;
  |     ^^^^^^^^^ cannot assign twice to immutable variable
```

---

## নিজে অনুশীলন করুন
1. `ministore/src/main.rs`-এ ট্যাক্স পার্সেন্টেজ রাখার জন্য একটি ইমিউটেবল ভ্যারিয়েবল যোগ করুন (যেমন `let tax_rate_percent: u32 = 5;`)।
2. একটি ফাংশন লিখুন `calculate_tax(price_cents: u32, tax_rate_percent: u32) -> u32`।
3. একটি ইউনিট টেস্ট লিখুন যা যাচাই করবে `$50.00` (`5000` সেন্ট) মূল্যে `5%` হারে ট্যাক্স হয় `250` সেন্ট।
4. `cargo test` রান করে টেস্টটি সফল হয়েছে কিনা যাচাই করুন।

---

## চেকপয়েন্ট
- [x] `let`, `let mut` এবং ভ্যারিয়েবল শ্যাডোয়িংয়ের পার্থক্য স্পষ্ট হয়েছে।
- [x] স্কেলার টাইপস এবং ফ্লোটের ঝুঁকি সম্পর্কে জেনেছেন।
- [x] টাপল ও ফিক্সড-সাইজ অ্যারে ব্যবহার করতে পেরেছেন।
- [x] `cargo test` দিয়ে ইউনিট টেস্ট সফলভাবে যাচাই করেছেন।

---

## আমরা কী শিখলাম
- ডিফল্ট ইমিউটেবিলিটি কোডকে অনেক বেশি নির্ভরযোগ্য ও সুরক্ষিত করে তোলে।
- শ্যাডোয়িংয়ের মাধ্যমে অপ্রয়োজনীয় নাম ছাড়াই সুন্দরভাবে টাইপ ট্রান্সফর্ম করা যায়।
- আর্থিক বা বাণিজ্যিক সিস্টেমে স্পষ্ট টাইপ ব্যবহার করা কতটা গুরুত্বপূর্ণ।

---

## পরবর্তী অধ্যায়ে কী থাকছে
**অধ্যায় ৪: কন্ট্রোল ফ্লো**-তে আমরা শিখব কীভাবে `if / else` এক্সপ্রেশন মান রিটার্ন করতে পারে, এবং `loop`, `while`, ও `for` লুপের মাধ্যমে শপিং কার্টের আইটেম প্রসেস করতে হয়।
