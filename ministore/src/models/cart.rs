use serde::{Deserialize, Serialize};

/// Represents a single line item in a shopping cart or order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: u64,
    pub quantity: u32,
    pub unit_price_cents: u32,
}

impl CartItem {
    pub fn new(product_id: u64, quantity: u32, unit_price_cents: u32) -> Self {
        Self {
            product_id,
            quantity,
            unit_price_cents,
        }
    }

    pub fn line_total(&self) -> u32 {
        self.unit_price_cents * self.quantity
    }
}

/// A dynamic shopping cart backed by `Vec<CartItem>`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ShoppingCart {
    pub items: Vec<CartItem>,
}

impl ShoppingCart {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn get_item(&self, product_id: u64) -> Option<&CartItem> {
        self.items.iter().find(|item| item.product_id == product_id)
    }

    pub fn get_item_mut(&mut self, product_id: u64) -> Option<&mut CartItem> {
        self.items
            .iter_mut()
            .find(|item| item.product_id == product_id)
    }

    pub fn add_item(&mut self, product_id: u64, quantity: u32, unit_price_cents: u32) {
        if let Some(item) = self.get_item_mut(product_id) {
            item.quantity += quantity;
        } else {
            self.items
                .push(CartItem::new(product_id, quantity, unit_price_cents));
        }
    }

    pub fn remove_item(&mut self, product_id: u64) -> bool {
        if let Some(pos) = self
            .items
            .iter()
            .position(|item| item.product_id == product_id)
        {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn total_units(&self) -> u32 {
        self.items.iter().map(|item| item.quantity).sum()
    }

    pub fn subtotal_cents(&self) -> u32 {
        self.items.iter().map(|item| item.line_total()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns an iterator over immutable references to the cart items.
    pub fn iter(&self) -> std::slice::Iter<'_, CartItem> {
        self.items.iter()
    }

    /// Returns an iterator over mutable references to the cart items.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, CartItem> {
        self.items.iter_mut()
    }

    /// Checks if a product exists in the cart using iterator `.any()`.
    pub fn has_product(&self, product_id: u64) -> bool {
        self.items.iter().any(|item| item.product_id == product_id)
    }

    /// Applies a percentage discount to all cart items using iterator `.iter_mut().for_each()`.
    pub fn apply_promotional_discount(&mut self, percentage: u32) {
        self.items.iter_mut().for_each(|item| {
            let discount = (item.unit_price_cents * percentage) / 100;
            item.unit_price_cents = item.unit_price_cents.saturating_sub(discount);
        });
    }

    /// Applies a custom discount closure to each cart item in place.
    ///
    /// Accepts a mutable closure (`FnMut(&CartItem) -> u32`) allowing stateful discount calculations.
    pub fn apply_custom_discount<F>(&mut self, mut discount_calc: F)
    where
        F: FnMut(&CartItem) -> u32,
    {
        for item in &mut self.items {
            let discount = discount_calc(item);
            item.unit_price_cents = item.unit_price_cents.saturating_sub(discount);
        }
    }

    /// Returns a custom report iterator for line-by-line summary generation.
    pub fn report_iter(&self) -> CartReportIterator<'_> {
        CartReportIterator::new(self)
    }
}

// ----------------------------------------------------------------------------
// IntoIterator Implementations for ShoppingCart
// ----------------------------------------------------------------------------

impl<'a> IntoIterator for &'a ShoppingCart {
    type Item = &'a CartItem;
    type IntoIter = std::slice::Iter<'a, CartItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<'a> IntoIterator for &'a mut ShoppingCart {
    type Item = &'a mut CartItem;
    type IntoIter = std::slice::IterMut<'a, CartItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter_mut()
    }
}

impl IntoIterator for ShoppingCart {
    type Item = CartItem;
    type IntoIter = std::vec::IntoIter<CartItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

// ----------------------------------------------------------------------------
// Custom Iterators
// ----------------------------------------------------------------------------

/// A custom iterator that yields human-readable formatted summary strings for cart items.
#[derive(Debug, Clone)]
pub struct CartReportIterator<'a> {
    cart: &'a ShoppingCart,
    index: usize,
}

impl<'a> CartReportIterator<'a> {
    pub fn new(cart: &'a ShoppingCart) -> Self {
        Self { cart, index: 0 }
    }
}

impl<'a> Iterator for CartReportIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.cart.items.len() {
            let item = &self.cart.items[self.index];
            self.index += 1;
            Some(format!(
                "Item #{}: Product #{} (Qty: {}) - ${:.2}",
                self.index,
                item.product_id,
                item.quantity,
                item.line_total() as f64 / 100.0
            ))
        } else {
            None
        }
    }
}

/// A custom iterator generating progressive tiered discount percentages (e.g. 5%, 10%, 15%).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscountTierIter {
    current: u32,
    step: u32,
    max: u32,
}

impl DiscountTierIter {
    pub fn new(step: u32, max: u32) -> Self {
        Self {
            current: 0,
            step,
            max,
        }
    }
}

impl Iterator for DiscountTierIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let next_val = self.current + self.step;
        if next_val <= self.max {
            self.current = next_val;
            Some(self.current)
        } else {
            None
        }
    }
}
