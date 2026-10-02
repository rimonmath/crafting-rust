use crate::models::Product;
use std::collections::HashMap;

/// Catalog of products indexed by SKU for O(1) lookups returning `Option<&Product>`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Catalog {
    products: HashMap<String, Product>,
}

impl Catalog {
    pub fn new() -> Self {
        Self {
            products: HashMap::new(),
        }
    }

    pub fn add_product(&mut self, product: Product) {
        self.products.insert(product.sku.clone(), product);
    }

    pub fn find_by_sku(&self, sku: &str) -> Option<&Product> {
        self.products.get(sku)
    }

    pub fn find_by_sku_mut(&mut self, sku: &str) -> Option<&mut Product> {
        self.products.get_mut(sku)
    }

    pub fn find_by_id(&self, id: u64) -> Option<&Product> {
        self.products.values().find(|product| product.id == id)
    }

    pub fn find_by_id_mut(&mut self, id: u64) -> Option<&mut Product> {
        self.products.values_mut().find(|product| product.id == id)
    }

    pub fn product_price(&self, sku: &str) -> Option<u32> {
        self.find_by_sku(sku).map(|p| p.price_cents)
    }

    pub fn is_product_in_stock(&self, sku: &str) -> bool {
        self.find_by_sku(sku)
            .map(|p| p.is_in_stock())
            .unwrap_or(false)
    }

    pub fn total_products(&self) -> usize {
        self.products.len()
    }

    /// Returns a sorted list of all products in the catalog by ID.
    pub fn get_products(&self) -> Vec<Product> {
        let mut list: Vec<Product> = self.products.values().cloned().collect();
        list.sort_by_key(|p| p.id);
        list
    }

    /// Paginates products using the generic `Page<T>` abstraction.
    pub fn paginate(&self, page: usize, per_page: usize) -> crate::models::Page<Product> {
        crate::models::paginate(self.get_products(), page, per_page)
    }

    /// Returns an iterator yielding immutable references to all catalog products.
    pub fn iter(&self) -> std::collections::hash_map::Values<'_, String, Product> {
        self.products.values()
    }

    /// Finds products matching a dynamic closure predicate.
    ///
    /// Demonstrates passing a predicate closure bounded by `Fn(&Product) -> bool`.
    pub fn find_products<P>(&self, predicate: P) -> Vec<&Product>
    where
        P: Fn(&Product) -> bool,
    {
        self.products.values().filter(|p| predicate(p)).collect()
    }

    /// Filters products by category using iterator `.filter()` and collects into a `Vec<&Product>`.
    pub fn filter_by_category(&self, category: crate::models::ProductCategory) -> Vec<&Product> {
        self.products
            .values()
            .filter(|p| p.category == category)
            .collect()
    }

    /// Finds products within a price range using iterator filtering.
    pub fn products_in_price_range(&self, min_cents: u32, max_cents: u32) -> Vec<&Product> {
        self.products
            .values()
            .filter(|p| p.price_cents >= min_cents && p.price_cents <= max_cents)
            .collect()
    }

    /// Calculates total inventory valuation in cents using `.map()` and `.sum()`.
    pub fn total_inventory_valuation(&self) -> u64 {
        self.products
            .values()
            .map(|p| (p.price_cents as u64) * (p.stock as u64))
            .sum()
    }

    /// Extracts a list of all product SKUs using `.map()` and `.collect()`.
    pub fn skus(&self) -> Vec<&str> {
        self.products.values().map(|p| p.sku.as_str()).collect()
    }
}

impl<'a> IntoIterator for &'a Catalog {
    type Item = &'a Product;
    type IntoIter = std::collections::hash_map::Values<'a, String, Product>;

    fn into_iter(self) -> Self::IntoIter {
        self.products.values()
    }
}
