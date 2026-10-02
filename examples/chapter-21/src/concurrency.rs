//! Thread-safe concurrency primitives, message passing channels, and shared-state synchronization.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Messages transmitted across threads for asynchronous order notifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderNotification {
    OrderPlaced { order_id: u64, amount_cents: u32 },
    OrderStatusUpdated { order_id: u64, status: String },
    AuditAlert { message: String },
}

/// An asynchronous order processing channel wrapping `std::sync::mpsc`.
///
/// Demonstrates the "multiple producer, single consumer" message-passing model.
pub struct OrderNotificationChannel {
    sender: Sender<OrderNotification>,
}

impl OrderNotificationChannel {
    /// Creates a new notification channel returning the sender wrapper and receiver handle.
    pub fn new() -> (Self, Receiver<OrderNotification>) {
        let (sender, receiver) = mpsc::channel();
        (Self { sender }, receiver)
    }

    /// Obtains a new producer sender handle (`Sender::clone`).
    pub fn clone_sender(&self) -> Sender<OrderNotification> {
        self.sender.clone()
    }

    /// Sends a notification into the channel.
    pub fn send(
        &self,
        notification: OrderNotification,
    ) -> Result<(), mpsc::SendError<OrderNotification>> {
        self.sender.send(notification)
    }
}

/// Aggregated sales metrics protected by a Mutex.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SalesMetrics {
    pub total_revenue_cents: u64,
    pub transactions_count: u64,
}

/// A thread-safe sales tracker demonstrating `Arc<Mutex<T>>`.
///
/// Multiple threads can clone the `Arc` pointer and call `record_sale` concurrently.
/// The `Mutex` guarantees exclusive mutable access without data races.
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

    /// Records a sale transaction, safely acquiring the lock and mutating metrics.
    pub fn record_sale(&self, amount_cents: u32) {
        let mut guard = self.inner.lock().expect("Mutex poisoned");
        guard.total_revenue_cents += amount_cents as u64;
        guard.transactions_count += 1;
        // Lock is automatically released when `guard` goes out of scope (via Drop)!
    }

    /// Returns the current total revenue in cents.
    pub fn total_revenue(&self) -> u64 {
        let guard = self.inner.lock().expect("Mutex poisoned");
        guard.total_revenue_cents
    }

    /// Returns the total number of transactions recorded.
    pub fn transactions_count(&self) -> u64 {
        let guard = self.inner.lock().expect("Mutex poisoned");
        guard.transactions_count
    }

    /// Returns the active reference count of owners sharing this tracker.
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

/// Computes the total inventory valuation across item chunks using parallel worker threads.
///
/// Spawns `num_workers` threads via `std::thread::spawn`, chunks the work,
/// and aggregates partial sums using `JoinHandle::join`.
pub fn parallel_batch_valuation(items: Vec<(u32, u32)>, num_workers: usize) -> u64 {
    if items.is_empty() || num_workers == 0 {
        return 0;
    }

    let chunk_size = items.len().div_ceil(num_workers);
    let chunks: Vec<Vec<(u32, u32)>> = items.chunks(chunk_size).map(|c| c.to_vec()).collect();

    let mut handles: Vec<JoinHandle<u64>> = Vec::new();

    for chunk in chunks {
        // `move` transfers ownership of `chunk` to the spawned thread
        let handle = thread::spawn(move || {
            chunk
                .into_iter()
                .map(|(price, qty)| (price as u64) * (qty as u64))
                .sum::<u64>()
        });
        handles.push(handle);
    }

    // Join all threads and sum up partial valuations
    handles
        .into_iter()
        .map(|handle| handle.join().expect("Thread panicked"))
        .sum()
}
