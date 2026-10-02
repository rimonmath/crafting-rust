use super::cart::CartItem;
use super::customer::{Customer, OrderId};
use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq)]
pub struct Coupon {
    pub code: String,
    pub discount_percent: u32,
}

impl Coupon {
    pub fn new(code: String, discount_percent: u32) -> Self {
        Self {
            code,
            discount_percent,
        }
    }

    /// Validates that coupon has a non-empty code and a realistic discount percentage.
    pub fn validate(&self) -> Result<(), StoreError> {
        if self.code.trim().is_empty() {
            Err(StoreError::InvalidCoupon {
                code: self.code.clone(),
                reason: String::from("Coupon code cannot be empty"),
            })
        } else if self.discount_percent == 0 || self.discount_percent > 100 {
            Err(StoreError::InvalidCoupon {
                code: self.code.clone(),
                reason: format!(
                    "Discount percentage {} must be between 1 and 100",
                    self.discount_percent
                ),
            })
        } else {
            Ok(())
        }
    }
}

/// Represents payment instruments accepted by MiniStore.
#[derive(Debug, Clone, PartialEq)]
pub enum PaymentMethod {
    CreditCard { last_four: String },
    BankTransfer { reference: String },
    CashOnDelivery,
}

impl PaymentMethod {
    pub fn fee_cents(&self) -> u32 {
        match self {
            Self::CreditCard { .. } => 150,
            Self::BankTransfer { .. } => 0,
            Self::CashOnDelivery => 300,
        }
    }

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
    pub fn display_status(&self) -> String {
        match self {
            Self::Pending => String::from("Awaiting Confirmation"),
            Self::Confirmed { receipt_id } => format!("Confirmed (Receipt: {receipt_id})"),
            Self::Shipped { tracking_number } => format!("Shipped (Tracking: {tracking_number})"),
            Self::Delivered => String::from("Delivered to Customer"),
            Self::Cancelled { reason } => format!("Cancelled (Reason: {reason})"),
        }
    }

    pub fn can_cancel(&self) -> bool {
        match self {
            Self::Pending | Self::Confirmed { .. } => true,
            Self::Shipped { .. } | Self::Delivered | Self::Cancelled { .. } => false,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Delivered | Self::Cancelled { .. })
    }
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_status())
    }
}

/// Full Order domain model with state-machine lifecycle and optional coupon discount.
#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub order_id: OrderId,
    pub customer: Customer,
    pub items: Vec<CartItem>,
    pub payment: PaymentMethod,
    pub status: OrderStatus,
    pub coupon: Option<Coupon>,
}

impl Order {
    pub fn new(
        order_id: OrderId,
        customer: Customer,
        items: Vec<CartItem>,
        payment: PaymentMethod,
        coupon: Option<Coupon>,
    ) -> Self {
        Self {
            order_id,
            customer,
            items,
            payment,
            status: OrderStatus::Pending,
            coupon,
        }
    }

    pub fn apply_coupon(&mut self, coupon: Coupon) {
        self.coupon = Some(coupon);
    }

    pub fn remove_coupon(&mut self) -> Option<Coupon> {
        self.coupon.take()
    }

    pub fn confirm(&mut self, receipt_id: String) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Pending => {
                self.status = OrderStatus::Confirmed { receipt_id };
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                current: self.status.display_status(),
                action: String::from("confirm"),
            }),
        }
    }

    pub fn ship(&mut self, tracking_number: String) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Confirmed { .. } => {
                self.status = OrderStatus::Shipped { tracking_number };
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                current: self.status.display_status(),
                action: String::from("ship"),
            }),
        }
    }

    pub fn mark_delivered(&mut self) -> Result<(), StoreError> {
        match &self.status {
            OrderStatus::Shipped { .. } => {
                self.status = OrderStatus::Delivered;
                Ok(())
            }
            _ => Err(StoreError::InvalidStateTransition {
                current: self.status.display_status(),
                action: String::from("deliver"),
            }),
        }
    }

    pub fn cancel(&mut self, reason: String) -> Result<(), StoreError> {
        if self.status.can_cancel() {
            self.status = OrderStatus::Cancelled { reason };
            Ok(())
        } else {
            Err(StoreError::InvalidStateTransition {
                current: self.status.display_status(),
                action: String::from("cancel"),
            })
        }
    }

    pub fn subtotal_cents(&self) -> u32 {
        self.items.iter().map(|item| item.line_total()).sum()
    }

    pub fn total_cents(&self) -> u32 {
        let subtotal = self.subtotal_cents();
        let discount =
            crate::checkout::calculate_discount(&self.customer, self.coupon.as_ref(), subtotal);
        subtotal.saturating_sub(discount) + self.payment.fee_cents()
    }

    /// Returns an iterator over immutable references to order items.
    pub fn iter(&self) -> std::slice::Iter<'_, CartItem> {
        self.items.iter()
    }

    /// Returns total quantity of units ordered using `.map()` and `.sum()`.
    pub fn total_quantity(&self) -> u32 {
        self.items.iter().map(|item| item.quantity).sum()
    }
}

impl<'a> IntoIterator for &'a Order {
    type Item = &'a CartItem;
    type IntoIter = std::slice::Iter<'a, CartItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}
