export interface SourceInfo {
  id: string;
  label: string;
  kind: "oauth" | "file";
  domains: string[];
  connected: boolean;
  credentials_configured: boolean;
  last_sync_at: string | null;
  redirect_uri: string | null;
  developer_url: string | null;
  reauth: ReauthInfo | null;
  extensions: string[];
}

export interface ReauthInfo {
  status: "soon" | "expired";
  expires_at: number;
}

export interface SourceOutcome {
  source: string;
  label: string;
  count: number;
  errors: string[];
  failed: boolean;
}

export interface SyncProgress {
  source: string;
  etape: number;
  total: number;
  libelle: string;
}

export interface InvalidatedSource {
  source: string;
  label: string;
  tokens: boolean;
  credentials: boolean;
}

export interface InvalidatedSecrets {
  sources: InvalidatedSource[];
}

export interface MetricPriority {
  slug: string;
  label: string;
  sources: string[];
  custom: boolean;
}

export interface ImportTally {
  observations: number;
  observations_ignored: number;
  blood_pressure: number;
  sleep: number;
  activity: number;
  workouts: number;
  devices: Record<string, number>;
}

export interface ImportReport {
  source: string;
  cancelled: boolean;
  tally: ImportTally;
  skipped: { reason: string; count: number }[];
  duration_ms: number;
}

export interface ImportProgress {
  source: string;
  percent: number;
  records: number;
}

export interface CsvColumnMap {
  column: string;
  metric: string;
  unit: string;
}

export interface CsvConfig {
  delimiter: string;
  decimal: string;
  skip_rows: number;
  date_column: string;
  time_column: string | null;
  date_format: string;
  columns: CsvColumnMap[];
}

export interface CsvDateFormat {
  id: string;
  label: string;
  example: string;
}

export interface CsvTarget {
  metric: string;
  label: string;
  units: { id: string; label: string }[];
}

export interface CsvSavedMapping {
  id: number;
  name: string;
  config: CsvConfig;
}

export interface CsvInspection {
  header: string[];
  sample: string[][];
  row_count: number;
  signature: string;
  suggestion: CsvConfig;
  preset: string | null;
  formats: CsvDateFormat[];
  saved: CsvSavedMapping[];
  unreadable_mappings: number;
  targets: CsvTarget[];
}
