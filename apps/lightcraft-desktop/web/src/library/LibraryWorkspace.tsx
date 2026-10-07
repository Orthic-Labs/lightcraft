import { useCallback, useEffect, useMemo, useRef, useState, type CSSProperties, type MouseEvent, type KeyboardEvent, type UIEvent } from 'react';
import type { DesktopSnapshot, PhotoSummary } from '../types';
import { useDesktop } from '../desktop';
import { PhotoPreview } from '../preview/PhotoPreview';
import { libraryGroups, usePhotoSlice, type LibraryNavItem } from './libraryData';
import './library.css';

const PAGE = 128;
const MIN_THUMB = 96;
const MAX_THUMB = 480;

type MenuState = { x: number; y: number; photo: PhotoSummary } | null;

function sourceKey(source: unknown): string {
  if (typeof source === 'string') return source;
  if (source && typeof source === 'object' && 'kind' in source) return String((source as { kind?: unknown }).kind ?? 'all');
  return 'all';
}

function photoAspect(photo: PhotoSummary): number {
  return photo.w > 0 && photo.h > 0 ? photo.w / photo.h : 1.5;
}

function PhotoBadges({ photo }: { photo: PhotoSummary }) {
  return (
    <div className="lc-photo-badges" aria-label="Photo metadata">
      {photo.flag === 'pick' && <span className="lc-badge lc-badge-pick" title="Picked">●</span>}
      {photo.flag === 'reject' && <span className="lc-badge lc-badge-reject" title="Rejected">×</span>}
      {photo.label && <span className={`lc-badge lc-badge-${photo.label}`} title={`${photo.label} label`} />}
      {photo.edited && <span className="lc-badge lc-badge-edited" title="Edited">✎</span>}
      {photo.previewOnly && <span className="lc-badge lc-badge-preview" title="Preview only">JPEG</span>}
    </div>
  );
}

function stars(rating: number): string {
  return rating > 0 ? `${'★'.repeat(Math.min(5, rating))}${'☆'.repeat(Math.max(0, 5 - rating))}` : '☆☆☆☆☆';
}

function navIsActive(snapshot: DesktopSnapshot | null, item: LibraryNavItem): boolean {
  if (!snapshot) return false;
  const source = sourceKey(snapshot.source);
  if (item.id === 'all') return source === 'all';
  if (item.id === 'recently-added') return source === 'recentlyAdded';
  if (item.id === 'picks') return source === 'picks';
  if (item.id === 'recently-deleted') return source === 'recentlyDeleted';
  if (item.id === 'local') return source === 'folder';
  if (item.params?.kind === 'album') {
    return source === 'album' && String((snapshot.source as { id?: unknown } | null)?.id ?? '') === String(item.params.id);
  }
  return false;
}

function NavIcon({ icon }: { icon: string }) {
  const glyph: Record<string, string> = { photos: '▦', clock: '◷', 'flag-pick': '⚑', trash: '⌫', album: '▱', folder: '□', sparkles: '✦', calendar: '▣', warning: '!' };
  return <span className={`lc-nav-icon lc-nav-icon-${icon}`} aria-hidden="true">{glyph[icon] ?? '·'}</span>;
}

function SourceSidebar({ snapshot, collapsed, onNavigate }: { snapshot: DesktopSnapshot | null; collapsed: boolean; onNavigate: (item: LibraryNavItem) => void }) {
  const groups = useMemo(() => libraryGroups(snapshot), [snapshot]);
  return (
    <aside className={`lc-library-sidebar${collapsed ? ' is-collapsed' : ''}`} aria-label="Library sources">
      {groups.map((group) => (
        <section className="lc-source-group" key={group.id}>
          {!collapsed && <h2>{group.title}</h2>}
          {group.items.map((item) => (
            <button
              className={`lc-source-item${navIsActive(snapshot, item) ? ' is-active' : ''}`}
              type="button"
              key={item.id}
              onClick={() => onNavigate(item)}
              title={collapsed ? item.label : undefined}
              aria-label={item.label}
              aria-current={navIsActive(snapshot, item) ? 'page' : undefined}
            >
              <NavIcon icon={item.icon} />
              {!collapsed && <span className="lc-source-label">{item.label}</span>}
              {!collapsed && item.count !== undefined && <span className="lc-source-count">{item.count}</span>}
            </button>
          ))}
        </section>
      ))}
      {!collapsed && <div className="lc-sidebar-status" aria-live="polite">
        {snapshot?.status.unsaved && <span>Unsaved changes</span>}
        {snapshot?.status.importing && <span>Importing…</span>}
        {snapshot?.status.exporting && <span>Exporting…</span>}
        {!snapshot?.status.unsaved && !snapshot?.status.importing && !snapshot?.status.exporting && <span>Library ready</span>}
      </div>}
    </aside>
  );
}

function FilterBar({ snapshot, filterText, setFilterText }: { snapshot: DesktopSnapshot | null; filterText: string; setFilterText: (value: string) => void }) {
  const { run, setDialog } = useDesktop();
  const filter = snapshot?.filter as Record<string, unknown> | null;
  const [expanded, setExpanded] = useState(false);
  const [savedName, setSavedName] = useState('');
  const [presets, setPresets] = useState<Array<{ name: string }>>([]);
  const setFilter = (patch: Record<string, unknown>) => { void run('library.filter', patch); };
  const clear = () => { void run('library.clearFilter'); setFilterText(''); };
  const save = () => {
    const name = savedName.trim();
    if (name) { void run('filter.savePreset', { name }); setSavedName(''); }
  };
  useEffect(() => {
    if (!expanded) return;
    let live = true;
    void run('filter.presets', {}).then((value: unknown) => {
      if (!live || !Array.isArray(value)) return;
      setPresets(value.filter((item): item is { name: string } => Boolean(item && typeof item === 'object' && typeof (item as { name?: unknown }).name === 'string')));
    }).catch(() => undefined);
    return () => { live = false; };
  }, [expanded, run]);
  return (
    <div className={`lc-filter-bar${expanded ? ' is-expanded' : ''}`}>
      <label className="lc-filter-search">
        <span aria-hidden="true">⌕</span>
        <input value={filterText} onChange={(event) => setFilterText(event.target.value)} placeholder="Search photos" aria-label="Search photos" />
        {filterText && <button type="button" aria-label="Clear search" onClick={() => setFilterText('')}>×</button>}
      </label>
      <button className="lc-filter-pill" type="button" aria-pressed={Boolean(filter?.rating)} onClick={() => setFilter({ rating: filter?.rating ? 0 : 5, ratingOp: 'atLeast' })}>★ 5+</button>
      <button className="lc-filter-pill" type="button" aria-pressed={filter?.flag === 'pick'} onClick={() => setFilter({ flag: filter?.flag === 'pick' ? null : 'pick' })}>⚑ Picks</button>
      <button className="lc-filter-pill" type="button" aria-pressed={filter?.edited === true} onClick={() => setFilter({ edited: filter?.edited === true ? null : true })}>✎ Edited</button>
      <button className="lc-filter-more" type="button" aria-expanded={expanded} onClick={() => setExpanded((value) => !value)}>Filters <span aria-hidden="true">⌄</span></button>
      {(filter && Object.keys(filter).some((key) => filter[key] !== null && filter[key] !== undefined && filter[key] !== '' && filter[key] !== false && filter[key] !== 0)) && <button className="lc-filter-clear" type="button" onClick={clear}>Clear all</button>}
      {expanded && <div className="lc-filter-panel" role="region" aria-label="Full filters">
        <label>Rating <select value={String(filter?.rating ?? 0)} onChange={(event) => setFilter({ rating: Number(event.target.value) || 0 })}><option value="0">Any</option>{[1, 2, 3, 4, 5].map((rating) => <option key={rating} value={rating}>{rating}+ stars</option>)}</select></label>
        <label>Flag <select value={String(filter?.flag ?? '')} onChange={(event) => setFilter({ flag: event.target.value || null })}><option value="">Any</option><option value="pick">Picked</option><option value="reject">Rejected</option><option value="none">Unflagged</option></select></label>
        <label>Label <select value={String(filter?.label ?? '')} onChange={(event) => setFilter({ label: event.target.value || null })}><option value="">Any</option>{['red', 'yellow', 'green', 'blue', 'purple'].map((label) => <option key={label} value={label}>{label}</option>)}</select></label>
        <label>Kind <select value={String(filter?.kind ?? '')} onChange={(event) => setFilter({ kind: event.target.value || null })}><option value="">Any</option><option value="image">Image</option><option value="raw">Raw</option><option value="video">Video</option></select></label>
        <label>Date <input value={String(filter?.date ?? '')} onChange={(event) => setFilter({ date: event.target.value || null })} placeholder="YYYY-MM-DD" /></label>
        <label>Keyword <input value={String(filter?.keyword ?? '')} onChange={(event) => setFilter({ keyword: event.target.value || null })} /></label>
        <label>Camera <input value={String(filter?.camera ?? '')} onChange={(event) => setFilter({ camera: event.target.value || null })} /></label>
        <label className="lc-filter-check"><input type="checkbox" checked={filter?.edited === true} onChange={(event) => setFilter({ edited: event.target.checked ? true : null })} /> Edited only</label>
        {presets.length > 0 && <label>Saved <select defaultValue="" onChange={(event) => { if (event.target.value) void run('filter.applyPreset', { name: event.target.value }); }}><option value="">Choose…</option>{presets.map((preset) => <option key={preset.name} value={preset.name}>{preset.name}</option>)}</select></label>}
        <div className="lc-filter-save"><input value={savedName} onChange={(event) => setSavedName(event.target.value)} placeholder="Saved filter name" aria-label="Saved filter name" /><button type="button" onClick={save}>Save filter</button><button type="button" onClick={() => setDialog({ kind: 'smartAlbum', params: { fromFilter: true } })}>Smart album…</button></div>
      </div>}
    </div>
  );
}

function GridMenu({ menu, close, albums }: { menu: MenuState; close: () => void; albums: DesktopSnapshot['albums'] }) {
  const { run } = useDesktop();
  if (!menu) return null;
  const target = { ids: [menu.photo.id] };
  const execute = (id: string, params: Record<string, unknown> = {}) => { close(); void run(id, { ...params, ...target }); };
  return <div className="lc-grid-menu" role="menu" style={{ left: menu.x, top: menu.y }} onMouseLeave={close}>
    <button type="button" role="menuitem" onClick={() => execute('photo.pick')}>Pick <kbd>P</kbd></button>
    <button type="button" role="menuitem" onClick={() => execute('photo.reject')}>Reject <kbd>X</kbd></button>
    <button type="button" role="menuitem" onClick={() => execute('photo.unflag')}>Unflag <kbd>U</kbd></button>
    <div className="lc-menu-label">Rating</div>
    <div className="lc-menu-rating">{[1, 2, 3, 4, 5].map((rating) => <button key={rating} type="button" role="menuitem" aria-label={`${rating} stars`} onClick={() => execute('photo.rate', { rating })}>{rating <= menu.photo.rating ? '★' : '☆'}</button>)}</div>
    <div className="lc-menu-label">Label</div>
    <div className="lc-menu-labels">{['red', 'yellow', 'green', 'blue', 'purple', 'none'].map((label) => <button type="button" role="menuitem" className={`lc-label-dot ${label}`} key={label} aria-label={`${label} label`} onClick={() => execute('photo.label', { label })} />)}</div>
    <div className="lc-menu-label">Add to album</div>
    {albums.filter((album) => !album.folder && !album.smart).map((album) => <button type="button" role="menuitem" key={album.id} onClick={() => execute('album.addPhotos', { id: album.id })}>{album.name}</button>)}
    {albums.every((album) => album.folder || album.smart) && <span className="lc-menu-empty">No writable albums</span>}
    <button type="button" role="menuitem" onClick={() => execute('app.showInFinder')}>Reveal original</button>
    <button type="button" role="menuitem" onClick={() => execute(menu.photo.deleted ? 'photo.restore' : 'photo.delete')}>{menu.photo.deleted ? 'Restore' : 'Move to Recently Deleted'}</button>
  </div>;
}

function VirtualPhotoGrid({ snapshot, mode, thumbSize }: { snapshot: DesktopSnapshot; mode: 'photoGrid' | 'squareGrid'; thumbSize: number }) {
  const { run, ui, setUi } = useDesktop();
  const [scrollTop, setScrollTop] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(600);
  const [viewportWidth, setViewportWidth] = useState(900);
  const [menu, setMenu] = useState<MenuState>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const gap = 8;
  const columns = Math.max(1, Math.floor((viewportWidth + gap) / (thumbSize + gap)));
  const rowHeight = thumbSize + 76;
  const rows = Math.ceil(snapshot.total / columns);
  const chunkRow = Math.max(0, Math.floor(scrollTop / rowHeight) - 2);
  const chunkOffset = Math.floor((chunkRow * columns) / PAGE) * PAGE;
  const chunkStartRow = Math.floor(chunkOffset / columns);
  const slice = usePhotoSlice(chunkOffset, PAGE);
  const photos = slice.photos;
  const selected = new Set(snapshot.selection);
  const active = snapshot.active;
  const onScroll = (event: UIEvent<HTMLDivElement>) => setScrollTop(event.currentTarget.scrollTop);
  useEffect(() => {
    const element = scrollRef.current;
    if (!element) return;
    const resize = () => { setViewportWidth(element.clientWidth); setViewportHeight(element.clientHeight); };
    resize();
    const observer = new ResizeObserver(resize);
    observer.observe(element);
    return () => observer.disconnect();
  }, []);
  const select = (photo: PhotoSummary, event?: MouseEvent | KeyboardEvent) => {
    const modifier = Boolean(event && ('metaKey' in event ? event.metaKey : false) || event && ('ctrlKey' in event ? event.ctrlKey : false));
    const modeName = event?.shiftKey ? 'range' : modifier ? 'toggle' : 'replace';
    void run('library.select', { ids: [photo.id], active: photo.id, mode: modeName });
  };
  const moveSelection = (delta: number, event: KeyboardEvent) => {
    const visibleIndex = active === null ? -1 : photos.findIndex((photo) => photo.id === active);
    const index = Math.max(0, (visibleIndex >= 0 ? chunkOffset + visibleIndex : 0) + delta);
    const targetIndex = Math.max(0, Math.min(snapshot.total - 1, index));
    const targetOffset = Math.floor(targetIndex / PAGE) * PAGE;
    if (targetOffset !== chunkOffset) {
      scrollRef.current?.scrollTo({ top: Math.floor(targetIndex / columns) * rowHeight });
      return;
    }
    const photo = photos[targetIndex - chunkOffset];
    if (photo) select(photo, event);
  };
  const totalHeight = Math.max(viewportHeight, rows * rowHeight + 24);
  return (
    <div className={`lc-grid-scroll ${mode === 'squareGrid' ? 'is-square' : 'is-aspect'}`} ref={scrollRef} onScroll={onScroll} onClick={() => setMenu(null)}>
      <div className="lc-grid-spacer" style={{ height: totalHeight }}>
        <div className="lc-grid-window" style={{ top: chunkStartRow * rowHeight, gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`, gap }} onKeyDown={(event) => {
          if (event.key === 'ArrowRight') { event.preventDefault(); moveSelection(1, event); }
          if (event.key === 'ArrowLeft') { event.preventDefault(); moveSelection(-1, event); }
          if (event.key === 'ArrowDown') { event.preventDefault(); moveSelection(columns, event); }
          if (event.key === 'ArrowUp') { event.preventDefault(); moveSelection(-columns, event); }
          if (event.key === 'Home') { event.preventDefault(); moveSelection(-snapshot.total, event); }
          if (event.key === 'End') { event.preventDefault(); moveSelection(snapshot.total, event); }
        }} role="grid" aria-rowcount={rows} aria-busy={slice.loading} tabIndex={0}>
          {photos.map((photo) => {
            const isSelected = selected.has(photo.id);
            const style = { '--lc-aspect': mode === 'squareGrid' ? '1' : String(photoAspect(photo)) } as CSSProperties;
            return <button className={`lc-photo-cell${isSelected ? ' is-selected' : ''}${active === photo.id ? ' is-active' : ''}`} style={style} type="button" role="gridcell" aria-selected={isSelected} key={photo.id} onClick={(event) => { event.stopPropagation(); select(photo, event); }} onDoubleClick={() => setUi({ view: 'detail', panel: 'info' })} onContextMenu={(event) => { event.preventDefault(); event.stopPropagation(); setMenu({ x: event.clientX, y: event.clientY, photo }); }}>
              <span className="lc-photo-frame"><PhotoPreview photoId={photo.id} slot={`grid-${photo.id}`} viewGeneration={snapshot.viewGeneration} width={Math.max(96, thumbSize)} height={Math.max(96, Math.round(thumbSize / Math.max(0.5, photoAspect(photo))))} quality="draft" className="lc-photo-preview" /></span>
              <PhotoBadges photo={photo} />
              <span className="lc-photo-caption">{ui.gridInfo && <span>{photo.fileName}</span>}<span>{stars(photo.rating)}</span></span>
            </button>;
          })}
          {slice.loading && photos.length === 0 && <div className="lc-grid-loading" role="status">Loading photos…</div>}
          {slice.error && <div className="lc-grid-error" role="alert">Unable to load photos: {slice.error}</div>}
        </div>
      </div>
      <GridMenu menu={menu} albums={snapshot.albums} close={() => setMenu(null)} />
    </div>
  );
}

function Header({ snapshot, mode, setMode }: { snapshot: DesktopSnapshot; mode: 'photoGrid' | 'squareGrid'; setMode: (mode: 'photoGrid' | 'squareGrid') => void }) {
  const { run, setDialog, setUi } = useDesktop();
  const sourceLabel = typeof snapshot.source === 'object' && snapshot.source && 'label' in snapshot.source ? String((snapshot.source as { label?: unknown }).label) : sourceKey(snapshot.source) === 'all' ? 'All Photos' : sourceKey(snapshot.source);
  const [sortOpen, setSortOpen] = useState(false);
  return <header className="lc-library-header">
    <div className="lc-library-title"><h1>{sourceLabel}</h1><span>{snapshot.selection.length > 1 ? `${snapshot.selection.length} selected · ` : ''}{snapshot.total} photos</span></div>
    <div className="lc-library-actions">
      <button type="button" className="lc-header-action" onClick={() => void run('file.addPhotos', {})}>Import</button>
      <button type="button" className="lc-header-action" onClick={() => setDialog({ kind: 'export' })}>Export</button>
      <button type="button" className={`lc-view-toggle ${mode === 'photoGrid' ? 'is-active' : ''}`} aria-pressed={mode === 'photoGrid'} onClick={() => { setMode('photoGrid'); setUi({ view: 'photoGrid' }); }}>▦ Photo Grid</button>
      <button type="button" className={`lc-view-toggle ${mode === 'squareGrid' ? 'is-active' : ''}`} aria-pressed={mode === 'squareGrid'} onClick={() => { setMode('squareGrid'); setUi({ view: 'squareGrid' }); }}>▦ Square</button>
      <div className="lc-sort-wrap"><button type="button" className="lc-header-action" aria-expanded={sortOpen} onClick={() => setSortOpen((value) => !value)}>Sort ▾</button>{sortOpen && <div className="lc-sort-menu" role="menu">{[['captureDate', 'Capture date'], ['importDate', 'Import date'], ['editDate', 'Modified'], ['fileName', 'Filename'], ['rating', 'Rating']].map(([key, label]) => <button type="button" role="menuitem" key={key} onClick={() => { setSortOpen(false); void run('library.sort', { key }); }}>{label}</button>)}<hr /><button type="button" role="menuitem" onClick={() => { setSortOpen(false); void run('library.sort', { ascending: true }); }}>Ascending</button><button type="button" role="menuitem" onClick={() => { setSortOpen(false); void run('library.sort', { ascending: false }); }}>Descending</button></div>}</div>
    </div>
  </header>;
}

function Footer({ snapshot, mode, setMode, thumbSize }: { snapshot: DesktopSnapshot; mode: 'photoGrid' | 'squareGrid'; setMode: (mode: 'photoGrid' | 'squareGrid') => void; thumbSize: number }) {
  const { ui, setDialog, setUi } = useDesktop();
  return <footer className="lc-library-footer">
    <div className="lc-footer-views"><button type="button" className={mode === 'photoGrid' ? 'is-active' : ''} onClick={() => { setMode('photoGrid'); setUi({ view: 'photoGrid' }); }}>▦</button><button type="button" className={mode === 'squareGrid' ? 'is-active' : ''} onClick={() => { setMode('squareGrid'); setUi({ view: 'squareGrid' }); }}>□</button><button type="button" onClick={() => setUi({ view: 'detail', panel: 'info' })}>▣</button></div>
    <div className="lc-footer-actions"><label>Thumbnail size <input type="range" min={MIN_THUMB} max={MAX_THUMB} step={4} value={thumbSize} onChange={(event) => setUi({ thumbSize: Number(event.target.value) })} /></label><button type="button" onClick={() => setUi({ gridInfo: !ui.gridInfo })}>Grid info: {ui.gridInfo ? 'on' : 'off'}</button><button type="button" onClick={() => setDialog({ kind: 'copySettings' })}>Copy settings</button><button type="button" onClick={() => setDialog({ kind: 'pasteSettings' })}>Paste settings</button></div>
  </footer>;
}

export default function LibraryWorkspace({ showSidebar = true }: { showSidebar?: boolean } = {}) {
  const { snapshot, ui, setUi, run, native } = useDesktop();
  const [filterText, setFilterText] = useState(ui.filterText ?? '');
  const mode: 'photoGrid' | 'squareGrid' = ui.view === 'squareGrid' ? 'squareGrid' : 'photoGrid';
  const thumbSize = Math.max(MIN_THUMB, Math.min(MAX_THUMB, ui.thumbSize || 180));
  const [sidebarCollapsed, setSidebarCollapsed] = useState(ui.sidebarCollapsed);
  useEffect(() => { const handle = window.setTimeout(() => { setUi({ filterText }); void run('library.filter', { text: filterText || null }); }, 220); return () => window.clearTimeout(handle); }, [filterText, run, setUi]);
  const navigate = useCallback(async (item: LibraryNavItem) => {
    if (item.id === 'local') {
      const result = await native('pickFolder', {});
      if (result && typeof result === 'object' && 'path' in result && typeof (result as { path?: unknown }).path === 'string') {
        const path = (result as { path: string }).path;
        await run('library.browse', { path, subfolders: true });
      }
      return;
    }
    await run(item.command, item.params ?? {});
  }, [native, run]);
  if (!snapshot) return <main className="lc-library-empty" role="status">Reconnect to load library.</main>;
  return <div className={`lc-library-workspace${showSidebar ? '' : ' lc-library-workspace-shell-nav'}`} style={{ '--lc-sidebar-width': showSidebar ? (sidebarCollapsed ? '48px' : `${ui.sidebarWidth || 268}px`) : '0px' } as CSSProperties}>
    {showSidebar && <SourceSidebar snapshot={snapshot} collapsed={sidebarCollapsed} onNavigate={navigate} />}
    <section className="lc-library-main" aria-label="Library">
      {showSidebar && <div className="lc-library-sidebar-toggle"><button type="button" aria-label={sidebarCollapsed ? 'Expand library sidebar' : 'Collapse library sidebar'} aria-pressed={sidebarCollapsed} onClick={() => { const next = !sidebarCollapsed; setSidebarCollapsed(next); setUi({ sidebarCollapsed: next }); }}>☰</button></div>}
      <Header snapshot={snapshot} mode={mode} setMode={(next) => setUi({ view: next })} />
      <FilterBar snapshot={snapshot} filterText={filterText} setFilterText={setFilterText} />
      <VirtualPhotoGrid snapshot={snapshot} mode={mode} thumbSize={thumbSize} />
      <Footer snapshot={snapshot} mode={mode} setMode={(next) => setUi({ view: next })} thumbSize={thumbSize} />
    </section>
    {snapshot.status.error && <div className="lc-library-error" role="alert">{snapshot.status.error}</div>}
  </div>;
}
