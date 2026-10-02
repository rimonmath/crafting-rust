use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum ProductCategory {
    Electronics,
    OfficeSupplies,
    Furniture,
    Custom(String),
}

impl ProductCategory {
    pub fn default_tax_rate(&self) -> u32 {
        match self {
            Self::Electronics => 15,
            Self::OfficeSupplies => 5,
            Self::Furniture => 10,
            Self::Custom(_) => 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub sku: String,
    pub name: String,
    pub category: ProductCategory,
    pub price_cents: u32,
    pub stock: u32,
}

impl Product {
    pub fn new(
        id: u64,
        sku: String,
        name: String,
        category: ProductCategory,
        price_cents: u32,
        stock: u32,
    ) -> Self {
        Self {
            id,
            sku,
            name,
            category,
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

    /// Returns a string slice (`&str`) borrowing the department code from `self.sku`.
    pub fn department_code(&self) -> &str {
        match self.sku.find('-') {
            Some(idx) => &self.sku[..idx],
            None => &self.sku[..],
        }
    }

    /// Returns a slice of the product name truncated to `max_bytes` safely.
    pub fn truncated_name(&self, max_bytes: usize) -> &str {
        if max_bytes >= self.name.len() {
            &self.name[..]
        } else {
            let mut end = max_bytes;
            while end > 0 && !self.name.is_char_boundary(end) {
                end -= 1;
            }
            &self.name[..end]
        }
    }

    /// Checks if the product SKU begins with the requested prefix slice.
    pub fn matches_sku_prefix(&self, prefix: &str) -> bool {
        self.sku.starts_with(prefix)
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
    pub tags: HashSet<String>,
}

impl Customer {
    pub fn new(id: u64, name: String, email: String, is_vip: bool) -> Self {
        Self {
            id,
            name,
            email,
            is_vip,
            tags: HashSet::new(),
        }
    }

    pub fn add_tag(&mut self, tag: &str) -> bool {
        self.tags.insert(tag.to_string())
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    pub fn display_badge(&self) -> String {
        if self.is_vip {
            format!("[VIP Member] {}", self.name)
        } else {
            format!("[Standard Member] {}", self.name)
        }
    }

    pub fn upgrade_to_vip(&mut self) {
        self.is_vip = true;
        self.tags.insert(String::from("vip"));
    }

    pub fn update_email(&mut self, new_email: String) {
        self.email = new_email;
    }
}

/// OrderId implements `Copy`: 8-byte scalar on the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(pub u64);

/// Represents payment instruments accepted by MiniStore.
#[derive(Debug, Clone, PartialEq)]
pub enum PaymentMethod {
    CreditCard { last_four: String },
    BankTransfer { reference: String },
    CashOnDelivery,
}

impl PaymentMethod {
    /// Returns transaction or handling fee in cents based on payment method.
    pub fn fee_cents(&self) -> u32 {
        match self {
            Self::CreditCard { .. } => 150, // $1.50 processing fee
            Self::BankTransfer { .. } => 0, // Free
            Self::CashOnDelivery => 300,    // $3.00 handling fee
        }
    }

    /// User-friendly description of payment channel.
    pub fn description(&self) -> String {
        match self {
            Self::CreditCard { last_four } => format!("Credit Card (ending in {last_four})"),
            Self::BankTransfer { reference } => format!("Bank Transfer (Ref: {reference})"),
            Self::CashOnDelivery => String::from("Cash on Delivery"),
        }
    }
}

/// Explicit lifecycle states for an Order. Enums prevent impossible states!
#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Confirmed { receipt_id: String },
    Shipped { tracking_number: String },
    Delivered,
    Cancelled { reason: String },
}

impl OrderStatus {
    /// Formats state for display using pattern matching.
    pub fn display_status(&self) -> String {
        match self {
            Self::Pending => String::from("Awaiting Confirmation"),
            Self::Confirmed { receipt_id } => format!("Confirmed (Receipt: {receipt_id})"),
            Self::Shipped { tracking_number } => format!("Shipped (Tracking: {tracking_number})"),
            Self::Delivered => String::from("Delivered to Customer"),
            Self::Cancelled { reason } => format!("Cancelled (Reason: {reason})"),
        }
    }

    /// Orders can only be cancelled while still Pending or Confirmed.
    pub fn can_cancel(&self) -> bool {
        match self {
            Self::Pending | Self::Confirmed { .. } => true,
            Self::Shipped { .. } | Self::Delivered | Self::Cancelled { .. } => false,
        }
    }

    /// Checks if order is in a final, immutable terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Delivered | Self::Cancelled { .. })
    }
}

/// Represents a single line item in a shopping cart or order.
#[derive(Debug, Clone, PartialEq)]
pub struct CartItem {
    pub product_id: u64,
    pub quantity: u32,
    pub unit_price_cents: u32,
}

impl CartItem {
    pub fn new(product_id: u64, quantity: u32, unit_price_cents: u32) -> Self {
        Self {
            product_id,
            quantity,
            unit_price_cents,
        }
    }

    pub fn line_total(&self) -> u32 {
        self.unit_price_cents * self.quantity
    }
}

/// A dynamic shopping cart backed by `Vec<CartItem>`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShoppingCart {
    pub items: Vec<CartItem>,
}

impl ShoppingCart {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(&mut self, product_id: u64, quantity: u32, unit_price_cents: u32) {
        for item in &mut self.items {
            if item.product_id == product_id {
                item.quantity += quantity;
                return;
            }
        }
        self.items
            .push(CartItem::new(product_id, quantity, unit_price_cents));
    }

    pub fn remove_item(&mut self, product_id: u64) -> bool {
        if let Some(pos) = self
            .items
            .iter()
            .position(|item| item.product_id == product_id)
        {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn total_units(&self) -> u32 {
        self.items.iter().map(|item| item.quantity).sum()
    }

    pub fn subtotal_cents(&self) -> u32 {
        self.items.iter().map(|item| item.line_total()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Full Order domain model with state-machine lifecycle enforcement via Enums.
#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub order_id: OrderId,
    pub customer: Customer,
    pub items: Vec<CartItem>,
    pub payment: PaymentMethod,
    pub status: OrderStatus,
}

impl Order {
    pub fn new(
        order_id: OrderId,
        customer: Customer,
        items: Vec<CartItem>,
        payment: PaymentMethod,
    ) -> Self {
        Self {
            order_id,
            customer,
            items,
            payment,
            status: OrderStatus::Pending,
        }
    }

    /// Transitions Pending -> Confirmed { receipt_id }.
    pub fn confirm(&mut self, receipt_id: String) -> Result<(), &'static str> {
        match &self.status {
            OrderStatus::Pending => {
                self.status = OrderStatus::Confirmed { receipt_id };
                Ok(())
            }
            OrderStatus::Confirmed { .. } => Err("Order is already confirmed"),
            OrderStatus::Shipped { .. } => Err("Cannot confirm an order that is already shipped"),
            OrderStatus::Delivered => Err("Cannot confirm a delivered order"),
            OrderStatus::Cancelled { .. } => Err("Cannot confirm a cancelled order"),
        }
    }

    /// Transitions Confirmed -> Shipped { tracking_number }.
    pub fn ship(&mut self, tracking_number: String) -> Result<(), &'static str> {
        match &self.status {
            OrderStatus::Confirmed { .. } => {
                self.status = OrderStatus::Shipped { tracking_number };
                Ok(())
            }
            OrderStatus::Pending => Err("Order must be confirmed before shipping"),
            OrderStatus::Shipped { .. } => Err("Order is already shipped"),
            OrderStatus::Delivered => Err("Order is already delivered"),
            OrderStatus::Cancelled { .. } => Err("Cannot ship a cancelled order"),
        }
    }

    /// Transitions to Delivered.
    pub fn mark_delivered(&mut self) -> Result<(), &'static str> {
        match &self.status {
            OrderStatus::Shipped { .. } => {
                self.status = OrderStatus::Delivered;
                Ok(())
            }
            OrderStatus::Pending | OrderStatus::Confirmed { .. } => {
                Err("Order must be shipped before delivery")
            }
            OrderStatus::Delivered => Err("Order is already marked delivered"),
            OrderStatus::Cancelled { .. } => Err("Cannot deliver a cancelled order"),
        }
    }

    /// Cancels order if current state permits.
    pub fn cancel(&mut self, reason: String) -> Result<(), &'static str> {
        if self.status.can_cancel() {
            self.status = OrderStatus::Cancelled { reason };
            Ok(())
        } else {
            Err("Order cannot be cancelled in its current state")
        }
    }

    pub fn subtotal_cents(&self) -> u32 {
        self.items.iter().map(|item| item.line_total()).sum()
    }

    pub fn total_cents(&self) -> u32 {
        let subtotal = self.subtotal_cents();
        let discount = calculate_discount(&self.customer, subtotal);
        subtotal - discount + self.payment.fee_cents()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculates discount based on customer VIP status or tags.
pub fn calculate_discount(customer: &Customer, subtotal_cents: u32) -> u32 {
    if customer.is_vip || customer.has_tag("vip") {
        (subtotal_cents * 10) / 100 // 10% discount for VIPs
    } else {
        0
    }
}

/// Groups products by department and counts them using `HashMap` and the Entry API.
pub fn count_products_by_department(products: &[Product]) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for product in products {
        let dept = product.department_code().to_string();
        *counts.entry(dept).or_insert(0) += 1;
    }
    counts
}

/// Evaluates dispatch message based on OrderStatus using exhaustive pattern matching.
pub fn order_dispatch_advisory(status: &OrderStatus) -> &'static str {
    match status {
        OrderStatus::Pending => "Hold in warehouse: waiting for customer payment.",
        OrderStatus::Confirmed { .. } => "Ready to pick and pack at fulfillment center.",
        OrderStatus::Shipped { .. } => "In transit with logistics courier.",
        OrderStatus::Delivered => "Package successfully delivered to recipient.",
        OrderStatus::Cancelled { .. } => "Halted: restock inventory items immediately.",
    }
}

fn main() {
    println!("=== MiniStore: Enums & Pattern Matching (Part II) ===\n");

    // 1. Enums with Data Payloads (Tagged Unions)
    println!("1. Modeling Payments with Data-Carrying Enums:");
    let card_payment = PaymentMethod::CreditCard {
        last_four: String::from("4242"),
    };
    let cod_payment = PaymentMethod::CashOnDelivery;

    println!(
        "   Method: {} (Fee: ${:.2})",
        card_payment.description(),
        card_payment.fee_cents() as f64 / 100.0
    );
    println!(
        "   Method: {} (Fee: ${:.2})",
        cod_payment.description(),
        cod_payment.fee_cents() as f64 / 100.0
    );

    // 2. Products with Category Enums
    let keyboard = Product::new(
        101,
        String::from("TECH-KEY-001"),
        String::from("Tenkeyless Mechanical Keyboard"),
        ProductCategory::Electronics,
        12000,
        15,
    );
    println!("\n2. Product Category Classification:");
    println!(
        "   Product: {} | Category: {:?} | Tax Rate: {}%",
        keyboard.name,
        keyboard.category,
        keyboard.category.default_tax_rate()
    );

    // 3. Order Lifecycle State Machine
    println!("\n3. Order State Machine & Transitions:");
    let mut customer = Customer::new(
        301,
        String::from("Margaret Hamilton"),
        String::from("margaret@apollo.nasa.gov"),
        true,
    );
    customer.add_tag("pioneer");

    let mut cart = ShoppingCart::new();
    cart.add_item(keyboard.id, 2, keyboard.price_cents);

    let mut order = Order::new(
        OrderId(901),
        customer,
        cart.items,
        PaymentMethod::CreditCard {
            last_four: String::from("9876"),
        },
    );

    println!("   Initial Status: {}", order.status.display_status());
    println!(
        "   Advisory:       {}",
        order_dispatch_advisory(&order.status)
    );
    println!("   Can Cancel?     {}", order.status.can_cancel());

    // Transition 1: Confirm order
    order.confirm(String::from("REC-901- Hamilton")).unwrap();
    println!("\n   After Confirm:  {}", order.status.display_status());
    println!(
        "   Advisory:       {}",
        order_dispatch_advisory(&order.status)
    );

    // Transition 2: Ship order
    order.ship(String::from("TRK-USPS-774921")).unwrap();
    println!("\n   After Shipping: {}", order.status.display_status());
    println!(
        "   Advisory:       {}",
        order_dispatch_advisory(&order.status)
    );
    println!("   Can Cancel?     {}", order.status.can_cancel());

    // Attempting invalid transition: cannot cancel once shipped!
    let cancel_result = order.cancel(String::from("Customer changed mind"));
    println!(
        "   Attempt Cancel: Failed as expected -> {:?}",
        cancel_result.unwrap_err()
    );

    // Transition 3: Mark delivered
    order.mark_delivered().unwrap();
    println!("\n   After Delivery: {}", order.status.display_status());
    println!("   Is Terminal?    {}", order.status.is_terminal());

    // 4. Pattern Matching with 'if let'
    if let OrderStatus::Delivered = order.status {
        println!("\n4. 'if let' Pattern Match: Order was safely delivered!");
    }

    println!(
        "\nTotal Paid: ${:.2} (Subtotal: ${:.2}, 10% VIP Discount, + ${:.2} Card Fee)",
        order.total_cents() as f64 / 100.0,
        order.subtotal_cents() as f64 / 100.0,
        order.payment.fee_cents() as f64 / 100.0
    );
}

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
        let customer = Customer::new(1, String::from("Alice"), String::from("a@a.com"), false);
        let items = vec![CartItem::new(10, 1, 5000)];
        let mut order = Order::new(OrderId(100), customer, items, PaymentMethod::CashOnDelivery);

        // Starts pending
        assert_eq!(order.status, OrderStatus::Pending);
        assert!(order.status.can_cancel());
        assert!(!order.status.is_terminal());

        // Cannot ship while pending
        assert!(order.ship(String::from("TRK-1")).is_err());

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
        let customer = Customer::new(2, String::from("Bob"), String::from("b@b.com"), false);
        let items = vec![CartItem::new(20, 2, 2500)];
        let mut order = Order::new(OrderId(200), customer, items, PaymentMethod::CashOnDelivery);

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
        assert!(order.confirm(String::from("REC-200")).is_err());
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
        let mut customer = Customer::new(3, String::from("Carol"), String::from("c@c.com"), false);
        customer.upgrade_to_vip(); // 10% discount

        let items = vec![CartItem::new(1, 1, 10000)]; // $100.00 subtotal
        let order = Order::new(
            OrderId(300),
            customer,
            items,
            PaymentMethod::CreditCard {
                last_four: String::from("1111"),
            }, // $1.50 (150 cents) fee
        );

        // Subtotal: 10000 cents
        // VIP discount: 1000 cents
        // Card fee: 150 cents
        // Total: 10000 - 1000 + 150 = 9150 cents ($91.50)
        assert_eq!(order.total_cents(), 9150);
    }
}
