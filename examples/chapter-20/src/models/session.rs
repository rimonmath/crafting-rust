//! StoreSession smart pointer demonstrating Deref, DerefMut, and Drop traits.

use std::cell::RefCell;
use std::ops::{Deref, DerefMut, Drop};
use std::rc::Rc;

/// A custom smart pointer wrapping a session payload, demonstrating `Deref`, `DerefMut`, and `Drop`.
///
/// - `Deref` allows transparent method calls and field access on the inner payload `T` through deref coercion.
/// - `DerefMut` allows mutating the inner payload through mutable references.
/// - `Drop` executes deterministic cleanup when the session leaves scope, updating an optional lifecycle tracker.
#[derive(Debug, PartialEq, Eq)]
pub struct StoreSession<T> {
    pub session_id: String,
    pub inner: T,
    drop_flag: Option<Rc<RefCell<bool>>>,
}

impl<T> StoreSession<T> {
    pub fn new(session_id: impl Into<String>, inner: T) -> Self {
        Self {
            session_id: session_id.into(),
            inner,
            drop_flag: None,
        }
    }

    /// Creates a session with an attached drop notification flag.
    ///
    /// The flag is set to `true` on creation, and reset to `false` when dropped.
    pub fn with_drop_flag(
        session_id: impl Into<String>,
        inner: T,
        drop_flag: Rc<RefCell<bool>>,
    ) -> Self {
        *drop_flag.borrow_mut() = true;
        Self {
            session_id: session_id.into(),
            inner,
            drop_flag: Some(drop_flag),
        }
    }
}

impl<T> Deref for StoreSession<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for StoreSession<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Drop for StoreSession<T> {
    fn drop(&mut self) {
        if let Some(ref flag) = self.drop_flag {
            *flag.borrow_mut() = false;
        }
    }
}
