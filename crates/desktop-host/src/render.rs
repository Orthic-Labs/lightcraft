//! Desktop preview renderer: bounded requests, detached engine jobs, and presentation gating.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::collections::HashMap;
use std::sync::Arc;

use lightcraft_catalog::PhotoId;
use lightcraft_engine::Session;
use lightcraft_engine::media::{QuickJob, RenderJob, RenderResult};
use lightcraft_pipeline::StageCache;
use lightcraft_preview::JobPool;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::preview::{PreviewStore, encode_png};

const MAX_DIMENSION: u32 = 8192;
const MAX_PIXELS: u64 = 16 * 1024 * 1024;
const MAX_SLOT_BYTES: usize = 96;
const MAX_PENDING: usize = 96;
const MAX_SLOTS: usize = 256;
const MAX_THUMBS_PER_PHOTO: usize = 4;
const MAX_THUMBS: usize = 512;
const MAX_SEQUENCE: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PreviewQuality {
    Draft,
    Full,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewRequest {
    pub photo_id: u64,
    pub slot: String,
    pub view_generation: u64,
    pub width: u32,
    pub height: u32,
    pub quality: PreviewQuality,
    pub before: bool,
    pub sequence: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewDescriptor {
    pub handle: String,
    pub photo_id: u64,
    pub slot: String,
    pub view_generation: u64,
    pub sequence: u64,
    pub key: String,
    pub width: u32,
    pub height: u32,
    pub histogram: Value,
    pub render_ms: f64,
    pub encoding: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct SlotId(u64);

struct Pending {
    ticket: u64,
    request: PreviewRequest,
    key: u64,
    priority: u32,
    reply: std::sync::mpsc::Sender<Result<PreviewDescriptor, String>>,
}

struct JobOutput {
    ticket: u64,
    view_generation: u64,
    sequence: u64,
    result: RenderResult,
    payload: Result<EncodedPreview, String>,
}

struct EncodedPreview {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    histogram: Value,
}

struct CachedThumb {
    key: u64,
    handle: String,
    width: u32,
    height: u32,
    histogram: Value,
}

/// Host-side render port. It is owned by the Session owner thread; worker jobs contain no Session.
pub struct Renderer {
    store: PreviewStore,
    pool: JobPool<SlotId, JobOutput>,
    slots: HashMap<String, SlotId>,
    next_slot: u64,
    next_ticket: u64,
    pending: HashMap<SlotId, Pending>,
    failed: HashMap<(SlotId, u64), String>,
    stages: HashMap<SlotId, Arc<StageCache>>,
    thumbs: HashMap<u64, Vec<CachedThumb>>,
    thumb_count: usize,
}

impl Renderer {
    pub fn new(store: PreviewStore) -> Self {
        let threads = lightcraft_preview::JobPool::<SlotId, JobOutput>::default_threads().min(6);
        Self {
            store,
            pool: JobPool::new(threads),
            slots: HashMap::new(),
            next_slot: 1,
            next_ticket: 1,
            pending: HashMap::new(),
            failed: HashMap::new(),
            stages: HashMap::new(),
            thumbs: HashMap::new(),
            thumb_count: 0,
        }
    }

    pub fn store(&self) -> PreviewStore {
        self.store.clone()
    }

    /// Queue a render. A newer sequence replaces a queued request for same slot.
    pub fn request(
        &mut self,
        session: &mut Session,
        request: PreviewRequest,
        reply: std::sync::mpsc::Sender<Result<PreviewDescriptor, String>>,
    ) -> Result<(), String> {
        if let Err(error) = validate_request(&request) {
            let _ = reply.send(Err(error));
            return Ok(());
        }
        let slot = self.slot_id(&request.slot);
        if self.pending.get(&slot).is_some_and(|pending| pending.request.sequence >= request.sequence) {
            let _ = reply.send(Err("preview request is stale".to_string()));
            return Ok(());
        }
        let photo = PhotoId(request.photo_id);
        let job = session
            .render_job(photo, request.width as usize, request.height as usize, request.before, true)
            .ok_or_else(|| "photo is unavailable".to_string());
        let mut job = match job {
            Ok(job) => job,
            Err(error) => {
                let _ = reply.send(Err(error));
                return Ok(());
            }
        };
        if request.quality == PreviewQuality::Draft {
            job = job.draft();
        }
        if !request.before && self.is_loupe_slot(&request.slot) {
            if let Some(loupe) = session.loupe_job(photo, request.width as usize, request.height as usize, true) {
                job = if request.quality == PreviewQuality::Draft { loupe.draft() } else { loupe };
            }
        }
        let key = job.key;
        if let Some(previous) = self.pending.remove(&slot) {
            let _ = previous.reply.send(Err("preview superseded".to_string()));
            self.drop_queued();
        }
        if let Some(error) = self.failed.get(&(slot, key)).cloned() {
            let _ = reply.send(Err(error));
            return Ok(());
        }
        if self.is_thumb_slot(&request.slot)
            && let Some(descriptor) = self.cached_thumb(photo.0, key, &request)
        {
            let _ = reply.send(Ok(descriptor));
            return Ok(());
        }
        let stages = self.stages.entry(slot).or_default().clone();
        self.submit(slot, request, key, job.with_stages(stages), reply);
        self.apply_pressure();
        Ok(())
    }

    /// Queue engine's existing quick stand-in for a slot. The regular `request` remains the
    /// authoritative path for final output; callers use this for an immediate first paint.
    pub fn request_quick(
        &mut self,
        session: &mut Session,
        request: PreviewRequest,
        reply: std::sync::mpsc::Sender<Result<PreviewDescriptor, String>>,
    ) -> Result<(), String> {
        if let Err(error) = validate_request(&request) {
            let _ = reply.send(Err(error));
            return Ok(());
        }
        if request.before {
            return self.request(session, request, reply);
        }
        let slot = self.slot_id(&request.slot);
        if self.pending.get(&slot).is_some_and(|pending| pending.request.sequence >= request.sequence) {
            let _ = reply.send(Err("preview request is stale".to_string()));
            return Ok(());
        }
        let job: QuickJob = session
            .quick_view_job(PhotoId(request.photo_id), request.width.max(request.height) as usize, true)
            .ok_or_else(|| "photo is unavailable".to_string());
        let job = match job {
            Ok(job) => job,
            Err(error) => {
                let _ = reply.send(Err(error));
                return Ok(());
            }
        };
        let key = job.key;
        if let Some(previous) = self.pending.remove(&slot) {
            let _ = previous.reply.send(Err("preview superseded".to_string()));
            self.drop_queued();
        }
        self.submit_quick(slot, request, key, job, reply);
        self.apply_pressure();
        Ok(())
    }

    /// Drain completed jobs. Session accepts every result, while only current request metadata is
    /// allowed through the presentation gate.
    pub fn poll(&mut self, session: &mut Session) -> usize {
        let mut completed = 0;
        while let Some(done) = self.pool.try_recv() {
            completed += 1;
            let slot = done.slot;
            let JobOutput { ticket, view_generation, sequence, result, payload } = done.result;
            session.accept(&result);
            let Some(pending) = self.pending.get(&slot) else { continue };
            let (visible_generation, _) = session.visible_shared();
            if pending.request.view_generation != visible_generation {
                if let Some(stale) = self.pending.remove(&slot) {
                    let _ = stale.reply.send(Err("preview view is stale".to_string()));
                }
                self.drop_queued();
                continue;
            }
            if !publishable(pending, ticket, view_generation, sequence, result.key, result.photo) {
                continue;
            }
            let Some(pending) = self.pending.remove(&slot) else { continue };
            let payload = match payload {
                Ok(payload) => payload,
                Err(error) => {
                    self.failed.insert((slot, result.key), error.clone());
                    let _ = pending.reply.send(Err(error));
                    continue;
                }
            };
            let handle = match self.store.insert(Arc::<[u8]>::from(payload.bytes), "image/png") {
                Ok(handle) => handle,
                Err(error) => {
                    self.failed.insert((slot, result.key), error.clone());
                    let _ = pending.reply.send(Err(error));
                    continue;
                }
            };
            let width = payload.width;
            let height = payload.height;
            let descriptor = PreviewDescriptor {
                handle: handle.clone(),
                photo_id: pending.request.photo_id,
                slot: pending.request.slot.clone(),
                view_generation: pending.request.view_generation,
                sequence: pending.request.sequence,
                key: format!("{:016x}", result.key),
                width,
                height,
                histogram: payload.histogram.clone(),
                render_ms: done.ms,
                encoding: "png",
            };
            self.failed.retain(|(candidate, key), _| *candidate != slot || *key != result.key);
            if self.is_thumb_slot(&pending.request.slot) {
                self.cache_thumb(pending.request.photo_id, CachedThumb { key: result.key, handle, width, height, histogram: payload.histogram });
            }
            let _ = pending.reply.send(Ok(descriptor));
        }
        completed
    }

    pub fn cancel(&mut self, slot: &str) -> bool {
        let Some(slot_id) = self.slots.get(slot).copied() else { return false };
        let Some(pending) = self.pending.remove(&slot_id) else { return false };
        let _ = pending.reply.send(Err("preview cancelled".to_string()));
        self.drop_queued();
        true
    }

    /// Finish every outstanding IPC request before the owner thread exits. Running worker jobs
    /// may still complete, but their results are accepted and discarded by the normal poll path.
    pub fn cancel_all(&mut self) -> usize {
        let pending = std::mem::take(&mut self.pending);
        let count = pending.len();
        for (_, request) in pending {
            let _ = request.reply.send(Err("preview cancelled".to_string()));
        }
        self.drop_queued();
        count
    }

    /// Shutdown alias for host owner teardown.
    pub fn shutdown(&mut self) {
        let _ = self.cancel_all();
    }

    pub fn retry(&mut self, slot: &str) {
        if let Some(slot_id) = self.slots.get(slot).copied() {
            self.failed.retain(|(candidate, _), _| *candidate != slot_id);
        }
    }

    pub fn pending(&self) -> usize {
        self.pending.len()
    }

    fn submit(
        &mut self,
        slot: SlotId,
        request: PreviewRequest,
        key: u64,
        job: RenderJob,
        reply: std::sync::mpsc::Sender<Result<PreviewDescriptor, String>>,
    ) {
        let priority = priority(&request.slot);
        let ticket = self.ticket();
        let view_generation = request.view_generation;
        let sequence = request.sequence;
        self.pool.submit(
            slot,
            key,
            priority,
            Box::new(move || {
                let result = job.run();
                let payload = encode_result(&result);
                JobOutput { ticket, view_generation, sequence, result, payload }
            }),
        );
        self.pending.insert(slot, Pending { ticket, request, key, priority, reply });
    }

    fn submit_quick(
        &mut self,
        slot: SlotId,
        request: PreviewRequest,
        key: u64,
        job: QuickJob,
        reply: std::sync::mpsc::Sender<Result<PreviewDescriptor, String>>,
    ) {
        let priority = priority(&request.slot).saturating_add(1);
        let ticket = self.ticket();
        let view_generation = request.view_generation;
        let sequence = request.sequence;
        self.pool.submit(
            slot,
            key,
            priority,
            Box::new(move || {
                let result = job.run();
                let payload = encode_result(&result);
                JobOutput { ticket, view_generation, sequence, result, payload }
            }),
        );
        self.pending.insert(slot, Pending { ticket, request, key, priority, reply });
    }

    fn ticket(&mut self) -> u64 {
        let ticket = self.next_ticket;
        self.next_ticket = self.next_ticket.wrapping_add(1).max(1);
        ticket
    }

    fn slot_id(&mut self, name: &str) -> SlotId {
        if let Some(slot) = self.slots.get(name) {
            return *slot;
        }
        if self.slots.len() >= MAX_SLOTS {
            if let Some(victim) = self.slots.iter().find(|(_, slot)| !self.pending.contains_key(slot)).map(|(name, slot)| (name.clone(), *slot)) {
                self.slots.remove(&victim.0);
                self.stages.remove(&victim.1);
                self.failed.retain(|(slot, _), _| *slot != victim.1);
                self.drop_queued();
            }
        }
        let slot = SlotId(self.next_slot);
        self.next_slot = self.next_slot.wrapping_add(1).max(1);
        self.slots.insert(name.to_owned(), slot);
        slot
    }

    fn is_loupe_slot(&self, slot: &str) -> bool {
        matches!(slot, "main" | "preview" | "before" | "compare-a" | "compare-b")
    }

    fn is_thumb_slot(&self, slot: &str) -> bool {
        slot.starts_with("thumb:") || slot.starts_with("filmstrip:") || slot.starts_with("grid-") || slot.starts_with("filmstrip-")
    }

    fn cached_thumb(&mut self, photo: u64, key: u64, request: &PreviewRequest) -> Option<PreviewDescriptor> {
        let store = self.store.clone();
        let entries = self.thumbs.get_mut(&photo)?;
        let index = entries.iter().position(|entry| entry.key == key && entry.width == request.width && entry.height == request.height)?;
        let (handle, width, height, histogram) = {
            let entry = entries.get(index)?;
            (entry.handle.clone(), entry.width, entry.height, entry.histogram.clone())
        };
        if store.get(&handle).is_none() {
            entries.remove(index);
            self.thumb_count = self.thumb_count.saturating_sub(1);
            return None;
        }
        Some(PreviewDescriptor {
            handle,
            photo_id: request.photo_id,
            slot: request.slot.clone(),
            view_generation: request.view_generation,
            sequence: request.sequence,
            key: format!("{:016x}", key),
            width,
            height,
            histogram,
            render_ms: 0.0,
            encoding: "png",
        })
    }

    fn cache_thumb(&mut self, photo: u64, entry: CachedThumb) {
        let entries = self.thumbs.entry(photo).or_default();
        if let Some(old) = entries.iter_mut().find(|old| old.key == entry.key) {
            *old = entry;
            return;
        }
        entries.push(entry);
        self.thumb_count += 1;
        while entries.len() > MAX_THUMBS_PER_PHOTO {
            entries.remove(0);
            self.thumb_count = self.thumb_count.saturating_sub(1);
        }
        while self.thumb_count > MAX_THUMBS {
            let Some(photo_id) = self.thumbs.keys().next().copied() else { break };
            let remove_photo = if let Some(items) = self.thumbs.get_mut(&photo_id) {
                let had = items.pop().is_some();
                had && items.is_empty()
            } else {
                false
            };
            self.thumb_count = self.thumb_count.saturating_sub(1);
            if remove_photo {
                self.thumbs.remove(&photo_id);
            }
        }
    }

    fn apply_pressure(&mut self) {
        while self.pending.len() > MAX_PENDING {
            let Some((slot, _)) = self
                .pending
                .iter()
                .min_by_key(|(_, pending)| (pending.priority, pending.request.sequence))
                .map(|(slot, pending)| (*slot, pending.priority))
            else {
                break;
            };
            if let Some(pending) = self.pending.remove(&slot) {
                let _ = pending.reply.send(Err("preview queue pressure".to_string()));
            }
        }
        self.drop_queued();
    }

    fn drop_queued(&self) {
        let pending = &self.pending;
        let _ = self.pool.reprioritize(|slot, priority| pending.contains_key(slot).then_some(priority));
    }
}

fn encode_result(result: &RenderResult) -> Result<EncodedPreview, String> {
    let rendered = result.rendered.as_ref().map_err(|error| error.clone())?;
    let bytes = encode_png(&rendered.image)?;
    let histogram = serde_json::to_value(&rendered.histogram).map_err(|error| format!("preview histogram: {error}"))?;
    let width = u32::try_from(rendered.image.width).map_err(|_| "preview width exceeds transport limit".to_string())?;
    let height = u32::try_from(rendered.image.height).map_err(|_| "preview height exceeds transport limit".to_string())?;
    Ok(EncodedPreview { bytes, width, height, histogram })
}

fn validate_request(request: &PreviewRequest) -> Result<(), String> {
    if request.photo_id > MAX_SEQUENCE || request.view_generation > MAX_SEQUENCE || request.sequence > MAX_SEQUENCE {
        return Err("preview identifier exceeds safe integer range".to_string());
    }
    if request.slot.is_empty()
        || request.slot.len() > MAX_SLOT_BYTES
        || !request.slot.is_ascii()
        || request.slot.bytes().any(|byte| byte == b'/' || byte == b'\\')
    {
        return Err("preview slot is invalid".to_string());
    }
    if request.width == 0 || request.height == 0 || request.width > MAX_DIMENSION || request.height > MAX_DIMENSION {
        return Err("preview dimensions are out of bounds".to_string());
    }
    if u64::from(request.width) * u64::from(request.height) > MAX_PIXELS {
        return Err("preview pixel budget exceeded".to_string());
    }
    Ok(())
}

fn priority(slot: &str) -> u32 {
    if matches!(slot, "main" | "preview" | "before" | "compare-a" | "compare-b") {
        return 100;
    }
    if slot.starts_with("thumb:") || slot.starts_with("filmstrip:") || slot.starts_with("grid-") || slot.starts_with("filmstrip-") {
        return 50;
    }
    if slot.starts_with("prefetch:") {
        return 10;
    }
    25
}

fn publishable(pending: &Pending, result_ticket: u64, result_generation: u64, result_sequence: u64, result_key: u64, result_photo: PhotoId) -> bool {
    pending.ticket == result_ticket
        && pending.request.view_generation == result_generation
        && pending.request.sequence == result_sequence
        && pending.key == result_key
        && PhotoId(pending.request.photo_id) == result_photo
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_bounds_reject_untrusted_dimensions_and_slots() {
        let mut request = PreviewRequest {
            photo_id: 1,
            slot: "main".into(),
            view_generation: 1,
            width: 1,
            height: 1,
            quality: PreviewQuality::Full,
            before: false,
            sequence: 1,
        };
        assert!(validate_request(&request).is_ok());
        request.width = MAX_DIMENSION + 1;
        assert!(validate_request(&request).is_err());
        request.width = 1;
        request.slot = "/tmp/photo.png".into();
        assert!(validate_request(&request).is_err());
    }

    #[test]
    fn priorities_keep_interactive_work_ahead_of_prefetch() {
        assert!(priority("main") > priority("thumb:1"));
        assert!(priority("thumb:1") > priority("prefetch:1"));
    }

    #[test]
    fn stale_result_never_passes_presentation_gate() {
        let (reply, _) = std::sync::mpsc::channel();
        let pending = Pending {
            ticket: 8,
            request: PreviewRequest {
                photo_id: 42,
                slot: "main".into(),
                view_generation: 3,
                width: 1,
                height: 1,
                quality: PreviewQuality::Full,
                before: false,
                sequence: 9,
            },
            key: 17,
            priority: 100,
            reply,
        };
        assert!(publishable(&pending, 8, 3, 9, 17, PhotoId(42)));
        assert!(!publishable(&pending, 9, 3, 9, 17, PhotoId(42)));
        assert!(!publishable(&pending, 8, 4, 9, 17, PhotoId(42)));
        assert!(!publishable(&pending, 8, 3, 10, 17, PhotoId(42)));
        assert!(!publishable(&pending, 8, 3, 9, 17, PhotoId(43)));
    }
}
