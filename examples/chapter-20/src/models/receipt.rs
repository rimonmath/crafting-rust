use crate::models::{Order, OrderId, Product};
use crate::traits::Summarizable;

/// A zero-copy receipt view that borrows an `Order`, customer name, and cashier note.
///
/// The lifetime parameter `'a` guarantees that the `OrderReceipt` cannot outlive
/// the `Order` or the string slices it references.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderReceipt<'a> {
    pub order: &'a Order,
    pub customer_name: &'a str,
    pub cashier_note: &'a str,
}

impl<'a> OrderReceipt<'a> {
    /// Constructs a new `OrderReceipt` borrowing from the given `Order` and slices.
    pub fn new(order: &'a Order, customer_name: &'a str, cashier_note: &'a str) -> Self {
        Self {
            order,
            customer_name,
            cashier_note,
        }
    }

    /// Returns the borrowed customer's name with lifetime `'a`.
    ///
    /// Notice: We explicitly annotate `'a` so the returned reference is tied to
    /// the underlying customer string, not the short-lived `&self` reference!
    pub fn customer_name(&self) -> &'a str {
        self.customer_name
    }

    /// Returns the cashier note with lifetime `'a`.
    pub fn note(&self) -> &'a str {
        self.cashier_note
    }

    /// Returns the associated `OrderId`.
    pub fn order_id(&self) -> OrderId {
        self.order.order_id
    }

    /// Generates a formatted multi-line receipt slip.
    pub fn generate_slip(&self) -> String {
        format!(
            "=== RECEIPT: {} ===\nCustomer: {}\nItems: {}\nTotal: ${:.2}\nStatus: {}\nNote: {}\n===================",
            self.order.order_id,
            self.customer_name,
            self.order.items.len(),
            self.order.total_cents() as f64 / 100.0,
            self.order.status,
            self.cashier_note
        )
    }
}

impl<'a> std::fmt::Display for OrderReceipt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Receipt for Order {} ({}) - Total: ${:.2}",
            self.order.order_id,
            self.customer_name,
            self.order.total_cents() as f64 / 100.0
        )
    }
}

impl<'a> Summarizable for OrderReceipt<'a> {
    fn summary(&self) -> String {
        format!(
            "Receipt: Order {} for {} | Total: ${:.2}",
            self.order.order_id,
            self.customer_name,
            self.order.total_cents() as f64 / 100.0
        )
    }
}

// ----------------------------------------------------------------------------
// Lifetime-Annotated Functions
// ----------------------------------------------------------------------------

/// Compares two products and returns a reference to the one with the higher unit price.
///
/// Lifetime parameter `'a` signifies that both input products must live at least
/// as long as `'a`, and the returned product reference is valid for that same lifetime.
pub fn find_higher_priced<'a>(p1: &'a Product, p2: &'a Product) -> &'a Product {
    if p1.price_cents >= p2.price_cents {
        p1
    } else {
        p2
    }
}

/// Chooses the best available contact info: primary phone if provided, or fallback email.
///
/// Demonstrates returning a slice tied to one of multiple input references.
pub fn best_contact_info<'a>(primary_phone: Option<&'a str>, fallback_email: &'a str) -> &'a str {
    match primary_phone {
        Some(phone) if !phone.trim().is_empty() => phone,
        _ => fallback_email,
    }
}

/// Returns the store's static return and exchange policy.
///
/// `'static` indicates that the string data lives in read-only memory
/// for the entire duration of the program execution.
pub fn store_policy() -> &'static str {
    "MiniStore Guarantee: 30-Day Hassle-Free Returns & 1-Year Warranty"
}
