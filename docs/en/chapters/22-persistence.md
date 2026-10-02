# Chapter 22: Persistence in Rust

## What You'll Learn
- Why **in-memory application state is ephemeral** and why real applications require persistent storage.
- Standard library file I/O fundamentals using **`std::fs`**, **`std::path::{Path, PathBuf}`**, and **`std::io`**.
- What **Serialization** and **Deserialization** are and how the **Serde** framework achieves zero-overhead conversion.
- Deriving **`Serialize`** and **`Deserialize`** on complex structs and enums.
- Handling dynamically sized types with the relaxed **`?Sized`** trait bound in generic serialization helpers.
- Designing **atomic, crash-resilient file writes** using temporary files and atomic filesystem renaming (`fs::rename`).
- Converting low-level I/O and JSON parsing errors into user-friendly domain errors (`StoreError::IoError` and `StoreError::SerializationError`).
- Integrating persistent storage into MiniStore:
  - Saving and loading the product catalog from `catalog.json`.
  - Storing customer records in `customers.json`.
  - Persisting order transaction history in `orders.json`.
  - Creating full point-in-time backup snapshots with `StoreSnapshot` and restoring state on application cold starts.

---

## Why Do We Need This?

Throughout Chapters 1 to 21, MiniStore has evolved from basic structs to a concurrent, asynchronous powerhouse. However, all our products, carts, orders, and sales metrics shared a fatal limitation:

> **They lived exclusively in volatile RAM.**

Whenever your program ends, the terminal window closes, or the server machine restarts:
- Every product added to the catalog vanishes.
- Every order completed by customers disappears.
- Every sales audit metric is wiped clean.

```
Volatile RAM (Ephemeral):
[Program Running] ──► Catalog, Orders, Customers exist in RAM
      │
[Process Exits]   ──► RAM freed by OS
      │
[Restart]         ──► Empty Memory! All data permanently lost!
```

To build a real-world store, data must survive across process restarts. Before reaching for heavy relational databases or full web APIs, the fundamental foundation of software engineering is **filesystem persistence**: saving state to durable storage (SSD/HDD) in human-readable, standard formats such as **JSON (JavaScript Object Notation)**.

```
Durable File Storage (Persistent):
[Program Running] ──► Serializes in-memory structs to JSON
      │
[Atomic Write]    ──► Saved safely to disk (e.g. data/catalog.json)
      │
[Process Exits]   ──► File remains safely on disk
      │
[Next Startup]    ──► Reads & deserializes JSON back into strongly typed structs
```

---

## Core Rust Concepts

### 1. Filesystem I/O with `std::fs` and `std::path`

Rust's standard library provides robust, cross-platform file manipulation tools:

- **`std::path::Path`**: An unsized slice representing a filesystem path (analogous to `&str`).
- **`std::path::PathBuf`**: An owned, mutable path on the heap (analogous to `String`).
- **`std::fs::read_to_string(path)`**: Reads the entire contents of a file into an owned `String`.
- **`std::fs::write(path, bytes)`**: Writes a byte slice to a file, creating or truncating it.
- **`std::fs::create_dir_all(path)`**: Recursively creates a directory and all parent components.
- **`std::fs::rename(from, to)`**: Atomically moves or renames a filesystem path.

```rust
use std::fs;
use std::path::PathBuf;

let dir = PathBuf::from("data");
fs::create_dir_all(&dir).expect("Directory creation failed");

let file_path = dir.join("note.txt");
fs::write(&file_path, b"MiniStore Persistent Note").expect("Write failed");

let contents = fs::read_to_string(&file_path).expect("Read failed");
println!("Read from disk: {contents}");
```

---

### 2. What is Serde?

**Serde** (short for **Ser**ialize / **De**serialize) is the standard serialization framework for Rust. It consists of two complementary operations:

1. **Serialization**: Converting an in-memory Rust data structure into a portable data format (such as JSON, YAML, TOML, or binary MessagePack).
2. **Deserialization**: Parsing external structured data back into strongly typed Rust data structures.

Unlike other languages that rely on slow runtime reflection or dynamic typing, Serde generates specialized, compile-time conversion code using Rust's trait system and procedural derive macros (`#[derive(Serialize, Deserialize)]`).

```
                    Serialization
Rust Struct / Enum ───────────────► JSON String / File
                   ◄───────────────
                    Deserialization
```

Add Serde to your `Cargo.toml`:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

### 3. Deriving `Serialize` and `Deserialize`

When you add `#[derive(Serialize, Deserialize)]` to a struct or enum, the compiler automatically generates implementations for both Serde traits:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ProductCategory {
    Electronics,
    OfficeSupplies,
    Furniture,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: u64,
    pub sku: String,
    pub name: String,
    pub category: ProductCategory,
    pub price_cents: u32,
    pub stock: u32,
}
```

Serializing to a JSON string:
```rust
let product = Product {
    id: 101,
    sku: String::from("TECH-KEY-001"),
    name: String::from("Mechanical Keyboard"),
    category: ProductCategory::Electronics,
    price_cents: 12000,
    stock: 10,
};

let json_string = serde_json::to_string_pretty(&product).unwrap();
println!("{json_string}");
```

Output:
```json
{
  "id": 101,
  "sku": "TECH-KEY-001",
  "name": "Mechanical Keyboard",
  "category": "Electronics",
  "price_cents": 12000,
  "stock": 10
}
```

Deserializing back into a Rust struct:
```rust
let restored: Product = serde_json::from_str(&json_string).unwrap();
assert_eq!(product, restored);
```

---

### 4. Dynamically Sized Types & The `?Sized` Bound

Consider a generic helper that saves data to disk:

```rust
pub fn save_json<T: Serialize>(&self, filename: &str, data: &T) -> Result<PathBuf, StoreError>
```

In Rust, every generic type parameter `T` has an implicit `Sized` bound (`T: Sized`). This means the size of `T` must be known at compile time.

However, if you want to pass a borrowed slice like `&[Order]` or `&[Customer]`, the underlying type `[Order]` is a **Dynamically Sized Type (DST)** whose size depends on the runtime slice length! If you pass `&[Order]`, the compiler will reject the call with `error[E0277]: the size for values of type [Order] cannot be known at compilation time`.

To allow passing both sized types (like `&Catalog`) and unsized slices (like `&[Order]`), you relax the implicit bound with **`?Sized`**:

```rust
pub fn save_json<T: Serialize + ?Sized>(
    &self,
    filename: &str,
    data: &T,
) -> Result<PathBuf, StoreError> {
    // Both &Catalog and &[Order] can be passed here!
}
```

---

### 5. Atomic Writes for Crash Safety

What happens if the computer loses power or the operating system crashes right while your program is writing `catalog.json`?
If you write directly to `catalog.json`, you could leave a half-written, corrupt file on disk that can never be deserialized again!

To prevent data corruption, production systems use **atomic rename semantics**:
1. Serialize data to pretty JSON.
2. Write the JSON payload to a temporary file (`catalog.json.tmp`).
3. Use `std::fs::rename` to atomically overwrite the destination `catalog.json`.

On POSIX and modern Windows filesystems, renaming a file is an atomic directory metadata update. The file on disk is either 100% the old version or 100% the new version—never a corrupted half-written state!

```
[Serialize to Memory]
        │
[Write to Temp File]   ──► catalog.json.tmp (written completely)
        │
[Atomic fs::rename]    ──► catalog.json (replaced instantaneously)
```

---

## MiniStore Implementation

### 1. Domain Error Extensions

In `ministore/src/error.rs`, we added two dedicated variants to `StoreError` to provide crystal-clear diagnostics when I/O or JSON formatting fails:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum StoreError {
    // ... existing variants ...
    IoError { path: String, message: String },
    SerializationError { message: String },
}
```

### 2. The `StorePersistence` Engine

In `ministore/src/persistence.rs`, we created the storage engine:

```rust
use crate::catalog::Catalog;
use crate::error::StoreError;
use crate::models::{Customer, Order};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreSnapshot {
    pub catalog: Catalog,
    pub customers: Vec<Customer>,
    pub orders: Vec<Order>,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct StorePersistence {
    base_dir: PathBuf,
}

impl StorePersistence {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

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

        let json_text = serde_json::to_string_pretty(data).map_err(|e| {
            StoreError::SerializationError {
                message: format!("Failed to serialize {filename}: {e}"),
            }
        })?;

        fs::write(&temp_path, json_text.as_bytes()).map_err(|e| StoreError::IoError {
            path: temp_path.display().to_string(),
            message: format!("Failed to write temporary file: {e}"),
        })?;

        fs::rename(&temp_path, &target_path).map_err(|e| StoreError::IoError {
            path: target_path.display().to_string(),
            message: format!("Failed to rename temporary file to target: {e}"),
        })?;

        Ok(target_path)
    }

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

    pub fn save_catalog(&self, catalog: &Catalog) -> Result<PathBuf, StoreError> {
        self.save_json("catalog.json", catalog)
    }

    pub fn load_catalog(&self) -> Result<Catalog, StoreError> {
        self.load_json("catalog.json")
    }

    pub fn save_orders(&self, orders: &[Order]) -> Result<PathBuf, StoreError> {
        self.save_json("orders.json", orders)
    }

    pub fn load_orders(&self) -> Result<Vec<Order>, StoreError> {
        self.load_json("orders.json")
    }

    pub fn export_snapshot(&self, snapshot: &StoreSnapshot) -> Result<PathBuf, StoreError> {
        self.save_json("snapshot.json", snapshot)
    }

    pub fn import_snapshot(&self) -> Result<StoreSnapshot, StoreError> {
        self.load_json("snapshot.json")
    }
}
```

---

## Testing Your Code

All persistence features are thoroughly covered by unit tests in `src/lib.rs`:

```bash
cargo test
```

Key test scenarios:
1. **`test_product_and_category_json_roundtrip`**: Verifies that custom enums and structs serialize to valid JSON and deserialize back to identical values.
2. **`test_order_and_customer_json_roundtrip`**: Ensures complex nested domain models (including `HashSet<String>` tags and `OrderStatus` enums) roundtrip faithfully.
3. **`test_store_persistence_catalog_file_io`**: Tests saving to a temporary directory, verifying file creation on disk, and loading back into an empty catalog.
4. **`test_store_persistence_snapshot_export_import`**: Tests exporting a unified point-in-time snapshot and restoring the entire store state.
5. **`test_store_persistence_errors`**: Asserts that missing files return `StoreError::IoError` with the file path, and corrupt JSON files return `StoreError::SerializationError`.

---

## Checkpoint & Summary

With Chapter 22 complete, MiniStore data no longer vanishes when the process ends. You have mastered:
- Standard library file I/O (`std::fs`, `std::path::PathBuf`).
- Serialization & Deserialization with `serde` and `serde_json`.
- The `?Sized` trait bound for dynamically sized slices.
- Crash-safe atomic file writes using temporary files and `fs::rename`.
- Domain backup snapshots for complete store state survival.

In the next chapter, we will build a modern **Rust Web API** using the Axum framework to expose MiniStore over HTTP!
