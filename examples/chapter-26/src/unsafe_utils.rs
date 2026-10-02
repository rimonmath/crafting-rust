//! Unsafe Rust, raw pointers, manual memory management, and Foreign Function Interface (FFI).
//!
//! Demonstrates how low-level systems programming primitives are encapsulated
//! within 100% safe, leak-free abstractions in MiniStore.

use crate::error::StoreError;
use std::alloc::{Layout, alloc, dealloc, realloc};
use std::ptr::NonNull;

/// A high-performance contiguous byte buffer for barcode scanning and receipt serialization,
/// backed by raw heap allocation, raw pointers, and pointer arithmetic.
///
/// Encapsulates unsafe manual memory management within a completely safe public API.
#[derive(Debug)]
pub struct RawBarcodeBuffer {
    ptr: NonNull<u8>,
    capacity: usize,
    len: usize,
}

// SAFETY: RawBarcodeBuffer uniquely owns its heap allocation and has no shared mutable aliasing.
unsafe impl Send for RawBarcodeBuffer {}
unsafe impl Sync for RawBarcodeBuffer {}

impl RawBarcodeBuffer {
    /// Allocates a new buffer with the requested initial capacity.
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

    /// Returns the number of initialized bytes in the buffer.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the buffer contains no bytes.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the total allocated capacity of the buffer.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Appends a single byte using raw pointer offset arithmetic.
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

    /// Appends a slice of bytes into the buffer.
    pub fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), StoreError> {
        for &b in bytes {
            self.push(b)?;
        }
        Ok(())
    }

    /// Appends an ASCII string representation of a SKU or barcode.
    pub fn push_str(&mut self, s: &str) -> Result<(), StoreError> {
        self.extend_from_slice(s.as_bytes())
    }

    /// Exposes a safe, immutable byte slice view over the initialized portion.
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY:
        // 1. `self.ptr` points to `self.len` initialized, valid bytes.
        // 2. The returned slice's lifetime is bound to `&self`, preventing mutation while borrowed.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Returns the buffer content as a UTF-8 string slice if valid.
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.as_slice())
    }

    /// Clears the buffer without deallocating the underlying memory.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Reallocates the buffer with double the capacity.
    fn grow(&mut self) -> Result<(), StoreError> {
        let new_capacity =
            self.capacity
                .checked_mul(2)
                .ok_or_else(|| StoreError::InvalidPayment {
                    reason: "Capacity overflow during buffer growth".to_string(),
                })?;

        let old_layout =
            Layout::array::<u8>(self.capacity).map_err(|e| StoreError::InvalidPayment {
                reason: format!("Old layout error: {e}"),
            })?;
        let new_size = new_capacity;

        // SAFETY:
        // 1. `self.ptr.as_ptr()` was allocated with `old_layout`.
        // 2. `new_size > 0` and is properly aligned for `u8`.
        let new_ptr = unsafe { realloc(self.ptr.as_ptr(), old_layout, new_size) };
        let non_null = NonNull::new(new_ptr).ok_or_else(|| StoreError::InvalidPayment {
            reason: "Reallocation failed: out of memory".to_string(),
        })?;

        self.ptr = non_null;
        self.capacity = new_capacity;
        Ok(())
    }
}

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

/// Swaps two 32-bit values in place using raw pointer dereferencing.
///
/// # Safety
/// The caller must ensure that `a` and `b` are valid, aligned, non-null, and non-overlapping.
#[allow(clippy::manual_swap)]
pub unsafe fn unsafe_raw_swap(a: *mut u32, b: *mut u32) {
    // SAFETY: The safety invariants are guaranteed by the caller's contract.
    unsafe {
        let temp = *a;
        *a = *b;
        *b = temp;
    }
}

/// Safe public wrapper around `unsafe_raw_swap`.
pub fn safe_swap_prices(a: &mut u32, b: &mut u32) {
    // SAFETY: References `&mut` are guaranteed by the compiler to be valid, aligned,
    // non-null, and non-overlapping.
    unsafe {
        unsafe_raw_swap(a as *mut u32, b as *mut u32);
    }
}

/// Computes the absolute difference between two integer prices using standard C ABI FFI.
///
/// Demonstrates calling an external C standard library function (`abs`) safely.
pub fn c_abi_price_diff(price_a: i32, price_b: i32) -> i32 {
    unsafe extern "C" {
        fn abs(x: std::ffi::c_int) -> std::ffi::c_int;
    }
    // SAFETY: `abs` is a pure C standard library function with no memory side effects.
    unsafe { abs(price_a - price_b) }
}

/// C-compatible ABI function exported for external system integration (Python, C, Go, Node.js).
///
/// Calculates tax amount in cents given a subtotal and tax rate percentage.
#[unsafe(no_mangle)]
pub extern "C" fn ministore_c_calculate_tax(subtotal_cents: u32, tax_rate_percent: u32) -> u32 {
    (subtotal_cents * tax_rate_percent) / 100
}
