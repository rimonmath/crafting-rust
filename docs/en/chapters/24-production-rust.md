# Chapter 24: Production Rust

## What You'll Learn
- The critical differences between a development prototype and a **production-ready backend service**.
- Managing 12-factor application settings with **type-safe configuration (`AppConfig`)** and environment variable overrides.
- Why standard `println!` statements are an anti-pattern in production, and how **Structured Logging & Tracing (`tracing`)** revolutionizes observability.
- The distinction between **Events** (points in time) and **Spans** (durations of time with contextual fields).
- Monitoring HTTP latency and request lifecycles automatically with **`tower_http::trace::TraceLayer`**.
- Implementing enterprise health probes: **Liveness (`/health/live`)** vs. **Readiness (`/health/ready`)**.
- Handling **Graceful Shutdowns** via OS signals (`SIGINT` and `SIGTERM`) to ensure in-flight transactions and storage flushes complete cleanly.
- The complete production-grade MiniStore backend architecture.

---

## Why Do We Need This?

In Chapter 23, we successfully connected MiniStore to the world via an Axum HTTP Web API. However, if you deploy that code directly to a production server or a container cluster (like Docker or Kubernetes), several critical problems emerge:

### 1. Hardcoded Configuration
In development, binding to `127.0.0.1:3000` and writing to a local `"data"` folder works fine. But in production:
- Containers often must bind to `0.0.0.0` or dynamic cloud ports (e.g., AWS ECS, Fly.io, or Heroku `$PORT`).
- Storage paths must point to mounted persistent volumes (e.g., `/var/data/ministore`).
- Staging and Production environments require different logging verbosities and security profiles.

### 2. The `println!` Blind Spot
Standard `println!` is synchronous and unindexed:
- It blocks the current thread when writing to stdout under heavy load.
- It produces unstructured strings without timestamps, machine-readable log levels (`INFO`, `WARN`, `ERROR`), or correlation IDs.
- Log aggregation systems (Datadog, Grafana Loki, AWS CloudWatch) cannot efficiently query, filter, or alert on raw standard output.

```
Raw stdout (Unstructured / Fragile):
MiniStore started on 127.0.0.1:3000
Order placed 901 Margaret Hamilton 14150

Structured Tracing (Production-Grade JSON/Spans):
{"timestamp":"2026-09-28T16:55:08Z","level":"INFO","target":"ministore","env":"production","order_id":901,"customer":"Margaret Hamilton","amount_cents":14150,"message":"Order checkout completed"}
```

### 3. Abrupt Termination & Data Corruption
When Kubernetes scales down a pod or you restart a server with `kill` (`SIGTERM`) or `Ctrl+C` (`SIGINT`):
- Without graceful shutdown, the operating system instantly kills the process.
- Clients in the middle of a checkout transaction receive sudden `Connection reset by peer` errors.
- Unflushed disk buffers or active JSON writes can leave files corrupted.

---

## Core Production Concepts

### 1. Type-Safe Configuration (`AppConfig`)

The **Twelve-Factor App** methodology dictates that configuration should be strictly separated from code and read from environment variables.

In Rust, we model this as a strongly typed struct with sensible defaults:

```rust
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Environment {
    #[default]
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "prod" | "production" => Self::Production,
            "stage" | "staging" => Self::Staging,
            _ => Self::Development,
        }
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Staging => "staging",
            Self::Production => "production",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub data_dir: PathBuf,
    pub log_level: String,
    pub env: Environment,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: String::from("127.0.0.1"),
            port: 3000,
            data_dir: PathBuf::from("data"),
            log_level: String::from("info"),
            env: Environment::Development,
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        let default = Self::default();

        let host = env::var("MINISTORE_HOST").unwrap_or(default.host);
        let port = env::var("MINISTORE_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(default.port);
        let data_dir = env::var("MINISTORE_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or(default.data_dir);
        let log_level = env::var("MINISTORE_LOG").unwrap_or(default.log_level);
        let env = env::var("MINISTORE_ENV")
            .map(|e| Environment::from_str_loose(&e))
            .unwrap_or(default.env);

        Self {
            host,
            port,
            data_dir,
            log_level,
            env,
        }
    }

    pub fn socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.host, self.port).parse()
    }
}
```

---

### 2. Structured Telemetry with `tracing`

In asynchronous Rust, standard logging is insufficient because multiple asynchronous tasks execute concurrently on worker threads. If thread A prints `"Processing checkout"`, thread B prints `"Product created"`, and thread A prints `"Payment received"`, the logs are interleaved and incomprehensible.

The **`tracing`** ecosystem introduces two fundamental abstractions:

1. **Spans**: Represent a continuous span of time (such as handling a single HTTP request or a checkout workflow). Spans have a name, beginning, end, and attached key-value context.
2. **Events**: Represent a single point in time within a span (like an error or a milestone), inheriting all context from active enclosing spans.

```rust
// Initializing subscriber with environment-controlled filtering:
tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::new(&config.log_level))
    .init();

// Emitting structured diagnostic events:
tracing::info!(
    order_id = %order.order_id,
    customer = %order.customer.name,
    amount_cents = order.total_cents(),
    "Order checkout completed"
);
```

#### Automatic HTTP Telemetry via `TraceLayer`
By wrapping our Axum router with `TraceLayer`, incoming requests, latency, status codes, and HTTP method details are automatically instrumented:

```rust
let router = Router::new()
    // ... routes ...
    .layer(tower_http::trace::TraceLayer::new_for_http())
    .with_state(state);
```

---

### 3. Enterprise Health Probes: Liveness vs. Readiness

In modern orchestrators like Kubernetes, container health is monitored via automated probes:

1. **Liveness Probe (`GET /health/live`)**:
   - Question: *Is the application process alive and responsive?*
   - If this fails, the orchestrator restarts the container.
   - It should perform minimal work (just return `200 OK`) to avoid triggering false-positive restarts during heavy load.
2. **Readiness Probe (`GET /health/ready`)**:
   - Question: *Is the application ready to handle user traffic?*
   - If this fails, the orchestrator temporarily removes the pod from the load balancer without killing the container.
   - In MiniStore, the readiness probe validates that persistent storage is accessible and writable.

```rust
pub async fn health_liveness() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "alive" }))
}

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
            Json(serde_json::json!({ "status": "ready", "storage": "accessible" })),
        )
    } else {
        tracing::error!("Readiness check failed: storage directory inaccessible");
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "status": "not_ready", "storage": "inaccessible" })),
        )
    }
}
```

---

### 4. Graceful Shutdown

When an operator stops the application or Kubernetes drains a node, the container receives a termination signal:
- `SIGINT` (when the developer hits `Ctrl+C` in the terminal).
- `SIGTERM` (when Docker or Kubernetes commands a container to stop).

A graceful shutdown ensures that:
1. The server stops accepting new incoming connections.
2. All in-flight HTTP requests are allowed to complete.
3. Active database transactions or storage snapshots are committed to disk.

```rust
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
```

Binding the graceful shutdown listener to an Axum server:
```rust
let listener = tokio::net::TcpListener::bind(config.socket_addr()?).await?;
tracing::info!("MiniStore server listening on {}", config.socket_addr()?);

axum::serve(listener, router)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
```

---

## Testing Your Code

Run the full validation suite across the workspace:

```bash
cargo test
```

Key test scenarios:
1. **`test_app_config_defaults_and_env`**: Verifies that default configuration values are properly set, custom environment mappings parse accurately, and host/port combinations parse into valid `SocketAddr` objects.
2. **`test_web_api_liveness_and_readiness_probes`**: Dispatches simulated in-memory requests to `/health/live` and `/health/ready`, asserting `200 OK` and verifying storage accessibility checks.
3. **HTTP Web API Endpoints**: All existing CRUD and transactional checkout tests run under the new `TraceLayer` middleware seamlessly.

---

## Checkpoint & Summary

With Chapter 24 complete, MiniStore has evolved into an enterprise-ready backend application. You have mastered:
- 12-Factor type-safe configuration parsing with environment overrides.
- Asynchronous structured observability with `tracing` and `TraceLayer`.
- Designing production health probes (`/health/live` and `/health/ready`).
- Implementing graceful shutdowns across OS signals (`SIGINT` / `SIGTERM`).

MiniStore is now robust, resilient, and ready for deployment in any cloud container environment!
