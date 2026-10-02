use chapter_20::{
    best_contact_info, checkout, find_higher_priced, format_tax_summary, make_percentage_discount,
    make_threshold_discount, make_vip_discount, parallel_batch_valuation, store_policy,
    summarize_item, ApiResponse, Catalog, CategoryNode, ConcurrentSalesTracker, Coupon, Customer,
    DiscountTierIter, OrderId, OrderNotification, OrderNotificationChannel, OrderReceipt, Page,
    PaymentMethod, Product, ProductCategory, PromotionPipeline, SharedAuditor, ShoppingCart,
    StoreSession, Summarizable,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;

fn main() {
    println!("=== MiniStore: Concurrency & Fearless Multi-Threading (Part IV) ===\n");

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

    // Recursive Category Hierarchy using Box<CategoryNode>
    let mut root_category = CategoryNode::new("Electronics");
    let mut computers = CategoryNode::new("Computers");
    computers.add_subcategory(CategoryNode::new("Laptops"));
    computers.add_subcategory(CategoryNode::new("Desktops"));
    let mut peripherals = CategoryNode::new("Peripherals");
    peripherals.add_subcategory(CategoryNode::new("Keyboards"));
    peripherals.add_subcategory(CategoryNode::new("Mice"));
    root_category.add_subcategory(computers);
    root_category.add_subcategory(peripherals);

    println!(
        "   Recursive Category Tree (Box<CategoryNode>): {} categories across depth {}",
        root_category.total_categories(),
        root_category.depth()
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
    let shared_customer = Rc::new(customer.clone());
    println!(
        "\n5. Customer: {} ({}) [Rc Strong Count: {}]",
        shared_customer.name,
        shared_customer.formatted_phone(),
        Rc::strong_count(&shared_customer)
    );
    let session_customer_ref = Rc::clone(&shared_customer);
    println!(
        "   Cloned Rc handle for store session. New Strong Count: {}",
        Rc::strong_count(&shared_customer)
    );

    // 6. Dynamic Promotion Pipelines via Box<dyn Fn> & SharedAuditor (Rc<RefCell>)
    println!("\n6. Dynamic Promotion Pipelines via Box<dyn Fn> & Interior Mutability:");
    let mut pipeline = PromotionPipeline::new();
    pipeline.add_rule(make_percentage_discount(10)); // 10%
    pipeline.add_rule(make_threshold_discount(15000, 2000)); // $20 off orders >= $150
    println!(
        "   Promotion Pipeline initialized with {} heterogeneous boxed rules.",
        pipeline.len()
    );

    let sample_subtotal = 21000; // $210.00
    println!(
        "   Simulating discounts on sample subtotal (${:.2}):",
        sample_subtotal as f64 / 100.0
    );
    println!(
        "   - Combined Savings (apply_all):  ${:.2}",
        pipeline.apply_all(sample_subtotal) as f64 / 100.0
    );
    println!(
        "   - Best Single Deal (apply_best): ${:.2}",
        pipeline.apply_best(sample_subtotal) as f64 / 100.0
    );

    let promo_vip = make_vip_discount(10);
    println!(
        "   - VIP Member Benefit:            ${:.2}",
        promo_vip(&customer, sample_subtotal) as f64 / 100.0
    );

    // Shared interior mutability audit tracker
    let shared_auditor = SharedAuditor::new();
    let checkout_auditor_handle = shared_auditor.clone();
    checkout_auditor_handle.record(pipeline.apply_best(sample_subtotal));
    println!(
        "   Shared Auditor (Rc<RefCell>): Recorded {} op(s), Total Saved: ${:.2} [Strong Count: {}]",
        shared_auditor.operations_count(),
        shared_auditor.total_discount_given() as f64 / 100.0,
        shared_auditor.strong_count()
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

    // 13. Custom Smart Pointer with Deref & Drop (StoreSession<T>)
    println!("\n13. Custom Smart Pointer with Deref & Drop (StoreSession<T>):");
    let session_active_tracker = Rc::new(RefCell::new(false));
    {
        let session = StoreSession::with_drop_flag(
            "SESS-HAMILTON-001",
            (*session_customer_ref).clone(),
            Rc::clone(&session_active_tracker),
        );
        println!(
            "   Session [{}] active status: {}",
            session.session_id,
            *session_active_tracker.borrow()
        );
        // Transparent deref coercion to inner Customer methods:
        println!("   Deref coercion invocation: {}", session.display_badge());
        println!("   Ending session scope...");
    }
    println!(
        "   Session scope exited. Drop hook executed. Active status: {}",
        *session_active_tracker.borrow()
    );

    // 14. Concurrency: Multi-Threading, Channels, and Arc<Mutex<T>> (Part IV)
    println!("\n14. Concurrency: Threads, Message Channels & Arc<Mutex<T>>:");

    // A. Native Thread Spawning & Joining
    let tax_worker = thread::spawn(|| {
        let sample_line = 12000 * 2; // $240.00
        (sample_line * 15) / 100
    });
    let calculated_tax = tax_worker.join().expect("Worker thread panicked");
    println!(
        "   Thread-computed background tax: ${:.2}",
        calculated_tax as f64 / 100.0
    );

    // B. Message Passing with MPSC Channel
    let (channel, rx) = OrderNotificationChannel::new();
    let tx1 = channel.clone_sender();
    let tx2 = channel.clone_sender();

    let p1 = thread::spawn(move || {
        tx1.send(OrderNotification::OrderPlaced {
            order_id: 902,
            amount_cents: 8900,
        })
        .unwrap();
    });

    let p2 = thread::spawn(move || {
        tx2.send(OrderNotification::OrderStatusUpdated {
            order_id: 902,
            status: String::from("Confirmed"),
        })
        .unwrap();
    });

    p1.join().unwrap();
    p2.join().unwrap();
    drop(channel); // Close original sender so rx consumer loop finishes

    println!("   Received asynchronous order notifications across threads:");
    while let Ok(msg) = rx.recv() {
        match msg {
            OrderNotification::OrderPlaced {
                order_id,
                amount_cents,
            } => {
                println!(
                    "   -> [Channel] New Order #{order_id} placed for ${:.2}",
                    amount_cents as f64 / 100.0
                );
            }
            OrderNotification::OrderStatusUpdated { order_id, status } => {
                println!("   -> [Channel] Order #{order_id} status updated to '{status}'");
            }
            OrderNotification::AuditAlert { message } => {
                println!("   -> [Channel] Alert: {message}");
            }
        }
    }

    // C. Shared Mutable State with Arc<Mutex<T>>
    let sales_tracker = ConcurrentSalesTracker::new();
    let mut checkout_threads = Vec::new();

    for thread_id in 1..=4 {
        let tracker_clone = sales_tracker.clone();
        let handle = thread::spawn(move || {
            // Each thread records 3 transactions of $25.00
            for _ in 0..3 {
                tracker_clone.record_sale(2500);
            }
            thread_id
        });
        checkout_threads.push(handle);
    }

    for handle in checkout_threads {
        let tid = handle.join().unwrap();
        println!("   -> Checkout thread #{tid} finished recording concurrent sales.");
    }

    println!(
        "   Concurrent Sales Tracker (Arc<Mutex>): {} transactions, Total Revenue: ${:.2}",
        sales_tracker.transactions_count(),
        sales_tracker.total_revenue() as f64 / 100.0
    );

    // D. Parallel Inventory Valuation across CPU workers
    let batch_items = vec![(12000, 5), (4500, 10), (35000, 3), (2500, 20)];
    let parallel_total = parallel_batch_valuation(batch_items, 2);
    println!(
        "   Parallel Inventory Valuation (2 workers): ${:.2}",
        parallel_total as f64 / 100.0
    );
}
