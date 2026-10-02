use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

/// Environment in which MiniStore is deployed.
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

/// Central application configuration supporting environment variables and fallbacks.
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
    /// Loads configuration from environment variables with sensible defaults.
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

    /// Custom constructor for programmatic instantiation.
    pub fn new(
        host: impl Into<String>,
        port: u16,
        data_dir: impl Into<PathBuf>,
        log_level: impl Into<String>,
        env: Environment,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            data_dir: data_dir.into(),
            log_level: log_level.into(),
            env,
        }
    }

    /// Parses the configured host and port into a valid `SocketAddr`.
    pub fn socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.host, self.port).parse()
    }
}
