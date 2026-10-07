import { convertFileSrc as tauriConvertFileSrc } from '@tauri-apps/api/core';
import { invoke } from '@tauri-apps/api/core';
import type { DesktopSnapshot, JsonObject, PreviewDescriptor, PreviewRequest, ViewSlice } from './types';

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

export function requestPreview(request: PreviewRequest): Promise<PreviewDescriptor> {
  return invoke<PreviewDescriptor>('lc_preview', { request });
}

export function acknowledgePreview(handle: string): Promise<unknown> {
  return invoke('lc_preview_ack', { handle });
}

export function nativeAction(action: string, params: JsonObject = {}): Promise<unknown> {
  return invoke('lc_native', { action, params });
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
