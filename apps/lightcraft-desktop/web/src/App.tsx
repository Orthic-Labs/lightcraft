import { useEffect, useMemo, useRef, useState, type CSSProperties } from 'react';
import { AppShell, ShellProvider, createThemeStore, useCommands, useShortcuts, type Command, type NavGroup } from '@rightkit/app-shell/react';
import { getPlatform } from '@rightkit/platform-ui';
import { createShell } from '@rightkit/shell';
import { DesktopProvider, useDesktop } from './desktop';
import { paletteCommands } from './commands';
import { Icon } from './icons';
import { LibraryWorkspace, libraryGroups, type LibraryGroup } from './library';
import { StageWorkspace } from './stage/StageWorkspace';
import { Inspector } from './inspector';
import { DialogHost } from './dialogs';
import './app.css';

const themeStore = createThemeStore({ storageKey: 'lightcraft.theme', storage: null });
const shell = createShell();

const sourceIcons: Record<string, 'library' | 'search' | 'import' | 'export' | 'chevron'> = {
  photos: 'library', clock: 'search', 'flag-pick': 'export', trash: 'chevron', album: 'library', folder: 'import', sparkles: 'export', calendar: 'search', warning: 'chevron',
};

function sourceNav(groups: LibraryGroup[]): NavGroup[] {
  return groups.map((group) => ({
    title: group.title,
    items: group.items.map((item) => ({ id: item.id, label: item.label, icon: <Icon name={sourceIcons[item.icon] ?? 'library'} />, badge: item.count === undefined ? undefined : String(item.count), keywords: [item.id, item.icon] })),
  }));
}

function sourceId(snapshot: { source: unknown } | null): string {
  const source = snapshot?.source;
  if (!source || typeof source !== 'object') return 'all';
  const value = source as { kind?: unknown; id?: unknown };
  if (value.kind === 'album' && value.id !== undefined) return `album:${String(value.id)}`;
  if (typeof value.kind === 'string') {
    if (value.kind === 'folder') return 'local';
    return value.kind === 'all' ? 'all' : value.kind.replace(/[A-Z]/g, (part) => `-${part.toLowerCase()}`);
  }
  return 'all';
}

function ShortcutBindings({ commands }: { commands: Command[] }) {
  useShortcuts(commands.filter((command) => command.chord).map((command) => ({
    id: command.id,
    chord: String(command.chord).replace(/Cmd/g, 'mod').replace(/Ctrl/g, 'mod'),
    label: command.label,
    group: command.group,
    run: () => { command.run(); },
    preventDefault: true,
  })));
  return null;
}

function Workspace() {
  const desktop = useDesktop();
  const { snapshot, ui, setUi, run, native, error, notice } = desktop;
  // Rust preferences own persisted choice; RightKit applies matching CSS tokens.
  useEffect(() => { themeStore.save(ui.theme); }, [ui.theme]);
  const groups = useMemo(() => libraryGroups(snapshot), [snapshot]);
  const navGroups = useMemo(() => sourceNav(groups), [groups]);
  const navItems = useMemo(() => groups.flatMap((group) => group.items), [groups]);
  const commands = useMemo(() => paletteCommands(snapshot, (id) => run(id)), [run, snapshot]);
  useCommands(commands);
  useEffect(() => {
    const timer = window.setTimeout(() => { void shell.ready(); }, 0);
    return () => window.clearTimeout(timer);
  }, []);
  return <>
    <ShortcutBindings commands={commands} />
    <AppShell
      groups={navGroups}
      activeId={sourceId(snapshot)}
      onNavigate={(id) => {
        const item = navItems.find((candidate) => candidate.id === id);
        if (!item) return;
        setUi({ view: 'photoGrid' });
        if (item.nativeAction) void native(item.nativeAction, item.params ?? {}).then((result) => {
          const path = result && typeof result === 'object' && 'path' in result && typeof (result as { path?: unknown }).path === 'string' ? (result as { path: string }).path : undefined;
          if (path) void run(item.command, { ...(item.params ?? {}), path });
        }).catch(() => undefined);
        else void run(item.command, item.params ?? {});
      }}
      title={ui.view === 'photoGrid' || ui.view === 'squareGrid' ? 'Library' : ui.view[0].toUpperCase() + ui.view.slice(1)}
      wordmark={<span className="lc-wordmark"><span>Light</span>Craft</span>}
      sidebarWidth={ui.sidebarCollapsed ? 48 : ui.sidebarWidth}
      sidebarCollapsed={ui.sidebarCollapsed}
      onSidebarCollapsedChange={(collapsed) => setUi({ sidebarCollapsed: collapsed })}
      bridge={shell.bridge()}
      platform={getPlatform()}
      theme={ui.theme}
      onThemeChange={(theme) => { setUi({ theme }); }}
      commands={commands}
      titlebarEnd={<div className="lc-title-status" aria-live="polite">{snapshot?.status.unsaved && <span className="lc-status-dot" title="Unsaved changes" />} {error && <span className="lc-title-error">{error}</span>}{notice && !error && <span>{notice}</span>}</div>}
    >
      <div className={`lc-content ${ui.sidebarCollapsed ? 'is-shell-collapsed' : ''}`}>
        {(ui.view === 'photoGrid' || ui.view === 'squareGrid') ? <LibraryWorkspace showSidebar={false} /> : <StageLayout />}
      </div>
    </AppShell>
    <DialogHost />
  </>;
}

function StageLayout() {
  const { ui, setUi } = useDesktop();
  const drag = useRef<{ x: number; width: number } | null>(null);
  const [dragging, setDragging] = useState(false);
  useEffect(() => {
    if (!dragging) return;
    const move = (event: PointerEvent) => { if (!drag.current) return; setUi({ inspectorWidth: drag.current.width - (event.clientX - drag.current.x) }); };
    const up = () => { drag.current = null; setDragging(false); };
    window.addEventListener('pointermove', move); window.addEventListener('pointerup', up);
    return () => { window.removeEventListener('pointermove', move); window.removeEventListener('pointerup', up); };
  }, [dragging, setUi]);
  return <div className="lc-stage-layout" style={{ '--lc-inspector-width': `${ui.inspectorCollapsed ? 0 : ui.inspectorWidth}px` } as CSSProperties}>
    <StageWorkspace />
    {!ui.inspectorCollapsed && <button className="lc-inspector-resizer" aria-label="Resize inspector" onPointerDown={(event) => { drag.current = { x: event.clientX, width: ui.inspectorWidth }; setDragging(true); event.currentTarget.setPointerCapture(event.pointerId); }} />}
    {!ui.inspectorCollapsed && <Inspector />}
    {ui.inspectorCollapsed && <button type="button" className="lc-inspector-collapsed" aria-label="Expand inspector" onClick={() => setUi({ inspectorCollapsed: false })}>◧</button>}
  </div>;
}

export default function App() {
  return <DesktopProvider><ShellProvider bridge={shell.bridge()} themeStore={themeStore} platform={getPlatform()}><Workspace /></ShellProvider></DesktopProvider>;
}
