use crate::catalog::Catalog;
use crate::error::StoreError;
use crate::models::{
    Coupon, Customer, Order, OrderId, OrderStatus, PaymentMethod, Product, ShoppingCart,
};
use std::collections::HashMap;

/// Processes a shopping cart into a confirmed order, validating stock and coupon with `?`.
pub fn checkout(
    order_id: OrderId,
    customer: Customer,
    cart: &mut ShoppingCart,
    catalog: &mut Catalog,
    payment: PaymentMethod,
    coupon: Option<Coupon>,
) -> Result<Order, StoreError> {
    if cart.is_empty() {
        return Err(StoreError::EmptyCart);
    }

    if let Some(c) = &coupon {
        c.validate()?;
    }

    // Check inventory and deduct stock for every cart item using `?` error propagation
    for item in &cart.items {
        let product =
            catalog
                .find_by_id_mut(item.product_id)
                .ok_or_else(|| StoreError::ProductNotFound {
                    identifier: format!("ID #{}", item.product_id),
                })?;

        product.reduce_stock(item.quantity)?;
    }

    let items = std::mem::take(&mut cart.items);
    Ok(Order::new(order_id, customer, items, payment, coupon))
}

/// Calculates discount based on customer VIP status and optional Coupon.
pub fn calculate_discount(
    customer: &Customer,
    coupon: Option<&Coupon>,
    subtotal_cents: u32,
) -> u32 {
    let vip_discount = if customer.is_vip || customer.has_tag("vip") {
        (subtotal_cents * 10) / 100 // 10% discount for VIPs
    } else {
        0
    };

    let coupon_discount = coupon
        .map(|c| (subtotal_cents * c.discount_percent) / 100)
        .unwrap_or(0);

    vip_discount + coupon_discount
}

/// Groups products by department and counts them using `HashMap` and the Entry API.
pub fn count_products_by_department(products: &[Product]) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for product in products {
        let dept = product.department_code().to_string();
        *counts.entry(dept).or_insert(0) += 1;
    }
    counts
}

/// Evaluates dispatch message based on OrderStatus using exhaustive pattern matching.
pub fn order_dispatch_advisory(status: &OrderStatus) -> &'static str {
    match status {
        OrderStatus::Pending => "Hold in warehouse: waiting for customer payment.",
        OrderStatus::Confirmed { .. } => "Ready to pick and pack at fulfillment center.",
        OrderStatus::Shipped { .. } => "In transit with logistics courier.",
        OrderStatus::Delivered => "Package successfully delivered to recipient.",
        OrderStatus::Cancelled { .. } => "Halted: restock inventory items immediately.",
    }
}
