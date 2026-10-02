# ২৩. রাস্টে ওয়েব এপিআই তৈরি (Building a Rust Web API with Axum)

## আপনি যা শিখবেন
- কেন বাস্তব জীবনের অ্যাপ্লিকেশনগুলোকে লোকাল সিএলআই (CLI) বাইনারির গণ্ডি পেরিয়ে **এইচটিটিপি ওয়েব এপিআই (HTTP Web API)** আকারে উন্মুক্ত করতে হয়।
- **অ্যাক্সাম (Axum)**, **টোকিও (Tokio)** এবং **টাওয়ার (Tower)** দিয়ে আধুনিক ও দ্রুতগতির রাস্ট ওয়েব অ্যাপ্লিকেশনের আর্কিটেকচার।
- কোনো জটিল ম্যাক্রো ছাড়া **`axum::Router`** এবং এইচটিটিপি মেথড হ্যান্ডলার (`get`, `post`) দিয়ে রাউটিং ডিজাইন করা।
- **টাইপ-সেফ এক্সট্রাক্টর্স (Type-Safe Extractors)**:
  - **`Path<T>`**: ডাইনামিক ইউআরএল প্যারামিটার পার্স করা (যেমন `/api/products/{id}`)।
  - **`Json<T>`**: রিকোয়েস্ট বডির JSON পে-লোডকে সরাসরি টাইপড রাস্ট স্ট্রাক্টে ডিসিরিয়ালাইজ ও ভ্যালিডেট করা।
  - **`State<T>`**: বিভিন্ন হ্যান্ডলারের মাঝে থ্রেড-সেফ শেয়ার্ড স্টেট ইনজেক্ট করা।
- **টাইপ-সেফ এইচটিটিপি রেসপন্স**:
  - ডোমেন এররের জন্য **`IntoResponse`** ট্রেইট ইমপ্লিমেন্ট করা (`StoreError` ──► HTTP Status Codes + JSON Error)।
  - সঠিক এইচটিটিপি স্ট্যাটাস কোড প্রদান (`StatusCode::OK`, `StatusCode::CREATED`, `StatusCode::NOT_FOUND`, `StatusCode::BAD_REQUEST`)।
- কনকারেন্ট অ্যাক্সেসের জন্য **`Arc<RwLock<Catalog>>`** এবং **`Arc<Mutex<Vec<Order>>>`** দিয়ে মেমরি সুরক্ষা নিশ্চিত করা।
- কোনো নেটওয়ার্ক পোর্ট বা সকেট বাইন্ডিং ছাড়া **`tower::ServiceExt::oneshot`** ব্যবহার করে ইন-মেমরি এপিআই টেস্টিং করা।
- মিনিস্টোরের পূর্ণাঙ্গ ওয়েব এপিআই তৈরি:
  - সিস্টেম হেলথ চেক (`GET /health`)।
  - ক্যাটালগ কোয়েরি ও নতুন প্রোডাক্ট তৈরি (`GET /api/products`, `POST /api/products`, `GET /api/products/{id}`)।
  - অর্ডার ইতিহাস অনুসন্ধান (`GET /api/orders`, `GET /api/orders/{id}`)।
  - ট্রানজ্যাকশনাল অর্ডার চেকআউট এপিআই (`POST /api/checkout`)।

---

## আমাদের এটি কেন প্রয়োজন?

২২তম অধ্যায় পর্যন্ত মিনিস্টোর একটি চমৎকার একক কম্পিউটার প্রোগ্রাম হিসেবে কাজ করেছে। কিন্তু যখন কোনো ইউজার `cargo run` দেয়, প্রোগ্রামটি তার লোকাল টার্মিনালে চালু হয়ে কাজ শেষ করে প্রস্থান করে।

বাস্তব জগতের আধুনিক ই-কমার্স এভাবে চলে না। একটি স্টোরকে একই সাথে সেবা দিতে হয়:
- রিঅ্যাক্ট (React), ভিউ (Vue) কিংবা সভেল্ট (Svelte) দিয়ে তৈরি ফ্রন্টএন্ড ওয়েব স্টোরফ্রন্টকে।
- অ্যান্ড্রয়েড ও আইওএস (iOS) মোবাইল অ্যাপ্লিকেশনের কোটি কোটি গ্রাহককে।
- পেমেন্ট গেটওয়ের কলব্যাক এবং কুরিয়ার সার্ভিসের ট্র্যাকিং ওয়েবহুককে।
- তৃতীয় পক্ষের ইনভেন্টরি ও ইআরপি সিস্টেমকে।

```
সিএলআই মডেল (একক মেশিন, বিচ্ছিন্ন):
[টার্মিনাল] ──► cargo run ──► মেমরি/ফাইলে কাজ শেষ ──► প্রস্থান

এইচটিটিপি ওয়েব এপিআই মডেল (গ্লোবাল ও ডিস্ট্রিবিউটেড):
[ওয়েব ব্রাউজার] ──┐
[মোবাইল অ্যাপ]  ──┼──► HTTP রিকোয়েস্ট (GET/POST) ──► [অ্যাক্সাম সার্ভার] ──► মিনিস্টোর কোর
[পার্টনার এপিআই] ──┘                                      │
                                                   JSON রেসপন্স
```

মিনিস্টোরকে গোটা বিশ্বের সাথে সংযুক্ত করতে আমাদের একটি **ওয়েব এপিআই (Web API)** প্রয়োজন: এমন একটি সার্ভিস যা ব্যাকগ্রাউন্ডে সারাক্ষণ নেটওয়ার্ক রিকোয়েস্টের অপেক্ষায় থাকবে, ক্লায়েন্টের পাঠানো JSON ডাটা পার্স করবে, বিজনেস লজিক চালাবে এবং প্রমিত এইচটিটিপি স্ট্যাটাস কোডসহ JSON রেসপন্স ফেরত পাঠাবে।

---

## অ্যাক্সাম (Axum) কেন?

রাস্ট ইকোসিস্টেমে বেশ কয়েকটি ওয়েব ফ্রেমওয়ার্ক রয়েছে (যেমন Actix Web, Rocket, Warp)। তবে অফিসিয়াল টোকিও (Tokio) টিমের তৈরি **অ্যাক্সাম (Axum)** আজ গোটা কমিউনিটিতে সবচেয়ে আধুনিক ও জনপ্রিয় মানদণ্ড হিসেবে প্রতিষ্ঠিত। এর তিনটি প্রধান কারণ:

১. **টোকিও-নেটিভ আর্কিটেকচার**: অ্যাক্সাম সরাসরি টোকিও (Tokio async runtime), হাইপার (Hyper HTTP engine) এবং টাওয়ারের (Tower middleware) উপর ভিত্তি করে নির্মিত। ফলে কোনো এক্সটার্নাল অ্যাডাপ্টার ছাড়াই সর্বোচ্চ পারফরম্যান্স নিশ্চিত হয়।
২. **ম্যাক্রো-মুক্ত পরিষ্কার কোড**: জাভা (Spring) বা পাইথন (FastAPI)-এর মতো ফ্রেমওয়ার্কে মেথডের উপরে জটিল অ্যানোটেশন বা ডেকোরেটরের ওপর নির্ভর করতে হয়। অ্যাক্সাম কোনো অ্যানোটেশন ছাড়াই সাধারণ রাস্ট ফাংশন ও ট্রেইট দিয়ে কাজ করে।
৩. **টাইপ-সেফ এক্সট্রাক্টর (Type-Safe Extractors)**: হ্যান্ডলার ফাংশনের আর্গুমেন্টে আপনি যে প্যারামিটারগুলো দেবেন (যেমন `Json<T>`, `Path<id>`, `State`), অ্যাক্সাম স্বয়ংক্রিয়ভাবে সেগুলোকে রিকোয়েস্ট থেকে পার্স ও ভ্যালিডেট করে নেয়। ডাটা ভুল থাকলে অ্যাক্সাম নিজেই যথাযথ এরর রেসপন্স পাঠিয়ে দেয়।

```
                   টাইপ-সেফ এক্সট্রাক্টর:
HTTP রিকোয়েস্ট ──► Path(id), Json(body), State(state) ──► হ্যান্ডলার async fn
                                                                │
                                                                ▼
HTTP রেসপন্স  ◄── IntoResponse: (StatusCode, Json(data)) ◄──────┘
```

`Cargo.toml`-এ অ্যাক্সাম ও টাওয়ার যোগ করুন:
```toml
[dependencies]
axum = "0.8"
tower = { version = "0.5", features = ["util"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

## রাস্টে ওয়েব এপিআই-এর মূলনীতি ও গঠন

### ১. অ্যাক্সাম রাউটার ও হ্যান্ডলার ফাংশন

অ্যাক্সামে প্রতিটি রাউট হ্যান্ডলার হলো একটি সাধারণ অ্যাসিঙ্ক ফাংশন (`async fn`):

```rust
use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "healthy" }))
}

let app = Router::new().route("/health", get(health_check));
```

হ্যান্ডলারগুলো আর্গুমেন্ট হিসেবে বিভিন্ন এক্সট্রাক্টর গ্রহণ করতে পারে এবং এমন যেকোনো টাইপ রিটার্ন করতে পারে যা `IntoResponse` ট্রেইট ইমপ্লিমেন্ট করে।

---

### ২. এক্সট্রাক্টরস: `Path`, `Json`, এবং `State`

ইনকামিং এইচটিটিপি রিকোয়েস্ট থেকে প্রয়োজনীয় তথ্য বের করে আনার উপায় হলো এক্সট্রাক্টর:

- **`Path<T>`**: ইউআরএল পাথের ডায়নামিক অংশ পার্স করে (যেমন `{id}`):
  ```rust
  async fn get_product(Path(id): Path<u64>) -> String {
      format!("প্রোডাক্ট আইডি: {id}")
  }
  ```
- **`Json<T>`**: রিকোয়েস্ট বডির JSON পে-লোডকে সরাসরি টাইপড স্ট্রাক্ট `T`-তে ডিসিরিয়ালাইজ করে:
  ```rust
  async fn create_product(Json(payload): Json<CreateProductRequest>) -> StatusCode {
      println!("নতুন প্রোডাক্ট তৈরি: {}", payload.sku);
      StatusCode::CREATED
  }
  ```
- **`State<AppState>`**: ডাটাবেস সংযোগ, ক্যাটালগ বা কনকারেন্ট স্টেট হ্যান্ডলারের ভেতর ইনজেক্ট করে।

---

### ৩. শেয়ার্ড স্টেট: `Arc<RwLock<T>>` বনাম `Arc<Mutex<T>>`

মাল্টি-থ্রেডেড ওয়েব সার্ভারে একসাথে শত শত রিকোয়েস্ট টোকিও র্ওকার থ্রেডগুলোতে প্যারালালে এক্সিকিউট হয়।

রেস কন্ডিশন ছাড়া শেয়ার্ড ডাটা রিড ও রাইট করার জন্য আমরা স্মার্ট পয়েন্টার ব্যবহার করি:

```rust
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Clone)]
pub struct AppState {
    // RwLock একই সাথে শত শত ক্লায়েন্টকে ক্যাটালগ ব্রাউজ (Read) করতে দেয়,
    // কিন্তু নতুন প্রোডাক্ট যোগ করার সময় একজনকেই এক্সক্লুসিভ (Write) অনুমতি দেয়
    pub catalog: Arc<RwLock<Catalog>>,
    
    // Mutex অর্ডারের লিস্টে নতুন অর্ডার যোগ করার জন্য মিউচুয়াল এক্সক্লুশন দেয়
    pub orders: Arc<Mutex<Vec<Order>>>,
}
```

> **কেন `std::sync::RwLock`-এর বদলে `tokio::sync::RwLock`?**
> অ্যাসিঙ্ক হ্যান্ডলারে কোনো লকের ভেতর `.await` থাকলে সাধারণ সিনক্রোনাস লক ওএস থ্রেডকে ব্লক করে ফেলে, যার ফলে সার্ভার স্লো হয়ে যায়। টোকিওর অ্যাসিঙ্ক লক কোঅপারেটিভ পদ্ধতিতে লক হ্যান্ডেল করে, ফলে থ্রেড স্টারভেশন ঘটে না।

---

### ৪. `IntoResponse` দিয়ে কাস্টম এরর হ্যান্ডলিং

প্রতিটি হ্যান্ডলারে বারবার ম্যানুয়ালি এইচটিটিপি স্ট্যাটাস কোড লেখার বদলে রাস্টের সেরা অভ্যাস হলো ডোমেন এররের জন্য **`IntoResponse`** ট্রেইট ইমপ্লিমেন্ট করা:

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

এখন যেকোনো হ্যান্ডলার সরাসরি `Result<Json<Product>, StoreError>` রিটার্ন করতে পারে। ফাংশনটি যদি `Err(StoreError::ProductNotFound)` রিটার্ন করে, অ্যাক্সাম নিজে থেকেই `404 Not Found` স্ট্যাটাস কোড ও JSON বডি ক্লায়েন্টকে পাঠিয়ে দেবে!

---

### ৫. সকেট পোর্ট ছাড়া ইন-মেমরি এপিআই টেস্টিং

টেস্টিংয়ের সময় লোকালহোস্টে `127.0.0.1:3000` সার্ভার চালু করলে পোর্ট অকুপাইড এরর বা ফায়ারওয়াল পারমিশনের ঝামেলা হতে পারে।

অ্যাক্সাম রাউটার যেহেতু টাওয়ারের **`Service<Request>`** ট্রেইট ইমপ্লিমেন্ট করে, তাই আমরা **`tower::ServiceExt::oneshot`** দিয়ে সরাসরি মেমরির ভেতরেই নকল এইচটিটিপি রিকোয়েস্ট পাঠিয়ে যাচাই করতে পারি:

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

এটি ইন-মেমরি গতিতে চলে—কয়েক মিলিমেকেন্ডে শত শত পূর্ণাঙ্গ এন্ডপয়েন্ট টেস্ট শেষ হয়ে যায়!

---

## মিনিস্টোরে ওয়েব এপিআই ইমপ্লিমেন্টেশন

`ministore/src/web_api.rs`-এ তৈরি করা হয়েছে সম্পূর্ণ ওয়েব লেয়ার:

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

### ট্রানজ্যাকশনাল চেকআউট এন্ডপয়েন্ট

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

    // ক্যাটালগ থেকে সর্বশেষ মূল্য সংগ্রহ করে কার্ট তৈরি
    for item in payload.items {
        let product = catalog_lock
            .find_by_id(item.product_id)
            .ok_or_else(|| StoreError::ProductNotFound {
                identifier: item.product_id.to_string(),
            })?;

        cart.add_item(product.id, item.quantity, product.price_cents);
    }

    // কোর ডোমেন চেকআউট সম্পন্ন
    let order = checkout(
        OrderId(payload.order_id),
        payload.customer,
        &mut cart,
        &mut catalog_lock,
        payload.payment,
        payload.coupon,
    )?;

    // অর্ডার সংরক্ষণ ও পারসিস্টেন্সে সিঙ্ক
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

## কোড পরীক্ষা ও ভ্যালিডেশন

সম্পূর্ণ টেস্ট স্যুট রান করুন:

```bash
cargo test
```

ওয়েব এপিআই-এর জন্য তিনটি প্রধান ইন্টিগ্রেশন টেস্ট সংযুক্ত রয়েছে:
1. **`test_web_api_health_check`**: `GET /health` রিকোয়েস্ট `200 OK` এবং হেলথ পে-লোড দিচ্ছে কিনা তা নিশ্চিত করে।
2. **`test_web_api_products_flow`**: `POST /api/products` দিয়ে পণ্য তৈরি (`201 Created`), `GET /api/products/201` দিয়ে তা কোয়েরি করা (`200 OK`), এবং অস্তিত্বহীন পণ্যে `404 Not Found` রিটার্ন হওয়া যাচাই করে।
3. **`test_web_api_checkout_flow`**: এইচটিটিপির মাধ্যমে অর্ডার চেকআউট করা (`POST /api/checkout`), স্টক কমা, `GET /api/orders`-এ অর্ডার তালিকাভুক্ত হওয়া এবং খালি কার্টে `400 Bad Request` আসার সঠিকতা পরীক্ষা করে।

---

## সারসংক্ষেপ ও চেকপয়েন্ট

২৩তম অধ্যায় সম্পন্ন করার মাধ্যমে মিনিস্টোর এখন আর শুধুমাত্র একটি লোকাল টার্মিনাল অ্যাপ্লিকেশনে সীমাবদ্ধ নয়—এটি বিশ্বমানের ওয়েব এপিআই-এ পরিণত হয়েছে। আপনি শিখেছেন:
- কীভাবে অ্যাক্সাম, টোকিও এবং টাওয়ার মিলে শক্তিশালী অ্যাসিঙ্ক্রোনাস ওয়েব আর্কিটেকচার গঠন করে।
- টাইপ-সেফ এক্সট্রাক্টরের (`Path`, `Json`, `State`) মাধ্যমে বয়লারপ্লেট কোড পরিহার করা।
- `Arc<RwLock<T>>` এবং `Arc<Mutex<T>>` দিয়ে নিরাপদে সমান্তরাল শেয়ার্ড স্টেট পরিচালনা।
- `IntoResponse` ট্রেইট দিয়ে ডোমেন এররকে পরিষ্কার এইচটিটিপি স্ট্যাটাস ও JSON এররে রূপান্তর।
- সকেট ছাড়াই `tower::ServiceExt::oneshot` দিয়ে দ্রুততম ইন-মেমরি এপিআই টেস্টিং।

পরবর্তী অধ্যায়ে আমরা মিনিস্টোরকে প্রোডাকশন-রেডি আর্কিটেকচার এবং ডিপ্লয়মেন্ট কনফিগারেশনের জন্য প্রস্তুত করব!
