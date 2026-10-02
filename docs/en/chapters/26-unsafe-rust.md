# Chapter 26: Unsafe Rust & Safe Abstractions

## What You'll Learn

- What **Unsafe Rust** actually is (and the common myth: the borrow checker and type checker are _not_ turned off!).
- The **5 Superpowers of Unsafe Rust** that safe Rust forbids.
- The differences between **Safe References (`&T`, `&mut T`)** and **Raw Pointers (`*const T`, `*mut T`)**.
- Manual heap memory allocation, resizing, and deallocation via **`std::alloc` (`alloc`, `realloc`, `dealloc`, `Layout`)**.
- The core philosophy of Rust: **Building 100% Safe Abstractions over Unsafe Primitives**.
- How standard collections (`Vec`, `String`, `Box`, `Arc`) use unsafe code under the hood without compromising external safety.
- **Foreign Function Interface (FFI)**: Calling standard C libraries (`extern "C"`) and exporting C-compatible ABI functions (`#[unsafe(no_mangle)]`).
- Documenting safety contracts with **`// SAFETY:`** comments to prevent Undefined Behavior (UB).

---

## Why Do We Need Unsafe Rust?

Rust's primary claim to fame is compile-time memory safety without a garbage collector. The borrow checker ensures there are no dangling pointers, double frees, or data races.

However, computer hardware is inherently unsafe:

1. **Low-Level Hardware & Operating Systems**: CPUs, device drivers, and kernel system calls communicate through raw memory addresses, DMA buffers, and registers.
2. **Foreign Function Interface (FFI)**: Most operating system APIs (Linux POSIX, Windows Win32, macOS Cocoa) and legacy enterprise systems are written in C. C has no borrow checker, no lifetimes, and passes raw pointers everywhere.
3. **Fundamental Data Structures**: Many foundational structures—like doubly-linked lists, lock-free ring buffers, custom allocators, and even Rust's own `Vec<T>` and `HashMap<K, V>`—cannot be statically verified by the compiler's strict single-mutable-reference aliasing rules.

```
+--------------------------------------------------------------+
|                     Safe Rust (99% of code)                 |
|   - Ownership & Lifetimes enforced statically                |
|   - Zero data races, no dangling pointers                   |
+--------------------------------------------------------------+
                               |
                               | Built on top of
                               v
+--------------------------------------------------------------+
|              Safe Abstraction Boundary (API Contract)        |
+--------------------------------------------------------------+
                               |
                               | Internal implementation
                               v
+--------------------------------------------------------------+
|                    Unsafe Rust (1% of code)                 |
|   - Raw pointers (*const T, *mut T)                          |
|   - Manual heap alloc/dealloc via std::alloc                 |
|   - Foreign Function Interface (extern "C")                  |
+--------------------------------------------------------------+
```

If Rust did not have an unsafe escape hatch, developers would be forced to write high-performance subsystems and OS interfaces in C or C++. Unsafe Rust allows the programmer to tell the compiler: **"Trust me; I have manually verified the invariants you cannot prove statically."**

---

## The 5 Superpowers of Unsafe Rust

Inside an `unsafe { ... }` block, you gain exactly five specific capabilities not allowed in safe Rust:

1. **Dereference raw pointers** (`*const T` and `*mut T`).
2. **Call an unsafe function or method** (including FFI C functions).
3. **Implement an unsafe trait** (such as `Send` and `Sync`).
4. **Mutate a mutable static variable** or access it.
5. **Access fields of a `union`**.

> [!NOTE]
> `unsafe` does **not** turn off the borrow checker! Types, lifetimes, ownership, and borrow checking are still strictly enforced inside unsafe blocks. Only the 5 superpowers above become accessible.

---

## Raw Pointers vs. Safe References

Safe Rust gives us references: `&T` (shared, immutable) and `&mut T` (exclusive, mutable).
Unsafe Rust introduces **Raw Pointers**: `*const T` (immutable raw pointer) and `*mut T` (mutable raw pointer).

| Feature               | Safe Reference (`&T`, `&mut T`)           | Raw Pointer (`*const T`, `*mut T`)                |
| :-------------------- | :---------------------------------------- | :------------------------------------------------ |
| **Nullability**       | Guaranteed non-null                       | Can be null (`std::ptr::null()`)                  |
| **Validity**          | Always points to valid initialized memory | Can dangle or point to arbitrary addresses        |
| **Aliasing Rules**    | Strict: 1 mutable XOR many immutable      | Can have multiple mutable pointers to same memory |
| **Alignment**         | Automatically guaranteed by compiler      | Can be unaligned                                  |
| **Automatic Cleanup** | Governed by ownership & `Drop`            | No automatic cleanup; manual deallocation         |
| **Creation**          | Safe                                      | Safe (`&val as *const T`)                         |
| **Dereferencing**     | Safe (`*reference`)                       | **UNSAFE** (`unsafe { *raw_ptr }`)                |

Notice that **creating** a raw pointer is 100% safe! Only **dereferencing** it (reading or writing to the target address) requires an `unsafe` block.

```rust
let mut price: u32 = 15000;

// Creating raw pointers is completely SAFE:
let raw_const: *const u32 = &price;
let raw_mut: *mut u32 = &mut price;

// Reading or writing through raw pointers is UNSAFE:
unsafe {
    println!("Price via raw pointer: {}", *raw_const);
    *raw_mut = 18000;
}
```

---

## Building Safe Abstractions: `RawBarcodeBuffer`

The golden rule of Rust systems programming is: **Never expose `unsafe` to your callers if you can encapsulate it in a safe API.**

In MiniStore, we need a high-performance contiguous byte buffer for barcode scanning, receipt printing, and high-throughput serialization: [`RawBarcodeBuffer`](https://github.com/rimonmath/crafting-rust/blob/main/ministore/src/unsafe_utils.rs).

### 1. Structure Definition & `NonNull`

Instead of storing raw `*mut u8`, modern Rust uses `NonNull<u8>`, which promises the compiler the pointer is never null, enabling niche optimizations (like `Option<NonNull<T>>` having the same size as a pointer):

```rust
use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ptr::NonNull;

#[derive(Debug)]
pub struct RawBarcodeBuffer {
    ptr: NonNull<u8>,
    capacity: usize,
    len: usize,
}

// SAFETY: RawBarcodeBuffer owns its unique heap memory and has no shared mutable aliasing.
unsafe impl Send for RawBarcodeBuffer {}
unsafe impl Sync for RawBarcodeBuffer {}
```

### 2. Manual Memory Allocation with `std::alloc`

In safe Rust, `Vec::with_capacity` handles memory allocation. In low-level systems programming, we interact directly with the global allocator using `std::alloc::Layout`:

```rust
impl RawBarcodeBuffer {
    pub fn with_capacity(capacity: usize) -> Result<Self, StoreError> {
        let cap = capacity.max(1);
        let layout = Layout::array::<u8>(cap).map_err(|e| StoreError::InvalidPayment {
            reason: format!("Invalid memory layout: {e}"),
        })?;

        // SAFETY: The layout has non-zero size and is properly aligned for `u8`.
        let raw_ptr = unsafe { alloc(layout) };
        let ptr = NonNull::new(raw_ptr).ok_or_else(|| StoreError::InvalidPayment {
            reason: "Out of memory: raw heap allocation failed".to_string(),
        })?;

        Ok(Self {
            ptr,
            capacity: cap,
            len: 0,
        })
    }
}
```

### 3. Pointer Arithmetic & Writes

To append bytes, we calculate the offset in memory and write directly to the pointer address:

```rust
pub fn push(&mut self, byte: u8) -> Result<(), StoreError> {
    if self.len >= self.capacity {
        self.grow()?;
    }

    // SAFETY:
    // 1. `self.ptr` is non-null and valid for writes.
    // 2. `self.len < self.capacity`, so `self.len` offset is within allocated bounds.
    unsafe {
        let dest = self.ptr.as_ptr().add(self.len);
        std::ptr::write(dest, byte);
    }
    self.len += 1;
    Ok(())
}
```

### 4. RAII Cleanup with `Drop`

If we allocate heap memory with `alloc()`, Rust will **not** automatically free it when `RawBarcodeBuffer` goes out of scope. We must implement `Drop` to call `dealloc()`:

```rust
impl Drop for RawBarcodeBuffer {
    fn drop(&mut self) {
        if self.capacity > 0 {
            let layout = Layout::array::<u8>(self.capacity).expect("Valid layout during drop");
            // SAFETY:
            // 1. `self.ptr` was allocated with this exact layout.
            // 2. `drop` is executed exactly once when the buffer goes out of scope.
            unsafe {
                dealloc(self.ptr.as_ptr(), layout);
            }
        }
    }
}
```

Thanks to RAII (Resource Acquisition Is Initialization), consumers of `RawBarcodeBuffer` never worry about memory leaks!

---

## Foreign Function Interface (FFI)

Rust can seamlessly call C libraries and be called by foreign runtimes (Python, Go, Node.js, C#) using the standard **C Application Binary Interface (C ABI)**.

### 1. Calling C Standard Library (`extern "C"`)

In Rust 2024 edition, external function blocks must be declared with `unsafe extern "C"`:

```rust
/// Computes the absolute difference between two integer prices using standard C ABI FFI.
pub fn c_abi_price_diff(price_a: i32, price_b: i32) -> i32 {
    unsafe extern "C" {
        fn abs(x: std::ffi::c_int) -> std::ffi::c_int;
    }
    // SAFETY: `abs` is a pure C standard library function with no memory side effects.
    unsafe { abs(price_a - price_b) }
}
```

### 2. Exporting Rust Functions to Foreign Languages

To make a Rust function callable from external languages:

- We add `extern "C"` to specify the standard C calling convention.
- We add `#[unsafe(no_mangle)]` (Rust 2024 edition) so the compiler does not mangle the function name into an internal symbol like `_ZN9ministore...`.

```rust
/// C-compatible ABI function exported for external system integration.
#[unsafe(no_mangle)]
pub extern "C" fn ministore_c_calculate_tax(subtotal_cents: u32, tax_rate_percent: u32) -> u32 {
    (subtotal_cents * tax_rate_percent) / 100
}
```

External C, Python (`ctypes`), or Node.js (`ffi-napi`) code can now dynamically load this symbol directly!

---

## Testing Your Code

Run the full test suite across the workspace:

```bash
cargo test
```

Key test scenarios in `ministore/src/lib.rs`:

1. **`test_unsafe_raw_barcode_buffer_safe_abstraction`**:
   - Tests heap allocation with small initial capacity.
   - Tests appending bytes and dynamic reallocation (`grow`).
   - Verifies UTF-8 slice conversion (`as_str()`) and buffer reuse (`clear()`).
   - Verifies clean memory deallocation on drop.
2. **`test_unsafe_raw_pointer_swap_and_ffi`**:
   - Tests raw pointer swap via `unsafe_raw_swap` and `safe_swap_prices`.
   - Tests C standard library FFI (`c_abi_price_diff`).
   - Tests exported C ABI tax calculation (`ministore_c_calculate_tax`).

---

## Checkpoint & Summary

With Chapter 26 complete, you have conquered the final frontier of the Rust language:

- You know what Unsafe Rust is and the 5 superpowers it grants.
- You understand raw pointers (`*const T`, `*mut T`) and how they differ from safe references.
- You mastered manual memory allocation (`Layout`, `alloc`, `realloc`, `dealloc`) and RAII cleanup with `Drop`.
- You built a 100% safe, high-performance abstraction (`RawBarcodeBuffer`).
- You bridged Rust and C via Foreign Function Interface (FFI) imports and exports.

**Congratulations!** You have traveled from the absolute fundamentals of Rust (variables, ownership, borrowing) through intermediate modeling (structs, enums, traits, generics) and advanced systems (concurrency, async, persistence, web APIs, macros, unsafe). MiniStore stands as a complete, robust, enterprise-grade Rust application!
