# অধ্যায় ৪: কন্ট্রোল ফ্লো (Control Flow)

## আপনি কী শিখবেন
- রাস্টে স্টেটমেন্ট (Statement) এবং এক্সপ্রেশন (Expression)-এর মৌলিক পার্থক্য।
- মান প্রদানকারী (Value-yielding) এক্সপ্রেশন হিসেবে `if / else`-এর ব্যবহার (এবং কেন রাস্টে টার্নারি অপারেটরের প্রয়োজন নেই)।
- শর্তহীন লুপ `loop` এবং `break <value>` ব্যবহার করে লুপ থেকে সরাসরি মান বের করে আনার কৌশল।
- একাধিক নেস্টেড লুপের মধ্যে নির্দিষ্ট লুপ ব্রেক করার জন্য **লুপ লেবেল (Loop Labels)**-এর ব্যবহার (যেমন `'outer: loop`)।
- শর্তসাপেক্ষ লুপ হিসেবে `while`-এর ব্যবহার।
- রেঞ্জ এবং অ্যারের ওপর নিরাপদ ও ইডিওম্যাটিক উপায়ে `for` লুপের মাধ্যমে পুনরাবৃত্তি (Iteration)।
- MiniStore-এ সম্পূর্ণ শপিং কার্ট ও অর্ডার প্রসেসিং ইঞ্জিন বাস্তবায়ন: মোট মূল্য নির্ণয়, শর্তসাপেক্ষ টিয়ার্ড ডিসকাউন্ট, ডেলিভারি চার্জ নির্ধারণ এবং ওয়্যারহাউস প্যাকেজিং সিমুলেশন।

---

## আমাদের কেন এটি প্রয়োজন?
বাস্তব সফটওয়্যার কখনো সোজা লাইনে একমুখী নির্দেশনায় চলে না। সফটওয়্যারে নানা ধরণের সিদ্ধান্ত নিতে হয় এবং বিভিন্ন ডাটার ওপর বারবার অপারেশন চালাতে হয়:
- কার্টের মোট মূল্যের ওপর ভিত্তি করে বিশেষ ডিসকাউন্ট নির্ধারণ করা।
- নির্দিষ্ট অংকের বেশি কেনাকাটা করলে বিনামূল্যে হোম ডেলিভারি দেওয়া।
- কাস্টমারের শপিং কার্টের প্রতিটি আইটেম স্ক্যান করে মোট বিল ও ভ্যাট হিসাব করা।
- ওয়্যারহাউসে পণ্য কার্টনে প্যাক করে ডেলিভারি কিউতে পাঠানো।

অধিকাংশ প্রচলিত ভাষায় (যেমন C, Java, বা Python) `if` বা `switch` হলো **স্টেটমেন্ট (Statement)**। এগুলো শুধু সাইড-ইফেক্ট তৈরি করে (যেমন ব্লকের বাইরে আগে থেকে ঘোষিত কোনো ভ্যারিয়েবলের মান পরিবর্তন করা):

```javascript
// JavaScript / Java: মিউটেবল ভ্যারিয়েবলসহ সাধারণ স্টেটমেন্ট
let discountRate;
if (subtotal > 10000) {
    discountRate = 0.15;
} else {
    discountRate = 0.05;
}
```

খেয়াল করুন, এখানে `discountRate` ভ্যারিয়েবলটিকে বাধ্য হয়ে মান ছাড়া অথবা পরিবর্তনশীল (`let` / `var`) হিসেবে ঘোষণা করতে হয়েছে। যদি কোনো ডেভেলপার ভুলবশত `else` কন্ডিশন লিখতে ভুলে যায় বা কোডের নিচে অসাবধানতাবশত এর মান বদলে দেয়, তবে অ্যাপ্লিকেশনে নীরব ও মারাত্মক বাগ সৃষ্টি হয়।

রাস্ট একে সম্পূর্ণ ভিন্ন ও কার্যকর রূপ দিয়েছে: **`if` হলো একটি এক্সপ্রেশন (Expression)**। অর্থাৎ এটি সরাসরি ফলাফল বা মান ফেরত দেয়। এর ফলে ভ্যারিয়েবলটি ১০০% ইমিউটেবল রাখা সম্ভব:

```rust
// Rust: মান প্রদানকারী এক্সপ্রেশন, যা সরাসরি ইমিউটেবল ভ্যারিয়েবলে বসে যায়
let discount_rate = if subtotal > 10000 { 15 } else { 5 };
```

---

## সমস্যাটি কী?
MiniStore-এর অর্ডার প্রসেসিং লজিক তৈরির সময় আমরা বাস্তবধর্মী তিনটি চ্যালেঞ্জের মুখোমুখি হই:

1. **যাচাইয়ের সময় অনাকাঙ্ক্ষিত স্টেট মিউটেশন**: অর্ডারের সাবটোটাল, ছাড় এবং ডেলিভারি ফি বের করতে গিয়ে অপ্রয়োজনীয় মিউটেবল ভ্যারিয়েবল তৈরি হওয়া এড়ানো।
2. **লুপ থেকে মান গ্রহণ করা**: কার্টের শত শত পণ্যের মধ্য থেকে নির্দিষ্ট পণ্যটি (যেমন সবচেয়ে দামি আইটেম) খুঁজে বের করতে কোনো বাড়তি ফ্ল্যাগ বা লুপের বাইরের খালি ভ্যারিয়েবল ছাড়া লুপ থেকেই সরাসরি মান রিটার্ন পাওয়া।
3. **অ্যারে ইনডেক্স ভুলের ঝুঁকি (Index Out of Bounds)**: সি-স্টাইলের `for (int i = 0; i <= count; i++)` লুপে এক ঘর কম-বেশির ভুলে প্রায়ই মেমোরি ক্র্যাশ বা প্যানিক হয়। আমাদের এমন লুপ দরকার যা গাণিতিকভাবে ইনডেক্স লিমিট অতিক্রম করা অসম্ভব করে তোলে।

---

## রাস্টের সমাধান (Rust Concept)

### ১. স্টেটমেন্ট বনাম এক্সপ্রেশন (Statements vs Expressions)
রাস্ট মূলত একটি **এক্সপ্রেশন-ভিত্তিক ভাষা (Expression-based Language)**:
- **স্টেটমেন্ট (Statement)**: এমন কিছু নির্দেশ যা কোনো কাজ সম্পাদন করে কিন্তু কোনো মান ফেরত দেয় না। স্টেটমেন্টের শেষে সেমিকোলন (`;`) থাকে। যেমন `let x = 5;` একটি স্টেটমেন্ট।
- **এক্সপ্রেশন (Expression)**: যা একটি চূড়ান্ত মান মূল্যায়ন (evaluate) করে। এক্সপ্রেশনের শেষে কোনো সেমিকোলন বসে না। যদি আপনি কোনো এক্সপ্রেশনের শেষে সেমিকোলন দেন, তবে তা স্টেটমেন্টে পরিণত হয় এবং ফলাফল হিসেবে খালি **ইউনিট টাইপ** `()` ফেরত দেয়।

```rust
// { ... } ব্লকের শেষ লাইনে সেমিকোলন নেই, তাই এটি ১৫ প্রদান করে
let total = {
    let base = 10;
    let tax = 5;
    base + tax // টেইল এক্সপ্রেশন (Tail Expression): ১৫ রিটার্ন করে
};
```

### ২. এক্সপ্রেশন হিসেবে `if / else`
যেহেতু `if` একটি এক্সপ্রেশন, তাই এর প্রতিটি শাখা (Branch) থেকে **একই নির্দিষ্ট টাইপের মান** ফেরত দিতে হবে:

```rust
let shipping_fee = if order_total >= 10000 {
    0
} else {
    599
};
```

> [!IMPORTANT]
> **টার্নারি অপারেটরের অনুপস্থিতি**: রাস্টে `condition ? a : b` সিনট্যাক্স নেই। কারণ `if condition { a } else { b }` এক্সপ্রেশনটি কোনো প্রকার অতিরিক্ত পারফরম্যান্স লস ছাড়াই আরও পরিচ্ছন্নভাবে একই কাজ করে।

যদি কোনো ভ্যারিয়েবলে মান নির্ধারণের সময় আপনি `else` বাদ দেন, তবে কম্পাইলার এরর দেবে। কারণ শর্ত মিথ্যা হলে ভ্যারিয়েবলটি কী মান গ্রহণ করবে তা কম্পাইলার নিশ্চিত হতে পারে না।

### ৩. `loop` দিয়ে মান রিটার্ন নেওয়া
শর্তহীন অনন্ত লুপ চালানোর জন্য রাস্টে রয়েছে `loop`:
```rust
loop {
    println!("ব্রেক না করা পর্যন্ত এটি চলতেই থাকবে");
    break;
}
```
রাস্টে `loop`-ও একটি এক্সপ্রেশন! আপনি `break`-এর সাথে যেকোনো মান পাঠিয়ে দিতে পারেন, এবং পুরো `loop` এক্সপ্রেশনটি সেই মানটি প্রদান করবে:

```rust
let mut counter = 0;
let result = loop {
    counter += 1;
    if counter == 10 {
        break counter * 2; // লুপটি সমাপ্ত করে ২০ মান ফেরত দেবে
    }
};
```

### ৪. লুপ লেবেল (Loop Labels)
যখন একটি লুপের ভেতর আরেকটি লুপ (Nested loop) থাকে, তখন সাধারণ `break` বা `continue` ভেতরের লুপটির ওপর কাজ করে। কিন্তু ভেতরের লুপ থেকে বাইরের লুপটি থামাতে চাইলে রাস্টে **লুপ লেবেল** (একটি সিঙ্গেল কোট `'` দিয়ে শুরু হয়) ব্যবহার করা হয়:

```rust
'outer: loop {
    'inner: loop {
        break 'outer; // সরাসরি বাইরের 'outer লুপটিকে থামিয়ে বের হয়ে যাবে!
    }
}
```

### ৫. `while` লুপ
শর্ত যাচাই করে পুনরাবৃত্তি চালানোর জন্য `while` ব্যবহার করা হয়:

```rust
let mut queue = 3;
while queue > 0 {
    println!("অর্ডার প্রসেস হচ্ছে: #{queue}");
    queue -= 1;
}
```

### ৬. `for` লুপের ইডিওম্যাটিক ব্যবহার
ইনডেক্স ধরে ধরে ঘোরার চেয়ে (`while i < items.len()`), রাস্টের স্ট্যান্ডার্ড উপায় হলো `for .. in`:

```rust
// অ্যারের উপাদানের ওপর পুনরাবৃত্তি
for item in cart {
    println!("{}", item.0);
}

// এক্সক্লুসিভ রেঞ্জ 0..3 (0, 1, 2)
for i in 0..3 {
    println!("ধাপ {i}");
}

// ইনক্লুসিভ রেঞ্জ 1..=5 (1, 2, 3, 4, 5)
for i in 1..=5 {
    println!("গণনা {i}");
}
```

রাস্টে `for` ব্যবহারের দুটি প্রধান সুবিধা:
1. **নিরাপদ**: সীমা ছাড়িয়ে রানটাইম প্যানিক বা ক্র্যাশ হওয়ার কোনো সুযোগ নেই।
2. **দ্রুত ও পারফরম্যান্ট**: রাস্ট কম্পাইলার আগে থেকেই বাউন্ডারি প্রমাণ করে ফেলে, ফলে প্রতিটি ধাপে রানটাইম বাউন্ডস চেকিং করার প্রয়োজন পড়ে না (Zero-cost abstraction)।

---

## অন্যান্য ভাষা থেকে আসলে যা জানা দরকার

| ধারণা | Python | Go | Java / C# / C++ | Rust |
| :--- | :--- | :--- | :--- | :--- |
| **এক্সপ্রেশন হিসেবে `if`** | `a if cond else b` (শুধুমাত্র টার্নারি) | শুধুই স্টেটমেন্ট | `cond ? a : b` (টার্নারি) | **নেটিভ `if / else` এক্সপ্রেশন** |
| **ব্রাঞ্চ টাইপ যাচাই** | ডাইনামিক টাইপ হতে পারে | প্রযোজ্য নয় | টার্নারিতে সামঞ্জস্যপূর্ণ টাইপ লাগে | **প্রতিটি আর্ম থেকে অবিকল এক টাইপ আসতে হবে** |
| **লুপ থেকে মান রিটার্ন** | সম্ভব নয় | সম্ভব নয় | সম্ভব নয় | **`loop` এর ভেতরে `break <value>;`** |
| **লুপ লেবেল** | নেই | লেবেলসহ `break Label` | `label: for(...)` ও `break label;` | **`'label: loop` ও `break 'label;`** |
| **অ্যারে আইটারেশন** | `for item in items:` | `for _, item := range items` | `for (T item : items)` | **`for item in items` (জিরো-কস্ট আইটারেটর)** |

---

## ছোট উদাহরণ (Small Example)

```rust
fn main() {
    // ১. ইমিউটেবল ভ্যারিয়েবলে `if` এক্সপ্রেশন থেকে মান নির্ধারণ
    let cart_value = 12000; // ১২০ ডলার (সেন্টে)
    let discount_pct = if cart_value > 10000 { 15 } else { 5 };
    println!("ডিসকাউন্ট: {discount_pct}%");

    // ২. `loop` এক্সপ্রেশন থেকে ফলাফল বের করে নেওয়া
    let mut attempt = 0;
    let auth_token = loop {
        attempt += 1;
        if attempt == 3 {
            break "TOKEN_SECURE_XYZ_99";
        }
    };
    println!("{attempt} বার চেষ্টার পর টোকেন পাওয়া গেছে: {auth_token}");
}
```

---

## MiniStore-এ প্রয়োগ
এই অধ্যায়ে MiniStore একটি পূর্ণাঙ্গ **অর্ডার ও কার্ট প্রসেসিং** প্ল্যাটফর্মে রূপ নিচ্ছে:
1. কার্টের পণ্যগুলোকে `(&str, u32, u32)` টাপলের একটি ফিক্সড-সাইজ অ্যারেতে রাখা হয়েছে, যার মধ্যে রয়েছে `(নাম, প্রতি ইউনিটের দাম সেন্টে, সংখ্যা)`।
2. নিখুঁত `for` লুপের সাহায্যে কার্টের মোট সাবটোটাল মূল্য হিসাব করা।
3. `if / else` এক্সপ্রেশন ব্যবহার করে ভলিউম ডিসকাউন্ট (০%, ৫%, ১০% বা ১৫%) নির্ধারণ করা।
4. শিপিং চার্জ নির্ধারণ: অর্ডার যদি ১০০ ডলার বা তার বেশি হয় তবে ফ্রি শিপিং; নতুবা সাধারণ ৫.৯৯ ডলার।
5. `loop` এক্সপ্রেশনের ভেতর `break value` দিয়ে কার্টের সবচেয়ে দামি সিঙ্গেল আইটেমটি চিহ্নিত করা।
6. নেস্টেড লুপ এবং **লুপ লেবেল** ব্যবহার করে শিপিং বক্সে প্রতিটিতে সর্বোচ্চ ৩টি পণ্য প্যাকেট করার সিমুলেশন।
7. প্রস্তুতকৃত অর্ডারগুলো `while` লুপের মাধ্যমে ক্যুরিয়ারে ডিসপ্যাচ করা।

---

## কোড (Code)

### `src/main.rs`
```rust
fn main() {
    println!("=== MiniStore: Cart & Order Processing ===");

    // Cart representation using fixed-size array of tuples:
    // (item_name: &str, unit_price_cents: u32, quantity: u32)
    let cart: [(&str, u32, u32); 4] = [
        ("Mechanical Keyboard", 8999, 1),
        ("USB-C Cable", 1299, 2),
        ("Mouse Pad", 1999, 1),
        ("Keycap Puller", 599, 3),
    ];

    println!("--- Cart Items ---");
    // 1. `for` loop: idiomatic iteration over array collections
    for item in cart {
        let (name, price_cents, qty) = item;
        let item_total = price_cents * qty;
        println!(
            "- {name} x {qty} @ ${:.2} each = ${:.2}",
            price_cents as f64 / 100.0,
            item_total as f64 / 100.0
        );
    }

    let subtotal_cents = calculate_subtotal(&cart);
    println!("\nCart Subtotal: ${:.2}", subtotal_cents as f64 / 100.0);

    // 2. `if / else` expression: assigns discount percentage based on subtotal tiers
    let discount_percent: u32 = calculate_tier_discount(subtotal_cents);
    let discount_amount_cents = (subtotal_cents * discount_percent) / 100;
    let discounted_subtotal = subtotal_cents - discount_amount_cents;

    println!(
        "Tier Discount: {discount_percent}% (-${:.2})",
        discount_amount_cents as f64 / 100.0
    );
    println!(
        "Discounted Subtotal: ${:.2}",
        discounted_subtotal as f64 / 100.0
    );

    // 3. `if` expression to compute shipping cost (free shipping over $100.00 = 10000 cents)
    let shipping_fee_cents: u32 = if discounted_subtotal >= 10000 {
        0
    } else {
        599 // $5.99 standard shipping
    };

    if shipping_fee_cents == 0 {
        println!("Shipping: FREE (Orders over $100 qualify for free shipping)");
    } else {
        println!("Shipping: ${:.2}", shipping_fee_cents as f64 / 100.0);
    }

    let total_order_cents = discounted_subtotal + shipping_fee_cents;
    println!(
        "Final Order Total: ${:.2}",
        total_order_cents as f64 / 100.0
    );

    // 4. `loop` with `break value`: identify the highest-value single item in cart
    let most_expensive_item = find_most_expensive_item(&cart);
    println!(
        "Highest Value Item: {} (${:.2})",
        most_expensive_item.0,
        most_expensive_item.1 as f64 / 100.0
    );

    // 5. Nested loop with loop label: batch packaging items into boxes
    println!("\n--- Packaging Simulation (Nested Loops with Labels) ---");
    let mut total_packed = 0;
    let total_items_count: u32 = 7; // 1 + 2 + 1 + 3

    'outer_box: loop {
        println!("Opened a new shipping box...");

        'pack_items: loop {
            total_packed += 1;
            println!("  Packed item {total_packed} of {total_items_count}");

            if total_packed >= total_items_count {
                println!("All items safely packed!");
                break 'outer_box; // breaks the outer loop directly!
            }

            // Each shipping box holds a maximum of 3 items
            if total_packed % 3 == 0 {
                println!("  Box full (3 items). Sealing box.");
                break 'pack_items; // breaks inner loop to start next box
            }
        }
    }

    // 6. `while` loop: inventory dispatch queue
    println!("\n--- Warehouse Dispatch Queue (while loop) ---");
    let mut orders_in_queue = 3;
    while orders_in_queue > 0 {
        println!("Dispatching order #{orders_in_queue} to carrier...");
        orders_in_queue -= 1;
    }
    println!("All pending orders dispatched!");
}

/// Calculates the subtotal price in cents for an array of cart items
pub fn calculate_subtotal(cart: &[(&str, u32, u32)]) -> u32 {
    let mut total = 0;
    for &(_, price_cents, quantity) in cart {
        total += price_cents * quantity;
    }
    total
}

/// Returns a discount percentage (0, 5, 10, or 15) using an `if / else` expression
pub fn calculate_tier_discount(subtotal_cents: u32) -> u32 {
    if subtotal_cents >= 15000 {
        15 // 15% discount for orders >= $150
    } else if subtotal_cents >= 10000 {
        10 // 10% discount for orders >= $100
    } else if subtotal_cents >= 5000 {
        5 // 5% discount for orders >= $50
    } else {
        0 // No discount
    }
}

/// Computes shipping fee: free if order meets threshold, standard otherwise
pub fn calculate_shipping_fee(discounted_subtotal_cents: u32) -> u32 {
    if discounted_subtotal_cents >= 10000 {
        0
    } else {
        599
    }
}

/// Scans the cart using a `loop` expression with `break value` to return the highest-priced item
pub fn find_most_expensive_item<'a>(cart: &[(&'a str, u32, u32)]) -> (&'a str, u32) {
    if cart.is_empty() {
        return ("None", 0);
    }

    let mut index = 0;
    let mut highest = cart[0];

    loop {
        if index >= cart.len() {
            break (highest.0, highest.1);
        }

        if cart[index].1 > highest.1 {
            highest = cart[index];
        }

        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_subtotal() {
        let sample_cart = [
            ("Item A", 1000, 2), // 2000
            ("Item B", 2500, 1), // 2500
        ];
        assert_eq!(calculate_subtotal(&sample_cart), 4500);
    }

    #[test]
    fn test_tier_discounts() {
        assert_eq!(calculate_tier_discount(16000), 15);
        assert_eq!(calculate_tier_discount(15000), 15);
        assert_eq!(calculate_tier_discount(12000), 10);
        assert_eq!(calculate_tier_discount(10000), 10);
        assert_eq!(calculate_tier_discount(7500), 5);
        assert_eq!(calculate_tier_discount(5000), 5);
        assert_eq!(calculate_tier_discount(4999), 0);
    }

    #[test]
    fn test_shipping_fee_threshold() {
        assert_eq!(calculate_shipping_fee(10000), 0);
        assert_eq!(calculate_shipping_fee(15000), 0);
        assert_eq!(calculate_shipping_fee(9999), 599);
        assert_eq!(calculate_shipping_fee(0), 599);
    }

    #[test]
    fn test_find_most_expensive_item() {
        let sample_cart = [("Cable", 500, 2), ("Monitor", 25000, 1), ("Mouse", 3000, 1)];
        let (name, price) = find_most_expensive_item(&sample_cart);
        assert_eq!(name, "Monitor");
        assert_eq!(price, 25000);
    }
}
```

---

## কোড পর্যালোচনা (Understanding The Code)

1. **রিটার্ন এক্সপ্রেশন হিসেবে `if / else`**:
   ```rust
   pub fn calculate_tier_discount(subtotal_cents: u32) -> u32 {
       if subtotal_cents >= 15000 {
           15
       } else if subtotal_cents >= 10000 {
           10
       } else if subtotal_cents >= 5000 {
           5
       } else {
           0
       }
   }
   ```
   এখানে লক্ষ্য করুন কোনো `return` কিওয়ার্ড বা সংখ্যার শেষে কোনো সেমিকোলন ব্যবহার করা হয়নি। পুরো `if / else if / else` ব্লকটি একটি সিঙ্গেল এক্সপ্রেশন হিসেবে কাজ করছে এবং স্বয়ংক্রিয়ভাবে ফাংশন থেকে মানটি রিটার্ন করছে।

2. **মানসহ লুপ থেকে প্রস্থান (Break with value)**:
   ```rust
   loop {
       if index >= cart.len() {
           break (highest.0, highest.1);
       }
       ...
   }
   ```
   `break (highest.0, highest.1);` নির্দেশটি শুধু লুপ থামায় না, একই সাথে পুরো `loop` এক্সপ্রেশনের চূড়ান্ত মান হিসেবে `(&str, u32)` টাপলটি ফেরত দেয়।

3. **লুপ লেবেল (Loop Labels)**:
   ```rust
   'outer_box: loop {
       'pack_items: loop {
           if total_packed >= total_items_count {
               break 'outer_box;
           }
       }
   }
   ```
   লেবেল থাকার কারণে কোনো কৃত্রিম বুলিয়ান ফ্ল্যাগ ভ্যারিয়েবল (যেমন `let mut should_exit = false;`) ছাড়াই খুব সহজেই যেকোনো গভীরতার নেস্টেড লুপ থেকে সরাসরি বাইরের লুপ ব্রেক করা যায়।

---

## সাধারণ ভুলসমূহ (Common Mistakes)

### ১. `if / else`-এর শাখাগুলোতে অসম টাইপের মান প্রদান
`if / else` এক্সপ্রেশনের প্রতিটি শাখা থেকে অবিকল একই টাইপের মান ফেরত আসতে হবে:

```rust
// কম্পাইলার এরর! টাইপের অসঙ্গতি
let fee = if is_vip {
    0 // ইন্টিজার
} else {
    "standard" // স্ট্রিং স্লাইস!
};
```

### ২. টেইল এক্সপ্রেশনের শেষে সেমিকোলন দেওয়া
ফাংশনের বা ব্লকের শেষ লাইনে সেমিকোলন দিয়ে দিলে তা আর এক্সপ্রেশন থাকে না, স্টেটমেন্ট হয়ে যায় এবং `()` (ইউনিট টাইপ) প্রদান করে:

```rust
fn get_discount() -> u32 {
    let subtotal = 10000;
    if subtotal > 5000 {
        10; // ভুল! সেমিকোলন দেওয়ার ফলে এটি () রিটার্ন করে, যা u32-এর সাথে মেলে না
    } else {
        0;
    }
}
```

---

## কম্পাইলার এরর (Compiler Errors)
কোনো ভ্যারিয়েবলে মান বসানোর সময় যদি আপনি `else` ছাড়া শুধু `if` ব্যবহার করেন, তাহলে কম্পাইলার কী বলে?

```rust
fn main() {
    let condition = true;
    let score = if condition { 100 };
}
```

কম্পাইলার এরর আউটপুট:
```text
error[E0317]: `if` may be missing an `else` clause
 --> src/main.rs:3:17
  |
3 |     let score = if condition { 100 };
  |                 ^^^^^^^^^^^^^^^^^^^^ expected `()`, found integer
  |
  = note: `if` expressions without `else` have the type `()`
  = help: consider adding an `else` block that evaluates to the expected type
```
যেহেতু `condition` মিথ্যা হলে `score` এর মান কী হবে তা কম্পাইলার অনুমান করতে পারে না, তাই `else` ছাড়া যেকোনো `if` কেবল `()` রিটার্ন করতে পারে।

---

## অনুশীলন (Practice)
1. `calculate_shipping_fee` ফাংশনটিকে আপডেট করুন যাতে এটি আরেকটি প্যারামিটার `is_express: bool` গ্রহণ করে। যদি গ্রাহক এক্সপ্রেস ডেলিভারি চায়, তবে স্ট্যান্ডার্ড ফির সাথে অতিরিক্ত `1000` সেন্ট (১০ ডলার) যুক্ত হবে; তবে মোট অর্ডার ১০০ ডলারের বেশি হলে এক্সপ্রেস চার্জ হবে মাত্র `500` সেন্ট।
2. একটি নতুন টেস্ট ফাংশন `test_express_shipping()` লিখুন যা উভয় ক্ষেত্র সফলভাবে যাচাই করে।
3. `cargo test` চালিয়ে আপনার লেখা টেস্ট পাস হয়েছে কিনা নিশ্চিত করুন।

---

## চেকবক্স (Checkpoint)
- [x] স্টেটমেন্ট (সেমিকোলনযুক্ত) এবং এক্সপ্রেশনের (মান প্রদানকারী) পার্থক্য আয়ত্ত করেছি।
- [x] টার্নারি অপারেটরের বদলে রাস্টের নিজস্ব `if / else` এক্সপ্রেশন ব্যবহার করেছি।
- [x] `loop` থেকে `break value` দিয়ে হিসাবের ফলাফল বের করেছি।
- [x] লুপ লেবেল (`'label:`) দিয়ে নেস্টেড লুপ সফলভাবে নিয়ন্ত্রণ করেছি।
- [x] কোনো ইনডেক্স এরর ছাড়াই নিরাপদে `for` লুপে ডাটা প্রসেস করেছি।
- [x] `cargo test` দিয়ে সবগুলো ইউনিট টেস্ট ভেরিফাই করেছি।

---

## আমরা কী শিখলাম
- রাস্টের এক্সপ্রেশন-ভিত্তিক ডিজাইন কোডের অপ্রয়োজনীয় মিউটেশন দূর করে সফটওয়্যারকে অনেক বেশি নির্ভরযোগ্য ও সুরক্ষিত করে তোলে।
- টার্নারি অপারেটর ছাড়াই `if / else` দিয়ে শক্তিশালী ও টাইপ-সেফ কোড লেখা যায়।
- রাস্টের `for` লুপ স্বয়ংক্রিয়ভাবে বাউন্ডস চেকিং নিশ্চিত করে, ফলে কোনো ইনডেক্সিং বাগ তৈরি হতে পারে না।

---

## পরবর্তী অধ্যায়ে কী আসছে
**অধ্যায় ৫: স্ট্রাক্ট: বাস্তব জিনিসের মডেলিং (Structs: Modeling Real Things)**-এ আমরা কাঁচা টাপল এবং অ্যারের গণ্ডি পেরিয়ে আমাদের নিজস্ব ডোমেন মডেল (`Product`, `CartItem`, `Order`) তৈরি করব এবং তাতে মেথড ইমপ্লিমেন্টেশন শিখব।
