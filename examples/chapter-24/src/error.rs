/// Domain errors that can occur during MiniStore business operations.
#[derive(Debug, Clone, PartialEq)]
pub enum StoreError {
    InsufficientStock { available: u32, requested: u32 },
    InvalidStateTransition { current: String, action: String },
    ProductNotFound { identifier: String },
    InvalidPayment { reason: String },
    InvalidCoupon { code: String, reason: String },
    EmptyCart,
    IoError { path: String, message: String },
    SerializationError { message: String },
}

impl StoreError {
    /// User-friendly descriptive error message.
    pub fn message(&self) -> String {
        match self {
            Self::InsufficientStock {
                available,
                requested,
            } => {
                format!("Insufficient stock: requested {requested}, but only {available} available")
            }
            Self::InvalidStateTransition { current, action } => {
                format!("Cannot perform action '{action}' while order is in '{current}' state")
            }
            Self::ProductNotFound { identifier } => {
                format!("Product '{identifier}' was not found in catalog")
            }
            Self::InvalidPayment { reason } => {
                format!("Payment processing failed: {reason}")
            }
            Self::InvalidCoupon { code, reason } => {
                format!("Coupon '{code}' is invalid: {reason}")
            }
            Self::EmptyCart => String::from("Cannot checkout with an empty shopping cart"),
            Self::IoError { path, message } => {
                format!("I/O error at '{path}': {message}")
            }
            Self::SerializationError { message } => {
                format!("Serialization error: {message}")
            }
        }
    }
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for StoreError {}
