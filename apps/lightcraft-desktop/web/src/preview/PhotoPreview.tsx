import type { CSSProperties, ImgHTMLAttributes } from "react";
import { useMemo } from "react";
import { convertFileSrc } from "../api";
import type { PreviewQuality } from "../desktop/types";
import { usePreview } from "./usePreview";

export interface PhotoPreviewProps extends Omit<ImgHTMLAttributes<HTMLImageElement>, "src" | "width" | "height" | "onError"> {
  photoId: number;
  slot: string;
  width: number;
  height: number;
  viewGeneration?: number;
  quality?: PreviewQuality;
  before?: boolean;
  onHistogram?: (histogram: unknown) => void;
}

/** Real native preview image. Loading/error labels remain available to keyboard and screen readers. */
export function PhotoPreview({ photoId, slot, width, height, viewGeneration = 0, quality = "full", before = false, className, style, onHistogram, alt = "", ...imageProps }: PhotoPreviewProps) {
  const preview = usePreview({ photoId, slot, width, height, viewGeneration, quality, before, onHistogram });
  const url = useMemo(() => {
    if (!preview.state.url || preview.state.descriptor?.photoId !== photoId) return null;
    return convertFileSrc(preview.state.url.replace("lightcraft-preview://", ""), "lightcraft-preview");
  }, [photoId, preview.state.descriptor?.photoId, preview.state.url]);
  const frameStyle: CSSProperties = { width: "100%", height: "100%", minWidth: 1, minHeight: 1, ...style };
  const descriptor = preview.state.descriptor;
  // Bind DOM events to descriptor which produced this URL. Hook-level refs reject late
  // events from an old image after a request/photo swap.
  const onLoad = descriptor ? () => preview.onImageLoad(descriptor) : undefined;
  const onError = descriptor ? () => preview.onImageError(descriptor) : undefined;
  const image = url
    ? <img {...imageProps} {...(alt === "" ? { alt: "Photo preview" } : { alt })} className={className} style={frameStyle} src={url} width={width} height={height} decoding="async" draggable={false} onLoad={onLoad} onError={onError} />
    : null;

  if (preview.state.status === "error") {
    if (image) {
      return (
        <div className={className} style={{ ...frameStyle, position: "relative" }} role="group" aria-label={`${alt || "Photo"}: ${preview.state.error}`}>
          {image}
          <div role="alert" style={{ position: "absolute", inset: "auto 8px 8px", display: "flex", alignItems: "center", justifyContent: "space-between", gap: 8, padding: "6px 8px", background: "rgba(0, 0, 0, 0.72)", color: "white" }}>
            <span data-preview-state="error">{preview.state.error}</span>
            <button type="button" onClick={preview.retry}>Retry</button>
          </div>
        </div>
      );
    }
    return (
      <div className={className} style={{ ...frameStyle, display: "grid", placeItems: "center" }} role="img" aria-label={`${alt || "Photo"}: ${preview.state.error}`}>
        <span data-preview-state="error">Preview unavailable</span>
        <button type="button" onClick={preview.retry}>Retry</button>
      </div>
    );
  }
  if (!url) {
    return (
      <div className={className} style={{ ...frameStyle, display: "grid", placeItems: "center" }} role="status" aria-label="Loading preview">
        <span data-preview-state="loading">Loading preview…</span>
      </div>
    );
  }
  return image;
}

export default PhotoPreview;
