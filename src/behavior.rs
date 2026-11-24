//! Behavior executive: drives high-level robot behaviors each tick.
//!
//! Currently a minimal placeholder matching the structure of the original
//! Java `BehaviorExecutive`. Extend with actual behavior trees as needed.

/// Drives active behaviors each update cycle (called every ~10 ms).
#[derive(Debug, Default)]
pub struct BehaviorExecutive;

impl BehaviorExecutive {
    pub fn new() -> Self {
        Self
    }

    /// Advance all active behaviors by one tick.
    // Placeholder: will use `self` once behaviors exist.
    #[allow(clippy::unused_self)]
    pub fn update(&mut self) {
        // No-op until behaviors are implemented.
    }
}
