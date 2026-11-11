export const SYNC_PROGRESS_EVENT = "sync-progress";
export const IMPORT_PROGRESS_EVENT = "import-progress";
export const BACKUP_FAILED_EVENT = "backup-failed";
export const SOURCE_EVENTS = ["auth-success", "auth-error", "auth-timeout"] as const;
export type SourceEvent = (typeof SOURCE_EVENTS)[number];
