use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u64,
    pub sku: String,
    pub name: String,
    pub price_cents: u32,
    pub stock: u32,
}

impl Product {
    pub fn new(id: u64, sku: String, name: String, price_cents: u32, stock: u32) -> Self {
        Self {
            id,
            sku,
            name,
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
    /// Zero heap allocation: returns a 16-byte fat pointer (ptr + len) directly into `sku`.
    pub fn department_code(&self) -> &str {
        match self.sku.find('-') {
            Some(idx) => &self.sku[..idx],
            None => &self.sku[..],
        }
    }

    /// Returns a slice of the product name truncated to `max_bytes` without reallocating,
    /// ensuring the slice boundary respects UTF-8 character boundaries.
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

    /// Adds a tag to customer's unique tag set.
    pub fn add_tag(&mut self, tag: &str) -> bool {
        self.tags.insert(tag.to_string())
    }

    /// Checks if customer has a specific tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    /// Borrows `&self` immutably: reads customer data to display membership badge.
    pub fn display_badge(&self) -> String {
        if self.is_vip {
            format!("[VIP Member] {}", self.name)
        } else {
            format!("[Standard Member] {}", self.name)
        }
    }

    /// Borrows `&mut self` mutably: grants VIP membership status in place.
    pub fn upgrade_to_vip(&mut self) {
        self.is_vip = true;
        self.tags.insert(String::from("vip"));
    }

    /// Borrows `&mut self` mutably: updates email address in place.
    pub fn update_email(&mut self, new_email: String) {
        self.email = new_email;
    }
}

/// OrderId implements `Copy`: 8-byte scalar on the stack, never moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(pub u64);

/// Represents a single line item in a shopping cart.
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

/// A dynamic, multi-item shopping cart backed by a heap-allocated `Vec<CartItem>`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShoppingCart {
    pub items: Vec<CartItem>,
}

impl ShoppingCart {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Adds a product to the cart. If already present, increments quantity in place.
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

    /// Removes an item from the cart by product ID.
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

/// An unconfirmed order owning a dynamic list of items.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingOrder {
    pub order_id: OrderId,
    pub customer: Customer,
    pub items: Vec<CartItem>,
}

/// Finalized invoice/receipt produced when checkout consumes `PendingOrder`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmedReceipt {
    pub receipt_id: String,
    pub order_id: OrderId,
    pub customer_name: String,
    pub item_count: u32,
    pub total_cents: u32,
}

// ============================================================================
// Collection Utility Functions (Vec, HashMap, HashSet & Entry API)
// ============================================================================

/// Groups products by department and counts them using `HashMap` and the Entry API.
pub fn count_products_by_department(products: &[Product]) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for product in products {
        let dept = product.department_code().to_string();
        // The Entry API: zero duplicate lookups!
        *counts.entry(dept).or_insert(0) += 1;
    }
    counts
}

/// Indexes an array or slice of products into a `HashMap<String, Product>` by SKU for O(1) lookups.
pub fn build_sku_index(products: &[Product]) -> HashMap<String, Product> {
    let mut map = HashMap::new();
    for product in products {
        map.insert(product.sku.clone(), product.clone());
    }
    map
}

/// Calculates discount for a multi-item subtotal based on VIP status.
pub fn calculate_discount(customer: &Customer, subtotal_cents: u32) -> u32 {
    if customer.is_vip || customer.has_tag("vip") {
        (subtotal_cents * 10) / 100 // 10% discount for VIPs
    } else {
        0
    }
}

/// Deduplicates user IDs using a `HashSet`.
pub fn collect_unique_customers(orders: &[PendingOrder]) -> HashSet<u64> {
    let mut unique_ids = HashSet::new();
    for order in orders {
        unique_ids.insert(order.customer.id);
    }
    unique_ids
}

/// Consumes ownership of `PendingOrder` by value (Move Semantics) to finalize multi-item checkout.
pub fn finalize_order(order: PendingOrder) -> ConfirmedReceipt {
    let subtotal: u32 = order.items.iter().map(|item| item.line_total()).sum();
    let discount = calculate_discount(&order.customer, subtotal);
    let total_cents = subtotal - discount;
    let item_count = order.items.iter().map(|item| item.quantity).sum();

    ConfirmedReceipt {
        receipt_id: format!("REC-{}-{}", order.order_id.0, order.customer.id),
        order_id: order.order_id,
        customer_name: order.customer.name,
        item_count,
        total_cents,
    }
}

fn main() {
    println!("=== MiniStore: Collections (Vec, HashMap, HashSet & Entry API) ===\n");

    // 1. Dynamic Heap Array: Vec<T> for Products & Cart Items
    println!("1. Dynamic Vectors (Vec<T>):");
    let mut catalog_vec: Vec<Product> = vec![
        Product::new(
            101,
            String::from("TECH-KEY-001"),
            String::from("Mechanical Keyboard"),
            12000,
            10,
        ),
        Product::new(
            102,
            String::from("TECH-MOU-002"),
            String::from("Wireless Gaming Mouse"),
            4500,
            25,
        ),
        Product::new(
            103,
            String::from("OFFC-CHR-003"),
            String::from("Ergonomic Desk Chair"),
            35000,
            5,
        ),
    ];

    // Demonstrating dynamic growth via push:
    catalog_vec.push(Product::new(
        104,
        String::from("OFFC-DSK-004"),
        String::from("Standing Desk Frame"),
        45000,
        4,
    ));

    println!("   Catalog count: {} products", catalog_vec.len());
    println!(
        "   Vector capacity: {} (Allocated on heap)",
        catalog_vec.capacity()
    );

    // 2. HashMap<K, V> with O(1) Fast Lookups & The Entry API
    println!("\n2. Fast Key-Value Lookups with HashMap & Entry API:");
    let sku_map = build_sku_index(&catalog_vec);
    if let Some(mouse) = sku_map.get("TECH-MOU-002") {
        println!("   Found product by SKU 'TECH-MOU-002': {}", mouse.name);
    }

    let dept_counts = count_products_by_department(&catalog_vec);
    println!("   Products grouped by department (via Entry API):");
    for (dept, count) in &dept_counts {
        println!("     - {dept}: {count} item(s)");
    }

    // 3. HashSet<T> for Unique Tags & Deduplication
    println!("\n3. Unique Elements with HashSet<T>:");
    let mut customer = Customer::new(
        501,
        String::from("Grace Hopper"),
        String::from("grace@navy.mil"),
        false,
    );
    customer.add_tag("newsletter");
    customer.add_tag("early_adopter");
    customer.add_tag("newsletter"); // Duplicate insertion is ignored

    println!("   Customer: {}", customer.name);
    println!("   Tags set: {:?}", customer.tags);
    println!(
        "   Has 'newsletter' tag: {}",
        customer.has_tag("newsletter")
    );
    customer.upgrade_to_vip();
    println!("   After VIP upgrade: {:?}", customer.tags);

    // 4. Multi-Item ShoppingCart backed by Vec<CartItem>
    println!("\n4. Shopping Cart Operations (Dynamic Items List):");
    let mut cart = ShoppingCart::new();
    cart.add_item(101, 1, 12000); // 1x Keyboard @ $120.00
    cart.add_item(102, 2, 4500); // 2x Mouse @ $45.00
    cart.add_item(101, 1, 12000); // 1 more Keyboard (increments quantity to 2)

    println!("   Total line items in cart: {}", cart.items.len());
    println!("   Total physical units:     {}", cart.total_units());
    println!(
        "   Cart subtotal:            ${:.2}",
        cart.subtotal_cents() as f64 / 100.0
    );

    // 5. Finalizing Multi-Item Order (Ownership Consumption)
    println!("\n5. Multi-Item Order Finalization:");
    let pending_order = PendingOrder {
        order_id: OrderId(8801),
        customer,
        items: cart.items, // Ownership of Vec<CartItem> moves into pending_order
    };

    let receipt = finalize_order(pending_order);
    println!("   Receipt ID:   {}", receipt.receipt_id);
    println!("   Customer:     {}", receipt.customer_name);
    println!("   Total Units:  {}", receipt.item_count);
    println!(
        "   Total Paid:   ${:.2} (10% VIP discount applied)",
        receipt.total_cents as f64 / 100.0
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_shopping_cart_add_and_aggregate() {
        let mut cart = ShoppingCart::new();
        assert!(cart.is_empty());

        cart.add_item(1, 2, 1000); // 2 x $10.00 = $20.00
        cart.add_item(2, 1, 2500); // 1 x $25.00 = $25.00
        cart.add_item(1, 1, 1000); // Adds 1 more to product 1 -> 3 x $10.00 = $30.00

        assert_eq!(cart.items.len(), 2);
        assert_eq!(cart.total_units(), 4);
        assert_eq!(cart.subtotal_cents(), 5500); // $30.00 + $25.00 = $55.00
    }

    #[test]
    fn test_vec_shopping_cart_remove_item() {
        let mut cart = ShoppingCart::new();
        cart.add_item(10, 2, 500);
        cart.add_item(20, 1, 1500);

        assert_eq!(cart.items.len(), 2);
        let removed = cart.remove_item(10);
        assert!(removed);
        assert_eq!(cart.items.len(), 1);
        assert_eq!(cart.items[0].product_id, 20);

        let not_found = cart.remove_item(999);
        assert!(!not_found);
    }

    #[test]
    fn test_hashmap_entry_api_department_counts() {
        let products = [
            Product::new(1, String::from("TECH-1"), String::from("P1"), 100, 5),
            Product::new(2, String::from("TECH-2"), String::from("P2"), 200, 5),
            Product::new(3, String::from("HOME-1"), String::from("P3"), 300, 5),
        ];

        let counts = count_products_by_department(&products);
        assert_eq!(counts.get("TECH"), Some(&2));
        assert_eq!(counts.get("HOME"), Some(&1));
        assert_eq!(counts.get("FOOD"), None);
    }

    #[test]
    fn test_hashset_customer_tags() {
        let mut customer = Customer::new(1, String::from("Alice"), String::from("a@a.com"), false);
        assert!(!customer.has_tag("vip"));

        customer.add_tag("beta_tester");
        customer.add_tag("beta_tester"); // Duplicate
        assert_eq!(customer.tags.len(), 1);
        assert!(customer.has_tag("beta_tester"));

        customer.upgrade_to_vip();
        assert!(customer.is_vip);
        assert!(customer.has_tag("vip"));
    }

    #[test]
    fn test_multi_item_order_finalize() {
        let mut customer = Customer::new(1, String::from("Bob"), String::from("b@b.com"), false);
        customer.upgrade_to_vip();

        let items = vec![
            CartItem::new(1, 2, 2000), // 2 * $20 = $40.00 (4000 cents)
            CartItem::new(2, 1, 6000), // 1 * $60 = $60.00 (6000 cents)
        ]; // Subtotal: 10000 cents ($100.00)

        let order = PendingOrder {
            order_id: OrderId(42),
            customer,
            items,
        };

        // 10% VIP discount on 10000 = 1000 -> Total: 9000 cents ($90.00)
        let receipt = finalize_order(order);
        assert_eq!(receipt.order_id, OrderId(42));
        assert_eq!(receipt.customer_name, "Bob");
        assert_eq!(receipt.item_count, 3);
        assert_eq!(receipt.total_cents, 9000);
    }
}
