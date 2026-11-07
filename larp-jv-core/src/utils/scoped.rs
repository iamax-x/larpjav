//! Scoped utilities for thread-local and async-scoped storage

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::LocalSet;

/// Thread-local storage
thread_local! {
    static THREAD_LOCAL_STORAGE: RefCell<HashMap<String, Box<dyn std::any::Any + Send>>> = RefCell::new(HashMap::new());
}

/// Set thread-local value
pub fn set_thread_local<T: 'static + Send>(key: &str, value: T) {
    THREAD_LOCAL_STORAGE.with(|storage| {
        storage.borrow_mut().insert(key.to_string(), Box::new(value));
    });
}

/// Get thread-local value
pub fn get_thread_local<T: 'static + Clone>(key: &str) -> Option<T> {
    THREAD_LOCAL_STORAGE.with(|storage| {
        storage.borrow().get(key).and_then(|v| v.downcast_ref::<T>().cloned())
    })
}

/// Remove thread-local value
pub fn remove_thread_local(key: &str) {
    THREAD_LOCAL_STORAGE.with(|storage| {
        storage.borrow_mut().remove(key);
    });
}

/// Clear all thread-local storage
pub fn clear_thread_local() {
    THREAD_LOCAL_STORAGE.with(|storage| {
        storage.borrow_mut().clear();
    });
}

/// Async-scoped storage (using LocalSet)
pub struct ScopedStorage {
    storage: HashMap<String, Box<dyn std::any::Any + Send>>,
}

impl ScopedStorage {
    pub fn new() -> Self {
        Self { storage: HashMap::new() }
    }

    pub fn set<T: 'static + Send>(&mut self, key: &str, value: T) {
        self.storage.insert(key.to_string(), Box::new(value));
    }

    pub fn get<T: 'static + Clone>(&self, key: &str) -> Option<T> {
        self.storage.get(key).and_then(|v| v.downcast_ref::<T>().cloned())
    }

    pub fn remove(&mut self, key: &str) {
        self.storage.remove(key);
    }
}

impl Default for ScopedStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Run with scoped storage
pub async fn with_scoped<F, Fut, T>(f: F) -> T
where
    F: FnOnce(&mut ScopedStorage) -> Fut,
    Fut: Future<Output = T>,
{
    let mut storage = ScopedStorage::new();
    f(&mut storage).await
}

/// Context variable for async propagation
pub struct ContextVar<T> {
    key: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ContextVar<T> {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn set(&self, value: T) {
        set_thread_local(&self.key, value);
    }

    pub fn get(&self) -> Option<T>
    where
        T: Clone + 'static + Send,
    {
        get_thread_local(&self.key)
    }

    pub fn clear(&self) {
        remove_thread_local(&self.key);
    }
}

/// Current span context (for tracing)
#[cfg(feature = "tracing")]
pub mod tracing_scope {
    use tracing::Span;

    pub fn current_span() -> Span {
        Span::current()
    }

    pub fn enter_span<F, R>(span: Span, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _enter = span.enter();
        f()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_local() {
        set_thread_local("test_key", 42i32);
        assert_eq!(get_thread_local::<i32>("test_key"), Some(42));
        remove_thread_local("test_key");
        assert_eq!(get_thread_local::<i32>("test_key"), None);
    }

    #[test]
    fn test_scoped_storage() {
        let mut storage = ScopedStorage::new();
        storage.set("key", "value");
        assert_eq!(storage.get::<&str>("key"), Some("value"));
        storage.remove("key");
        assert_eq!(storage.get::<&str>("key"), None);
    }

    #[test]
    fn test_context_var() {
        let var = ContextVar::<String>::new("test_var");
        var.set("hello".to_string());
        assert_eq!(var.get(), Some("hello".to_string()));
        var.clear();
        assert_eq!(var.get(), None);
    }
}
