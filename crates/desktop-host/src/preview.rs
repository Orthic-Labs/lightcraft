//! Bounded, in-process transport for decoded previews.
//!
//! Handles are opaque and contain no filesystem information.  The store owns encoded bytes in
//! `Arc`s so a WebView decode can keep a previous image alive while a newer request is published.

#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Maximum number of live handles in the default transport store.
pub const DEFAULT_MAX_ITEMS: usize = 256;
/// Maximum encoded bytes in the default transport store.
pub const DEFAULT_MAX_BYTES: usize = 64 * 1024 * 1024;
/// Handles older than this are eligible for expiry when pressure arrives.
pub const DEFAULT_TTL: Duration = Duration::from_secs(30);

/// Immutable bytes returned by [`PreviewStore::get`].
#[derive(Clone, Debug)]
pub struct PreviewBytes {
    pub bytes: Arc<[u8]>,
    pub content_type: &'static str,
}

struct Entry {
    handle: String,
    bytes: Arc<[u8]>,
    content_type: &'static str,
    used: Instant,
}

struct Inner {
    entries: Vec<Entry>,
    next: u64,
    bytes: usize,
}

/// Cloneable accessor shared by the host transport and renderer.
#[derive(Clone)]
pub struct PreviewStore {
    inner: Arc<Mutex<Inner>>,
    max_items: usize,
    max_bytes: usize,
    ttl: Duration,
}

impl Default for PreviewStore {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_ITEMS, DEFAULT_MAX_BYTES, DEFAULT_TTL)
    }
}

impl PreviewStore {
    pub fn new(max_items: usize, max_bytes: usize, ttl: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner { entries: Vec::new(), next: 1, bytes: 0 })),
            max_items: max_items.max(1),
            max_bytes: max_bytes.max(1),
            ttl: ttl.max(Duration::from_millis(1)),
        }
    }

    /// Insert encoded bytes and return an opaque, non-path handle.
    pub fn insert(&self, bytes: impl Into<Arc<[u8]>>, content_type: &'static str) -> Result<String, String> {
        let bytes = bytes.into();
        if bytes.len() > self.max_bytes {
            return Err("preview payload exceeds transport budget".to_string());
        }
        let mut inner = self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        self.prune_locked(&mut inner, Instant::now());
        let handle = loop {
            let candidate = format!("lc-preview-{:016x}", inner.next);
            inner.next = inner.next.wrapping_add(1).max(1);
            if !inner.entries.iter().any(|entry| entry.handle == candidate) {
                break candidate;
            }
        };
        let size = bytes.len();
        inner.entries.push(Entry { handle: handle.clone(), bytes, content_type, used: Instant::now() });
        inner.bytes = inner.bytes.saturating_add(size);
        self.evict_locked(&mut inner);
        Ok(handle)
    }

    /// Fetch bytes without exposing a path or mutable backing storage.
    pub fn get(&self, handle: &str) -> Option<PreviewBytes> {
        if handle.len() > 64 || !handle.starts_with("lc-preview-") {
            return None;
        }
        let mut inner = self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        self.prune_locked(&mut inner, Instant::now());
        let entry = inner.entries.iter_mut().find(|entry| entry.handle == handle)?;
        entry.used = Instant::now();
        Some(PreviewBytes { bytes: entry.bytes.clone(), content_type: entry.content_type })
    }

    /// A WebView may acknowledge a decode once it no longer needs an older handle.
    pub fn acknowledge(&self, handle: &str) -> bool {
        let mut inner = self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(index) = inner.entries.iter().position(|entry| entry.handle == handle) else { return false };
        let entry = inner.entries.remove(index);
        inner.bytes = inner.bytes.saturating_sub(entry.bytes.len());
        true
    }

    pub fn len(&self) -> usize {
        let mut inner = self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        self.prune_locked(&mut inner, Instant::now());
        inner.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn bytes(&self) -> usize {
        let mut inner = self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        self.prune_locked(&mut inner, Instant::now());
        inner.bytes
    }

    fn prune_locked(&self, inner: &mut Inner, now: Instant) {
        let ttl = self.ttl;
        inner.entries.retain(|entry| now.duration_since(entry.used) <= ttl);
        inner.bytes = inner.entries.iter().map(|entry| entry.bytes.len()).sum();
    }

    fn evict_locked(&self, inner: &mut Inner) {
        while inner.entries.len() > self.max_items || inner.bytes > self.max_bytes {
            let Some(index) = inner.entries.iter().enumerate().min_by_key(|(_, entry)| entry.used).map(|(index, _)| index) else { break };
            let entry = inner.entries.remove(index);
            inner.bytes = inner.bytes.saturating_sub(entry.bytes.len());
        }
    }
}

/// Encode display pixels as lossless sRGB PNG, retaining alpha for future overlays.
pub fn encode_png(image: &lightcraft_raster::Rgba8) -> Result<Vec<u8>, String> {
    let width = u32::try_from(image.width).map_err(|_| "preview width exceeds PNG limit".to_string())?;
    let height = u32::try_from(image.height).map_err(|_| "preview height exceeds PNG limit".to_string())?;
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        let mut writer = encoder.write_header().map_err(|error| format!("preview PNG header: {error}"))?;
        writer.write_image_data(&image.as_bytes()).map_err(|error| format!("preview PNG data: {error}"))?;
    }
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_handles_survive_replacement_until_bound() {
        let store = PreviewStore::new(2, 1024, Duration::from_secs(30));
        let Ok(first) = store.insert(Arc::<[u8]>::from(vec![1, 2]), "image/png") else { return };
        let Ok(second) = store.insert(Arc::<[u8]>::from(vec![3, 4]), "image/png") else { return };
        assert_eq!(store.get(&first).map(|value| value.bytes.len()), Some(2));
        let Ok(_third) = store.insert(Arc::<[u8]>::from(vec![5, 6]), "image/png") else { return };
        assert!(store.get(&first).is_some());
        assert!(store.get(&second).is_none());
        assert!(store.len() <= 2);
    }

    #[test]
    fn acknowledging_one_shared_payload_lease_keeps_other_lease_live() {
        let store = PreviewStore::new(4, 1024, Duration::from_secs(30));
        let bytes = Arc::<[u8]>::from(vec![1, 2, 3]);
        let first_result = store.insert(bytes.clone(), "image/png");
        assert!(first_result.is_ok(), "first preview lease should be inserted");
        let first = first_result.unwrap_or_default();
        let second_result = store.insert(bytes, "image/png");
        assert!(second_result.is_ok(), "second preview lease should be inserted");
        let second = second_result.unwrap_or_default();
        assert_ne!(first, second);
        assert!(store.acknowledge(&first));
        assert!(store.get(&first).is_none());
        assert_eq!(store.get(&second).map(|value| value.bytes.to_vec()), Some(vec![1, 2, 3]));
    }

    #[test]
    fn malformed_or_expired_handles_are_not_served() {
        let store = PreviewStore::new(4, 1024, Duration::from_millis(1));
        let Ok(handle) = store.insert(Arc::<[u8]>::from(vec![1]), "image/png") else { return };
        assert!(store.get("/tmp/photo.png").is_none());
        std::thread::sleep(Duration::from_millis(3));
        assert!(store.get(&handle).is_none());
    }

    #[test]
    fn oversized_payload_is_rejected_without_a_dangling_handle() {
        let store = PreviewStore::new(4, 2, Duration::from_secs(30));
        assert!(store.insert(Arc::<[u8]>::from(vec![1, 2, 3]), "image/png").is_err());
        assert!(store.is_empty());
    }
}
