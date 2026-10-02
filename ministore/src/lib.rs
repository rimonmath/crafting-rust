//! MiniStore Core Library
//!
//! Provides the core e-commerce domain models, catalog lookups,
//! transactional checkout, and error handling for the MiniStore application.

#[macro_use]
pub mod macros;
pub mod async_services;
pub mod catalog;
pub mod checkout;
pub mod concurrency;
pub mod config;
pub mod error;
pub mod models;
pub mod persistence;
pub mod promotions;
pub mod traits;
pub mod unsafe_utils;
pub mod web_api;

// Convenient top-level re-exports (facade)
pub use async_services::{
    AsyncNotificationBus, MockPaymentClient, WarehouseClient, aggregate_warehouse_stock,
    async_checkout,
};
pub use catalog::{Catalog, CategoryNode};
pub use checkout::{
    calculate_discount, checkout, count_products_by_department, order_dispatch_advisory,
};
pub use concurrency::{
    ConcurrentSalesTracker, OrderNotification, OrderNotificationChannel, parallel_batch_valuation,
};
pub use config::{AppConfig, Environment};
pub use error::StoreError;
pub use models::{
    ApiResponse, CartItem, CartReportIterator, Coupon, Customer, DiscountTierIter, Order, OrderId,
    OrderReceipt, OrderStatus, Page, PaymentMethod, Product, ProductCategory, ShoppingCart,
    StoreSession, best_contact_info, find_higher_priced, paginate, store_policy,
};
pub use persistence::{StorePersistence, StoreSnapshot};
pub use promotions::{
    BoxedDiscount, DiscountAuditor, PromotionPipeline, SharedAuditor, calculate_total_promotions,
    find_best_promotion, make_percentage_discount, make_threshold_discount, make_vip_discount,
};
pub use traits::{Summarizable, Taxable, format_tax_summary, summarize_item};
pub use unsafe_utils::{
    RawBarcodeBuffer, c_abi_price_diff, ministore_c_calculate_tax, safe_swap_prices,
    unsafe_raw_swap,
};
pub use web_api::{
    ApiError, AppState, CheckoutItemRequest, CheckoutRequest, CreateProductRequest, create_router,
    shutdown_signal,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_method_fees_and_descriptions() {
        let card = PaymentMethod::CreditCard {
            last_four: String::from("1234"),
        };
        let transfer = PaymentMethod::BankTransfer {
            reference: String::from("TX-99"),
        };
        let cod = PaymentMethod::CashOnDelivery;

        assert_eq!(card.fee_cents(), 150);
        assert_eq!(transfer.fee_cents(), 0);
        assert_eq!(cod.fee_cents(), 300);

        assert_eq!(card.description(), "Credit Card (ending in 1234)");
        assert_eq!(transfer.description(), "Bank Transfer (Ref: TX-99)");
        assert_eq!(cod.description(), "Cash on Delivery");
    }

    #[test]
    fn test_order_status_valid_lifecycle() {
        let customer = Customer::new(
            1,
            String::from("Alice"),
            String::from("a@a.com"),
            None,
            false,
        );
        let items = vec![CartItem::new(10, 1, 5000)];
        let mut order = Order::new(
            OrderId(100),
            customer,
            items,
            PaymentMethod::CashOnDelivery,
            None,
        );

        // Starts pending
        assert_eq!(order.status, OrderStatus::Pending);
        assert!(order.status.can_cancel());
        assert!(!order.status.is_terminal());

        // Cannot ship while pending
        let ship_err = order.ship(String::from("TRK-1")).unwrap_err();
        assert_eq!(
            ship_err,
            StoreError::InvalidStateTransition {
                current: String::from("Awaiting Confirmation"),
                action: String::from("ship"),
            }
        );

        // Confirm
        assert!(order.confirm(String::from("REC-100")).is_ok());
        assert_eq!(
            order.status,
            OrderStatus::Confirmed {
                receipt_id: String::from("REC-100")
            }
        );
        assert!(order.status.can_cancel());

        // Ship
        assert!(order.ship(String::from("TRK-100")).is_ok());
        assert_eq!(
            order.status,
            OrderStatus::Shipped {
                tracking_number: String::from("TRK-100")
            }
        );
        assert!(!order.status.can_cancel());

        // Deliver
        assert!(order.mark_delivered().is_ok());
        assert_eq!(order.status, OrderStatus::Delivered);
        assert!(order.status.is_terminal());
    }

    #[test]
    fn test_order_cancellation_prevention() {
        let customer = Customer::new(2, String::from("Bob"), String::from("b@b.com"), None, false);
        let items = vec![CartItem::new(20, 2, 2500)];
        let mut order = Order::new(
            OrderId(200),
            customer,
            items,
            PaymentMethod::CashOnDelivery,
            None,
        );

        // Cancel while pending succeeds
        assert!(order.cancel(String::from("Out of stock")).is_ok());
        assert_eq!(
            order.status,
            OrderStatus::Cancelled {
                reason: String::from("Out of stock")
            }
        );
        assert!(order.status.is_terminal());

        // Cannot confirm a cancelled order
        let confirm_err = order.confirm(String::from("REC-200")).unwrap_err();
        assert_eq!(
            confirm_err,
            StoreError::InvalidStateTransition {
                current: String::from("Cancelled (Reason: Out of stock)"),
                action: String::from("confirm"),
            }
        );
    }

    #[test]
    fn test_product_category_tax_rates() {
        assert_eq!(ProductCategory::Electronics.default_tax_rate(), 15);
        assert_eq!(ProductCategory::OfficeSupplies.default_tax_rate(), 5);
        assert_eq!(ProductCategory::Furniture.default_tax_rate(), 10);
        assert_eq!(
            ProductCategory::Custom(String::from("Handmade")).default_tax_rate(),
            8
        );
    }

    #[test]
    fn test_order_total_with_payment_fee() {
        let mut customer = Customer::new(
            3,
            String::from("Carol"),
            String::from("c@c.com"),
            None,
            false,
        );
        customer.upgrade_to_vip(); // 10% discount

        let items = vec![CartItem::new(1, 1, 10000)]; // $100.00 subtotal
        let order = Order::new(
            OrderId(300),
            customer,
            items,
            PaymentMethod::CreditCard {
                last_four: String::from("1111"),
            }, // $1.50 (150 cents) fee
            None,
        );

        // Subtotal: 10000 cents
        // VIP discount: 1000 cents
        // Card fee: 150 cents
        // Total: 10000 - 1000 + 150 = 9150 cents ($91.50)
        assert_eq!(order.total_cents(), 9150);
    }

    #[test]
    fn test_catalog_option_lookups() {
        let mut catalog = Catalog::new();
        let prod = Product::new(
            50,
            String::from("OFF-DESK-01"),
            String::from("Standing Desk"),
            ProductCategory::Furniture,
            35000,
            5,
        );
        catalog.add_product(prod);

        // Existing lookups return Some(&Product)
        assert!(catalog.find_by_sku("OFF-DESK-01").is_some());
        assert_eq!(catalog.find_by_sku("OFF-DESK-01").unwrap().id, 50);
        assert_eq!(
            catalog.find_by_id(50).map(|p| p.sku.as_str()),
            Some("OFF-DESK-01")
        );
        assert_eq!(catalog.product_price("OFF-DESK-01"), Some(35000));
        assert!(catalog.is_product_in_stock("OFF-DESK-01"));

        // Non-existent lookups return None
        assert_eq!(catalog.find_by_sku("UNKNOWN-SKU"), None);
        assert_eq!(catalog.find_by_id(999), None);
        assert_eq!(catalog.product_price("UNKNOWN-SKU"), None);
        assert!(!catalog.is_product_in_stock("UNKNOWN-SKU"));
    }

    #[test]
    fn test_customer_optional_phone() {
        let with_phone = Customer::new(
            1,
            String::from("Alice"),
            String::from("alice@ex.com"),
            Some(String::from("+1-202-555-0143")),
            false,
        );
        let no_phone = Customer::new(
            2,
            String::from("Bob"),
            String::from("bob@ex.com"),
            None,
            false,
        );

        assert_eq!(with_phone.formatted_phone(), "+1-202-555-0143");
        assert_eq!(no_phone.formatted_phone(), "Unspecified");
        assert!(with_phone.phone.is_some());
        assert!(no_phone.phone.is_none());
    }

    #[test]
    fn test_cart_item_option_lookup() {
        let mut cart = ShoppingCart::new();
        cart.add_item(10, 2, 1500);

        assert!(cart.get_item(10).is_some());
        assert_eq!(cart.get_item(10).unwrap().quantity, 2);
        assert!(cart.get_item(99).is_none());

        // Increment existing item via add_item
        cart.add_item(10, 3, 1500);
        assert_eq!(cart.get_item(10).unwrap().quantity, 5);
    }

    #[test]
    fn test_coupon_discount_and_take() {
        let customer = Customer::new(
            10,
            String::from("Dave"),
            String::from("d@d.com"),
            None,
            false,
        );
        let items = vec![CartItem::new(1, 1, 20000)]; // $200.00
        let coupon = Coupon::new(String::from("SAVE15"), 15); // 15% discount = $30.00 (3000 cents)

        let mut order = Order::new(
            OrderId(500),
            customer,
            items,
            PaymentMethod::BankTransfer {
                reference: String::from("REF1"),
            },
            Some(coupon),
        );

        assert_eq!(order.total_cents(), 17000);

        let extracted = order.remove_coupon();
        assert_eq!(
            extracted,
            Some(Coupon {
                code: String::from("SAVE15"),
                discount_percent: 15
            })
        );
        assert_eq!(order.coupon, None);
        assert_eq!(order.total_cents(), 20000);
    }

    #[test]
    fn test_product_stock_reduction_error() {
        let mut product = Product::new(
            1,
            String::from("SKU-1"),
            String::from("Book"),
            ProductCategory::OfficeSupplies,
            1000,
            5,
        );

        // Success
        assert_eq!(product.reduce_stock(3), Ok(2));
        assert_eq!(product.stock, 2);

        // Insufficient stock error
        let err = product.reduce_stock(5).unwrap_err();
        assert_eq!(
            err,
            StoreError::InsufficientStock {
                available: 2,
                requested: 5,
            }
        );
        assert_eq!(
            err.message(),
            "Insufficient stock: requested 5, but only 2 available"
        );
    }

    #[test]
    fn test_coupon_validation_error() {
        let empty_coupon = Coupon::new(String::from("  "), 10);
        assert_eq!(
            empty_coupon.validate(),
            Err(StoreError::InvalidCoupon {
                code: String::from("  "),
                reason: String::from("Coupon code cannot be empty"),
            })
        );

        let excessive_coupon = Coupon::new(String::from("MAX150"), 150);
        assert_eq!(
            excessive_coupon.validate(),
            Err(StoreError::InvalidCoupon {
                code: String::from("MAX150"),
                reason: String::from("Discount percentage 150 must be between 1 and 100"),
            })
        );

        let valid = Coupon::new(String::from("DISC25"), 25);
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_checkout_error_propagation_and_success() {
        let mut catalog = Catalog::new();
        catalog.add_product(Product::new(
            10,
            String::from("PEN-01"),
            String::from("Gel Pen"),
            ProductCategory::OfficeSupplies,
            200,
            4,
        ));

        let customer = Customer::new(5, String::from("Eve"), String::from("e@e.com"), None, false);

        // 1. Empty cart error
        let mut empty_cart = ShoppingCart::new();
        let res = checkout(
            OrderId(1),
            customer.clone(),
            &mut empty_cart,
            &mut catalog,
            PaymentMethod::CashOnDelivery,
            None,
        );
        assert_eq!(res.unwrap_err(), StoreError::EmptyCart);

        // 2. Product not in catalog
        let mut ghost_cart = ShoppingCart::new();
        ghost_cart.add_item(999, 1, 500);
        let res = checkout(
            OrderId(2),
            customer.clone(),
            &mut ghost_cart,
            &mut catalog,
            PaymentMethod::CashOnDelivery,
            None,
        );
        assert_eq!(
            res.unwrap_err(),
            StoreError::ProductNotFound {
                identifier: String::from("ID #999")
            }
        );

        // 3. Insufficient stock error
        let mut big_cart = ShoppingCart::new();
        big_cart.add_item(10, 10, 200); // Catalog only has 4!
        let res = checkout(
            OrderId(3),
            customer.clone(),
            &mut big_cart,
            &mut catalog,
            PaymentMethod::CashOnDelivery,
            None,
        );
        assert_eq!(
            res.unwrap_err(),
            StoreError::InsufficientStock {
                available: 4,
                requested: 10,
            }
        );

        // 4. Successful checkout
        let mut valid_cart = ShoppingCart::new();
        valid_cart.add_item(10, 3, 200);
        let res = checkout(
            OrderId(4),
            customer,
            &mut valid_cart,
            &mut catalog,
            PaymentMethod::CashOnDelivery,
            None,
        );
        assert!(res.is_ok());
        let order = res.unwrap();
        assert_eq!(order.order_id, OrderId(4));
        assert_eq!(catalog.find_by_id(10).unwrap().stock, 1); // 4 - 3 = 1 remaining!
        assert!(valid_cart.is_empty()); // Items moved into order
    }

    #[test]
    fn test_generic_pagination_with_integers() {
        let numbers: Vec<i32> = vec![10, 20, 30, 40, 50];

        // Page 1 with 2 per page -> [10, 20]
        let page1 = paginate(numbers.clone(), 1, 2);
        assert_eq!(page1.items, vec![10, 20]);
        assert_eq!(page1.page, 1);
        assert_eq!(page1.per_page, 2);
        assert_eq!(page1.total_items, 5);
        assert_eq!(page1.total_pages(), 3);
        assert!(page1.has_next());
        assert!(!page1.has_previous());

        // Page 2 with 2 per page -> [30, 40]
        let page2 = paginate(numbers.clone(), 2, 2);
        assert_eq!(page2.items, vec![30, 40]);
        assert!(page2.has_next());
        assert!(page2.has_previous());

        // Page 3 with 2 per page -> [50]
        let page3 = paginate(numbers.clone(), 3, 2);
        assert_eq!(page3.items, vec![50]);
        assert!(!page3.has_next());
        assert!(page3.has_previous());

        // Out of bounds page
        let page_empty = paginate(numbers, 4, 2);
        assert!(page_empty.is_empty());
    }

    #[test]
    fn test_generic_catalog_product_pagination() {
        let mut catalog = Catalog::new();
        catalog.add_product(Product::new(
            1,
            String::from("SKU-1"),
            String::from("Keyboard"),
            ProductCategory::Electronics,
            5000,
            10,
        ));
        catalog.add_product(Product::new(
            2,
            String::from("SKU-2"),
            String::from("Mouse"),
            ProductCategory::Electronics,
            2500,
            20,
        ));
        catalog.add_product(Product::new(
            3,
            String::from("SKU-3"),
            String::from("Monitor"),
            ProductCategory::Electronics,
            20000,
            5,
        ));

        let page1 = catalog.paginate(1, 2);
        assert_eq!(page1.item_count(), 2);
        assert_eq!(page1.total_pages(), 2);
        assert_eq!(page1.items[0].sku, "SKU-1");
        assert_eq!(page1.items[1].sku, "SKU-2");

        let page2 = catalog.paginate(2, 2);
        assert_eq!(page2.item_count(), 1);
        assert_eq!(page2.items[0].sku, "SKU-3");
    }

    #[test]
    fn test_generic_page_map_transformation() {
        let products = vec![
            Product::new(
                1,
                String::from("SKU-A"),
                String::from("Rust Book"),
                ProductCategory::OfficeSupplies,
                3000,
                10,
            ),
            Product::new(
                2,
                String::from("SKU-B"),
                String::from("Go Book"),
                ProductCategory::OfficeSupplies,
                2800,
                15,
            ),
        ];

        let product_page = paginate(products, 1, 10);
        // Transform Page<Product> into Page<String> (names)
        let name_page: Page<String> = product_page.map(|p| p.name);

        assert_eq!(
            name_page.items,
            vec![String::from("Rust Book"), String::from("Go Book")]
        );
        assert_eq!(name_page.page, 1);
        assert_eq!(name_page.total_items, 2);
    }

    #[test]
    fn test_generic_api_response_wrapper() {
        let success_resp: ApiResponse<String> =
            ApiResponse::ok(String::from("Operation complete"), 1);
        assert!(success_resp.is_success());
        assert_eq!(
            success_resp.data(),
            Some(&String::from("Operation complete"))
        );

        let error_resp: ApiResponse<String> = ApiResponse::err(String::from("Invalid credentials"));
        assert!(!error_resp.is_success());
        assert_eq!(error_resp.data(), None);

        // Nested generic: ApiResponse containing a Page of integers
        let paged_data = paginate(vec![1, 2, 3], 1, 2);
        let nested_resp = ApiResponse::ok(paged_data, 3);
        assert!(nested_resp.is_success());
        assert_eq!(nested_resp.data().unwrap().total_pages(), 2);
    }

    #[test]
    fn test_taxable_trait_and_default_method() {
        let category = ProductCategory::Electronics;
        assert_eq!(category.tax_rate(), 15);
        // Default calculate_tax implementation: (10000 * 15) / 100 = 1500
        assert_eq!(category.calculate_tax(10000), 1500);

        let product = Product::new(
            1,
            String::from("SKU-1"),
            String::from("Headphones"),
            ProductCategory::Electronics,
            8000,
            10,
        );
        assert_eq!(product.tax_rate(), 15);
        assert_eq!(product.calculate_tax(8000), 1200);
    }

    #[test]
    fn test_summarizable_trait_on_domain_models() {
        let product = Product::new(
            10,
            String::from("SKU-KEY"),
            String::from("Mechanical Keyboard"),
            ProductCategory::Electronics,
            12000,
            5,
        );
        assert_eq!(
            product.summary(),
            "Product #10: Mechanical Keyboard [SKU-KEY] - $120.00"
        );

        let customer = Customer::new(
            20,
            String::from("Ada Lovelace"),
            String::from("ada@analytics.org"),
            None,
            true,
        );
        assert_eq!(
            customer.summary(),
            "Customer #20: Ada Lovelace <ada@analytics.org>"
        );
    }

    #[test]
    fn test_generic_trait_bound_functions() {
        let product = Product::new(
            1,
            String::from("SKU-MOU"),
            String::from("Mouse"),
            ProductCategory::Electronics,
            4000,
            10,
        );
        let summary = summarize_item(&product);
        assert!(summary.contains("Mouse"));
        assert!(summary.contains("$40.00"));

        let tax_summary = format_tax_summary(&product, product.price_cents);
        assert!(tax_summary.contains("Mouse"));
        assert!(tax_summary.contains("Tax: $6.00 (15%)"));
    }

    #[test]
    fn test_display_trait_implementations() {
        let order_id = OrderId(99);
        assert_eq!(format!("{order_id}"), "#99");

        let status = OrderStatus::Shipped {
            tracking_number: String::from("TRK-1234"),
        };
        assert_eq!(format!("{status}"), "Shipped (Tracking: TRK-1234)");

        let err = StoreError::EmptyCart;
        assert_eq!(
            format!("{err}"),
            "Cannot checkout with an empty shopping cart"
        );
    }

    #[test]
    fn test_lifetime_annotated_product_comparison() {
        let p1 = Product::new(
            1,
            String::from("SKU-1"),
            String::from("Budget Mouse"),
            ProductCategory::Electronics,
            2500,
            10,
        );
        let p2 = Product::new(
            2,
            String::from("SKU-2"),
            String::from("Pro Keyboard"),
            ProductCategory::Electronics,
            11000,
            5,
        );

        let higher = find_higher_priced(&p1, &p2);
        assert_eq!(higher.id, 2);
        assert_eq!(higher.name, "Pro Keyboard");

        // Comparison order invariance
        let higher_rev = find_higher_priced(&p2, &p1);
        assert_eq!(higher_rev.id, 2);
    }

    #[test]
    fn test_lifetime_annotated_contact_resolution() {
        let phone = String::from("+1-800-RUST");
        let email = String::from("contact@ministore.rs");

        // Primary phone available
        let contact1 = best_contact_info(Some(&phone), &email);
        assert_eq!(contact1, "+1-800-RUST");

        // Fallback to email when phone is None
        let contact2 = best_contact_info(None, &email);
        assert_eq!(contact2, "contact@ministore.rs");

        // Fallback to email when phone is empty string
        let empty_phone = String::from("   ");
        let contact3 = best_contact_info(Some(&empty_phone), &email);
        assert_eq!(contact3, "contact@ministore.rs");
    }

    #[test]
    fn test_zero_copy_order_receipt_and_traits() {
        let customer = Customer::new(
            1,
            String::from("Katherine Johnson"),
            String::from("katherine@nasa.gov"),
            Some(String::from("+1-555-0100")),
            true,
        );
        let items = vec![CartItem::new(10, 2, 4500)];
        let order = Order::new(
            OrderId(701),
            customer,
            items,
            PaymentMethod::CashOnDelivery,
            None,
        );

        let cust_name = String::from("Katherine Johnson");
        let note = "Priority courier handling";

        let receipt = OrderReceipt::new(&order, &cust_name, note);

        assert_eq!(receipt.order_id(), OrderId(701));
        assert_eq!(receipt.customer_name(), "Katherine Johnson");
        assert_eq!(receipt.note(), "Priority courier handling");

        // Display trait test
        let display_str = format!("{receipt}");
        assert!(display_str.contains("Receipt for Order #701 (Katherine Johnson)"));
        assert!(display_str.contains("$84.00")); // $90.00 - 10% VIP discount ($9.00) + $3.00 COD fee

        // Summarizable trait test
        let summary_str = receipt.summary();
        assert!(summary_str.contains("Receipt: Order #701 for Katherine Johnson"));

        // Slip generation test
        let slip = receipt.generate_slip();
        assert!(slip.contains("=== RECEIPT: #701 ==="));
        assert!(slip.contains("Customer: Katherine Johnson"));
        assert!(slip.contains("Priority courier handling"));
    }

    #[test]
    fn test_static_lifetime_policy() {
        let policy: &'static str = store_policy();
        assert!(policy.contains("30-Day Hassle-Free Returns"));
    }

    #[test]
    fn test_catalog_iterator_queries() {
        let mut catalog = Catalog::new();
        catalog.add_product(Product::new(
            1,
            String::from("TECH-1"),
            String::from("Keyboard"),
            ProductCategory::Electronics,
            12000,
            5,
        ));
        catalog.add_product(Product::new(
            2,
            String::from("OFFICE-1"),
            String::from("Desk Pad"),
            ProductCategory::OfficeSupplies,
            6000,
            10,
        ));
        catalog.add_product(Product::new(
            3,
            String::from("TECH-2"),
            String::from("Mouse"),
            ProductCategory::Electronics,
            4000,
            8,
        ));

        // Category filter using iterator
        let electronics = catalog.filter_by_category(ProductCategory::Electronics);
        assert_eq!(electronics.len(), 2);

        // Price range filtering
        let mid_range = catalog.products_in_price_range(5000, 15000);
        assert_eq!(mid_range.len(), 2); // Keyboard ($120.00) & Jacket ($60.00)

        // Inventory valuation using map & sum
        // Keyboard: 12000 * 5 = 60000
        // Jacket:    6000 * 10 = 60000
        // Mouse:     4000 * 8 = 32000
        // Total = 152000 cents
        assert_eq!(catalog.total_inventory_valuation(), 152000);

        // IntoIterator for &Catalog
        let mut count = 0;
        for product in &catalog {
            assert!(product.price_cents > 0);
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn test_shopping_cart_iterators_and_into_iterator() {
        let mut cart = ShoppingCart::new();
        cart.add_item(10, 2, 5000); // $100.00
        cart.add_item(20, 1, 3000); // $30.00

        // Test has_product via .any()
        assert!(cart.has_product(10));
        assert!(!cart.has_product(99));

        // Test iter_mut via promotional discount (10% off)
        cart.apply_promotional_discount(10);
        assert_eq!(cart.get_item(10).unwrap().unit_price_cents, 4500); // $50 - 10% = $45
        assert_eq!(cart.get_item(20).unwrap().unit_price_cents, 2700); // $30 - 10% = $27

        // Test IntoIterator for &ShoppingCart
        let total_items: u32 = (&cart).into_iter().map(|item| item.quantity).sum();
        assert_eq!(total_items, 3);

        // Test IntoIterator for ShoppingCart (consuming)
        let items_vec: Vec<CartItem> = cart.into_iter().collect();
        assert_eq!(items_vec.len(), 2);
    }

    #[test]
    fn test_order_iterator_methods() {
        let customer = Customer::new(
            1,
            String::from("Ada"),
            String::from("ada@test.com"),
            None,
            false,
        );
        let items = vec![CartItem::new(1, 3, 2000), CartItem::new(2, 2, 4000)];
        let order = Order::new(
            OrderId(10),
            customer,
            items,
            PaymentMethod::CashOnDelivery,
            None,
        );

        assert_eq!(order.total_quantity(), 5);

        // Using for loop directly on &order via IntoIterator
        let mut product_ids = Vec::new();
        for item in &order {
            product_ids.push(item.product_id);
        }
        assert_eq!(product_ids, vec![1, 2]);
    }

    #[test]
    fn test_custom_cart_report_iterator() {
        let mut cart = ShoppingCart::new();
        cart.add_item(101, 2, 1500);
        cart.add_item(102, 1, 4000);

        let reports: Vec<String> = cart.report_iter().collect();
        assert_eq!(reports.len(), 2);
        assert_eq!(reports[0], "Item #1: Product #101 (Qty: 2) - $30.00");
        assert_eq!(reports[1], "Item #2: Product #102 (Qty: 1) - $40.00");
    }

    #[test]
    fn test_custom_discount_tier_iter() {
        let tiers = DiscountTierIter::new(5, 20);
        let tier_list: Vec<u32> = tiers.collect();
        assert_eq!(tier_list, vec![5, 10, 15, 20]);

        // Testing standard iterator functional adapters on our custom iterator
        let sum_evens: u32 = DiscountTierIter::new(5, 25)
            .filter(|&rate| rate % 10 == 0) // keeps 10, 20
            .map(|rate| rate * 2) // 20, 40
            .sum(); // 60
        assert_eq!(sum_evens, 60);
    }

    #[test]
    fn test_closure_dynamic_filtering_in_catalog() {
        let mut catalog = Catalog::new();
        catalog.add_product(Product::new(
            1,
            String::from("TECH-1"),
            String::from("RGB Keyboard"),
            ProductCategory::Electronics,
            12000,
            5,
        ));
        catalog.add_product(Product::new(
            2,
            String::from("FURN-1"),
            String::from("Standing Desk"),
            ProductCategory::Furniture,
            35000,
            2,
        ));
        catalog.add_product(Product::new(
            3,
            String::from("TECH-2"),
            String::from("Wireless Mouse"),
            ProductCategory::Electronics,
            4500,
            10,
        ));

        // Closure capturing an environment variable
        let max_budget = 15000;
        let affordable = catalog.find_products(|p| p.price_cents <= max_budget);
        assert_eq!(affordable.len(), 2);

        // Closure checking category and stock
        let low_stock_furniture =
            catalog.find_products(|p| p.category == ProductCategory::Furniture && p.stock <= 2);
        assert_eq!(low_stock_furniture.len(), 1);
        assert_eq!(low_stock_furniture[0].name, "Standing Desk");
    }

    #[test]
    fn test_closure_factories_with_move() {
        // Factory returning `impl Fn(u32) -> u32` with `move`
        let discount_15 = make_percentage_discount(15);
        assert_eq!(discount_15(10000), 1500); // 15% of $100 is $15
        assert_eq!(discount_15(20000), 3000); // 15% of $200 is $30

        // Threshold factory: $20 off for orders over $150
        let spend_150_save_20 = make_threshold_discount(15000, 2000);
        assert_eq!(spend_150_save_20(10000), 0); // Below threshold ($100)
        assert_eq!(spend_150_save_20(15000), 2000); // Exactly threshold ($150)
        assert_eq!(spend_150_save_20(25000), 2000); // Above threshold ($250)
    }

    #[test]
    fn test_vip_discount_closure() {
        let vip_rule = make_vip_discount(10); // 10% VIP discount

        let regular_customer = Customer::new(
            1,
            String::from("Bob"),
            String::from("bob@test.com"),
            None,
            false,
        );
        let vip_customer = Customer::new(
            2,
            String::from("Alice"),
            String::from("alice@test.com"),
            None,
            true,
        );

        assert_eq!(vip_rule(&regular_customer, 10000), 0);
        assert_eq!(vip_rule(&vip_customer, 10000), 1000);
    }

    #[test]
    fn test_fn_mut_stateful_cart_discount() {
        let mut cart = ShoppingCart::new();
        cart.add_item(1, 2, 5000); // $50 each
        cart.add_item(2, 1, 8000); // $80 each

        // FnMut closure that tracks how much total discount was applied
        let mut total_discount_recorded = 0;
        let mut discount_steps = 0;

        cart.apply_custom_discount(|item| {
            let discount = if item.quantity > 1 { 1000 } else { 500 };
            total_discount_recorded += discount;
            discount_steps += 1;
            discount
        });

        assert_eq!(discount_steps, 2);
        assert_eq!(total_discount_recorded, 1500); // 1000 + 500
        assert_eq!(cart.get_item(1).unwrap().unit_price_cents, 4000); // 5000 - 1000
        assert_eq!(cart.get_item(2).unwrap().unit_price_cents, 7500); // 8000 - 500
    }

    #[test]
    fn test_evaluate_promotions_and_auditor() {
        let order_subtotal = 20000; // $200.00

        let rule1 = make_percentage_discount(10); // 10% = $20.00
        let rule2 = make_percentage_discount(5); // 5%  = $10.00
        let rules = [rule1, rule2];

        // Total combined discount
        let total_discount = calculate_total_promotions(order_subtotal, &rules);
        assert_eq!(total_discount, 3000); // $20 + $10 = $30

        // Best promotion (single best deal)
        let best_deal = find_best_promotion(order_subtotal, &rules);
        assert_eq!(best_deal, 2000); // $20 is better than $10

        // Test DiscountAuditor
        let mut auditor = DiscountAuditor::new();
        let mut record_closure = |amt| auditor.record(amt);

        record_closure(1500);
        record_closure(2500);

        assert_eq!(auditor.operations_count, 2);
        assert_eq!(auditor.total_discount_given, 4000);
    }

    #[test]
    fn test_boxed_dynamic_dispatch_promotion_pipeline() {
        let mut pipeline = PromotionPipeline::new();
        assert!(pipeline.is_empty());

        // Heterogeneous closures: Percentage AND Threshold closures stored in the same collection!
        // In Chapter 18, putting both into a Vec<F> failed because they had different anonymous types.
        pipeline.add_rule(make_percentage_discount(10)); // 10% discount
        pipeline.add_rule(make_threshold_discount(15000, 2000)); // $20 off orders >= $150
        assert_eq!(pipeline.len(), 2);

        // Subtotal $200.00 (20,000 cents):
        // Rule 1: 10% of 20000 = 2000 cents ($20)
        // Rule 2: 20000 >= 15000 = 2000 cents ($20)
        assert_eq!(pipeline.apply_all(20000), 4000); // $40 total
        assert_eq!(pipeline.apply_best(20000), 2000); // $20 best

        // Subtotal $100.00 (10,000 cents):
        // Rule 1: 10% of 10000 = 1000 cents ($10)
        // Rule 2: 10000 < 15000 = 0 cents ($0)
        assert_eq!(pipeline.apply_all(10000), 1000);
        assert_eq!(pipeline.apply_best(10000), 1000);
    }

    #[test]
    fn test_recursive_category_tree_with_box() {
        let mut root = CategoryNode::new("Electronics");

        let mut computers = CategoryNode::new("Computers");
        computers.add_subcategory(CategoryNode::new("Laptops"));
        computers.add_subcategory(CategoryNode::new("Desktops"));

        let mut accessories = CategoryNode::new("Accessories");
        accessories.add_subcategory(CategoryNode::new("Keyboards"));

        root.add_subcategory(computers);
        root.add_subcategory(accessories);

        // Structure:
        // Electronics (1)
        //  ├── Computers (2)
        //  │    ├── Laptops (3)
        //  │    └── Desktops (4)
        //  └── Accessories (5)
        //       └── Keyboards (6)
        assert_eq!(root.total_categories(), 6);
        assert_eq!(root.depth(), 3);

        assert!(root.contains("laptops"));
        assert!(root.contains("KEYBOARDS"));
        assert!(root.contains("Electronics"));
        assert!(!root.contains("Groceries"));
    }

    #[test]
    fn test_rc_multiple_ownership_customer() {
        use std::rc::Rc;

        let customer = Rc::new(Customer::new(
            501,
            String::from("Katherine Johnson"),
            String::from("katherine@nasa.gov"),
            Some(String::from("+1-555-4321")),
            true,
        ));

        // Initial reference count: 1 owner
        assert_eq!(Rc::strong_count(&customer), 1);

        // Shared ownership: Cloning the Rc only copies the pointer, not the customer data
        let checkout_ref = Rc::clone(&customer);
        let audit_ref = Rc::clone(&customer);

        assert_eq!(Rc::strong_count(&customer), 3);
        assert_eq!(checkout_ref.name, "Katherine Johnson");
        assert_eq!(audit_ref.name, "Katherine Johnson");
        assert!(checkout_ref.is_vip);

        // Dropping one handle decrements the strong count
        drop(audit_ref);
        assert_eq!(Rc::strong_count(&customer), 2);
    }

    #[test]
    fn test_refcell_interior_mutability_shared_auditor() {
        let auditor = SharedAuditor::new();
        let pipeline_handle = auditor.clone();

        assert_eq!(auditor.strong_count(), 2);
        assert_eq!(auditor.operations_count(), 0);
        assert_eq!(auditor.total_discount_given(), 0);

        // Record through the pipeline handle using immutable &self
        pipeline_handle.record(1500);

        // Both handles observe the change because they share the underlying RefCell<DiscountAuditor>
        assert_eq!(auditor.operations_count(), 1);
        assert_eq!(auditor.total_discount_given(), 1500);
        assert_eq!(pipeline_handle.total_discount_given(), 1500);

        // Record through original handle
        auditor.record(2500);
        assert_eq!(pipeline_handle.operations_count(), 2);
        assert_eq!(pipeline_handle.total_discount_given(), 4000);
    }

    #[test]
    fn test_custom_smart_pointer_deref_and_drop() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let drop_flag = Rc::new(RefCell::new(false));

        {
            let customer = Customer::new(
                502,
                String::from("Ada Lovelace"),
                String::from("ada@analytical.engine"),
                None,
                false,
            );

            // Create StoreSession with active drop flag tracker
            let mut session =
                StoreSession::with_drop_flag("SESS-ADA-001", customer, Rc::clone(&drop_flag));

            assert!(*drop_flag.borrow()); // Session is active

            // Deref coercion: call Customer methods directly on StoreSession!
            assert_eq!(session.display_badge(), "[Standard Member] Ada Lovelace");
            assert_eq!(session.name, "Ada Lovelace");

            // DerefMut: mutate customer through the session wrapper
            session.upgrade_to_vip();
            assert!(session.is_vip);
            assert_eq!(session.display_badge(), "[VIP Member] Ada Lovelace");

            // Session is about to go out of scope here
        }

        // When session dropped, Drop::drop executed and reset the flag!
        assert!(!*drop_flag.borrow());
    }

    #[test]
    fn test_thread_spawn_and_join() {
        use std::thread;

        // Spawning an OS thread with move closure and returning a calculated result
        let item_price = 4500;
        let quantity = 3;

        let handle = thread::spawn(move || {
            let line_total = item_price * quantity;
            line_total + 500 // Adding shipping cost in thread
        });

        // Joining the thread waits for execution to complete and yields Result<T, Err>
        let result = handle.join().expect("Thread panicked");
        assert_eq!(result, 14000);
    }

    #[test]
    fn test_mpsc_channel_message_passing() {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();

        // Spawn a background producer thread
        thread::spawn(move || {
            let notification = OrderNotification::OrderPlaced {
                order_id: 1001,
                amount_cents: 25000,
            };
            tx.send(notification).expect("Failed to send notification");
        });

        // Main thread acts as the consumer
        let received = rx.recv().expect("Failed to receive message");
        assert_eq!(
            received,
            OrderNotification::OrderPlaced {
                order_id: 1001,
                amount_cents: 25000,
            }
        );
    }

    #[test]
    fn test_mpsc_multiple_producers() {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();

        // Multiple producers by cloning `tx`
        let tx1 = tx.clone();
        let tx2 = tx.clone();
        drop(tx); // Drop the original sender so the channel closes when tx1 and tx2 finish

        let h1 = thread::spawn(move || {
            tx1.send(OrderNotification::OrderStatusUpdated {
                order_id: 201,
                status: String::from("Confirmed"),
            })
            .unwrap();
        });

        let h2 = thread::spawn(move || {
            tx2.send(OrderNotification::OrderStatusUpdated {
                order_id: 202,
                status: String::from("Shipped"),
            })
            .unwrap();
        });

        h1.join().unwrap();
        h2.join().unwrap();

        // Collect all messages until channel closes
        let mut messages = Vec::new();
        while let Ok(msg) = rx.recv() {
            messages.push(msg);
        }

        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_arc_mutex_concurrent_sales_tracker() {
        let tracker = ConcurrentSalesTracker::new();
        assert_eq!(tracker.total_revenue(), 0);
        assert_eq!(tracker.transactions_count(), 0);

        let mut handles = Vec::new();

        // Spawn 8 concurrent checkout threads
        for _ in 0..8 {
            let tracker_clone = tracker.clone();
            let handle = std::thread::spawn(move || {
                // Each thread records 5 sales of $10.00 (1000 cents) each
                for _ in 0..5 {
                    tracker_clone.record_sale(1000);
                }
            });
            handles.push(handle);
        }

        // Wait for all 8 threads to complete
        for handle in handles {
            handle.join().expect("Worker thread panicked");
        }

        // Total sales: 8 threads * 5 transactions = 40 transactions
        // Total revenue: 40 * 1000 = 40,000 cents ($400.00)
        assert_eq!(tracker.transactions_count(), 40);
        assert_eq!(tracker.total_revenue(), 40000);
        // All worker clones have dropped, so strong count is back to 1
        assert_eq!(tracker.strong_count(), 1);
    }

    #[test]
    fn test_parallel_batch_valuation() {
        let inventory = vec![
            (12000, 5), // $120 * 5 = $600 (60,000 cents)
            (4500, 10), // $45 * 10 = $450 (45,000 cents)
            (35000, 3), // $350 * 3 = $1050 (105,000 cents)
            (2500, 20), // $25 * 20 = $500 (50,000 cents)
        ];

        // Total expected = 60,000 + 45,000 + 105,000 + 50,000 = 260,000 cents ($2,600.00)
        let total = parallel_batch_valuation(inventory, 2);
        assert_eq!(total, 260000);

        // Empty inventory check
        assert_eq!(parallel_batch_valuation(vec![], 4), 0);
    }

    #[tokio::test]
    async fn test_async_payment_processing() {
        let payment_client = MockPaymentClient::new(10); // 10ms simulated latency
        let card = PaymentMethod::CreditCard {
            last_four: String::from("4321"),
        };

        let tx_id = payment_client
            .process_payment(&card, 15000)
            .await
            .expect("Payment processing failed");

        assert_eq!(tx_id, "TX-CARD-4321-15000");

        // Zero-amount payment error validation
        let err = payment_client.process_payment(&card, 0).await.unwrap_err();
        assert!(matches!(err, StoreError::InvalidStateTransition { .. }));
    }

    #[tokio::test]
    async fn test_async_warehouse_single_stock() {
        let warehouse = WarehouseClient::new("East Warehouse", 5);
        let stock = warehouse.check_stock("TECH-KEY-001").await.unwrap();
        assert_eq!(stock, 15);

        let unknown_stock = warehouse.check_stock("UNKNOWN-SKU").await.unwrap();
        assert_eq!(unknown_stock, 0);
    }

    #[tokio::test]
    async fn test_async_warehouse_stock_aggregation_join() {
        let east = WarehouseClient::new("East Warehouse", 10);
        let west = WarehouseClient::new("West Warehouse", 10);

        // Uses tokio::join! to execute both futures concurrently
        let total_stock = aggregate_warehouse_stock(&east, &west, "TECH-KEY-001")
            .await
            .unwrap();

        // 15 from east + 15 from west = 30
        assert_eq!(total_stock, 30);
    }

    #[tokio::test]
    async fn test_async_notification_bus_mpsc() {
        let (bus, mut rx) = AsyncNotificationBus::new(16);
        let bus_clone = bus.clone_sender();

        // Spawn a background Tokio task (green thread)
        let task = tokio::spawn(async move {
            bus_clone
                .send(OrderNotification::AuditAlert {
                    message: String::from("Fraud detection clearance verified"),
                })
                .await
                .unwrap();
        });

        task.await.unwrap();

        let received = rx.recv().await.expect("Channel closed");
        assert_eq!(
            received,
            OrderNotification::AuditAlert {
                message: String::from("Fraud detection clearance verified"),
            }
        );
    }

    #[tokio::test]
    async fn test_async_checkout_workflow() {
        let mut catalog = Catalog::new();
        let keyboard = Product::new(
            101,
            String::from("TECH-KEY-001"),
            String::from("Tenkeyless Mechanical Keyboard"),
            ProductCategory::Electronics,
            12000,
            10,
        );
        catalog.add_product(keyboard);

        let mut cart = ShoppingCart::new();
        cart.add_item(101, 2, 12000);

        let customer = Customer::new(
            601,
            String::from("Grace Hopper"),
            String::from("grace@navy.mil"),
            None,
            false,
        );

        let payment_client = MockPaymentClient::new(5);
        let payment_method = PaymentMethod::CreditCard {
            last_four: String::from("1944"),
        };

        let (order, tx_receipt) = async_checkout(
            OrderId(701),
            customer,
            &mut cart,
            &mut catalog,
            payment_method,
            None,
            &payment_client,
        )
        .await
        .expect("Async checkout failed");

        assert_eq!(order.order_id, OrderId(701));
        assert_eq!(order.total_quantity(), 2);
        assert_eq!(tx_receipt, format!("TX-CARD-1944-{}", order.total_cents()));
        assert_eq!(catalog.find_by_id(101).unwrap().stock, 8); // 10 - 2
    }

    #[test]
    fn test_product_and_category_json_roundtrip() {
        let product = Product::new(
            50,
            String::from("BOOK-RUST-01"),
            String::from("Rust by Building MiniStore"),
            ProductCategory::OfficeSupplies,
            2999,
            50,
        );

        let json = serde_json::to_string(&product).expect("Failed to serialize product");
        let deserialized: Product =
            serde_json::from_str(&json).expect("Failed to deserialize product");

        assert_eq!(product, deserialized);
        assert_eq!(deserialized.sku, "BOOK-RUST-01");
        assert_eq!(deserialized.price_cents, 2999);
    }

    #[test]
    fn test_order_and_customer_json_roundtrip() {
        let mut customer = Customer::new(
            301,
            String::from("Alice Walker"),
            String::from("alice@example.com"),
            Some(String::from("+8801700000000")),
            true,
        );
        customer.add_tag("early-adopter");

        let items = vec![CartItem::new(10, 2, 1500), CartItem::new(20, 1, 3500)];
        let order = Order::new(
            OrderId(9001),
            customer.clone(),
            items,
            PaymentMethod::CreditCard {
                last_four: String::from("4242"),
            },
            Some(Coupon::new(String::from("RUST10"), 10)),
        );

        let json = serde_json::to_string_pretty(&order).expect("Failed to serialize order");
        let deserialized: Order = serde_json::from_str(&json).expect("Failed to deserialize order");

        assert_eq!(order, deserialized);
        assert_eq!(deserialized.order_id, OrderId(9001));
        assert_eq!(deserialized.customer.name, "Alice Walker");
        assert!(deserialized.customer.has_tag("early-adopter"));
        assert_eq!(deserialized.items.len(), 2);
    }

    #[test]
    fn test_store_persistence_catalog_file_io() {
        let test_dir =
            std::env::temp_dir().join(format!("ministore_test_cat_{}", std::process::id()));
        let persistence = StorePersistence::new(&test_dir);

        let mut catalog = Catalog::new();
        catalog.add_product(Product::new(
            1,
            String::from("ELEC-MON-01"),
            String::from("4K UHD Monitor"),
            ProductCategory::Electronics,
            34999,
            5,
        ));
        catalog.add_product(Product::new(
            2,
            String::from("FURN-DSK-01"),
            String::from("Standing Desk"),
            ProductCategory::Furniture,
            49999,
            3,
        ));

        // Save catalog
        let saved_path = persistence
            .save_catalog(&catalog)
            .expect("Failed to save catalog");
        assert!(saved_path.exists());

        // Load catalog into a fresh instance
        let loaded_catalog = persistence.load_catalog().expect("Failed to load catalog");
        assert_eq!(catalog, loaded_catalog);
        assert_eq!(loaded_catalog.total_products(), 2);
        assert_eq!(
            loaded_catalog.find_by_sku("ELEC-MON-01").unwrap().name,
            "4K UHD Monitor"
        );

        // Clean up
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_store_persistence_snapshot_export_import() {
        let test_dir =
            std::env::temp_dir().join(format!("ministore_test_snap_{}", std::process::id()));
        let persistence = StorePersistence::new(&test_dir);

        let mut catalog = Catalog::new();
        catalog.add_product(Product::new(
            10,
            String::from("TECH-MOU-01"),
            String::from("Wireless Mouse"),
            ProductCategory::Electronics,
            2500,
            12,
        ));

        let customer = Customer::new(
            50,
            String::from("Linus Torvalds"),
            String::from("linus@linux.org"),
            None,
            true,
        );

        let order = Order::new(
            OrderId(100),
            customer.clone(),
            vec![CartItem::new(10, 1, 2500)],
            PaymentMethod::CashOnDelivery,
            None,
        );

        let snapshot = StoreSnapshot::new(
            catalog,
            vec![customer],
            vec![order],
            String::from("2026-09-28T16:00:00Z"),
        );

        // Export snapshot
        let path = persistence
            .export_snapshot(&snapshot)
            .expect("Snapshot export failed");
        assert!(path.exists());

        // Import snapshot
        let imported = persistence
            .import_snapshot()
            .expect("Snapshot import failed");
        assert_eq!(snapshot, imported);
        assert_eq!(imported.timestamp, "2026-09-28T16:00:00Z");
        assert_eq!(imported.orders.len(), 1);

        // Clean up
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_store_persistence_errors() {
        let test_dir =
            std::env::temp_dir().join(format!("ministore_test_err_{}", std::process::id()));
        let persistence = StorePersistence::new(&test_dir);

        // 1. Missing file error
        let err = persistence.load_catalog().unwrap_err();
        match err {
            StoreError::IoError { path, message } => {
                assert!(path.contains("catalog.json"));
                assert!(!message.is_empty());
            }
            other => panic!("Expected IoError, got: {:?}", other),
        }

        // 2. Corrupt JSON error
        std::fs::create_dir_all(&test_dir).unwrap();
        std::fs::write(test_dir.join("catalog.json"), b"{ this is corrupt json! }").unwrap();

        let err2 = persistence.load_catalog().unwrap_err();
        match err2 {
            StoreError::SerializationError { message } => {
                assert!(!message.is_empty());
            }
            other => panic!("Expected SerializationError, got: {:?}", other),
        }

        // Clean up
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[tokio::test]
    async fn test_web_api_health_check() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        let state = AppState::new(Catalog::new(), Vec::new(), None);
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["status"], "healthy");
        assert_eq!(body["service"], "ministore-api");
    }

    #[tokio::test]
    async fn test_web_api_products_flow() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode, header};
        use tower::ServiceExt;

        let state = AppState::new(Catalog::new(), Vec::new(), None);
        let app = create_router(state);

        // 1. POST /api/products
        let new_product = CreateProductRequest {
            id: 201,
            sku: String::from("TECH-CAM-01"),
            name: String::from("4K Webcam"),
            category: ProductCategory::Electronics,
            price_cents: 8900,
            stock: 15,
        };
        let body_json = serde_json::to_string(&new_product).unwrap();

        let post_req = Request::builder()
            .method("POST")
            .uri("/api/products")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body_json))
            .unwrap();

        let post_res = app.clone().oneshot(post_req).await.unwrap();
        assert_eq!(post_res.status(), StatusCode::CREATED);

        // 2. GET /api/products/201 (Found)
        let get_req = Request::builder()
            .uri("/api/products/201")
            .body(Body::empty())
            .unwrap();
        let get_res = app.clone().oneshot(get_req).await.unwrap();
        assert_eq!(get_res.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(get_res.into_body(), usize::MAX)
            .await
            .unwrap();
        let product: Product = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(product.id, 201);
        assert_eq!(product.name, "4K Webcam");

        // 3. GET /api/products/999 (Not Found -> 404)
        let not_found_req = Request::builder()
            .uri("/api/products/999")
            .body(Body::empty())
            .unwrap();
        let not_found_res = app.oneshot(not_found_req).await.unwrap();
        assert_eq!(not_found_res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_web_api_checkout_flow() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode, header};
        use tower::ServiceExt;

        let mut catalog = Catalog::new();
        catalog.add_product(Product::new(
            10,
            String::from("BOOK-RUST-01"),
            String::from("Rust by Building MiniStore"),
            ProductCategory::OfficeSupplies,
            2500,
            10,
        ));

        let state = AppState::new(catalog, Vec::new(), None);
        let app = create_router(state);

        let customer = Customer::new(
            501,
            String::from("Ada Lovelace"),
            String::from("ada@first-programmer.org"),
            None,
            true,
        );

        // 1. Successful checkout
        let checkout_req = CheckoutRequest {
            order_id: 1001,
            customer: customer.clone(),
            items: vec![CheckoutItemRequest {
                product_id: 10,
                quantity: 2,
            }],
            payment: PaymentMethod::CashOnDelivery,
            coupon: None,
        };
        let body_json = serde_json::to_string(&checkout_req).unwrap();

        let req = Request::builder()
            .method("POST")
            .uri("/api/checkout")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body_json))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let order: Order = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(order.order_id, OrderId(1001));
        assert_eq!(order.total_quantity(), 2);

        // 2. Verify order is listed in GET /api/orders
        let list_orders_req = Request::builder()
            .uri("/api/orders")
            .body(Body::empty())
            .unwrap();
        let list_res = app.clone().oneshot(list_orders_req).await.unwrap();
        assert_eq!(list_res.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(list_res.into_body(), usize::MAX)
            .await
            .unwrap();
        let orders: Vec<Order> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].order_id, OrderId(1001));

        // 3. Failed checkout: Empty Cart -> 400 Bad Request
        let empty_checkout_req = CheckoutRequest {
            order_id: 1002,
            customer,
            items: Vec::new(),
            payment: PaymentMethod::CashOnDelivery,
            coupon: None,
        };
        let empty_req = Request::builder()
            .method("POST")
            .uri("/api/checkout")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_string(&empty_checkout_req).unwrap(),
            ))
            .unwrap();

        let empty_res = app.oneshot(empty_req).await.unwrap();
        assert_eq!(empty_res.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_app_config_defaults_and_env() {
        let config = AppConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
        assert_eq!(config.env, Environment::Development);
        assert!(!config.env.is_production());

        let addr = config
            .socket_addr()
            .expect("Failed to parse socket address");
        assert_eq!(addr.port(), 3000);

        let custom = AppConfig::new(
            "0.0.0.0",
            8080,
            "/var/ministore/data",
            "warn",
            Environment::Production,
        );
        assert!(custom.env.is_production());
        assert_eq!(custom.env.as_str(), "production");
        assert_eq!(custom.socket_addr().unwrap().port(), 8080);
    }

    #[tokio::test]
    async fn test_web_api_liveness_and_readiness_probes() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        let test_dir =
            std::env::temp_dir().join(format!("ministore_probe_test_{}", std::process::id()));
        let persistence = StorePersistence::new(&test_dir);
        let state = AppState::new(Catalog::new(), Vec::new(), Some(persistence));
        let app = create_router(state);

        // 1. Liveness Probe (GET /health/live)
        let live_req = Request::builder()
            .uri("/health/live")
            .body(Body::empty())
            .unwrap();
        let live_res = app.clone().oneshot(live_req).await.unwrap();
        assert_eq!(live_res.status(), StatusCode::OK);
        let live_bytes = axum::body::to_bytes(live_res.into_body(), usize::MAX)
            .await
            .unwrap();
        let live_json: serde_json::Value = serde_json::from_slice(&live_bytes).unwrap();
        assert_eq!(live_json["status"], "alive");

        // 2. Readiness Probe (GET /health/ready)
        let ready_req = Request::builder()
            .uri("/health/ready")
            .body(Body::empty())
            .unwrap();
        let ready_res = app.oneshot(ready_req).await.unwrap();
        assert_eq!(ready_res.status(), StatusCode::OK);
        let ready_bytes = axum::body::to_bytes(ready_res.into_body(), usize::MAX)
            .await
            .unwrap();
        let ready_json: serde_json::Value = serde_json::from_slice(&ready_bytes).unwrap();
        assert_eq!(ready_json["status"], "ready");
        assert_eq!(ready_json["storage"], "accessible");

        // Clean up
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_declarative_product_and_catalog_macros() {
        // 1. product! with category identifier shorthand and explicit stock
        let p1 = product!(1, "SKU-001", "Mechanical Keyboard", Electronics, 15000, 10);
        assert_eq!(p1.id, 1);
        assert_eq!(p1.sku, "SKU-001");
        assert_eq!(p1.name, "Mechanical Keyboard");
        assert_eq!(p1.category, ProductCategory::Electronics);
        assert_eq!(p1.price_cents, 15000);
        assert_eq!(p1.stock, 10);

        // 2. product! with default stock = 1
        let p2 = product!(2, "SKU-002", "Ballpoint Pen", OfficeSupplies, 150);
        assert_eq!(p2.stock, 1);
        assert_eq!(p2.category, ProductCategory::OfficeSupplies);

        // 3. product! with named arguments
        let p3 = product!(
            id: 3,
            sku: "SKU-003",
            name: "Standing Desk",
            category: Furniture,
            price: 45000,
            stock: 3
        );
        assert_eq!(p3.category, ProductCategory::Furniture);
        assert_eq!(p3.price_cents, 45000);

        // 4. catalog! macro with trailing comma
        let cat = catalog![
            p1,
            p2,
            p3,
            product!(4, "SKU-004", "Mousepad", OfficeSupplies, 1200, 20),
        ];
        assert_eq!(cat.total_products(), 4);
        assert!(cat.find_by_sku("SKU-001").is_some());
        assert_eq!(cat.product_price("SKU-004"), Some(1200));
    }

    #[test]
    fn test_declarative_cart_and_variadic_calculation_macros() {
        // 1. cart! using tuple syntax
        let cart_tuples = cart![(1, 2, 15000), (2, 5, 150),];
        assert_eq!(cart_tuples.items.len(), 2);
        assert_eq!(cart_tuples.items[0].product_id, 1);
        assert_eq!(cart_tuples.items[0].quantity, 2);
        assert_eq!(cart_tuples.items[1].unit_price_cents, 150);

        // 2. cart! using named syntax
        let cart_named = cart![
            (item: 10, qty: 1, price: 9900),
            (item: 20, qty: 3, price: 2500),
        ];
        assert_eq!(cart_named.items.len(), 2);
        assert_eq!(cart_named.items[0].product_id, 10);
        assert_eq!(cart_named.items[1].quantity, 3);

        // 3. calculate_total! variadic recursive expansion
        assert_eq!(calculate_total!(), 0);
        assert_eq!(calculate_total!(500), 500);
        assert_eq!(calculate_total!(100, 200, 300), 600);
        assert_eq!(calculate_total!(15000, 750, 9900, 100), 25750);

        // 4. assert_in_stock! macro
        let product = product!(1, "LAP-001", "Laptop", Electronics, 120000, 8);
        assert_in_stock!(product, 5);
        assert_in_stock!(product, 8);
    }

    #[test]
    fn test_unsafe_raw_barcode_buffer_safe_abstraction() {
        // Allocate buffer with capacity 4 (will test growing)
        let mut buffer = RawBarcodeBuffer::with_capacity(4).unwrap();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.capacity(), 4);

        // Push bytes within initial capacity
        buffer.push_str("SKU-").unwrap();
        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.as_str().unwrap(), "SKU-");

        // Push more bytes to trigger growth / reallocation via raw realloc
        buffer.push_str("TECH-999").unwrap();
        assert_eq!(buffer.len(), 12);
        assert!(buffer.capacity() >= 12);
        assert_eq!(buffer.as_str().unwrap(), "SKU-TECH-999");

        // Clear buffer
        buffer.clear();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());

        // Re-use cleared buffer
        buffer.push_str("BARCODE-12345").unwrap();
        assert_eq!(buffer.as_str().unwrap(), "BARCODE-12345");
        assert_eq!(buffer.as_slice(), b"BARCODE-12345");
        // buffer drops cleanly here, exercising manual dealloc
    }

    #[test]
    fn test_unsafe_raw_pointer_swap_and_ffi() {
        // 1. Safe pointer swapping
        let mut price_a = 15000u32;
        let mut price_b = 4500u32;
        safe_swap_prices(&mut price_a, &mut price_b);
        assert_eq!(price_a, 4500);
        assert_eq!(price_b, 15000);

        // 2. Direct unsafe raw pointer dereferencing
        unsafe {
            unsafe_raw_swap(&mut price_a as *mut u32, &mut price_b as *mut u32);
        }
        assert_eq!(price_a, 15000);
        assert_eq!(price_b, 4500);

        // 3. Foreign Function Interface (FFI) - calling C standard library abs
        let diff = c_abi_price_diff(12000, 15000);
        assert_eq!(diff, 3000);
        let diff_reverse = c_abi_price_diff(15000, 12000);
        assert_eq!(diff_reverse, 3000);

        // 4. Exported C-compatible ABI function
        let tax = ministore_c_calculate_tax(20000, 15);
        assert_eq!(tax, 3000);
    }
}
