# Architecture

Synk est une application Tauri v2 : une WebView React qui affiche, un processus Rust qui possède les données. Tout ce qui sort ou entre passe par des commandes IPC ([commandes.md](commandes.md) : 111 commandes, catalogue généré). Ce document décrit le code tel qu'il est ; les raisons de chaque choix sont dans [decisions.md](decisions.md), les pièges dans [pieges.md](pieges.md).

## Vue d'ensemble

```
WebView (React 19)                         Processus Rust (synk_lib)
 pages ─ hooks React Query ─ api.ts ──invoke──> commands/ (adaptateurs)
   │            │                                   │
 stores     événements <────── emit ────────  services/  (métier)
 (Zustand)  (app-ready, sync-progress,             │
             import-progress, <source>-auth-*)  metric_repository, db/  ──> SQLite (synk.db, WAL)
                                                   │
                                          sources/ (contrat Source, OAuth, 5 marques, 4 imports)
                                                   │
                                          crypto/ (coffre AES-GCM, trousseau Windows)
```

Les couches sont tenues par `src-tauri/tests/integration/architecture.rs` : aucun fichier hors de `commands/` et de `lib.rs` ne nomme `crate::commands`, et aucun module hors de `sources/<x>/`, du registre et de `lib.rs` ne nomme `sources::<x>`. `oauth` et `store` sont des modules partagés, pas des sources.

## Backend (`src-tauri/src/`)

| Dossier ou fichier | Rôle |
|---|---|
| `lib.rs` | Construit l'application : plugins (instance unique, ouverture d'URL, dialogue, journal), état de démarrage, liste des commandes (`generate_handler!`). |
| `boot.rs`, `boot/` | Démarrage typé : `run`, `runner` (instantané et relance), `auto_backup`, `periodic_backup`, `post_ready`, `schedule`. |
| `commands/` | Adaptateurs IPC : une commande extrait l'état Tauri, appelle un service ou une lecture, rend `Result<T, AppError>`. Un module par domaine (`body_measures`, `sleep`, `export/`, `import`, `sources`, `metrics`, `dashboard/`, etc.). |
| `services/` | Couche métier : `backup`, `backup_files`, `audit`, `error_log`, `sync_log`, `retention`, `insights` (et `detectors/`), `metrics` (choix de la source, écriture des observations), `sources` (état des sources, déconnexion), `import`. N'importe jamais `commands/`. |
| `sources/` | Le contrat `Source`, le registre, l'OAuth commun (`oauth/`), le magasin de jetons (`store/`), la synchro (`sync.rs`), les cinq marques (`withings/`, `oura/`, `polar/`, `whoop/`, `google_health/`) et les imports de fichiers (`import/`). |
| `db/` | `pool` (ouverture, PRAGMA), `bootstrap` (registre `MIGRATIONS`), `migrations` (exécuteur), `legacy` (tables `*_legacy` et vues), `seed`, `maintenance`, `gate` (portail d'écriture longue). |
| `crypto/` | `CryptoVault`, format `enc:v1:`, trousseau, migration des secrets en clair, invalidation des secrets illisibles. |
| `metric_repository.rs`, `metric_model.rs` | Lecture brute d'une série par métrique ; types partagés (`SourceId`, `Observation`) qui évitent un cycle entre `sources` et `services`. |
| `error.rs`, `validation.rs`, `utils/` | `AppError` ; contrôles de saisie ; dates, HMAC, HTTP (https seul, aucune redirection), masquage des secrets, texte. |
| `body_catalog.rs`, `logging.rs` | Catalogue des 47 mensurations lu depuis `catalog.json` ; niveau du journal. |

## Démarrage

Avant Tauri, `main` sonde le conteneur de l'app Claude (`boot::container`, D81) : un build de release qui s'y trouve se relance par `explorer.exe` et s'arrête, le journal dit ensuite `lancement hors conteneur`.

`spawn_boot` lance `boot::run` en tâche détachée et range l'issue dans un instantané (`Loading`, `Ready`, `Failed`) que le front peut relire par `get_boot_state`, puis l'émet (`app-ready` ou `app-error`, trois fois à 200 ms d'écart pour couvrir une WebView lente). `retry_boot` relance seulement après un échec : un second démarrage réussi ouvrirait un second pool.

| Étape | Action | Échec |
|---|---|---|
| 0 | Refus d'un build de dev sur l'identifiant de production | `DevOnRealData` |
| 1 | Dossier de données de l'application | `AppDataDir` |
| 2 | Sauvegarde avant migration, seulement si une migration est due | `BackupFailed` (fatal, l'écran propose la restauration) |
| 3 | Ouverture du pool, schéma, 3 tentatives (300 puis 600 ms) | `DbBusyAfterRetry`, `DbLocked`, `MigrationFailed` |
| 4 | `PRAGMA integrity_check` | avertissement `IntegrityWarning`, jamais fatal |
| 5 | Coffre de chiffrement (trousseau Windows) | `CryptoUnavailable` |
| 6 | Invalidation des secrets que la clé courante ne lit pas | `CryptoMigrationFailed` si le SQL échoue |
| 7 | Chiffrement des secrets encore en clair | `CryptoMigrationFailed` |

Après `Ready`, deux tâches détachées : amorçage des données héritées et des curseurs, puis une boucle horaire qui rappelle la sauvegarde quotidienne (elle garde sa règle de 20 heures) ; et la rétention des journaux. Côté front, `useAppReady` écoute les événements et relit l'instantané, `BootErrorScreen` propose la sortie adaptée à chaque `kind`, `BootStuckScreen` prend le relais si rien n'arrive en 12 secondes.

## Base de données

SQLite en mode WAL, 5 connexions, clés étrangères actives. Le schéma est le résultat du registre `MIGRATIONS` rejoué à chaque démarrage (idempotent) ; il compte 43 tables/vues après 23 migrations. Détail dans [database.md](database.md). Trois règles à connaître : on écrit sur les tables `*_legacy`, jamais sur leurs vues ; une métrique a une seule table source (`metric_repository::source_of`) ; un import long ferme le portail `db/gate.rs` et les autres écritures échouent vite avec un message clair.

## Sources et synchronisation

Une source déclare son identifiant, son libellé, son genre (`OAuth(&OAuthSpec)` ou `File(&FileSpec)`), ses domaines, son nombre d'étapes et, selon le cas, `sync`, `import`, `revoke`. Le registre `all_sources()` en liste neuf : Withings, Oura, Polar, Whoop, Google Health (OAuth) puis Apple Santé, FIT, traces GPX/TCX et CSV (fichiers). Une source neuve s'ajoute au registre et à la table `sources` ; un test refuse l'oubli.

`sync_sources` parcourt les sources connectées dans l'ordre du registre. Chaque source émet `sync-progress` (`source`, `etape`, `total`, `libelle`), son échec est une issue `SourceOutcome` journalisée dans `sync_log` sans arrêter les suivantes. Si au moins une source a réussi, les détecteurs de constats tournent une fois à la fin. La synchro refuse de démarrer pendant un import.

Les jetons passent par `sources/store/` (`source_auth`, `source_credentials`, chiffrés), le rafraîchissement par `oauth::refresh_locked` (verrou par source, relecture après acquisition). Les observations des sources API s'écrivent par `ObservationWriter` (`revising()` met à jour une valeur révisée), celles des fichiers gardent la première. Le détail par marque est dans [api-withings.md](api-withings.md) et [api-sources.md](api-sources.md).

### Imports de fichiers

`import_source_file` aiguille par source vers `Source::import`. Le pipeline commun (`sources/import/pipeline.rs`) lit dans `spawn_blocking`, envoie par lots de 2 000 sur un canal borné, écrit dans une seule transaction et rend tout annulable (retour arrière complet). Les lectures XML sont plafonnées (`limits.rs`). L'empreinte SHA-256 de chaque mesure rend le réimport sans doublon.

## Chiffrement

Les jetons et identifiants des sources sont chiffrés au repos en AES-256-GCM, format `enc:v1:<base64(nonce||ciphertext||tag)>`. La clé de 32 octets vit dans le Gestionnaire d'identification Windows (service `synk` en production, `synk-dev` en développement). La base de santé elle-même reste en clair, protégée par les droits du compte Windows. `open_str` refuse une valeur non chiffrée. Limites : une sauvegarde restaurée sur un autre PC ne se déchiffre pas, les jetons sont effacés au démarrage suivant et l'utilisateur se reconnecte ; les mesures restent lisibles.

## Erreurs

Toute commande rend `Result<T, AppError>`. `AppError` se sérialise en `{ kind, message, detail }` : `kind` est le nom de la variante (`Db`, `Validation`, `NotFound`, `Json`, `Forbidden`, `Internal`, `Http`, `Unauthorized`, `RateLimit`, `Crypto`), `message` le texte des journaux, `detail` la charge. `utils::redact` masque les secrets avant tout journal, événement ou `sync_log`. Côté front, `formatError` lit le `kind`, `errorText` met l'erreur dans une phrase, `reportError` l'écrit dans `error_log` et décide du toast.

## Journal

`tauri-plugin-log`, niveau `info` par défaut, réglé par la variable d'environnement `SYNK_LOG` (`error`, `warn`, `info`, `debug`, `trace` ; une valeur illisible garde `info` et le dit). Cinq fichiers d'un Mo au plus sont conservés.

## Sauvegardes

Toute copie de la base passe par `services::backup::copy_database_safely` (`VACUUM INTO`, copie vérifiée avant de prendre son nom). Trois dossiers sous le dossier de données : `backups/auto/` (avant migration, 3 gardées), `backups/daily/` (quotidienne, 14 gardées), `backups/` (manuelle, 5 gardées). La restauration prend la plus récente qui passe la vérification.

## Frontend (`src/`)

| Dossier | Contenu |
|---|---|
| `pages/` | 19 pages routées dans `App.tsx` (chargées à la demande) et leurs dossiers de sections. |
| `features/<domaine>/` | `api.ts` (seul endroit où l'on appelle `invoke`), `types.ts`, `hooks.ts` (React Query), `components/`. |
| `components/` | `ui/` (primitifs shadcn), `shared/`, `charts/`, `forms/`, `layout/`, `brand/`. |
| `hooks/` | `createCrudHook`, `useSync`, `useAppReady` et `appReady/`, `useTimeRange`, raccourcis, annonce de route. |
| `lib/` | Formatage, dates, erreurs, constantes de navigation, aides de graphiques. Ne dépend d'aucune couche supérieure. |
| `stores/` | `uiStore` (thème, accent, barre latérale, pages masquées), `syncStore` (synchro en cours, intervalle). |

Voir [frontend.md](frontend.md) pour les composants, le thème et l'accessibilité.

### Flux de données

Une page lit par un hook de `features/` (React Query), qui appelle `api.ts`, qui appelle `invoke`. Une mutation invalide ses clés au succès ; une synchro invalide la liste `SYNC_INVALIDATED_KEYS`. Le tableau de bord lit ses cartes en une commande (`get_dashboard_cards`).

### Connexion d'une source

1. Le front appelle `source_start_auth` (source nommée). Rust pose l'état attendu pour cette source et démarre, si besoin, le serveur de rappel unique sur `127.0.0.1:17483` (et `[::1]`).
2. Le navigateur ouvre la page d'autorisation de la marque.
3. Le rappel `/callback/<source>` (`/callback` pour Withings) est validé contre l'état du dernier essai ; le code est échangé et les jetons chiffrés enregistrés.
4. Rust émet `<source>-auth-success`, `<source>-auth-error` (erreur typée) ou `<source>-auth-timeout` (120 s). L'assistant de premier lancement et Paramètres écoutent ces événements.
