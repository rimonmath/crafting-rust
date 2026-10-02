use chapter_13::{
    checkout, Catalog, Coupon, Customer, OrderId, PaymentMethod, Product, ProductCategory,
    ShoppingCart,
};

fn main() {
    println!("=== MiniStore: Modules & Clean Project Architecture (Part II) ===\n");

    // 1. Initializing Catalog with Products
    let mut catalog = Catalog::new();
    let keyboard = Product::new(
        101,
        String::from("TECH-KEY-001"),
        String::from("Tenkeyless Mechanical Keyboard"),
        ProductCategory::Electronics,
        12000,
        2, // Only 2 in stock!
    );
    let mouse = Product::new(
        102,
        String::from("TECH-MOU-002"),
        String::from("Ergonomic Wireless Mouse"),
        ProductCategory::Electronics,
        4500,
        10,
    );
    catalog.add_product(keyboard);
    catalog.add_product(mouse);

    // 2. Initializing Customer
    let customer = Customer::new(
        301,
        String::from("Margaret Hamilton"),
        String::from("margaret@apollo.nasa.gov"),
        Some(String::from("+1-555-0199")),
        true,
    );

    println!(
        "1. Catalog initialized with {} products.",
        catalog.total_products()
    );
    println!(
        "2. Customer: {} ({})",
        customer.name,
        customer.formatted_phone()
    );

    // 3. Shopping Cart setup
    let mut cart = ShoppingCart::new();
    cart.add_item(101, 1, 12000); // 1 keyboard
    cart.add_item(102, 2, 4500); // 2 mice

    let coupon = Coupon::new(String::from("LAUNCH20"), 20);

    // 4. Processing Checkout through modular service layer
    println!("\n3. Processing checkout through modular services...");
    match checkout(
        OrderId(901),
        customer,
        &mut cart,
        &mut catalog,
        PaymentMethod::CreditCard {
            last_four: String::from("9876"),
        },
        Some(coupon),
    ) {
        Ok(mut order) => {
            println!(
                "   Checkout Order #{} created successfully!",
                order.order_id.0
            );
            println!(
                "   Subtotal: ${:.2} | Total: ${:.2}",
                order.subtotal_cents() as f64 / 100.0,
                order.total_cents() as f64 / 100.0
            );

            // 5. Order State Transitions
            println!("\n4. Order Lifecycle Transitions:");
            order.confirm(String::from("REC-901-HAMILTON")).unwrap();
            println!("   Order confirmed: {}", order.status.display_status());

            order.ship(String::from("TRK-FEDEX-77189")).unwrap();
            println!("   Order shipped:   {}", order.status.display_status());

            // Attempting illegal cancellation once shipped
            match order.cancel(String::from("Buyer changed mind")) {
                Ok(()) => println!("   Order cancelled!"),
                Err(err) => println!("   Cancellation prevented -> {}", err.message()),
            }

            order.mark_delivered().unwrap();
            println!(
                "   Final Lifecycle State: {}",
                order.status.display_status()
            );
        }
        Err(err) => println!("   Checkout failed: {}", err.message()),
    }
}
