export type BootErrorPayload =
  | { kind: "AppDataDir"; message: string }
  | { kind: "DbBusyAfterRetry"; attempts: number }
  | { kind: "DbLocked"; message: string }
  | { kind: "MigrationFailed"; version: number; message: string }
  | { kind: "CryptoUnavailable"; message: string }
  | { kind: "CryptoMigrationFailed"; message: string }
  | { kind: "BackupFailed"; message: string }
  | { kind: "DevOnRealData"; identifier: string };

export type IntegrityWarningPayload = { Details: string };

export type BootSnapshot =
  | { status: "loading" }
  | { status: "ready"; integrity_warning: IntegrityWarningPayload | null }
  | { status: "failed"; error: BootErrorPayload };

export interface RestoreReport {
  restored_from: string;
  broken_kept_at: string;
  skipped: string[];
}

export type AppReadyState =
  | { status: "loading" }
  | { status: "stuck"; lastSnapshot: BootSnapshot | null; lastError: string | null }
  | { status: "ready"; integrityWarning?: IntegrityWarningPayload }
  | { status: "error"; error: BootErrorPayload };

export const STUCK_TIMEOUT_MS = 12000;
