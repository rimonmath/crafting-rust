struct Product {
    id: u64,
    name: String,
    price_cents: u32,
    in_stock: bool,
}

fn main() {
    let item = Product {
        id: 101,
        name: String::from("Rust Engineering Guide"),
        price_cents: 2999,
        in_stock: true,
    };

    println!("MiniStore Item #{}: {}", item.id, item.name);
    println!("Price: ${:.2}", item.price_cents as f64 / 100.0);
    println!("Available: {}", if item.in_stock { "Yes" } else { "No" });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_creation() {
        let item = Product {
            id: 1,
            name: String::from("Test Product"),
            price_cents: 1000,
            in_stock: true,
        };
        assert_eq!(item.id, 1);
        assert_eq!(item.name, "Test Product");
        assert_eq!(item.price_cents, 1000);
        assert!(item.in_stock);
    }
}
