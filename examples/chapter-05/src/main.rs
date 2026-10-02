#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub name: String,
    pub price_cents: u32,
    pub stock: u32,
}

impl Product {
    /// Associated constructor function
    pub fn new(id: u64, name: String, price_cents: u32, stock: u32) -> Self {
        Self {
            id,
            name,
            price_cents,
            stock,
        }
    }

    /// Method taking an immutable reference &self to check stock availability
    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    /// Method taking a mutable reference &mut self to decrement stock on purchase
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

#[derive(Debug, Clone, PartialEq)]
pub struct CartItem {
    pub product: Product,
    pub quantity: u32,
}

impl CartItem {
    pub fn new(product: Product, quantity: u32) -> Self {
        Self { product, quantity }
    }

    /// Computes total price for this line item
    pub fn total_price_cents(&self) -> u32 {
        self.product.price_cents * self.quantity
    }
}

/// Tuple struct: Type-safe order identifier (Newtype Pattern)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderId(pub u64);

/// Unit-like struct: Marker for orders with standard packaging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandardPackaging;

#[derive(Debug, Clone, PartialEq)]
pub struct OrderSummary {
    pub order_id: OrderId,
    pub customer_id: u64,
    pub total_items: u32,
    pub subtotal_cents: u32,
    pub discount_cents: u32,
    pub shipping_fee_cents: u32,
    pub final_total_cents: u32,
}

impl OrderSummary {
    pub fn build(
        order_id: OrderId,
        customer: &Customer,
        cart_items: &[CartItem],
    ) -> Result<Self, &'static str> {
        if cart_items.is_empty() {
            return Err("Cart cannot be empty to build order summary");
        }

        let mut subtotal_cents = 0;
        let mut total_items = 0;

        for item in cart_items {
            subtotal_cents += item.total_price_cents();
            total_items += item.quantity;
        }

        // VIP customers receive an automatic 10% discount; others get tiered discount
        let discount_percent = if customer.is_vip {
            10
        } else if subtotal_cents >= 15000 {
            15
        } else if subtotal_cents >= 10000 {
            10
        } else if subtotal_cents >= 5000 {
            5
        } else {
            0
        };

        let discount_cents = (subtotal_cents * discount_percent) / 100;
        let discounted_subtotal = subtotal_cents - discount_cents;

        // Free shipping on discounted orders >= $100.00 (10000 cents)
        let shipping_fee_cents = if discounted_subtotal >= 10000 { 0 } else { 599 };

        let final_total_cents = discounted_subtotal + shipping_fee_cents;

        Ok(Self {
            order_id,
            customer_id: customer.id,
            total_items,
            subtotal_cents,
            discount_cents,
            shipping_fee_cents,
            final_total_cents,
        })
    }
}

fn main() {
    println!("=== MiniStore: Domain Models with Structs ===");

    // 1. Initializing customer domain struct
    let customer = Customer::new(
        101,
        String::from("Ada Lovelace"),
        String::from("ada@example.com"),
        true, // VIP member
    );
    println!(
        "Customer: {} ({}) | VIP: {}",
        customer.name, customer.email, customer.is_vip
    );

    // 2. Initializing products using associated constructor function
    let mut keyboard = Product::new(1001, String::from("Mechanical Keyboard"), 8999, 10);
    let mut mouse = Product::new(1002, String::from("Ergonomic Mouse"), 4999, 15);

    // 3. Demonstrating Struct Update Syntax
    // Create a special edition keyboard that shares price and stock with base keyboard
    let keyboard_rgb = Product {
        id: 1003,
        name: String::from("Mechanical Keyboard (RGB Edition)"),
        price_cents: 10999,
        ..keyboard.clone()
    };
    println!(
        "\nCatalog Item 1: {:?} (In Stock: {})",
        keyboard,
        keyboard.is_in_stock()
    );
    println!(
        "Catalog Item 2: {:?} (In Stock: {})",
        mouse,
        mouse.is_in_stock()
    );
    println!("Catalog Item 3 (from update syntax): {:?}", keyboard_rgb);

    // 4. Assembling CartItems
    let cart = [
        CartItem::new(keyboard.clone(), 1),
        CartItem::new(mouse.clone(), 2),
    ];

    println!("\n--- Customer Cart ---");
    for item in &cart {
        println!(
            "- {} x {} @ ${:.2} = ${:.2}",
            item.product.name,
            item.quantity,
            item.product.price_cents as f64 / 100.0,
            item.total_price_cents() as f64 / 100.0
        );
    }

    // 5. Building order summary
    let order_id = OrderId(5001);
    let summary = OrderSummary::build(order_id, &customer, &cart)
        .expect("Order summary build should succeed");

    println!("\n--- Order Summary ---");
    println!("Order ID: #{}", summary.order_id.0);
    println!("Total Items: {}", summary.total_items);
    println!("Subtotal: ${:.2}", summary.subtotal_cents as f64 / 100.0);
    println!("Discount: -${:.2}", summary.discount_cents as f64 / 100.0);
    println!(
        "Shipping: {}",
        if summary.shipping_fee_cents == 0 {
            "FREE".to_string()
        } else {
            format!("${:.2}", summary.shipping_fee_cents as f64 / 100.0)
        }
    );
    println!(
        "Final Total: ${:.2}",
        summary.final_total_cents as f64 / 100.0
    );

    // 6. Mutating inventory using &mut self method
    println!("\n--- Updating Inventory ---");
    keyboard.reduce_stock(1).expect("Stock should decrement");
    mouse.reduce_stock(2).expect("Stock should decrement");
    println!("Remaining Keyboard Stock: {}", keyboard.stock);
    println!("Remaining Mouse Stock: {}", mouse.stock);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_creation_and_stock_reduction() {
        let mut p = Product::new(1, String::from("Item"), 1000, 5);
        assert!(p.is_in_stock());
        assert_eq!(p.reduce_stock(2), Ok(3));
        assert_eq!(p.stock, 3);
        assert_eq!(p.reduce_stock(4), Err("Insufficient stock available"));
        assert_eq!(p.stock, 3); // stock unchanged on error
    }

    #[test]
    fn test_cart_item_total() {
        let p = Product::new(1, String::from("USB Hub"), 2500, 10);
        let item = CartItem::new(p, 3);
        assert_eq!(item.total_price_cents(), 7500);
    }

    #[test]
    fn test_struct_update_syntax() {
        let base = Product::new(10, String::from("Base"), 500, 20);
        let updated = Product {
            id: 11,
            name: String::from("Variant"),
            ..base
        };
        assert_eq!(updated.id, 11);
        assert_eq!(updated.name, "Variant");
        assert_eq!(updated.price_cents, 500);
        assert_eq!(updated.stock, 20);
    }

    #[test]
    fn test_order_summary_vip_discount() {
        let vip = Customer::new(1, String::from("VIP"), String::from("vip@test.com"), true);
        let p = Product::new(10, String::from("Item"), 10000, 5); // $100.00
        let items = [CartItem::new(p, 1)];

        let summary = OrderSummary::build(OrderId(1), &vip, &items).unwrap();
        // 10% VIP discount on 10000 = 1000 -> discounted subtotal = 9000 ($90.00)
        // Under $100 -> shipping fee = 599
        // Final total = 9000 + 599 = 9599
        assert_eq!(summary.subtotal_cents, 10000);
        assert_eq!(summary.discount_cents, 1000);
        assert_eq!(summary.shipping_fee_cents, 599);
        assert_eq!(summary.final_total_cents, 9599);
    }

    #[test]
    fn test_order_summary_empty_cart() {
        let customer = Customer::new(2, String::from("User"), String::from("u@test.com"), false);
        let empty_items: [CartItem; 0] = [];
        let res = OrderSummary::build(OrderId(2), &customer, &empty_items);
        assert!(res.is_err());
    }
}
