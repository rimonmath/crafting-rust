//! Asynchronous service layer, payment gateways, and concurrent warehouse lookups powered by Tokio.

use crate::catalog::Catalog;
use crate::concurrency::OrderNotification;
use crate::error::StoreError;
use crate::models::{Coupon, Customer, Order, OrderId, PaymentMethod, ShoppingCart};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

/// A mock asynchronous payment client simulating non-blocking network calls to payment processors.
#[derive(Debug, Clone, Default)]
pub struct MockPaymentClient {
    pub latency_ms: u64,
}

impl MockPaymentClient {
    pub fn new(latency_ms: u64) -> Self {
        Self { latency_ms }
    }

    /// Asynchronously processes a payment with simulated network roundtrip latency.
    pub async fn process_payment(
        &self,
        payment_method: &PaymentMethod,
        amount_cents: u32,
    ) -> Result<String, StoreError> {
        if self.latency_ms > 0 {
            sleep(Duration::from_millis(self.latency_ms)).await;
        }

        if amount_cents == 0 {
            return Err(StoreError::InvalidStateTransition {
                current: String::from("Empty"),
                action: String::from("process_payment"),
            });
        }

        let tx_id = match payment_method {
            PaymentMethod::CreditCard { last_four } => {
                format!("TX-CARD-{last_four}-{amount_cents}")
            }
            PaymentMethod::BankTransfer { reference } => {
                format!("TX-BANK-{reference}-{amount_cents}")
            }
            PaymentMethod::CashOnDelivery => {
                format!("TX-COD-{amount_cents}")
            }
        };

        Ok(tx_id)
    }
}

/// A client for querying remote warehouse inventory asynchronously.
#[derive(Debug, Clone)]
pub struct WarehouseClient {
    pub warehouse_name: String,
    pub latency_ms: u64,
}

impl WarehouseClient {
    pub fn new(name: impl Into<String>, latency_ms: u64) -> Self {
        Self {
            warehouse_name: name.into(),
            latency_ms,
        }
    }

    /// Queries the stock of a product by SKU asynchronously.
    pub async fn check_stock(&self, sku: &str) -> Result<u32, StoreError> {
        if self.latency_ms > 0 {
            sleep(Duration::from_millis(self.latency_ms)).await;
        }

        let stock = match sku {
            "TECH-KEY-001" => 15,
            "TECH-MOU-002" => 25,
            "TECH-MON-003" => 8,
            _ => 0,
        };

        Ok(stock)
    }
}

/// Queries two warehouses concurrently using `tokio::join!` and aggregates their total available stock.
///
/// Demonstrates concurrent future execution on the Tokio runtime without spawning OS threads.
pub async fn aggregate_warehouse_stock(
    warehouse_a: &WarehouseClient,
    warehouse_b: &WarehouseClient,
    sku: &str,
) -> Result<u32, StoreError> {
    let (res_a, res_b) = tokio::join!(warehouse_a.check_stock(sku), warehouse_b.check_stock(sku));

    let stock_a = res_a?;
    let stock_b = res_b?;
    Ok(stock_a + stock_b)
}

/// An asynchronous order event bus using Tokio's async bounded mpsc channels.
pub struct AsyncNotificationBus {
    sender: mpsc::Sender<OrderNotification>,
}

impl AsyncNotificationBus {
    pub fn new(buffer: usize) -> (Self, mpsc::Receiver<OrderNotification>) {
        let (sender, receiver) = mpsc::channel(buffer);
        (Self { sender }, receiver)
    }

    /// Dispatches an order event asynchronously into the channel buffer.
    pub async fn dispatch(
        &self,
        notification: OrderNotification,
    ) -> Result<(), mpsc::error::SendError<OrderNotification>> {
        self.sender.send(notification).await
    }

    pub fn clone_sender(&self) -> mpsc::Sender<OrderNotification> {
        self.sender.clone()
    }
}

/// Completes an asynchronous checkout: validates stock, processes payment asynchronously,
/// and returns the confirmed order along with its gateway transaction receipt.
pub async fn async_checkout(
    order_id: OrderId,
    customer: Customer,
    cart: &mut ShoppingCart,
    catalog: &mut Catalog,
    payment_method: PaymentMethod,
    coupon: Option<Coupon>,
    payment_client: &MockPaymentClient,
) -> Result<(Order, String), StoreError> {
    // 1. Perform transactional domain validation
    let order = crate::checkout::checkout(
        order_id,
        customer,
        cart,
        catalog,
        payment_method.clone(),
        coupon,
    )?;

    // 2. Asynchronously process the payment with non-blocking network I/O
    let tx_receipt = payment_client
        .process_payment(&payment_method, order.total_cents())
        .await?;

    Ok((order, tx_receipt))
}
