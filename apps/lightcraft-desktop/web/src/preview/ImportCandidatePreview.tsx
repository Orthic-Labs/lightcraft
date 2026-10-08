import type { CSSProperties, ImgHTMLAttributes } from "react";
import { useEffect, useState } from "react";
import { acknowledgePreview, convertFileSrc } from "../api";
import type { ImportCandidatePreview as CandidatePreview } from "../types";

export interface ImportCandidatePreviewProps extends Omit<ImgHTMLAttributes<HTMLImageElement>, "src" | "width" | "height"> {
  preview?: CandidatePreview | null;
  width?: number;
  height?: number;
}

/** Candidate thumbnail from import review; handle is leased until row leaves review. */
export function ImportCandidatePreview({ preview, width = 44, height = 36, className, style, alt = "", onError, ...props }: ImportCandidatePreviewProps) {
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    setFailed(false);
    return () => {
      if (preview?.handle) void acknowledgePreview(preview.handle).catch(() => undefined);
    };
  }, [preview?.handle]);
  const frameStyle: CSSProperties = { position: "relative", display: "grid", placeItems: "center", width, height, minWidth: 1, minHeight: 1, overflow: "hidden", ...style };
  if (!preview || failed) return <span className={className} style={{ ...frameStyle, color: "#5f7896", background: "#dce8f5", fontSize: 10, fontWeight: 600 }} aria-label={alt || "Preview unavailable"}>IMG</span>;
  return <span className="lc-import-candidate-preview" style={frameStyle}>
    <img {...props} className={className} src={convertFileSrc(preview.handle, "lightcraft-preview")} width={width} height={height} alt={alt || "Photo preview"} decoding="async" draggable={false} onError={(event) => { setFailed(true); void acknowledgePreview(preview.handle).catch(() => undefined); onError?.(event); }} style={{ width: "100%", height: "100%", objectFit: "cover" }} />
    <span aria-hidden="true" style={{ position: "absolute", right: 2, bottom: 2, padding: "1px 3px", borderRadius: 3, background: "rgba(10,14,20,.68)", color: "#fff", fontSize: 8, lineHeight: 1.2 }}>PREVIEW</span>
  </span>;
}

export default ImportCandidatePreview;
