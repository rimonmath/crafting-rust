# Chapter 21: Async Rust

## What You'll Learn
- What **Asynchronous Programming** is and how it differs from OS multi-threading.
- How Rust achieves **Zero-Cost Futures** with cooperative, poll-based state machines.
- The anatomy of the **`Future`** trait (`poll`, `Pin`, `Context`, and `Poll`).
- Why Rust does not include a built-in async runtime, and how **Tokio** powers the async ecosystem.
- Writing asynchronous functions using **`async fn`** and yielding execution via the **`.await`** operator.
- Running concurrent futures in parallel using **`tokio::join!`**.
- Spawning lightweight green tasks using **`tokio::spawn`**.
- Asynchronous message passing using **`tokio::sync::mpsc`**.
- Common async pitfalls and how to avoid them:
  - Blocking the async worker pool with synchronous operations (`std::thread::sleep` vs `tokio::time::sleep`).
  - Holding synchronous `MutexGuard` locks across `.await` points.
  - Forgetting to `.await` a future (the lazy future trap).
- Integrating Async Rust into MiniStore:
  - Simulating non-blocking payment gateway roundtrips with `MockPaymentClient`.
  - Concurrently querying distributed warehouses with `WarehouseClient` and `tokio::join!`.
  - Building an asynchronous event bus with `AsyncNotificationBus`.
  - Implementing an end-to-end `async_checkout` workflow.

---

## Why Do We Need This?

In Chapter 20, we learned how to achieve concurrency using native OS threads (`std::thread::spawn`). While OS threads are ideal for **CPU-bound tasks** (e.g. video rendering, cryptography, heavy math), they become terribly inefficient for **I/O-bound tasks**.

### The I/O-Bound Bottleneck
In an e-commerce platform like MiniStore, real-world services spend 95% of their time waiting for external responses:
- Waiting for credit card networks and payment gateways.
- Waiting for inventory databases and warehouse webhooks.
- Waiting for shipping courier APIs.

If you spawn an OS thread for each waiting connection:
1. **Memory Overhead**: Each OS thread allocates a stack (typically 2MB). Spawning 10,000 idle threads consumes 20GB of RAM!
2. **Context-Switching Penalties**: The operating system kernel must constantly swap CPU register states across thousands of threads, grinding throughput to a halt.

```
OS Threads (Preemptive / Heavy):
Thread 1 (2MB stack) ────► [Waiting for Payment API...] (Idle)
Thread 2 (2MB stack) ────► [Waiting for Warehouse API...] (Idle)
Thread 3 (2MB stack) ────► [Waiting for Database...] (Idle)
```

### The Async Solution: Cooperative Green Tasks
Instead of dedicating a heavy OS thread to every waiting request, **Asynchronous Programming** uses cooperative multitasking:
- A single OS thread can juggle thousands of simultaneous I/O tasks.
- When a task needs to wait for a network response, it pauses itself and yields the CPU to other ready tasks.
- When the network response arrives, the runtime wakes the task back up.

```
Async Runtime (Cooperative / Lightweight):
Single OS Thread ──► [Task 1: Process Cart] ──► [Task 2: Send Email] ──► [Task 1: Payment Arrived!]
Tasks weigh hundreds of BYTES, not megabytes!
```

---

## The Anatomy of Async in Rust

Rust's async model is fundamentally unique compared to languages like JavaScript, Go, or C#.

### 1. Futures are Lazy (Zero-Cost)
In JavaScript, a `Promise` starts executing the moment it is constructed. In Rust:
> **A Future does absolutely nothing unless it is polled or `.await`ed!**

If you call an async function without `.await`:
```rust
async fn process_payment() -> String {
    String::from("TX-OK")
}

// WARNING: Does not execute! Returns an unpolled Future struct:
let future = process_payment(); 
```
Because futures are lazy, they cost zero runtime overhead until you explicitly schedule them.

### 2. The `Future` Trait
Behind the scenes, every `async fn` is compiled into an anonymous state machine that implements the standard `std::future::Future` trait:

```rust
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

- When the runtime calls `poll`:
  - If the operation completed, it returns `Poll::Ready(result)`.
  - If the operation is still waiting (e.g. data hasn't arrived over the network), it returns `Poll::Pending` and registers a `Waker` with the runtime.

---

## The Async Runtime: Why Rust Needs Tokio

The Rust standard library intentionally includes only the core definitions (`Future`, `Poll`, `Pin`). It does **not** include an async runtime or event loop in the standard library.

This separation of concerns allows developers to choose the optimal runtime for their specific environment (e.g. embedded microcontrollers vs high-throughput servers).

In the server and application ecosystem, **Tokio** is the industry standard runtime:
- High-performance, multi-threaded work-stealing task scheduler.
- Non-blocking asynchronous timers, networking (`tokio::net`), and file I/O (`tokio::fs`).
- Asynchronous synchronization primitives (`tokio::sync`).

### Adding Tokio to `Cargo.toml`
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

### The `#[tokio::main]` Macro
To run async code in your application's entrypoint, annotate `main` with `#[tokio::main]`:

```rust
#[tokio::main]
async fn main() {
    println!("Hello from the Tokio async runtime!");
}
```
This macro transforms your synchronous OS `main` function into code that initializes the Tokio runtime and blocks on your asynchronous block.

---

## Non-Blocking I/O: `async fn` and `.await`

Writing async code in Rust feels just like writing synchronous code, with two additions:
1. `async fn`: Marks a function as returning a `Future`.
2. `.await`: Pauses execution until the future resolves, allowing other tasks to run on the thread in the meantime.

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn fetch_stock_online(sku: &str) -> u32 {
    // Non-blocking asynchronous sleep (simulates network latency)
    sleep(Duration::from_millis(50)).await;
    
    match sku {
        "TECH-KEY-001" => 15,
        _ => 0,
    }
}
```

> [!WARNING]
> Never call `std::thread::sleep` inside an `async fn`!
> `std::thread::sleep` blocks the entire underlying OS thread, preventing all other async tasks scheduled on that thread from executing. Always use `tokio::time::sleep`.

---

## Running Concurrent Futures: `tokio::join!`

What if you need to perform multiple async operations at the same time?

### Sequential Execution (Slow)
```rust
// Runs warehouse A, waits for it, then runs warehouse B:
let stock_a = warehouse_a.check_stock(sku).await?; // Takes 50ms
let stock_b = warehouse_b.check_stock(sku).await?; // Takes 50ms
// Total time: 100ms!
```

### Concurrent Execution via `tokio::join!` (Fast)
The `tokio::join!` macro takes multiple futures and executes them concurrently on the same task/runtime:

```rust
// Both futures start simultaneously!
let (res_a, res_b) = tokio::join!(
    warehouse_a.check_stock(sku),
    warehouse_b.check_stock(sku)
);
// Total time: ~50ms (the duration of the slowest future)!
```

In `ministore/src/async_services.rs`:
```rust
pub async fn aggregate_warehouse_stock(
    warehouse_a: &WarehouseClient,
    warehouse_b: &WarehouseClient,
    sku: &str,
) -> Result<u32, StoreError> {
    let (res_a, res_b) = tokio::join!(
        warehouse_a.check_stock(sku),
        warehouse_b.check_stock(sku)
    );

    let stock_a = res_a?;
    let stock_b = res_b?;
    Ok(stock_a + stock_b)
}
```

---

## Spawning Green Tasks with `tokio::spawn`

While `tokio::join!` runs futures concurrently within the *current* task, `tokio::spawn` launches an independent **Tokio Task** (a green thread):
- Tasks run cooperatively on Tokio's multi-threaded thread pool.
- Spawning a task takes less than 1 microsecond and a few hundred bytes of memory.
- `tokio::spawn` returns a `JoinHandle<T>` which can be `.await`ed.

```rust
let task = tokio::spawn(async move {
    // Background audit logging or analytics
    println!("Background telemetry processing...");
    42
});

let result = task.await.unwrap();
assert_eq!(result, 42);
```

---

## Asynchronous Channels with `tokio::sync::mpsc`

Standard library channels (`std::sync::mpsc`) are synchronous and block the calling thread. Tokio provides its own non-blocking asynchronous channels:

```rust
use tokio::sync::mpsc;

// Bounded channel with a capacity of 32 messages
let (tx, mut rx) = mpsc::channel(32);

tokio::spawn(async move {
    tx.send("Order #901 Completed").await.unwrap();
});

// Non-blocking receive:
if let Some(msg) = rx.recv().await {
    println!("Received async event: {msg}");
}
```

In MiniStore, we wrap this in `AsyncNotificationBus`:
```rust
pub struct AsyncNotificationBus {
    sender: mpsc::Sender<OrderNotification>,
}

impl AsyncNotificationBus {
    pub fn new(buffer: usize) -> (Self, mpsc::Receiver<OrderNotification>) {
        let (sender, receiver) = mpsc::channel(buffer);
        (Self { sender }, receiver)
    }

    pub async fn dispatch(
        &self,
        notification: OrderNotification,
    ) -> Result<(), mpsc::error::SendError<OrderNotification>> {
        self.sender.send(notification).await
    }
}
```

---

## MiniStore Implementation: Real-World Architecture

In this chapter, we integrated non-blocking asynchronous workflows into MiniStore:

```
ministore/
├── Cargo.toml              # tokio dependency with full features
├── src/
│   ├── async_services.rs   # MockPaymentClient, WarehouseClient, AsyncNotificationBus, async_checkout
│   ├── concurrency.rs
│   ├── catalog.rs
│   ├── promotions.rs
│   ├── models/
│   ├── lib.rs              # Re-exports async services & 49 passing unit tests
│   └── main.rs             # Demonstrates async payment, tokio::join!, and async checkout
```

### End-to-End Async Checkout
```rust
pub async fn async_checkout(
    order_id: OrderId,
    customer: Customer,
    cart: &mut ShoppingCart,
    catalog: &mut Catalog,
    payment_method: PaymentMethod,
    coupon: Option<Coupon>,
    payment_client: &MockPaymentClient,
) -> Result<(Order, String), StoreError> {
    // 1. Transactional stock reduction & validation
    let order = crate::checkout::checkout(
        order_id, customer, cart, catalog, payment_method.clone(), coupon
    )?;

    // 2. Asynchronous non-blocking payment gateway call
    let tx_receipt = payment_client
        .process_payment(&payment_method, order.total_cents())
        .await?;

    Ok((order, tx_receipt))
}
```

---

## Common Compiler & Runtime Errors

### 1. The Lazy Future Trap (Unused Must-Use)
```rust
payment_client.process_payment(&card, 5000);
// Warning: unused implementor of `Future` that must be used!
```
**Fix**: Append `.await` to poll the future to completion:
```rust
payment_client.process_payment(&card, 5000).await?;
```

### 2. Holding a `std::sync::MutexGuard` Across `.await`
```rust
let guard = std_mutex.lock().unwrap();
tokio::time::sleep(Duration::from_millis(10)).await; // COMPILER ERROR!
drop(guard);
```
**Why it fails**: Standard `MutexGuard` does not implement `Send` across `.await` yield points because another thread might resume the future.
**Fix**:
- Keep the synchronous lock scope strictly outside `.await` points: `{ let guard = m.lock(); ... }`.
- Or use `tokio::sync::Mutex` for locks that must be held across `.await` points.

---

## Validation & Test Suite

### Running All 49 Tests
```bash
cargo test
```
```text
running 49 tests
test tests::test_async_payment_processing ... ok
test tests::test_async_warehouse_single_stock ... ok
test tests::test_async_warehouse_stock_aggregation_join ... ok
test tests::test_async_notification_bus_mpsc ... ok
test tests::test_async_checkout_workflow ... ok
...
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

---

## Hands-On Exercises

1. **Async Timeout Protection**:
   Investigate `tokio::time::timeout`. Wrap `payment_client.process_payment(...)` in a 30ms timeout. Simulate a 100ms delayed gateway and verify that `timeout` returns an `Err(Elapsed)` error.
2. **Dynamic Multi-Warehouse Race with `tokio::select!`**:
   Use `tokio::select!` to query two warehouses and return whichever warehouse responds *first* (fastest responder pattern).
3. **Task Cancellation**:
   Create a long-running background task with `tokio::spawn`. Observe how calling `task.abort()` immediately cancels the future cooperatively.

---

## Chapter Summary & Async Cheatsheet

| Primitive | Module | Purpose | Overhead |
| :--- | :--- | :--- | :--- |
| **`async fn`** | Language Core | Declares a lazy future state machine | Zero cost |
| **`.await`** | Language Core | Yields execution until future is `Ready` | Minimal |
| **`tokio::main`** | Tokio | Initializes multi-threaded async runtime | Single runtime per app |
| **`tokio::join!`** | Tokio | Concurrently awaits multiple futures | Same task execution |
| **`tokio::spawn`** | Tokio | Spawns background green task | ~few hundred bytes |
| **`tokio::time::sleep`** | Tokio | Non-blocking timer delay | Timer wheel entry |
| **`tokio::sync::mpsc`** | Tokio | Asynchronous message passing channel | Bounded ring buffer |

In the next chapter, we transition from in-memory services to the **Application Layer**, implementing **Persistence** with file I/O and JSON serialization (Serde)!
