export const SOURCE_LABELS: Record<string, string> = {
  withings: "Withings",
  manual: "Saisie manuelle",
  derived: "Calculé",
  import: "Import",
  oura: "Oura",
  polar: "Polar",
  whoop: "Whoop",
  google_health: "Google Health",
  apple_health: "Apple Santé",
  fit: "Fichiers FIT",
  csv: "Fichiers CSV",
  trace: "Traces GPX et TCX",
};

export const DOMAIN_LABELS: Record<string, string> = {
  body: "Composition corporelle",
  blood_pressure: "Tension",
  sleep: "Sommeil",
  activity: "Activité",
  workouts: "Entraînements",
  heart: "Cœur",
  vitals: "Constantes",
  glucose: "Glycémie",
  devices: "Appareils",
};

export function domainLabel(id: string): string {
  return DOMAIN_LABELS[id] ?? id;
}

export const ANALYSES_ID = "analyses";

export function sourceLabel(id: string): string {
  if (id === ANALYSES_ID) return "Analyses";
  return SOURCE_LABELS[id] ?? id;
}
