#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub sku: String,
    pub name: String,
    pub price_cents: u32,
    pub stock: u32,
}

impl Product {
    pub fn new(id: u64, sku: String, name: String, price_cents: u32, stock: u32) -> Self {
        Self {
            id,
            sku,
            name,
            price_cents,
            stock,
        }
    }

    /// Borrows `&self` immutably: checks stock without consuming or mutating the product.
    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    /// Borrows `&self` immutably: formats price for presentation.
    pub fn formatted_price(&self) -> String {
        format!("${:.2}", self.price_cents as f64 / 100.0)
    }

    /// Returns a string slice (`&str`) borrowing the department code from `self.sku`.
    /// Zero heap allocation: returns a 16-byte fat pointer (ptr + len) directly into `sku`.
    pub fn department_code(&self) -> &str {
        match self.sku.find('-') {
            Some(idx) => &self.sku[..idx],
            None => &self.sku[..],
        }
    }

    /// Returns a slice of the product name truncated to `max_bytes` without reallocating,
    /// ensuring the slice boundary respects UTF-8 character boundaries.
    pub fn truncated_name(&self, max_bytes: usize) -> &str {
        if max_bytes >= self.name.len() {
            &self.name[..]
        } else {
            // Find the nearest valid UTF-8 character boundary at or below max_bytes
            let mut end = max_bytes;
            while end > 0 && !self.name.is_char_boundary(end) {
                end -= 1;
            }
            &self.name[..end]
        }
    }

    /// Checks if the product SKU begins with the requested prefix slice.
    pub fn matches_sku_prefix(&self, prefix: &str) -> bool {
        self.sku.starts_with(prefix)
    }

    /// Borrows `&mut self` mutably: updates inventory count in place.
    pub fn reduce_stock(&mut self, quantity: u32) -> Result<u32, &'static str> {
        if quantity > self.stock {
            Err("Insufficient stock available")
        } else {
            self.stock -= quantity;
            Ok(self.stock)
        }
    }

    /// Borrows `&mut self` mutably: adds inventory units in place.
    pub fn restock(&mut self, additional_units: u32) {
        self.stock += additional_units;
    }

    /// Borrows `&mut self` mutably: adjusts product price.
    pub fn update_price(&mut self, new_price_cents: u32) {
        self.price_cents = new_price_cents;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Customer {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub is_vip: bool,
}

impl Customer {
    pub fn new(id: u64, name: String, email: String, is_vip: bool) -> Self {
        Self {
            id,
            name,
            email,
            is_vip,
        }
    }

    /// Borrows `&self` immutably: reads customer data to display membership badge.
    pub fn display_badge(&self) -> String {
        if self.is_vip {
            format!("[VIP Member] {}", self.name)
        } else {
            format!("[Standard Member] {}", self.name)
        }
    }

    /// Borrows `&mut self` mutably: grants VIP membership status in place.
    pub fn upgrade_to_vip(&mut self) {
        self.is_vip = true;
    }

    /// Borrows `&mut self` mutably: updates email address in place.
    pub fn update_email(&mut self, new_email: String) {
        self.email = new_email;
    }
}

/// OrderId implements `Copy`: 8-byte scalar on the stack, never moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderId(pub u64);

/// An unconfirmed order owning its items.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingOrder {
    pub order_id: OrderId,
    pub customer: Customer,
    pub product: Product,
    pub quantity: u32,
}

/// Finalized invoice/receipt produced when checkout consumes `PendingOrder`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmedReceipt {
    pub receipt_id: String,
    pub order_id: OrderId,
    pub customer_name: String,
    pub product_name: String,
    pub quantity: u32,
    pub total_cents: u32,
}

// ============================================================================
// String & Array Slice Functions: Zero-Copy Views Into Memory
// ============================================================================

/// Borrows `&Product` immutably: calculates subtotal without taking ownership.
pub fn calculate_line_total(product: &Product, quantity: u32) -> u32 {
    product.price_cents * quantity
}

/// Borrows `&Customer` immutably: calculates discount based on VIP status.
pub fn calculate_discount(customer: &Customer, subtotal_cents: u32) -> u32 {
    if customer.is_vip {
        (subtotal_cents * 10) / 100 // 10% discount for VIPs
    } else {
        0
    }
}

/// Borrows an array slice of prices `&[u32]`.
/// Works seamlessly with fixed stack arrays `[u32; N]` or slices without copying data.
pub fn calculate_batch_total(prices: &[u32]) -> u32 {
    let mut total = 0;
    for &price in prices {
        total += price;
    }
    total
}

/// Borrows a mutable slice `&mut [u32]` to apply markdown discounts in-place.
pub fn apply_batch_markdown(prices: &mut [u32], markdown_cents: u32) {
    for price in prices.iter_mut() {
        if *price > markdown_cents {
            *price -= markdown_cents;
        } else {
            *price = 0;
        }
    }
}

/// Calculates inventory totals using a slice of Products `&[Product]`.
/// Returns `(total_units, total_valuation_cents)`.
pub fn summarize_inventory(catalog: &[Product]) -> (u32, u32) {
    let mut total_units = 0;
    let mut total_cents = 0;
    for product in catalog {
        total_units += product.stock;
        total_cents += product.price_cents * product.stock;
    }
    (total_units, total_cents)
}

/// Accepts any string slice `&str` (literal, heap slice, or &String via Deref coercion).
/// Validates promo codes and returns discount percentage.
pub fn validate_coupon(code: &str) -> Result<u32, &'static str> {
    let trimmed = code.trim();
    if trimmed.eq_ignore_ascii_case("SAVE10") {
        Ok(10)
    } else if trimmed.eq_ignore_ascii_case("SUMMER25") {
        Ok(25)
    } else if trimmed.is_empty() {
        Err("Coupon code cannot be empty")
    } else {
        Err("Invalid coupon code")
    }
}

/// Formats a single receipt line item using string slices `&str` to avoid unnecessary String copies.
pub fn format_line_summary(sku: &str, name: &str, price_cents: u32) -> String {
    format!("[{sku}] {name} - ${:.2}", price_cents as f64 / 100.0)
}

/// Borrows both `&Customer` and `&Product` immutably to render a preview.
pub fn print_order_preview(customer: &Customer, product: &Product, quantity: u32) {
    let subtotal = calculate_line_total(product, quantity);
    let discount = calculate_discount(customer, subtotal);
    let final_total = subtotal - discount;

    println!("--- Order Preview (Borrowed Read-Only) ---");
    println!("Customer: {}", customer.display_badge());
    println!(
        "Item:     {}",
        format_line_summary(&product.sku, &product.name, product.price_cents)
    );
    println!("Dept:     {}", product.department_code());
    println!("Quantity: {}", quantity);
    println!("Subtotal: ${:.2}", subtotal as f64 / 100.0);
    println!("Discount: ${:.2}", discount as f64 / 100.0);
    println!("Estimate: ${:.2}", final_total as f64 / 100.0);
}

/// Consumes ownership of `PendingOrder` by value (Move Semantics).
pub fn finalize_order(order: PendingOrder) -> ConfirmedReceipt {
    let subtotal = calculate_line_total(&order.product, order.quantity);
    let discount = calculate_discount(&order.customer, subtotal);
    let total_cents = subtotal - discount;

    ConfirmedReceipt {
        receipt_id: format!("REC-{}-{}", order.order_id.0, order.customer.id),
        order_id: order.order_id,
        customer_name: order.customer.name,
        product_name: order.product.name,
        quantity: order.quantity,
        total_cents,
    }
}

fn main() {
    println!("=== MiniStore: Strings, Slices & Zero-Copy Views ===\n");

    // 1. String Slices (&str) vs Owned Strings (String)
    let sample_sku = String::from("TECH-KEY-001");
    // Slicing without allocating: 16-byte fat pointer (ptr + len)
    let dept_slice: &str = &sample_sku[0..4];
    let id_slice: &str = &sample_sku[5..];
    println!("1. String Slice Decomposition (Zero Allocations):");
    println!("   Full SKU:    {sample_sku} (Heap String: 24 bytes on stack)");
    println!("   Dept Slice:  {dept_slice} (Slice view: 16 bytes on stack)");
    println!("   ID Slice:    {id_slice}\n");

    // 2. Deref Coercion: &String automatically coerces to &str
    println!("2. Universal &str Function Parameters & Deref Coercion:");
    let coupon_literal = "  SAVE10  "; // &'static str
    let coupon_heap = String::from("SUMMER25"); // String

    println!(
        "   Validating literal: {:?}",
        validate_coupon(coupon_literal)
    );
    println!(
        "   Validating heap string (&String coerces to &str): {:?}",
        validate_coupon(&coupon_heap)
    );

    // 3. Products with SKU Slices and Safe Truncation
    let keyboard = Product::new(
        501,
        String::from("TECH-KEY-001"),
        String::from("RGB Mechanical Gaming Keyboard with Hot-Swap Switches"),
        12000,
        15,
    );

    println!("\n3. Product Slice Methods:");
    println!("   Product SKU:       {}", keyboard.sku);
    println!("   Department:        {}", keyboard.department_code());
    println!("   Short Title (25B): {}...", keyboard.truncated_name(25));
    println!(
        "   Matches 'TECH':    {}",
        keyboard.matches_sku_prefix("TECH")
    );

    // 4. Array Slices (&[T]) & Mutable Slices (&mut [T])
    println!("\n4. Array Slices (&[T]) for Batch Pricing:");
    let mut clearance_prices = [2500, 4500, 12000, 8000]; // Stack array [u32; 4]
    println!(
        "   Initial prices:    {:?} (Total = ${:.2})",
        clearance_prices,
        calculate_batch_total(&clearance_prices) as f64 / 100.0
    );

    // Pass a subslice `&mut clearance_prices[0..2]` to apply markdown only to first two items
    apply_batch_markdown(&mut clearance_prices[0..2], 500);
    println!(
        "   After $5 markdown on first 2 items: {:?}",
        clearance_prices
    );

    // 5. Catalog Slices (&[Product])
    let catalog = [
        keyboard.clone(),
        Product::new(
            502,
            String::from("OFFC-CHAIR-002"),
            String::from("Ergonomic Mesh Chair"),
            35000,
            8,
        ),
        Product::new(
            503,
            String::from("TECH-MOU-003"),
            String::from("Wireless Laser Mouse"),
            4500,
            25,
        ),
    ];

    let (total_units, valuation) = summarize_inventory(&catalog);
    println!("\n5. Catalog Summary (Calculated via &[Product] slice):");
    println!("   Total Units:     {total_units}");
    println!("   Total Valuation: ${:.2}", valuation as f64 / 100.0);

    // 6. Preview and Checkout
    let customer = Customer::new(
        101,
        String::from("Ada Lovelace"),
        String::from("ada@analytical.org"),
        true,
    );

    println!("\n6. Order Preview & Checkout:");
    print_order_preview(&customer, &keyboard, 2);

    let pending_order = PendingOrder {
        order_id: OrderId(9001),
        customer,
        product: keyboard,
        quantity: 2,
    };

    let receipt = finalize_order(pending_order);
    println!("\n--- Checkout Confirmed ---");
    println!("Receipt: {}", receipt.receipt_id);
    println!("Item:    {} x {}", receipt.product_name, receipt.quantity);
    println!("Paid:    ${:.2}", receipt.total_cents as f64 / 100.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_slice_department_code() {
        let p = Product::new(
            1,
            String::from("ELEC-MON-001"),
            String::from("4K Monitor"),
            40000,
            5,
        );
        assert_eq!(p.department_code(), "ELEC");
        assert!(p.matches_sku_prefix("ELEC"));
        assert!(!p.matches_sku_prefix("FURN"));
    }

    #[test]
    fn test_truncated_name_ascii_and_boundaries() {
        let p = Product::new(
            2,
            String::from("BOOK-RUST-01"),
            String::from("Rust Programming in Depth"),
            5000,
            20,
        );
        assert_eq!(p.truncated_name(4), "Rust");
        assert_eq!(p.truncated_name(100), "Rust Programming in Depth");

        // UTF-8 multi-byte test (Bengali character: 'ব' is 3 bytes: 0xE0, 0xA6, 0xAC)
        let p_utf8 = Product::new(3, String::from("BOOK-BN-02"), String::from("বই"), 1000, 10);
        // Slicing at 2 bytes would slice inside 'ব'; truncated_name snaps safely to 0
        assert_eq!(p_utf8.truncated_name(2), "");
        assert_eq!(p_utf8.truncated_name(3), "ব");
    }

    #[test]
    fn test_validate_coupon_deref_coercion() {
        // String literal (&'static str)
        assert_eq!(validate_coupon("SAVE10"), Ok(10));
        assert_eq!(validate_coupon("  save10  "), Ok(10));

        // Owned String coerced via &coupon
        let dynamic_code = String::from("SUMMER25");
        assert_eq!(validate_coupon(&dynamic_code), Ok(25));

        assert!(validate_coupon("UNKNOWN").is_err());
        assert!(validate_coupon("   ").is_err());
    }

    #[test]
    fn test_array_slices_and_mutable_markdown() {
        let mut prices = [1000, 2500, 5000];

        // Slice calculation:
        assert_eq!(calculate_batch_total(&prices), 8500);
        assert_eq!(calculate_batch_total(&prices[1..3]), 7500);

        // Mutable slice markdown on first two elements only:
        apply_batch_markdown(&mut prices[0..2], 500);
        assert_eq!(prices, [500, 2000, 5000]);
    }

    #[test]
    fn test_catalog_inventory_summary() {
        let items = [
            Product::new(1, String::from("A-1"), String::from("Item 1"), 100, 10),
            Product::new(2, String::from("A-2"), String::from("Item 2"), 200, 5),
        ];

        let (units, valuation) = summarize_inventory(&items);
        assert_eq!(units, 15);
        assert_eq!(valuation, (100 * 10) + (200 * 5)); // 1000 + 1000 = 2000
    }
}
