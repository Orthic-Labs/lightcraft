import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { PointerEvent } from "react";
import { PhotoPreview } from "../preview/PhotoPreview";
import Filmstrip from "../library/Filmstrip";
import { useDesktop } from "../desktop";
import type { DesktopSnapshot, PhotoSummary, UiState, ViewMode } from "../types";
import { Icon } from "../icons";
import type { IconName } from "../icons";
import "./StageWorkspace.css";

type Point = { x: number; y: number };
type Box = { x0: number; y0: number; x1: number; y1: number };
type DevelopShape = Record<string, unknown>;

const views: Array<{ id: ViewMode; label: string; key: string }> = [
  { id: "detail", label: "Detail", key: "D" },
  { id: "compare", label: "Compare", key: "C" },
  { id: "survey", label: "Survey", key: "N" },
  { id: "reference", label: "Reference", key: "R" },
  { id: "people", label: "People", key: "P" },
];

const tools: Array<{ id: UiState["panel"]; label: string; icon: IconName }> = [
  { id: "edit", label: "Edit", icon: "develop" },
  { id: "crop", label: "Crop", icon: "crop" },
  { id: "remove", label: "Remove", icon: "remove" },
  { id: "masking", label: "Masking", icon: "masking" },
  { id: "redeye", label: "Red Eye", icon: "redeye" },
  { id: "presets", label: "Presets", icon: "presets" },
  { id: "versions", label: "Versions", icon: "versions" },
  { id: "activity", label: "History", icon: "history" },
  { id: "keywords", label: "Keywords", icon: "keywords" },
  { id: "info", label: "Info", icon: "info" },
];

const maskTools = [
  ["brush", "Brush"],
  ["linear", "Linear Gradient"],
  ["radial", "Radial Gradient"],
  ["composition", "Composition"],
  ["selection", "Selection"],
  ["pins", "Pins"],
] as const;

function record(value: unknown): DevelopShape {
  return value && typeof value === "object" && !Array.isArray(value) ? (value as DevelopShape) : {};
}

function number(value: unknown, fallback = 0): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function point(value: unknown): Point | null {
  if (Array.isArray(value) && value.length >= 2) return { x: number(value[0]), y: number(value[1]) };
  if (value && typeof value === "object") {
    const p = value as Record<string, unknown>;
    if (typeof p.x === "number" && typeof p.y === "number") return { x: p.x, y: p.y };
  }
  return null;
}

function box(value: unknown): Box | null {
  if (Array.isArray(value) && value.length >= 4) return { x0: number(value[0]), y0: number(value[1]), x1: number(value[2]), y1: number(value[3]) };
  if (value && typeof value === "object") {
    const p = value as Record<string, unknown>;
    if (["x0", "y0", "x1", "y1"].every((k) => typeof p[k] === "number")) return { x0: number(p.x0), y0: number(p.y0), x1: number(p.x1), y1: number(p.y1) };
  }
  return null;
}

function cropOf(develop: DevelopShape): { rect: Box; angle: number } {
  const crop = record(develop.crop);
  const geometry = record(crop.geometry);
  const rect = box(geometry.rect ?? crop.rect) ?? { x0: 0, y0: 0, x1: 1, y1: 1 };
  return { rect, angle: number(geometry.angle ?? crop.angle) };
}

function invertPhotoPoint(event: PointerEvent<HTMLElement>, element: HTMLElement, develop: DevelopShape, sourceAspect: number): Point {
  const bounds = element.getBoundingClientRect();
  const crop = cropOf(develop);
  const orientation = String(develop.orientation ?? "normal").toLowerCase();
  const orientedAspect = ["rotate90", "rotate270", "transpose", "transverse"].includes(orientation) ? 1 / Math.max(0.001, sourceAspect) : sourceAspect;
  const outputAspect = orientedAspect * Math.max(0.001, crop.rect.x1 - crop.rect.x0) / Math.max(0.001, crop.rect.y1 - crop.rect.y0);
  const boxAspect = bounds.height > 0 ? bounds.width / bounds.height : outputAspect;
  const drawWidth = boxAspect > outputAspect ? bounds.height * outputAspect : bounds.width;
  const drawHeight = boxAspect > outputAspect ? bounds.height : bounds.width / outputAspect;
  const drawLeft = bounds.left + (bounds.width - drawWidth) / 2;
  const drawTop = bounds.top + (bounds.height - drawHeight) / 2;
  let px = drawWidth > 0 ? (event.clientX - drawLeft) / drawWidth : 0.5;
  let py = drawHeight > 0 ? (event.clientY - drawTop) / drawHeight : 0.5;
  px = Math.max(0, Math.min(1, px));
  py = Math.max(0, Math.min(1, py));
  const cropValue = record(develop.crop);
  if (cropValue.flip_h === true || cropValue.flipH === true) px = 1 - px;
  if (cropValue.flip_v === true || cropValue.flipV === true) py = 1 - py;
  const straightX = crop.rect.x0 + px * (crop.rect.x1 - crop.rect.x0);
  const straightY = crop.rect.y0 + py * (crop.rect.y1 - crop.rect.y0);
  const radians = (-crop.angle * Math.PI) / 180;
  const cx = 0.5;
  const cy = 0.5;
  const x = straightX - cx;
  const y = straightY - cy;
  return { x: Math.max(0, Math.min(1, x * Math.cos(radians) - y * Math.sin(radians) + cx)), y: Math.max(0, Math.min(1, x * Math.sin(radians) + y * Math.cos(radians) + cy)) };
}

function rotatePoint(value: Point, degrees: number): Point {
  const radians = (degrees * Math.PI) / 180;
  const x = value.x - 0.5;
  const y = value.y - 0.5;
  return { x: x * Math.cos(radians) - y * Math.sin(radians) + 0.5, y: x * Math.sin(radians) + y * Math.cos(radians) + 0.5 };
}

function cropHandleAt(value: Point, rect: Box): number | null {
  const handles = [{ x: rect.x0, y: rect.y0 }, { x: rect.x1, y: rect.y0 }, { x: rect.x1, y: rect.y1 }, { x: rect.x0, y: rect.y1 }, { x: (rect.x0 + rect.x1) / 2, y: rect.y0 }, { x: rect.x1, y: (rect.y0 + rect.y1) / 2 }, { x: (rect.x0 + rect.x1) / 2, y: rect.y1 }, { x: rect.x0, y: (rect.y0 + rect.y1) / 2 }];
  let nearest: number | null = null;
  let distance = 0.045;
  handles.forEach((handle, index) => {
    const next = Math.hypot(value.x - handle.x, value.y - handle.y);
    if (next < distance) { distance = next; nearest = index; }
  });
  return nearest;
}

function photosIn(snapshot: DesktopSnapshot | null): PhotoSummary[] {
  const source = record(snapshot?.source);
  const cached = (snapshot as DesktopSnapshot & { photos?: PhotoSummary[] } | null)?.photos ?? (Array.isArray(source.photos) ? source.photos as PhotoSummary[] : undefined);
  return cached ?? [];
}

function photoForId(snapshot: DesktopSnapshot | null, id: number | null): PhotoSummary | null {
  return id == null ? null : photosIn(snapshot).find((photo) => photo.id === id) ?? null;
}

function inspectedSummary(value: unknown, fallbackId: number): PhotoSummary | null {
  const source = record(value);
  const meta = record(source.meta);
  const id = number(source.id, fallbackId);
  const width = number(source.width ?? source.w);
  const height = number(source.height ?? source.h);
  if (!Number.isFinite(id) || width <= 0 || height <= 0) return null;
  const flag = source.flag === "pick" || source.flag === "reject" ? source.flag : "none";
  const label = typeof source.label === "string" ? source.label : null;
  return {
    id,
    fileName: typeof source.file_name === "string" ? source.file_name : typeof source.fileName === "string" ? source.fileName : `Photo ${id}`,
    format: typeof source.format === "string" ? source.format : "",
    kind: typeof source.kind === "string" ? source.kind : "image",
    w: width,
    h: height,
    captured: typeof source.captured === "string" ? source.captured : null,
    rating: number(source.rating),
    flag,
    label,
    edited: typeof source.edited === "string" || source.edited === true,
    title: typeof meta.title === "string" ? meta.title : "",
    keywords: Array.isArray(meta.keywords) ? meta.keywords.filter((item): item is string => typeof item === "string") : [],
    camera: typeof meta.camera === "string" ? meta.camera : "",
    deleted: source.deleted === true,
    copyOf: typeof source.copy_of === "number" ? source.copy_of : typeof source.copyOf === "number" ? source.copyOf : null,
    copyName: typeof source.copy_name === "string" ? source.copy_name : typeof source.copyName === "string" ? source.copyName : undefined,
    previewOnly: source.preview_only === true || source.previewOnly === true,
  };
}

function useInspectedPhotos(ids: number[]): Map<number, PhotoSummary> {
  const { run } = useDesktop();
  const key = ids.join(",");
  const [photos, setPhotos] = useState<Map<number, PhotoSummary>>(() => new Map());
  useEffect(() => {
    let live = true;
    setPhotos(new Map());
    if (!ids.length) return () => { live = false; };
    void Promise.all(ids.map(async (id) => {
      try {
        return [id, inspectedSummary(await run("photo.inspect", { id }), id)] as const;
      } catch {
        return [id, null] as const;
      }
    })).then((entries) => {
      if (!live) return;
      setPhotos(new Map(entries.flatMap(([id, photo]) => photo ? [[id, photo] as const] : [])));
    });
    return () => { live = false; };
  }, [key, run]);
  return photos;
}

function previewDimensions(photo: PhotoSummary | null, previewEdge: number): { width: number; height: number } {
  const edge = Math.max(256, Math.min(8192, Math.round(Number.isFinite(previewEdge) ? previewEdge : 2560)));
  const width = photo && Number.isFinite(photo.w) && photo.w > 0 ? photo.w : 4;
  const height = photo && Number.isFinite(photo.h) && photo.h > 0 ? photo.h : 3;
  const scale = edge / Math.max(width, height);
  return { width: Math.max(1, Math.round(width * scale)), height: Math.max(1, Math.round(height * scale)) };
}

function previewSlot(view: ViewMode, index = 0): string {
  return `${view}-${index === 0 ? "main" : `candidate-${index}`}`;
}

function titleFor(photo: PhotoSummary | null): string {
  return photo?.title || photo?.fileName || "No photo selected";
}

function PreviewPane({
  photoId,
  photo,
  previewEdge,
  slot,
  viewGeneration,
  before,
  className,
  onHistogram,
}: {
  photoId: number | null;
  photo: PhotoSummary | null;
  previewEdge: number;
  slot: string;
  viewGeneration: number;
  before?: boolean;
  className?: string;
  onHistogram: (value: unknown) => void;
}) {
  if (photoId == null) return <div className={`stage-empty ${className ?? ""}`}>Select a photo to view its decoded preview</div>;
  if (!photo) return <div className={`stage-empty ${className ?? ""}`}>Loading photo metadata…</div>;
  const dimensions = previewDimensions(photo, previewEdge);
  return <PhotoPreview photoId={photoId} slot={slot} viewGeneration={viewGeneration} width={dimensions.width} height={dimensions.height} quality="full" before={before} className={className} onHistogram={onHistogram} />;
}

function StageImage({
  photoId,
  photo,
  previewEdge,
  slot,
  viewGeneration,
  before,
  develop,
  onHistogram,
  onPointerDown,
  onPointerMove,
  onPointerUp,
}: {
  photoId: number | null;
  photo: PhotoSummary | null;
  previewEdge: number;
  slot: string;
  viewGeneration: number;
  before?: boolean;
  develop: DevelopShape;
  onHistogram: (value: unknown) => void;
  onPointerDown?: (event: PointerEvent<HTMLElement>) => void;
  onPointerMove?: (event: PointerEvent<HTMLElement>) => void;
  onPointerUp?: (event: PointerEvent<HTMLElement>) => void;
}) {
  const crop = cropOf(develop);
  return (
    <div className="stage-image-wrap" onPointerDown={onPointerDown} onPointerMove={onPointerMove} onPointerUp={onPointerUp}>
      <div className="stage-image-transform" style={{ ["--stage-angle" as string]: `${crop.angle}deg` }}>
        <PreviewPane photoId={photoId} photo={photo} previewEdge={previewEdge} slot={slot} viewGeneration={viewGeneration} before={before} className="stage-preview" onHistogram={onHistogram} />
      </div>
    </div>
  );
}

function CropOverlay({ develop, guide = "thirds" }: { develop: DevelopShape; guide?: string }) {
  const { rect } = cropOf(develop);
  return <div className="crop-overlay" style={{ left: `${rect.x0 * 100}%`, top: `${rect.y0 * 100}%`, width: `${Math.max(0, rect.x1 - rect.x0) * 100}%`, height: `${Math.max(0, rect.y1 - rect.y0) * 100}%`, transform: `rotate(${cropOf(develop).angle}deg)` }}><i className="crop-handle crop-nw" /><i className="crop-handle crop-ne" /><i className="crop-handle crop-sw" /><i className="crop-handle crop-se" />{guide !== "off" ? <><i className="crop-guide crop-guide-v1" /><i className="crop-guide crop-guide-v2" /><i className="crop-guide crop-guide-h1" /><i className="crop-guide crop-guide-h2" /></> : null}</div>;
}

function MaskOverlay({ develop, pins }: { develop: DevelopShape; pins: boolean }) {
  const masks = Array.isArray(develop.masks) ? develop.masks : [];
  const active = number(develop.activeMask ?? develop.active_mask, -1);
  const visible = masks.flatMap((mask) => {
    const m = record(mask);
    const components = Array.isArray(m.components) ? m.components : [];
    return components.map((component) => record(component)).map((component) => record(component.shape));
  });
  return <div className="mask-overlay" aria-hidden="true">{visible.map((shape, index) => {
    const rect = box(shape.rect ?? shape.bounds) ?? { x0: 0.12 + index * 0.03, y0: 0.18 + index * 0.03, x1: 0.55, y1: 0.75 };
    return <span key={`${active}-${index}`} className="mask-shape" style={{ left: `${rect.x0 * 100}%`, top: `${rect.y0 * 100}%`, width: `${(rect.x1 - rect.x0) * 100}%`, height: `${(rect.y1 - rect.y0) * 100}%` }} />;
  })}{pins && visible.map((_, index) => <b key={`pin-${index}`} className="mask-pin" style={{ left: `${(0.22 + index * 0.1) * 100}%`, top: `${(0.3 + index * 0.1) * 100}%` }}>•</b>)}</div>;
}

function SpotsOverlay({ develop, eyes }: { develop: DevelopShape; eyes?: boolean }) {
  const values = Array.isArray(eyes ? develop.red_eye ?? develop.redEye : develop.spots) ? (eyes ? develop.red_eye ?? develop.redEye : develop.spots) as unknown[] : [];
  return <div className="spots-overlay" aria-hidden="true">{values.map((value, index) => {
    const s = record(value);
    const center = point(s.center ?? s.position) ?? { x: 0.5, y: 0.5 };
    const rx = number(s.rx ?? s.radius, 0.045);
    const ry = number(s.ry ?? s.radius, rx);
    return <span key={`${eyes ? "eye" : "spot"}-${index}`} className={eyes ? "red-eye-shape" : "spot-shape"} style={{ left: `${center.x * 100}%`, top: `${center.y * 100}%`, width: `${rx * 200}%`, height: `${ry * 200}%` }} />;
  })}</div>;
}

function Navigator({ zoom, pan, onChange }: { zoom: number; pan: Point; onChange: (p: Point) => void }) {
  return <aside className="navigator" aria-label="Navigator">
    <div className="navigator-preview"><span style={{ left: `${pan.x * 100}%`, top: `${pan.y * 100}%`, width: `${Math.min(100, 100 / Math.max(1, zoom))}%`, height: `${Math.min(100, 100 / Math.max(1, zoom))}%` }} /></div>
    <div className="navigator-controls"><button type="button" onClick={() => onChange({ x: Math.max(0, pan.x - 0.05), y: pan.y })} aria-label="Pan left">←</button><button type="button" onClick={() => onChange({ x: pan.x, y: Math.max(0, pan.y - 0.05) })} aria-label="Pan up">↑</button><button type="button" onClick={() => onChange({ x: pan.x, y: Math.min(1, pan.y + 0.05) })} aria-label="Pan down">↓</button><button type="button" onClick={() => onChange({ x: Math.min(1, pan.x + 0.05), y: pan.y })} aria-label="Pan right">→</button></div>
  </aside>;
}

function ToolStrip({ ui, setUi }: { ui: UiState; setUi: (patch: Partial<UiState>) => void }) {
  return <nav className="stage-toolstrip" aria-label="Develop tools">{tools.map((tool) => <button key={tool.id ?? "none"} type="button" className={ui.panel === tool.id ? "selected" : ""} onClick={() => setUi({ panel: tool.id, view: tool.id && ["edit", "crop", "remove", "masking", "redeye", "presets", "versions", "activity", "keywords", "info"].includes(tool.id) && tool.id !== "info" ? "detail" : ui.view })} title={tool.label} aria-label={tool.label}><span className="stage-tool-icon" aria-hidden="true"><Icon name={tool.icon} size={19} /></span><small>{tool.label}</small></button>)}</nav>;
}

function MaskToolbar({ ui, setUi, run }: { ui: UiState; setUi: (patch: Partial<UiState>) => void; run: (id: string, params?: Record<string, unknown>) => Promise<unknown> }) {
  return <div className="mask-toolbar" role="toolbar" aria-label="Mask tools">{maskTools.map(([id, label]) => <button key={id} type="button" className={ui.tool === id ? "selected" : ""} onClick={() => { setUi({ tool: id, maskOverlay: true }); if (id === "linear" || id === "radial" || id === "brush") void run("mask.add", { kind: id }); if (id === "composition") void run("mask.addComponent", { kind: "brush", op: "add" }).catch(() => run("mask.add", { kind: "brush" })); }} title={label}>{label}</button>)}<button type="button" className={ui.maskOverlay ? "selected" : ""} onClick={() => setUi({ maskOverlay: !ui.maskOverlay })}>Overlay</button><button type="button" className={ui.maskPins ? "selected" : ""} onClick={() => setUi({ maskPins: !ui.maskPins })}>Pins</button></div>;
}

function CropToolbar({ ui, setUi, run }: { ui: UiState; setUi: (patch: Partial<UiState>) => void; run: (id: string, params?: Record<string, unknown>) => Promise<unknown> }) {
  return <div className="mask-toolbar crop-toolbar" role="toolbar" aria-label="Crop options"><button type="button" onClick={() => void run("crop.aspect", { aspect: "toggle" })}>Aspect</button><button type="button" onClick={() => void run("crop.rotateAspect", {})}>Rotate Aspect</button><button type="button" onClick={() => void run("crop.autoStraighten", {})}>Auto Straighten</button><button type="button" onClick={() => setUi({ cropOverlay: ui.cropOverlay === "off" ? "thirds" : "off" })} aria-pressed={ui.cropOverlay !== "off"}>Guides</button><button type="button" onClick={() => setUi({ cropOverlay: ui.cropOverlay === "thirds" ? "diagonal" : "thirds" })}>Guide Type</button><button type="button" onClick={() => void run("crop.reset", {})}>Reset Crop</button></div>;
}

function RemoveToolbar({ ui, setUi, run }: { ui: UiState; setUi: (patch: Partial<UiState>) => void; run: (id: string, params?: Record<string, unknown>) => Promise<unknown> }) {
  return <div className="mask-toolbar crop-toolbar" role="toolbar" aria-label="Remove options"><button type="button" className={ui.tool === "remove" ? "selected" : ""} onClick={() => setUi({ tool: "remove" })}>Remove</button><button type="button" className={ui.tool === "heal" ? "selected" : ""} onClick={() => setUi({ tool: "heal" })}>Heal</button><button type="button" className={ui.tool === "clone" ? "selected" : ""} onClick={() => setUi({ tool: "clone" })}>Clone</button><button type="button" onClick={() => void run("spot.findDust", { add: true })}>Find Dust</button><button type="button" onClick={() => void run("spot.delete", {})}>Delete Spot</button></div>;
}

function Footer({ ui, setUi, run, onNative }: { ui: UiState; setUi: (patch: Partial<UiState>) => void; run: (id: string, params?: Record<string, unknown>) => Promise<unknown>; onNative: (action: string, params?: Record<string, unknown>) => Promise<unknown> }) {
  const zoomValue = typeof ui.zoom === "number" ? ui.zoom : ui.zoom === "fill" ? 1.25 : 1;
  return <footer className="stage-footer"><div className="footer-left"><button type="button" onClick={() => setUi({ filmstrip: !ui.filmstrip })} aria-pressed={ui.filmstrip}>Filmstrip</button><button type="button" onClick={() => setUi({ navigator: !ui.navigator })} aria-pressed={ui.navigator}>Navigator</button><button type="button" onClick={() => setUi({ beforeAfter: ui.beforeAfter === "off" ? "sideBySide" : "off" })} aria-pressed={ui.beforeAfter !== "off"}>Before / After</button><button type="button" onClick={() => setUi({ clipping: !ui.clipping })} aria-pressed={ui.clipping}>Clipping</button><button type="button" onClick={() => setUi({ softProof: !ui.softProof })} aria-pressed={ui.softProof}>Proof</button></div><div className="footer-center"><button type="button" onClick={() => setUi({ zoom: "fit" })}>Fit</button><button type="button" onClick={() => setUi({ zoom: "fill" })}>Fill</button><button type="button" onClick={() => setUi({ zoom: 1 })}>100%</button><input aria-label="Zoom" type="range" min="0.25" max="4" step="0.05" value={zoomValue} onChange={(event) => setUi({ zoom: Number(event.target.value) })} /><output>{Math.round(zoomValue * 100)}%</output></div><div className="footer-right"><button type="button" onClick={() => setUi({ infoOverlay: ui.infoOverlay ? 0 : 1 })}>Info</button><button type="button" onClick={() => setUi({ slideshow: !ui.slideshow })}>{ui.slideshow ? "Pause" : "Slideshow"}</button><button type="button" onClick={() => void onNative("fullscreen", { enabled: true })}>Fullscreen</button><button type="button" onClick={() => void onNative("secondWindow", { view: "detail" })}>Second window</button><button type="button" onClick={() => void run("view.beforeAfter", {})}>Compare</button></div></footer>;
}

function InfoOverlay({ photo, develop, onClose }: { photo: PhotoSummary | null; develop: DevelopShape; onClose: () => void }) {
  if (!photo) return null;
  const wb = record(develop.wb ?? develop.whiteBalance);
  return <aside className="info-overlay"><button type="button" onClick={onClose} aria-label="Close photo info">×</button><strong>{titleFor(photo)}</strong><span>{photo.w} × {photo.h} · {photo.format}</span><span>{photo.camera || "Camera unknown"}</span><span>WB {number(wb.temp, 0).toFixed(0)}K · {number(wb.tint, 0).toFixed(0)}</span></aside>;
}

function MultiViewActions({ view, candidate, reference, run, setUi }: { view: ViewMode; candidate: number | null; reference: number | null; run: (id: string, params?: Record<string, unknown>) => Promise<unknown>; setUi: (patch: Partial<UiState>) => void }) {
  if (view !== "compare" && view !== "reference") return null;
  return <div className="multi-view-actions"><button type="button" onClick={() => void run("compare.swap", {})}>Swap</button><button type="button" disabled={candidate == null} onClick={() => void run("compare.makeSelect", {})}>Make Select</button>{view === "reference" ? <button type="button" onClick={() => { setUi({ referenceId: reference }); void run("photo.setReference", { id: reference }); }}>Keep Reference</button> : null}</div>;
}

function BeforeAfterStage({ mode, photoId, photo, previewEdge, viewGeneration, develop, onHistogram, onPointerDown, onPointerMove, onPointerUp }: { mode: UiState["beforeAfter"]; photoId: number | null; photo: PhotoSummary | null; previewEdge: number; viewGeneration: number; develop: DevelopShape; onHistogram: (value: unknown) => void; onPointerDown: (event: PointerEvent<HTMLElement>) => void; onPointerMove: (event: PointerEvent<HTMLElement>) => void; onPointerUp: (event: PointerEvent<HTMLElement>) => void }) {
  const vertical = mode === "topBottom" || mode === "splitTopBottom";
  return <div className={`before-after-panes ${vertical ? "vertical" : "horizontal"} ${mode === "split" || mode === "splitTopBottom" ? "split-mode" : ""}`}><div className="before-after-pane"><StageImage photoId={photoId} photo={photo} previewEdge={previewEdge} slot="before" before viewGeneration={viewGeneration} develop={develop} onHistogram={onHistogram} /><span className="pane-label">Before</span></div><div className="before-after-pane"><StageImage photoId={photoId} photo={photo} previewEdge={previewEdge} slot="after" viewGeneration={viewGeneration} develop={develop} onHistogram={onHistogram} onPointerDown={onPointerDown} onPointerMove={onPointerMove} onPointerUp={onPointerUp} /><span className="pane-label">After</span></div></div>;
}

export function StageWorkspace() {
  const { snapshot, ui, setUi, run, native, histogram, setHistogram, error, notice } = useDesktop();
  const stageRef = useRef<HTMLDivElement>(null);
  const active = snapshot?.active ?? null;
  const develop = record(snapshot?.develop);
  const previewGeneration = snapshot?.viewGeneration ?? 0;
  const [pan, setPan] = useState<Point>({ x: 0, y: 0 });
  const [drag, setDrag] = useState<{ kind: string; start: Point; last: Point; points?: Point[]; startBox?: Box; handle?: number; startAngle?: number; startPointerAngle?: number } | null>(null);
  const [candidateIndex, setCandidateIndex] = useState(0);
  const burst = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pending = useRef<{ id: string; params: Record<string, unknown> } | null>(null);

  const selection: number[] = useMemo(() => snapshot?.selection ?? [], [snapshot?.selection]);
  const candidateIds = selection.filter((id) => id !== active);
  const candidate = candidateIds[candidateIndex % Math.max(1, candidateIds.length)] ?? null;
  const detailIds = useMemo(() => {
    const visibleIds = ui.view === "survey" ? selection
      : ui.view === "compare" ? [candidate]
      : ui.view === "reference" ? [ui.referenceId] : [];
    return Array.from(new Set([active, ...visibleIds].filter((id): id is number => typeof id === "number")));
  }, [active, candidate, selection, ui.referenceId, ui.view]);
  const inspected = useInspectedPhotos(detailIds);
  const photoFor = useCallback((id: number | null) => photoForId(snapshot, id) ?? (id == null ? null : inspected.get(id) ?? null), [inspected, snapshot]);
  const photo = photoFor(active);
  const zoom = typeof ui.zoom === "number" ? ui.zoom : ui.zoom === "fill" ? 1.25 : 1;
  const sourceAspect = photo && photo.h > 0 ? photo.w / photo.h : 1.5;
  const previewEdge = Math.max(256, Math.min(8192, Math.round(Number.isFinite(ui.previewEdge) ? ui.previewEdge : 2560)));

  const send = useCallback(async (id: string, params: Record<string, unknown> = {}) => { await run(id, params); }, [run]);

  const commitBurst = useCallback(() => {
    if (burst.current) clearTimeout(burst.current);
    const next = pending.current;
    pending.current = null;
    if (next) void send(next.id, next.params);
  }, [send]);

  const gestureValue = useCallback((id: string, params: Record<string, unknown>) => {
    pending.current = { id, params };
    if (burst.current) clearTimeout(burst.current);
    burst.current = setTimeout(commitBurst, 400);
  }, [commitBurst]);

  useEffect(() => () => { if (burst.current) clearTimeout(burst.current); }, []);

  const pointAt = useCallback((event: PointerEvent<HTMLElement>) => invertPhotoPoint(event, event.currentTarget, develop, sourceAspect), [develop, sourceAspect]);

  const beginPointer = useCallback((event: PointerEvent<HTMLElement>) => {
    if (active == null) return;
    const p = pointAt(event);
    const crop = cropOf(develop).rect;
    if (ui.panel === "crop") {
      event.currentTarget.setPointerCapture(event.pointerId);
      const angle = cropOf(develop).angle;
      const straight = rotatePoint(p, angle);
      const handle = cropHandleAt(straight, crop);
      const center = { x: (crop.x0 + crop.x1) / 2, y: (crop.y0 + crop.y1) / 2 };
      const pointerAngle = Math.atan2(straight.y - center.y, straight.x - center.x);
      const kind = handle == null && (straight.x < crop.x0 || straight.x > crop.x1 || straight.y < crop.y0 || straight.y > crop.y1) ? "cropAngle" : "crop";
      setDrag({ kind, start: straight, last: straight, startBox: crop, handle: handle ?? 8, startAngle: angle, startPointerAngle: pointerAngle });
      void send("develop.beginInteraction", { label: "Crop" });
      return;
    }
    if (ui.panel === "remove") { event.currentTarget.setPointerCapture(event.pointerId); setDrag({ kind: "spot", start: p, last: p }); void send("develop.beginInteraction", { label: "Remove" }); return; }
    if (ui.panel === "redeye") { event.currentTarget.setPointerCapture(event.pointerId); setDrag({ kind: "eye", start: p, last: p }); void send("develop.beginInteraction", { label: "Red Eye" }); return; }
    if (ui.panel === "masking") {
      event.currentTarget.setPointerCapture(event.pointerId);
      setDrag({ kind: ui.tool || "brush", start: p, last: p, points: ui.tool === "brush" ? [p] : undefined });
      void send("develop.beginInteraction", { label: "Mask" });
      if (ui.tool === "brush") void send("mask.brushStroke", { points: [[p.x, p.y]], size: ui.brushSize / 1000, feather: ui.brushFeather / 100 });
      if (ui.tool === "selection") void send("mask.objectPoint", { x: p.x, y: p.y, exclude: false });
    }
    if (ui.panel === "edit" && ui.tool === "wbPicker") { void send("develop.wbPick", { x: p.x, y: p.y }); setUi({ tool: "" }); }
  }, [active, develop, pointAt, send, setUi, ui.brushFeather, ui.brushSize, ui.panel, ui.tool]);

  const movePointer = useCallback((event: PointerEvent<HTMLElement>) => {
    if (!drag) return;
    const p = pointAt(event);
    if (drag.kind === "crop" && drag.startBox) {
      const straight = rotatePoint(p, cropOf(develop).angle);
      const dx = straight.x - drag.start.x;
      const dy = straight.y - drag.start.y;
      const next = { x0: drag.startBox.x0, y0: drag.startBox.y0, x1: drag.startBox.x1, y1: drag.startBox.y1 };
      if (drag.handle === 0 || drag.handle === 7 || drag.handle === 3) next.x0 += dx;
      if (drag.handle === 1 || drag.handle === 5 || drag.handle === 2) next.x1 += dx;
      if (drag.handle === 0 || drag.handle === 4 || drag.handle === 1) next.y0 += dy;
      if (drag.handle === 3 || drag.handle === 6 || drag.handle === 2) next.y1 += dy;
      if (drag.handle === 8) { next.x0 += dx; next.x1 += dx; next.y0 += dy; next.y1 += dy; }
      const x0 = Math.max(0, Math.min(0.98, Math.min(next.x0, next.x1 - 0.02)));
      const y0 = Math.max(0, Math.min(0.98, Math.min(next.y0, next.y1 - 0.02)));
      const x1 = Math.min(1, Math.max(0.02, Math.max(next.x1, next.x0 + 0.02)));
      const y1 = Math.min(1, Math.max(0.02, Math.max(next.y1, next.y0 + 0.02)));
      gestureValue("crop.set", { rect: [x0, y0, x1, y1] });
    } else if (drag.kind === "cropAngle" && drag.startBox != null && drag.startAngle != null && drag.startPointerAngle != null) {
      const straight = rotatePoint(p, cropOf(develop).angle);
      const center = { x: (drag.startBox.x0 + drag.startBox.x1) / 2, y: (drag.startBox.y0 + drag.startBox.y1) / 2 };
      const currentAngle = Math.atan2(straight.y - center.y, straight.x - center.x);
      const degrees = drag.startAngle + ((currentAngle - drag.startPointerAngle) * 180) / Math.PI;
      gestureValue("crop.straighten", { angle: Math.max(-45, Math.min(45, degrees)) });
    } else if (drag.kind === "brush") {
      const points = [...(drag.points ?? []), p];
      setDrag((current) => current ? { ...current, last: p, points } : current);
      gestureValue("mask.brushStroke", { points: points.map((value) => [value.x, value.y]), size: ui.brushSize / 1000, feather: ui.brushFeather / 100 });
    } else if (drag.kind === "linear") {
      gestureValue("mask.update", { shape: { kind: drag.kind, start: { x: drag.start.x, y: drag.start.y }, end: { x: p.x, y: p.y } } });
    } else if (drag.kind === "radial") {
      const orientation = String(develop.orientation ?? "normal").toLowerCase();
      const orientedAspect = ["rotate90", "rotate270", "transpose", "transverse"].includes(orientation) ? 1 / Math.max(0.001, sourceAspect) : sourceAspect;
      const longEdge = Math.max(orientedAspect, 1);
      const center = { x: (drag.start.x + p.x) / 2, y: (drag.start.y + p.y) / 2 };
      const rx = Math.max(0.001, Math.abs(p.x - drag.start.x) * orientedAspect / longEdge / 2);
      const ry = Math.max(0.001, Math.abs(p.y - drag.start.y) / longEdge / 2);
      gestureValue("mask.update", { shape: { kind: "radial", center, rx, ry, angle: 0, feather: 50, invert: false } });
    } else if (drag.kind === "spot") {
      setDrag((current) => current ? { ...current, last: p } : current);
    } else if (drag.kind === "eye") {
      setDrag((current) => current ? { ...current, last: p } : current);
    }
  }, [drag, gestureValue, pointAt, ui.brushFeather, ui.brushSize]);

  const endPointer = useCallback((event: PointerEvent<HTMLElement>) => {
    if (!drag) return;
    const p = pointAt(event);
    if (drag.kind === "spot") {
      const radius = Math.max(0.005, Math.abs(p.x - drag.start.x) || 0.025);
      const mode = ui.tool === "clone" ? "clone" : ui.tool === "heal" ? "heal" : "remove";
      void send("spot.add", { mode, points: [[drag.start.x, drag.start.y]], size: radius, source: [p.x - drag.start.x, p.y - drag.start.y] });
    } else if (drag.kind === "eye") {
      void send("redeye.add", { center: [drag.start.x, drag.start.y], rx: Math.max(0.01, Math.abs(p.x - drag.start.x)), ry: Math.max(0.01, Math.abs(p.y - drag.start.y)) });
    }
    commitBurst();
    void send("develop.endInteraction", {});
    setDrag(null);
  }, [commitBurst, drag, pointAt, send, ui.tool]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape" && drag) { void send("develop.cancelInteraction", {}); setDrag(null); return; }
      if (event.key === "Escape") { setUi({ infoOverlay: 0, slideshow: false }); return; }
      if (event.key === " ") { event.preventDefault(); setUi({ slideshow: !ui.slideshow }); return; }
      if (event.key === "ArrowLeft" || event.key === "ArrowRight") { event.preventDefault(); void send(event.key === "ArrowLeft" ? "library.previous" : "library.next", {}); }
      if (event.key.toLowerCase() === "w" && ui.panel === "edit") setUi({ tool: "wbPicker" });
      if (event.key.toLowerCase() === "o" && ui.panel === "crop") setUi({ cropOverlay: ui.cropOverlay === "thirds" ? "diagonal" : "thirds" });
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [drag, send, setUi, ui.cropOverlay, ui.panel, ui.slideshow]);

  const routeView = useCallback((view: ViewMode) => {
    setUi({ view, panel: view === "people" ? "info" : ui.panel });
    void (async () => {
      await send(`view.${view}`, {});
      if (view === "reference" && active != null && candidate != null && candidate !== active) {
        setUi({ referenceId: active });
        await send("library.select", { ids: [candidate], active: candidate, mode: "replace" });
      }
    })();
  }, [active, candidate, send, setUi, ui.panel]);
  const stageClass = `stage-workspace stage-${ui.view} ${ui.theme === "dark" ? "stage-dark" : "stage-light"} ${ui.beforeAfter !== "off" ? "has-before-after" : ""}`;

  return <section ref={stageRef} className={stageClass} aria-label="Photo stage">
    <div className="stage-topbar"><div className="stage-route-tabs">{views.map((view) => <button key={view.id} type="button" className={ui.view === view.id ? "selected" : ""} onClick={() => routeView(view.id)}><span>{view.label}</span><kbd>{view.key}</kbd></button>)}</div><div className="stage-photo-title"><strong>{titleFor(photo)}</strong><span>{photo ? `${photo.w} × ${photo.h}` : ""}</span></div><MultiViewActions view={ui.view} candidate={candidate} reference={ui.referenceId} run={run} setUi={setUi} /><div className="stage-status">{snapshot?.status.previewBuild ? <span className="stage-loading">Rendering preview…</span> : null}{error ? <span className="stage-error">{error}</span> : null}{notice ? <span>{notice}</span> : null}</div></div>
    <div className="stage-body">
      <div className="stage-canvas" style={{ ["--stage-zoom" as string]: zoom, ["--stage-pan-x" as string]: `${pan.x * 100}%`, ["--stage-pan-y" as string]: `${pan.y * 100}%` }}>
        {ui.view === "compare" ? <div className="compare-panes"><div className="compare-pane"><StageImage photoId={active} photo={photoFor(active)} previewEdge={previewEdge} slot={previewSlot(ui.view)} before={ui.beforeAfter === "original"} viewGeneration={previewGeneration} develop={develop} onHistogram={setHistogram} onPointerDown={beginPointer} onPointerMove={movePointer} onPointerUp={endPointer} /><span className="pane-label">Select</span></div><div className="compare-pane"><StageImage photoId={candidate} photo={photoFor(candidate)} previewEdge={previewEdge} slot={previewSlot(ui.view, 1)} viewGeneration={previewGeneration} develop={develop} onHistogram={setHistogram} /><span className="pane-label">Candidate {candidate ?? "—"}</span></div></div> : ui.view === "reference" ? <div className="compare-panes"><div className="compare-pane"><StageImage photoId={ui.referenceId} photo={photoFor(ui.referenceId)} previewEdge={previewEdge} slot="reference" viewGeneration={previewGeneration} develop={develop} onHistogram={setHistogram} /><span className="pane-label">Reference {ui.referenceId ?? "—"}</span></div><div className="compare-pane"><StageImage photoId={active} photo={photoFor(active)} previewEdge={previewEdge} slot="reference-active" viewGeneration={previewGeneration} develop={develop} onHistogram={setHistogram} onPointerDown={beginPointer} onPointerMove={movePointer} onPointerUp={endPointer} /><span className="pane-label">Active</span></div></div> : ui.view === "survey" ? <div className="survey-grid">{(selection.length ? selection : active == null ? [] : [active]).map((id, index) => <button type="button" className={id === active ? "survey-photo selected" : "survey-photo"} key={id} onClick={() => { setUi({ view: "detail" }); void send("library.select", { ids: [id], active: id, mode: "replace" }); }}><StageImage photoId={id} photo={photoFor(id)} previewEdge={previewEdge} slot={previewSlot(ui.view, index)} viewGeneration={previewGeneration} develop={develop} onHistogram={setHistogram} /><span>{index + 1}</span></button>)}</div> : ui.view === "people" ? <div className="people-stage"><StageImage photoId={active} photo={photoFor(active)} previewEdge={previewEdge} slot={previewSlot(ui.view)} viewGeneration={previewGeneration} develop={develop} onHistogram={setHistogram} /><div className="face-boxes">{Array.isArray(record(snapshot?.source).faces) ? (record(snapshot?.source).faces as unknown[]).map((face, index) => { const f = record(face); const r = box(f.rect ?? f.bounds) ?? { x0: 0.3 + index * 0.05, y0: 0.25, x1: 0.44 + index * 0.05, y1: 0.42 }; return <span key={index} style={{ left: `${r.x0 * 100}%`, top: `${r.y0 * 100}%`, width: `${(r.x1 - r.x0) * 100}%`, height: `${(r.y1 - r.y0) * 100}%` }} />; }) : null}</div></div> : ui.beforeAfter !== "off" && ui.beforeAfter !== "original" ? <BeforeAfterStage mode={ui.beforeAfter} photoId={active} photo={photoFor(active)} previewEdge={previewEdge} viewGeneration={previewGeneration} develop={develop} onHistogram={setHistogram} onPointerDown={beginPointer} onPointerMove={movePointer} onPointerUp={endPointer} /> : <div className="single-pane"><StageImage photoId={active} photo={photoFor(active)} previewEdge={previewEdge} slot={previewSlot(ui.view)} before={ui.beforeAfter === "original"} viewGeneration={previewGeneration} develop={develop} onHistogram={setHistogram} onPointerDown={beginPointer} onPointerMove={movePointer} onPointerUp={endPointer} />{ui.panel === "crop" ? <CropOverlay develop={develop} guide={ui.cropOverlay} /> : null}{ui.maskOverlay && ui.panel === "masking" ? <MaskOverlay develop={develop} pins={ui.maskPins} /> : null}{ui.panel === "remove" ? <SpotsOverlay develop={develop} /> : null}{ui.panel === "redeye" ? <SpotsOverlay develop={develop} eyes /> : null}{ui.clipping ? <div className="clipping-overlay" aria-label="Clipping preview" /> : null}</div>}
        {ui.infoOverlay ? <InfoOverlay photo={photo} develop={develop} onClose={() => setUi({ infoOverlay: 0 })} /> : null}
        {ui.navigator ? <Navigator zoom={zoom} pan={pan} onChange={setPan} /> : null}
        {histogram ? <div className="histogram-badge" aria-label="Histogram available">Histogram</div> : null}
      </div>
      <ToolStrip ui={ui} setUi={setUi} />
      {ui.panel === "crop" ? <CropToolbar ui={ui} setUi={setUi} run={run} /> : null}
      {ui.panel === "remove" ? <RemoveToolbar ui={ui} setUi={setUi} run={run} /> : null}
      {ui.panel === "masking" ? <MaskToolbar ui={ui} setUi={setUi} run={run} /> : null}
    </div>
    {ui.filmstrip ? <div className="stage-filmstrip"><Filmstrip /></div> : null}
    <Footer ui={ui} setUi={setUi} run={run} onNative={native} />
  </section>;
}

export default StageWorkspace;
