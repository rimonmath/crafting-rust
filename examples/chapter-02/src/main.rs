fn main() {
    println!("=== MiniStore ===");
    println!("Version: 0.1.0");
    println!("Status: Initialized");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_initialization() {
        let app_name = "MiniStore";
        assert_eq!(app_name, "MiniStore");
    }
}
