//! Trait definitions, implementations, and generic trait bounds for MiniStore.
//!
//! Demonstrates defining traits, default method implementations, trait bounds,
//! multiple bounds with `+`, and `where` clauses.

use crate::models::{Customer, Order, Product, ProductCategory};

/// Defines sales tax calculation behavior.
pub trait Taxable {
    /// Returns the tax percentage rate (e.g. 15 for 15%).
    fn tax_rate(&self) -> u32;

    /// Calculates the tax amount in cents given a base price in cents.
    ///
    /// Provides a default implementation so implementors only need to supply `tax_rate`.
    fn calculate_tax(&self, price_cents: u32) -> u32 {
        (price_cents * self.tax_rate()) / 100
    }
}

/// Defines human-readable summary generation behavior.
pub trait Summarizable {
    /// Returns a concise single-line summary of the entity.
    fn summary(&self) -> String;
}

// ----------------------------------------------------------------------------
// Implementations for MiniStore Domain Models
// ----------------------------------------------------------------------------

impl Taxable for ProductCategory {
    fn tax_rate(&self) -> u32 {
        self.default_tax_rate()
    }
}

impl Taxable for Product {
    fn tax_rate(&self) -> u32 {
        self.category.tax_rate()
    }
}

impl Summarizable for Product {
    fn summary(&self) -> String {
        format!(
            "Product #{}: {} [{}] - ${:.2}",
            self.id,
            self.name,
            self.sku,
            self.price_cents as f64 / 100.0
        )
    }
}

impl Summarizable for Customer {
    fn summary(&self) -> String {
        format!("Customer #{}: {} <{}>", self.id, self.name, self.email)
    }
}

impl Summarizable for Order {
    fn summary(&self) -> String {
        format!(
            "Order #{}: {} item(s), Total: ${:.2} [{}]",
            self.order_id.0,
            self.items.len(),
            self.total_cents() as f64 / 100.0,
            self.status.display_status()
        )
    }
}

// ----------------------------------------------------------------------------
// Generic Functions with Trait Bounds
// ----------------------------------------------------------------------------

/// Generates a summary for any item that implements `Summarizable`.
///
/// Demonstrates the basic trait bound syntax `<T: Summarizable>`.
pub fn summarize_item<T: Summarizable>(item: &T) -> String {
    item.summary()
}

/// Formats a complete pricing and tax summary for any item that is both
/// `Taxable` and `Summarizable`.
///
/// Demonstrates multiple trait bounds and the cleaner `where` clause syntax.
pub fn format_tax_summary<T>(item: &T, price_cents: u32) -> String
where
    T: Taxable + Summarizable,
{
    let tax = item.calculate_tax(price_cents);
    format!(
        "{} | Tax: ${:.2} ({}%)",
        item.summary(),
        tax as f64 / 100.0,
        item.tax_rate()
    )
}
