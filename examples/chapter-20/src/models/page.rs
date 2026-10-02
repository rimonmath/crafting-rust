/// Generic pagination container and helper functions for MiniStore.
///
/// Demonstrates generic structs (`Page<T>`), generic functions (`paginate<T>`),
/// and generic enums (`ApiResponse<T>`).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: usize,
    pub per_page: usize,
    pub total_items: usize,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, page: usize, per_page: usize, total_items: usize) -> Self {
        Self {
            items,
            page,
            per_page,
            total_items,
        }
    }

    pub fn total_pages(&self) -> usize {
        if self.per_page == 0 {
            0
        } else {
            self.total_items.div_ceil(self.per_page)
        }
    }

    pub fn has_next(&self) -> bool {
        self.page > 0 && self.page < self.total_pages()
    }

    pub fn has_previous(&self) -> bool {
        self.page > 1 && self.page <= self.total_pages() + 1
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Transforms a `Page<T>` into a `Page<U>` using a function pointer.
    ///
    /// Demonstrates generic methods introducing new type parameters (`U`).
    pub fn map<U>(self, transform: fn(T) -> U) -> Page<U> {
        let mapped_items = self.items.into_iter().map(transform).collect();
        Page {
            items: mapped_items,
            page: self.page,
            per_page: self.per_page,
            total_items: self.total_items,
        }
    }
}

/// Generic function that partitions an owned vector into a paginated slice container.
pub fn paginate<T>(items: Vec<T>, page: usize, per_page: usize) -> Page<T> {
    let total_items = items.len();
    if per_page == 0 || page == 0 {
        return Page::new(Vec::new(), page, per_page, total_items);
    }

    let start_index = (page - 1) * per_page;
    if start_index >= total_items {
        return Page::new(Vec::new(), page, per_page, total_items);
    }

    let end_index = (start_index + per_page).min(total_items);
    let page_items: Vec<T> = items
        .into_iter()
        .skip(start_index)
        .take(end_index - start_index)
        .collect();

    Page::new(page_items, page, per_page, total_items)
}

/// Generic API response wrapper demonstrating generic enums with payload data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiResponse<T> {
    Success { data: T, total: usize },
    Error { message: String },
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T, total: usize) -> Self {
        Self::Success { data, total }
    }

    pub fn err(message: String) -> Self {
        Self::Error { message }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    pub fn data(&self) -> Option<&T> {
        match self {
            Self::Success { data, .. } => Some(data),
            Self::Error { .. } => None,
        }
    }
}
