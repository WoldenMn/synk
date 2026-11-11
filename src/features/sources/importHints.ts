export const IMPORT_HINTS: Record<string, string> = {
  apple_health:
    "Dans l'app Santé de l'iPhone : votre photo de profil, « Exporter toutes les données de santé », puis copiez export.zip sur ce PC.",
  fit: "Fichiers .fit : exportés depuis Garmin Connect (activité, « Exporter l'original »), Coros, Suunto, ou copiés depuis le dossier Activity de la montre branchée en USB.",
  trace:
    "Fichiers .gpx ou .tcx : exportés par Strava, Komoot, Garmin Connect ou l'appli de l'appareil. Seul le résumé de la séance est gardé, pas le tracé.",
  csv: "Fichier .csv d'une balance, d'un tensiomètre ou d'un lecteur de glycémie : un assistant propose la correspondance des colonnes.",
};

export function importHint(id: string): string | null {
  return IMPORT_HINTS[id] ?? null;
}
