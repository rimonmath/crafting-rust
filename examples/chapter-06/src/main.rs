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

    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    pub fn reduce_stock(&mut self, quantity: u32) -> Result<u32, &'static str> {
        if quantity > self.stock {
            Err("Insufficient stock available")
        } else {
            self.stock -= quantity;
            Ok(self.stock)
        }
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
}

/// OrderId implements `Copy`: simple stack value, never moved
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderId(pub u64);

/// An unconfirmed shopping order owning its line items
#[derive(Debug, Clone, PartialEq)]
pub struct PendingOrder {
    pub order_id: OrderId,
    pub customer: Customer,
    pub product: Product,
    pub quantity: u32,
}

/// Finalized invoice/receipt created when a PendingOrder is processed.
/// By taking ownership of `PendingOrder`, the order is consumed and cannot be re-processed!
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmedReceipt {
    pub receipt_id: String,
    pub order_id: OrderId,
    pub customer_name: String,
    pub product_name: String,
    pub quantity: u32,
    pub total_cents: u32,
}

/// Consumes ownership of a `PendingOrder` by value (Move Semantics).
/// The caller cannot reuse `order` after passing it here.
pub fn finalize_order(order: PendingOrder) -> ConfirmedReceipt {
    let subtotal = order.product.price_cents * order.quantity;
    let discount = if order.customer.is_vip {
        (subtotal * 10) / 100
    } else {
        0
    };
    let total_cents = subtotal - discount;

    ConfirmedReceipt {
        receipt_id: format!("REC-{}-{}", order.order_id.0, order.customer.id),
        order_id: order.order_id,
        customer_name: order.customer.name, // ownership of `String` moves into receipt
        product_name: order.product.name,   // ownership of `String` moves into receipt
        quantity: order.quantity,
        total_cents,
    }
}

fn main() {
    println!("=== MiniStore: Ownership, Move Semantics & Memory ===");

    // 1. Stack Allocation & Copy Trait: Primitive scalar types and Copy structs
    let order_id_1 = OrderId(9001);
    let order_id_2 = order_id_1; // Copied! Both remain fully valid on the stack.
    println!(
        "Copy Demonstration: id_1 = {:?}, id_2 = {:?}",
        order_id_1, order_id_2
    );

    // 2. Heap Allocation: String owns its character buffer on the heap
    let customer_name = String::from("Grace Hopper");
    let customer = Customer::new(
        201,
        customer_name, // Ownership of heap buffer MOVES into `customer`
        String::from("grace@example.com"),
        true,
    );
    // Note: `customer_name` is no longer valid here! Its ownership moved into `customer`.
    println!(
        "Customer Created: {} (VIP: {})",
        customer.name, customer.is_vip
    );

    // 3. Move Semantics vs Explicit Clone
    let product_original = Product::new(
        501,
        String::from("4K Ultra-Wide Monitor"),
        49999, // $499.99
        8,
    );

    // Explicit deep clone: allocates a separate String buffer on the heap
    let product_for_order = product_original.clone();
    println!(
        "Original Product retained: {} (Stock: {})",
        product_original.name, product_original.stock
    );
    println!(
        "Cloned Product for Order: {} (Stock: {})",
        product_for_order.name, product_for_order.stock
    );

    // 4. Moving ownership into an order pipeline
    let pending_order = PendingOrder {
        order_id: order_id_1,
        customer: customer.clone(),
        product: product_for_order,
        quantity: 1,
    };

    println!(
        "\nPending Order #{} created for {}",
        pending_order.order_id.0, pending_order.customer.name
    );

    // 5. Transferring ownership into `finalize_order` (Consuming the order)
    // `pending_order` is MOVED into `finalize_order`. It cannot be used again!
    let receipt = finalize_order(pending_order);

    println!("\n--- Order Confirmed (Ownership Consumed) ---");
    println!("Receipt ID: {}", receipt.receipt_id);
    println!("Customer:   {}", receipt.customer_name);
    println!(
        "Product:    {} x {}",
        receipt.product_name, receipt.quantity
    );
    println!("Total Paid: ${:.2}", receipt.total_cents as f64 / 100.0);

    // 6. Demonstrating Scope & RAII (Resource Acquisition Is Initialization)
    {
        println!("\n--- Entering Temporary Promotion Scope ---");
        let promo_code = String::from("SPRING_CLEANUP_2026");
        println!("Promo code active: {promo_code}");
        // When this block ends, `promo_code` goes out of scope and its heap buffer is automatically dropped!
    }
    println!(
        "Exited promotion scope: heap memory was automatically freed without garbage collection!"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_trait_preserves_original() {
        let id_a = OrderId(101);
        let id_b = id_a; // Copy
        assert_eq!(id_a, id_b);
        assert_eq!(id_a.0, 101);
        assert_eq!(id_b.0, 101);
    }

    #[test]
    fn test_clone_creates_independent_heap_allocation() {
        let original = Product::new(1, String::from("Item A"), 1000, 10);
        let mut cloned = original.clone();

        // Mutating cloned does not mutate original
        cloned.reduce_stock(5).unwrap();
        assert_eq!(cloned.stock, 5);
        assert_eq!(original.stock, 10);
        assert_eq!(original.name, cloned.name);
    }

    #[test]
    fn test_finalize_order_consumes_and_transforms() {
        let customer = Customer::new(1, String::from("Alice"), String::from("a@test.com"), true);
        let product = Product::new(10, String::from("Desk Pad"), 2000, 5); // $20.00
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
