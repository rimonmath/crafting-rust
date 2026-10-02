use chapter_15::{
    checkout, format_tax_summary, summarize_item, ApiResponse, Catalog, Coupon, Customer, OrderId,
    Page, PaymentMethod, Product, ProductCategory, ShoppingCart,
};

fn main() {
    println!("=== MiniStore: Traits & Shared Behavior (Part III) ===\n");

    // 1. Initializing Catalog with Products
    let mut catalog = Catalog::new();
    let keyboard = Product::new(
        101,
        String::from("TECH-KEY-001"),
        String::from("Tenkeyless Mechanical Keyboard"),
        ProductCategory::Electronics,
        12000,
        5,
    );
    let mouse = Product::new(
        102,
        String::from("TECH-MOU-002"),
        String::from("Ergonomic Wireless Mouse"),
        ProductCategory::Electronics,
        4500,
        10,
    );
    let monitor = Product::new(
        103,
        String::from("TECH-MON-003"),
        String::from("27-inch 4K IPS Display"),
        ProductCategory::Electronics,
        35000,
        3,
    );
    catalog.add_product(keyboard.clone());
    catalog.add_product(mouse.clone());
    catalog.add_product(monitor);

    println!(
        "1. Catalog initialized with {} products.",
        catalog.total_products()
    );

    // 2. Generic Pagination Demonstration (Page<Product>)
    println!("\n2. Browsing Catalog with Generic Pagination (Page<Product>):");
    let product_page: Page<Product> = catalog.paginate(1, 2);
    println!(
        "   Page {} of {} (Total Items: {})",
        product_page.page,
        product_page.total_pages(),
        product_page.total_items
    );
    for item in &product_page.items {
        println!(
            "   - [{}] {} (${:.2})",
            item.sku,
            item.name,
            item.price_cents as f64 / 100.0
        );
    }
    println!("   Has next page? {}", product_page.has_next());

    // Generic transformation: Page<Product> -> Page<String>
    let name_page: Page<String> = product_page.map(|p| p.name);
    println!("   Transformed to Page<String>: {:?}", name_page.items);

    // Generic API response container
    let api_response = ApiResponse::ok(catalog.paginate(2, 2), catalog.total_products());
    if let ApiResponse::Success { data, total } = api_response {
        println!(
            "   API Page 2 response: {} product(s) returned out of {} total.",
            data.item_count(),
            total
        );
    }

    // 3. Customer Profile setup
    let customer = Customer::new(
        301,
        String::from("Margaret Hamilton"),
        String::from("margaret@apollo.nasa.gov"),
        Some(String::from("+1-555-0199")),
        true,
    );
    println!(
        "\n3. Customer: {} ({})",
        customer.name,
        customer.formatted_phone()
    );

    // 4. Shared Behaviors via Traits (Taxable & Summarizable)
    println!("\n4. Shared Behaviors via Traits:");
    println!(
        "   Tax Summary: {}",
        format_tax_summary(&keyboard, keyboard.price_cents)
    );
    println!(
        "   Tax Summary: {}",
        format_tax_summary(&mouse, mouse.price_cents)
    );
    println!("   Customer Summary: {}", summarize_item(&customer));

    // 5. Shopping Cart setup
    let mut cart = ShoppingCart::new();
    cart.add_item(101, 1, 12000); // 1 keyboard
    cart.add_item(102, 2, 4500); // 2 mice

    let coupon = Coupon::new(String::from("LAUNCH20"), 20);

    // 6. Processing Checkout through modular service layer
    println!("\n5. Processing checkout through modular services...");
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
            // Using Display trait implementation for OrderId!
            println!("   Checkout Order {} created successfully!", order.order_id);
            println!(
                "   Subtotal: ${:.2} | Total: ${:.2}",
                order.subtotal_cents() as f64 / 100.0,
                order.total_cents() as f64 / 100.0
            );

            // 7. Order State Transitions using Display trait!
            println!("\n6. Order Lifecycle Transitions:");
            order.confirm(String::from("REC-901-HAMILTON")).unwrap();
            println!("   Order confirmed: {}", order.status);

            order.ship(String::from("TRK-FEDEX-77189")).unwrap();
            println!("   Order shipped:   {}", order.status);

            // Attempting illegal cancellation once shipped
            match order.cancel(String::from("Buyer changed mind")) {
                Ok(()) => println!("   Order cancelled!"),
                Err(err) => println!("   Cancellation prevented -> {err}"),
            }

            order.mark_delivered().unwrap();
            println!("   Final Lifecycle State: {}", order.status);
            println!("   Order Summary: {}", summarize_item(&order));
        }
        Err(err) => println!("   Checkout failed: {err}"),
    }
}
