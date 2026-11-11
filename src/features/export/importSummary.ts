import type { ImportPreview, TableImport } from "./api";

const IMPORT_LABELS: Record<string, [string, string]> = {
  body_measures: ["pesée", "pesées"],
  blood_pressure: ["mesure de tension", "mesures de tension"],
  sleep_records: ["nuit", "nuits"],
  activity: ["journée d'activité", "journées d'activité"],
  workouts: ["entraînement", "entraînements"],
  heart_records: ["ECG", "ECG"],
  vitals: ["mesure vitale", "mesures vitales"],
  measurement_types: ["type de mensuration", "types de mensuration"],
  manual_measurements: ["mensuration", "mensurations"],
  metric_definitions: ["définition de métrique", "définitions de métriques"],
  metrics: ["métrique", "métriques"],
  metric_source_priority: ["priorité de source", "priorités de source"],
  goals: ["objectif", "objectifs"],
  user_profile: ["profil", "profil"],
  medications: ["médicament", "médicaments"],
  medication_logs: ["prise", "prises"],
  lab_results: ["analyse", "analyses"],
  journal_entries: ["note du journal", "notes du journal"],
  health_alerts: ["alerte", "alertes"],
  alert_events: ["déclenchement d'alerte", "déclenchements d'alerte"],
  clothing_sizes: ["taille", "tailles"],
};

export function describePreview(preview: ImportPreview[]): string {
  const lines = preview
    .filter((p) => p.rows > 0)
    .map((p) => {
      const [one, many] = IMPORT_LABELS[p.key] ?? [p.key, p.key];
      return `${p.rows} ${p.rows > 1 ? many : one}`;
    });
  return `${lines.join(", ")}. Les lignes déjà présentes sont ignorées, rien n'est écrasé ; le profil n'est complété que là où il est vide.`;
}

export function summarizeImport(tables: TableImport[]): string {
  const sum = (f: (t: TableImport) => number) => tables.reduce((n, t) => n + f(t), 0);
  const added = sum((t) => t.inserted);
  const present = sum((t) => t.ignored);
  const completed = sum((t) => t.updated);
  return `Import terminé : ${added} ${added > 1 ? "ajoutées" : "ajoutée"}, ${present} ${
    present > 1 ? "déjà présentes" : "déjà présente"
  }, ${completed} ${completed > 1 ? "complétées" : "complétée"}`;
}
