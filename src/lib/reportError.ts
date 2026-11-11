import { logErrorEntry } from "@/features/errorLog/api";
import { toast } from "sonner";
import { formatError, type AppError } from "./errors";
import { logError } from "./logger";

export type ErrorSource = "sync" | "mutation" | "boundary" | "manual" | "query";

interface ReportOpts {
  source: ErrorSource;
  context?: string;
  prefix?: string;
  toast?: boolean;
}

export function reportError(error: unknown, opts: ReportOpts): AppError {
  const formatted = formatError(error);
  if (opts.toast !== false) {
    toast.error(`${opts.prefix ?? ""}${formatted.userMessage}`, { duration: 10_000 });
  }
  logErrorEntry({
    source: opts.source,
    code: formatted.code,
    message: formatted.userMessage,
    context: opts.context ?? null,
  }).catch((e) => logError("log_error failed:", e));
  return formatted;
}

export function notifyError(message: string, id?: string): void {
  toast.error(message, { duration: 10_000, id });
}
