export type Point = { x: number; y: number };
export type Box = { x0: number; y0: number; x1: number; y1: number };
export type DevelopShape = Record<string, unknown>;
export type PointerBounds = { left: number; top: number; width: number; height: number };

function record(value: unknown): DevelopShape {
  return value && typeof value === "object" && !Array.isArray(value) ? value as DevelopShape : {};
}

function number(value: unknown, fallback = 0): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function box(value: unknown): Box | null {
  if (Array.isArray(value) && value.length >= 4) return { x0: number(value[0]), y0: number(value[1]), x1: number(value[2]), y1: number(value[3]) };
  if (value && typeof value === "object") {
    const p = value as Record<string, unknown>;
    if (["x0", "y0", "x1", "y1"].every((key) => typeof p[key] === "number")) return { x0: number(p.x0), y0: number(p.y0), x1: number(p.x1), y1: number(p.y1) };
  }
  return null;
}

function shapePoint(value: unknown): Point | null {
  if (Array.isArray(value) && value.length >= 2) return { x: number(value[0]), y: number(value[1]) };
  if (value && typeof value === "object") {
    const p = value as Record<string, unknown>;
    if (typeof p.x === "number" && typeof p.y === "number") return { x: p.x, y: p.y };
  }
  return null;
}

function cropOf(develop: DevelopShape): { rect: Box; angle: number } {
  const crop = record(develop.crop);
  const geometry = record(crop.geometry);
  const rect = box(geometry.rect ?? crop.rect) ?? { x0: 0, y0: 0, x1: 1, y1: 1 };
  return { rect, angle: number(geometry.angle ?? crop.angle) };
}

export function orientedAspectFor(develop: DevelopShape, sourceAspect: number): number {
  const orientation = String(develop.orientation ?? "normal").toLowerCase();
  const aspect = Math.max(0.001, Number.isFinite(sourceAspect) ? sourceAspect : 1);
  return ["rotate90", "rotate270", "transpose", "transverse"].includes(orientation) ? 1 / aspect : aspect;
}

export function normalizedToLong(value: Point, aspect: number): Point {
  const safeAspect = Math.max(0.001, Number.isFinite(aspect) ? aspect : 1);
  const longEdge = Math.max(1, safeAspect);
  return { x: value.x * safeAspect / longEdge, y: value.y / longEdge };
}

export function longToNormalized(value: Point, aspect: number): Point {
  const safeAspect = Math.max(0.001, Number.isFinite(aspect) ? aspect : 1);
  const longEdge = Math.max(1, safeAspect);
  return { x: value.x * longEdge / safeAspect, y: value.y * longEdge };
}

export function rotateNormalizedPoint(value: Point, degrees: number, aspect: number): Point {
  const radians = degrees * Math.PI / 180;
  const center = normalizedToLong({ x: 0.5, y: 0.5 }, aspect);
  const source = normalizedToLong(value, aspect);
  const dx = source.x - center.x;
  const dy = source.y - center.y;
  return longToNormalized({
    x: center.x + dx * Math.cos(radians) - dy * Math.sin(radians),
    y: center.y + dx * Math.sin(radians) + dy * Math.cos(radians),
  }, aspect);
}

export function normalizedPhotoPoint(
  clientX: number,
  clientY: number,
  bounds: PointerBounds,
  develop: DevelopShape,
  sourceAspect: number,
  zoom = 1,
  pan: Point = { x: 0, y: 0 },
): Point {
  const crop = cropOf(develop);
  const orientedAspect = orientedAspectFor(develop, sourceAspect);
  const outputAspect = orientedAspect * Math.max(0.001, crop.rect.x1 - crop.rect.x0) / Math.max(0.001, crop.rect.y1 - crop.rect.y0);
  const boxAspect = bounds.height > 0 ? bounds.width / bounds.height : outputAspect;
  const drawWidth = boxAspect > outputAspect ? bounds.height * outputAspect : bounds.width;
  const drawHeight = boxAspect > outputAspect ? bounds.height : bounds.width / outputAspect;
  const drawLeft = bounds.left + (bounds.width - drawWidth) / 2;
  const drawTop = bounds.top + (bounds.height - drawHeight) / 2;
  const safeZoom = Math.max(0.001, Number.isFinite(zoom) ? zoom : 1);
  let tx = bounds.width > 0 ? (clientX - bounds.left) / bounds.width : 0.5;
  let ty = bounds.height > 0 ? (clientY - bounds.top) / bounds.height : 0.5;
  tx -= Number.isFinite(pan.x) ? pan.x : 0;
  ty -= Number.isFinite(pan.y) ? pan.y : 0;
  tx = (tx - 0.5) / safeZoom + 0.5;
  ty = (ty - 0.5) / safeZoom + 0.5;
  const radians = -crop.angle * Math.PI / 180;
  const dx = tx * bounds.width - bounds.width / 2;
  const dy = ty * bounds.height - bounds.height / 2;
  const screenX = bounds.left + bounds.width / 2 + dx * Math.cos(radians) - dy * Math.sin(radians);
  const screenY = bounds.top + bounds.height / 2 + dx * Math.sin(radians) + dy * Math.cos(radians);
  let px = drawWidth > 0 ? (screenX - drawLeft) / drawWidth : 0.5;
  let py = drawHeight > 0 ? (screenY - drawTop) / drawHeight : 0.5;
  px = Math.max(0, Math.min(1, px));
  py = Math.max(0, Math.min(1, py));
  const cropValue = record(develop.crop);
  if (cropValue.flip_h === true || cropValue.flipH === true) px = 1 - px;
  if (cropValue.flip_v === true || cropValue.flipV === true) py = 1 - py;
  const rotated = rotateNormalizedPoint({
    x: crop.rect.x0 + px * (crop.rect.x1 - crop.rect.x0),
    y: crop.rect.y0 + py * (crop.rect.y1 - crop.rect.y0),
  }, -crop.angle, orientedAspect);
  return { x: Math.max(0, Math.min(1, rotated.x)), y: Math.max(0, Math.min(1, rotated.y)) };
}

export function radialShapeFromDrag(start: Point, end: Point, sourceAspect: number, orientation = "normal"): DevelopShape {
  const rawAspect = Math.max(0.001, sourceAspect);
  const aspect = ["rotate90", "rotate270", "transpose", "transverse"].includes(String(orientation).toLowerCase()) ? 1 / rawAspect : rawAspect;
  const longEdge = Math.max(aspect, 1);
  return {
    kind: "radial",
    center: { x: (start.x + end.x) / 2, y: (start.y + end.y) / 2 },
    rx: Math.max(0.001, Math.abs(end.x - start.x) * aspect / longEdge / 2),
    ry: Math.max(0.001, Math.abs(end.y - start.y) / longEdge / 2),
    angle: 0,
    feather: 50,
    invert: false,
  };
}

function radialHandlePoint(center: Point, radius: Point, angle: number, aspect: number): Point {
  const centerLong = normalizedToLong(center, aspect);
  const next = {
    x: radius.x * Math.cos(angle) - radius.y * Math.sin(angle),
    y: radius.x * Math.sin(angle) + radius.y * Math.cos(angle),
  };
  return longToNormalized({ x: centerLong.x + next.x, y: centerLong.y + next.y }, aspect);
}

export function radialHandles(shape: DevelopShape, aspect: number): Point[] {
  const center = shapePoint(shape.center);
  const rx = number(shape.rx);
  const ry = number(shape.ry);
  if (!center || !(rx > 0 && ry > 0)) return [];
  const angle = number(shape.angle) * Math.PI / 180;
  return [{ x: rx, y: 0 }, { x: 0, y: ry }, { x: 0, y: -(ry + 0.06 / Math.max(1, aspect)) }].map((value) => radialHandlePoint(center, value, angle, aspect));
}

export function radialHandleAt(value: Point, shape: DevelopShape, aspect: number): "move" | "rx" | "ry" | "rotate" | null {
  const center = shapePoint(shape.center);
  const rx = number(shape.rx);
  const ry = number(shape.ry);
  if (!center || !(rx > 0 && ry > 0)) return null;
  const angle = number(shape.angle) * Math.PI / 180;
  const distance = (a: Point, b: Point) => Math.hypot(a.x - b.x, a.y - b.y);
  const [right, bottom, top] = radialHandles(shape, aspect);
  if (distance(value, right) < 0.045) return "rx";
  if (distance(value, bottom) < 0.045) return "ry";
  if (distance(value, top) < 0.045) return "rotate";
  const local = normalizedToLong({ x: value.x - center.x, y: value.y - center.y }, aspect);
  const localAngle = -angle;
  const rotated = { x: local.x * Math.cos(localAngle) - local.y * Math.sin(localAngle), y: local.x * Math.sin(localAngle) + local.y * Math.cos(localAngle) };
  return (rotated.x * rotated.x) / (rx * rx) + (rotated.y * rotated.y) / (ry * ry) <= 1 ? "move" : null;
}
