//! Dynamic promotion engines and closure-based discount builders.

use crate::models::Customer;

/// Creates a percentage discount closure that captures the discount percentage by value.
///
/// Demonstrates returning a closure (`impl Fn`) using the `move` keyword.
pub fn make_percentage_discount(percentage: u32) -> impl Fn(u32) -> u32 {
    move |amount_cents| (amount_cents * percentage) / 100
}

/// Creates a threshold discount closure (e.g., "$10 off orders over $100").
///
/// If `amount_cents` is greater than or equal to `min_cents`, returns `discount_cents`; otherwise 0.
pub fn make_threshold_discount(min_cents: u32, discount_cents: u32) -> impl Fn(u32) -> u32 {
    move |amount_cents| {
        if amount_cents >= min_cents {
            discount_cents
        } else {
            0
        }
    }
}

/// Creates a VIP customer discount closure that captures the VIP discount rate.
pub fn make_vip_discount(vip_rate: u32) -> impl Fn(&Customer, u32) -> u32 {
    move |customer, amount_cents| {
        if customer.is_vip || customer.has_tag("vip") {
            (amount_cents * vip_rate) / 100
        } else {
            0
        }
    }
}

/// Evaluates a list of discount closures and returns the total combined discount.
///
/// Demonstrates passing closures as generic parameters bounded by `Fn`.
pub fn calculate_total_promotions<F>(amount_cents: u32, rules: &[F]) -> u32
where
    F: Fn(u32) -> u32,
{
    rules.iter().map(|rule| rule(amount_cents)).sum()
}

/// Evaluates a list of discount closures and returns the single largest discount.
pub fn find_best_promotion<F>(amount_cents: u32, rules: &[F]) -> u32
where
    F: Fn(u32) -> u32,
{
    rules
        .iter()
        .map(|rule| rule(amount_cents))
        .max()
        .unwrap_or(0)
}

/// An audit tracker that records discount applications using a mutating closure (`FnMut`).
///
/// Demonstrates `FnMut` capturing and mutating enclosing environment state across invocations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscountAuditor {
    pub total_discount_given: u32,
    pub operations_count: u32,
}

impl DiscountAuditor {
    pub fn new() -> Self {
        Self {
            total_discount_given: 0,
            operations_count: 0,
        }
    }

    /// Records an applied discount, mutating internal accounting counters.
    pub fn record(&mut self, discount_cents: u32) {
        self.total_discount_given += discount_cents;
        self.operations_count += 1;
    }
}

impl Default for DiscountAuditor {
    fn default() -> Self {
        Self::new()
    }
}
