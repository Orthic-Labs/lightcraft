import { useCallback, useEffect, useRef, useState } from 'react';
import type { DesktopSnapshot, PhotoSummary, ViewSlice } from '../types';
import { useDesktop } from '../desktop';
import { getViewSlice } from '../api';

const MAX_SLICE = 512;
const MAX_CACHE = 12;

export interface PhotoSliceState {
  photos: PhotoSummary[];
  total: number;
  generation: number | null;
  offset: number;
  loading: boolean;
  error: string | null;
  refresh: () => void;
}

function boundedOffset(value: number): number {
  return Number.isSafeInteger(value) && value > 0 ? value : 0;
}

function boundedLimit(value: number): number {
  if (!Number.isFinite(value)) return MAX_SLICE;
  return Math.max(1, Math.min(MAX_SLICE, Math.floor(value)));
}

/** Fetch one generation-bound catalog page. Pages are cached locally by offset. */
export function usePhotoSlice(offset: number, limit = MAX_SLICE): PhotoSliceState {
  const { snapshot } = useDesktop();
  const safeOffset = boundedOffset(offset);
  const safeLimit = boundedLimit(limit);
  const generation = snapshot?.viewGeneration ?? null;
  const cache = useRef(new Map<string, ViewSlice>());
  const [state, setState] = useState<PhotoSliceState>(() => ({
    photos: [],
    total: snapshot?.total ?? 0,
    generation,
    offset: safeOffset,
    loading: Boolean(snapshot),
    error: null,
    refresh: () => undefined,
  }));
  const [nonce, setNonce] = useState(0);
  const refresh = useCallback(() => setNonce((value) => value + 1), []);

  useEffect(() => {
    cache.current.clear();
  }, [generation]);

  useEffect(() => {
    let live = true;
    if (!snapshot || generation === null) {
      setState((current) => ({ ...current, photos: [], total: 0, generation, offset: safeOffset, loading: false, error: null, refresh }));
      return () => {
        live = false;
      };
    }
    const key = `${generation}:${safeOffset}:${safeLimit}:${nonce}`;
    const cached = cache.current.get(key);
    if (cached) {
      setState({ photos: cached.photos, total: cached.total, generation: cached.generation, offset: safeOffset, loading: false, error: null, refresh });
      return () => {
        live = false;
      };
    }
    setState((current) => ({ ...current, total: snapshot.total, generation, offset: safeOffset, loading: true, error: null, refresh }));
    getViewSlice(generation, safeOffset, safeLimit)
      .then((slice) => {
        if (!live) return;
        const bounded: ViewSlice = { ...slice, photos: Array.isArray(slice.photos) ? slice.photos.slice(0, safeLimit) : [] };
        if (bounded.generation !== generation) cache.current.clear();
        const responseKey = `${bounded.generation}:${safeOffset}:${safeLimit}:${nonce}`;
        cache.current.set(responseKey, bounded);
        while (cache.current.size > MAX_CACHE) {
          const first = cache.current.keys().next().value;
          if (first === undefined) break;
          cache.current.delete(first);
        }
        setState({ photos: bounded.photos, total: bounded.total, generation: bounded.generation, offset: safeOffset, loading: false, error: null, refresh });
      })
      .catch((reason: unknown) => {
        if (!live) return;
        const message = reason instanceof Error ? reason.message : String(reason);
        setState((current) => ({ ...current, total: snapshot.total, generation, offset: safeOffset, loading: false, error: message, refresh }));
      });
    return () => {
      live = false;
    };
  }, [generation, nonce, refresh, safeLimit, safeOffset, snapshot]);

  return state;
}

export interface LibraryNavItem {
  id: string;
  label: string;
  icon: string;
  count?: number;
  command: string;
  params?: Record<string, unknown>;
  nativeAction?: string;
  children?: LibraryNavItem[];
}

export interface LibraryGroup {
  id: string;
  /** AppShell NavGroup-compatible heading. */
  title: string;
  items: LibraryNavItem[];
}

function count(snapshot: DesktopSnapshot, ...keys: string[]): number | undefined {
  for (const key of keys) {
    const value = snapshot.counts[key];
    if (typeof value === 'number' && Number.isFinite(value)) return value;
  }
  return undefined;
}

/** Convert authoritative snapshot data to AppShell's source group shape. */
export function libraryGroups(snapshot: DesktopSnapshot | null): LibraryGroup[] {
  if (!snapshot) return [];
  const albums = Array.isArray(snapshot.albums) ? snapshot.albums : [];
  const catalogCount = count(snapshot, 'catalog');
  const deletedCount = count(snapshot, 'deleted') ?? 0;
  const allCount = catalogCount === undefined ? count(snapshot, 'visible') : Math.max(0, catalogCount - deletedCount);
  const albumItems = albums
    .filter((album) => !album.folder)
    .map((album) => ({ id: `album:${album.id}`, label: album.name, icon: album.smart ? 'sparkles' : 'album', count: album.count, command: 'library.source', params: { kind: 'album', id: album.id } }));
  return [
    {
      id: 'my-photos',
      title: 'My Photos',
      items: [
        { id: 'all', label: 'All Photos', icon: 'photos', count: allCount, command: 'library.source', params: { kind: 'all' } },
        { id: 'recently-added', label: 'Recently Added', icon: 'clock', count: count(snapshot, 'recentlyAdded'), command: 'library.source', params: { kind: 'recentlyAdded' } },
        { id: 'picks', label: 'Picks', icon: 'flag-pick', count: count(snapshot, 'picks'), command: 'library.source', params: { kind: 'picks' } },
        { id: 'recently-deleted', label: 'Recently Deleted', icon: 'trash', count: count(snapshot, 'deleted', 'recentlyDeleted'), command: 'library.source', params: { kind: 'recentlyDeleted' } },
      ],
    },
    // Engine treats folders as containers, not library sources; expose writable & smart albums only.
    { id: 'albums', title: 'Albums', items: albumItems },
    {
      id: 'browse',
      title: 'Browse',
      items: [
        { id: 'by-date', label: 'By Date', icon: 'calendar', command: 'library.sort', params: { group: 'day' } },
        { id: 'local', label: 'Local', icon: 'folder', command: 'library.browse', params: { subfolders: true }, nativeAction: 'pickFolder' },
        { id: 'missing', label: 'Missing Photos', icon: 'warning', count: count(snapshot, 'missing'), command: 'library.source', params: { kind: 'missing' } },
      ],
    },
  ];
}
