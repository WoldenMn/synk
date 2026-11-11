import type { CsvConfig, CsvTarget } from "./types";

export function columnMetric(config: CsvConfig, column: string): string {
  return config.columns.find((c) => c.column === column)?.metric ?? "";
}

export function columnUnit(config: CsvConfig, column: string): string {
  return config.columns.find((c) => c.column === column)?.unit ?? "";
}

export function setColumnMetric(
  config: CsvConfig,
  column: string,
  metric: string,
  targets: CsvTarget[]
): CsvConfig {
  const others = config.columns.filter((c) => c.column !== column);
  const target = targets.find((t) => t.metric === metric);
  if (!target) return { ...config, columns: others };
  const previous = config.columns.find((c) => c.column === column);
  const keep = previous && target.units.some((u) => u.id === previous.unit);
  const unit = keep ? previous.unit : (target.units[0]?.id ?? "");
  return { ...config, columns: [...others, { column, metric, unit }] };
}

export function setColumnUnit(config: CsvConfig, column: string, unit: string): CsvConfig {
  return {
    ...config,
    columns: config.columns.map((c) => (c.column === column ? { ...c, unit } : c)),
  };
}

export function pressureIssue(config: CsvConfig): string | null {
  const count = (metric: string) => config.columns.filter((c) => c.metric === metric).length;
  if (count("systolic") !== count("diastolic") || count("systolic") > 1) {
    return "La tension demande une colonne systolique et une colonne diastolique.";
  }
  return null;
}

export function importIssue(config: CsvConfig): string | null {
  if (!config.date_column) return "Choisissez la colonne qui porte la date.";
  if (config.columns.length === 0) return "Associez au moins une colonne à une mesure.";
  return pressureIssue(config);
}
