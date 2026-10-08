import { useEffect, useRef, useState, type KeyboardEvent } from 'react';
import { useDesktop } from '../desktop';
import type { DesktopSnapshot, PhotoSummary } from '../types';
import { PhotoPreview } from '../preview/PhotoPreview';
import { usePhotoSlice } from './libraryData';
import { getViewSlice } from '../api';
import './library.css';

const PAGE = 128;
const ITEM_WIDTH = 124;
const ITEM_GAP = 8;
const ITEM_STRIDE = ITEM_WIDTH + ITEM_GAP;

type ActiveSearch = { key: string; cancelled: boolean };

function badge(photo: PhotoSummary): string {
  const values: string[] = [];
  if (photo.flag === 'pick') values.push('●');
  if (photo.flag === 'reject') values.push('×');
  if (photo.edited) values.push('✎');
  return values.join(' ');
}

function badgeLabel(photo: PhotoSummary): string {
  const values: string[] = [];
  if (photo.rating > 0) values.push(`${photo.rating} star${photo.rating === 1 ? '' : 's'}`);
  if (photo.flag === 'pick') values.push('picked');
  if (photo.flag === 'reject') values.push('rejected');
  if (photo.edited) values.push('edited');
  return values.join(', ');
}

export default function Filmstrip() {
  const { snapshot, run, ui } = useDesktop();
  const viewport = useRef<HTMLDivElement>(null);
  const [scrollLeft, setScrollLeft] = useState(0);
  const [viewportWidth, setViewportWidth] = useState(900);
  const [activeIndex, setActiveIndex] = useState<number | null>(null);
  const activeSearch = useRef<ActiveSearch | null>(null);
  const chunkOffset = Math.floor(Math.max(0, Math.floor(scrollLeft / ITEM_STRIDE) - 2) / PAGE) * PAGE;
  const slice = usePhotoSlice(chunkOffset, PAGE);
  const active = snapshot?.active ?? null;
  const generation = snapshot?.viewGeneration;
  const total = snapshot?.total ?? 0;
  const activeIndexHint = snapshot && typeof (snapshot as DesktopSnapshot & { activeIndex?: unknown }).activeIndex === 'number'
    ? (snapshot as DesktopSnapshot & { activeIndex?: number }).activeIndex ?? null
    : null;
  useEffect(() => {
    const node = viewport.current;
    if (!node) return;
    const resize = () => setViewportWidth(node.clientWidth);
    resize();
    const observer = new ResizeObserver(resize);
    observer.observe(node);
    return () => observer.disconnect();
  }, []);
  useEffect(() => {
    if (active === null || generation === undefined || total === 0) {
      setActiveIndex(null);
      if (activeSearch.current) activeSearch.current.cancelled = true;
      activeSearch.current = null;
      return;
    }
    const key = `${generation}:${active}`;
    const existing = activeSearch.current;
    if (existing?.key === key && !existing.cancelled) return;
    if (activeIndexHint !== null && Number.isSafeInteger(activeIndexHint) && activeIndexHint >= 0 && activeIndexHint < total) {
      setActiveIndex(activeIndexHint);
      activeSearch.current = { key, cancelled: false };
      return;
    }
    const search: ActiveSearch = { key, cancelled: false };
    activeSearch.current = search;
    let live = true;
    void (async () => {
      const pages = Math.ceil(total / PAGE);
      for (let page = 0; page < pages; page += 1) {
        if (!live || search.cancelled) break;
        const offset = page * PAGE;
        try {
          const result = await getViewSlice(generation, offset, PAGE);
          if (!live || search.cancelled) break;
          const index = result.photos.findIndex((photo) => photo.id === active);
          if (index >= 0) {
            setActiveIndex(offset + index);
            break;
          }
        } catch {
          break;
        }
      }
    })();
    return () => {
      live = false;
      search.cancelled = true;
      if (activeSearch.current === search) activeSearch.current = null;
    };
  }, [active, activeIndexHint, generation, total]);
  useEffect(() => {
    if (active === null) return;
    const localIndex = slice.photos.findIndex((photo) => photo.id === active);
    if (localIndex < 0) return;
    setActiveIndex(slice.offset + localIndex);
    const search = activeSearch.current;
    if (search?.key === `${generation}:${active}`) search.cancelled = true;
  }, [active, generation, slice.offset, slice.photos]);
  useEffect(() => {
    if (activeIndex === null || !viewport.current) return;
    const target = Math.max(0, activeIndex * ITEM_STRIDE - Math.max(0, (viewportWidth - ITEM_WIDTH) / 2));
    if (Math.abs(viewport.current.scrollLeft - target) > ITEM_STRIDE) {
      const reduced = typeof window.matchMedia === 'function' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      viewport.current.scrollTo({ left: target, behavior: reduced ? 'auto' : 'smooth' });
    }
  }, [activeIndex, viewportWidth]);
  if (!snapshot) return null;
  const choose = (photo: PhotoSummary) => { void run('library.select', { ids: [photo.id], active: photo.id, mode: 'replace' }); };
  const move = (delta: number, event: KeyboardEvent) => {
    event.preventDefault();
    event.stopPropagation();
    if (active === null) return;
    const local = slice.photos.findIndex((photo) => photo.id === active);
    if (local < 0) {
      void run(delta < 0 ? 'library.previous' : 'library.next', {});
      return;
    }
    const next = local + delta;
    const photo = slice.photos[next];
    if (photo) choose(photo);
    else void run(delta < 0 ? 'library.previous' : 'library.next', {});
  };
  const jumpToEdge = (edge: 'first' | 'last', event: KeyboardEvent) => {
    event.preventDefault();
    event.stopPropagation();
    if (!snapshot || generation === undefined || total === 0) return;
    const targetIndex = edge === 'first' ? 0 : total - 1;
    const offset = Math.floor(targetIndex / PAGE) * PAGE;
    void getViewSlice(generation, offset, PAGE).then((result) => {
      const photo = edge === 'first' ? result.photos[0] : result.photos[result.photos.length - 1];
      if (!photo) return;
      setActiveIndex(targetIndex);
      choose(photo);
    }).catch(() => undefined);
  };
  const totalWidth = Math.max(viewportWidth, snapshot.total * (ITEM_WIDTH + ITEM_GAP) + ITEM_GAP);
  return <div className="lc-filmstrip" aria-label="Filmstrip">
    <div className="lc-filmstrip-scroll" ref={viewport} onScroll={(event) => setScrollLeft(event.currentTarget.scrollLeft)}>
      <div className="lc-filmstrip-spacer" style={{ width: totalWidth }}>
        <div className="lc-filmstrip-window" style={{ left: chunkOffset * ITEM_STRIDE }} onKeyDown={(event) => {
          if (event.key === 'ArrowRight') move(1, event);
          if (event.key === 'ArrowLeft') move(-1, event);
          if (event.key === 'Home') jumpToEdge('first', event);
          if (event.key === 'End') jumpToEdge('last', event);
        }} role="listbox" aria-label="Photos" aria-busy={slice.loading} tabIndex={0}>
          {slice.photos.map((photo) => {
            const details = ui.filmBadges ? badgeLabel(photo) : '';
            const visibleBadge = ui.filmBadges ? badge(photo) : '';
            const label = [ui.filmNames ? photo.fileName : `Photo ${photo.id}`, details].filter(Boolean).join(', ');
            return <button className={`lc-filmstrip-item${active === photo.id ? ' is-active' : ''}`} type="button" role="option" aria-selected={active === photo.id} aria-label={label} key={photo.id} onClick={() => choose(photo)} title={photo.fileName}>
            <span className="lc-filmstrip-image"><PhotoPreview photoId={photo.id} slot={`filmstrip-${photo.id}`} viewGeneration={snapshot.viewGeneration} width={ITEM_WIDTH} height={72} quality="draft" className="lc-filmstrip-preview" /></span>
            {(ui.filmNames || ui.filmBadges) && <span className="lc-filmstrip-meta">
              {ui.filmNames && <span className="lc-filmstrip-name" title={photo.fileName} style={{ minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{photo.fileName}</span>}
              {ui.filmBadges && <span aria-label={details}>{visibleBadge}{photo.rating ? `${visibleBadge ? ' ' : ''}${photo.rating}★` : ''}</span>}
            </span>}
          </button>;
          })}
        </div>
      </div>
    </div>
    {slice.error && <span className="lc-filmstrip-error" role="alert">{slice.error}</span>}
  </div>;
}
