fn main() {
    println!("=== MiniStore: Inventory & Pricing ===");

    // 1. Immutable scalar values (explicit types)
    let product_id: u64 = 1001;
    let product_name: &str = "Mechanical Keyboard";
    let base_price_cents: u32 = 8999; // $89.99

    // 2. Mutable variable (stock changes as sales occur)
    let mut stock_quantity: u32 = 25;
    println!("Product #{product_id}: {product_name}");
    println!("Price: ${:.2}", base_price_cents as f64 / 100.0);
    println!("Initial Stock: {stock_quantity}");

    // Simulate customer purchases 2 units using helper function
    let units_sold: u32 = 2;
    match reduce_stock(stock_quantity, units_sold) {
        Ok(new_stock) => {
            stock_quantity = new_stock;
            println!("Units Sold: {units_sold}");
            println!("Remaining Stock: {stock_quantity}");
        }
        Err(err) => println!("Failed to sell units: {err}"),
    }

    // 3. Variable Shadowing (transforming a discount representation)
    let discount = "10"; // user entered string "10"%
    let discount: u32 = discount.parse().unwrap_or(0); // parsed into integer 10
    let final_price_cents = calculate_discounted_price(base_price_cents, discount);
    let discount_amount = base_price_cents - final_price_cents;
    println!(
        "Discount: {discount}% (-${:.2})",
        discount_amount as f64 / 100.0
    );
    println!("Final Price: ${:.2}", final_price_cents as f64 / 100.0);

    // 4. Compound Types: Tuples and Arrays
    let item_summary: (u64, &str, u32) = (product_id, product_name, final_price_cents);
    println!("Item Summary Tuple: {:?}", item_summary);

    let last_three_days_sales: [u32; 3] = [5, 8, 2];
    let total_recent_sales: u32 =
        last_three_days_sales[0] + last_three_days_sales[1] + last_three_days_sales[2];
    println!("Total Sales (Last 3 Days): {total_recent_sales}");
}

/// Applies a percentage discount in integer arithmetic to prevent floating-point inaccuracies
fn calculate_discounted_price(price_cents: u32, discount_percent: u32) -> u32 {
    let discount_amount = (price_cents * discount_percent) / 100;
    price_cents - discount_amount
}

/// Safely decrements stock, ensuring we do not underflow
fn reduce_stock(current_stock: u32, quantity: u32) -> Result<u32, &'static str> {
    if quantity > current_stock {
        Err("Insufficient stock")
    } else {
        Ok(current_stock - quantity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discount_calculation() {
        let original_price = 10000; // $100.00
        let discounted = calculate_discounted_price(original_price, 20); // 20% off
        assert_eq!(discounted, 8000); // $80.00
    }

    #[test]
    fn test_stock_reduction_success() {
        let stock = 10;
        let updated = reduce_stock(stock, 3).expect("Should succeed");
        assert_eq!(updated, 7);
    }

    #[test]
    fn test_stock_reduction_insufficient() {
        let stock = 2;
        let result = reduce_stock(stock, 5);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient stock");
    }

    #[test]
    fn test_shadowing_type_change() {
        let input = "42";
        let input: u32 = input.parse().unwrap();
        assert_eq!(input, 42);
    }
}
