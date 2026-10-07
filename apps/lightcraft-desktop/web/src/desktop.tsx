import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import { listen } from '@tauri-apps/api/event';
import { getSnapshot, nativeAction, runCommand, savePreferences } from './api';
import { applyUiCommand, UI_COMMAND_IDS } from './commands';
import type { DesktopContextValue, DesktopSnapshot, DialogState, JsonObject, UiState } from './types';

const MIN_STAGE = 360;
const DEFAULT_UI: UiState = {
  view: 'photoGrid', panel: 'info', tool: '', zoom: 'fit', clickZoom: 1, beforeAfter: 'off',
  sidebarCollapsed: false, inspectorCollapsed: false, sidebarWidth: 268, inspectorWidth: 300,
  thumbSize: 180, filmstrip: false, filterBar: true, referenceId: null, compareIds: [], navigator: false,
  slideshow: false, infoOverlay: 0, maskOverlay: false, maskOverlayMode: 'selected', maskPins: true,
  clipping: false, theme: 'system', locale: 'en', sections: { light: true, color: true, effects: true, detail: false, optics: false },
  autoAdvance: false, gridInfo: true, softProof: false, brushSize: 100, brushFeather: 50, cropOverlay: 'thirds', filterText: '',
};

const DesktopContext = createContext<DesktopContextValue | null>(null);

function messageOf(reason: unknown): string {
  if (reason instanceof Error) return reason.message || 'Desktop session unavailable';
  if (typeof reason === 'string') return reason;
  if (reason && typeof reason === 'object' && 'message' in reason) return String((reason as { message?: unknown }).message);
  return 'Desktop session unavailable';
}

function mergeUi(value: unknown): Partial<UiState> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return {};
  const source = value as Record<string, unknown>;
  const next: Partial<UiState> = {};
  (Object.keys(DEFAULT_UI) as Array<keyof UiState>).forEach((key) => {
    const candidate = source[key];
    if (candidate !== undefined) (next[key] as unknown) = candidate;
  });
  return next;
}

function clampUi(next: Partial<UiState>, current: UiState): Partial<UiState> {
  const result = { ...next } as Partial<UiState>;
  if (typeof result.sidebarWidth === 'number') result.sidebarWidth = Math.max(220, Math.min(420, result.sidebarWidth));
  if (typeof result.inspectorWidth === 'number') result.inspectorWidth = Math.max(260, Math.min(520, result.inspectorWidth));
  if (typeof result.sidebarWidth === 'number' || typeof result.inspectorWidth === 'number') {
    const width = typeof window === 'undefined' ? 1600 : window.innerWidth;
    const sidebar = result.sidebarWidth ?? current.sidebarWidth;
    const inspector = result.inspectorWidth ?? current.inspectorWidth;
    const shellSidebar = current.sidebarCollapsed ? 48 : sidebar;
    const maxPanels = Math.max(220 + 260, width - MIN_STAGE - shellSidebar);
    if (sidebar + inspector > maxPanels) {
      if (next.inspectorWidth !== undefined) result.sidebarWidth = Math.max(220, maxPanels - inspector);
      else result.inspectorWidth = Math.max(260, maxPanels - sidebar);
    }
  }
  return result;
}

function preferenceUi(preferences: JsonObject | undefined): Partial<UiState> {
  if (!preferences) return {};
  return mergeUi(preferences.ui ?? preferences.layout ?? preferences);
}

function dialogForCommand(id: string): DialogState | null {
  if (id === 'app.settings') return { kind: 'settings' };
  if (id === 'app.about') return { kind: 'about' };
  if (id === 'app.systemInfo') return { kind: 'systemInfo' };
  if (id === 'app.whatsNew') return { kind: 'whatsNew' };
  if (id === 'app.shortcuts') return { kind: 'shortcuts' };
  if (id.startsWith('dialog.')) return { kind: id.slice(7) };
  return null;
}

function pathsFrom(value: unknown): string[] {
  if (typeof value === 'string') return value ? [value] : [];
  if (Array.isArray(value)) return value.filter((path): path is string => typeof path === 'string' && path.length > 0);
  if (!value || typeof value !== 'object') return [];
  const record = value as Record<string, unknown>;
  return pathsFrom(record.paths ?? record.files ?? record.selected ?? record.path);
}

function firstPath(value: unknown): string | undefined { return pathsFrom(value)[0]; }

const HELP_URLS: Record<string, string> = {
  'app.help': 'https://lightcraft.photo/help',
  'app.discord': 'https://discord.gg/artcraft',
  'app.feedback': 'https://github.com/storytold/lightcraft-can/issues/new',
  'app.website': 'https://lightcraft.photo',
  'app.github': 'https://github.com/storytold/lightcraft-can',
  'app.artcraft': 'https://artcraft.ai',
};

export function DesktopProvider({ children }: { children: ReactNode }) {
  const [snapshot, setSnapshot] = useState<DesktopSnapshot | null>(null);
  const [ui, setUiState] = useState<UiState>(DEFAULT_UI);
  const [dialog, setDialog] = useState<DialogState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [histogram, setHistogram] = useState<unknown>(null);
  const prefHydrated = useRef(false);
  const refreshInFlight = useRef<Promise<void> | null>(null);

  const refresh = useCallback(async () => {
    if (refreshInFlight.current) return refreshInFlight.current;
    const operation = getSnapshot().then((next) => {
      setSnapshot(next);
      setError(null);
      if (!prefHydrated.current) {
        prefHydrated.current = true;
        setUiState((current) => ({ ...current, ...preferenceUi(next.preferences) }));
      }
      const statusNotice = next.status?.notices?.at(-1);
      if (statusNotice) setNotice(statusNotice);
    }).catch((reason: unknown) => {
      setError(messageOf(reason));
    }).finally(() => {
      refreshInFlight.current = null;
    });
    refreshInFlight.current = operation;
    return operation;
  }, []);

  useEffect(() => { void refresh(); }, [refresh]);

  useEffect(() => {
    let timer: number | undefined;
    let live = true;
    const tick = () => {
      if (!live) return;
      void refresh();
      const active = snapshot?.status.importing || snapshot?.status.exporting || snapshot?.status.previewBuild || (snapshot?.status.jobs.length ?? 0) > 0;
      timer = window.setTimeout(tick, active ? 250 : 1200);
    };
    timer = window.setTimeout(tick, 1200);
    return () => { live = false; if (timer !== undefined) window.clearTimeout(timer); };
  }, [refresh, snapshot?.status.exporting, snapshot?.status.importing, snapshot?.status.jobs.length, snapshot?.status.previewBuild]);

  useEffect(() => {
    if (!prefHydrated.current) return;
    const timer = window.setTimeout(() => { void savePreferences({ ui: ui as unknown as JsonObject }).catch((reason: unknown) => setError(messageOf(reason))); }, 350);
    return () => window.clearTimeout(timer);
  }, [ui]);

  const setUi = useCallback((patch: Partial<UiState> | ((current: UiState) => Partial<UiState>)) => {
    setUiState((current) => {
      const change = typeof patch === 'function' ? patch(current) : patch;
      return { ...current, ...clampUi(change, current) };
    });
  }, []);

  const run = useCallback(async (id: string, params: JsonObject = {}): Promise<unknown> => {
    try {
      if ((UI_COMMAND_IDS as readonly string[]).includes(id)) {
        const nextDialog = dialogForCommand(id);
        if (nextDialog) setDialog(nextDialog);
        setUi((current) => applyUiCommand(id, current) ?? {});
        if (id === 'view.secondWindow') return nativeAction('secondWindow', params);
        if (id === 'view.fullScreenPreview') return nativeAction('toggleFullscreen', { ...params, enabled: params.enabled });
        if (id === 'view.enterFullScreen') return nativeAction('fullscreen', params);
        if (id === 'app.quit') return nativeAction('closeWindow', params);
        if (id === 'app.help' || id === 'app.discord' || id === 'app.feedback' || id === 'app.website' || id === 'app.github' || id === 'app.artcraft') {
          return nativeAction('openUrl', { url: HELP_URLS[id] });
        }
        if (id === 'app.openLibrary') {
          const selected = firstPath(params.path) || firstPath(await nativeAction('pickFolder', params));
          if (!selected) return null;
          const value = await runCommand('library.browse', { ...params, path: selected });
          await refresh();
          return value;
        }
        if (id === 'file.addPhotos' || id === 'file.addFolder' || id === 'file.addFromDevice') {
          const action = id === 'file.addFolder' ? 'pickFolder' : id === 'file.addFromDevice' ? 'pickDevice' : 'pickFiles';
          const picked = pathsFrom(params.paths ?? params.path) .length ? (params.paths ?? params.path) : await nativeAction(action, params);
          const paths = pathsFrom(picked);
          if (!paths.length) return null;
          const report = await runCommand('library.importPreview', { paths });
          setDialog({ kind: 'import', params: { paths, mode: id === 'file.addFromDevice' ? 'copy' : 'add', candidates: (report as JsonObject)?.candidates } });
          return report;
        }
        if (id === 'file.importLightroom') {
          const path = firstPath(params.path) || firstPath(await nativeAction('pickLightroomCatalog', params));
          if (!path) return null;
          const report = await runCommand('library.inspectLightroom', { path });
          setDialog({ kind: 'lightroom', params: { path, report } });
          return report;
        }
        if (id === 'file.findMissing') {
          const folder = firstPath(params.folder) || firstPath(await nativeAction('chooseFolder', params));
          if (!folder) return null;
          const value = await runCommand('library.findMissing', { ...params, folder });
          await refresh();
          return value;
        }
        if (id === 'file.backupLibrary' || id === 'file.restoreLibrary') {
          return nativeAction(id === 'file.backupLibrary' ? 'backupLibrary' : 'restoreLibrary', params);
        }
        if (id === 'photo.locate') {
          const idValue = params.id ?? snapshot?.active;
          if (typeof idValue !== 'number') throw new Error('no photo selected');
          const path = firstPath(params.path) || firstPath(await nativeAction('chooseFiles', { ...params, multiple: false }));
          if (!path) return null;
          const value = await runCommand('photo.relink', { ...params, id: idValue, path });
          await refresh();
          return value;
        }
        if (id === 'photo.editInExternal') {
          const result = await runCommand('photo.editExternal', params) as JsonObject;
          const path = firstPath(result?.path);
          if (path) await nativeAction('openExternalEditor', { path, app: params.app });
          return result;
        }
        if (id === 'app.showInFinder') {
          const inspected = await runCommand('photo.inspect', params) as JsonObject;
          const path = firstPath(inspected?.path ?? inspected?.filePath);
          if (!path) throw new Error('active photo has no file path');
          return nativeAction('reveal', { path });
        }
        if (id === 'photo.tagFromTracklog') {
          const path = firstPath(params.path) || firstPath(await nativeAction('pickTracklog', params));
          if (!path) return null;
          if (params.offset === undefined) {
            setDialog({ kind: 'textPrompt', params: { title: 'Auto-Tag from Tracklog', hint: 'Camera time zone, e.g. -07:00 (empty: UTC)', command: 'photo.autoTagTracklog', key: 'offset', path } });
            return null;
          }
          const value = await runCommand('photo.autoTagTracklog', { ...params, path });
          await refresh();
          return value;
        }
        if (id === 'file.importPresets') {
          const selected = pathsFrom(params.paths).length ? params.paths : await nativeAction('pickPresetFiles', params);
          const paths = pathsFrom(selected);
          if (!paths.length) return null;
          const value = await runCommand('preset.import', { paths });
          await refresh();
          return value;
        }
        if (id === 'file.exportPresets' || id === 'file.exportCurvePresets') {
          const curve = id === 'file.exportCurvePresets';
          const path = firstPath(params.path) || firstPath(await nativeAction(curve ? 'saveCurvePresetFile' : 'savePresetFile', params));
          if (!path) return null;
          const value = await runCommand(curve ? 'curve.exportPresets' : 'preset.export', { ...params, path });
          await refresh();
          return value;
        }
        if (id === 'file.importCurvePresets') {
          const selected = pathsFrom(params.paths).length ? params.paths : await nativeAction('pickCurvePresetFiles', params);
          const paths = pathsFrom(selected);
          if (!paths.length) return null;
          const value = await runCommand('curve.importPresets', { paths });
          await refresh();
          return value;
        }
        if (id === 'app.systemInfo') {
          const info = await runCommand('library.info', params);
          setDialog({ kind: 'systemInfo', params: { rows: Array.isArray(info) ? info : [] } });
          return info;
        }
        if (id === 'local.addRoot') {
          const path = firstPath(params.path) || firstPath(await nativeAction('chooseFolder', params));
          if (!path) return null;
          const value = await runCommand('local.addRoot', { ...params, path });
          await refresh();
          return value;
        }
        if (id === 'local.hide' || id === 'local.restoreHidden' || id === 'compare.swap' || id === 'compare.makeSelect') {
          const value = await runCommand(id, params);
          await refresh();
          return value;
        }
        if (id === 'photo.setReference') {
          const referenceId = typeof params.id === 'number' ? params.id : snapshot?.active;
          if (typeof referenceId === 'number') setUi({ referenceId });
          return { reference: referenceId ?? null };
        }
        if (id === 'view.reference' && snapshot?.active !== null && snapshot?.active !== undefined) {
          setUi({ referenceId: snapshot.active });
          return { reference: snapshot.active };
        }
        if (id === 'dialog.allMetadata') {
          const metadata = await runCommand('photo.allMetadata', params);
          setDialog({ kind: 'allMetadata', params: { metadata } });
          return metadata;
        }
        if (id === 'app.export' || id === 'app.exportPrevious') {
          const value = await runCommand(id, params);
          await refresh();
          return value;
        }
        if (id === 'merge.hdrLast' || id === 'merge.panoramaLast' || id === 'merge.hdrPanoramaLast') {
          const kind = id.slice('merge.'.length).replace('Last', '');
          setDialog({ kind: `merge.${kind}`, params: { ...params, last: true } });
          return null;
        }
        if (id === 'view.focusSearch') {
          setUi({ filterBar: true });
          window.requestAnimationFrame(() => document.querySelector<HTMLInputElement>('.lc-filter-search input')?.focus());
          return null;
        }
        if (id === 'view.back' && dialog) { setDialog(null); return null; }
        if (id === 'dialog.export') { setDialog({ kind: 'export', params }); return null; }
        if (id === 'dialog.copySettings') { setDialog({ kind: 'copySettings', params }); return null; }
        if (id === 'dialog.pasteSettings') { setDialog({ kind: 'pasteSettings', params }); return null; }
        if (id === 'dialog.mergeHdr' || id === 'dialog.mergePanorama' || id === 'dialog.mergeHdrPanorama') {
          const kind = id === 'dialog.mergeHdr' ? 'merge.hdr' : id === 'dialog.mergePanorama' ? 'merge.panorama' : 'merge.hdrPanorama';
          setDialog({ kind, params });
          return null;
        }
        if (id === 'dialog.saveMetadataPreset') { setDialog({ kind: 'saveMetadataPreset', params }); return null; }
        if (id === 'dialog.cull') { setDialog({ kind: 'cull', params }); return null; }
        return null;
      }
      const value = await runCommand(id, params);
      await refresh();
      return value;
    } catch (reason: unknown) {
      const text = messageOf(reason);
      setError(text);
      throw reason;
    }
  }, [refresh, setUi]);

  const native = useCallback(async (action: string, params: JsonObject = {}): Promise<unknown> => {
    try {
      const result = await nativeAction(action, params);
      setError(null);
      return result;
    } catch (reason: unknown) {
      const text = messageOf(reason);
      setError(text);
      throw reason;
    }
  }, []);

  useEffect(() => {
    let live = true;
    let unlisten: Array<() => void> = [];
    void Promise.all([
      listen<JsonObject>('lc://native-drop', (event) => {
        if (!live) return;
        const paths = pathsFrom(event.payload);
        if (!paths.length) return;
        void nativeAction('dropImport', { paths }).then((validated) => {
          const accepted = pathsFrom(validated);
          if (accepted.length) void run('file.addPhotos', { paths: accepted });
        }).catch((reason: unknown) => setError(messageOf(reason)));
      }),
      listen<JsonObject>('lc://error', (event) => { if (live) setError(messageOf(event.payload)); }),
      listen<JsonObject>('lc://notice', (event) => { if (live) setNotice(messageOf(event.payload)); }),
      listen<JsonObject>('lc://close-requested', (event) => {
        if (!live || event.payload?.unsaved !== true) return;
        setDialog({ kind: 'unsavedQuit', params: { ...event.payload } });
        void refresh();
      }),
    ]).then((stops) => { if (live) unlisten = stops; else stops.forEach((stop) => stop()); }).catch(() => undefined);
    return () => { live = false; unlisten.forEach((stop) => stop()); };
  }, [run]);

  const value = useMemo<DesktopContextValue>(() => ({ snapshot, ui, setUi, run, native, refresh, dialog, setDialog, error, notice, setNotice, histogram, setHistogram }), [dialog, error, histogram, native, notice, refresh, run, setUi, snapshot, ui]);
  return <DesktopContext.Provider value={value}>{children}</DesktopContext.Provider>;
}

export function useDesktop(): DesktopContextValue {
  const value = useContext(DesktopContext);
  if (!value) throw new Error('useDesktop must be used inside DesktopProvider');
  return value;
}
