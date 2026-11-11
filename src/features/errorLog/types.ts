import type { ErrorCode } from "@/lib/errors";

export interface ErrorLogEntry {
  id: number;
  occurred_at: string;
  source: string;
  code: ErrorCode | string;
  message: string;
  context: string | null;
}
