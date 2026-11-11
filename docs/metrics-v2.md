# Stack Metriques V2 — Architecture

Reference de la stack unifiee introduite en migrations 005-007, finalisee par la page Explorer et
arretee a une **source unique par metrique** en R28 (2026-10-25). Spec :
`docs/specs/2026-10-25-couche-metier-et-metriques-v2-design.md`, note de travail hors du depot public.

## Probleme resolu

Avant V2, chaque type de mesure avait sa propre table (`body_measures`, `manual_measurements`,
`vitals`). La V2 a ajoute un catalogue `metric_definitions` et une table `metrics` pour lire
n'importe quelle mesure de la meme facon (Explorer, alertes). La migration n'est jamais allee
jusqu'au bout : les pages lisent toujours les tables historiques, et la synchro ecrivait aussi dans
`metrics`. R28 a choisi de **ne pas finir** la migration des pages et de supprimer le recouvrement.

## Schema (2 tables)

### `metric_definitions` (catalogue)

| Colonne | Type | Description |
|---|---|---|
| `id` | INTEGER PK | Identifiant interne |
| `slug` | TEXT UNIQUE | Identifiant stable (`weight`, `fat_ratio`, `biceps_left`) |
| `label` | TEXT | Nom affichable francais |
| `unit` | TEXT | `kg`, `%`, `mmHg`, `bpm`, `cm`, `s`, `kcal`... |
| `category` | TEXT | `body_composition` / `circumference` / `body_scan` / `vital_bp` / `vital_sleep` / `vital_activity` / `derived` / `custom` |
| `withings_type` | INTEGER NULL | Type ID Withings (1=poids, 9=diastolique, 10=systolique, 11=pouls...) |
| `is_builtin` | INTEGER (0/1) | 1 = predefinie (seed), 0 = ajoutee par utilisateur |
| `sort_order` | INTEGER | Ordre d'affichage |
| `decimals` | INTEGER | Precision (0-3) |
| `min_value`, `max_value` | REAL NULL | Bornes attendues (validation de la synchro, zoom) |

Seed de la migration 005 : **42 entrees**. Les sources en ajoutent d'autres par `Source::metric_definitions` (`hrv`, `readiness_score`, `recovery_score`, `strain`, etc.), semees au demarrage par `seed_source_definitions` (Rust, `db/seed.rs`) ; leur categorie les range en `Metrics` ou dans une table historique selon `classify`.

### `metrics` (mesures)

| Colonne | Type | Description |
|---|---|---|
| `id` | INTEGER PK | |
| `definition_id` | INTEGER FK -> metric_definitions | |
| `measured_at` | TEXT | ISO 8601 UTC |
| `value` | REAL | |
| `source` | TEXT FK -> sources | n'importe quelle ligne de `sources` (`withings`, `oura`, `csv`, `manual`...) |
| `external_id` | TEXT NULL | ID chez la source, pour l'idempotence (`UNIQUE` avec `definition_id`, `measured_at`, `source`) |
| `notes` | TEXT NULL | |
| `device` | TEXT NULL | appareil ou app d'origine d'un import |

Index : `(definition_id, measured_at)`, `external_id`.

`metric_targets` et `derived_metrics` ont ete retirees par la migration 018 : aucun ecran, aucune
commande ni aucune synchro n'y lisait ou n'y ecrivait. Les objectifs vivent dans `goals`.

## Une source par metrique

Les anciennes tables (`body_measures`, `manual_measurements`, `vitals`, `measurement_types`) sont
des **vues** sur des tables `*_legacy` depuis la migration 005 ; les ecritures visent les `*_legacy`.
`metric_repository::source_of(pool, slug)` rend la source de chaque metrique :

| `MetricSource` | Metriques | Lecture | Ecriture de la synchro |
|---|---|---|---|
| `Legacy(table)` | `body_composition` (`body_measures_legacy`), `circumference` et `custom` (`manual_measurements_legacy`), `body_scan` (`vitals_legacy`), `systolic` et `diastolic` (`blood_pressure`), `sleep_score` (`sleep_records`), `steps` (`activity`) | la table, seule | la table, seule |
| `Split(BloodPressure)` | `heart_rate` | `blood_pressure.heart_rate` plus les lignes de `metrics` sans jumelle au meme instant (jointure d'exclusion SQL) | le pouls d'un groupe qui forme une mesure de tension reste dans `blood_pressure` ; le pouls d'un groupe sans tension va dans `metrics` |
| `Metrics` | les six `derived` (jamais ecrites) | `metrics` | aucune |
| aucune | slug inconnu, ou slug de `vital_bp` hors des trois connus | aucune ligne | sans objet |

Pourquoi `heart_rate` fait exception : dans la copie de dev (2 060 lignes dans `metrics`), 117
pouls n'existent que dans `metrics` (groupes Withings sans tension) et 120 pouls n'existent que
dans `blood_pressure` (mesures manuelles, anciens imports). Une source unique aurait retire des
points visibles dans l'Explorer. C'est la seule metrique dans ce cas.

Regles :

- Une lecture ne fusionne jamais deux tables en Rust : plus de deduplication en O(n·m). Deux
  lignes d'une meme table au meme instant (sources differentes) ne font qu'un point.
- Une metrique nouvelle n'a qu'un endroit a toucher : une ligne dans `metric_repository::classify`.
  Le test `every_metric_has_exactly_one_source` parcourt `metric_definitions` et echoue pour un slug
  sans regle.
- Les lignes de `metrics` ecrites avant R28 pour une metrique `Legacy` restent en place et ne sont
  plus lues. Inventaire de la copie de dev (2026-10-25) : 1 943 jumelles exactes d'une ligne
  historique (poids et composition 938, body scan 618, tension 285 dont 95 pouls, sommeil 102) et
  117 sans jumelle (toutes `heart_rate`). Aucune suppression : une base reelle ne se lit pas ici, et
  le gain de place est nul.
- La temperature (migration 013) n'est pas propre a `metrics` : elle est dans `vitals_legacy`
  (`vital_type = 'temperature'`).

## Fonction cle : `read_metric_series`

Implementation : `src-tauri/src/metric_repository.rs`.

```
read_metric_series(pool, slug, from, to) -> Vec<MetricPoint>
```

1. `source_of` resout `slug` en `(definition_id, MetricSource)`.
2. `Legacy` : une requete sur la table historique, triee par instant, un point par instant.
3. `Metrics` : une requete sur `metrics`.
4. `Split` : la requete historique, plus la requete `metrics` filtree par `NOT EXISTS` sur
   `blood_pressure`, puis tri.

`read_metric_latest` suit la meme regle (`Split` : le plus recent des deux, la table historique
l'emporte a egalite). Les alertes passent par lui.

## Choix de la source (R33)

Plusieurs sources peuvent porter la meme metrique (une balance Withings et un CSV importe). Rien n'est fusionne a l'ecriture : chaque ligne garde sa `source` et son `external_id`. `services::metrics::select_series(slug, from, to)` choisit, pour chaque jour, **une seule source** : d'abord l'ordre de l'utilisateur (`metric_source_priority`, `set_priority` et `get_priority`), puis la source dont le point est le plus recent sur la periode, puis l'ordre du registre (`sources.sort_order`). Une source absente un jour laisse le jour a la suivante : elle ne vide pas l'historique d'une autre. Les cumuls journaliers (pas) ne s'additionnent donc jamais entre sources. `select_latest` et `select_latest_before` suivent le meme choix. Series, dernier point, tendances du tableau de bord, quatre de ses six cartes (poids, tension, sommeil, activite : la ligne montree est celle de la source retenue pour le dernier jour de la mesure, `Resolved::latest` puis `select_at`), alertes, records (poids, pas, score de sommeil) et insights du poids et des pas passent par cette fonction ; les cartes entrainement et ECG n'ont pas de mesure a classer et gardent la derniere ligne ; `metric_repository::read_metric_series` reste la lecture brute de toutes les sources. `write_observations` ecrit des `Observation` avec un `external_id` (celui de la source, sinon `metrique@instant`) et `ON CONFLICT DO NOTHING` : un reimport n'ajoute rien. Les pages qui listent les lignes d'une table (nuits, entrainements, tensions, ECG) lisent toutes les sources (un historique n'est pas une courbe) et nomment la source de chaque ligne dans une colonne cherchable (R34). `select_secondary` rend les points que la courbe n'a pas retenus (`get_metric_secondary_points`), l'explorateur les dessine en points discrets ; `keep_preferred` applique le meme choix par jour aux detecteurs du sommeil (classement du score de sommeil), de la tension (systolique) et de l'ECG (pouls). `get_metric_priorities` liste les mesures qui ont au moins deux sources avec leur ordre, `set_metric_priority` l'enregistre (liste vide : retour a l'ordre automatique).

## Commandes Tauri exposees

| Commande | Signature TS | Usage |
|---|---|---|
| `get_metric_definitions` | `(category?: string) -> MetricDefinition[]` | Catalogue (filtrable) |
| `get_metric_series` | `(slug, from, to) -> MetricPoint[]` | Serie temporelle |
| `get_metric_latest` | `(slug) -> MetricPoint \| null` | Derniere mesure |
| `get_metric_priorities` | `() -> MetricPriority[]` | Mesures a au moins deux sources, avec leur ordre |
| `set_metric_priority` | `(slug, sources: string[]) -> void` | Enregistre l'ordre (liste vide : ordre automatique) |
| `get_metric_secondary_points` | `(slug, from, to) -> MetricPoint[]` | Points que la courbe n'a pas retenus |
| `get_dashboard_cards` | `(from, to) -> DashboardCards` | Six derniers releves et quatre tendances du tableau de bord, en une lecture (N-13) |

## Hooks frontend

`src/features/metrics/hooks.ts` : `useMetricDefinitions`, `useMetricSeries`, `useMetricLatest`.
`src/features/dashboard/hooks.ts` : `useDashboardCards(from, to)` (cle `["dashboardCards", from, to]`,
invalidee par toute ecriture reussie et par la synchro).

## Composants frontend

| Fichier | Role |
|---|---|
| `src/pages/Explorer.tsx` | Page dediee : selection metrique + visualisation |
| `src/components/charts/GenericMetricChart.tsx` | AreaChart Recharts generique |
| `src/components/shared/MetricSelector.tsx` | Combobox HTML native groupee par categorie |

## Ajouter une nouvelle metrique

1. Une definition dans `metric_definitions` : une migration versionnee (fichier `NNN_*.sql` plus une ligne dans `MIGRATIONS`), ou, pour une metrique qu'apporte une source, `Source::metric_definitions` (semee en Rust).
2. Dire sa source : une ligne dans `metric_repository::classify` (table historique existante, ou
   `Metrics` si aucune table ne la porte). Le test de couverture des slugs l'exige.
3. Si elle vient de Withings : mapper le `withings_type` (`sources/withings/mapping/measures.rs`) et ecrire
   dans la source choisie (`sources/withings/sync/measures/`).
4. Optionnel : une couleur dans `src/lib/constants.ts` -> `METRIC_COLORS`.

Une metrique personnalisee de l'utilisateur est creee depuis **Mensurations -> Gerer** ; elle vit
dans `manual_measurements_legacy` (categorie `custom`).

## Tests

`src-tauri/tests/integration/metrics_source.rs` : une source par slug, la source suit l'inventaire,
une ligne `metrics` perimee n'est pas lue pour une metrique historique, `heart_rate` garde ses deux
sources et compte une jumelle une fois. `metric_dedup.rs` : plages UTC. `schema.rs` : tables mortes
absentes, migration 018. `architecture.rs` : aucun import de `commands/` hors de `commands/`.

```
cargo test --manifest-path src-tauri/Cargo.toml --test integration metrics_source
```

## Limites

- Les pages legacy lisent toujours leurs tables historiques ; la fin de la migration (option A de la
  spec) est ecartee faute de benefice visible.
- Le jour d'une plage est le jour UTC (N-14 accepte) : une mesure entre 0 h et 2 h heure de Paris
  tombe sur la veille. Voir `utils/dates.rs` et la spec, section 7.
- Aucune formule derivee n'est calculee : les six metriques `derived` n'ont pas de donnees.
