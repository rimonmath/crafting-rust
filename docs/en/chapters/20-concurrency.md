# Chapter 20: Concurrency

## What You'll Learn
- What **Concurrency** is and how Rust delivers **Fearless Concurrency** without data races.
- Spawning native OS threads using **`std::thread::spawn`** and collecting results via **`JoinHandle<T>`**.
- Transferring environment ownership to spawned threads using the **`move`** keyword.
- Message-passing concurrency using **Channels** (**`std::sync::mpsc`**):
  - The Multiple Producer, Single Consumer model.
  - Cloning sender handles (`tx.clone()`) for multiple concurrent worker threads.
  - Safe, blocking, and non-blocking message consumption (`rx.recv()` vs `rx.try_recv()`).
- Shared-state concurrency using **`Arc<T>`** and **`Mutex<T>`**:
  - Why `Rc<T>` and `RefCell<T>` are restricted to single threads.
  - Atomic Reference Counting with **`Arc<T>`**.
  - Mutual Exclusion with **`Mutex<T>`** and automatic lock release via RAII guards (**`MutexGuard<T>`**).
  - Combining **`Arc<Mutex<T>>`** for thread-safe shared mutable state.
- The **`Send`** and **`Sync`** marker traits:
  - How Rust's type system statically prevents data races at compile time.
- Common compiler errors and concurrency pitfalls:
  - `error[E0277]: 'Rc<...>' cannot be sent between threads safely`
  - `error[E0382]: use of moved value in closure`
  - `error[E0597]: borrowed value does not live long enough`
  - Mutex poisoning and deadlocks.
- Integrating Concurrency into MiniStore:
  - Asynchronous order notifications via `OrderNotificationChannel`.
  - Multi-threaded checkout sales telemetry via `ConcurrentSalesTracker` (`Arc<Mutex<SalesMetrics>>`).
  - Parallel inventory valuation across CPU cores using `parallel_batch_valuation`.

---

## Why Do We Need This?

Until now, MiniStore executed every operation sequentially on a single thread:
1. Product lookups ran one after another.
2. Checkout transactions processed one by one.
3. Inventory reports and valuations computed iteratively on the main thread.

While sequential execution is simple to reason about, modern computers possess multi-core CPUs. In an enterprise store serving thousands of concurrent shoppers, sequential processing introduces severe bottlenecks:
- A heavy batch inventory audit blocks customers from completing their checkouts.
- Order confirmation emails and warehouse notifications freeze the UI while waiting for completion.

### The Concurrency Dilemma in Other Languages
In languages like C or C++, multi-threading is fraught with peril:
- **Data Races**: Two threads accessing the same memory location concurrently, where at least one writes, without synchronization.
- **Race Conditions**: Inconsistent program states caused by unpredictable execution order.
- **Memory Corruption**: Wild pointers, use-after-free, and segmentation faults across threads.

In garbage-collected languages (Java, Go, Python), runtime exceptions (`ConcurrentModificationException`) or subtle race conditions still require diligent manual vigilance.

### Rust's Solution: Fearless Concurrency
Rust guarantees **Fearless Concurrency**:
> If your multi-threaded Rust code compiles without errors, it is mathematically guaranteed to be free of data races!

The exact same ownership, borrowing, and lifetime rules that prevent dangling pointers in single-threaded code also eliminate data races across thread boundaries.

---

## Native Threads with `std::thread`

Rust maps its threads directly onto **native OS threads** (1:1 threading model). Each thread has its own stack and executes independently.

### 1. Spawning Threads
Use `std::thread::spawn` to launch a new thread, providing a closure with the code to execute:

```rust
use std::thread;
use std::time::Duration;

fn main() {
    thread::spawn(|| {
        for i in 1..=5 {
            println!("Hi from spawned thread: {i}");
            thread::sleep(Duration::from_millis(1));
        }
    });

    for i in 1..=3 {
        println!("Hi from main thread: {i}");
        thread::sleep(Duration::from_millis(1));
    }
}
```

If the main thread completes its execution, all spawned background threads are abruptly terminated, regardless of whether they finished their work.

### 2. Waiting with `JoinHandle`
`thread::spawn` returns a `JoinHandle<T>`, where `T` is the return type of the closure. Calling `.join()` on the handle pauses the current thread until the spawned thread terminates and returns a `Result`:

```rust
let handle = thread::spawn(|| {
    let item_price = 12000;
    let qty = 2;
    (item_price * qty) + 500 // Return value from thread
});

// `join()` blocks until the spawned thread finishes:
let total = handle.join().expect("Worker thread panicked!");
assert_eq!(total, 24500);
```

### 3. The `move` Keyword with Threads
When a thread uses local variables from its enclosing environment, Rust insists that the closure take full ownership of them using the **`move`** keyword.

```rust
let customer_name = String::from("Margaret Hamilton");

// Without `move`: COMPILER ERROR!
// The compiler cannot guarantee how long the spawned thread will run.
// It might outlive `customer_name`'s stack frame!
let handle = thread::spawn(move || {
    println!("Processing order for: {customer_name}");
});

handle.join().unwrap();
```

---

## Message Passing: Channels with `std::sync::mpsc`

One popular and safe model for concurrent programming is **message passing**, popularized by Erlang and Go:
> "Do not communicate by sharing memory; instead, share memory by communicating."

Rust's standard library provides **`mpsc`**, which stands for **Multiple Producer, Single Consumer**.

### 1. Creating Channels & Sending Messages
`mpsc::channel()` creates an asynchronous channel, returning a tuple `(Sender<T>, Receiver<T>)`:

```rust
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();

thread::spawn(move || {
    let notification = String::from("Order #901 Confirmed");
    tx.send(notification).unwrap();
    // `notification` was moved into `send`! You cannot use it here anymore.
});

// `recv()` blocks until a message is received or the sender drops:
let msg = rx.recv().expect("Sender disconnected");
println!("Received: {msg}");
```

### 2. Multiple Producers via `tx.clone()`
Because `mpsc` supports multiple producers, we can clone the sender (`tx.clone()`) and pass separate senders to multiple worker threads:

```rust
let (tx, rx) = mpsc::channel();

let tx1 = tx.clone();
let tx2 = tx.clone();
drop(tx); // Drop the original handle so rx knows when all producers are done!

thread::spawn(move || {
    tx1.send("Order #901").unwrap();
});

thread::spawn(move || {
    tx2.send("Order #902").unwrap();
});

// When all senders (tx1, tx2) drop, the receiver's loop terminates automatically:
for order_id in rx {
    println!("Processing: {order_id}");
}
```

### 3. MiniStore Notification Channel
In MiniStore (`ministore/src/concurrency.rs`), we wrap this pattern into a domain-specific notification channel:

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

## Shared-State Concurrency: `Arc<T>` & `Mutex<T>`

While message passing is excellent for pipelines and event dispatch, some architectures require multiple threads to read and mutate the *same* shared state (such as a shared sales ledger or live inventory count).

### 1. Why Not `Rc<T>`?
In Chapter 19, we used `Rc<T>` for multiple ownership. However, if you attempt to send an `Rc<T>` to another thread:
```rust
// COMPILER ERROR!
// `Rc<T>` cannot be sent between threads safely (`!Send`)
let rc = Rc::new(5);
thread::spawn(move || { println!("{rc}"); });
```
`Rc<T>` uses standard, non-atomic integer operations to update its reference count. If two threads increment the counter concurrently, memory corruption could occur.

### 2. Enter `Arc<T>`: Atomic Reference Counting
`Arc<T>` stands for **Atomically Reference Counted**. It uses CPU-level atomic operations to synchronize its reference count safely across CPU cores:
```rust
use std::sync::Arc;
use std::thread;

let shared_data = Arc::new(vec![1, 2, 3]);
let data_clone = Arc::clone(&shared_data);

thread::spawn(move || {
    println!("Thread data: {:?}", data_clone);
});
```

### 3. Mutual Exclusion with `Mutex<T>`
Just like `Rc<T>`, `Arc<T>` only grants **immutable** references (`&T`) to its contents. To mutate shared data across threads, we need a thread-safe interior mutability primitive: **`Mutex<T>`** (Mutual Exclusion).

A `Mutex` ensures that only one thread can access data at any given moment:
1. A thread calls `.lock()` to request access.
2. If another thread is currently holding the lock, the calling thread is put to sleep until the lock becomes available.
3. `.lock()` returns a `LockResult<MutexGuard<T>>`.
4. The `MutexGuard<T>` smart pointer implements:
   - `Deref` / `DerefMut`: Allows reading and writing the inner data.
   - `Drop`: **Automatically unlocks the mutex** when the guard goes out of scope!

```rust
use std::sync::Mutex;

let m = Mutex::new(10);

{
    let mut guard = m.lock().unwrap();
    *guard += 5;
} // <-- guard goes out of scope here; mutex is automatically unlocked!

assert_eq!(*m.lock().unwrap(), 15);
```

### 4. The Canonical Pair: `Arc<Mutex<T>>`
By combining `Arc` and `Mutex`, we achieve thread-safe shared mutable state across arbitrary numbers of concurrent threads:

```
              ┌───────────────────────────────┐
Thread 1 ────►│      Arc (Strong Count: 2)     │
              │              │                │
Thread 2 ────►│              ▼                │
              │    Mutex<SalesMetrics>        │
              │    (Exclusive lock guard)     │
              └───────────────────────────────┘
```

### MiniStore Implementation: `ConcurrentSalesTracker`
In `ministore/src/concurrency.rs`:

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
        // Automatically unlocked when `guard` drops!
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

for _ in 0..8 {
    let tracker_clone = tracker.clone();
    let handle = std::thread::spawn(move || {
        for _ in 0..5 {
            tracker_clone.record_sale(1000); // Record $10 sale
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

## The `Send` and `Sync` Marker Traits

How does the compiler know which types can be moved or shared across threads?
Rust defines two built-in **marker traits** in `std::marker`:

1. **`Send`**: Indicates that ownership of the type can be transferred across thread boundaries.
   - Almost all Rust types are `Send` (primitives, `String`, `Vec`, `Box`, `Arc`, `Mutex`).
   - Notable exceptions: `Rc<T>` (not atomic), raw pointers `*const T` / `*mut T`.
2. **`Sync`**: Indicates that it is safe to share references (`&T`) between multiple threads concurrently.
   - In other words: `T` is `Sync` if and only if `&T` is `Send`.
   - Primitive types, immutable structs, and `Arc<T>` are `Sync`.
   - `RefCell<T>` is `Send`, but **not `Sync`** (runtime borrow counter is not atomic).
   - `Mutex<T>` is **`Sync`** because its internal lock synchronizes concurrent mutable access.

Because these traits are automatically implemented by the compiler for composite types composed of `Send` and `Sync` components, you almost never need to implement them manually.

---

## Parallel Workloads: `parallel_batch_valuation`

Concurrency is also essential for parallel data processing (divide-and-conquer). In `ministore/src/concurrency.rs`, we divide catalog items into chunks and compute valuations across worker threads:

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

## Common Compiler & Runtime Errors

### 1. `error[E0277]: 'Rc<...>' cannot be sent between threads safely`
```rust
let counter = Rc::new(0);
thread::spawn(move || { println!("{counter}"); });
```
**Fix**: Replace `Rc<T>` with `Arc<T>`.

### 2. `error[E0382]: use of moved value in closure`
```rust
let data = Arc::new(100);
for _ in 0..2 {
    thread::spawn(move || { println!("{data}"); }); // Error: data moved on 1st iteration!
}
```
**Fix**: Clone the `Arc` before moving it into the closure:
```rust
for _ in 0..2 {
    let data_clone = Arc::clone(&data);
    thread::spawn(move || { println!("{data_clone}"); });
}
```

### 3. Deadlocks
A **deadlock** occurs when thread A holds Lock 1 and waits for Lock 2, while thread B holds Lock 2 and waits for Lock 1. Both threads are blocked forever.
**Fixes**:
- Always acquire locks in the exact same consistent order across all threads.
- Keep lock scopes as small as possible so guards are dropped quickly.

### 4. Mutex Poisoning
If a thread panics while holding a `MutexGuard`, the mutex becomes **poisoned**. Subsequent attempts to call `.lock()` will return `Err(PoisonError)`. Calling `.unwrap()` or `.expect()` propagates the panic or lets you recover by calling `poison_err.into_inner()`.

---

## MiniStore Architecture & Validation

```
ministore/
├── src/
│   ├── concurrency.rs      # OrderNotification, OrderNotificationChannel, ConcurrentSalesTracker, parallel_batch_valuation
│   ├── catalog.rs
│   ├── promotions.rs
│   ├── models/
│   ├── lib.rs              # Re-exports concurrency API & 44 unit tests
│   └── main.rs             # Demonstrates multi-threading, MPSC channels, Arc<Mutex>, and parallel valuation
```

### Running All 44 Unit Tests
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

## Hands-On Exercises

1. **Timeout Channel Receiver**:
   Investigate `rx.recv_timeout(Duration::from_millis(500))`. Modify `OrderNotificationChannel` to implement a consumer that waits at most 500ms for incoming notifications and logs a heartbeat warning if no events arrive.
2. **Dynamic Work Pool**:
   Create a channel-backed worker pool where a fixed number of worker threads (e.g. 4 threads) continuously pull inventory audit jobs from a shared `Arc<Mutex<Receiver<InventoryJob>>>`.
3. **Benchmarking Parallel vs Serial**:
   Construct an inventory of 1,000,000 items and compare the execution time of serial calculation (`.iter().map().sum()`) versus `parallel_batch_valuation` with 2, 4, and 8 worker threads using `std::time::Instant`.

---

## Chapter Summary & Concurrency Cheatsheet

| Primitive | Module | Concept | Mutability | Thread-Safe? |
| :--- | :--- | :--- | :--- | :--- |
| **`thread::spawn`** | `std::thread` | Native OS thread execution | N/A | Yes |
| **`JoinHandle<T>`** | `std::thread` | Wait for thread completion | N/A | Yes |
| **`mpsc::channel`** | `std::sync::mpsc` | Message passing channel | Move ownership | Yes |
| **`Rc<T>`** | `std::rc` | Single-threaded ref counting | Immutable | **NO (`!Send`)** |
| **`RefCell<T>`** | `std::cell` | Single-threaded interior mutability | Mutable (runtime) | **NO (`!Sync`)** |
| **`Arc<T>`** | `std::sync` | Atomic reference counting | Immutable | Yes (`Send + Sync`) |
| **`Mutex<T>`** | `std::sync` | Mutual exclusion lock | Mutable (locked) | Yes (`Send + Sync`) |
| **`Arc<Mutex<T>>`** | Combined | Multi-threaded shared mutable state | Mutable (locked) | Yes (`Send + Sync`) |

In the next chapter, we expand from native synchronous threads to **Async Rust & Tokio**, learning how non-blocking cooperative concurrency powers millions of simultaneous connections!
