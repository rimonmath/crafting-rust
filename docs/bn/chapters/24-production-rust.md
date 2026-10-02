# ২৪. প্রোডাকশন রাস্ট (Production Rust)

## আপনি যা শিখবেন
- ডেভেলপমেন্ট প্রোটোটাইপ এবং একটি **প্রোডাকশন-রেডি ব্যাকএন্ড সার্ভিসের** মধ্যকার মৌলিক পার্থক্য।
- **টাইপ-সেফ কনফিগারেশন (`AppConfig`)** এবং এনভায়রনমেন্ট ভেরিয়েবল ওভাররাইড দিয়ে 12-ফ্যাক্টর অ্যাপ্লিকেশন তৈরি করা।
- প্রোডাকশনে সাধারণ `println!` কেন ক্ষতিকর (অ্যান্টি-প্যাটার্ন) এবং **স্ট্রাকচার্ড লগিং ও ট্রেসিং (`tracing`)** কীভাবে পর্যবেক্ষণে বৈপ্লবিক পরিবর্তন আনে।
- **ইভেন্ট (Events)** (সময়ের নির্দিষ্ট মুহূর্ত) এবং **স্প্যান (Spans)** (কনটেক্সচুয়াল ফিল্ডসহ সময়ের বিস্তার)-এর মধ্যকার পার্থক্য।
- **`tower_http::trace::TraceLayer`** দিয়ে স্বয়ংক্রিয়ভাবে এইচটিটিপি রিকোয়েস্ট লেটেন্সি ও লাইফসাইকেল পর্যবেক্ষণ করা।
- এন্টারপ্রাইজ গ্রেড হেলথ প্রোব তৈরি: **লাইভনেস (`/health/live`)** বনাম **রেডিনেস (`/health/ready`)**।
- ওএস সিগন্যাল (`SIGINT` এবং `SIGTERM`) ইন্টারসেপ্ট করে **গ্রেসফুল শাটডাউন (Graceful Shutdown)** নিশ্চিত করা, যাতে চলমান রিকোয়েস্ট ও ফাইল রাইট ড্রপ না হয়।
- মিনিস্টোরের পূর্ণাঙ্গ প্রোডাকশন-গ্রেড ব্যাকএন্ড আর্কিটেকচার।

---

## আমাদের এটি কেন প্রয়োজন?

২৩তম অধ্যায়ে আমরা অ্যাক্সাম (Axum) দিয়ে মিনিস্টোরের জন্য একটি কার্যকরী এইচটিটিপি ওয়েব এপিআই তৈরি করেছি। কিন্তু সেই কোডটিকে যদি সরাসরি কোনো প্রোডাকশন সার্ভার বা ক্লাউড ক্লাস্টারে (যেমন ডকার বা কুবারনেটিস) ডেপ্লয় করা হয়, তবে বেশ কিছু গুরুতর সমস্যার সৃষ্টি হবে:

### ১. হার্ডকোডেড কনফিগারেশন (Hardcoded Configuration)
লোকাল ডেভেলপমেন্টের সময় `127.0.0.1:3000` অ্যাড্রেসে বাইন্ড করা কিংবা লোকাল `"data"` ফোল্ডারে ফাইল রাখা স্বাভাবিক। কিন্তু প্রোডাকশন পরিবেশে:
- কন্টেইনারগুলোকে সাধারণত `0.0.0.0` অ্যাড্রেসে বাইন্ড করতে হয় কিংবা ক্লাউড প্ল্যাটফর্মের ডাইনামিক পোর্ট (যেমন AWS ECS, Fly.io বা Heroku-এর `$PORT`) মেনে চলতে হয়।
- ডাটা ডিরেক্টরি অবশ্যই মাউন্টেড পারসিস্টেন্ট ভলিউম (যেমন `/var/data/ministore`)-এ নির্দেশ করতে হয়।
- স্টেজিং (Staging) ও প্রোডাকশন (Production) এনভায়রনমেন্টে আলাদা আলাদা লগ লেভেল ও সিকিউরিটি কনফিগারেশন প্রয়োজন হয়।

### ২. `println!`-এর অন্ধবিন্দু (The `println!` Blind Spot)
সাধারণ `println!` সিঙ্ক্রোনাস এবং আনস্ট্রাকচার্ড:
- অতিরিক্ত ট্রাফিকের চাপে টার্মিনাল বা stdout-এ রাইট করার সময় এটি থ্রেড ব্লক করে দেয়।
- এতে কোনো টাইমস্ট্যাম্প, লগ লেভেল (`INFO`, `WARN`, `ERROR`), বা রিকোয়েস্ট ট্র্যাকিং আইডি থাকে না।
- ক্লাউড লগ সিস্টেমগুলো (যেমন Datadog, Grafana Loki, AWS CloudWatch) সাধারণ টেক্সট থেকে কুয়েরি বা অ্যালার্ট তৈরি করতে পারে না।

```
সাধারণ stdout (আনস্ট্রাকচার্ড / ভঙ্গুর):
MiniStore started on 127.0.0.1:3000
Order placed 901 Margaret Hamilton 14150

স্ট্রাকচার্ড ট্রেসিং (প্রোডাকশন-গ্রেড JSON / Spans):
{"timestamp":"2026-09-28T16:55:08Z","level":"INFO","target":"ministore","env":"production","order_id":901,"customer":"Margaret Hamilton","amount_cents":14150,"message":"Order checkout completed"}
```

### ৩. হঠাৎ বন্ধ হয়ে যাওয়া ও ডাটা করাপশন (Abrupt Termination)
যখন কুবারনেটিস কোনো কন্টেইনার বন্ধ করে বা কোনো অপারেটর `kill` (`SIGTERM`) অথবা টার্মিনালে `Ctrl+C` (`SIGINT`) চাপেন:
- কোনো গ্রেসফুল শাটডাউন হ্যান্ডলার না থাকলে অপারেটিং সিস্টেম তাৎক্ষণিকভাবে প্রসেসটি মেরে ফেলে।
- যে ক্লায়েন্ট সেই মুহূর্তে চেকআউট রিকোয়েস্ট পাঠাচ্ছিল, সে হঠাৎ `Connection reset by peer` এরর পায়।
- ডিস্কের রাইট বাফার বা আংশিক সম্পন্ন JSON ফাইল করাপ্ট হয়ে যেতে পারে।

---

## প্রোডাকশন রেডি রাস্টের মূল ধারণাসমূহ

### ১. টাইপ-সেফ কনফিগারেশন (`AppConfig`)

**ট্যুয়েলভ-ফ্যাক্টর অ্যাপ (12-Factor App)** আর্কিটেকচার অনুযায়ী কনফিগারেশনকে কোড থেকে আলাদা রাখতে হয় এবং এনভায়রনমেন্ট ভেরিয়েবল থেকে পড়তে হয়।

রাস্টে আমরা এটিকে একটি টাইপ-সেফ স্ট্রাক্ট এবং সংবেদনশীল ডিফল্ট মানের সমন্বয়ে রূপান্তর করি:

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

### ২. `tracing` দিয়ে স্ট্রাকচার্ড টেলিমেট্রি

অ্যাসিঙ্ক্রোনাস রাস্টে সাধারণ লগিং অনুপযুক্ত, কারণ একাধিক অ্যাসিঙ্ক টাস্ক একই সাথে বিভিন্ন থ্রেডে এক্সিকিউট হয়। যদি টাস্ক ক প্রিন্ট করে `"Processing checkout"`, টাস্ক খ প্রিন্ট করে `"Product created"`, এবং আবার টাস্ক ক প্রিন্ট করে `"Payment received"`, তবে লগলাইনে এগুলো জগাখিচুড়ি হয়ে যায়।

**`tracing`** ইকোসিস্টেমে দুটি প্রধান ধারণা রয়েছে:

১. **স্প্যান (Spans)**: একটি সময়ের বিস্তারকে নির্দেশ করে (যেমন একটি সম্পূর্ণ এইচটিটিপি রিকোয়েস্ট বা চেকআউট ট্রানজ্যাকশন)। স্প্যানের একটি নাম, শুরু, শেষ এবং সাথে সংযুক্ত কী-ভ্যালু কনটেক্সট থাকে।
২. **ইভেন্ট (Events)**: স্প্যানের অভ্যন্তরে সময়ের একটি নির্দিষ্ট মুহূর্তকে নির্দেশ করে (যেমন কোনো মাইলফলক বা এরর), যা প্যারেন্ট স্প্যানের কনটেক্সট স্বয়ংক্রিয়ভাবে ধারণ করে।

```rust
// এনভায়রনমেন্ট ফিল্টারসহ সাবস্ক্রাইবার ইনিশিয়ালাইজেশন:
tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::new(&config.log_level))
    .init();

// স্ট্রাকচার্ড ডায়াগনস্টিক ইভেন্ট তৈরি:
tracing::info!(
    order_id = %order.order_id,
    customer = %order.customer.name,
    amount_cents = order.total_cents(),
    "Order checkout completed"
);
```

#### `TraceLayer` দিয়ে স্বয়ংক্রিয় এইচটিটিপি টেলিমেট্রি
অ্যাক্সাম রাউটারে টাওয়ারের `TraceLayer` যুক্ত করলে প্রতিটি ইনকামিং রিকোয়েস্টের পাথ, লেটেন্সি, স্ট্যাটাস কোড এবং মেথড স্বয়ংক্রিয়ভাবে লগ হয়ে যায়:

```rust
let router = Router::new()
    // ... routes ...
    .layer(tower_http::trace::TraceLayer::new_for_http())
    .with_state(state);
```

---

### ৩. এন্টারপ্রাইজ হেলথ প্রোবস: লাইভনেস বনাম রেডিনেস

কুবারনেটিসের মতো আধুনিক কন্টেইনার অর্কেস্ট্রেটরে অ্যাপ্লিকেশনের স্বাস্থ্য স্বয়ংক্রিয়ভাবে দুটি প্রোবের মাধ্যমে যাচাই করা হয়:

১. **লাইভনেস প্রোব (`GET /health/live`)**:
   - প্রশ্ন: *অ্যাপ্লিকেশন প্রসেসটি কি এখনো বেঁচে আছে এবং সাড়া দিচ্ছে?*
   - এটি ব্যর্থ হলে অর্কেস্ট্রেটর প্রসেসটি রিস্টার্ট করে।
   - এটি সবসময় হালকা হওয়া উচিত (সাধারণত `200 OK` রিটার্ন করে) যাতে অতিরিক্ত লোডের কারণে ভুলবশত পড রিস্টার্ট না হয়ে যায়।
২. **রেডিনেস প্রোব (`GET /health/ready`)**:
   - প্রশ্ন: *অ্যাপ্লিকেশনটি কি এই মুহূর্তে ইউজার ট্রাফিক নেওয়ার জন্য প্রস্তুত?*
   - এটি ব্যর্থ হলে অর্কেস্ট্রেটর পডটিকে কিল না করে লোড ব্যালেন্সার থেকে সাময়িকভাবে সরিয়ে রাখে।
   - মিনিস্টোরে রেডিনেস প্রোব যাচাই করে যে পারসিস্টেন্ট স্টোরেজ ফোল্ডারটি অ্যাক্সেসিবল এবং রাইটযোগ্য কি না।

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

### ৪. গ্রেসফুল শাটডাউন (Graceful Shutdown)

যখন কোনো কন্টেইনার বন্ধ করা হয়, তখন অপারেটিং সিস্টেম টার্মিনেশন সিগন্যাল পাঠায়:
- `SIGINT` (যখন ডেভেলপার টার্মিনালে `Ctrl+C` চাপেন)।
- `SIGTERM` (যখন ডকার বা কুবারনেটিস কন্টেইনার স্টপ করার নির্দেশ দেয়)।

গ্রেসফুল শাটডাউন নিশ্চিত করে যে:
১. সার্ভার তাৎক্ষণিকভাবে নতুন কানেকশন নেওয়া বন্ধ করবে।
২. ইতিমধ্যে প্রক্রিয়াধীন থাকা (in-flight) রিকোয়েস্টগুলোকে নির্বিঘ্নে শেষ হতে দেবে।
৩. চলমান ডাটাবেস ট্রানজ্যাকশন বা পারসিস্টেন্স ফাইলগুলো সুরক্ষিতভাবে ডিস্কে ফ্লাশ করবে।

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

অ্যাক্সাম সার্ভারের সাথে গ্রেসফুল শাটডাউন লিসেনার যুক্ত করা:
```rust
let listener = tokio::net::TcpListener::bind(config.socket_addr()?).await?;
tracing::info!("MiniStore server listening on {}", config.socket_addr()?);

axum::serve(listener, router)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
```

---

## কোড টেস্ট করা (Testing Your Code)

পুরো টেস্ট স্যুট রান করতে টার্মিনালে চালান:

```bash
cargo test
```

প্রধান টেস্ট সিনারিওসমূহ:
১. **`test_app_config_defaults_and_env`**: ডিফল্ট কনফিগারেশন মান, কাস্টম এনভায়রনমেন্ট ভেরিয়েবল পার্সিং এবং সঠিক `SocketAddr` তৈরি যাচাই করে।
২. **`test_web_api_liveness_and_readiness_probes`**: `/health/live` এবং `/health/ready` পাথে ইন-মেমরি রিকোয়েস্ট পাঠিয়ে `200 OK` ও স্টোরেজ ভ্যালিডেশন নিশ্চিত করে।
৩. **এইচটিটিপি ওয়েব এপিআই টেস্ট**: বিদ্যমান সকল সিআরইউডি ও চেকআউট টেস্ট নতুন `TraceLayer` মিডলওয়্যারের সাথে কোনো সমস্যা ছাড়াই নির্বিঘ্নে পাস করে।

---

## চেকপয়েন্ট ও সারসংক্ষেপ

২৪তম অধ্যায় সমাপ্তির মধ্য দিয়ে মিনিস্টোর একটি পূর্ণাঙ্গ এন্টারপ্রাইজ-গ্রেড ব্যাকএন্ড সার্ভিসে রূপান্তরিত হয়েছে। আপনি সফলভাবে শিখেছেন:
- 12-ফ্যাক্টর অ্যাপ্লিকেশন অনুযায়ী টাইপ-সেফ কনফিগারেশন ম্যানেজমেন্ট।
- `tracing` এবং `TraceLayer` দিয়ে অ্যাসিঙ্ক্রোনাস স্ট্রাকচার্ড অবজারভেবিলিটি।
- প্রোডাকশন হেলথ প্রোবস (`/health/live` এবং `/health/ready`) ডিজাইন।
- ওএস সিগন্যাল (`SIGINT` / `SIGTERM`) ইন্টারসেপ্ট করে গ্রেসফুল শাটডাউন সম্পাদন।

মিনিস্টোর এখন যেকোনো ক্লাউড কন্টেইনার পরিবেশে ডেপ্লয় করার জন্য সম্পূর্ণ প্রস্তুত ও সুরক্ষিত!
