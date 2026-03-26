import { DURATION_LABELS } from "../types";

export function formatDuration(secs: number): string {
  return DURATION_LABELS[secs] ?? `${secs}s`;
}

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

export function formatTimestamp(iso: string): string {
  const d = new Date(iso);
  return d.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function formatBufferSecs(secs: number): string {
  if (secs < 60) return `${Math.round(secs)}s buffered`;
  return `${Math.round(secs / 60)}m buffered`;
}
