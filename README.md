# Synk

Application desktop de suivi santé qui rassemble Withings, Oura, Polar, Whoop, Google Health et des fichiers exportés (Apple Santé, FIT, GPX, TCX, CSV). Tout reste sur la machine : base SQLite locale, jetons OAuth chiffrés, aucune télémétrie.

Version courante : 1.0.0 (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`).

## Prérequis

| Outil | Version | D'où vient le chiffre |
|---|---|---|
| Rust (rustup) | chaîne épinglée | `rust-toolchain.toml` (rustup la télécharge au premier appel de `cargo`) |
| Node.js + npm | `engines.node` | `package.json`, appliqué par `engine-strict` dans `.npmrc` |
| Visual Studio Build Tools 2022, workload « Desktop development with C++ » | — | requis par Tauri v2 pour compiler le backend Rust sur Windows |

Rust et les Build Tools ne servent qu'à `npm run tauri:dev` et au build de l'installeur. La veille, la couverture Rust et la mutation demandent aussi des outils cargo, listés dans [`docs/release.md`](./docs/release.md). Pour le frontend seul (`npm run dev`), Node suffit. Le CLI Tauri est une dépendance de développement : `npm install` suffit, aucune installation globale.

## Installation

```bash
npm install
```

Aucun fichier `.env` : la seule variable d'environnement lue par le backend est `SYNK_LOG` (niveau du journal, voir `docs/faq.md`), et `vite.config.ts` lit `TAURI_DEV_HOST` directement dans l'environnement du shell. Les identifiants de chaque marque se saisissent dans l'app (voir « Premier lancement »).

## Lancer

```bash
npm run dev          # frontend seul (Vite, port 1420, strictPort)
npm run tauri:dev    # app complète, backend Rust + WebView, sur une copie de développement
```

`npm run tauri:dev` utilise l'identifiant `com.synk.health.dev` : base, journal et clé de chiffrement séparés de l'app installée. Un build de développement sur l'identifiant de production est refusé au démarrage ; ne jamais lancer `npm run tauri dev` nu.

Builds :

```bash
npm run build              # build web -> dist/
npm run release -- X.Y.Z   # release complète : contrôles, installeur NSIS, somme SHA-256 (docs/release.md)
```

Purge du cache de compilation de développement : `cargo clean --profile dev` (depuis `src-tauri/`). Jamais `cargo clean` seul : il vide aussi `target/release`, où vit l'installeur déjà construit.

`bundle.targets` vaut `["nsis"]` dans `src-tauri/tauri.conf.json`. La sortie Windows est donc un seul installeur :

- `src-tauri/target/release/bundle/nsis/Synk_<version>_x64-setup.exe`

Aucun MSI n'est produit. Pour en obtenir un, ajouter `"msi"` a cette liste.

Process de release complet : [`docs/release.md`](docs/release.md).

## Premier lancement utilisateur

1. Installer l'EXE NSIS : aucun MSI n'est produit. Le binaire n'est pas signé, accepter l'avertissement SmartScreen via « Plus d'infos » puis « Exécuter quand même ».
2. L'assistant de premier lancement propose de brancher des sources, ce qu'on peut passer. Chaque marque demande de créer sa propre application (gratuit) et de coller l'identifiant et le secret dans Paramètres, Sources ; le pas à pas par marque est dans le guide utilisateur.
3. Cliquer sur Connecter, faire le tour OAuth, la synchronisation démarre. Un fichier exporté s'importe depuis le même écran.
4. Naviguer dans les 19 pages (barre latérale en 5 sections).

Guide complet : [`docs/user-guide.md`](docs/user-guide.md). Dépannage (jeton expiré, sync échouée, base corrompue) : [`docs/faq.md`](docs/faq.md).

## Vérifier

Une fois par clone, poser le hook de commit. Il délègue au hook global du poste, qui décide seul s'il faut scanner les secrets (gitleaks, uniquement pour un dépôt destiné à GitHub : ce dépôt est sur la forge maison, où rien n'est scanné), puis lance `npm run check`. Il refuse le commit si le hook global est introuvable :

```bash
npm run hooks:install
```

Ensuite, deux niveaux :

```bash
npm run check    # rapide, lancé à chaque commit : prettier, tsc, eslint sans avertissement, taille des fichiers, cargo fmt
npm run verify   # complet, avant toute fusion : le niveau rapide, puis vitest avec couverture, build, budget du bundle, clippy -D warnings, cargo test
```

`npm run verify` s'arrête au premier échec, affiche la durée de chaque étape et l'ajoute à `logs/verify.log`. Il échoue aussi quand le nombre de tests passe sous les planchers de `scripts/verify-floors.json` (seule source de ces nombres) et quand la documentation cite un chiffre que le code contredit (`src/__tests__/doc-drift.test.ts`) ; un nombre en hausse demande de relever le plancher. Cargo tourne avec `-j 2` et sans incrémental, deux réglages portés par `src-tauri/.cargo/config.toml` (sans quoi la mémoire du poste s'épuise) : lancer `npm run verify` plutôt que `cargo` à la main. Chaque ligne du journal porte le commit et l'arbre vérifiés (`arbre=modifie` si l'arbre n'était pas propre).

Le hook `pre-push` refuse un push de `main` tant que `logs/verify.log` n'a pas de passage complet réussi sur l'arbre du commit poussé (décision D12) : vérifier la branche propre, puis la fusionner en squash, qui garde le même arbre. Les autres branches partent sans contrôle.

Les tests vivent à côté du code : `src/**/*.test.{ts,tsx}` et `scripts/**/*.test.mjs` pour Vitest, modules `#[cfg(test)]` et `src-tauri/tests/` pour Rust.

## Fonctionnalités

- Composition corporelle : poids, IMC, masse grasse / musculaire / osseuse, hydratation
- Tension artérielle : systolique, diastolique, fréquence cardiaque, classification OMS
- Sommeil : durée, phases (profond / léger / REM), score, efficacité, apnée
- Activité : pas, distance, calories, zones de fréquence cardiaque
- Entraînements : catégories Withings, durée, FC, distance, calories
- Coeur et ECG : statut ECG, fréquence cardiaque, durée QRS
- Vitaux Body Scan : VO2 Max, âge vasculaire, eau intracellulaire, masse viscérale
- Mensurations : tours de corps et types personnalisés, objectifs, sparklines
- Médicaments : suivi des prises et des sauts, observance
- Analyses labo : résultats biochimiques avec sparklines et zones de référence
- Insights automatiques : tendances et anomalies (poids, FC, sommeil, activité, tension)
- Journal : notes datées avec tags
- Alertes : neuf métriques avec seuils configurables
- Profil : carte d'identite (corps, mensurations cles, traitements, tailles, records), export PDF
- Corrélations : Pearson double-axe entre deux métriques
- Explorer : toutes les métriques, avec les points des sources non retenues en discret
- Sources : Withings, Oura, Polar, Whoop, Google Health par OAuth ; Apple Santé, FIT, GPX, TCX, CSV par import tout ou rien, sans doublon au réimport
- Priorités : une source par mesure et par jour, ordre réglable
- Export : CSV, PDF, JSON (export complet et import)

## Stack

| Couche | Technologie |
|---|---|
| Desktop | Tauri 2.11 (Rust + WebView), contrainte `~2.11.6` de `src-tauri/Cargo.toml` |
| Backend | Rust (chaîne épinglée par `rust-toolchain.toml`, `rust-version` 1.94), sqlx 0.9, reqwest 0.13, tokio, chrono, printpdf 0.12.8, csv, quick-xml, zip, fitparser |
| Frontend | React 19, TypeScript 6 strict, Vite 8 |
| UI | Tailwind CSS 4, shadcn/ui, Recharts 3, Lucide 1 |
| State | Zustand 5, TanStack Query 5 |
| Base de données | SQLite locale via sqlx, 23 migrations versionnées (`src-tauri/migrations/001` à `023`) |
| API | Withings, Oura, Polar AccessLink, Whoop, Google Health (OAuth 2) |
| Chiffrement | aes-gcm 0.11 et keyring-core 1 : les secrets OAuth sont chiffrés au repos, la clé maître vit dans le Gestionnaire d'identification de Windows |

Décisions notables : la CSP de `tauri.conf.json` ne laisse sortir de la WebView que l'IPC (tous les appels aux marques partent de Rust, en https seul et sans redirection), le profil release compile en `lto = true`, `codegen-units = 1` et `strip = true`, et le boot migre automatiquement les secrets stockés en clair vers leur forme chiffrée.

## Structure du dépôt

```text
Synk/
├── src/                       Frontend React/TypeScript
│   ├── pages/                 19 pages (dont Explorer), chargées à la demande
│   ├── features/              un dossier par domaine métier (api.ts, types.ts, hooks.ts, components/)
│   ├── components/
│   │   ├── charts/            graphiques + TimeRangeSelector + GenericMetricChart
│   │   ├── forms/             GoalForm + factory createManualForm
│   │   ├── shared/            StatCard, DataTable, EmptyState, OnboardingWizard, etc.
│   │   ├── layout/            Sidebar, Header, AppLayout, QuickAddButton
│   │   └── ui/                primitifs shadcn/ui
│   ├── hooks/                 hooks cross-domain (createCrudHook, useAppReady, useSync, useTimeRange)
│   ├── lib/                   formatters, constants, stats, errors, storage, logger, timeRange
│   └── stores/                Zustand (uiStore, syncStore)
├── src-tauri/                 Backend Rust
│   ├── migrations/            001 à 023
│   └── src/
│       ├── commands/          adaptateurs IPC (docs/commandes.md)
│       ├── services/          couche métier : sauvegardes, métriques, constats, import, journaux
│       ├── sources/           contrat Source, registre, OAuth commun ; une marque par dossier ; import/ pour les fichiers
│       ├── crypto/            chiffrement AES-256-GCM des secrets + clé du trousseau
│       ├── metric_repository.rs  lecture brute d'une série par métrique
│       ├── boot.rs, boot/     séquence de démarrage, sauvegardes, relance
│       ├── error.rs           enum AppError
│       └── db/                pool, migrations, semis, portail d'écriture longue
├── docs/                      documentation technique et utilisateur
├── CHANGELOG.md
├── CLAUDE.md                  consignes pour un agent (règles, index des décisions et des pièges)
└── LICENSE
```

## Documentation

Pour utiliser :

- [Guide utilisateur](docs/user-guide.md) : installation, connexion des sources, navigation
- [FAQ et dépannage](docs/faq.md)
- [CHANGELOG](CHANGELOG.md)

Pour développer :

- [Guide développeur](docs/dev-guide.md) : ajouter une page, un type
- [Architecture](docs/architecture.md) : couches, démarrage, chiffrement, erreurs, sources
- [Commandes IPC](docs/commandes.md) : catalogue généré
- [Décisions](docs/decisions.md) et [pièges](docs/pieges.md)
- [Base de données](docs/database.md)
- [Stack Métriques V2](docs/metrics-v2.md) : table unifiée, repository, hooks
- [API Withings](docs/api-withings.md) : endpoints, OAuth, mapping
- [API des autres sources](docs/api-sources.md) : Oura, Polar, Whoop, Google Health, imports
- [Frontend](docs/frontend.md) : composants, hooks, conventions
- [Build et release](docs/release.md) : `npm run release`, bundle NSIS, signature, distribution

Licence : MIT, voir [LICENSE](LICENSE).
