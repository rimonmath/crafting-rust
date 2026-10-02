//! Declarative macros for ergonomic MiniStore domain modeling and testing.

/// Constructs a `Product` with ergonomic syntax, automatic `.to_string()` conversion,
/// and shorthand `ProductCategory` variants.
///
/// # Examples
/// ```
/// use ministore::{product, ProductCategory};
///
/// // Full syntax with category ident shorthand:
/// let p1 = product!(1, "LAP-001", "Laptop", Electronics, 120000, 10);
/// assert_eq!(p1.price_cents, 120000);
/// assert_eq!(p1.category, ProductCategory::Electronics);
///
/// // Default stock (defaults to 1):
/// let p2 = product!(2, "PEN-001", "Pen", OfficeSupplies, 150);
/// assert_eq!(p2.stock, 1);
/// ```
#[macro_export]
macro_rules! product {
    // Branch 1: Named argument style
    (id: $id:expr, sku: $sku:expr, name: $name:expr, category: $cat:ident, price: $price:expr, stock: $stock:expr) => {
        $crate::models::Product::new(
            $id,
            $sku.to_string(),
            $name.to_string(),
            $crate::models::ProductCategory::$cat,
            $price,
            $stock,
        )
    };

    // Branch 2: Positional with explicit category variant ident and stock
    ($id:expr, $sku:expr, $name:expr, $cat:ident, $price:expr, $stock:expr) => {
        $crate::models::Product::new(
            $id,
            $sku.to_string(),
            $name.to_string(),
            $crate::models::ProductCategory::$cat,
            $price,
            $stock,
        )
    };

    // Branch 3: Positional with explicit category expression and stock
    ($id:expr, $sku:expr, $name:expr, expr $cat:expr, $price:expr, $stock:expr) => {
        $crate::models::Product::new(
            $id,
            $sku.to_string(),
            $name.to_string(),
            $cat,
            $price,
            $stock,
        )
    };

    // Branch 4: Positional with default stock = 1
    ($id:expr, $sku:expr, $name:expr, $cat:ident, $price:expr) => {
        $crate::product!($id, $sku, $name, $cat, $price, 1)
    };
}

/// Constructs a populated `Catalog` from a list of products.
///
/// # Examples
/// ```
/// use ministore::{catalog, product};
///
/// let catalog = catalog![
///     product!(1, "LAP-001", "Laptop", Electronics, 120000, 5),
///     product!(2, "DESK-01", "Desk", Furniture, 45000, 2),
/// ];
/// assert_eq!(catalog.total_products(), 2);
/// ```
#[macro_export]
macro_rules! catalog {
    ( $( $prod:expr ),* $(,)? ) => {
        {
            let mut catalog = $crate::catalog::Catalog::new();
            $(
                catalog.add_product($prod);
            )*
            catalog
        }
    };
}

/// Constructs a `ShoppingCart` populated with items.
///
/// Supports either tuple items `(product_id, quantity, unit_price_cents)`
/// or named field items `(item: id, qty: count, price: cents)`.
///
/// # Examples
/// ```
/// use ministore::cart;
///
/// let cart = cart![
///     (1, 2, 120000),
///     (2, 5, 150),
/// ];
/// assert_eq!(cart.items.len(), 2);
/// ```
#[macro_export]
macro_rules! cart {
    // Empty cart
    () => {
        $crate::models::ShoppingCart::new()
    };

    // Tuple style: (product_id, quantity, unit_price_cents)
    ( $( ( $prod_id:expr, $qty:expr, $price:expr ) ),* $(,)? ) => {
        {
            let mut cart = $crate::models::ShoppingCart::new();
            $(
                cart.add_item($prod_id, $qty, $price);
            )*
            cart
        }
    };

    // Named style: (item: $id, qty: $qty, price: $price)
    ( $( ( item: $prod_id:expr, qty: $qty:expr, price: $price:expr ) ),* $(,)? ) => {
        {
            let mut cart = $crate::models::ShoppingCart::new();
            $(
                cart.add_item($prod_id, $qty, $price);
            )*
            cart
        }
    };
}

/// Computes the sum of money amounts or numbers using a recursive variadic macro pattern.
///
/// # Examples
/// ```
/// use ministore::calculate_total;
///
/// let total = calculate_total!(100, 200, 300);
/// assert_eq!(total, 600);
/// ```
#[macro_export]
macro_rules! calculate_total {
    // Base case: empty
    () => {
        0u32
    };

    // Single item base case
    ($head:expr) => {
        $head
    };

    // Recursive case: head + tail
    ($head:expr, $($tail:expr),+ $(,)?) => {
        $head + $crate::calculate_total!($($tail),+)
    };
}

/// Asserts that a product has sufficient stock, formatting an informative error message if false.
///
/// # Examples
/// ```
/// use ministore::{assert_in_stock, product};
///
/// let p = product!(1, "LAP-001", "Laptop", Electronics, 120000, 10);
/// assert_in_stock!(p, 5);
/// ```
#[macro_export]
macro_rules! assert_in_stock {
    ($prod:expr, $required:expr) => {
        assert!(
            $prod.stock >= $required,
            "Stock assertion failed for SKU '{}' ({}): available {}, required {}",
            $prod.sku,
            $prod.name,
            $prod.stock,
            $required
        );
    };
}
