#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub name: String,
    pub price_cents: u32,
    pub stock: u32,
}

impl Product {
    pub fn new(id: u64, name: String, price_cents: u32, stock: u32) -> Self {
        Self {
            id,
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
// Borrowing Demonstration Functions: Shared (`&T`) & Mutable (`&mut T`)
// ============================================================================

/// Borrows `&Product` immutably: calculates subtotal without taking ownership.
/// The caller retains full ownership of `product`!
pub fn calculate_line_total(product: &Product, quantity: u32) -> u32 {
    product.price_cents * quantity
}

/// Borrows `&Customer` immutably: calculates discount based on VIP status.
/// The caller retains full ownership of `customer`!
pub fn calculate_discount(customer: &Customer, subtotal_cents: u32) -> u32 {
    if customer.is_vip {
        (subtotal_cents * 10) / 100 // 10% discount for VIPs
    } else {
        0
    }
}

/// Borrows both `&Customer` and `&Product` immutably to render a preview.
/// Neither value is moved or cloned.
pub fn print_order_preview(customer: &Customer, product: &Product, quantity: u32) {
    let subtotal = calculate_line_total(product, quantity);
    let discount = calculate_discount(customer, subtotal);
    let final_total = subtotal - discount;

    println!("--- Order Preview (Borrowed Read-Only) ---");
    println!("Customer: {}", customer.display_badge());
    println!(
        "Item:     {} x {} @ {}",
        product.name,
        quantity,
        product.formatted_price()
    );
    println!("Subtotal: ${:.2}", subtotal as f64 / 100.0);
    println!("Discount: ${:.2}", discount as f64 / 100.0);
    println!("Estimate: ${:.2}", final_total as f64 / 100.0);
}

/// Consumes ownership of `PendingOrder` by value (Move Semantics).
/// Contrast this with the borrowed functions above: once passed here, `order` cannot be used again!
pub fn finalize_order(order: PendingOrder) -> ConfirmedReceipt {
    let subtotal = calculate_line_total(&order.product, order.quantity);
    let discount = calculate_discount(&order.customer, subtotal);
    let total_cents = subtotal - discount;

    ConfirmedReceipt {
        receipt_id: format!("REC-{}-{}", order.order_id.0, order.customer.id),
        order_id: order.order_id,
        customer_name: order.customer.name, // Ownership of heap String moves to receipt
        product_name: order.product.name,   // Ownership of heap String moves to receipt
        quantity: order.quantity,
        total_cents,
    }
}

fn main() {
    println!("=== MiniStore: Borrowing, References & Aliasing XOR Mutability ===\n");

    // 1. Shared References (&T): Multiple Readers Without Moving Ownership
    let customer = Customer::new(
        101,
        String::from("Ada Lovelace"),
        String::from("ada@analytical.org"),
        false,
    );

    let mut product = Product::new(501, String::from("Mechanical Keyboard"), 12000, 15);

    println!("1. Shared References (&T) in Action:");
    // Both references borrow simultaneously and peacefully:
    let ref1 = &product;
    let ref2 = &product;
    println!(
        "   Simultaneous shared borrows: ref1: {}, ref2 price: {}",
        ref1.name,
        ref2.formatted_price()
    );

    // Call functions borrowing `&customer` and `&product`:
    print_order_preview(&customer, &product, 2);

    // Notice: `customer` and `product` were NOT moved! We can continue using them:
    println!(
        "\n   Original product still valid after preview: {} (Stock: {})",
        product.name, product.stock
    );

    // 2. Mutable References (&mut T): Exclusive In-Place Modification
    println!("\n2. Mutable References (&mut T) in Action:");
    // Borrow mutably to restock:
    product.restock(5);
    println!("   Restocked product: new stock = {}", product.stock);

    // Update price through a mutable borrow:
    let product_mut_ref = &mut product;
    product_mut_ref.update_price(11500); // On sale for $115.00!
    println!(
        "   Price updated via mutable reference: {}",
        product_mut_ref.formatted_price()
    );

    // 3. Non-Lexical Lifetimes (NLL) and The Golden Rule (Aliasing XOR Mutability):
    // Once `product_mut_ref` is no longer used, we can take a shared borrow again:
    let shared_after_mutation = &product;
    println!(
        "   New shared borrow after mutation completed: {} @ {}",
        shared_after_mutation.name,
        shared_after_mutation.formatted_price()
    );

    // 4. Modifying Customer via Mutable Reference:
    let mut customer_mutable = customer;
    println!("\n3. Mutating Customer Profile In-Place:");
    println!("   Before upgrade: {}", customer_mutable.display_badge());
    customer_mutable.upgrade_to_vip();
    customer_mutable.update_email(String::from("ada.lovelace@computing.org"));
    println!(
        "   After upgrade:  {} ({})",
        customer_mutable.display_badge(),
        customer_mutable.email
    );

    // 5. Finalizing an Order (Contrasting Borrowing with Moving Ownership):
    println!("\n4. Finalizing Order (Moving Ownership):");
    let pending_order = PendingOrder {
        order_id: OrderId(7001),
        customer: customer_mutable,
        product,
        quantity: 2,
    };

    // `finalize_order` takes ownership of `pending_order`:
    let receipt = finalize_order(pending_order);
    println!("   Receipt issued: {}", receipt.receipt_id);
    println!("   Customer:       {}", receipt.customer_name);
    println!(
        "   Product:        {} x {}",
        receipt.product_name, receipt.quantity
    );
    println!(
        "   Total Paid:     ${:.2}",
        receipt.total_cents as f64 / 100.0
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_borrowing_allows_multiple_readers() {
        let product = Product::new(1, String::from("USB-C Cable"), 1500, 50);

        let ref_a = &product;
        let ref_b = &product;

        // Multiple shared borrows can read simultaneously
        assert_eq!(ref_a.id, ref_b.id);
        assert_eq!(ref_a.price_cents, 1500);
        assert_eq!(ref_b.formatted_price(), "$15.00");
        assert!(ref_a.is_in_stock());
    }

    #[test]
    fn test_preview_does_not_consume_values() {
        let customer = Customer::new(1, String::from("Bob"), String::from("bob@test.com"), true);
        let product = Product::new(2, String::from("Mousepad"), 2000, 10);

        // Pass by shared reference:
        let total = calculate_line_total(&product, 3);
        let discount = calculate_discount(&customer, total);

        assert_eq!(total, 6000);
        assert_eq!(discount, 600); // 10% VIP discount on 6000

        // Customer and Product are NOT moved, still fully accessible!
        assert_eq!(customer.name, "Bob");
        assert_eq!(product.name, "Mousepad");
        assert_eq!(product.stock, 10);
    }

    #[test]
    fn test_mutable_borrowing_updates_in_place() {
        let mut product = Product::new(3, String::from("Ergo Chair"), 35000, 5);

        // Mutate via method borrowing &mut self
        product.restock(10);
        assert_eq!(product.stock, 15);

        product.reduce_stock(3).expect("Reduction should succeed");
        assert_eq!(product.stock, 12);

        // Mutate price via explicit &mut borrow
        let ref_mut = &mut product;
        ref_mut.update_price(32000);
        assert_eq!(ref_mut.price_cents, 32000);

        // Original variable reflects the in-place mutations
        assert_eq!(product.stock, 12);
        assert_eq!(product.price_cents, 32000);
    }

    #[test]
    fn test_customer_mutations_via_ref() {
        let mut customer = Customer::new(
            10,
            String::from("Carol"),
            String::from("carol@old.com"),
            false,
        );

        assert_eq!(customer.display_badge(), "[Standard Member] Carol");
        assert!(!customer.is_vip);

        customer.upgrade_to_vip();
        customer.update_email(String::from("carol@new.com"));

        assert_eq!(customer.display_badge(), "[VIP Member] Carol");
        assert!(customer.is_vip);
        assert_eq!(customer.email, "carol@new.com");
    }

    #[test]
    fn test_finalize_order_consumes_and_calculates_vip_discount() {
        let customer = Customer::new(1, String::from("Alice"), String::from("a@test.com"), true);
        let product = Product::new(10, String::from("Desk Pad"), 2000, 5);
        let order = PendingOrder {
            order_id: OrderId(77),
            customer,
            product,
            quantity: 2, // 2 * $20.00 = $40.00 ($4000 cents)
        };

        // VIP discount: 10% off $4000 = $400 -> $3600 cents ($36.00)
        let receipt = finalize_order(order);
        assert_eq!(receipt.order_id, OrderId(77));
        assert_eq!(receipt.customer_name, "Alice");
        assert_eq!(receipt.product_name, "Desk Pad");
        assert_eq!(receipt.quantity, 2);
        assert_eq!(receipt.total_cents, 3600);
    }
}
