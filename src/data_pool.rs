//! Global character state: a singleton holding the loaded animation and
//! kinematic tree.
//!
//! Replaces `DataPool.cpp`'s static singleton pattern using `OnceLock` +
//! `Mutex` for safe concurrent access.

use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crate::loader;
use crate::types::{Animation, KineTree};

/// The complete loaded state for one character.
pub struct LoadedCharacter {
    pub animation: Animation,
    pub kine_tree: KineTree,
    /// Filename without extension, used as the character name.
    pub name: String,
}

static STATE: OnceLock<Mutex<Option<LoadedCharacter>>> = OnceLock::new();

fn state() -> &'static Mutex<Option<LoadedCharacter>> {
    STATE.get_or_init(|| Mutex::new(None))
}

/// Load a character from a `.glb` file, replacing any previously loaded one.
///
/// # Errors
///
/// Returns an error string if the file cannot be parsed.
pub fn load(path: &Path) -> Result<(), String> {
    let (animation, kine_tree) = loader::load_character(path)?;
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut guard = state().lock().unwrap();
    *guard = Some(LoadedCharacter {
        animation,
        kine_tree,
        name,
    });
    Ok(())
}

/// Run a closure with shared access to the loaded character.
///
/// Returns `None` if no character has been loaded yet.
pub fn with_state<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&LoadedCharacter) -> R,
{
    let guard = state().lock().unwrap();
    guard.as_ref().map(f)
}
