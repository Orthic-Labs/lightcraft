import { useEffect, useRef, useState, type KeyboardEvent } from 'react';
import { useDesktop } from '../desktop';
import type { PhotoSummary } from '../types';
import { PhotoPreview } from '../preview/PhotoPreview';
import { usePhotoSlice } from './libraryData';
import './library.css';

const PAGE = 128;
const ITEM_WIDTH = 124;
const ITEM_GAP = 8;

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
  const chunkOffset = Math.floor(Math.max(0, Math.floor(scrollLeft / (ITEM_WIDTH + ITEM_GAP)) - 2) / PAGE) * PAGE;
  const slice = usePhotoSlice(chunkOffset, PAGE);
  const active = snapshot?.active ?? null;
  useEffect(() => {
    const node = viewport.current;
    if (!node) return;
    const resize = () => setViewportWidth(node.clientWidth);
    resize();
    const observer = new ResizeObserver(resize);
    observer.observe(node);
    return () => observer.disconnect();
  }, []);
  if (!snapshot) return null;
  const choose = (photo: PhotoSummary) => { void run('library.select', { ids: [photo.id], active: photo.id, mode: 'replace' }); };
  const move = (delta: number, event: KeyboardEvent) => {
    event.preventDefault();
    if (active === null) return;
    const local = slice.photos.findIndex((photo) => photo.id === active);
    if (local < 0) {
      void run(delta < 0 ? 'library.previous' : 'library.next', {});
      return;
    }
    const next = Math.max(0, Math.min(slice.photos.length - 1, local + delta));
    const photo = slice.photos[next];
    if (photo) choose(photo);
    else void run(delta < 0 ? 'library.previous' : 'library.next', {});
  };
  const totalWidth = Math.max(viewportWidth, snapshot.total * (ITEM_WIDTH + ITEM_GAP) + ITEM_GAP);
  return <div className="lc-filmstrip" aria-label="Filmstrip">
    <div className="lc-filmstrip-scroll" ref={viewport} onScroll={(event) => setScrollLeft(event.currentTarget.scrollLeft)}>
      <div className="lc-filmstrip-spacer" style={{ width: totalWidth }}>
        <div className="lc-filmstrip-window" style={{ left: chunkOffset * (ITEM_WIDTH + ITEM_GAP) }} onKeyDown={(event) => {
          if (event.key === 'ArrowRight') move(1, event);
          if (event.key === 'ArrowLeft') move(-1, event);
          if (event.key === 'Home') { event.preventDefault(); void run('library.previous', {}); }
          if (event.key === 'End') { event.preventDefault(); void run('library.next', {}); }
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
