import React, { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { useDesktop } from '../desktop';
import type { DesktopContextValue, DialogState, JsonObject, UiState } from '../types';
import './dialogs.css';

type AnyRecord = Record<string, any>;
type HostProps = { desktop?: DesktopContextValue };

const text = (v: unknown, fallback = '') => typeof v === 'string' ? v : fallback;
const bool = (v: unknown, fallback = false) => typeof v === 'boolean' ? v : fallback;
const num = (v: unknown, fallback = 0) => typeof v === 'number' && Number.isFinite(v) ? v : fallback;
const arr = (v: unknown): any[] => Array.isArray(v) ? v : [];
const errorText = (reason: unknown) => reason instanceof Error ? reason.message : reason && typeof reason === 'object' && 'message' in reason ? text((reason as AnyRecord).message, JSON.stringify(reason)) : String(reason);
const displayValue = (value: unknown): string => {
  if (typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean') return String(value);
  if (Array.isArray(value)) return value.map(displayValue).join(' · ');
  if (value && typeof value === 'object') return Object.entries(value as AnyRecord).map(([key, item]) => `${key}: ${displayValue(item)}`).join(', ');
  return String(value ?? 'Unknown');
};

let lastNonOverlayFocus: HTMLElement | null = null;
function restorableFocusTarget(value: Element | null): value is HTMLElement {
  if (!(value instanceof HTMLElement) || !value.isConnected || value === document.body) return false;
  if (value.closest('.rk-overlay, .lc-dialog-backdrop, [aria-hidden="true"]')) return false;
  let node: HTMLElement | null = value;
  while (node) { if (node.inert) return false; node = node.parentElement; }
  return true;
}
function focusCandidate(value: EventTarget | null): Element | null {
  if (!(value instanceof Element)) return null;
  return value.matches('button,a[href],input,select,textarea,[tabindex]:not([tabindex="-1"])') ? value : value.closest('button,a[href],input,select,textarea,[tabindex]:not([tabindex="-1"])');
}
function useFocusHistory() {
  useEffect(() => {
    const remember = (event: FocusEvent) => {
      const target = focusCandidate(event.target);
      if (restorableFocusTarget(target)) lastNonOverlayFocus = target;
    };
    const rememberPointer = (event: PointerEvent) => {
      const target = focusCandidate(event.target);
      if (restorableFocusTarget(target)) lastNonOverlayFocus = target;
    };
    document.addEventListener('focusin', remember, true);
    document.addEventListener('pointerdown', rememberPointer, true);
    return () => { document.removeEventListener('focusin', remember, true); document.removeEventListener('pointerdown', rememberPointer, true); };
  }, []);
}

function useModalScope(open: boolean, onClose: () => void, dismissible = true) {
  const ref = useRef<HTMLDivElement>(null);
  const closeRef = useRef(onClose);
  const dismissibleRef = useRef(dismissible);
  closeRef.current = onClose;
  dismissibleRef.current = dismissible;
  useEffect(() => {
    if (!open) return;
    const active = document.activeElement as HTMLElement | null;
    const restoreTarget = restorableFocusTarget(active) ? active : lastNonOverlayFocus;
    const root = ref.current;
    const siblings = Array.from(document.body.children).filter((node) => node !== root?.parentElement);
    const previous = siblings.map((node) => ({ node, inert: (node as HTMLElement).inert, hidden: node.getAttribute('aria-hidden') }));
    siblings.forEach((node) => { (node as HTMLElement).inert = true; node.setAttribute('aria-hidden', 'true'); });
    const focusables = () => root ? Array.from(root.querySelectorAll<HTMLElement>('button,[href],input,select,textarea,[tabindex]:not([tabindex="-1"])')).filter((el) => !el.hasAttribute('disabled')) : [];
    focusables()[0]?.focus();
    const keydown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && dismissibleRef.current) { event.preventDefault(); closeRef.current(); return; }
      if (event.key !== 'Tab') return;
      const fields = focusables();
      if (!fields.length) return;
      const first = fields[0]; const last = fields[fields.length - 1];
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    };
    document.addEventListener('keydown', keydown);
    return () => { document.removeEventListener('keydown', keydown); previous.forEach(({ node, inert, hidden }) => { (node as HTMLElement).inert = inert; if (hidden === null) node.removeAttribute('aria-hidden'); else node.setAttribute('aria-hidden', hidden); }); if (restorableFocusTarget(restoreTarget)) restoreTarget.focus(); };
  // onClose & dismissible are interaction policy, not scope lifecycle; refs prevent input rerenders from resetting focus.
  }, [open]);
  return ref;
}

function Frame({ title, children, actions, onClose, wide = false, busy = false, dismissible = true }: { title: string; children: React.ReactNode; actions?: React.ReactNode; onClose: () => void; wide?: boolean; busy?: boolean; dismissible?: boolean }) {
  const ref = useModalScope(true, onClose, dismissible);
  const frame = <div className="lc-dialog-backdrop" role="presentation" onMouseDown={(e) => { if (e.target === e.currentTarget && dismissible) onClose(); }}>
    <section ref={ref} className={`lc-dialog ${wide ? 'lc-dialog-wide' : ''}`} role="dialog" aria-modal="true" aria-labelledby="lc-dialog-title">
      <header className="lc-dialog-header"><h2 id="lc-dialog-title">{title}</h2>{dismissible && <button className="lc-icon-button" aria-label="Close" onClick={onClose}>×</button>}</header>
      <div className="lc-dialog-body">{children}</div>
      {actions && <footer className="lc-dialog-actions">{busy && <span className="lc-dialog-busy" aria-live="polite">Working…</span>}<span className="lc-dialog-action-group">{actions}</span></footer>}
    </section>
  </div>;
  return typeof document === 'undefined' ? frame : createPortal(frame, document.body);
}

function Button({ children, primary = false, disabled = false, onClick, type = 'button' }: { children: React.ReactNode; primary?: boolean; disabled?: boolean; onClick?: () => void; type?: 'button' | 'submit' }) { return <button type={type} disabled={disabled} className={primary ? 'lc-button lc-button-primary' : 'lc-button'} onClick={onClick}>{children}</button>; }
function Field({ label, value, onChange, type = 'text', min, max, step, placeholder, disabled = false }: { label: string; value: string | number; onChange: (v: string) => void; type?: string; min?: number; max?: number; step?: number; placeholder?: string; disabled?: boolean }) { return <label className="lc-field"><span>{label}</span><input type={type} value={value} onChange={(e) => onChange(e.target.value)} min={min} max={max} step={step} placeholder={placeholder} disabled={disabled} /></label>; }
function TextArea({ label, value, onChange, placeholder }: { label: string; value: string; onChange: (v: string) => void; placeholder?: string }) { return <label className="lc-field"><span>{label}</span><textarea rows={3} value={value} onChange={(e) => onChange(e.target.value)} placeholder={placeholder} style={{ width: '100%', boxSizing: 'border-box', minHeight: 72, padding: '6px 9px', border: '1px solid var(--rk-color-border,#c8d0da)', borderRadius: 7, background: 'var(--rk-color-surface,#fff)', color: 'inherit', font: '13px Inter,system-ui,sans-serif', resize: 'vertical' }} /></label>; }
function Select({ label, value, options, onChange, disabled = false }: { label: string; value: string; options: [string, string][]; onChange: (v: string) => void; disabled?: boolean }) { return <label className="lc-field"><span>{label}</span><select value={value} onChange={(e) => onChange(e.target.value)} disabled={disabled}>{options.map(([v, l]) => <option key={v} value={v}>{l}</option>)}</select></label>; }
function Check({ children, checked, onChange, disabled = false }: { children: React.ReactNode; checked: boolean; onChange: (v: boolean) => void; disabled?: boolean }) { return <label className="lc-check"><input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} disabled={disabled} /><span>{children}</span></label>; }
function Note({ children, tone = 'normal' }: { children: React.ReactNode; tone?: 'normal' | 'warning' | 'error' }) { return <p className={`lc-note lc-note-${tone}`} role={tone === 'error' ? 'alert' : undefined}>{children}</p>; }
function Progress({ snapshot, label, onCancel }: { snapshot: AnyRecord | null; label: string; onCancel?: () => void }) { const job = snapshot?.status?.jobs?.find((j: AnyRecord) => j.kind === 'import' || j.kind === 'export' || j.kind === 'merge'); const completed = num(job?.completed); const total = Math.max(1, num(job?.total, 1)); return <div className="lc-progress" aria-live="polite"><div className="lc-progress-label"><span>{job?.label || label}</span><span>{Math.min(100, Math.round(completed / total * 100))}%</span></div><progress max={total} value={completed} />{onCancel && job?.cancellable !== false && <Button onClick={onCancel}>Cancel</Button>}</div>; }

function JobDialog({ kind, desktop, d }: { kind: string; desktop: DesktopContextValue; d: DialogState }) {
  const [done, setDone] = useState(false); const jobKind = kind.includes('import') ? 'import' : kind.includes('merge') ? 'merge' : 'export';
  useEffect(() => { let live = true; const poll = async () => { if (!live) return; await desktop.refresh(); const active = desktop.snapshot?.status?.jobs?.some((j) => j.kind === jobKind); if (!active) setDone(true); }; void poll(); const timer = window.setInterval(() => void poll(), 300); return () => { live = false; window.clearInterval(timer); }; }, [desktop, jobKind]);
  const reveal = async () => { const files = arr(d.params?.files || d.params?.paths); const path = text(files[0] || d.params?.path || d.params?.dir); if (path) await desktop.native('reveal', { path }); };
  const cancel = async () => { const job = desktop.snapshot?.status?.jobs?.find((j) => j.kind === jobKind); if (job) await desktop.run('task.cancel', { id: job.id }); };
  return <Frame title={kind.includes('import') ? 'Importing Photos' : kind.includes('merge') ? 'Merging Photos' : 'Exporting Photos'} dismissible={done} onClose={() => done && desktop.setDialog(null)} actions={<><Button disabled={!done} onClick={() => desktop.setDialog(null)}>Close</Button>{(kind.includes('export') && done) && <Button primary onClick={() => void reveal()}>Show in Folder</Button>}</>}><Progress snapshot={desktop.snapshot} label={kind.includes('import') ? 'Importing photos' : kind.includes('merge') ? 'Building merged photo' : 'Exporting photos'} onCancel={done ? undefined : () => void cancel()} />{done && <Note>Operation complete. Changes are recorded in library history.</Note>}</Frame>;
}

function UnsavedQuitDialog({ desktop, d }: { desktop: DesktopContextValue; d: DialogState }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const keepOpen = () => desktop.setDialog(null);
  const saveAndQuit = async () => {
    setBusy(true);
    setError('');
    try {
      await desktop.native('saveBeforeClose', d.params || {});
      desktop.setDialog(null);
    } catch (reason) {
      setError(errorText(reason));
    } finally {
      setBusy(false);
    }
  };
  const quitAnyway = async () => {
    setBusy(true);
    setError('');
    try {
      await desktop.native('closeWindow', { force: true });
    } catch (reason) {
      setError(errorText(reason));
      setBusy(false);
    }
  };
  return <Frame title="Quit with unsaved changes?" busy={busy} onClose={keepOpen} actions={<><Button disabled={busy} onClick={keepOpen}>Keep Open</Button><Button disabled={busy} onClick={() => void quitAnyway()}>Quit Anyway</Button><Button primary disabled={busy} onClick={() => void saveAndQuit()}>Retry Save &amp; Quit</Button></>}><Note tone="warning">Some library changes could not be saved. Try saving again before closing.</Note>{typeof d.params?.error === 'string' && !error && <Note>{d.params.error}</Note>}{error && <Note tone="error">{error}</Note>}</Frame>;
}

function ConfirmDeleteDialog({ desktop, d }: { desktop: DesktopContextValue; d: DialogState }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const command = d.params?.command === 'photo.deletePermanently' ? 'photo.deletePermanently' : 'photo.delete';
  const permanent = command === 'photo.deletePermanently';
  const confirm = async () => {
    setBusy(true);
    setError('');
    try {
      const payload: AnyRecord = { ...(d.params || {}), confirmed: true, skipConfirmation: true };
      delete payload.command;
      await desktop.run(command, payload);
      desktop.setDialog(null);
    } catch (reason) {
      setError(errorText(reason));
    } finally {
      setBusy(false);
    }
  };
  return <Frame title={permanent ? 'Delete Photos Permanently?' : 'Move Photos to Recently Deleted?'} busy={busy} onClose={() => desktop.setDialog(null)} actions={<><Button disabled={busy} onClick={() => desktop.setDialog(null)}>Cancel</Button><Button primary disabled={busy} onClick={() => void confirm()}>{permanent ? 'Delete Permanently' : 'Delete Photos'}</Button></>}><Note tone="warning">{permanent ? 'These photos cannot be restored.' : 'Photos can be restored from Recently Deleted.'}</Note>{error && <Note tone="error">{error}</Note>}</Frame>;
}

async function choose(native: DesktopContextValue['native'], action: string, params: JsonObject = {}) { const result: AnyRecord = await native(action, params) as AnyRecord; if (typeof result === 'string') return [result]; if (Array.isArray(result)) return result.filter((p) => typeof p === 'string'); return arr(result?.paths || result?.files || result?.selected || (result?.path ? [result.path] : [] )).filter((p) => typeof p === 'string'); }

function ImportDialog({ d, desktop }: { d: DialogState; desktop: DesktopContextValue }) {
  const p = d.params || {}; const [source, setSource] = useState(text(p.source)); const [mode, setMode] = useState(text(p.mode, 'add')); const [album, setAlbum] = useState(text(p.album)); const [newAlbum, setNewAlbum] = useState(''); const [keywords, setKeywords] = useState(''); const [organize, setOrganize] = useState(text(p.organize, 'date')); const [folderTemplate, setFolderTemplate] = useState(text(p.folderTemplate, '{date:%Y}/{date:%Y%m%d}')); const [destination, setDestination] = useState(text(p.destination)); const [rename, setRename] = useState(''); const [renameStart, setRenameStart] = useState('1'); const [preset, setPreset] = useState(''); const [metadataPreset, setMetadataPreset] = useState(''); const [dng, setDng] = useState(false); const [candidates, setCandidates] = useState<AnyRecord[]>(arr(p.candidates)); const [checked, setChecked] = useState<boolean[]>(arr(p.checked).length ? arr(p.checked).map(Boolean) : arr(p.candidates).map((c) => !c.duplicate && !c.error)); const [busy, setBusy] = useState(false); const [error, setError] = useState('');
  const scan = async (paths: string[]) => { if (!paths.length) return; setSource(paths.join(', ')); setBusy(true); setError(''); try { const result = (await desktop.run('library.importPreview', { paths })) as AnyRecord; setCandidates(arr(result?.candidates)); setChecked(arr(result?.candidates).map((c) => !c.duplicate && !c.error)); } catch (e) { setError(String(e)); } finally { setBusy(false); } };
  const pick = async (kind: 'files' | 'folder' | 'device') => { const paths = await choose(desktop.native, kind === 'folder' ? 'chooseFolder' : kind === 'device' ? 'chooseDevice' : 'chooseFiles', { multiple: kind === 'files' }); await scan(paths); if (kind === 'device') setMode('copy'); };
  const submit = async () => { const paths = candidates.filter((_, i) => checked[i]).map((c) => c.path).filter(Boolean); if (!paths.length) { setError('Choose at least one photo.'); return; } const custom = folderTemplate.trim(); const templateError = mode !== 'add' && organize === 'custom' ? folderTemplateError(custom) : null; if (templateError) { setError(templateError); return; } setBusy(true); setError(''); try { await desktop.run('library.import', { paths, mode, album: album ? Number(album) : undefined, albumName: newAlbum.trim() || undefined, keywords: keywords.split(',').map((v) => v.trim()).filter(Boolean), preset: preset || undefined, metadataPreset: metadataPreset || undefined, destination: destination || undefined, organize: mode === 'add' ? undefined : organize === 'custom' ? custom : organize || undefined, rename: rename || undefined, renameStart: renameStart ? Number(renameStart) : undefined, dng: dng && mode === 'copy' }); desktop.setDialog({ kind: 'importProgress', params: { paths } }); } catch (e) { setError(String(e)); } finally { setBusy(false); } };
  return <Frame title="Import Photos" wide busy={busy} onClose={() => desktop.setDialog(null)} actions={<><Button onClick={() => desktop.setDialog(null)}>Cancel</Button><Button primary disabled={busy || !candidates.length} onClick={() => void submit()}>Import {candidates.filter((_, i) => checked[i]).length || ''} Photos</Button></>}><div className="lc-dialog-toolbar"><Button onClick={() => void pick('files')}>Choose Photos…</Button><Button onClick={() => void pick('folder')}>Choose Folder…</Button><Button onClick={() => void pick('device')}>Camera or Card…</Button></div>{source && <Note>Source: {source}</Note>}{error && <Note tone="error">{error}</Note>}<div className="lc-import-layout"><div className="lc-candidate-list"><div className="lc-section-heading"><strong>Review</strong><span>{candidates.length} found</span></div>{candidates.length ? candidates.map((c, i) => <label className={`lc-candidate ${c.duplicate || c.error ? 'lc-candidate-muted' : ''}`} key={`${c.path || c.name || i}-${i}`}><input type="checkbox" checked={checked[i] ?? false} disabled={Boolean(c.duplicate || c.error)} onChange={(e) => setChecked((old) => old.map((v, n) => n === i ? e.target.checked : v))} /><span className="lc-thumb-placeholder" aria-hidden="true">{text(c.format, 'IMG').slice(0, 3).toUpperCase()}</span><span className="lc-candidate-copy"><strong>{text(c.name, text(c.path, 'Photo'))}</strong><small>{text(c.path)}{c.duplicate ? ` · Duplicate (${c.duplicate})` : c.error ? ` · ${c.error}` : ''}</small></span></label>) : <Note>Choose photos or a folder to review files before adding them.</Note>}</div><div className="lc-form-stack"><Select label="Add photos" value={mode} options={[["add", 'In place'], ['copy', 'Copy into library'], ['move', 'Move into library']]} onChange={setMode} /><Field label="Existing album ID" value={album} onChange={setAlbum} placeholder="Optional" /><Field label="New album" value={newAlbum} onChange={setNewAlbum} placeholder="Optional" /><Field label="Keywords" value={keywords} onChange={setKeywords} placeholder="Comma-separated" />{mode !== 'add' && <><Field label="Destination folder" value={destination} onChange={setDestination} placeholder="Library Originals by default" /><Select label="Organize copies" value={organize} options={[["date", 'By capture date'], ['month', 'By month'], ['flat', 'One folder'], ['custom', 'Custom template']]} onChange={setOrganize} />{organize === 'custom' && <><Field label="Folder template" value={folderTemplate} onChange={setFolderTemplate} placeholder="{date:%Y}/{date:%Y%m%d}" /><Note>Use relative folders & photo tags such as {`{date:%Y}`}; no drive or parent folders.</Note></>}<Field label="File naming" value={rename} onChange={setRename} placeholder="Keep original names" /><Check checked={dng} onChange={setDng} disabled={mode !== 'copy'}>Copy raw files as DNG</Check></>}<Field label="Develop preset ID" value={preset} onChange={setPreset} placeholder="Optional" /><Field label="Metadata preset" value={metadataPreset} onChange={setMetadataPreset} placeholder="Optional" /></div></div></Frame>;
}

function folderTemplateError(template: string): string | null { const value = template.trim(); if (!value) return 'Enter a folder template.'; if (value.startsWith('/') || value.startsWith('\\') || value.startsWith('~') || /^[A-Za-z]:/.test(value)) return 'Folder template must stay inside destination folder.'; if (value.split(/[\\/]/).some((part) => part.trim() === '.' || part.trim() === '..')) return 'Folder template cannot contain . or .. folders.'; return null; }

function LightroomDialog({ d, desktop }: { d: DialogState; desktop: DesktopContextValue }) {
  const [path, setPath] = useState(text(d.params?.path)); const [report, setReport] = useState<AnyRecord | null>((d.params?.report as AnyRecord) || null); const [updateExisting, setUpdateExisting] = useState(bool(d.params?.updateExisting)); const [busy, setBusy] = useState(false); const [error, setError] = useState('');
  const inspect = async (chosen?: string) => { const catalog = chosen || path; if (!catalog) return; setPath(catalog); setBusy(true); setError(''); try { setReport(await desktop.run('library.inspectLightroom', { path: catalog }) as AnyRecord); } catch (e) { setError(String(e)); } finally { setBusy(false); } };
  const importCatalog = async () => { if (!path) return; setBusy(true); setError(''); try { const result = await desktop.run('library.importLightroom', { path, updateExisting }); desktop.setDialog({ kind: 'lightroomResult', params: { path, updateExisting, result: (result as AnyRecord) || {} } }); } catch (e) { setError(String(e)); } finally { setBusy(false); } };
  return <Frame title="Import Lightroom Catalog" wide busy={busy} onClose={() => desktop.setDialog(null)} actions={<><Button onClick={() => desktop.setDialog(null)}>Cancel</Button><Button primary disabled={!report || busy} onClick={() => void importCatalog()}>Import Catalog</Button></>}><div className="lc-dialog-toolbar"><Button onClick={async () => { const paths = await choose(desktop.native, 'pickLightroomCatalog'); if (paths[0]) void inspect(paths[0]); }}>Choose .lrcat…</Button></div><Field label="Catalog" value={path} onChange={setPath} placeholder="Select a Lightroom .lrcat catalog" />{error && <Note tone="error">{error}</Note>}{report ? <><div className="lc-summary-grid"><div><strong>{num(report.photos)}</strong><span>Photos</span></div><div><strong>{num(report.collections)}</strong><span>Collections</span></div><div><strong>{arr(report.missing).length}</strong><span>Missing originals</span></div><div><strong>{arr(report.warnings).length}</strong><span>Warnings</span></div></div><Check checked={updateExisting} onChange={setUpdateExisting}>Update existing edits when source catalog has changes</Check>{arr(report.missing).length > 0 && <Note tone="warning">Missing originals stay in catalog for later relinking.</Note>}{arr(report.warnings).length > 0 && <details className="lc-details"><summary>Review warnings</summary><ul>{arr(report.warnings).map((warning, i) => <li key={i}>{displayValue(warning)}</li>)}</ul></details>}</> : <Note>LightCraft reads catalog data without changing source catalog. Inspect first to review missing originals & warnings.</Note>}</Frame>;
}

function watermarkDefaults(value: unknown): AnyRecord {
  const source = typeof value === 'string' ? { text: value } : value && typeof value === 'object' && !Array.isArray(value) ? value as AnyRecord : {};
  return { ...source, text: text(source.text), vertical: bool(source.vertical), size: num(source.size, 0.035), opacity: num(source.opacity, 0.7), anchor: text(source.anchor, 'bottomRight'), inset: num(source.inset, 0.025), color: Array.isArray(source.color) ? source.color : [255, 255, 255], shadow: bool(source.shadow, true), image: text(source.image), imageWidth: num(source.imageWidth, 0.2) };
}
function watermarkHex(value: unknown): string { const color = arr(value).slice(0, 3).map((v) => Math.max(0, Math.min(255, num(v, 255)))); return `#${color.map((v) => Math.round(v).toString(16).padStart(2, '0')).join('') || 'ffffff'}`; }
function watermarkColor(value: string): number[] { const hex = value.replace('#', ''); return hex.length === 6 ? [0, 2, 4].map((offset) => Number.parseInt(hex.slice(offset, offset + 2), 16)) : [255, 255, 255]; }
function exportPercent(value: string, fallback: number, min: number, max: number): number { const parsed = Number(value); return Number.isFinite(parsed) ? Math.max(min, Math.min(max, parsed)) / 100 : fallback; }

function ExportDialog({ d, desktop }: { d: DialogState; desktop: DesktopContextValue }) {
  const p = d.params || {}; const supplied = (p.opts as AnyRecord) || {}; const [opts, setOpts] = useState<AnyRecord>({ ...supplied }); const [dir, setDir] = useState(text(p.dir)); const [fullSize, setFullSize] = useState(bool(p.fullSize, false)); const [preset, setPreset] = useState(text(p.preset)); const [watermark, setWatermark] = useState<AnyRecord>(() => watermarkDefaults(supplied.watermark)); const [watermarkOn, setWatermarkOn] = useState(() => { const value = watermarkDefaults(supplied.watermark); return Boolean(text(value.text).trim() || text(value.image).trim()); }); const [watermarkStyle, setWatermarkStyle] = useState(() => text(watermarkDefaults(supplied.watermark).image) ? 'graphic' : 'text'); const [busy, setBusy] = useState(false); const [error, setError] = useState(''); const set = (key: string, value: unknown) => setOpts((old) => ({ ...old, [key]: value })); const setWm = (key: string, value: unknown) => setWatermark((old) => ({ ...old, [key]: value })); const format = text(opts.format, 'jpeg'); const rendered = format !== 'dng' && format !== 'original';
  const submit = async () => { if (!dir) { setError('Choose an export folder.'); return; } setBusy(true); setError(''); try { const payload: AnyRecord = { ...opts, dir, ids: p.ids }; delete payload.rename; delete payload.resizeMode; delete payload.resizeValue; delete payload.fullSize; if (opts.rename) payload.naming = opts.rename; if (preset.trim()) payload.preset = preset.trim(); ['resize', 'longEdge', 'shortEdge', 'width', 'height', 'megapixels', 'percent'].forEach((key) => { delete payload[key]; }); if (fullSize) payload.longEdge = 0; else { const mode = text(opts.resizeMode, 'longEdge'); const key = mode === 'dimensions' ? 'width' : mode; payload[key] = num(opts.resizeValue, 2048); if (mode === 'dimensions') payload.height = num(opts.resizeHeight, num(opts.resizeValue, 2048)); payload.dontEnlarge = bool(opts.dontEnlarge, true); } if (watermarkOn && rendered) payload.watermark = { ...watermark }; else delete payload.watermark; const result = await desktop.run('app.export', payload) as AnyRecord; desktop.setDialog({ kind: 'exportProgress', params: { dir, files: result?.files || [] } }); } catch (e) { setError(String(e)); setBusy(false); } };
  return (
    <Frame title="Export" wide busy={busy} onClose={() => desktop.setDialog(null)} actions={<><Button onClick={() => desktop.setDialog(null)}>Cancel</Button><Button primary disabled={busy} onClick={() => void submit()}>Export</Button></>}>
      <div className="lc-dialog-toolbar">
        <Button onClick={async () => { const paths = await choose(desktop.native, 'chooseFolder'); if (paths[0]) setDir(paths[0]); }}>Choose Export Folder…</Button>
        <Field label="Export folder" value={dir} onChange={setDir} placeholder="Choose or enter an export folder" />
      </div>
      {error && <Note tone="error">{error}</Note>}
      <div className="lc-export-columns">
        <div className="lc-form-stack">
          <Select label="Format" value={format} options={[["jpeg", 'JPEG'], ['png', 'PNG'], ['tiff', 'TIFF'], ['webp', 'WebP'], ['avif', 'AVIF'], ['original', 'Original + sidecar'], ['dng', 'DNG']]} onChange={(v) => set('format', v)} />
          {rendered && <>
            <Select label="Color space" value={text(opts.colorSpace, 'srgb')} options={[["srgb", 'sRGB'], ['displayP3', 'Display P3'], ['adobeRgb', 'Adobe RGB'], ['proPhoto', 'ProPhoto RGB'], ['rec2020', 'Rec. 2020']]} onChange={(v) => set('colorSpace', v)} />
            <Select label="Bit depth" value={String(opts.bitDepth ?? 8)} options={[["8", '8 bit'], ['16', '16 bit'], ['32', '32 bit']]} onChange={(v) => set('bitDepth', Number(v))} />
            <Field label="Quality" type="number" value={num(opts.quality, 90)} min={1} max={100} onChange={(v) => set('quality', Number(v))} />
            <Select label="Sharpen" value={text(opts.sharpen, 'none')} options={[["none", 'None'], ['screen', 'Screen'], ['matte', 'Matte'], ['glossy', 'Glossy']]} onChange={(v) => set('sharpen', v)} />
            {text(opts.sharpen, 'none') !== 'none' && <Select label="Sharpen amount" value={text(opts.sharpenAmount, 'standard')} options={[["low", 'Low'], ['standard', 'Standard'], ['high', 'High']]} onChange={(v) => set('sharpenAmount', v)} />}
          </>}
          <Field label="Naming template" value={text(opts.rename, text(opts.naming, '{name}'))} onChange={(v) => set('rename', v)} placeholder="{name}" />
          <Field label="Start number" type="number" value={num(opts.startNumber, 1)} min={1} onChange={(v) => set('startNumber', Number(v))} />
          <Select label="If file exists" value={text(opts.conflict, 'unique')} options={[["unique", 'Add a number'], ['overwrite', 'Overwrite'], ['skip', 'Skip']]} onChange={(v) => set('conflict', v)} />
          {format === 'tiff' && <Select label="TIFF compression" value={text(opts.tiffCompression, 'deflate')} options={[["none", 'None'], ['lzw', 'LZW'], ['deflate', 'ZIP']]} onChange={(v) => set('tiffCompression', v)} />}
          {format === 'dng' && <Select label="DNG compression" value={text(opts.dngCompression, 'lossless')} options={[["lossless", 'Lossless'], ['deflate', 'ZIP'], ['uncompressed', 'None']]} onChange={(v) => set('dngCompression', v)} />}
        </div>
        <div className="lc-form-stack">
          <Check checked={fullSize} onChange={setFullSize}>Full size</Check>
          {!fullSize && <>
            <Select label="Resize" value={text(opts.resizeMode, 'longEdge')} options={[["longEdge", 'Long edge'], ['shortEdge', 'Short edge'], ['width', 'Width'], ['height', 'Height'], ['dimensions', 'Width × height'], ['megapixels', 'Megapixels'], ['percent', 'Percentage']]} onChange={(v) => set('resizeMode', v)} />
            <Field label="Size" type="number" value={num(opts.resizeValue, 2048)} min={16} onChange={(v) => set('resizeValue', Number(v))} />
            {text(opts.resizeMode) === 'dimensions' && <Field label="Height" type="number" value={num(opts.resizeHeight, 2048)} min={16} onChange={(v) => set('resizeHeight', Number(v))} />}
            <Check checked={bool(opts.dontEnlarge, true)} onChange={(v) => set('dontEnlarge', v)}>Don't enlarge</Check>
          </>}
          <Field label="Resolution (ppi)" type="number" value={num(opts.ppi, 240)} min={1} onChange={(v) => set('ppi', Number(v))} />
          {rendered && <>
            <Check checked={watermarkOn} onChange={(enabled) => { setWatermarkOn(enabled); if (enabled && !text(watermark.text).trim() && !text(watermark.image).trim()) setWm('text', '© '); }}>Watermark</Check>
            {watermarkOn && <>
              <Select label="Watermark style" value={watermarkStyle} options={[["text", 'Text'], ['graphic', 'Graphic']]} onChange={(v) => { setWatermarkStyle(v); if (v === 'text') setWm('image', ''); }} />
              {watermarkStyle === 'graphic' ? <>
                <div className="lc-dialog-toolbar"><Button onClick={async () => { const paths = await choose(desktop.native, 'chooseFiles', { multiple: false }); if (paths[0]) setWm('image', paths[0]); }}>Choose Watermark Image…</Button></div>
                <Field label="Graphic path" value={text(watermark.image)} onChange={(v) => setWm('image', v)} placeholder="logo.png" />
                <Field label="Image width (%)" type="number" value={num(watermark.imageWidth, 0.2) * 100} min={2} max={100} step={1} onChange={(v) => setWm('imageWidth', exportPercent(v, 0.2, 2, 100))} />
              </> : <>
                <TextArea label="Text" value={text(watermark.text)} onChange={(v) => setWm('text', v)} placeholder="© Your Name" />
                <Check checked={bool(watermark.vertical)} onChange={(v) => setWm('vertical', v)}>Vertical text</Check>
                <Field label="Size (%)" type="number" value={num(watermark.size, 0.035) * 100} min={1} max={15} step={0.5} onChange={(v) => setWm('size', exportPercent(v, 0.035, 1, 15))} />
                <Field label="Colour" type="color" value={watermarkHex(watermark.color)} onChange={(v) => setWm('color', watermarkColor(v))} />
                <Check checked={bool(watermark.shadow, true)} onChange={(v) => setWm('shadow', v)}>Shadow</Check>
              </>}
              <Select label="Watermark position" value={text(watermark.anchor, 'bottomRight')} options={[["topLeft", 'Top left'], ['topRight', 'Top right'], ['center', 'Center'], ['bottomLeft', 'Bottom left'], ['bottomRight', 'Bottom right']]} onChange={(v) => setWm('anchor', v)} />
              <Field label="Opacity (%)" type="number" value={num(watermark.opacity, 0.7) * 100} min={5} max={100} step={1} onChange={(v) => setWm('opacity', exportPercent(v, 0.7, 5, 100))} />
            </>}
          </>}
          <Select label="Metadata" value={text(opts.metadata, 'all')} options={[["all", 'All metadata'], ['allExceptCamera', 'All except camera'], ['copyright', 'Copyright only'], ['none', 'None']]} onChange={(v) => set('metadata', v)} />
          <Check checked={bool(opts.removeLocation)} onChange={(v) => set('removeLocation', v)}>Remove location</Check>
          <Field label="Subfolder" value={text(opts.subfolder)} onChange={(v) => set('subfolder', v)} placeholder="Optional" />
          <Field label="Limit file size (KB)" type="number" value={num(opts.limitKb, 0)} min={0} onChange={(v) => set('limitKb', Number(v))} />
          <Field label="Export preset" value={preset} onChange={setPreset} placeholder="Optional preset name" />
        </div>
      </div>
    </Frame>
  );
}

function MergeDialog({ kind, d, desktop }: { kind: string; d: DialogState; desktop: DesktopContextValue }) {
  const [options, setOptions] = useState<AnyRecord>({ ...(d.params?.options as AnyRecord || {}) }); const [error, setError] = useState(''); const set = (key: string, value: unknown) => setOptions((old) => ({ ...old, [key]: value }));
  const merge = async () => { try { await desktop.run(kind, options); desktop.setDialog({ kind: `${kind}Progress`, params: {} }); } catch (e) { setError(String(e)); } };
  const pano = kind !== 'merge.hdr'; const hdr = kind !== 'merge.panorama';
  return <Frame title={kind === 'merge.panorama' ? 'Panorama Merge' : kind === 'merge.hdrPanorama' ? 'HDR Panorama Merge' : 'HDR Merge'} wide onClose={() => desktop.setDialog(null)} actions={<><Button onClick={() => desktop.setDialog(null)}>Cancel</Button><Button primary onClick={() => void merge()}>Merge</Button></>}><div className="lc-merge-layout"><div className="lc-merge-preview" aria-label="Merge preview"><span>Preview updates as options change</span></div><div className="lc-form-stack">{hdr && <><Check checked={bool(options.align, true)} onChange={(v) => set('align', v)}>Align photos</Check><Select label="Deghost" value={text(options.deghost, 'none')} options={[["none", 'None'], ['low', 'Low'], ['medium', 'Medium'], ['high', 'High']]} onChange={(v) => set('deghost', v)} /><Check checked={bool(options.autoSettings, true)} onChange={(v) => set('autoSettings', v)}>Auto settings</Check></>}{pano && <><Select label="Projection" value={text(options.projection, 'auto')} options={[["auto", 'Auto'], ['spherical', 'Spherical'], ['cylindrical', 'Cylindrical'], ['perspective', 'Perspective']]} onChange={(v) => set('projection', v)} /><Field label="Boundary warp" type="number" value={num(options.boundaryWarp, 0)} min={0} max={100} onChange={(v) => set('boundaryWarp', Number(v))} /><Check checked={bool(options.autoCrop, true)} onChange={(v) => set('autoCrop', v)}>Auto crop</Check><Check checked={bool(options.fillEdges)} onChange={(v) => set('fillEdges', v)}>Fill edges</Check></>}{error && <Note tone="error">{error}</Note>}<Check checked={bool(options.stack)} onChange={(v) => set('stack', v)}>Stack originals with result</Check></div></div></Frame>;
}

function AlbumDialog({ folder, desktop, d }: { folder: boolean; desktop: DesktopContextValue; d: DialogState }) { const [name, setName] = useState(text(d.params?.name)); const [error, setError] = useState(''); const save = async () => { if (!name.trim()) { setError(`Enter a ${folder ? 'folder' : 'album'} name.`); return; } try { await desktop.run(folder ? 'album.create' : 'album.create', { name: name.trim(), folder, addSelected: !folder }); desktop.setDialog(null); } catch (e) { setError(String(e)); } }; return <Frame title={folder ? 'New Folder' : 'New Album'} onClose={() => desktop.setDialog(null)} actions={<><Button onClick={() => desktop.setDialog(null)}>Cancel</Button><Button primary onClick={() => void save()}>Create</Button></>}><Field label="Name" value={name} onChange={setName} placeholder={folder ? 'Folder name' : 'Album name'} />{error && <Note tone="error">{error}</Note>}</Frame>; }
function SmartAlbumDialog({ desktop, d, kind = 'smartAlbum' }: { desktop: DesktopContextValue; d: DialogState; kind?: string }) { const editing = kind === 'smartRules'; const [name, setName] = useState(text(d.params?.name)); const [field, setField] = useState(text(d.params?.field, 'rating')); const [op, setOp] = useState(text(d.params?.operator, field === 'rating' ? 'gte' : 'is')); const [value, setValue] = useState(text(d.params?.value, '3')); const save = async () => { try { const numeric = ['rating', 'iso', 'aperture', 'focalLength', 'megapixels', 'album', 'sharpness'].includes(field); const boolean = field === 'edited'; const parsed = Number(value); const ruleValue = numeric ? (Number.isFinite(parsed) ? parsed : 0) : boolean ? ['true', 'yes', '1'].includes(value.trim().toLowerCase()) : value; const rules = { ruleSet: { match: 'all', rules: [{ field, op, value: ruleValue }] } }; if (editing) await desktop.run('album.setRules', { id: num(d.params?.id), rules, replace: true }); else await desktop.run('album.createSmart', { name: name.trim() || 'Smart Album', rules }); desktop.setDialog(null); } catch (e) { desktop.setNotice(String(e)); } }; const options: [string, string][] = field === 'rating' ? [["is", 'Is'], ['gte', 'At least'], ['lte', 'At most']] : field === 'camera' || field === 'keywords' ? [["contains", 'Contains'], ['is', 'Is'], ['isNot', 'Is not']] : [["is", 'Is'], ['isNot', 'Is not']]; return <Frame title={editing ? 'Edit Smart Album' : 'Smart Album'} onClose={() => desktop.setDialog(null)} actions={<><Button onClick={() => desktop.setDialog(null)}>Cancel</Button><Button primary onClick={() => void save()}>{editing ? 'Save Rules' : 'Create Smart Album'}</Button></>}><Field label="Name" value={name} onChange={setName} placeholder="Smart Album" /><Select label="Match field" value={field} options={[["rating", 'Rating'], ['flag', 'Pick flag'], ['label', 'Color label'], ['camera', 'Camera'], ['keywords', 'Keywords'], ['edited', 'Has edits']]} onChange={(v) => { setField(v); setOp(v === 'rating' ? 'gte' : v === 'camera' || v === 'keywords' ? 'contains' : 'is'); }} /><Select label="Condition" value={op} options={options} onChange={setOp} /><Field label="Value" value={value} onChange={setValue} /></Frame>; }

function SettingsDialog({ desktop, d }: { desktop: DesktopContextValue; d: DialogState }) {
  const record = (value: unknown): AnyRecord => value && typeof value === 'object' && !Array.isArray(value) ? value as AnyRecord : {};
  const persisted = record(desktop.snapshot?.preferences);
  const persistedUi = record(persisted.ui ?? persisted.layout ?? persisted);
  const persistedGeneral = record(persistedUi.general);
  const persistedPerformance = record(persistedUi.performance);
  const legacy = record(persisted.settings);
  const supplied = record(d.params?.values);
  const initial: AnyRecord = {
    ...legacy,
    ...persistedUi,
    ...persistedGeneral,
    ...persistedPerformance,
    ...supplied,
    theme: text(supplied.theme, text(persistedUi.theme, text(persistedGeneral.theme, text(legacy.theme, desktop.ui.theme)))),
    locale: text(supplied.locale, text(persistedUi.locale, text(persistedGeneral.locale, text(legacy.locale, desktop.ui.locale)))),
    confirmDelete: typeof supplied.confirmDelete === 'boolean' ? supplied.confirmDelete : typeof persistedUi.confirmDelete === 'boolean' ? persistedUi.confirmDelete : typeof persistedGeneral.confirmDelete === 'boolean' ? persistedGeneral.confirmDelete : typeof legacy.confirmDelete === 'boolean' ? legacy.confirmDelete : desktop.ui.confirmDelete,
    previewEdge: num(supplied.previewEdge, num(persistedUi.previewEdge, num(persistedPerformance.previewEdge, num(legacy.previewEdge, desktop.ui.previewEdge)))),
    cacheMb: num(supplied.cacheMb, num(persistedUi.cacheMb, num(persistedPerformance.cacheMb, num(persisted.cacheMb, 0)))),
    memoryMb: Number(supplied.memoryMb ?? supplied.memoryBudget ?? persistedUi.memoryMb ?? persistedUi.memoryBudget ?? persistedPerformance.memoryMb ?? persistedPerformance.memoryBudget ?? desktop.ui.memoryMb),
    gpu: typeof supplied.gpu === 'boolean' ? supplied.gpu : typeof persistedUi.gpu === 'boolean' ? persistedUi.gpu : typeof persistedPerformance.gpu === 'boolean' ? persistedPerformance.gpu : desktop.ui.gpu,
  };
  const requestedTab = text(d.params?.tab, 'general');
  const [tab, setTab] = useState(['general', 'import', 'performance', 'language'].includes(requestedTab) ? requestedTab : 'general');
  const [values, setValues] = useState<AnyRecord>(initial);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const tabs = ['general', 'import', 'performance', 'language'];
  const update = (key: string, value: unknown) => setValues((old) => ({ ...old, [key]: value }));

  useEffect(() => {
    let live = true;
    const load = async () => {
      try {
        const [library, gpu, memory] = await Promise.all([
          desktop.run('library.preferences', {}),
          desktop.run('app.gpu', {}),
          desktop.run('app.memoryBudget', {}),
        ]);
        if (!live) return;
        const libraryPrefs = record(library);
        const importPrefs = record(libraryPrefs.import);
        const gpuReport = record(gpu);
        const memoryReport = record(memory);
        setValues((old) => ({
          ...old,
          rawPreset: importPrefs.rawPreset == null ? old.rawPreset : text(importPrefs.rawPreset, 'default'),
          otherPreset: importPrefs.otherPreset == null ? old.otherPreset : text(importPrefs.otherPreset, 'default'),
          perCamera: typeof importPrefs.perCamera === 'boolean' ? importPrefs.perCamera : old.perCamera,
          cameras: importPrefs.cameras ?? old.cameras,
          copyright: text(importPrefs.copyright, text(old.copyright)),
          creator: text(importPrefs.creator, text(old.creator)),
          metadataPreset: importPrefs.metadataPreset == null ? '' : text(importPrefs.metadataPreset),
          cacheMb: num(libraryPrefs.cacheMb, num(old.cacheMb)),
          gpu: typeof gpuReport.enabled === 'boolean' ? gpuReport.enabled : old.gpu,
          memoryMb: num(old.memoryMb) === 0 ? 0 : num(memoryReport.budget, num(old.memoryMb * 1048576)) / 1048576,
        }));
      } catch (reason) {
        if (live) setError(errorText(reason));
      }
    };
    void load();
    return () => { live = false; };
  }, []);

  const save = async () => {
    setBusy(true);
    setError('');
    try {
      if (tab === 'import') {
        const allowed = ['rawPreset', 'otherPreset', 'perCamera', 'cameras', 'copyright', 'creator', 'metadataPreset'];
        const settings = Object.fromEntries(allowed.filter((key) => values[key] !== undefined).map((key) => [key, values[key]]));
        await desktop.run('library.preferences', { import: settings });
      }
      if (values.cacheMb != null) await desktop.run('library.preferences', { cacheMb: Math.max(0, Math.round(num(values.cacheMb))) });
      if (typeof values.gpu === 'boolean') await desktop.run('app.gpu', { enabled: values.gpu });
      const memoryMb = num(values.memoryMb, num(values.memoryBudget));
      if (memoryMb >= 64) await desktop.run('app.memoryBudget', { mb: Math.round(memoryMb) });
      const memoryChoice = memoryMb >= 0 ? Math.round(memoryMb) : desktop.ui.memoryMb;
      const edge = Math.max(256, Math.min(8192, Math.round(num(values.previewEdge, desktop.ui.previewEdge))));
      if (tab === 'performance' && edge !== desktop.ui.previewEdge) {
        await desktop.run('library.buildPreviews', { size: 'standard', edge });
      }
      const uiPatch: AnyRecord = {
        theme: values.theme,
        locale: values.locale,
        confirmDelete: values.confirmDelete,
        previewEdge: edge,
        gpu: values.gpu,
        memoryMb: memoryChoice,
      };
      await desktop.native('preferences.patch', { ui: Object.fromEntries(Object.entries(uiPatch).filter(([, value]) => value !== undefined)) });
      desktop.setUi(uiPatch as Partial<UiState>);
      desktop.setDialog(null);
    } catch (reason) {
      setError(errorText(reason));
    } finally {
      setBusy(false);
    }
  };

  return <Frame title="Settings" wide busy={busy} onClose={() => desktop.setDialog(null)} actions={<><Button disabled={busy} onClick={() => desktop.setDialog(null)}>Cancel</Button><Button primary disabled={busy} onClick={() => void save()}>Save</Button></>}><div className="lc-settings"><nav aria-label="Settings sections">{tabs.map((item) => <button key={item} className={tab === item ? 'lc-settings-tab active' : 'lc-settings-tab'} onClick={() => setTab(item)}>{item[0].toUpperCase() + item.slice(1)}</button>)}</nav><div className="lc-settings-panel">{tab === 'general' && <><Select label="Theme" value={text(values.theme, desktop.ui.theme)} options={[["system", 'System'], ['light', 'Light'], ['dark', 'Dark']]} onChange={(v) => update('theme', v)} /><Check checked={bool(values.confirmDelete, desktop.ui.confirmDelete)} onChange={(v) => update('confirmDelete', v)}>Confirm photo deletion</Check></>}{tab === 'import' && <><Field label="Raw develop preset ID" value={text(values.rawPreset, 'default')} onChange={(v) => update('rawPreset', v)} placeholder="default" /><Field label="Other photo preset ID" value={text(values.otherPreset, 'default')} onChange={(v) => update('otherPreset', v)} placeholder="default" /><Check checked={bool(values.perCamera)} onChange={(v) => update('perCamera', v)}>Use per-camera defaults</Check><Field label="Default copyright" value={text(values.copyright)} onChange={(v) => update('copyright', v)} /><Field label="Default creator" value={text(values.creator)} onChange={(v) => update('creator', v)} /><Field label="Metadata preset" value={text(values.metadataPreset)} onChange={(v) => update('metadataPreset', v)} /></>}{tab === 'performance' && <><Field label="Preview edge" type="number" value={num(values.previewEdge, desktop.ui.previewEdge)} min={256} max={8192} step={128} onChange={(v) => update('previewEdge', Number(v))} /><Field label="Cache size (MB)" type="number" value={num(values.cacheMb, 0)} min={0} onChange={(v) => update('cacheMb', Number(v))} /><Field label="Memory budget (MB; 0 = automatic)" type="number" value={num(values.memoryMb, desktop.ui.memoryMb)} min={0} onChange={(v) => update('memoryMb', Number(v))} /><Check checked={bool(values.gpu, desktop.ui.gpu)} onChange={(v) => update('gpu', v)}>Use GPU rendering when available</Check></>}{tab === 'language' && <Select label="Language" value={text(values.locale, desktop.ui.locale)} options={[["en", 'English'], ['zh-hans', '简体中文'], ['zh-hant', '繁體中文'], ['ja', '日本語']]} onChange={(v) => update('locale', v)} />}</div></div></Frame>;
}

const SETTINGS_GROUPS = ['basic', 'tone', 'curve', 'color', 'effects', 'detail', 'optics', 'geometry', 'masks', 'spotRemoval', 'redEye'];
function GroupsDialog({ paste, desktop, d }: { paste: boolean; desktop: DesktopContextValue; d: DialogState }) { const initial = arr(d.params?.groups).map(String); const [groups, setGroups] = useState<string[]>(initial.length ? initial : SETTINGS_GROUPS.slice(0, 8)); const toggle = (group: string) => setGroups((old) => old.includes(group) ? old.filter((item) => item !== group) : [...old, group]); const submit = async () => { try { await desktop.run(paste ? 'develop.paste' : 'develop.copy', { groups }); desktop.setDialog(null); } catch (e) { desktop.setNotice(String(e)); } }; return <Frame title={paste ? 'Paste Selected Edit Settings' : 'Choose Edit Settings to Copy'} onClose={() => desktop.setDialog(null)} actions={<><Button onClick={() => desktop.setDialog(null)}>Cancel</Button><Button primary onClick={() => void submit()}>{paste ? 'Paste Settings' : 'Copy Settings'}</Button></>}><div className="lc-group-list">{SETTINGS_GROUPS.map((group) => <Check key={group} checked={groups.includes(group)} onChange={() => toggle(group)}>{group === 'spotRemoval' ? 'Spot removal' : group === 'redEye' ? 'Red eye' : group[0].toUpperCase() + group.slice(1)}</Check>)}</div><div className="lc-dialog-toolbar"><Button onClick={() => setGroups([...SETTINGS_GROUPS])}>Select all</Button><Button onClick={() => setGroups([])}>Clear</Button></div></Frame>; }

function PresetFileDialog({ exportMode, curve, desktop, d }: { exportMode: boolean; curve: boolean; desktop: DesktopContextValue; d: DialogState }) { const [path, setPath] = useState(text(d.params?.path)); const [busy, setBusy] = useState(false); const chooseAction = exportMode ? (curve ? 'saveCurvePresetFile' : 'savePresetFile') : (curve ? 'pickCurvePresetFiles' : 'pickPresetFiles'); const command = curve ? (exportMode ? 'curve.exportPresets' : 'curve.importPresets') : (exportMode ? 'preset.export' : 'preset.import'); const submit = async () => { const paths = path ? [path] : await choose(desktop.native, chooseAction); if (!paths[0]) return; setPath(paths[0]); setBusy(true); try { await desktop.run(command, exportMode ? { path: paths[0] } : { paths }); desktop.setDialog(null); } catch (e) { desktop.setNotice(String(e)); } finally { setBusy(false); } }; return <Frame title={`${exportMode ? 'Export' : 'Import'} ${curve ? 'Curve Presets' : 'Presets'}`} busy={busy} onClose={() => desktop.setDialog(null)} actions={<><Button onClick={() => desktop.setDialog(null)}>Cancel</Button><Button primary disabled={busy} onClick={() => void submit()}>{exportMode ? 'Export' : 'Import'}</Button></>}><div className="lc-dialog-toolbar"><Button onClick={() => void submit()}>{exportMode ? 'Choose Save Location…' : 'Choose Preset Files…'}</Button></div><Field label={exportMode ? 'Save as' : 'Files or folder'} value={path} onChange={setPath} placeholder={curve ? '.lccurve' : '.lcpreset, .xmp or .lrtemplate'} /><Note>Imported presets keep source names & report settings that cannot be carried over.</Note></Frame>; }

function MetadataDialog({ d, desktop }: { d: DialogState; desktop: DesktopContextValue }) { const metadata = (d.params?.metadata as AnyRecord) || {}; return <Frame title="All Metadata" wide onClose={() => desktop.setDialog(null)} actions={<Button primary onClick={() => desktop.setDialog(null)}>Done</Button>}><dl className="lc-info-list">{Object.entries(metadata).map(([key, value]) => <React.Fragment key={key}><dt>{key}</dt><dd>{typeof value === 'string' ? value : JSON.stringify(value)}</dd></React.Fragment>)}</dl></Frame>; }

function InfoDialog({ kind, desktop, d }: { kind: string; desktop: DesktopContextValue; d: DialogState }) { const title = kind === 'about' ? 'About LightCraft' : kind === 'systeminfo' ? 'System Information' : kind === 'shortcuts' ? 'Keyboard Shortcuts' : "What's New"; const initialRows = arr(d.params?.rows); const [rows, setRows] = useState<any[]>(initialRows); const loaded = useRef(false); useEffect(() => { if (kind === 'systeminfo' && !rows.length && !loaded.current) { loaded.current = true; void desktop.run('library.info', {}).then((info) => { if (info && typeof info === 'object') setRows(Object.entries(info as AnyRecord)); }); } }, [kind, desktop, rows.length]); return <Frame title={title} onClose={() => desktop.setDialog(null)} actions={<Button primary onClick={() => desktop.setDialog(null)}>Done</Button>}>{kind === 'about' && <div className="lc-about"><div className="lc-app-mark">LC</div><h3>LightCraft</h3><p>Native photo library & non-destructive developer.</p><small>Open source · version {text(d.params?.version, '0.2.1')}</small></div>}{kind === 'systeminfo' && <dl className="lc-info-list">{rows.map((row, i) => { const pair = Array.isArray(row) ? row : [row.label, row.value]; return <React.Fragment key={i}><dt>{text(pair[0])}</dt><dd>{text(pair[1], JSON.stringify(pair[1]))}</dd></React.Fragment>; })}</dl>}{kind === 'shortcuts' && <div className="lc-shortcuts">{arr(d.params?.shortcuts).map((shortcut, i) => <div key={i}><span>{text(shortcut.label, text(shortcut.id))}</span><kbd>{text(shortcut.shortcut)}</kbd></div>)}</div>}{kind === 'whatsnew' && <div className="lc-release-notes">{text(d.params?.text, 'Recent improvements to import, develop, export & catalog interoperability.')}</div>}</Frame>; }

function FormDialog({ kind, d, desktop }: { kind: string; d: DialogState; desktop: DesktopContextValue }) { const formKind = kind.toLowerCase(); const p = d.params || {}; const [values, setValues] = useState<AnyRecord>({ ...p }); const update = (key: string, value: unknown) => setValues((old) => ({ ...old, [key]: value })); const config: AnyRecord = { rename: ['Rename Photos', 'photo.rename', [['template', 'Name template'], ['start', 'Start number']]], renamealbum: ['Rename Album', 'album.rename', [['id', 'Album ID'], ['name', 'Name']]], capturetime: ['Edit Capture Time', 'photo.setCaptureTime', [['time', 'Capture time'], ['shift', 'Shift seconds'], ['hours', 'Time-zone shift hours']]], autostack: ['Auto-Stack by Capture Time', 'stack.auto', [['gap', 'Maximum gap (seconds)']]], createpreset: ['Create Preset', 'preset.create', [['name', 'Preset name'], ['group', 'Group']]], copysettings: ['Copy Edit Settings', 'develop.copy', []], pastesettings: ['Paste Edit Settings', 'develop.paste', []], labelnames: ['Color Label Names', 'label.setNames', [['red', 'Red'], ['yellow', 'Yellow'], ['green', 'Green'], ['blue', 'Blue'], ['purple', 'Purple']]], savemetadatapreset: ['Save Metadata Preset', 'metadata.savePreset', [['name', 'Preset name']]], cull: ['Assisted Culling', 'photo.analyze', [['rejectBelow', 'Reject below focus score']]], textprompt: [text(p.title, 'Enter a value'), text(p.command), [[text(p.key, 'value'), text(p.hint, 'Value')]]], metadatapreset: ['Metadata Preset', 'metadata.applyPreset', [['name', 'Preset name']]], renamekeyword: ['Rename Keyword', 'keyword.rename', [['from', 'Current keyword'], ['to', 'New keyword']]], mergekeywords: ['Merge Keywords', 'keyword.merge', [['from', 'Keywords to merge (comma-separated)'], ['into', 'Merge into']]] }; const c = config[formKind]; const submit = async () => { try { let payload: AnyRecord = { ...values }; if (formKind === 'labelnames') payload = { names: Object.fromEntries(['red', 'yellow', 'green', 'blue', 'purple'].map((label) => [label, text(values[label]) || null])) }; else if (formKind === 'capturetime') { payload = {}; if (Array.isArray(values.ids)) payload.ids = values.ids; if (text(values.time)) payload.time = text(values.time); else if (values.shift !== undefined && text(values.shift) !== '') payload.shift = num(values.shift); else if (values.hours !== undefined && text(values.hours) !== '') payload.hours = num(values.hours); if (bool(values.each)) payload.each = true; } else if (formKind === 'rename') payload = { template: text(values.template), start: num(values.start, 1), ...(Array.isArray(values.ids) ? { ids: values.ids } : {}) }; else if (formKind === 'renamealbum') payload = { id: num(values.id), name: text(values.name) }; else if (formKind === 'autostack') payload = { gap: num(values.gap), ...(bool(values.preview) ? { preview: true } : {}) }; else if (formKind === 'createpreset') payload = { name: text(values.name), ...(text(values.group) ? { group: text(values.group) } : {}) }; else if (formKind === 'copysettings' || formKind === 'pastesettings') payload = { ...(Array.isArray(values.groups) ? { groups: values.groups } : {}) }; else if (formKind === 'metadatapreset') payload = { name: text(values.name) }; else if (formKind === 'savemetadatapreset') payload = { name: text(values.name), ...(values.fields ? { fields: values.fields } : {}) }; else if (formKind === 'renamekeyword') payload = { from: text(values.from), to: text(values.to) }; else if (formKind === 'mergekeywords') payload = { from: text(values.from).split(',').map((item) => item.trim()).filter(Boolean), into: text(values.into) }; else if (formKind === 'cull') payload = { ...(Array.isArray(values.ids) ? { ids: values.ids } : {}), rejectBelow: num(values.rejectBelow), pickBest: bool(values.pickBest, true) }; else if (formKind === 'textprompt') { const promptKey = text(p.key, 'value'); payload = { ...(typeof values.path === 'string' ? { path: values.path } : {}), [promptKey]: text(values[promptKey]) }; } await desktop.run(c[1], payload); desktop.setDialog(null); } catch (e) { desktop.setNotice(String(e)); } }; return <Frame title={c[0]} onClose={() => desktop.setDialog(null)} actions={<><Button onClick={() => desktop.setDialog(null)}>Cancel</Button><Button primary onClick={() => void submit()}>Apply</Button></>}><div className="lc-form-stack">{arr(c[2]).map((field: any[]) => <Field key={field[0]} label={field[1]} value={typeof values[field[0]] === 'number' ? values[field[0]] : text(values[field[0]])} type={typeof values[field[0]] === 'number' ? 'number' : 'text'} onChange={(v) => update(field[0], typeof values[field[0]] === 'number' ? Number(v) : v)} />)}{formKind === 'capturetime' && <Check checked={bool(values.each)} onChange={(v) => update('each', v)}>Set this time on every selected photo</Check>}{['copysettings', 'pastesettings'].includes(formKind) && <Note>Choose groups in Inspector before applying settings.</Note>}{formKind === 'cull' && <Check checked={bool(values.pickBest, true)} onChange={(v) => update('pickBest', v)}>Keep best frame in each burst</Check>}</div></Frame>; }

const EXPLICIT_FORM_KINDS = new Set(['rename', 'renamealbum', 'capturetime', 'autostack', 'createpreset', 'copysettings', 'pastesettings', 'labelnames', 'savemetadatapreset', 'cull', 'textprompt', 'metadatapreset', 'renamekeyword', 'mergekeywords']);
function UnsupportedDialog({ desktop }: { kind: string; desktop: DesktopContextValue }) { return <Frame title="Dialog unavailable" onClose={() => desktop.setDialog(null)} actions={<Button primary onClick={() => desktop.setDialog(null)}>Close</Button>}><Note tone="error">Sorry, this action isn’t available in this version.</Note></Frame>; }

function MissingDialog({ desktop, d }: { desktop: DesktopContextValue; d: DialogState }) { const [busy, setBusy] = useState(false); const [error, setError] = useState(''); const run = async () => { const paths = await choose(desktop.native, 'chooseFolder'); if (!paths[0]) return; setBusy(true); try { await desktop.run('library.findMissing', { folder: paths[0] }); desktop.setDialog(null); } catch (e) { setError(String(e)); } finally { setBusy(false); } }; return <Frame title="Find Missing Photos" busy={busy} onClose={() => desktop.setDialog(null)} actions={<><Button onClick={() => desktop.setDialog(null)}>Cancel</Button><Button primary disabled={busy} onClick={() => void run()}>Choose Search Folder…</Button></>}>{error && <Note tone="error">{error}</Note>}<Note>LightCraft checks names, sizes & content hashes, then relinks matches in one undoable step.</Note>{d.params?.count != null && <p>{num(d.params.count)} photos are missing.</p>}</Frame>; }
function LocalRootDialog({ desktop }: { desktop: DesktopContextValue }) { const [subfolders, setSubfolders] = useState(true); const chooseRoot = async () => { const paths = await choose(desktop.native, 'chooseFolder'); if (!paths[0]) return; try { await desktop.run('library.browse', { path: paths[0], subfolders }); desktop.setDialog(null); } catch (e) { desktop.setNotice(String(e)); } }; return <Frame title="Browse Local Folder" onClose={() => desktop.setDialog(null)} actions={<><Button onClick={() => desktop.setDialog(null)}>Cancel</Button><Button primary onClick={() => void chooseRoot()}>Choose Folder…</Button></>}><Check checked={subfolders} onChange={setSubfolders}>Include subfolders</Check><Note>Photos stay in place & remain outside catalog until you import them.</Note></Frame>; }
function LightroomResult({ d, desktop }: { d: DialogState; desktop: DesktopContextValue }) {
  const [result, setResult] = useState<AnyRecord>((d.params?.result as AnyRecord) || {});
  const [busy, setBusy] = useState(false); const [error, setError] = useState('');
  const catalog = text(d.params?.path); const missing = arr(result.missing); const failed = arr(result.failed); const warnings = arr(result.warnings); const unmapped = result.unmapped && typeof result.unmapped === 'object' ? Object.entries(result.unmapped as AnyRecord) : [];
  const run = async (operation: () => Promise<unknown>, close = false, updateResult = false) => { setBusy(true); setError(''); try { const value = await operation(); if (close) desktop.setDialog(null); else if (updateResult && value && typeof value === 'object') setResult(value as AnyRecord); } catch (reason) { setError(errorText(reason)); } finally { setBusy(false); } };
  const reimport = () => catalog ? run(async () => desktop.run('library.importLightroom', { path: catalog, updateExisting: bool(d.params?.updateExisting) }), false, true) : setError('Original catalog path is unavailable.');
  const undo = () => run(() => desktop.run('edit.undo'), true);
  const reveal = () => { const archive = text(result.archive); return archive ? run(() => desktop.native('reveal', { path: archive })) : setError('No recovery archive was created for this library.'); };
  const imported = Array.isArray(result.imported) ? result.imported.length : num(result.imported); const photos = num(result.photos, imported); const archive = text(result.archive);
  return <Frame title="Lightroom Catalog Imported" busy={busy} onClose={() => desktop.setDialog(null)} actions={<Button primary disabled={busy} onClick={() => desktop.setDialog(null)}>Done</Button>}>
    <div className="lc-summary-grid"><div><strong>{photos}</strong><span>Photos in catalog</span></div><div><strong>{imported}</strong><span>Files imported</span></div><div><strong>{num(result.collections)}</strong><span>Collections</span></div><div><strong>{missing.length}</strong><span>Missing originals</span></div><div><strong>{num(result.preservedExistingEdits)}</strong><span>Edits preserved</span></div></div>
    {error && <Note tone="error">{error}</Note>}
    <Note>Source catalog stayed read-only. LightCraft saved import identity & recovery data for this catalog.</Note>
    {archive ? <div className="lc-dialog-toolbar"><span className="lc-path">Recovery archive: {archive}</span><Button disabled={busy} onClick={() => void reveal()}>Show Recovery Archive</Button></div> : <Note>No recovery archive was created for this library.</Note>}
    {missing.length > 0 && <details className="lc-details" open><summary>Missing originals ({missing.length})</summary><ul>{missing.map((path, i) => <li key={i}>{displayValue(path)}</li>)}</ul></details>}
    {warnings.length > 0 && <details className="lc-details"><summary>Warnings ({warnings.length})</summary><ul>{warnings.map((warning, i) => <li key={i}>{displayValue(warning)}</li>)}</ul></details>}
    {failed.length > 0 && <details className="lc-details"><summary>Files not imported ({failed.length})</summary><ul>{failed.map((failure, i) => <li key={i}>{displayValue(failure)}</li>)}</ul></details>}
    {unmapped.length > 0 && <details className="lc-details"><summary>Settings not mapped ({unmapped.length})</summary><ul>{unmapped.map(([id, values]) => <li key={id}>Photo {id}: {displayValue(values)}</li>)}</ul></details>}
    <div className="lc-dialog-toolbar"><Button disabled={busy || !catalog} onClick={() => void reimport()}>Reimport Same Catalog</Button><Button disabled={busy} onClick={() => void undo()}>Undo Import</Button></div>
  </Frame>;
}

export default function DialogHost({ desktop: provided }: HostProps = {}) {
  useFocusHistory();
  const hookContext = useDesktop(); const context = provided || hookContext; const { dialog } = context; if (!dialog) return null; const kind = dialog.kind.toLowerCase();
  if (kind === 'unsavedquit') return <UnsavedQuitDialog d={dialog} desktop={context} />;
  if (kind === 'confirmdelete' || kind === 'confirm-delete') return <ConfirmDeleteDialog d={dialog} desktop={context} />;
  if (kind === 'import' || kind === 'importphotos' || kind === 'importfolder' || kind === 'importdevice') return <ImportDialog d={dialog} desktop={context} />;
  if (kind === 'lightroom' || kind === 'lightroomimport' || kind === 'importlightroom') return <LightroomDialog d={dialog} desktop={context} />;
  if (kind === 'export') return <ExportDialog d={dialog} desktop={context} />;
  if (kind === 'importprogress' || kind === 'exportprogress' || kind.endsWith('progress')) return <JobDialog kind={kind} d={dialog} desktop={context} />;
  if (kind === 'merge.hdr' || kind === 'merge.panorama' || kind === 'merge.hdrpanorama') return <MergeDialog kind={dialog.kind} d={dialog} desktop={context} />;
  if (kind === 'newalbum') return <AlbumDialog folder={false} d={dialog} desktop={context} />;
  if (kind === 'newfolder') return <AlbumDialog folder d={dialog} desktop={context} />;
  if (kind === 'smartalbum' || kind === 'newsmartalbum') return <SmartAlbumDialog d={dialog} desktop={context} />;
  if (kind === 'smartrules') return <SmartAlbumDialog kind="smartRules" d={dialog} desktop={context} />;
  if (kind === 'missing' || kind === 'findmissing' || kind === 'locatemissing') return <MissingDialog d={dialog} desktop={context} />;
  if (kind === 'localroot' || kind === 'localroots' || kind === 'browsefolder') return <LocalRootDialog desktop={context} />;
  if (kind === 'lightroomresult') return <LightroomResult d={dialog} desktop={context} />;
  if (kind === 'allmetadata') return <MetadataDialog d={dialog} desktop={context} />;
  if (kind === 'about' || kind === 'systeminfo' || kind === 'shortcuts' || kind === 'whatsnew') return <InfoDialog kind={kind} d={dialog} desktop={context} />;
  if (kind === 'settings') return <SettingsDialog d={dialog} desktop={context} />;
  if (kind === 'copysettings') return <GroupsDialog paste={false} d={dialog} desktop={context} />;
  if (kind === 'pastsettings' || kind === 'pastesettings') return <GroupsDialog paste d={dialog} desktop={context} />;
  if (kind === 'presetimport') return <PresetFileDialog exportMode={false} curve={false} d={dialog} desktop={context} />;
  if (kind === 'presetexport') return <PresetFileDialog exportMode curve={false} d={dialog} desktop={context} />;
  if (kind === 'curveimportpresets') return <PresetFileDialog exportMode={false} curve d={dialog} desktop={context} />;
  if (kind === 'curveexportpresets') return <PresetFileDialog exportMode curve d={dialog} desktop={context} />;
  if (EXPLICIT_FORM_KINDS.has(kind)) return <FormDialog kind={kind} d={dialog} desktop={context} />;
  return <UnsupportedDialog kind={dialog.kind} desktop={context} />;
}
