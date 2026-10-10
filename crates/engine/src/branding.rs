//! Fork identity & compatibility with existing LightCraft data directories.

use std::path::{Path, PathBuf};

pub const NAME: &str = "Ember";
pub const APPLICATION_ID: &str = "com.orthiclabs.ember";

/// Prefer Ember data, then an existing legacy directory. Never move or overwrite user data.
/// An inaccessible path is retained so its reader reports the error instead of opening empty data.
pub(crate) fn data_dir(root: &Path, current: &str, legacy: &str) -> PathBuf {
    let current = root.join(current);
    let legacy = root.join(legacy);
    if current.try_exists().unwrap_or(true) || !legacy.try_exists().unwrap_or(true) { current } else { legacy }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("ember-branding-{}-{stamp}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn existing_library_is_reused_without_moving_it() {
        let root = Scratch::new();
        let legacy = root.path().join("LightCraft Library");
        std::fs::create_dir(&legacy).unwrap();
        std::fs::write(legacy.join("catalog.snap"), b"existing catalog").unwrap();
        assert_eq!(data_dir(root.path(), "Ember Library", "LightCraft Library"), legacy);
        assert_eq!(std::fs::read(legacy.join("catalog.snap")).unwrap(), b"existing catalog");
        assert!(!root.path().join("Ember Library").exists());
    }

    #[test]
    fn new_ember_data_wins_without_overwriting_legacy_data() {
        let root = Scratch::new();
        assert_eq!(data_dir(root.path(), "Ember", "LightCraft"), root.path().join("Ember"));
        std::fs::create_dir(root.path().join("LightCraft")).unwrap();
        assert_eq!(data_dir(root.path(), "Ember", "LightCraft"), root.path().join("LightCraft"));
        std::fs::create_dir(root.path().join("Ember")).unwrap();
        assert_eq!(data_dir(root.path(), "Ember", "LightCraft"), root.path().join("Ember"));
        assert!(root.path().join("LightCraft").exists());
    }
}
