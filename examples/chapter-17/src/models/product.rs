use crate::error::StoreError;

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

    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    pub fn formatted_price(&self) -> String {
        format!("${:.2}", self.price_cents as f64 / 100.0)
    }

    pub fn department_code(&self) -> &str {
        match self.sku.find('-') {
            Some(idx) => &self.sku[..idx],
            None => &self.sku[..],
        }
    }

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

    pub fn matches_sku_prefix(&self, prefix: &str) -> bool {
        self.sku.starts_with(prefix)
    }

    pub fn reduce_stock(&mut self, quantity: u32) -> Result<u32, StoreError> {
        if quantity > self.stock {
            Err(StoreError::InsufficientStock {
                available: self.stock,
                requested: quantity,
            })
        } else {
            self.stock -= quantity;
            Ok(self.stock)
        }
    }

    pub fn restock(&mut self, additional_units: u32) {
        self.stock += additional_units;
    }

    pub fn update_price(&mut self, new_price_cents: u32) {
        self.price_cents = new_price_cents;
    }
}
