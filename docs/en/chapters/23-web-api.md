# Chapter 23: Building a Rust Web API

## What You'll Learn
- Why real-world applications must expose services over **HTTP Web APIs** rather than running solely as local CLI binaries.
- The anatomy of modern Rust web applications powered by **Axum**, **Tokio**, and **Tower**.
- Macro-free routing using **`axum::Router`** and HTTP method handlers (`get`, `post`).
- **Type-safe extractors**:
  - **`Path<T>`**: Parsing dynamic route parameters (e.g., `/api/products/{id}`).
  - **`Json<T>`**: Parsing and validating deserializable request bodies.
  - **`State<T>`**: Thread-safe sharing of application dependencies.
- **Type-safe HTTP responses**:
  - Implementing **`IntoResponse`** for domain errors (`StoreError`).
  - Returning typed status codes (`StatusCode::OK`, `StatusCode::CREATED`, `StatusCode::NOT_FOUND`, `StatusCode::BAD_REQUEST`).
- Protecting shared mutable application state concurrently with **`Arc<RwLock<Catalog>>`** and **`Arc<Mutex<Vec<Order>>>`**.
- Testing HTTP routers in memory with **`tower::ServiceExt::oneshot`** without binding to actual OS network ports.
- Implementing the MiniStore Web API:
  - System health check (`GET /health`).
  - Product catalog querying and creation (`GET /api/products`, `POST /api/products`, `GET /api/products/{id}`).
  - Order transaction history (`GET /api/orders`, `GET /api/orders/{id}`).
  - Transactional order checkout API (`POST /api/checkout`).

---

## Why Do We Need This?

Up to Chapter 22, MiniStore is a standalone binary running on a single computer. When a user runs `cargo run`, the program executes in their terminal and exits.

However, modern commerce does not happen inside a single developer's terminal. It must serve:
- Web storefronts built with React, Vue, or Svelte.
- Native mobile applications on iOS and Android.
- Payment gateway webhooks and shipping courier tracking callbacks.
- Third-party ERP and inventory management platforms.

```
CLI Model (Single Machine, Isolated):
[User Terminal] ──► cargo run ──► Executes in RAM/File ──► Exits

HTTP Web API Model (Distributed, Global):
[Web Browser]  ──┐
[Mobile App]   ──┼──► HTTP Requests (GET/POST) ──► [Axum Server] ──► MiniStore Core
[Partner API]  ──┘                                        │
                                                   JSON Responses
```

To connect MiniStore to the world, we need an **HTTP Web API**: a long-running service that listens for HTTP network requests, parses JSON payloads, routes execution to domain logic, and replies with standard HTTP status codes and JSON envelopes.

---

## Why Axum?

The Rust ecosystem offers several web frameworks (Actix Web, Rocket, Warp). Among them, **Axum** (created and maintained by the official Tokio team) has become the gold standard for idiomatic Rust web development for three key reasons:

1. **Native Tokio Integration**: Built directly on top of Tokio (the async runtime), Hyper (blazing-fast HTTP/1.1 and HTTP/2 engine), and Tower (middleware ecosystem).
2. **Macro-Free Design**: Unlike frameworks in other languages (such as Spring in Java or FastAPI in Python) or older Rust frameworks that rely heavily on complex attribute macros, Axum uses standard Rust functions and traits.
3. **Type-Safe Extractors**: If your function takes arguments that implement Axum's `FromRequest` or `FromRequestParts` traits, Axum automatically parses and validates them before calling your handler. If parsing fails, Axum rejects the request with an appropriate HTTP error automatically.

```
                  Type-Safe Extractors:
HTTP Request ──► Path(id), Json(body), State(state) ──► Handler async fn
                                                                │
                                                                ▼
HTTP Response ◄── IntoResponse: (StatusCode, Json(data)) ◄──────┘
```

Add Axum and Tower to `Cargo.toml`:
```toml
[dependencies]
axum = "0.8"
tower = { version = "0.5", features = ["util"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

## Core Web API Concepts in Rust

### 1. The Axum Router & Handlers

In Axum, a route handler is simply an `async fn`:

```rust
use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "healthy" }))
}

let app = Router::new().route("/health", get(health_check));
```

Handlers can accept multiple extractors as arguments and return any type that implements `IntoResponse`.

---

### 2. Extractors: `Path`, `Json`, and `State`

Extractors are how handlers pull data out of an incoming HTTP request:

- **`Path<T>`**: Deserializes route segments matching `{param}`:
  ```rust
  async fn get_product(Path(id): Path<u64>) -> String {
      format!("Fetching product ID: {id}")
  }
  ```
- **`Json<T>`**: Buffers and deserializes the JSON request body into a strongly typed struct `T` implementing `serde::Deserialize`:
  ```rust
  async fn create_product(Json(payload): Json<CreateProductRequest>) -> StatusCode {
      println!("Created SKU: {}", payload.sku);
      StatusCode::CREATED
  }
  ```
- **`State<AppState>`**: Injects shared dependencies (database pools, caches, mutexes) into handlers safely across threads.

---

### 3. Shared State: `Arc<RwLock<T>>` vs `Arc<Mutex<T>>`

In a multi-threaded web server, incoming requests are handled concurrently across Tokio's worker threads.

To allow handlers to read and write store data safely without race conditions, we wrap shared structures in smart pointers:

```rust
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Clone)]
pub struct AppState {
    // RwLock allows hundreds of simultaneous readers (browsing catalog)
    // but grants exclusive access to a single writer (adding/updating product)
    pub catalog: Arc<RwLock<Catalog>>,
    
    // Mutex provides exclusive mutual exclusion for order history writes
    pub orders: Arc<Mutex<Vec<Order>>>,
}
```

> **Why `tokio::sync::RwLock` instead of `std::sync::RwLock`?**
> In async handlers, if you hold a lock across an `.await` suspension point, using standard library locks will block the underlying OS executor thread. Tokio's async locks yield cooperative control if the lock is contested, preventing thread starvation.

---

### 4. Custom Error Handling with `IntoResponse`

Rather than manually constructing HTTP status codes in every error branch, idiomatic Rust maps domain errors into HTTP responses using the **`IntoResponse`** trait:

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

impl IntoResponse for StoreError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::ProductNotFound { .. } => (StatusCode::NOT_FOUND, "PRODUCT_NOT_FOUND"),
            Self::InsufficientStock { .. } => (StatusCode::BAD_REQUEST, "INSUFFICIENT_STOCK"),
            Self::EmptyCart => (StatusCode::BAD_REQUEST, "EMPTY_CART"),
            Self::InvalidCoupon { .. } => (StatusCode::UNPROCESSABLE_ENTITY, "INVALID_COUPON"),
            Self::InvalidStateTransition { .. } => (StatusCode::CONFLICT, "INVALID_STATE_TRANSITION"),
            Self::InvalidPayment { .. } => (StatusCode::PAYMENT_REQUIRED, "INVALID_PAYMENT"),
            Self::IoError { .. } | Self::SerializationError { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "STORAGE_ERROR")
            }
        };

        let body = Json(ApiError {
            error: code.to_string(),
            message: self.message(),
        });

        (status, body).into_response()
    }
}
```

Now, any handler can simply return `Result<Json<Product>, StoreError>`. If the function returns `Err(StoreError::ProductNotFound { .. })`, Axum automatically serializes the error payload and sets HTTP status `404 Not Found`!

---

### 5. In-Memory Testing without Sockets

Starting an actual HTTP server on `127.0.0.1:3000` during unit tests can cause port collision errors, firewall prompts, and slow network setup.

Because Axum routers implement Tower's **`Service<Request>`** trait, we can dispatch simulated HTTP requests directly into the router in memory using **`tower::ServiceExt::oneshot`**:

```rust
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

let app = create_router(state);

let response = app
    .oneshot(
        Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

assert_eq!(response.status(), StatusCode::OK);
```

This runs at in-memory CPU speed—hundreds of full HTTP tests execute in milliseconds!

---

## MiniStore Implementation

In `ministore/src/web_api.rs`, we established the complete web layer:

```rust
use crate::catalog::Catalog;
use crate::checkout::checkout;
use crate::error::StoreError;
use crate::models::{
    Coupon, Customer, Order, OrderId, PaymentMethod, Product, ProductCategory, ShoppingCart,
};
use crate::persistence::StorePersistence;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub catalog: Arc<RwLock<Catalog>>,
    pub orders: Arc<Mutex<Vec<Order>>>,
    pub persistence: Option<StorePersistence>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/products", get(list_products).post(create_product))
        .route("/api/products/{id}", get(get_product))
        .route("/api/orders", get(list_orders))
        .route("/api/orders/{id}", get(get_order))
        .route("/api/checkout", post(process_checkout))
        .with_state(state)
}
```

### The Transactional Checkout Endpoint

```rust
pub async fn process_checkout(
    State(state): State<AppState>,
    Json(payload): Json<CheckoutRequest>,
) -> Result<(StatusCode, Json<Order>), StoreError> {
    if payload.items.is_empty() {
        return Err(StoreError::EmptyCart);
    }

    let mut cart = ShoppingCart::new();
    let mut catalog_lock = state.catalog.write().await;

    // Fetch live prices from catalog
    for item in payload.items {
        let product = catalog_lock
            .find_by_id(item.product_id)
            .ok_or_else(|| StoreError::ProductNotFound {
                identifier: item.product_id.to_string(),
            })?;

        cart.add_item(product.id, item.quantity, product.price_cents);
    }

    // Execute core domain checkout logic
    let order = checkout(
        OrderId(payload.order_id),
        payload.customer,
        &mut cart,
        &mut catalog_lock,
        payload.payment,
        payload.coupon,
    )?;

    // Record order and persist changes
    {
        let mut orders_lock = state.orders.lock().await;
        orders_lock.push(order.clone());

        if let Some(ref persistence) = state.persistence {
            let _ = persistence.save_catalog(&catalog_lock);
            let _ = persistence.save_orders(&orders_lock);
        }
    }

    Ok((StatusCode::CREATED, Json(order)))
}
```

---

## Testing Your Code

Run the full test suite across both crates:

```bash
cargo test
```

Three dedicated web API integration tests verify the endpoints:
1. **`test_web_api_health_check`**: Validates `GET /health` returns `200 OK` with JSON `{"service":"ministore-api","status":"healthy","version":"1.0.0"}`.
2. **`test_web_api_products_flow`**: Validates creating a product via `POST /api/products` (`201 Created`), querying it by ID via `GET /api/products/201` (`200 OK`), and asserting that querying a nonexistent ID returns `404 Not Found`.
3. **`test_web_api_checkout_flow`**: Validates checking out an order over HTTP (`POST /api/checkout`), asserting stock decrement, checking `GET /api/orders`, and verifying that checking out an empty cart triggers `400 Bad Request`.

---

## Checkpoint & Summary

With Chapter 23 complete, MiniStore is no longer confined to a command-line terminal—it is a production-ready Web API capable of serving real clients worldwide! You have learned:
- How Axum, Tokio, and Tower form the modern asynchronous web stack in Rust.
- How type-safe extractors (`Path`, `Json`, `State`) eliminate boilerplate validation.
- How to manage shared mutable state concurrently using `Arc<RwLock<T>>` and `Arc<Mutex<T>>`.
- How to convert domain errors into clean HTTP status codes and JSON payloads with `IntoResponse`.
- How to test web APIs in memory using `tower::ServiceExt::oneshot`.

In Chapter 24, we will bring everything together into a robust production deployment architecture!
