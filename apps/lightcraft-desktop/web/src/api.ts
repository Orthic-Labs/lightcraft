import { convertFileSrc as tauriConvertFileSrc } from '@tauri-apps/api/core';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { DesktopSnapshot, JsonObject, PreviewDescriptor, PreviewRequest, ViewMode, ViewSlice } from './types';

/** Thin typed IPC boundary. Rust remains source of truth for catalog, edits, jobs & preferences. */
export function getSnapshot(): Promise<DesktopSnapshot> {
  return invoke<DesktopSnapshot>('lc_snapshot');
}

export function runCommand(id: string, params: JsonObject = {}): Promise<unknown> {
  return invoke('lc_run', { id, params });
}

export function getViewSlice(generation: number | undefined, offset: number, limit: number): Promise<ViewSlice> {
  return invoke<ViewSlice>('lc_view_slice', { generation, offset, limit });
}

function scopedPreviewSlot(slot: string): string {
  let label = 'main';
  try {
    label = getCurrentWindow().label;
  } catch {
    // Browser harnesses have no Tauri window; retain deterministic main namespace.
  }
  label = label === 'second-main' ? 'second-main' : 'main';
  const suffix = `__${label}`;
  return `${slot.slice(0, 96 - suffix.length)}${suffix}`;
}

export async function requestPreview(request: PreviewRequest): Promise<PreviewDescriptor> {
  // Renderer slots are shared by all WebViews over one Session. Namespace requests by
  // native window, then restore logical slot for usePreview's freshness check.
  const scopedSlot = scopedPreviewSlot(request.slot);
  const descriptor = await invoke<PreviewDescriptor>('lc_preview', { request: { ...request, slot: scopedSlot } });
  if (descriptor.slot !== scopedSlot) throw new Error('Preview response slot did not match request');
  return { ...descriptor, slot: request.slot };
}

export function acknowledgePreview(handle: string): Promise<unknown> {
  return invoke('lc_preview_ack', { handle });
}

export function nativeAction(action: string, params: JsonObject = {}): Promise<unknown> {
  return invoke('lc_native', { action, params });
}

/** Open shared-session second window with independent UI bootstrap state. */
export function openSecondWindow(view: ViewMode = 'detail', active: number | null = null): Promise<unknown> {
  return nativeAction('secondWindow', { view, ...(active === null ? {} : { active }) });
}

export function getPreferences(): Promise<JsonObject> {
  return invoke<JsonObject>('lc_preferences', { patch: null });
}

export function savePreferences(patch: JsonObject): Promise<JsonObject> {
  return invoke<JsonObject>('lc_preferences', { patch });
}

/** Scoped preview protocol URL; handles are opaque and never interpreted as file paths. */
export function convertFileSrc(handle: string, protocol = 'lightcraft-preview'): string {
  return tauriConvertFileSrc(handle, protocol);
}
