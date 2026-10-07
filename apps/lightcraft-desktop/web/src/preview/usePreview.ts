import { useCallback, useEffect, useRef, useState } from "react";
import * as previewApi from "../api";
import { requestPreview } from "../api";
import type { PreviewDescriptor, PreviewQuality, PreviewRequest } from "../desktop/types";

export type PreviewState =
  | { status: "idle"; descriptor: null; url: null; error: null }
  | { status: "loading"; descriptor: PreviewDescriptor | null; url: string | null; error: null }
  | { status: "ready"; descriptor: PreviewDescriptor; url: string; error: null }
  | { status: "error"; descriptor: PreviewDescriptor | null; url: string | null; error: string };

export interface UsePreviewOptions {
  photoId: number;
  slot: string;
  viewGeneration: number;
  width: number;
  height: number;
  quality?: PreviewQuality;
  before?: boolean;
  enabled?: boolean;
  onHistogram?: (histogram: unknown) => void;
}
export interface UsePreviewResult {
  state: PreviewState;
  request: PreviewRequest;
  retry: () => void;
  onImageLoad: () => void;
  onImageError: () => void;
}

const MAX_DIMENSION = 8192;
const MAX_SEQUENCE = Number.MAX_SAFE_INTEGER;
let nextSequenceValue = 0;

function nextSequence(): number {
  nextSequenceValue = nextSequenceValue >= MAX_SEQUENCE ? 1 : nextSequenceValue + 1;
  return nextSequenceValue;
}

function boundedDimension(value: number): number {
  return Math.max(1, Math.min(MAX_DIMENSION, Math.round(Number.isFinite(value) ? value : 1)));
}

/** Request native decoded pixels and ignore any result from an older generation or sequence. */
export function usePreview(options: UsePreviewOptions): UsePreviewResult {
  const latestSequence = useRef(0);
  const latestDescriptor = useRef<PreviewDescriptor | null>(null);
  const lastPhoto = useRef(options.photoId);
  const [retryValue, setRetryValue] = useState(0);
  const [state, setState] = useState<PreviewState>({ status: "idle", descriptor: null, url: null, error: null });
  const request = {
    photoId: options.photoId,
    slot: options.slot,
    viewGeneration: options.viewGeneration,
    width: boundedDimension(options.width),
    height: boundedDimension(options.height),
    quality: options.quality ?? "full",
    before: options.before ?? false,
    sequence: latestSequence.current,
  } satisfies PreviewRequest;

  const retry = useCallback(() => {
    setRetryValue((value) => value + 1);
  }, []);

  const acknowledge = useCallback((handle: string) => {
    const bridge = (previewApi as unknown as { acknowledgePreview?: (value: string) => Promise<unknown> | unknown }).acknowledgePreview;
    if (!bridge) return;
    void Promise.resolve(bridge(handle)).catch(() => undefined);
  }, []);

  const onImageLoad = useCallback(() => {
    const descriptor = latestDescriptor.current;
    if (descriptor) acknowledge(descriptor.handle);
  }, [acknowledge]);

  const onImageError = useCallback(() => {
    const descriptor = latestDescriptor.current;
    if (descriptor) acknowledge(descriptor.handle);
    setState((previous) => ({ status: "error", descriptor: previous.descriptor, url: null, error: "Preview could not be decoded" }));
  }, [acknowledge]);

  useEffect(() => {
    if (options.enabled === false || !Number.isSafeInteger(options.photoId) || options.photoId < 0) {
      setState({ status: "idle", descriptor: null, url: null, error: null });
      return;
    }
    const currentSequence = nextSequence();
    latestSequence.current = currentSequence;
    const current: PreviewRequest = { ...request, sequence: currentSequence };
    let cancelled = false;
    const photoChanged = lastPhoto.current !== options.photoId;
    lastPhoto.current = options.photoId;
    setState((previous) => photoChanged
      ? { status: "loading", descriptor: null, url: null, error: null }
      : { status: "loading", descriptor: previous.descriptor, url: previous.url, error: null });
    void requestPreview(current)
      .then((descriptor) => {
        if (cancelled || descriptor.photoId !== current.photoId || descriptor.viewGeneration !== current.viewGeneration || descriptor.sequence !== current.sequence) return;
        const url = `lightcraft-preview://${descriptor.handle}`;
        latestDescriptor.current = descriptor;
        options.onHistogram?.(descriptor.histogram);
        setState({ status: "ready", descriptor, url, error: null });
      })
      .catch((error: unknown) => {
        if (cancelled) return;
        const message = error instanceof Error ? error.message : String(error);
        setState((previous) => ({ status: "error", descriptor: previous.descriptor, url: previous.url, error: message || "Preview unavailable" }));
      });
    return () => {
      cancelled = true;
    };
    // request values are represented by explicit dependencies below; callback identity is caller-owned.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [options.enabled, options.photoId, options.slot, options.viewGeneration, request.width, request.height, request.quality, request.before, retryValue]);

  return { state, request: { ...request, sequence: latestSequence.current }, retry, onImageLoad, onImageError };
}
