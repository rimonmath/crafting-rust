use crate::catalog::Catalog;
use crate::error::StoreError;
use crate::models::{Customer, Order};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Complete point-in-time snapshot of MiniStore state for backups and restores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreSnapshot {
    pub catalog: Catalog,
    pub customers: Vec<Customer>,
    pub orders: Vec<Order>,
    pub timestamp: String,
}

impl StoreSnapshot {
    pub fn new(
        catalog: Catalog,
        customers: Vec<Customer>,
        orders: Vec<Order>,
        timestamp: String,
    ) -> Self {
        Self {
            catalog,
            customers,
            orders,
            timestamp,
        }
    }
}

/// Service managing persistent storage of MiniStore state on the local filesystem using JSON.
#[derive(Debug, Clone)]
pub struct StorePersistence {
    base_dir: PathBuf,
}

impl StorePersistence {
    /// Creates a new persistence service configured with a root storage directory.
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// Returns the configured base directory path.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Generic helper to serialize any `Serialize` type and atomically write it to a JSON file.
    pub fn save_json<T: Serialize + ?Sized>(
        &self,
        filename: &str,
        data: &T,
    ) -> Result<PathBuf, StoreError> {
        let dir = &self.base_dir;
        fs::create_dir_all(dir).map_err(|e| StoreError::IoError {
            path: dir.display().to_string(),
            message: format!("Failed to create storage directory: {e}"),
        })?;

        let target_path = dir.join(filename);
        let temp_path = dir.join(format!("{filename}.tmp"));

        let json_text =
            serde_json::to_string_pretty(data).map_err(|e| StoreError::SerializationError {
                message: format!("Failed to serialize {filename}: {e}"),
            })?;

        // Write to temporary file first for atomic crash-safety
        fs::write(&temp_path, json_text.as_bytes()).map_err(|e| StoreError::IoError {
            path: temp_path.display().to_string(),
            message: format!("Failed to write temporary file: {e}"),
        })?;

        // Atomically replace target file
        fs::rename(&temp_path, &target_path).map_err(|e| StoreError::IoError {
            path: target_path.display().to_string(),
            message: format!("Failed to rename temporary file to target: {e}"),
        })?;

        Ok(target_path)
    }

    /// Generic helper to read and deserialize any `DeserializeOwned` type from a JSON file.
    pub fn load_json<T: serde::de::DeserializeOwned>(
        &self,
        filename: &str,
    ) -> Result<T, StoreError> {
        let file_path = self.base_dir.join(filename);

        let content = fs::read_to_string(&file_path).map_err(|e| StoreError::IoError {
            path: file_path.display().to_string(),
            message: format!("Failed to read file: {e}"),
        })?;

        serde_json::from_str(&content).map_err(|e| StoreError::SerializationError {
            message: format!("Failed to deserialize {filename}: {e}"),
        })
    }

    /// Persists the product catalog to `catalog.json`.
    pub fn save_catalog(&self, catalog: &Catalog) -> Result<PathBuf, StoreError> {
        self.save_json("catalog.json", catalog)
    }

    /// Loads the product catalog from `catalog.json`.
    pub fn load_catalog(&self) -> Result<Catalog, StoreError> {
        self.load_json("catalog.json")
    }

    /// Persists orders to `orders.json`.
    pub fn save_orders(&self, orders: &[Order]) -> Result<PathBuf, StoreError> {
        self.save_json("orders.json", orders)
    }

    /// Loads orders from `orders.json`.
    pub fn load_orders(&self) -> Result<Vec<Order>, StoreError> {
        self.load_json("orders.json")
    }

    /// Persists customers to `customers.json`.
    pub fn save_customers(&self, customers: &[Customer]) -> Result<PathBuf, StoreError> {
        self.save_json("customers.json", customers)
    }

    /// Loads customers from `customers.json`.
    pub fn load_customers(&self) -> Result<Vec<Customer>, StoreError> {
        self.load_json("customers.json")
    }

    /// Exports a complete snapshot to `snapshot.json`.
    pub fn export_snapshot(&self, snapshot: &StoreSnapshot) -> Result<PathBuf, StoreError> {
        self.save_json("snapshot.json", snapshot)
    }

    /// Imports a complete snapshot from `snapshot.json`.
    pub fn import_snapshot(&self) -> Result<StoreSnapshot, StoreError> {
        self.load_json("snapshot.json")
    }
}
