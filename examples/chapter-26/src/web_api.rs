use crate::catalog::Catalog;
use crate::checkout::checkout;
use crate::error::StoreError;
use crate::models::{
    Coupon, Customer, Order, OrderId, PaymentMethod, Product, ProductCategory, ShoppingCart,
};
use crate::persistence::StorePersistence;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Shared thread-safe application state accessed across concurrent HTTP request handlers.
#[derive(Clone)]
pub struct AppState {
    pub catalog: Arc<RwLock<Catalog>>,
    pub orders: Arc<Mutex<Vec<Order>>>,
    pub persistence: Option<StorePersistence>,
}

impl AppState {
    pub fn new(
        catalog: Catalog,
        orders: Vec<Order>,
        persistence: Option<StorePersistence>,
    ) -> Self {
        Self {
            catalog: Arc::new(RwLock::new(catalog)),
            orders: Arc::new(Mutex::new(orders)),
            persistence,
        }
    }
}

/// Structured API error response for standardized JSON error payloads.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

impl ApiError {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
        }
    }
}

impl IntoResponse for StoreError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::ProductNotFound { .. } => (StatusCode::NOT_FOUND, "PRODUCT_NOT_FOUND"),
            Self::InsufficientStock { .. } => (StatusCode::BAD_REQUEST, "INSUFFICIENT_STOCK"),
            Self::EmptyCart => (StatusCode::BAD_REQUEST, "EMPTY_CART"),
            Self::InvalidCoupon { .. } => (StatusCode::UNPROCESSABLE_ENTITY, "INVALID_COUPON"),
            Self::InvalidStateTransition { .. } => {
                (StatusCode::CONFLICT, "INVALID_STATE_TRANSITION")
            }
            Self::InvalidPayment { .. } => (StatusCode::PAYMENT_REQUIRED, "INVALID_PAYMENT"),
            Self::IoError { .. } | Self::SerializationError { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "STORAGE_ERROR")
            }
        };

        let body = Json(ApiError::new(code, self.message()));
        (status, body).into_response()
    }
}

/// Request DTO for creating a new product via HTTP POST.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub id: u64,
    pub sku: String,
    pub name: String,
    pub category: ProductCategory,
    pub price_cents: u32,
    pub stock: u32,
}

/// Single item in checkout request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutItemRequest {
    pub product_id: u64,
    pub quantity: u32,
}

/// Request DTO for processing a checkout via HTTP POST.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutRequest {
    pub order_id: u64,
    pub customer: Customer,
    pub items: Vec<CheckoutItemRequest>,
    pub payment: PaymentMethod,
    pub coupon: Option<Coupon>,
}

// ---------------------------------------------------------------------------
// Route Handlers
// ---------------------------------------------------------------------------

/// GET /health - General service metadata.
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "ministore-api",
        "version": "1.0.0"
    }))
}

/// GET /health/live - Liveness probe checking that the application process is running.
pub async fn health_liveness() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "alive"
    }))
}

/// GET /health/ready - Readiness probe ensuring backend storage dependencies are operational.
pub async fn health_readiness(
    State(state): State<AppState>,
) -> (StatusCode, Json<serde_json::Value>) {
    let storage_ready = if let Some(ref p) = state.persistence {
        std::fs::create_dir_all(p.base_dir()).is_ok()
    } else {
        true
    };

    if storage_ready {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ready",
                "storage": "accessible"
            })),
        )
    } else {
        tracing::error!("Readiness check failed: storage directory inaccessible");
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "not_ready",
                "storage": "inaccessible"
            })),
        )
    }
}

/// GET /api/products - Lists all products in the catalog.
pub async fn list_products(State(state): State<AppState>) -> Json<Vec<Product>> {
    let catalog = state.catalog.read().await;
    Json(catalog.get_products())
}

/// GET /api/products/{id} - Fetches a single product by numeric ID.
pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Product>, StoreError> {
    let catalog = state.catalog.read().await;
    match catalog.find_by_id(id) {
        Some(product) => Ok(Json(product.clone())),
        None => Err(StoreError::ProductNotFound {
            identifier: id.to_string(),
        }),
    }
}

/// POST /api/products - Creates a new product in the catalog.
pub async fn create_product(
    State(state): State<AppState>,
    Json(payload): Json<CreateProductRequest>,
) -> (StatusCode, Json<Product>) {
    let product = Product::new(
        payload.id,
        payload.sku,
        payload.name,
        payload.category,
        payload.price_cents,
        payload.stock,
    );

    {
        let mut catalog = state.catalog.write().await;
        catalog.add_product(product.clone());

        if let Some(ref persistence) = state.persistence {
            let _ = persistence.save_catalog(&catalog);
        }
    }

    tracing::info!(product_id = product.id, sku = %product.sku, "New product added to catalog via API");
    (StatusCode::CREATED, Json(product))
}

/// GET /api/orders - Lists all submitted orders.
pub async fn list_orders(State(state): State<AppState>) -> Json<Vec<Order>> {
    let orders = state.orders.lock().await;
    Json(orders.clone())
}

/// GET /api/orders/{id} - Fetches a specific order by OrderId.
pub async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Order>, StoreError> {
    let orders = state.orders.lock().await;
    match orders.iter().find(|o| o.order_id == OrderId(id)) {
        Some(order) => Ok(Json(order.clone())),
        None => Err(StoreError::ProductNotFound {
            identifier: format!("Order #{id}"),
        }),
    }
}

/// POST /api/checkout - Transactional order checkout API.
pub async fn process_checkout(
    State(state): State<AppState>,
    Json(payload): Json<CheckoutRequest>,
) -> Result<(StatusCode, Json<Order>), StoreError> {
    if payload.items.is_empty() {
        return Err(StoreError::EmptyCart);
    }

    let mut cart = ShoppingCart::new();
    let mut catalog_lock = state.catalog.write().await;

    // Build cart items matching catalog prices
    for item in payload.items {
        let product = catalog_lock.find_by_id(item.product_id).ok_or_else(|| {
            StoreError::ProductNotFound {
                identifier: item.product_id.to_string(),
            }
        })?;

        cart.add_item(product.id, item.quantity, product.price_cents);
    }

    // Execute core domain checkout
    let order = checkout(
        OrderId(payload.order_id),
        payload.customer,
        &mut cart,
        &mut catalog_lock,
        payload.payment,
        payload.coupon,
    )?;

    // Record order in shared orders list
    {
        let mut orders_lock = state.orders.lock().await;
        orders_lock.push(order.clone());

        if let Some(ref persistence) = state.persistence {
            let _ = persistence.save_catalog(&catalog_lock);
            let _ = persistence.save_orders(&orders_lock);
        }
    }

    tracing::info!(
        order_id = order.order_id.0,
        customer = %order.customer.name,
        total_cents = order.total_cents(),
        "Transactional checkout completed via API"
    );

    Ok((StatusCode::CREATED, Json(order)))
}

/// Graceful shutdown listener for SIGINT and SIGTERM OS termination signals.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, initiating graceful server shutdown...");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful server shutdown...");
        },
    }
}

/// Assembles the complete Axum router configured with all routes, tracing layer, and application state.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/live", get(health_liveness))
        .route("/health/ready", get(health_readiness))
        .route("/api/products", get(list_products).post(create_product))
        .route("/api/products/{id}", get(get_product))
        .route("/api/orders", get(list_orders))
        .route("/api/orders/{id}", get(get_order))
        .route("/api/checkout", post(process_checkout))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}
