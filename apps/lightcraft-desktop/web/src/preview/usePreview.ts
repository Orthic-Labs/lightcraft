import { useCallback, useEffect, useRef, useState } from "react";
import * as previewApi from "../api";
import { convertFileSrc, requestPreview } from "../api";
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
  onImageReady: (descriptor: PreviewDescriptor) => boolean;
  onImageError: (descriptor: PreviewDescriptor) => void;
}

const MAX_DIMENSION = 8192;
const MAX_SEQUENCE = Number.MAX_SAFE_INTEGER;
const MAX_ACKNOWLEDGED_HANDLES = 256;
let nextSequenceValue = 0;

function nextSequence(): number {
  nextSequenceValue = nextSequenceValue >= MAX_SEQUENCE ? 1 : nextSequenceValue + 1;
  return nextSequenceValue;
}

function boundedDimension(value: number): number {
  return Math.max(1, Math.min(MAX_DIMENSION, Math.round(Number.isFinite(value) ? value : 1)));
}

function decodePreviewUrl(url: string): Promise<void> {
  if (typeof Image === "undefined") return Promise.resolve();
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.decoding = "async";
    image.onload = () => resolve();
    image.onerror = () => reject(new Error("Preview could not be decoded"));
    try {
      image.src = url;
      if (typeof image.decode === "function") {
        void image.decode().then(resolve, () => reject(new Error("Preview could not be decoded")));
      }
    } catch {
      reject(new Error("Preview could not be decoded"));
    }
  });
}

/** Request native decoded pixels and ignore any result from an older generation or sequence. */
export function usePreview(options: UsePreviewOptions): UsePreviewResult {
  const latestSequence = useRef(0);
  const activeDescriptor = useRef<PreviewDescriptor | null>(null);
  const presentedPreview = useRef<{ descriptor: PreviewDescriptor; url: string } | null>(null);
  const presentedHandle = useRef<string | null>(null);
  const pendingRequest = useRef<PreviewRequest | null>(null);
  const acknowledgedHandles = useRef<Set<string>>(new Set());
  const acknowledgedOrder = useRef<string[]>([]);
  const pendingRetirement = useRef<{ handle: string; replacementHandle: string } | null>(null);
  const staleRetryKey = useRef("");
  const staleRetryCount = useRef(0);
  const staleRetryTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const staleRecoveryScheduled = useRef(false);
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
    acknowledgedOrder.current.push(handle);
    if (acknowledgedOrder.current.length > MAX_ACKNOWLEDGED_HANDLES) {
      const expired = acknowledgedOrder.current.shift();
      if (expired) acknowledgedHandles.current.delete(expired);
    }
    const bridge = (previewApi as unknown as { acknowledgePreview?: (value: string) => Promise<unknown> | unknown }).acknowledgePreview;
    if (!bridge) return;
    void Promise.resolve().then(() => bridge(handle)).catch(() => undefined);
  }, []);

  const retirePending = useCallback(() => {
    const pending = pendingRetirement.current;
    pendingRetirement.current = null;
    if (pending) acknowledge(pending.handle);
  }, [acknowledge]);

  const onImageReady = useCallback((descriptor: PreviewDescriptor) => {
    if (activeDescriptor.current?.handle !== descriptor.handle || acknowledgedHandles.current.has(descriptor.handle)) return false;
    presentedHandle.current = descriptor.handle;
    presentedPreview.current = { descriptor, url: `lightcraft-preview://${descriptor.handle}` };
    const pending = pendingRetirement.current;
    if (!pending || pending.replacementHandle !== descriptor.handle) return true;
    pendingRetirement.current = null;
    acknowledge(pending.handle);
    return true;
  }, [acknowledge]);

  const onImageError = useCallback((descriptor: PreviewDescriptor) => {
    if (activeDescriptor.current?.handle !== descriptor.handle) {
      acknowledge(descriptor.handle);
      return;
    }
    acknowledge(descriptor.handle);
    activeDescriptor.current = null;
    const retained = presentedPreview.current?.descriptor.photoId === descriptor.photoId && presentedPreview.current.descriptor.handle !== descriptor.handle
      ? presentedPreview.current
      : null;
    // A newer request owns presentation, but released current pixels must not be retained
    // as its fallback while that request is pending or fails.
    if (pendingRequest.current && pendingRequest.current.sequence !== descriptor.sequence) {
      setState({ status: "loading", descriptor: retained?.descriptor ?? null, url: retained?.url ?? null, error: null });
      return;
    }
    setState({ status: "error", descriptor: retained?.descriptor ?? null, url: retained?.url ?? null, error: "Preview could not be decoded" });
  }, [acknowledge]);

  useEffect(() => {
    if (options.enabled === false || !Number.isSafeInteger(options.photoId) || options.photoId < 0) {
      retirePending();
      if (activeDescriptor.current) acknowledge(activeDescriptor.current.handle);
      activeDescriptor.current = null;
      pendingRequest.current = null;
      presentedPreview.current = null;
      presentedHandle.current = null;
      setState({ status: "idle", descriptor: null, url: null, error: null });
      return;
    }
    const currentSequence = nextSequence();
    latestSequence.current = currentSequence;
    const current: PreviewRequest = { ...request, sequence: currentSequence };
    const requestKey = JSON.stringify([current.photoId, current.slot, current.viewGeneration, current.width, current.height, current.quality, current.before]);
    if (staleRetryKey.current !== requestKey) {
      staleRetryKey.current = requestKey;
      staleRetryCount.current = 0;
      staleRecoveryScheduled.current = false;
    }
    let cancelled = false;
    const photoChanged = lastPhoto.current !== options.photoId;
    lastPhoto.current = options.photoId;
    if (photoChanged) {
      retirePending();
      if (activeDescriptor.current) acknowledge(activeDescriptor.current.handle);
      activeDescriptor.current = null;
      presentedPreview.current = null;
      presentedHandle.current = null;
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
        const protocolUrl = `lightcraft-preview://${descriptor.handle}`;
        const browserUrl = convertFileSrc(descriptor.handle, "lightcraft-preview");
        void decodePreviewUrl(browserUrl).then(() => {
          if (cancelled || pendingRequest.current?.sequence !== current.sequence) {
            acknowledge(descriptor.handle);
            return;
          }
          pendingRequest.current = null;
          staleRetryCount.current = 0;
          staleRecoveryScheduled.current = false;
          // Keep current handle live until replacement state commits, so mounted URL stays
          // valid while React swaps decoded pixels.
          const previous = activeDescriptor.current;
          if (previous && previous.handle !== descriptor.handle) {
            if (presentedHandle.current === previous.handle) {
              pendingRetirement.current = { handle: previous.handle, replacementHandle: descriptor.handle };
            } else {
              acknowledge(previous.handle);
              if (pendingRetirement.current) pendingRetirement.current.replacementHandle = descriptor.handle;
            }
          } else if (pendingRetirement.current) {
            // Keep retiring visible pixels only after newest candidate mounts.
            pendingRetirement.current.replacementHandle = descriptor.handle;
          }
          activeDescriptor.current = descriptor;
          options.onHistogram?.(descriptor.histogram);
          setState({ status: "ready", descriptor, url: protocolUrl, error: null });
        }).catch((error: unknown) => {
          if (cancelled || pendingRequest.current?.sequence !== current.sequence) {
            acknowledge(descriptor.handle);
            return;
          }
          pendingRequest.current = null;
          acknowledge(descriptor.handle);
          const retained = presentedPreview.current?.descriptor.photoId === current.photoId ? presentedPreview.current : null;
          const message = error instanceof Error ? error.message : String(error);
          setState({ status: "error", descriptor: retained?.descriptor ?? null, url: retained?.url ?? null, error: message || "Preview could not be decoded" });
        });
      })
      .catch((error: unknown) => {
        if (cancelled || pendingRequest.current?.sequence !== current.sequence) return;
        pendingRequest.current = null;
        const message = error instanceof Error ? error.message : String(error);
        const retained = presentedPreview.current?.descriptor.photoId === current.photoId ? presentedPreview.current : null;
        if (/preview (?:superseded|view is stale|request is stale)/i.test(message)) {
          if (staleRetryCount.current < 2) {
            const retryNumber = staleRetryCount.current;
            staleRetryCount.current += 1;
            if (staleRetryTimer.current !== null) clearTimeout(staleRetryTimer.current);
            setState({ status: "loading", descriptor: retained?.descriptor ?? null, url: retained?.url ?? null, error: null });
            staleRetryTimer.current = setTimeout(() => {
              staleRetryTimer.current = null;
              if (staleRetryKey.current !== requestKey) return;
              setRetryValue((value) => value + 1);
            }, retryNumber === 0 ? 100 : 250);
          } else if (retained) {
            setState({ status: "ready", descriptor: retained.descriptor, url: retained.url, error: null });
          } else {
            // Native can briefly reject every request while its generation advances.
            // Leave first-load UI in loading state until DesktopProvider's snapshot
            // refresh supplies current generation, rather than exposing a false Retry.
            setState({ status: "loading", descriptor: null, url: null, error: null });
            if (!staleRecoveryScheduled.current) {
              staleRecoveryScheduled.current = true;
              staleRetryTimer.current = setTimeout(() => {
                staleRetryTimer.current = null;
                if (staleRetryKey.current !== requestKey) return;
                setRetryValue((value) => value + 1);
              }, 1200);
            }
          }
          return;
        }
        setState({ status: "error", descriptor: retained?.descriptor ?? null, url: retained?.url ?? null, error: message || "Preview unavailable" });
      });
    return () => {
      cancelled = true;
      if (staleRetryTimer.current !== null) {
        clearTimeout(staleRetryTimer.current);
        staleRetryTimer.current = null;
      }
    };
    // request values are represented by explicit dependencies below; callback identity is caller-owned.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [options.enabled, options.photoId, options.slot, options.viewGeneration, request.width, request.height, request.quality, request.before, retryValue, retirePending]);

  useEffect(() => () => {
    retirePending();
    if (activeDescriptor.current) acknowledge(activeDescriptor.current.handle);
    activeDescriptor.current = null;
    presentedPreview.current = null;
    presentedHandle.current = null;
    pendingRequest.current = null;
  }, [acknowledge, retirePending]);

  return { state, request: { ...request, sequence: latestSequence.current }, retry, onImageReady, onImageError };
}
