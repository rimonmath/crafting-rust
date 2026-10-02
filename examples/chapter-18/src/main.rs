use chapter_18::{
    best_contact_info, checkout, find_higher_priced, format_tax_summary, make_percentage_discount,
    make_threshold_discount, make_vip_discount, store_policy, summarize_item, ApiResponse, Catalog,
    Coupon, Customer, DiscountTierIter, OrderId, OrderReceipt, Page, PaymentMethod, Product,
    ProductCategory, ShoppingCart, Summarizable,
};

fn main() {
    println!("=== MiniStore: Closures & Dynamic Policies (Part III) ===\n");

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

    // 2. Querying Catalog via Iterators & Functional Adapters
    println!("\n2. Catalog Iteration & Functional Queries:");
    let electronics = catalog.filter_by_category(ProductCategory::Electronics);
    println!(
        "   Found {} Electronics product(s): {:?}",
        electronics.len(),
        electronics
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
    );

    let affordable = catalog.products_in_price_range(4000, 15000);
    println!(
        "   Products under $150: {}",
        affordable
            .iter()
            .map(|p| format!("{} (${:.2})", p.name, p.price_cents as f64 / 100.0))
            .collect::<Vec<_>>()
            .join(", ")
    );

    println!(
        "   Total Inventory Valuation: ${:.2}",
        catalog.total_inventory_valuation() as f64 / 100.0
    );

    // 3. Dynamic Filtering via Closures (`Fn(&Product) -> bool`)
    println!("\n3. Dynamic Product Search via Closures:");
    let max_budget = 10000; // $100.00 captured from enclosing scope!
    let budget_items = catalog.find_products(|p| p.price_cents <= max_budget);
    println!(
        "   Products within budget (${:.2}): {:?}",
        max_budget as f64 / 100.0,
        budget_items
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
    );

    // 4. Generic Pagination Demonstration (Page<Product>)
    println!("\n4. Browsing Catalog with Generic Pagination (Page<Product>):");
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

    // 5. Customer Profile setup
    let customer_name = String::from("Margaret Hamilton");
    let customer = Customer::new(
        301,
        customer_name.clone(),
        String::from("margaret@apollo.nasa.gov"),
        Some(String::from("+1-555-0199")),
        true,
    );
    println!(
        "\n5. Customer: {} ({})",
        customer.name,
        customer.formatted_phone()
    );

    // 6. Closures as Dynamic Factories (`move` keyword)
    println!("\n6. Dynamic Promotion Factories via Closures:");
    let promo_10_percent = make_percentage_discount(10);
    let promo_threshold = make_threshold_discount(15000, 2000); // $20 off orders >= $150
    let promo_vip = make_vip_discount(10); // 10% for VIPs

    let sample_subtotal = 21000; // $210.00
    println!(
        "   Simulating discounts on sample subtotal (${:.2}):",
        sample_subtotal as f64 / 100.0
    );
    println!(
        "   - 10% General Sale:   ${:.2}",
        promo_10_percent(sample_subtotal) as f64 / 100.0
    );
    println!(
        "   - $20 over $150:      ${:.2}",
        promo_threshold(sample_subtotal) as f64 / 100.0
    );
    println!(
        "   - VIP Member Benefit: ${:.2}",
        promo_vip(&customer, sample_subtotal) as f64 / 100.0
    );

    // 7. Lifetimes & Reference Safety
    println!("\n7. Lifetimes & Reference Safety:");
    let higher_value = find_higher_priced(&keyboard, &mouse);
    println!(
        "   Higher priced item: {} (${:.2})",
        higher_value.name,
        higher_value.price_cents as f64 / 100.0
    );

    let contact = best_contact_info(customer.phone.as_deref(), &customer.email);
    println!("   Best contact info: {contact}");
    println!("   Store Policy ('static): {}", store_policy());

    // 8. Shared Behaviors via Traits (Taxable & Summarizable)
    println!("\n8. Shared Behaviors via Traits:");
    println!(
        "   Tax Summary: {}",
        format_tax_summary(&keyboard, keyboard.price_cents)
    );
    println!(
        "   Tax Summary: {}",
        format_tax_summary(&mouse, mouse.price_cents)
    );
    println!("   Customer Summary: {}", summarize_item(&customer));

    // 9. Shopping Cart setup & Stateful Closures (`FnMut`)
    let mut cart = ShoppingCart::new();
    cart.add_item(101, 1, 12000); // 1 keyboard
    cart.add_item(102, 2, 4500); // 2 mice

    println!("\n9. Cart Iteration & Stateful Closures (FnMut):");
    println!("   Iterating over cart items directly:");
    for item in &cart {
        println!(
            "   -> Product ID #{}: Qty {} @ ${:.2} each",
            item.product_id,
            item.quantity,
            item.unit_price_cents as f64 / 100.0
        );
    }

    println!("   Custom Cart Line Item Reports:");
    for report in cart.report_iter() {
        println!("      {report}");
    }

    // Demonstrating custom DiscountTierIter
    let discount_tiers: Vec<u32> = DiscountTierIter::new(5, 20).collect();
    println!("   Progressive Discount Tiers available: {discount_tiers:?}%");

    // Applying stateful FnMut discount: $5 off multi-quantity items
    let mut total_multi_discount = 0;
    cart.apply_custom_discount(|item| {
        if item.quantity > 1 {
            let disc = 500; // $5 off per unit
            total_multi_discount += disc;
            disc
        } else {
            0
        }
    });
    println!(
        "   Stateful FnMut applied ${:.2} discount on multi-quantity items!",
        total_multi_discount as f64 / 100.0
    );

    let coupon = Coupon::new(String::from("LAUNCH20"), 20);

    // 10. Processing Checkout through modular service layer
    println!("\n10. Processing checkout through modular services...");
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
            println!("   Checkout Order {} created successfully!", order.order_id);
            println!(
                "   Subtotal: ${:.2} | Total Units: {} | Final Total: ${:.2}",
                order.subtotal_cents() as f64 / 100.0,
                order.total_quantity(),
                order.total_cents() as f64 / 100.0
            );

            // 11. Order State Transitions using Display trait!
            println!("\n11. Order Lifecycle Transitions:");
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

            // 12. Zero-Copy Struct with Lifetime: OrderReceipt<'a>
            println!("\n12. Zero-Copy Receipt Borrowing Order & Slices (OrderReceipt<'a>):");
            let cashier_note = "VIP Client - Express courier delivery verified";
            let receipt = OrderReceipt::new(&order, &customer_name, cashier_note);

            println!("   Display format: {receipt}");
            println!("   Trait Summary:  {}", receipt.summary());
            println!("\n--- Printed Slip ---");
            println!("{}", receipt.generate_slip());
            println!("--------------------");
        }
        Err(err) => println!("   Checkout failed: {err}"),
    }
}
