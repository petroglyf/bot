//! Short-term memory: stores named 3-D targets (look-at, point-at) shared
//! between the main loop and the ZMQ server threads.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Thread-safe map from target name (e.g. `"Target"`) to a 3-D point.
#[derive(Debug, Clone, Default)]
pub struct SharedState {
    inner: Arc<Mutex<HashMap<String, [f64; 3]>>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or update a named 3-D target.
    pub fn set(&self, name: &str, value: [f64; 3]) {
        self.inner.lock().unwrap().insert(name.to_string(), value);
    }

    /// Read a named 3-D target, returning `None` if not present.
    pub fn get(&self, name: &str) -> Option<[f64; 3]> {
        self.inner.lock().unwrap().get(name).copied()
    }
}
