//! MiniStore Core Library
//!
//! Provides the core e-commerce domain models, catalog lookups,
//! transactional checkout, and error handling for the MiniStore application.

pub mod catalog;
pub mod checkout;
pub mod error;
pub mod models;
pub mod promotions;
pub mod traits;

// Convenient top-level re-exports (facade)
pub use catalog::Catalog;
pub use checkout::{
    calculate_discount, checkout, count_products_by_department, order_dispatch_advisory,
};
pub use error::StoreError;
pub use models::{
    best_contact_info, find_higher_priced, paginate, store_policy, ApiResponse, CartItem,
    CartReportIterator, Coupon, Customer, DiscountTierIter, Order, OrderId, OrderReceipt,
    OrderStatus, Page, PaymentMethod, Product, ProductCategory, ShoppingCart,
};
pub use promotions::{
    calculate_total_promotions, find_best_promotion, make_percentage_discount,
    make_threshold_discount, make_vip_discount, DiscountAuditor,
};
pub use traits::{format_tax_summary, summarize_item, Summarizable, Taxable};

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
}
