import assert from "node:assert/strict";
import test from "node:test";
import {
  longToNormalized,
  normalizedPhotoPoint,
  normalizedToLong,
  radialHandles,
  radialShapeFromDrag,
} from "../../apps/lightcraft-desktop/web/src/stage/maskGeometry.ts";

const bounds = { left: 10, top: 20, width: 1000, height: 700 };
const close = (actual, expected, message) => {
  assert.ok(Math.abs(actual - expected) < 1e-8, `${message}: ${actual} !== ${expected}`);
};
const closePoint = (actual, expected, message) => {
  close(actual.x, expected.x, `${message}.x`);
  close(actual.y, expected.y, `${message}.y`);
};
const rotateLong = (point, degrees, aspect) => {
  const center = normalizedToLong({ x: 0.5, y: 0.5 }, aspect);
  const source = normalizedToLong(point, aspect);
  const radians = degrees * Math.PI / 180;
  const dx = source.x - center.x;
  const dy = source.y - center.y;
  return longToNormalized({
    x: center.x + dx * Math.cos(radians) - dy * Math.sin(radians),
    y: center.y + dx * Math.sin(radians) + dy * Math.cos(radians),
  }, aspect);
};
const clientForSource = (source, { sourceAspect, orientation, crop, angle, zoom, pan }) => {
  const swaps = ["rotate90", "rotate270", "transpose", "transverse"].includes(orientation.toLowerCase());
  const aspect = swaps ? 1 / sourceAspect : sourceAspect;
  const outputAspect = aspect * (crop.x1 - crop.x0) / (crop.y1 - crop.y0);
  const boxAspect = bounds.width / bounds.height;
  const drawWidth = boxAspect > outputAspect ? bounds.height * outputAspect : bounds.width;
  const drawHeight = boxAspect > outputAspect ? bounds.height : bounds.width / outputAspect;
  const drawLeft = bounds.left + (bounds.width - drawWidth) / 2;
  const drawTop = bounds.top + (bounds.height - drawHeight) / 2;
  const straight = rotateLong(source, angle, aspect);
  const px = (straight.x - crop.x0) / (crop.x1 - crop.x0);
  const py = (straight.y - crop.y0) / (crop.y1 - crop.y0);
  const image = { x: drawLeft + px * drawWidth, y: drawTop + py * drawHeight };
  const radians = angle * Math.PI / 180;
  const dx = image.x - (bounds.left + bounds.width / 2);
  const dy = image.y - (bounds.top + bounds.height / 2);
  return {
    x: bounds.left + (bounds.width / 2 + (dx * Math.cos(radians) - dy * Math.sin(radians)) * zoom) + pan.x * bounds.width,
    y: bounds.top + (bounds.height / 2 + (dx * Math.sin(radians) + dy * Math.cos(radians)) * zoom) + pan.y * bounds.height,
  };
};

test("photo pointer inversion preserves landscape crop, straighten, zoom & pan", () => {
  const develop = { orientation: "normal", crop: { rect: { x0: 0.1, y0: 0.2, x1: 0.9, y1: 0.8 }, angle: 12 } };
  const source = { x: 0.72, y: 0.31 };
  const pointer = clientForSource(source, { sourceAspect: 2, orientation: "normal", crop: develop.crop.rect, angle: 12, zoom: 1.35, pan: { x: 0.08, y: -0.04 } });
  closePoint(normalizedPhotoPoint(pointer.x, pointer.y, bounds, develop, 2, 1.35, { x: 0.08, y: -0.04 }), source, "landscape inverse");
});

for (const orientation of ["rotate90", "rotate270"]) {
  test(`photo pointer inversion preserves portrait ${orientation} letterbox`, () => {
    const develop = { orientation, crop: { rect: { x0: 0.08, y0: 0.12, x1: 0.86, y1: 0.91 }, angle: -17 } };
    const source = { x: 0.33, y: 0.67 };
    const pointer = clientForSource(source, { sourceAspect: 2, orientation, crop: develop.crop.rect, angle: -17, zoom: 1, pan: { x: 0, y: 0 } });
    closePoint(normalizedPhotoPoint(pointer.x, pointer.y, bounds, develop, 2), source, `${orientation} inverse`);
  });
}

test("radial radii remain long-edge physical units across landscape & portrait", () => {
  const landscape = radialShapeFromDrag({ x: 0.2, y: 0.3 }, { x: 0.6, y: 0.7 }, 2, "normal");
  close(landscape.rx, 0.2, "landscape rx");
  close(landscape.ry, 0.1, "landscape ry");
  const portrait = radialShapeFromDrag({ x: 0.2, y: 0.3 }, { x: 0.6, y: 0.7 }, 2, "rotate90");
  close(portrait.rx, 0.1, "portrait rx");
  close(portrait.ry, 0.2, "portrait ry");
  const aspect = 2;
  const shape = { center: { x: 0.5, y: 0.5 }, rx: 0.2, ry: 0.1, angle: 31 };
  const [right, bottom] = radialHandles(shape, aspect);
  const center = normalizedToLong(shape.center, aspect);
  const rightLong = normalizedToLong(right, aspect);
  const bottomLong = normalizedToLong(bottom, aspect);
  close(Math.hypot(rightLong.x - center.x, rightLong.y - center.y), shape.rx, "rotated rx");
  close(Math.hypot(bottomLong.x - center.x, bottomLong.y - center.y), shape.ry, "rotated ry");
});
