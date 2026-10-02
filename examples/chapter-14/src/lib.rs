//! MiniStore Core Library
//!
//! Provides the core e-commerce domain models, catalog lookups,
//! transactional checkout, and error handling for the MiniStore application.

pub mod catalog;
pub mod checkout;
pub mod error;
pub mod models;

// Convenient top-level re-exports (facade)
pub use catalog::Catalog;
pub use checkout::{
    calculate_discount, checkout, count_products_by_department, order_dispatch_advisory,
};
pub use error::StoreError;
pub use models::{
    paginate, ApiResponse, CartItem, Coupon, Customer, Order, OrderId, OrderStatus, Page,
    PaymentMethod, Product, ProductCategory, ShoppingCart,
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
}
