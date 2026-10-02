use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct Customer {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub is_vip: bool,
    pub tags: HashSet<String>,
}

impl Customer {
    pub fn new(id: u64, name: String, email: String, phone: Option<String>, is_vip: bool) -> Self {
        Self {
            id,
            name,
            email,
            phone,
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

    pub fn formatted_phone(&self) -> &str {
        self.phone.as_deref().unwrap_or("Unspecified")
    }
}

/// OrderId implements `Copy`: 8-byte scalar on the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(pub u64);

impl std::fmt::Display for OrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}
