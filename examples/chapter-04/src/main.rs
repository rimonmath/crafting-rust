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
