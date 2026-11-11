# Guide de developpement

## Prerequis et commandes

- Node.js et npm : version dans `engines` de `package.json`
- Rust : chaine epinglee dans `rust-toolchain.toml`
- Le CLI Tauri est une dependance de developpement (`npm install`), aucune installation globale
- Une fois par clone : `npm run hooks:install` (le hook de commit lance `npm run check`)
- Outils cargo de la veille, de la couverture et de la mutation : table dans `release.md`

| Commande | Description |
|---|---|
| `npm run tauri:dev` | App complete (Rust + WebView) sur la copie de developpement, voir plus bas |
| `npm run dev` | Frontend seul (Vite, port 1420), sans backend Tauri |
| `npm run check` | Verification rapide : prettier, tsc, eslint sans avertissement, taille des fichiers, cargo fmt |
| `npm run verify` | Verification complete : le niveau rapide, puis vitest avec couverture, build, budget du bundle, clippy `-D warnings`, cargo test |
| `npm run couverture:rust` | Couverture Rust (`cargo llvm-cov`), a lancer en fin de vague |
| `npm run parcours` | Visite des 19 pages sur la copie de dev, par CDP |
| `npm run doc:commandes` | Regenere `docs/commandes.md` (le gardien `doc-drift` le compare au code) |
| `npm run release -- X.Y.Z` | Release complete en une commande, voir `release.md` ; `tauri build` a la main reste un diagnostic |
| `cargo clean --profile dev` | Purge de `target/debug` seul (depuis `src-tauri/`) ; jamais `cargo clean` nu, qui vide aussi `target/release` |

Les drapeaux `-j 2` et `incremental = false` vivent dans `src-tauri/.cargo/config.toml` : un `cargo` lance a la main les prend sans option.

## Lint

ESLint 10 en configuration plate (`eslint.config.mjs`, a la racine), `npm run lint` = `eslint src scripts`. Aucune regle n'est en avertissement : `npm run verify` passe `--max-warnings 0` et tout ecart est une erreur. Prettier couvre `src`, `scripts` et les fichiers de configuration (`*.config.*`, `tsconfig*.json`, `components.json`).

| Regle maison | Effet | Exceptions |
|---|---|---|
| `no-restricted-syntax`, `toast.error` | passer par `reportError` ou `notifyError` | tests, `lib/reportError.ts` |
| `no-restricted-syntax`, `toISOString().slice(...)` | date du jour par `nowDate()` de `@/lib/dates` (le jour UTC est la veille entre 0 h et 2 h a Paris) | tests, `lib/reportError.ts` |
| `no-restricted-imports`, `invoke` | uniquement dans les `api.ts` (`hooks/appReady/api.ts` pour le demarrage) | `api.ts`, tests, `test-setup.ts` |
| `max-lines-per-function` 150, `complexity` 15, `max-params` 5 | tailles de R15 | tests, `test-setup.ts` |
| `no-console` | le logger central est le seul site autorise | tests, `test-setup.ts` |
| `no-empty` | un `catch {}` vide est refuse | aucune |

Ajouter une regle = un fichier temoin qui la voit echouer (recreer sous `src/`, lancer `npx eslint <fichier>`, le ranger apres). `eslint --print-config <fichier>` liste les regles actives d'un fichier. `complexity` compte aussi `?.`, `??` et les valeurs par defaut depuis ESLint 9 : extraire un sous-composant plutot que de relever le plafond.

## Outillage de test et de build (R22)

TypeScript 6.0 (`~6.0.3` : typescript-eslint 8.72 exige moins de 6.1), Vite 8 et `@vitejs/plugin-react` 6, Vitest 5 avec `@vitest/coverage-v8` et `@vitest/ui` a la meme version exacte, jsdom 29 (la 30 exige Node 24.15, le poste a 24.14), `@testing-library/jest-dom` 7, `axe-core` 4.11 (la ligne que `@axe-core/react` impose, `~4.11.3`).

- **Accessibilite en test** : `expectNoAxeViolations(container, options?)` de `@/test-utils/axe` (axe-core direct, regles de couleur coupees comme le faisait jest-axe : jsdom ne calcule aucun contraste, la garde de R08 le tient). L'erreur liste identifiant, impact, nombre de noeuds et aide.
- **Un index qui peut manquer en test** : `defined(valeur, nom)` de `@/test-utils/defined` leve une erreur nommee au lieu d'un `!`.
- **`noUncheckedIndexedAccess` est actif** (`tsconfig.json`) : un acces `tableau[i]` ou `objet[cle]` vaut `T | undefined`. Corriger par une garde, une valeur par defaut dite ou un accesseur qui rend la valeur (`resolveBandIndex` rend `band`), jamais par `!` en masse.
- **Vitest 5** : `clearMocks` vaut `true` par defaut (l'historique d'un mock est vide avant chaque test, les implementations restent) ; les matchers de jest-dom se declarent par `@testing-library/jest-dom/vitest` (importe par `test-setup.ts`), `tsconfig.json` ne liste donc plus `@testing-library/jest-dom` dans `types` ; les rapports vont dans `.vitest/` (ignore par git).
- **Un import dynamique froid se paie au premier test** : sous la charge du lot complet, le premier test qui charge un composant `lazy` a pris 3,2 a 4,6 s contre 1,1 s seul ; chauffer les modules dans un `beforeAll` a delai propre (`QuickAddButton.test.tsx`, `a11y-pages.test.tsx`), jamais rallonger le delai du test.
- **Un composant qui charge des donnees se rend dans `act` asynchrone** : une page rendue puis jugee sans attendre ses requetes affiche des avertissements `act()` et teste un squelette (`renderPage` de `a11y-pages.test.tsx`).
- **Vite 8 (Rolldown)** : `build.rolldownOptions` remplace `rollupOptions` (rien a regler ici : `experimentalMinChunkSize` n'existe plus), la config lit `import.meta.dirname` et non `__dirname` (sinon avertissement du chargeur natif). `manualChunks` reste absent. Un groupe `codeSplitting` (`minShareCount` 2, 4, 8 ou 12, `minSize` 10 ko) ramene les 82 fichiers JS a 23, 51, 66 ou 68, mais fait passer le chemin critique de 139,5 a 255,9, 246, 232,8 ou plus de 140 ko : non retenu. Rolldown ecrit l'import dynamique de page entre accents graves et pose douze `modulepreload` sur le chemin critique ; `scripts/bundle-budget-lib.mjs` lit les deux formes.

## Veille des dependances

`npm run veille` (`scripts/veille.mjs`, logique pure et testee dans `scripts/veille-lib.mjs`) est le controle d'avant release ; le format des exceptions et la regle sont dans `docs/release.md`. Elle ne remplace pas Intendance, la veille de l'ecosysteme du poste (rapport nocturne `Desktop/Routines IA/intendance/index.html`, OSV et Renovate sur tous les projets) : elle donne un resultat immediat sur l'arbre en cours, ce que la nuit ne fait pas, et relit le rapport d'Intendance pour ce qu'OSV voit en plus (`--intendance=<fichier>` ou `VEILLE_INTENDANCE_DIR` pour un autre emplacement). Le rapport d'Intendance lit la branche par defaut a l'heure de la nuit : il n'est pris en compte que s'il est posterieur au dernier commit des deux verrous (`package-lock.json`, `src-tauri/Cargo.lock`) et au tampon de l'arbre ; sinon la veille le dit en note et l'ignore. Pour verifier la veille elle-meme : `npm audit --json` dans un fichier, un avis injecte dedans, puis `node scripts/veille.mjs --audit-npm=<fichier>` doit sortir en code 1.

## Tests

- `npm run verify` tient les planchers de nombre de tests (`scripts/verify-floors.json`) et la couverture TypeScript (`couverture_ts`).
- `npm run couverture:rust` mesure la couverture Rust en une compilation instrumentee (quelques minutes) et tient `couverture_rust` et `couverture_withings_client`. Methode, mesures et procedure : `docs/couverture.md`.
- `npm run parcours` lance `npm run tauri:dev` (identifiant de dev), visite les 19 routes par CDP sur le port 9333 et echoue sur une erreur de console, une page sans titre ou vide, ou l'ecran d'erreur de route. Il ferme l'app a la fin. Depuis R24 il vérifie aussi, sur la page réelle : le titre de la fenêtre (« Page | Synk »), le focus sur le titre après chaque changement de page, le menu d'ajout rapide au clavier (Entrée, flèche bas, Échap, par de vraies touches CDP) et axe-core en clair puis en sombre sur les 19 pages (le dégradé du fond est mis à plat pour la passe, sans quoi axe ne juge pas 85 % du texte ; le texte des graphiques SVG, que axe ne sait pas juger, est compté à part ; une page dont plus de 25 % du texte reste non jugé échoue). Tout défaut axe échoue, contraste compris : `scripts/axe-contraste.json` est vide depuis R25 (plafond 0 pour chaque page et chaque mode ; une page qui aurait besoin d'un plafond le déclare par la clé `<mode> <route>`) ; la mesure complète est écrite dans `logs/parcours-axe.json`.
- Les tests Rust d'integration vivent dans un seul binaire, `src-tauri/tests/integration/` : un nouveau fichier de tests y va et se declare dans `main.rs` (`mod <nom>;`). Aides communes dans `common/` (`memory_pool()`, `test_vault()`), modules partages `support/` (faux serveur Withings) et `pdf_text/` (lecture du texte d'un PDF). Un module se lance par `cargo test --test integration -- <module>::`. `keyring_persistence.rs` reste a part (trousseau reel, a ne jamais melanger aux autres).
- La mutation (`cargo-mutants`) se lance a la main, par fichier, apres un changement de la sauvegarde, de la restauration, du chiffrement ou de l'import : `docs/mutation.md`.

## Developper sans toucher aux vraies donnees

`npm run tauri:dev` passe `src-tauri/tauri.dev.conf.json` a `tauri dev` : l'identifiant devient `com.synk.health.dev`, la fenetre s'intitule « Synk (dev) ».

| | App installee | `npm run tauri:dev` |
|---|---|---|
| Identifiant | `com.synk.health` | `com.synk.health.dev` |
| Base | `%APPDATA%\com.synk.health\synk.db` | `%APPDATA%\com.synk.health.dev\synk.db` |
| Cle maitre (Gestionnaire d'identification) | `synk` | `synk-dev` (`service_for`, `crypto/keyring.rs`) |
| Journal | `%LOCALAPPDATA%\com.synk.health\logs\` | `%LOCALAPPDATA%\com.synk.health.dev\logs\Synk.log` |

Un build de developpement qui garderait l'identifiant de production s'arrete au demarrage, avant toute ouverture de base (`ensure_not_dev_on_real_data`, `boot.rs`) : l'ecran affiche « Mode developpement refuse sur les vraies donnees ». Un `npm run tauri dev` nu tombe dans ce cas, comme un `cargo run` direct. L'app installee et le build de dev peuvent tourner ensemble (le verrou d'instance unique suit l'identifiant).

Pourquoi un refus et pas seulement une cle separee : un build de dev sur la vraie base echouerait a dechiffrer les secrets Withings et l'etape 6 du demarrage (`invalidate_undecryptable_secrets`) les effacerait de la vraie base.

## Travailler sur une copie de ses donnees

1. Fermer l'app installee.
2. Copier `%APPDATA%\com.synk.health\synk.db` vers `%APPDATA%\com.synk.health.dev\synk.db`, jamais l'inverse, depuis l'Explorateur ou un terminal ouvert hors de l'app Claude (une session de l'app ecrirait la copie dans son calque MSIX, invisible pour le reste de Windows, voir CLAUDE.md).
3. Lancer `npm run tauri:dev`. Les secrets Withings de la copie ne se dechiffrent pas avec la cle de dev : ils sont effaces de la copie au premier lancement (un bandeau l'indique), ressaisir des identifiants de test.

## Conventions

| Element | Convention | Exemple |
|---|---|---|
| Fichier page | PascalCase.tsx | `Sleep.tsx` |
| Fichier composant | PascalCase.tsx | `StatCard.tsx` |
| Fichier hook | `hooks.ts` dans le dossier de domaine ; camelCase.ts pour un hook transverse | `features/sleep/hooks.ts`, `hooks/useSync.ts` |
| Fichier utilitaire | camelCase.ts | `formatters.ts` |
| Commande Tauri | snake_case | `get_sleep_records` |
| Route frontend | kebab-case | `/blood-pressure` |
| Table SQLite | snake_case | `sleep_records` |

- Barrel exports : `@/components/shared`, `@/components/charts`, `@/hooks`.
- Dates : les tables `sleep_records`, `activity`, `workouts` stockent `YYYY-MM-DD`, les autres des ISO datetime complets. `SUBSTR(?, 1, 10)` sur le parametre seulement, jamais sur la colonne filtree (il interdit l'index).
- Les formatters (`formatDate`, `formatNumber`) rendent `"—"` sur null/undefined ; `??` ou ternaire pour les valeurs optionnelles.

## Ajouter un domaine de donnees

Le modele est `src/features/sleep/` (`api.ts`, `types.ts`, `hooks.ts`, `components/`) et `src-tauri/src/commands/sleep.rs` ; les patterns (feature-folder, `createCrudHook`, `createManualForm`) sont decrits dans `CLAUDE.md`.

1. **Commande Rust** : `src-tauri/src/commands/<domaine>.rs` (`#[tauri::command]`, requete par `SUBSTR(date, 1, 10)`, `Result<T, AppError>`), declaree dans `commands/mod.rs`. Les ecritures visent les tables `*_legacy`, jamais les vues.
2. **Enregistrement** : la commande va dans `tauri::generate_handler![...]` de `src-tauri/src/lib.rs`.
3. **Frontend** : `src/features/<domaine>/api.ts` (les `invoke`), `types.ts`, `hooks.ts` (`createCrudHook` pour un CRUD simple, queryKey `[domaine, ...]`).
4. **Formulaire** : `createManualForm` dans `src/features/<domaine>/components/`.
5. **Affichage** : dans une page existante. Une nouvelle page (route dans `App.tsx`, entree dans `NAV_SECTIONS` de `src/lib/constants.ts`) seulement sur demande de l'utilisateur.
6. `npm run verify`, puis le controle dans `npm run tauri:dev`.

## Erreurs entre Rust et l'interface

Toute commande rend `Result<T, AppError>` (`src-tauri/src/error.rs`). `AppError` se serialise en `{ kind, message, detail }` : `kind` est le nom de la variante (`Db`, `Validation`, `NotFound`, `Json`, `Forbidden`, `Internal`, `Http`, `Unauthorized`, `RateLimit`, `Crypto`), `message` le texte des journaux (« Erreur de validation : ... »), `detail` la charge de la variante. `invoke` rejette avec cet objet ; `formatError` (`src/lib/errors.ts`) le lit par son `kind` et rend `{ code, message, userMessage }`. Les chaines des plugins (dialog, opener) et les erreurs JavaScript passent encore par des expressions regulieres. Pour afficher une erreur dans une phrase, `errorText(e)` : jamais `${e}`, qui ecrirait `[object Object]`.

Ajouter une variante :

1. `error.rs` : la variante, son `#[error(...)]`, sa ligne dans `kind()` et dans `detail()`.
2. `src/lib/errors.ts` : le `kind` dans `RustErrorKind` et sa phrase dans `BY_KIND`.
3. `npx vitest run src/lib/errors.contract.test.ts` : le contrat lit `error.rs` et echoue si une variante n'a ni `kind` ni phrase.

Une erreur d'une autre bibliotheque se convertit par `?` quand `From` existe (`sqlx`, `serde_json`, `reqwest`, `std::io`, `tauri`, `csv`), sinon `AppError::Internal(format!("<contexte> : {e}"))` ; une entree refusee est une `Validation`. Jamais `map_err(|e| e.to_string())` dans une commande. Un evenement Tauri qui porte une erreur envoie `AppError::into_event_payload()` (masque les secrets).

Verbes des commandes : `get_` lit sans effet, `add_` cree, `update_` modifie, `delete_` supprime, `upsert_` cree ou remplace, `set_` regle un singleton, `log_` ajoute un evenement, `evaluate_` calcule et ecrit un resultat (`evaluate_alerts`).

## Evenements et forme des charges (R43)

- Un evenement emis par Rust a son nom dans `src/features/sources/events.ts` ; `events.contract.test.ts` lit les sources Rust et echoue si un nom emis n'est pas declare. Pour une source : `<source>-auth-success`, `-auth-error`, `-auth-timeout`.
- Convention pour toute charge neuve : cles en anglais, `snake_case`. `SyncProgress` (`etape`, `libelle`) est historique et garde ses cles francaises ; `ImportProgress` suit la convention.
- Une source fichier declare son lecteur par `Source::import` (aucune edition de `services/import.rs`) ; une source OAuth neuve a son client REST en declarant un `Brand` (`sources/oauth/brand.rs`) et en appelant `RestClient`.

## Ajouter une source

Une source OAuth (modele : `src-tauri/src/sources/polar/`, la plus petite) :

1. Un dossier `sources/<marque>/` : `spec.rs` (l'`OAuthSpec` : URL d'autorisation, hote et chemin du jeton, portees, PKCE, parametres), `client.rs` (un `Brand` et un `RestClient`), `types.rs` (serde strict sur les dates), `mapping.rs` (pur, teste sans reseau), `persist.rs`, `window.rs`, `sync.rs` (etapes, curseurs avances apres l'ecriture), `revoke.rs` si la marque offre une revocation, et `source.rs` (l'implementation de `Source`).
2. Une ligne dans le tableau `SOURCES` de `sources/mod.rs` et une ligne dans la table `sources` par une migration : un test refuse l'une sans l'autre.
3. Le libelle dans `src/features/sources/labels.ts` (un test le compare a la graine de la migration), l'hote de la marque dans `capabilities/default.json` (`opener:allow-open-url`), le portail developpeur dans `developer_url`.
4. Les metriques neuves par `Source::metric_definitions` ; les tests contre un faux serveur (`tests/integration/support/`), jamais contre la marque.

Une source fichier declare un lecteur par `Source::import` et un `FileSpec` (extensions) ; le pipeline `sources/import/pipeline.rs` fournit le canal, la transaction unique et l'annulation. Details par marque : [api-sources.md](api-sources.md).

## Ajouter un type de mesure Withings

1. `src-tauri/src/sources/withings/mapping/measures.rs` : constante `MEASURE_XXX` et inscription dans `BODY_TYPES`, `BP_TYPES` ou `VITAL_TYPES` (un test verifie que chaque liste ne contient que ses types).
2. Les fonctions de classement du meme fichier (`is_body_composition`, `is_blood_pressure`) ou, pour un vital, `vital_type_name` dans `mapping/vitals.rs`.
3. Le module de synchro correspondant sous `sources/withings/sync/measures/` (`body.rs`, `bp.rs`, `vitals.rs`) ; `sync.rs` en assemble la liste `meastypes` depuis les constantes ci-dessus.
4. Un nouveau `vital_type` n'a plus de `CHECK` a modifier (`vitals_legacy` n'en porte plus depuis la migration 020) : il faut une definition de metrique (migration comme la 013, ou `Source::metric_definitions`) et sa ligne dans `metric_repository::classify`.

## Ajouter une mensuration integree

Editer `src/features/bodyProfile/catalog.json` : c'est la source unique, lue par Vite et par `body_catalog.rs`. `seed_body_catalog` (`db/seed.rs`) la rejoue a chaque demarrage. Jamais une migration, jamais `001_init.sql` : sur une base existante `measurement_types` est une vue, et l'insertion echoue sans bruit.

## Deboguer la synchro Withings

1. Lancer `npm run tauri:dev` et synchroniser ; un toast « Sync partielle » liste les erreurs.
2. Journal Rust : `%LOCALAPPDATA%\com.synk.health.dev\logs\Synk.log` (et la console du terminal).
3. Table `sync_log` : une ligne par synchro (type, duree, lignes ajoutees, statut, erreur). Table `error_log` : erreurs remontees a l'interface.
4. Erreurs courantes :
   - `JSON parse error` : l'API Withings renvoie un format inattendu, ajouter un deserializer tolerant (`docs/api-withings.md`).
   - `FOREIGN KEY constraint failed` : une valeur de `source` absente de la table `sources` (elle se declare dans le registre ET dans `sources`).
   - `no rows returned` : un curseur manque dans `source_state` (6 domaines Withings attendus), rejoue par `run_post_ready_seed`.
5. Inspecter une base de dev : DB Browser for SQLite sur `%APPDATA%\com.synk.health.dev\synk.db`, app fermee.

## Securite

- La CSP (`app.security.csp` de `tauri.conf.json`) ne s'applique qu'aux builds release : sous `npm run tauri:dev` la page est servie en direct et la CSP n'a aucun effet. Elle doit laisser passer l'IPC (`ipc: http://ipc.localhost`), faute de quoi Tauri retombe sur `postMessage` avec l'avertissement « IPC custom protocol failed ». Verification : `CARGO_BUILD_JOBS=1 npx tauri build --no-bundle --config src-tauri/tauri.dev.conf.json` puis `npm run parcours -- --exe=src-tauri/target/release/synk.exe` (le build remplace `target/release/synk.exe` par une copie de dev : ne jamais la copier ni l'installer).
- Permissions de la fenetre : `capabilities/default.json`, gardees par `src/lib/security-config.contract.test.ts` (dialog limite a `allow-open`, ni shell ni `open-path`).
- ACL des commandes applicatives ecartee : Synk n'a qu'une fenetre, sans contenu distant, et elle a besoin de toutes les commandes de [commandes.md](commandes.md) ; une ACL lui accorderait tout et ne retirerait rien a un script injecte. A rouvrir avec une seconde fenetre ou un contenu distant.
- Jamais de jeton ni de corps de reponse Withings dans une erreur : `utils::redact` masque les valeurs sensibles, les erreurs ne portent que la taille et la position.

## Metrique unifiee

La table `metrics`, ses definitions, le repository (`metric_repository.rs`) et le choix de la source par jour (`services/metrics`) sont decrits dans [`metrics-v2.md`](./metrics-v2.md).

## Voir aussi

- [`architecture.md`](./architecture.md), [`database.md`](./database.md), [`frontend.md`](./frontend.md)
- [`release.md`](./release.md) : build NSIS, distribution
- [`api-sources.md`](./api-sources.md), [`decisions.md`](./decisions.md), [`pieges.md`](./pieges.md), [`commandes.md`](./commandes.md)
- [`faq.md`](./faq.md) — troubleshooting utilisateur
