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
  onImageLoad: (descriptor: PreviewDescriptor) => void;
  onImageError: (descriptor: PreviewDescriptor) => void;
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
  const activeDescriptor = useRef<PreviewDescriptor | null>(null);
  const pendingRequest = useRef<PreviewRequest | null>(null);
  const acknowledgedHandles = useRef<Set<string>>(new Set());
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
    if (!handle || acknowledgedHandles.current.has(handle)) return;
    acknowledgedHandles.current.add(handle);
    const bridge = (previewApi as unknown as { acknowledgePreview?: (value: string) => Promise<unknown> | unknown }).acknowledgePreview;
    if (!bridge) return;
    void Promise.resolve(bridge(handle)).catch(() => undefined);
  }, []);

  const onImageLoad = useCallback((descriptor: PreviewDescriptor) => {
    // Handler is bound to descriptor rendered into its <img>. A late event from an old
    // element must never acknowledge a newer handle or alter current state.
    if (activeDescriptor.current?.handle === descriptor.handle) acknowledge(descriptor.handle);
  }, [acknowledge]);

  const onImageError = useCallback((descriptor: PreviewDescriptor) => {
    if (activeDescriptor.current?.handle !== descriptor.handle) {
      acknowledge(descriptor.handle);
      return;
    }
    acknowledge(descriptor.handle);
    // A request for a newer descriptor may already be replacing this image. Its eventual
    // result owns error presentation; an old decode error only retires its handle.
    if (pendingRequest.current && pendingRequest.current.sequence !== descriptor.sequence) return;
    setState({ status: "error", descriptor, url: null, error: "Preview could not be decoded" });
  }, [acknowledge]);

  useEffect(() => {
    if (options.enabled === false || !Number.isSafeInteger(options.photoId) || options.photoId < 0) {
      if (activeDescriptor.current) acknowledge(activeDescriptor.current.handle);
      activeDescriptor.current = null;
      pendingRequest.current = null;
      setState({ status: "idle", descriptor: null, url: null, error: null });
      return;
    }
    const currentSequence = nextSequence();
    latestSequence.current = currentSequence;
    const current: PreviewRequest = { ...request, sequence: currentSequence };
    let cancelled = false;
    const photoChanged = lastPhoto.current !== options.photoId;
    lastPhoto.current = options.photoId;
    if (photoChanged) {
      if (activeDescriptor.current) acknowledge(activeDescriptor.current.handle);
      activeDescriptor.current = null;
    }
    pendingRequest.current = current;
    setState((previous) => photoChanged
      ? { status: "loading", descriptor: null, url: null, error: null }
      : { status: "loading", descriptor: previous.descriptor, url: previous.url, error: null });
    void requestPreview(current)
      .then((descriptor) => {
        const matches = descriptor.photoId === current.photoId && descriptor.slot === current.slot && descriptor.viewGeneration === current.viewGeneration && descriptor.sequence === current.sequence;
        if (cancelled || !matches) {
          // Every returned handle must be retired, including responses which lose a race
          // against unmount, remount, photo swap, or a newer sequence.
          acknowledge(descriptor.handle);
          return;
        }
        pendingRequest.current = null;
        if (activeDescriptor.current && activeDescriptor.current.handle !== descriptor.handle) acknowledge(activeDescriptor.current.handle);
        activeDescriptor.current = descriptor;
        const url = `lightcraft-preview://${descriptor.handle}`;
        options.onHistogram?.(descriptor.histogram);
        setState({ status: "ready", descriptor, url, error: null });
      })
      .catch((error: unknown) => {
        if (cancelled || pendingRequest.current?.sequence !== current.sequence) return;
        pendingRequest.current = null;
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
