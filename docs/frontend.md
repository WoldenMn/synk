# Frontend

L'interface React 19 de Synk (TypeScript strict, Tailwind 4, TanStack Query 5, Zustand 5, Recharts 3, react-router 7), décrite d'après `src/`. Le lien avec Rust passe par les commandes de [commandes.md](commandes.md) ; les règles et les raisons sont dans [decisions.md](decisions.md).

## 1. Organisation

| Dossier | Contenu |
|---|---|
| `pages/` | 19 pages routées dans `App.tsx`, toutes chargées à la demande (`lazy`). Une page complexe a un dossier voisin de sections (`pages/Dashboard/sections/`, `pages/profile/sections/`…). |
| `features/<domaine>/` | Un domaine métier : `api.ts` (les `invoke`, seul endroit autorisé), `types.ts`, `hooks.ts` (React Query), `components/`. Domaines : activity, alerts, audit, backups, bloodPressure, bodyMeasures, bodyProfile, clothingSizes, dashboard, devices, errorLog, export, goals, heart, insights, journal, labs, manualMeasurements, medications, metrics, profile, profileCard, settings, sleep, sources, syncLog, workouts. |
| `components/ui/` | Primitifs de style shadcn : `Button`, `Card`, `Dialog`, `Input`, `Select`, `Textarea`, `Label`, `Badge`, `Separator`, `Skeleton`, `Tooltip`, `confirm-dialog`. |
| `components/shared/` | Composants communs (§ 3), assistant de premier lancement, écrans de démarrage. |
| `components/charts/` | Graphiques Recharts et `TimeRangeSelector`. |
| `components/forms/` | `createManualForm` (fabrique de formulaires de saisie) et `GoalForm`. |
| `components/layout/` | `AppLayout`, `Header`, `Sidebar`, `QuickAddButton`, `RouteErrorBoundary`. |
| `hooks/` | `createCrudHook`, `useSync`, `useAppReady` (+ `appReady/`), `useTimeRange`, `useKeyboardShortcuts`, `useLayoutShortcuts`, `useRouteAnnouncement`, `useEntranceOnce`. |
| `lib/` | Formatage, dates, erreurs, constantes de navigation, aides de graphiques (`lib/charts/`), stockage sûr, raccourcis. `lib` ne dépend d'aucune couche supérieure (règle ESLint). |
| `stores/` | `uiStore` (thème, couleur d'accent, barre latérale repliée, pages masquées, système de tailles, ajout rapide) et `syncStore` (synchro en cours, progression, intervalle). Les choix de l'utilisateur sont gardés dans `localStorage` par `lib/storage.ts`, jamais en base. |
| `styles/` | `globals.css` : thème et Tailwind 4 (section Theme et Tailwind 4). |

L'alias `@/` pointe vers `src/`. Un `index.ts` (barrel) existe dans `components/shared`, `components/charts` et `hooks`. Le barrel de `shared` ne ré-exporte jamais l'assistant de premier lancement ni les formulaires de saisie rapide, qui restent chargés à la demande.

## 2. Pages et navigation

La barre latérale est construite par `NAV_SECTIONS` et `NAV_SECONDARY` (`lib/constants.ts`) ; `PAGE_TITLES` en est dérivé.

| Section | Pages (route) |
|---|---|
| Sans titre | Tableau de bord (`/`) |
| Corps | Composition (`/body`), Tension (`/blood-pressure`), Mensurations (`/measurements`) |
| Activité | Activité (`/activity`), Entraînements (`/workouts`), Sommeil (`/sleep`), Cœur / ECG (`/heart`) |
| Suivi | Médicaments (`/medications`), Analyses labo (`/labs`), Journal (`/journal`) |
| Analyse | Profil (`/profile`), Corrélations (`/analyses`), Constats (`/insights`), Explorer (`/explorer`), Alertes (`/alerts`), Synchronisations (`/timeline`) |
| Secondaire | Exporter (`/export`), Paramètres (`/settings`) |

Le tableau de bord, Exporter et Paramètres ne peuvent pas être masqués ; les autres pages le sont depuis Paramètres > Apparence (`hiddenRoutes`). Une page neuve n'entre qu'à la demande explicite de l'utilisateur.

## 3. Composants partagés

| Composant | Props principales | Rôle |
|---|---|---|
| `StatCard` | `label`, `value`, `sub?`, `icon?`, `glow?`, `delay?` | Carte d'indicateur. |
| `DataTable<T>` | `label`, `columns`, `data`, `emptyMessage`, `searchable?`, `pageSize?`, `canDelete?`, `onDelete?` avec `rowLabel` | Tableau générique (`T` porte un `id`) : tri, recherche, pagination. `onDelete` ajoute une corbeille, qui exige `rowLabel` (nom accessible). |
| `EmptyState` | `icon`, `title`, `description`, `action?`, `compact?` | État vide. `PeriodEmptyState` distingue une période vide d'un historique vide. |
| `LoadingSkeleton` | `rows?`, `height?` | Squelettes ; `ChartSkeleton`, `StatCardGridSkeleton`, `TableSkeleton` pour les formes courantes. |
| `PageHeader` | `timeRange`, `onTimeRangeChange`, `onAdd?`, `addLabel?`, `children?` | En-tête de page : plage de dates et bouton d'ajout. |
| `MiniSparkline`, `MetricSelector` | | Mini-courbe en ligne ; liste déroulante des métriques groupée par catégorie. |
| `OnboardingWizard` | | Assistant de premier lancement : Bienvenue, Sources, Profil, Objectifs. L'étape Sources liste le registre et se passe sans brancher aucune source (`SourceConnect` porte la connexion d'une marque). |
| `BootErrorScreen`, `BootStuckScreen`, `IntegrityBanner` | | Écrans et bandeau du démarrage (§ 6). |

`Column<T>` : `key`, `label`, `render?`, `searchText?`. Sans `render`, la valeur brute s'affiche (`—` si vide). La recherche lit `searchText(row)` (le texte tel qu'il s'affiche : date formatée, valeur avec son unité), sinon la valeur brute, et ne lit pas une colonne rendue sans `searchText`. Les cinq historiques de mesures n'offrent la suppression que des lignes `source === "manual"` (`canDelete`).

## 4. Pattern d'une page de données

Body, BloodPressure, Sleep, Activity, Workouts, Heart et Measurements suivent le même squelette : `useTimeRange()` donne la plage ; un hook de `features/` lit la période ; `PageHeader` ; une grille de `StatCard` ; un graphique dans une `Card` ; un `DataTable` ; un formulaire en `Dialog`. Les indicateurs se calculent avec `avg`, `sum` et `latest` (`lib/stats.ts`), les valeurs s'affichent par `lib/formatters.ts`.

```tsx
const { timeRange, setTimeRange, from, to } = useTimeRange();
const { data, isLoading } = useSleepData(from, to);
if (isLoading) return <LoadingSkeleton rows={8} />;
if (!data?.length) return <EmptyState icon={Moon} title="..." description="..." />;
```

`useTimeRange` rend aussi `compareFrom`, `compareTo` et `setCompareRange` (comparaison de deux périodes, tableau de bord). La borne haute d'une période est la fin du jour, pour garder une clé de requête stable (P43).

## 5. Données : React Query

`createQueryClient` (`lib/queryClient.ts`) fixe `staleTime` 5 minutes, `gcTime` 30 minutes, pas de relecture au retour du focus, une relance. Une erreur de requête n'affiche un toast que lorsqu'une donnée est déjà à l'écran ou pour les requêtes du bandeau (un toast par clé et par minute) ; sans donnée, l'erreur remonte à la page par `throwOnError` et la `RouteErrorBoundary`, et toute écriture réussie invalide `dashboardCards`.

**Un domaine CRUD** tire ses hooks de `createCrudHook` (`hooks/createCrudHook.ts`) : `useList(from, to)` (clé `[domain, from, to]`), `useLatest()` (clé `[domain, "latest"]`), `useAdd()` et `useDelete()` (toast de succès, invalidation de `[domain]`, erreur par `reportError`). Cinq domaines l'utilisent : bodyMeasures (`useBodyMeasures`, `useLatestBodyMeasure`, `useAddBodyMeasure`…), bloodPressure, sleep (`useSleepData`, `useLatestSleep`…), activity, workouts. Les autres domaines écrivent leurs hooks à la main dans `features/<domaine>/hooks.ts` : goals, mensurations manuelles et leurs statistiques, alertes, médicaments, analyses, journal, constats, métriques (`useMetricDefinitions`, `useMetricSeries`, `useMetricLatest`), synchronisations, appareils, sauvegardes.

**La synchro** (`hooks/useSync.ts`) appelle `sync_all_sources` (ou `sync_source`), écoute `sync-progress` pour afficher « Marque : étape (n/total) », puis invalide la liste `SYNC_INVALIDATED_KEYS` (jamais une invalidation globale) et annonce le résultat (succès, synchro partielle avec la liste des erreurs, ou « aucune source connectée »). `AppLayout` la lance au démarrage si une source est connectée et que la dernière synchro réussie date de plus que l'intervalle choisi, ainsi que sur `<source>-auth-success`.

## 6. Démarrage et récupération

`useAppReady` rend un état `loading`, `stuck`, `ready` (avec un éventuel avertissement d'intégrité) ou `error`. Trois sous-hooks (`hooks/appReady/`) : `useBootListeners` écoute `app-ready`, `app-error` et `app-integrity-warning` ; `useBootSnapshot` relit `get_boot_state` après avoir attaché les écouteurs (un événement émis trop tôt n'est donc pas perdu) ; `useBootWatchdog` passe à `stuck` après 12 secondes. `App.tsx` écoute `app-ready` pour `prefetchDashboard` (la requête `dashboardCards` de la plage de 30 jours, dont la clé doit être exactement celle de `useDashboardCards`).

`BootErrorScreen` choisit titre, texte et boutons selon le `kind` (`bootErrorCopy.ts`) :

| `kind` | Boutons |
|---|---|
| `DbBusyAfterRetry`, `DbLocked` | Réessayer, Ouvrir le dossier des données |
| `MigrationFailed` | Restaurer la sauvegarde, Réessayer |
| `BackupFailed` | Restaurer la sauvegarde, Réessayer, Ouvrir le dossier des données |
| `AppDataDir` | Réessayer, Quitter |
| `CryptoUnavailable`, `CryptoMigrationFailed` | Réessayer |
| `DevOnRealData` | message de refus, rien à relancer |

« Réessayer » appelle `retry_boot` puis recharge la page ; « Restaurer la sauvegarde » appelle `restore_latest_backup` puis la même relance. Un `<details>` « Voir les détails techniques » déplie le JSON brut. L'`ErrorBoundary` de `App.tsx` attrape les erreurs de rendu et propose Recharger ; `RouteErrorBoundary` fait de même par route.

## 7. Erreurs côté interface

`lib/errors.ts` : `formatError(err)` lit le `kind` d'une erreur Rust `{ kind, message, detail }` (`BY_KIND`), garde des expressions régulières pour les chaînes des plugins et les erreurs JavaScript, et rend `{ code, message, userMessage }`. `errorText(e)` met l'erreur dans une phrase (jamais `${e}`). `reportError(error, { source, context, … })` (`lib/reportError.ts`) formate, écrit dans `error_log` et décide du toast ; `notifyError` pour un message simple. ESLint refuse `toast.error` direct. `lib/errors.contract.test.ts` tient la liste des variantes contre `error.rs`.

## 8. Constantes et formatage

`lib/constants.ts` : navigation, `PAGE_TITLES`, `TIME_RANGES` (7j, 30j, 90j, 1 an, Tout), zones de tension `BP_ZONES` (Optimal jusqu'à 120/80, Normal, Normal haut, Hypertension grades 1 à 3), `METRIC_COLORS` (chaque clé pointe vers un jeton de série `hsl(var(--series-<nom>))`), `DIRECTION_LABELS` (Diminuer, Augmenter, Maintenir).

`lib/formatters.ts` : `formatDate` (« 3 avr. 2026 »), `formatDateTime`, `formatNumber` (virgule décimale), `formatWeight`, `formatPercent`, `formatDuration` (« 7h 30min »), `formatBP` (« 120/80 »), `orDash(valeur, format)`. Tous rendent `—` sur une valeur absente ou invalide. Toute mesure est `number | null` : les formatters rendent `—`, les fonctions de `stats.ts` ignorent les `null`, Recharts ne trace pas le point. `lib/dates.ts` donne la date du jour locale (`nowDate`), jamais `toISOString().slice(...)` (le jour UTC est la veille entre 0 h et 2 h à Paris).

## 9. Tableau de bord

`Dashboard.tsx` lit les mesures de la période (composition, tension, sommeil, activité) et, si une comparaison est active, celles de la période de comparaison. Les cartes de dernier relevé et les tendances viennent de `get_dashboard_cards` en une lecture (`useDashboardCards`). `SourcesStatusBanner` dit en tête qu'aucune source n'est connectée, et le bandeau de ré-autorisation de Google Health s'y ajoute le cas échéant.

## 10. Paramètres, sources et imports

`pages/Settings.tsx` a deux colonnes : **Sources** (`SourcesPanel`) et **Application** (profil, apparence, dossier des données, sauvegardes, métriques de la base et purges).

`SourcesPanel` affiche une `SourceCard` par entrée de `list_sources` : état de connexion, identifiants (`SourceCredentials`), bouton Synchroniser, bouton Déconnecter (confirmé par une boîte de dialogue), appareils pour Withings, ré-autorisation à venir. Il porte l'intervalle de synchro (`SyncIntervalCard`), la carte de reconnexion (`ReconnectNoticeCard`, étape par étape après une invalidation de secrets), l'onglet **Priorités** (`PrioritiesTab` : ordre des sources par mesure qui en a au moins deux) et le bloc **Import** (`ImportCard`) : sélection du fichier par la boîte de dialogue du système, progression (`import-progress`), annulation, résumé. Le CSV passe par `CsvAssistant` (aperçu, correspondance des colonnes, préréglage LibreView, mémorisation).

`features/sources/` : `api.ts` (registre, connexion, déconnexion, priorités, imports), `hooks.ts` (une seule requête du registre, `SOURCES_KEY`, dont `useAnySourceConnected` et `useSourceCredentials` sont des sélections), `useOAuthFlow`, `useImportSource`, `useImportRunner`, `useCsvImport`, `events.ts` (noms des événements, tenus par `events.contract.test.ts`), `labels.ts` (libellés des sources et des domaines, comparés par un test à la graine de la migration 019), `SourceBadge` et `sourceColumn` (la colonne Source des listes), `reconnectNotice.ts`, `reauth.ts`.

Les listes de lignes (nuits, entraînements, tensions, ECG) gardent toutes les sources et nomment la source de chaque ligne dans une colonne ; les courbes n'en montrent qu'une par jour.

## Fiche de mensurations sur-mesure (feature-folder `bodyProfile`)

Feature-folder `src/features/bodyProfile/` (pattern decrit en D34) pour la
fiche de mensurations sur-mesure : releve de 47 mensurations corporelles via un assistant
pas a pas, consultation dans un onglet dedie, export PDF illustre.

### Structure

| Fichier | Role |
|---|---|
| `catalog.json` | Source unique des 47 mensurations (identite, zone, nature, bornes, aide a la mesure, ancre d'illustration). Partagee avec le backend Rust (`include_str!` dans `body_catalog.rs`), ne pas dupliquer. |
| `catalog.ts` | Charge `catalog.json` (`CATALOG`), expose `ZONES`, `KEY_MEASUREMENTS` (les 22 mensurations `isKey`), `findMeasurement(slug)`, `byZone()`. |
| `types.ts` | `BodyMeasurement`, `Anchor`, `SessionEntry`, `SheetRow` — miroir des structs Rust de `commands/body_profile.rs`. |
| `steps.ts` | `buildSteps(course)` construit les etapes du wizard a partir du catalogue : regroupe les paires laterisees (`pairKey`) en une seule etape a 2 champs, trie par `order`. Course `"essential"` (16 etapes, mesures `isKey` uniquement) ou `"full"` (33 etapes, catalogue complet). |
| `api.ts` | 3 invokes : `getBodySheet`, `addMeasurementSession(measuredAt, entries)`, `exportBodySheetPdf`. |
| `hooks.ts` | `useBodySheet` (query), `useAddMeasurementSession` (mutation, invalide `bodySheet`/`measurementStats`/`manualMeasurements`/`clothingInputs`), `useExportBodySheetPdf` (mutation, toast avec le chemin du fichier genere). |

### Composants (`components/`)

| Composant | Role |
|---|---|
| `MeasurementWizard` | Dialog principal. Ecran 1 : choix du parcours (Essentiel/Complet). Ecrans suivants : une etape par `WizardStepView` avec barre de progression, navigation Precedent/Suivant, puis `WizardRecap`. Confirmation (`useConfirm`) avant de fermer si une saisie non enregistree existe (`hasAnyValue`). La derniere valeur connue (`useBodySheet`) s'affiche en texte fantome (`placeholder`), jamais comme contenu du champ : seul ce qui est tape part vers `add_measurement_session`. |
| `WizardStepView` | Rend les 1 ou 2 champs numeriques (`Input type="number"`) de l'etape courante a cote d'un `BodyFigure` qui marque le point de l'etape en cours, et lui seul, avec la consigne de mesure (`guidance` du catalogue). |
| `WizardRecap` | Liste des mesures saisies (champs vides ignores) avant l'enregistrement final. |
| `BodyFigure` | Mannequin gris en PNG (`assets/body/front.png` / `back.png`, provenance dans `assets/body/PROVENANCE.md`, D83) avec un calque SVG de pastilles au repere de la figure (`viewBox 0 0 256 700`, `anchorGeometry.ts`, ancres du catalogue en pourcentage comptees depuis les pieds) ; `highlight` grossit le point de la mesure en cours, `annotations` affiche les points d'un jeu de mensurations (ex: `KEY_MEASUREMENTS` sur la fiche). Une ombre fine (`drop-shadow` sur le jeton `foreground`, coupee en sombre) detache le mannequin d'une carte claire. |

### Page Mensurations : 3 onglets, Fiche par defaut

`src/pages/Measurements.tsx` propose desormais **3 onglets** (`view: "sheet" | "sizes" | "body"`,
etat initial `"sheet"` — avant cette feature, seuls `"sizes"`/`"body"` existaient et l'etat
initial etait `"sizes"`) :

| Onglet | Composant | Contenu |
|---|---|---|
| **Fiche** (defaut) | `src/pages/measurements/SheetView.tsx` | Silhouettes face/dos annotees des 22 mesures cles, tableau des 47 mensurations groupees par zone, boutons "Nouveau releve" (ouvre `MeasurementWizard`) et "Exporter en PDF". |
| Tailles | `SizesView` | Assistant taille par marque (V2.13.0, voir historique projet). |
| Tours de corps | `MeasurementGrid` / `MeasurementDetail` | Grille de types de mensurations legacy (objectifs, historique par type). |

### Export PDF

`useExportBodySheetPdf` appelle `export_body_sheet_pdf` (commande Tauri) qui genere un PDF
deux pages (silhouettes annotees + tableau detaille) via `printpdf` et retourne le chemin du
fichier ecrit sur disque — affiche dans le toast de succes.

## Carte d'identite chiffree (feature-folder `profileCard`) — page Profil

Feature-folder `src/features/profileCard/` : page **Profil** (`src/pages/Profile.tsx`,
route `/profile`) qui **remplace la page Rapports** (supprimee, avec `HealthScoreRing`
et ses sections `Reports/sections/*`, devenus sans usage). La carte rassemble en un
ecran tout ce que Synk **peut deriver lui-meme** et s'exporte en PDF — aucune saisie
manuelle. Reduction de perimetre decidee par l'utilisateur le 2026-08-02 : les 9
champs d'identite saisis a la main (date de naissance, lateralite, couleurs,
groupe sanguin, allergies, correction visuelle, antecedents, vaccins) ont ete
retires, avec le formulaire d'edition et la commande d'ecriture qui allait avec.

### Structure

| Fichier | Role |
|---|---|
| `types.ts` | `Body`, `KeyMeasurement`, `Health`, `SizeEntry`, `RecordEntry`, `ProfileCard` — miroir des structs Rust de `commands/profile_card/`. |
| `api.ts` | 2 invokes : `getProfileCard`, `exportProfileCardPdf`. |
| `hooks.ts` | `useProfileCard` (query, cle `["profileCard"]`), `useExportProfileCardPdf` (mutation, toast avec le chemin du fichier genere). |

### Page Profil : 5 sections, lecture seule

`src/pages/Profile.tsx` charge la carte via `useProfileCard()` et rend, en grille 2
colonnes, les 5 sections de `src/pages/profile/sections/` — chacune miroir d'un bloc
du `ProfileCard` Rust. Aucun bouton d'edition : chaque bloc pointe vers sa page
source via un `Link` (`/body`, `/measurements`, `/medications`) :

| Section | Bloc `ProfileCard` | Contenu |
|---|---|---|
| `BodySection` | `body` | Age, sexe, taille (profil), puis poids, IMC, masse grasse, masse musculaire, hydratation, date de la derniere pesee (Withings) |
| `MeasurementsSection` | `key_measurements` | Mensurations cles (sous-ensemble `is_key` du catalogue bodyProfile) |
| `HealthSection` | `health` | Traitements en cours uniquement (lus depuis `medications`) |
| `SizesSection` | `sizes` | Tailles de vetements (lues depuis `clothing_sizes`) |
| `RecordsSection` | `records` | Records personnels avec leur date d'obtention (poids min/max, meilleur total de pas, plus longue nuit, meilleur score de sommeil, FC la plus basse, entrainement le plus long) |

### Export PDF

Le bouton "Exporter en PDF" appelle `export_profile_card_pdf` (commande Tauri), qui
agrege la carte via la meme fonction Rust que l'ecran (`get_profile_card_inner`) puis
la rend en PDF (5 sections, texte long passe a la ligne, page "Suite" ajoutee si le
contenu deborde) — chemin du fichier affiche dans le toast de succes.

## Theme et Tailwind 4 (R19, 2026-10-23)

Tailwind 4 se configure en CSS, dans `src/styles/globals.css` ; il n'y a plus de `tailwind.config.ts` ni de `postcss.config.js` (archives dans `_archive/2026-10-23_r19/`). Le plugin est `@tailwindcss/vite` (`vite.config.ts`).

| Quoi | Ou dans `globals.css` |
|---|---|
| Sources scannees | `@import "tailwindcss" source(none)` puis `@source` sur `index.html` et `src/**/*.{js,ts,jsx,tsx}` : sans cela la v4 lit aussi `docs/`, `audits/` et les `.md`, et sort des classes qui n'existent pas dans l'application |
| Mode sombre | `@custom-variant dark (&:where(.dark, .dark *))` : la classe `dark` est posee sur `<html>` par `AppLayout` |
| Jetons | canaux HSL nus dans `:root`, `.dark` et `.theme-<accent>` (lus par `src/lib/contrast.ts`) ; la correspondance vers les couleurs Tailwind est dans `@theme inline` (`--color-primary: hsl(var(--primary))`, rayons, hauteurs de ligne) |
| Utilitaire maison | `@utility glow-<couleur>` : une classe de `@layer utilities` n'est plus un utilitaire en v4, `hover:glow-*` disparaitrait sans erreur |
| `space-x-*` et `space-y-*` | redefinis a la formule de la v3 (marge haute sur les freres suivants) : la v4 met la marge basse sur les enfants autres que le dernier, ce qui ne marche pas sur un `<label>` en ligne |
| Preflight | `td`, `th`, `option` et le champ date gardent le remplissage du navigateur que la v3 laissait (`@layer base`) |

`divide-y` n'est plus employe : la v4 le calcule en bordure basse, la v3 en bordure haute, ce qui decale le texte d'un pixel ; les quatre listes portent `[&>:not([hidden])~:not([hidden])]:border-t`.

Verifier une montee de CSS : `npm run build` puis `node scripts/css-classes.mjs` (classes du CSS construit, a comparer par `diff` avec la liste d'avant) et `npm run parcours -- --captures=<dossier>` (46 captures 1920x1080, deux modes, cinq accents sur le tableau de bord, `manifest.json` avec des sondes de style). Un `diff` de listes ne voit pas un ecart de style ou de mise en page : comparer aussi les captures pixel a pixel.

## Graphiques recharts 3 et icones lucide 1 (R20, 2026-10-23)

### Graphiques

| Regle | Pourquoi |
|---|---|
| Une valeur d'infobulle se lit par `tooltipNumber(v)` (`components/charts/tooltipValue.ts`), jamais par un type de parametre | la 3 type la valeur `ValueType \| undefined` et le nom `NameType \| undefined` ; un `formatter={(v: number) => ...}` ne compile plus |
| `<Tooltip>` apres `<Legend>` dans le JSX | l'ordre de rendu fixe la superposition, l'infobulle doit passer au-dessus de la legende |
| `itemSorter={keepSeriesOrder}` (deja dans `tooltipProps`) sur les infobulles, `itemSorter={null}` sur les `<Legend>` | la 3 trie par defaut sur la cle de donnees (`awake`, `deep`, `light`, `rem`), la 2 gardait l'ordre des series |
| `axisTick` porte `fontSize: 12` | la 3 ne lit plus la classe `text-xs` posee sur l'axe, les graduations passaient a 16 px |
| `accessibilityLayer` reste actif | le graphique se parcourt au clavier (fleches) ; il ajoute des noeuds focalisables, `ChartA11yWrapper` garde son nom accessible (regle d'accessibilite R24 plus bas) |
| `react-is` est epingle `~19.2.4` et force par `overrides` | recharts 3 declare `react-is` en dependance paire ; une seule copie, de la ligne de React |

Ecarts de rendu releves a la montee (captures `npm run parcours` avant et apres, memes donnees) : une graduation de plus sur l'axe du poids (73 kg entre 71 et 74, la 2 la retirait faute de place), des ecarts de moins d'un pixel sur le dessin des barres et des courbes, rien d'autre. Les textes d'infobulle et l'ordre des series sont identiques.

### Densite des graphiques

| Graphique | Au-dela de | Traitement |
|---|---|---|
| Courbes et aires (poids, composition, tension, metrique generique, mensurations, correlation) | 500 points | moyennes par intervalle (`lib/charts/decimate.ts`), note « N points affiches sur M » ; points puis animation coupes a 90 et 200 points |
| Histogrammes du sommeil et de l'activite | 150 barres | une barre par semaine (lundi), puis par mois quand les semaines depassent encore 150 (`lib/charts/aggregateBars.ts`), moyenne des jours, infobulle « Semaine du ... » ou « mars 2020 », note « N semaines affichees pour M nuits » |

Mesure au CDP sur 3 650 jours de donnees, plage « Tout » : sommeil 14 600 barres, 30 330 noeuds, tache de 5,4 a 9,7 s (22,7 s avec recharts 3) avant ; 484 barres, 2 613 noeuds, 1,3 s apres. Activite 3 654 barres et 3,5 s avant (6,1 s avec la 3), 125 barres et 0,7 s apres. Correlation : 2,1 s avant, 0,13 a 0,2 s apres.

### Icones

lucide-react 1.x : plus aucun alias deprecie (`CircleAlert`, `TriangleAlert`, `CircleCheck`, `FileBraces`, `RotateCcwClock`, `ChartLine`, `LoaderCircle`, `Trash`, `CircleX`). La 1.x pose `aria-hidden="true"` sur chaque icone : une icone qui porte seule le sens d'un controle (bouton sans texte) recoit un `aria-label` explicite sur le bouton, jamais sur l'icone. Une nouvelle icone se verifie par `node logs/r20-alias-lucide.mjs` (nom canonique du paquet, copie de l'outil dans le journal du plan). La barre laterale designe ses icones par le nom de `constants.ts` (`"ChartLine"`).


## Accessibilité (R24, 2026-10-23)

| Besoin | Pratique | Où |
|---|---|---|
| Savoir où l'on est | `useRouteAnnouncement(mainRef, headingRef)` : à chaque changement de page, le titre de la fenêtre devient le nom de la page suivi de « Synk », le focus passe sur le `h1` du bandeau (`tabIndex={-1}`) et `main` remonte en haut ; au premier rendu le focus reste au lien d'évitement | `hooks/useRouteAnnouncement.ts`, `AppLayout`, `Header` |
| Titres dans l'ordre | `CardTitle` rend un `h2` par défaut, `as="h3"` ou `as="h4"` sous un titre de section ; `EmptyState` rend un `h2` ; la règle axe `heading-order` est active | `components/ui/card.tsx` |
| Bascule | `aria-pressed` sur chaque bouton de période, de vue, de système de tailles, de catégorie | `TimeRangeSelector`, `ViewTabs`, `SizesHeader`, `Explorer` |
| Interrupteur | `role="switch"` et `aria-checked` (pages masquables de Paramètres) | `AppearancePanel` |
| Accordéon | `aria-expanded` et `aria-controls` vers l'`id` du panneau, rendu dans un composant par ligne (`useId`) | `LabAnalysesList`, `ErrorLogSection` |
| Menu | `role="menu"` et `menuitem`, focus sur la première option à l'ouverture, flèches, Échap qui rend le focus au bouton | `QuickAddButton` |
| Erreur de champ | `validateAll` donne un message par champ, affiché sous le champ (`aria-invalid`, `aria-describedby`), focus sur le premier champ invalide ; `role="alert"` ne sert plus qu'à l'échec d'enregistrement | `createManualForm` |
| Tableau | `DataTable` exige `label` (nom du tableau et de sa recherche) et, avec `onDelete`, `rowLabel` (« la pesée du 28 juin 2026 ») ; chaque corbeille s'appelle « Supprimer la pesée du … » | `components/shared/DataTable.tsx` |
| Statut | jamais par la couleur ou une icône masquée seule : coche et triangle (forme) plus un `sr-only` en mots ; même règle pour la tendance, la gravité d'un insight, le statut d'une synchro | `Labs/helpers.tsx`, `Insights`, `Timeline` |
| Graphique | `ChartA11yWrapper` rend une `figure` nommée (et non un `role="img"`, qui rendait ses enfants présentationnels) avec `accessibilityLayer` ; les notes de la figure sont lues ; un histogramme agrégé le dit dans son nom (« affichées en moyennes par semaine ») | `components/charts/` |
| Progression | « Étape 2 sur 4 » en `sr-only` dans l'assistant de premier lancement | `StepLayout` |
| Texte | jamais sous 12 px (`text-xs` au minimum, en rem) ; la fenêtre se zoome (`zoomHotkeysEnabled`, Ctrl + molette, Ctrl +/-), le niveau ne survit pas au redémarrage | `lisibilite.test.ts`, `tauri.conf.json` |

Le zoom de la fenêtre ne se prouve pas par CDP : un `Input.dispatchMouseEvent` de type `mouseWheel` avec Ctrl, comme un Ctrl + `=` simulé, n'a rien changé (`devicePixelRatio` 1,25 et `innerWidth` 1280 avant et après, mesure du 2026-10-23), parce que WebView2 traite le zoom dans son processus et non dans la page. La seule preuve est le réglage `zoomHotkeysEnabled` (schéma de la CLI, `node_modules/@tauri-apps/cli/config.schema.json`, pilote `IsZoomControlEnabled` sous Windows), tenu par `lisibilite.test.ts` ; l'essai à la main au clavier reste à faire par le user.

Les icônes de lucide 1.x sont décoratives par défaut (`aria-hidden`, gardé par `icones.test.tsx`) : tout ce qu'elles disent doit exister en texte.

Le contraste ne se juge pas en jsdom (aucune mise en page : axe rend « incomplet », jamais une violation). `src/test-utils/axe.ts` coupe donc `cat.color` pour les tests de composant, et la règle `color-contrast` est tenue sur l'application réelle : `npm run parcours` injecte axe-core dans la WebView, en clair et en sombre sur les 19 pages, et compare le nombre d'éléments en défaut à `scripts/axe-contraste.json` (plafond par mode et par page, 0 par défaut, qui ne peut que baisser). Instrument éprouvé le 2026-10-23 : un texte gris clair injecté est signalé et un texte noir ne l'est pas ; sans correction, axe ne jugeait pas 54 éléments sur 63 du tableau de bord (le dégradé du fond, `bgGradient`), donc la passe met ce dégradé à plat et échoue si plus de 25 % du texte d'une page reste non jugé ; le texte des graphiques (`imgNode`) est compté à part. À la mesure du 2026-10-23 : cinq couples page et mode en défaut, ramenées à 0 par R25 (jetons d'état) ; `scripts/axe-contraste.json` est vide, le plafond de toute page vaut 0. Une infobulle de graphique ou un survol peint sous le pointeur peut s'y ajouter : la sortie de `parcours` liste alors les noeuds en défaut. Les paires de jetons restent tenues par `src/lib/contrast.test.ts`.


## Cohérence de l'interface (R25, 2026-10-23)

### Jetons d'état

Une couleur d'état s'écrit avec un jeton, jamais avec une couleur nommée de Tailwind ; `src/__tests__/palette.test.ts` refuse `text-emerald-600`, `bg-red-500/10`, `border-amber-*` et leurs semblables dans tout fichier de production, ainsi que `text-primary` (la couleur de remplissage, pas du texte) et un survol `hover:bg-accent` plein hors de `components/ui/`.

| En dur | Jeton | Lu sur |
|---|---|---|
| `text-emerald-*`, `text-green-*` | `text-success-text` | texte et icône |
| `text-amber-*`, `text-yellow-*`, `text-orange-*` | `text-warning-text` | texte et icône |
| `text-red-*`, `text-rose-*` | `text-destructive-text` | texte et icône |
| `text-blue-*`, `text-purple-*`, `text-indigo-*`, `text-primary` | `text-primary-text` | texte et icône |
| `bg-emerald-*/NN`, `bg-amber-*/NN`, `bg-red-*/NN` | `bg-success/NN`, `bg-warning/NN`, `bg-destructive/NN` | fonds teintés, barres de progression |
| `border-<couleur>-*` | `border-success`, `border-warning`, `border-destructive` | contours |
| `bg-indigo-600` (bouton flottant) | `bg-primary text-primary-foreground hover:bg-primary/90` | boutons |

`--success`, `--warning`, `--primary-text`, `--success-text`, `--warning-text` et `--series-*` sont dans `:root`, `.dark` et, pour `--primary-text`, dans chaque accent ; `contrast.test.ts` les tient (texte a 4,5:1, icones et series a 3:1, cinq accents, deux modes). Le placeholder des champs lit `--muted-foreground`. Les pastilles de choix d'accent (`AppearancePanel`) gardent des teintes fixes (`bg-[#6366f1]`...) : ce sont des donnees, pas des etats.

### Champs et controles natifs

`Input`, `Select` et `Textarea` (`components/ui/`) sont les seuls champs ; `src/__tests__/champs.test.ts` refuse un `<input>`, `<select>` ou `<textarea>` brut hors de `components/ui/` (cases a cocher, boutons radio, fichiers et champs caches exceptes). `globals.css` pose `color-scheme: light` sur `:root` et `dark` sur `.dark` (icone de calendrier, barres de defilement, listes deroulantes suivent le mode) et `accent-color: hsl(var(--primary))` sur cases et boutons radio.

### Graphiques temporels

`lib/charts/timeAxis.ts` : `withTimestamp(rows)` ajoute `ts` (minuit UTC du jour, tri croissant, ligne sans date lisible ecartee), `TIME_AXIS` (`dataKey: "ts"`, axe numerique a echelle de temps borne aux donnees), `formatTick(ts)` pour graduations et infobulles. Poids, tension, metrique generique et mensurations l'emploient : l'ecart entre deux repères suit le nombre de jours. Les histogrammes gardent une barre par jour ou par periode. `BodyCompositionChart` trace une courbe par serie (muscle, graisse, os), chacune avec son axe, en trois colonnes a partir de 1024 px ; une serie que l'appareil ne mesure pas n'est pas tracee.

### Titres

`CardTitle` vaut `text-base` par defaut ; le titre de page (`h1` du bandeau) `text-2xl font-semibold` ; un `h2` de section de page `text-lg font-semibold`.

### Raccourcis clavier (decision D11)

| Raccourci | Action |
|---|---|
| `Ctrl+Maj+S` | synchroniser |
| `Ctrl+Maj+A` | ajout rapide (va d'abord au tableau de bord hors des pages qui offrent le bouton) |
| `Ctrl+B` | replier ou deplier la barre laterale (choix garde dans `synk-sidebar-collapsed`) |

Definis dans `lib/shortcuts.ts`, branches par `hooks/useLayoutShortcuts.ts` sur `hooks/useKeyboardShortcuts.ts` (ignore la saisie dans un champ, exige les bons modificateurs, retient l'evenement). Annonces dans les infobulles et par `aria-keyshortcuts`. `lib/shortcuts.test.ts` verifie qu'aucun ne recouvre un accelerateur de WebView2 (`Ctrl+R`, `F5`, `Ctrl+P`, `Ctrl+F`, `Ctrl+N`...) ni une touche de l'application. Reserve : `Ctrl+Maj+S` est un accelerateur d'Edge (capture) que WebView2 a laisse actif malgre `AreBrowserAcceleratorKeysEnabled = false` (WebView2Feedback 5136) ; le crochet appelle `preventDefault`, ce qui suffit pour un accelerateur non reserve, mais un evenement synthetique de CDP ne passe pas par le chemin des accelerateurs du navigateur : la preuve sur la vraie fenetre est un essai a la main.

`QuickAddButton` n'est rendu que sur les pages ou l'on saisit une mesure (`lib/quickAdd.ts`, `showsQuickAdd`) et passe sous la modale de l'assistant (`z-40` contre `z-50`).

### Assistant de premier lancement

`lib/onboarding.ts` (`ONBOARDING_KEY`, `readOnboardingDone`, `markOnboardingDone`) : la cle `synk-onboarding-done` est ecrite `true` et relue par `isOnboardingDone`, qui accepte le booleen et la chaine `"true"` ecrits par les versions installees. L'assistant se quitte des la premiere etape (« Passer l'assistant ») ou par Echap, et s'en souvient. `components/shared/OnboardingWizard/OnboardingGate.tsx` (D82, charge a la demande) ne l'ouvre que sur une installation vide : cle absente, aucune mesure (`get_db_metrics`), aucune source connectee (`list_sources`) ; une base deja remplie reecrit la cle sans l'ouvrir.

### Theme avant React

`public/theme-init.js` (script classique charge par `index.html`, la CSP `script-src 'self'` refusant le script en ligne) pose `dark` et `theme-<accent>` sur `<html>` avant le premier rendu ; `tauri.conf.json` peint la fenetre en `#080c17` des le lancement. En mode clair memorise, un bref fond sombre de fenetre reste possible avant la page (le sombre est le mode par defaut). `vitest.config.ts` pose `publicDir: false` pour que les tests lisent `public/theme-init.js` par `?raw`.

### Couches

`lib` ne depend ni de `features`, ni de `components`, ni de `pages`, ni de `hooks`, ni de `stores` (regle `no-restricted-imports`, seule exception `lib/reportError.ts` qui ecrit le journal d'erreurs) ; le prechauffage du tableau de bord vit dans `hooks/appReady/bootPrefetch.ts`. `@typescript-eslint/no-non-null-assertion` est en erreur hors des tests.
