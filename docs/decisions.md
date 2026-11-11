# Décisions techniques

Chaque entrée est une décision qui tient le code d'aujourd'hui : la règle, la raison, et le test ou le fichier qui la garde. Les identifiants (D01 à D84) sont stables ; `CLAUDE.md` en reprend l'essentiel et renvoie ici pour le détail. Un récit de chantier plus long vit dans [historique-developpement.md](historique-developpement.md) ou dans les fiches de `docs/plans/`.

<a id="d01"></a>
## D01. Dates SQL

`SUBSTR(?, 1, 10)` pour normaliser ISO datetime vs YYYY-MM-DD cote parametre, jamais sur une colonne filtree (il interdit l'index, R27) ; ecriture en `AAAA-MM-JJ` ou ISO UTC avec `Z`

<a id="d02"></a>
## D02. Deserialization tolerante

`deserialize_more` (bool/int), `deserialize_lenient_i64` (ignore strings/objects), `deserialize_heart_rate` (int ou struct)

<a id="d03"></a>
## D03. bootstrap_schema

(`db/bootstrap.rs`) : applique le registre `MIGRATIONS` (22 entrees, `LATEST_VERSION` deduit, un fichier non enregistre echoue `tests/integration/schema.rs`) + `migrate_to_v2_schema` (rename/vues *_legacy, passe avant la 019 qui en depend, puis rejoue) a chaque demarrage ; idempotent via schema_version. `MIGRATION_001` y est rejouee a chaque lancement : ses echecs attendus (objet deja present, INSERT ou CREATE INDEX sur une table devenue vue) partent en `debug` via `migration_001_replay_is_benign`, tout autre en `warn`. Garde : `tests/integration/migration_atomicity.rs` rejoue la migration sur une base a jour et exige que chaque echec soit reconnu (l'oubli de « views may not be indexed » donnait 5 faux avertissements par demarrage)

<a id="d04"></a>
## D04. printpdf 0.12, une liste d'operations par page

`pdf/canvas.rs` (`Canvas`) tient le `PdfDocument` et une `Vec<Op>` par page ; `use_text` ouvre et ferme son propre bloc `BT`/`ET` (le `Td` de printpdf est relatif : deux textes dans un meme bloc additionneraient leurs positions). `save` ecrit sans compression (`optimize: false`, les tests lisent les octets) et transforme tout avertissement de severite `Error` en `Err`, jamais avale. Collecte async (sqlx) toujours AVANT le rendu synchrone. Migration depuis 0.7 le 2026-10-22 : lopdf 0.31 (RUSTSEC-2026-0187) et memmap2 0.6 sortis de l'arbre

<a id="d05"></a>
## D05. Mapping appareils

`friendly_device_name(model_id)` traduit les IDs en noms commerciaux

<a id="d06"></a>
## D06. Composants shared

StatCard, PageHeader, LoadingSkeleton eliminent les duplications

<a id="d07"></a>
## D07. Barrel exports

`index.ts` dans shared/, charts/, forms/, hooks/

<a id="d08"></a>
## D08. NAV_SECTIONS

Sidebar organisee en sections thematiques (Corps, Activite, Suivi, Analyse)

<a id="d09"></a>
## D09. Hook partage useSync

Logique de synchronisation centralisee dans `hooks/useSync.ts` ; l'invalidation passe par la liste `SYNC_INVALIDATED_KEYS` (clés ciblées, jamais une invalidation globale).

<a id="d10"></a>
## D10. Auto-sync

Au lancement, compare la derniere synchro reussie (`get_last_successful_sync`, lue dans `sync_log`) a `syncInterval` du store ; elle se lance aussi sur `<source>-auth-success`, de n'importe quelle source (ecoute par `useSourceEvents` dans `AppLayout`)

<a id="d11"></a>
## D11. Connexion Withings (R03, 2026-10-22)

`sources::withings::auth::is_connected_inner` juge sur le jeton de rafraichissement (un an), jamais sur le jeton d'acces (3 heures, rafraichi par `get_valid_client`). La synchro emet `sync-progress` (`{ source, etape, total, libelle }`, 6 etapes pour Withings, R35) ecoute par `useSync`

<a id="d12"></a>
## D12. Client Withings

Retry exponentiel 3x (1s/2s/4s), Semaphore(4) pour rate limiting, auto-refresh token sur 401, timeout 30s

<a id="d13"></a>
## D13. OAuth callback server

Boucle avec recv_timeout, ignore favicon et requetes hors rappel, delai de 120 s, emet `<source>-auth-success`, `<source>-auth-error` ou `<source>-auth-timeout` (voir P26).

<a id="d14"></a>
## D14. Sync log

Chaque type de sync est enregistre dans `sync_log` avec duree_ms, records_added, status, error

<a id="d15"></a>
## D15. Insights auto

Generes une seule fois par `sync_sources` apres la derniere source (au moins une reussie), plus dans la synchro Withings (R35) ; etape de progression `analyses`, 5 detecteurs (poids tendance, sommeil insuffisant, FC anomalie, activite delta, hypertension)

<a id="d16"></a>
## D16. ErrorBoundary

Wrapper global React qui affiche un message + bouton Recharger au lieu d'un ecran noir

<a id="d17"></a>
## D17. sqlx 0.9 : un SQL construit se marque (R21)

`query*` n'accepte que des `&'static str` ; une requete formee par `format!` passe par `sqlx::AssertSqlSafe(sql)` apres lecture de ce qu'elle interpole (identifiants du code : constantes de `CrudTable`, colonnes du `PRAGMA table_info`, noms figes ; jamais une valeur de l'utilisateur, qui se lie en parametre). `run_versioned_migration` prend un `&'static str`. libsqlite3-sys reste 0.30.1 (SQLite 3.46) ; la 0.37 (SQLite 3.51) est au backlog.

<a id="d18"></a>
## D18. Taille du bundle (R11, 2026-10-22, premier ecran R20)

Tenue par `npm run budget`, qui mesure l'entree, le chemin critique et le premier ecran reel (entree plus tableau de bord et ses importations statiques) ; il est dans `npm run verify` (build puis budget, une dizaine de secondes), plafonds dans `scripts/bundle-budget.json`, releves quand un gain est acquis ; framer-motion retire, les entrees animees sont en CSS (`animate-fade-in-up`) ; pas de `manualChunks` ; l'assistant de premier lancement et les formulaires de saisie rapide sont charges a la demande (`lazy` et `Suspense`, jamais re-exportes par le barrel `shared/index.ts`)

<a id="d19"></a>
## D19. Hydration separee

Le % d'hydration n'est plus dans BodyCompositionChart (evite le melange % et kg sur le meme axe)

<a id="d20"></a>
## D20. INSERT/UPDATE/DELETE pointent sur `*_legacy`

Apres `migrate_to_v2_schema`, `body_measures`, `manual_measurements`, `vitals`, `measurement_types` deviennent des VUES de retrocompat (V1.0.1). SQLite refuse les ecritures sans trigger INSTEAD OF — le code ecrit donc directement dans `body_measures_legacy`, etc. Les SELECT restent transparents (vues).

<a id="d21"></a>
## D21. Refresh token serialise par source

`oauth::refresh::lock_for(source)` rend un `tokio::sync::Mutex` par source (R32) ; il evite que 2 requetes simultanees en 401 invalident mutuellement le refresh_token (Withings revoque l'ancien a chaque echange). Apres acquisition du lock, re-lecture du magasin pour court-circuiter si un autre appel a deja rafraichi.

<a id="d22"></a>
## D22. Sauvegarde par `VACUUM INTO`, verifiee avant d'exister

Toute copie de la base passe par `copy_database_safely` (`services/backup.rs`). Elle execute `VACUUM INTO ?` (chemin lie en parametre, jamais interpole ; commande apparue en SQLite 3.27.0, la crate `libsqlite3-sys` 0.30.1 embarque la 3.46.0) vers `<dest>.partial`, cree vide et en creation exclusive (deux sauvegardes vers la meme cible ne partagent jamais ce fichier), rouvre cette copie seule en lecture seule, exige que `PRAGMA integrity_check` rende une seule ligne `ok` et qu'elle contienne au moins une table (un fichier vide passe le controle d'integrite), puis la renomme en `<dest>` (`tests/integration/backup_guards.rs`). `VACUUM INTO` lit un instantane coherent, sans checkpoint prealable ni copie des `-wal`/`-shm`, et produit une base en journal rollback. Toute erreur remonte a l'appelant avec le chemin et la cause, et le `.partial` est supprime ; son extension le tient de toute facon hors des listes `.db` de rotation et de restauration. Un fichier cible deja present est refuse, jamais ecrase. Trois appelants : `create_backup` (par `create_backup_in`, pool applicatif), `boot::periodic_backup_inner` (pool applicatif) et `boot::backup_before_migration` (`None`, une connexion ephemere est ouverte). Remplace le couple `PRAGMA wal_checkpoint(TRUNCATE)` puis `std::fs::copy` : un checkpoint refuse par un lecteur ouvert donnait un `.db` illisible seul, prouve par `a_backup_taken_while_a_write_is_pending_restores_every_committed_row` (`tests/integration/backup_integrity.rs`, rouge contre l'ancienne recette, vert sur `main`). Une copie pendant un checkpoint concurrent pouvait aussi donner un fichier dechire, mais aucun test ne le prouve : `a_backup_taken_during_concurrent_writes_is_a_consistent_snapshot` reste vert contre l'ancienne recette (course, constat du 2026-10-17).

<a id="d23"></a>
## D23. Cast f64->i64 borne

Dans `upsert_bp_group`, valeurs API clampees `(0.0..=500.0).contains(&raw)` avec rejet + warn log avant `as i64` (evite undefined behavior sur NaN/Infinity).

<a id="d24"></a>
## D24. gradient ID Recharts via useId

`GenericMetricChart` utilise `useId()` React au lieu du slug pour generer les ID de gradient SVG — evite collisions DOM si plusieurs charts du meme slug montes.

<a id="d25"></a>
## D25. Backend modulaire `commands/export/` (V1.2.0)

`commands/export.rs` (1043 lignes) split en `csv.rs` / `pdf.rs` / `json.rs` / `backup.rs` / `shared.rs` + `mod.rs`, puis `restore.rs` sorti de `backup.rs` le 2026-09-28 ; depuis R28 la logique de sauvegarde vit dans `services/backup.rs` et `commands/export/backup.rs` n'est plus qu'un adaptateur. Le `mod.rs` utilise `pub use module::*` (glob, pas re-exports nommes) car `tauri::generate_handler!` genere des `__cmd__<name>` caches que les re-exports nommes n'attrapent pas. Ne PAS nettoyer en re-exports nommes : ca casse le build.

<a id="d26"></a>
## D26. Frontend modulaire `lib/api/` et `lib/types/` (V1.2.0)

`lib/db.ts` et `lib/types.ts` ont ete decoupes par domaine en V1.2.0, puis remplaces par les dossiers de domaine (D34). Il ne reste aucun `@/lib/db`, `@/lib/types`, `@/lib/api/*` ni `@/lib/types/*` : un import vers l'un d'eux est une erreur.

<a id="d27"></a>
## D27. Factory `createCrudHook` (V1.2.0)

5 domaines (`bodyMeasures`, `bloodPressure`, `sleep`, `activity`, `workouts`) tirent leurs hooks de `createCrudHook<T, AddInput>` (`hooks/createCrudHook.ts`) : `useList` (cle `[domain, from, to]`), `useLatest` (cle `[domain, "latest"]`), `useAdd`, `useDelete`. Pour ajouter un domaine CRUD : `features/<domaine>/api.ts` puis l'appel de la factory dans `features/<domaine>/hooks.ts`.

<a id="d28"></a>
## D28. Sections de pages > 300 lignes (V1.2.0)

`Dashboard`, `Analyses`, `Labs`, `Medications`, `Alerts` et `Profile` decomposes en `pages/<Page>/sections/<Section>.tsx` + helpers purs dans `pages/<Page>/sections/helpers.ts`. La page parent reste un orchestrateur (~80-130 lignes) qui rend les sections en sequence.

<a id="d29"></a>
## D29. Modules Rust scindes round 2 (V1.3.0)

`commands/manual_measurements/` (types/measurements/stats/goals), `services/insights/` (avec `detectors/` par type d'insight : weight_trend, sleep_insufficient, heart_rate_anomaly, activity_delta, hypertension ; la commande est un adaptateur dans `commands/insights/`), `sources/withings/mapping/` (measures/vitals/workouts). Chaque `mod.rs` utilise `pub use *` (glob) pour preserver les `__cmd__<name>` Tauri.

<a id="d30"></a>
## D30. Module `db/` scinde (2026-10-03)

`db.rs` (453 lignes) devient `db/pool.rs` (`init_pool`), `db/bootstrap.rs` (le registre `MIGRATIONS` et `bootstrap_schema`), `db/maintenance.rs` (`PRAGMA optimize`), `db/migrations.rs` (`MigrationError`, `run_versioned_migration`), `db/legacy.rs` (tables `*_legacy`, vues de retrocompat, reprise des donnees V1) et `db/seed.rs` (catalogue, curseurs de `source_state`, `run_post_ready_seed`) ; `db/gate.rs` (portail `LongWrite`, D80) s'y est ajoute. `mod.rs` re-exporte par noms et non par glob : ce module ne porte aucune commande Tauri, donc aucun `__cmd__` a preserver. Les chemins `crate::db::*` et `synk_lib::db::*` sont inchanges.

<a id="d31"></a>
## D31. Factory `createManualForm` (V1.3.0)

`components/forms/manual/createManualForm.tsx` (sorti de `lib/forms/` en R14 : `lib` ne depend d'aucun composant, les dates sont dans `lib/dates.ts`) factorise 5/6 formulaires `Manual*Form` via une config typee `FormConfig<TInput>` (champs, validations, submit, success message, invalidateKeys). 6e formulaire (`ManualMeasurementForm`) reste custom car options dynamiques runtime + props additionnelles. Pattern : pour ajouter un nouveau formulaire CRUD simple, appeler `createManualForm({ fields, submit, ... })`.

<a id="d32"></a>
## D32. OnboardingWizard decompose (V1.3.0)

4 steps (Welcome, Sources, Profile, Goals ; l'etape Sources, `SourcesStep`, liste les sources du registre et se passe sans en brancher aucune, R34) dans `components/shared/OnboardingWizard/steps/`. Layout commun `shared/StepLayout.tsx`. `index.tsx` orchestre les transitions. `SourceConnect` porte la connexion d'une marque : ecoute de `<source>-auth-success|error|timeout` par `useOAuthFlow`, annulation et nouvel essai.

<a id="d33"></a>
## D33. useAppReady en sous-hooks (V1.3.0)

3 sous-hooks dans `hooks/appReady/` (`useBootListeners` Promise.all([listen]) + cleanup, `useBootSnapshot` invoke get_boot_state apres attache, `useBootWatchdog` timeout 12s + last-chance). `index.ts` orchestre, `useAppReady.ts` reste comme barrel re-export.

<a id="d34"></a>
## D34. Pattern feature-folder (V1.4.0)

Chaque domaine metier vit dans `src/features/<domaine>/` avec `api.ts` (invokes), `types.ts` (interfaces), `hooks.ts` (React Query hooks), et `components/` (forms specifiques). Co-location = quand on touche au domaine sleep, tout est dans `features/sleep/`. `lib/db.ts`, `lib/types.ts`, `lib/api/`, `lib/types/` SUPPRIMES. Pour ajouter un nouveau domaine : creer `features/<X>/api.ts` + appeler `createCrudHook` dans `features/<X>/hooks.ts` + ajouter form via `createManualForm` dans `features/<X>/components/`.

<a id="d35"></a>
## D35. Chiffrement OAuth (V2.8.0)

Les colonnes sensibles de chaque source (`source_auth.access_token` et `refresh_token`, `source_credentials.client_id` et `client_secret` ; `withings_auth` et `withings_credentials` ne sont plus que des vues depuis la migration 019) sont chiffrees au repos via AES-256-GCM. Cle maitre 32 bytes stockee dans le Gestionnaire d'identification Windows par `keyring-core` 1 et `windows-native-keyring-store` 1.1 (R21, 2026-10-23, a la place de `keyring` 3) : identifiant generique de cible `master_key.synk` (`{user}.{service}`), persistance `Enterprise`, identiques a ceux de keyring 3.6.3, donc la cle ecrite avant la montee se relit (temoin `synk-test-migration-r21`, `logs/r21/temoin.txt`). Le magasin est pose explicitement par `init_store` (`crypto/keyring.rs`) ; sans magasin par defaut, `Entry::new` echoue au lieu de se replier sans erreur sur un magasin en memoire, defaut qui regenerait la cle a chaque lancement jusqu'au correctif du 2026-09-13 (log `master key generated and stored in keyring (first run)` a chaque demarrage). Garde-fou : `tests/keyring_persistence.rs` (une seconde instance `Entry` doit relire l'entree, cible et persistance fixees). Format ciphertext `enc:v1:<base64(nonce||ct||tag)>`, tenu par un vecteur de reference calcule hors de Synk (`crypto/cipher.rs`, `opens_the_reference_vector`) qui a survecu a aes-gcm 0.10 vers 0.11 ; l'aleatoire vient de `getrandom::fill` (une panne d'entropie est une `AppError::Crypto`, jamais une panique). Migration transparente au boot (idempotente, transactionnelle). `open_str` refuse une valeur non chiffree (R17), vide exceptee ; la migration du demarrage chiffre tout secret en clair avant la premiere lecture.

<a id="d36"></a>
## D36. Secret `enc:v1` indechiffrable = invalide au boot, pas au fil de l'eau

L'etape `invalidate_undecryptable_secrets` (`crypto/recovery.rs`, step 6/7) tente de dechiffrer les colonnes de `source_auth` et de `source_credentials`, pour toutes les sources, avec la cle courante, dans une transaction. Tokens illisibles (l'un ou l'autre) -> ligne de `source_auth` supprimee ; credentials illisibles -> `client_id`/`client_secret` remis a `''`. Les curseurs de `source_state` et les mesures sante ne sont pas touches. Un echec SQL de l'etape reste fatal (`BootError::CryptoMigrationFailed`), un echec de dechiffrement ne l'est pas. Le rapport `InvalidatedSecrets { sources: [{ source, label, tokens, credentials }] }` est garde en `tauri::State` pour la session et lu par la commande `get_invalidated_secrets` : toast au demarrage (`AppLayout`) et carte d'etapes en tete de Parametres > Sources (`ReconnectNoticeCard`), qui disparait etape par etape. Le rapport ne survit pas au redemarrage : au lancement suivant, l'utilisateur voit seulement « Non connecte ». Tests : `tests/integration/crypto_recovery.rs`.

<a id="d37"></a>
## D37. CryptoVault injecte via tauri::State<Arc<CryptoVault>>

Instancie une fois au boot (step 5/7), accessible depuis toutes les commandes Tauri qui en ont besoin. WithingsClient stocke un `Option<Arc<CryptoVault>>` via `with_vault()` builder.

<a id="d38"></a>
## D38. Catalogue partage TS/Rust (feature bodyProfile)

`src/features/bodyProfile/catalog.json` (47 mensurations : identite, zone, nature, bornes, aide a la mesure, ancre d'illustration) est la source unique de verite, importee par Vite cote React et lue via `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../src/features/bodyProfile/catalog.json"))` cote Rust (`body_catalog.rs`). Evite que les coordonnees d'ancrage divergent entre la fiche ecran (`BodyFigure`) et le PDF exporte.

<a id="d39"></a>
## D39. Seed du catalogue de mensurations = fonction Rust, pas migration SQL

`seed_body_catalog` (`db/seed.rs`) seede les 47 types built-in APRES `migrate_to_v2_schema`, en ciblant `measurement_types_legacy`. Impossible via une migration versionnee classique : au moment ou une migration versionnee tourne, `measurement_types` peut deja etre la vue de retrocompat (bases existantes) et SQLite refuse `INSERT` dans une vue. Ce seed lui-meme n'ajoute aucune migration versionnee (`EXPECTED_LATEST_VERSION` egale `LATEST_VERSION`). Remplace l'ancien `seed_builtin_inseam` (feature tailles de vetements, seedait 1 seul type). Idempotent (`slug` UNIQUE -> INSERT OR IGNORE puis UPDATE des metadonnees) avec garde-fou anti-collision : si le slug appartient deja a un type personnalise (`is_builtin = 0`), le seed logue un `warn` et ne touche pas ce slug.

<a id="d40"></a>
## D40. Export PDF de la fiche de mensurations

`commands/export/pdf/sheet.rs` pose les deux figures de `src/assets/body/` (`front.png`, `back.png`, lues par `pdf/figures.rs` via `include_bytes!`) puis les libelles, les traits de rappel et les pastilles en Rust (`use_text`, `add_polyline`, `fill_discs`), places par `pdf/callouts.rs` (D84), jamais dans l'image. Frontiere async/sync preservee : fetch sqlx (`get_body_sheet_inner`) avant le rendu synchrone. La chaine SVG des versions 2.15 a 3.0.1 (`Svg::parse`, silhouettes pre-converties dans `pdf/figures.json`, `npm run figures:gen`) est remplacee par D83 : `Svg::parse` chargeait toutes les polices systeme a chaque appel, 9 s en release sur ce poste, et la feature `svg` tirait azul-layout et des dizaines d'autres crates.

<a id="d41"></a>
## D41. Agregation unique de la carte d'identite (feature profileCard)

`commands::profile_card::get_profile_card_inner` rassemble 5 blocs (corps, mensurations cles, traitements en cours, tailles, records) depuis 5 requetes paralleles via `tokio::try_join!` (profil age/sex/height_cm, derniere pesee, mensurations cles, traitements actifs, tailles) + le fetch des records. L'ecran (`pages/Profile.tsx`) et l'export PDF (`export_profile_card_pdf`) appellent tous les deux cette meme fonction : ils ne peuvent pas diverger.

<a id="d42"></a>
## D42. Records via `ORDER BY value LIMIT 1`, jamais `MIN()`/`MAX()`

(`commands/profile_card/records.rs`) : un agregat SQL donne la bonne valeur mais perd la date a laquelle elle a ete atteinte (ex : poids minimum affiche avec une date fausse, sans erreur visible). Chaque record (poids min/max, meilleur total de pas, plus longue nuit, meilleur score de sommeil, FC la plus basse, entrainement le plus long) est lu via une requete `SELECT valeur, date ... ORDER BY ... LIMIT 1` dediee.

<a id="d43"></a>
## D43. La carte ne contient que ce que Synk peut deriver lui-meme

Decision utilisateur du 2026-08-02, la carte d'identite a perdu ses 9 champs saisis a la main (date de naissance, lateralite, couleurs des yeux/cheveux, groupe sanguin, allergies, correction visuelle, antecedents, vaccins) — plus de formulaire, plus de commande d'ecriture. Poids, mensurations, tailles et traitements restent **lus** depuis leurs tables sources (`body_measures`, `manual_measurements`, `clothing_sizes`, `medications`) ; age/sexe viennent de `user_profile` (colonnes historiques, page Parametres). Les 9 colonnes de la migration 015 restent en base (**inutilisees**, jamais relues par le code) : migration additive, la supprimer apporterait du risque pour aucun benefice.

<a id="d44"></a>
## D44. Une seule commande d'ecriture sur `user_profile`

`update_user_profile` (page Parametres) ecrit `age`/`sex`/`height_cm`. `update_profile_identity` a existe brievement sur cette branche puis a ete supprimee avec ses 9 colonnes cibles.

<a id="d45"></a>
## D45. Tailles estimees persistees

(`clothing_sizes`, migration 016) : `save_estimated_sizes` ecrit avec `source = 'estimated'` via `ON CONFLICT ... DO UPDATE ... WHERE clothing_sizes.source = 'estimated'` — une nouvelle estimation ne remplace qu'une estimation precedente, jamais une ligne `manual`/`override` deja corrigee par l'utilisateur.

<a id="d46"></a>
## D46. Assistant de releve sans pre-remplissage

Le wizard affiche la derniere valeur connue en texte fantome (`placeholder`), jamais comme contenu du champ (`value`). Pre-remplir revenait a reenregistrer toutes les mesures deja connues a la date du jour des qu'une seule etait saisie (l'ecriture couvrait toute valeur non vide, y compris celles jamais retapees) : dates falsifiees, courbes d'evolution aplaties par les doublons. Le champ reste vide tant que rien n'est tape ; seul ce qui est reellement saisi part vers `add_measurement_session`.

<a id="d47"></a>
## D47. Planche anatomique : figures et reperes

Les deux figures de `src/assets/body/` ne sont plus tracees ni generees (D83) : `scripts/gen-body-plate.js` (Catmull-Rom -> bezier cubique, symetrie par miroir) est retire avec le dessin vectoriel, et les ordonnees de `plate-landmarks.json` sont mesurees sur les images puis tenues par `npm run plate:check`.

<a id="d48"></a>
## D48. Icones generees, jamais editees a la main (2.15.0)

`npm run icons:gen` (`scripts/build-icons.mjs`) rend les trois SVG de `src-tauri/icons/source/` par `tauri icon` et assemble `icon.ico` a la main : 16/20/24 px depuis `synk-icon-16.svg` (pouls seul), 32/40 depuis `synk-icon-32.svg` (anneau redessine sur la grille de 32), 48/64/256 et tout le jeu PNG depuis `synk-icon.svg`. `tauri icon` seul deduirait toutes les tailles du grand dessin, illisible en 16 px. Le composant `components/brand/SynkLogo.tsx` reprend le dessin de 32 px ; sa tuile suit la couleur d'accent choisie, l'icone Windows reste indigo. Favicon : `public/synk.svg`, copie par le script. Le meme script compose les deux BMP de l'installeur (`scripts/nsis-images.mjs`) : `nsis-sidebar.bmp` (164 x 314, degrade de la tuile et marque blanche, pages Bienvenue et Fin) et `nsis-header.bmp` (150 x 57, tuile calee a droite contre le titre). Ils sont branches dans `bundle.windows.nsis` avec `installerIcon` et `languages: ["French"]` ; `bundle.copyright` remplit le pied de l'installeur et le `LegalCopyright` des deux exe (vide, NSIS y affiche son propre nom). Le desinstalleur recoit la meme icone et le meme bandeau (`uninstallerIcon`, `uninstallerHeaderImage`, CLI 2.11 et plus).

<a id="d49"></a>
## D49. Demarrage relancable (2.15.0)

`boot/runner.rs` porte le snapshot, `spawn_boot` et deux commandes (`get_boot_state`, `retry_boot`). `retry_boot` refuse si le boot n'est pas en echec (un second boot ouvrirait un second pool) ; l'etat Tauri (pool, coffre, secrets invalides) n'est gere que sur un boot reussi, donc jamais deux fois. L'ecran d'erreur appelle `retry_boot` puis recharge la page : recharger seul rejouait l'echec fige dans le snapshot.

<a id="d50"></a>
## D50. Pas de migration sans sauvegarde (2.15.0)

Un echec de `backup_before_migration` est fatal (`BootError::BackupFailed`), l'ecran propose la restauration. Avant, il etait journalise et la migration partait sans filet. La sauvegarde ne s'execute que si une migration est due, donc l'erreur ne bloque jamais une base a jour.

<a id="d51"></a>
## D51. Sauvegardes et ouvertures de dossier (R05, 2026-10-22)

`tauri-plugin-opener` `~2.5.5` remplace le plugin shell (la 2.6 et la 2.7 exigent `tauri ^2.12`) ; capacite `opener:allow-open-url` limitee aux hotes des marques et de leurs portails developpeur (liste dans `capabilities/default.json`, tenue par `security-config.contract.test.ts`), `opener:allow-reveal-item-in-dir`, jamais `open-path` (qui lance un programme). Les liens `target="_blank"` passent par le script d'initialisation du plugin. `boot::periodic_backup_inner` ecrit dans `backups/daily/` (14 gardees, 20 h minimum) apres le seed ; l'echec va dans `error_log`, l'evenement `backup-failed` ET `BackupFailure` (etat lu par `get_backup_failure` au montage d'`AppLayout`), parce que l'evenement part avant que la webview ecoute. `all_backups_newest_first` (`services/backup_files.rs`) sert la liste de Parametres et la restauration.

<a id="d52"></a>
## D52. Restauration verifiee (2.15.0)

`restore_latest_backup_inner` (async) copie la sauvegarde la plus recente vers `synk.db.restoring`, la passe a `verify_backup`, ecarte et nomme dans `RestoreReport.skipped` celles qui echouent, puis seulement deplace la base courante et ses `-wal`/`-shm` et renomme la copie ; si le renommage echoue, la base courante est remise en place. Une sauvegarde illisible ne remplace jamais la base (`tests/integration/restore_backup.rs`, les deux gardes passent au rouge si `verify_backup` est neutralise).

<a id="d53"></a>
## D53. Rapport PDF sur plusieurs pages (2.15.0)

`pdf/report_lines.rs` met les lignes en forme, `pdf/render.rs` les ecrit via `ensure_room` ; plus de `LIMIT 15` dans `pdf/fetch.rs`, la periode entiere part, en ordre chronologique. Mise en forme francaise commune aux PDF dans `pdf/display.rs` (`fr_number`, `fr_measure`, `fr_duration`, `fr_date`, `record_value`). Garde : `tests/integration/report_pdf.rs` (120 pesees, trois pages au moins, premiere et derniere ligne presentes).

<a id="d54"></a>
## D54. Unites des records

Les durees (plus longue nuit, entrainement le plus long) sont en secondes comme leurs tables, unite `s` ; l'ecran (`features/profileCard/format.ts`) et le PDF les rendent en heures et minutes. Hydratation Withings (type 77) en **kg**, pas en %.

<a id="d55"></a>
## D55. Etats vides par periode

`components/shared/PeriodEmptyState.tsx` distingue une periode vide (« Aucune pesee sur cette periode » + « Afficher tout l'historique ») d'un historique vide. `features/sources/components/SourcesStatusBanner.tsx` signale en tete du tableau de bord qu'aucune source n'est connectee.

<a id="d56"></a>
## D56. Couverture, parcours, mutation (R13, 2026-10-22)

Couverture TypeScript (`@vitest/coverage-v8`) dans `npm run verify`, couverture Rust par `npm run couverture:rust` (`cargo llvm-cov`, fin de vague), planchers `couverture_ts`, `couverture_rust` et `couverture_withings_client` en cliquet dans `scripts/verify-floors.json` (`docs/couverture.md`) ; `npm run parcours` visite les 19 pages par CDP sur la copie de dev ; `cargo-mutants` a la main, `--in-place`, par fichier (`docs/mutation.md`).

<a id="d57"></a>
## D57. Tests d'integration en un binaire, contrat IPC (R14, 2026-10-23)

Les tests Rust d'integration vivent tous dans `src-tauri/tests/integration/` (un module par fichier, declare dans `main.rs`, lance par `cargo test --test integration -- <module>::`), aides communes dans `common/` (`memory_pool()`, `test_vault()`), modules partages `support/` (faux serveur Withings) et `pdf_text/` ; `keyring_persistence.rs` reste a part (trousseau reel). Le contrat IPC est tenu par `src/lib/commands.contract.test.ts` (commandes de `generate_handler!`, parametres Rust, appels `invoke` du front) ; `invoke` ne s'importe que dans les `api.ts` (regle ESLint `no-restricted-imports`, `hooks/appReady/api.ts` pour le demarrage).

<a id="d58"></a>
## D58. Trois gardes de taille (R15, 2026-10-23)

`npm run taille` (fichiers de production, 300 lignes, Rust sans ses tests en ligne ; exceptions motivees dans `scripts/taille-exceptions.json`, une exception perimee echoue), clippy `too_many_lines` 100 et `excessive_nesting` 6 (`src-tauri/clippy.toml`, avertissement de production donc erreur sous `-D warnings` ; `run` de `lib.rs` exemptee, c'est la liste des commandes), ESLint `max-lines-per-function` 150, `complexity` 15 et `max-params` 5 (tests exclus). Un `mod.rs` qui porte des commandes Tauri garde son glob `pub use` apres un decoupage.

<a id="d59"></a>
## D59. Corps de reponse Withings jamais dans une erreur (R17, 2026-10-23)

Taille et position seulement, apercu masque par `utils::redact` au niveau debug ; `TokenBody` a un `Debug` masque ; `log_sync` et l'echange de code masquent avant `sync_log` et l'evenement. `open_str` refuse un secret en clair (la migration du demarrage les a chiffres avant), le coffre efface sa cle a la liberation (`zeroize`, deja dans l'arbre par `keyring`). Permissions : `dialog:allow-open` seul, ni shell ni `open-path`. ACL des commandes ecartee (une fenetre, toutes les commandes de [commandes.md](commandes.md)), voir `docs/dev-guide.md`.

<a id="d60"></a>
## D60. Tailwind 4 en configuration CSS (R19, 2026-10-23)

Theme, `@utility` et compatibilite v3 dans `src/styles/globals.css` (voir `docs/frontend.md` section Theme et Tailwind 4), plus de `tailwind.config.ts`, `tw-animate-css` a la place de `tailwindcss-animate`, `npm audit` sans avis haut. `space-*` garde la formule de la v3 et `divide-y` n'est plus employe (classes `[&>:not([hidden])~:not([hidden])]:border-t`)

<a id="d61"></a>
## D61. Verification unique (R00, 2026-10-04)

`npm run verify` seul point d'entree, `npm run check` son niveau rapide, lance par `.githooks/pre-commit` apres la delegation au hook global, qui decide seul du scan gitleaks (vers GitHub seulement) ; hook global introuvable : commit refuse (`npm run hooks:install` par clone) ; detail dans le README, section Verifier. Cargo a la main : `-j 2` et `CARGO_INCREMENTAL=0`, sinon la memoire du poste s'epuise. Planchers de tests : `scripts/verify-floors.json` (absent ou illisible : echec), a relever a chaque ajout, au format prettier. Push de `main` (D12, 2026-10-22) : `.githooks/pre-push` exige dans `logs/verify.log` un passage complet `OK` portant l'arbre du commit pousse ; on verifie la branche propre, le squash `commit-tree` garde son arbre. Sauvegarde : `origin` pointe sur la forge du proprietaire ; reglages du clone a reposer sur un clone neuf.

<a id="d62"></a>
## D62. Veille des dependances (R23, 2026-10-23)

`npm run veille` avant chaque release (rouge = pas de release), exceptions motivees et datees dans `scripts/veille-exceptions.json` (trois mois au plus), politique de licences dans `src-tauri/deny.toml`, `rust-version` 1.94 egal au plancher reel de l'arbre (la veille echoue s'il passe dessous). Elle reutilise Intendance (rapport nocturne du poste) sans la reecrire et ne compile rien ; detail dans `docs/release.md`. Dependabot n'existe plus (`.github/` retire avant R23)

<a id="d63"></a>
## D63. recharts 3 et lucide 1 (R20, 2026-10-23)

Infobulles lues par `tooltipNumber`, `<Tooltip>` apres `<Legend>`, `itemSorter` pour garder l'ordre des series, histogrammes du sommeil et de l'activite agreges par semaine puis par mois au-dela de 150 barres (`lib/charts/aggregateBars.ts`), courbes decimees a 500 points ; detail dans `docs/frontend.md`

<a id="d64"></a>
## D64. Interface coherente (R25, 2026-10-23)

Une couleur d'etat est un jeton (`--success`, `--warning`, `--destructive`, `--primary-text` et leurs variantes `-text`, `--series-*` pour les series de graphique), tenu par `contrast.test.ts` et `palette.test.ts` (aucune couleur nommee de Tailwind, aucun survol `hover:bg-accent` plein) ; les champs passent par `Input`, `Select`, `Textarea` (`champs.test.ts`) ; l'axe des graphiques temporels est numerique (`lib/charts/timeAxis.ts`) ; trois raccourcis (`Ctrl+Maj+S`, `Ctrl+Maj+A`, `Ctrl+B`, `lib/shortcuts.ts`, decision D11) ; theme pose avant React par `public/theme-init.js` (la CSP interdit le script en ligne) ; `lib` ne depend pas de `features` (lint) ; `synk-onboarding-done` se lit par `lib/onboarding.ts`. Detail : `docs/frontend.md` section Coherence de l'interface.

<a id="d65"></a>
## D65. Erreurs IPC typees (R26, 2026-10-25)

Toute commande rend `Result<T, AppError>` ; `AppError` se serialise en `{ kind, message, detail }` (`kind` = nom de la variante, `message` = texte des journaux, `detail` = charge ; `Http` et les evenements passent par `utils::redact`) et `formatError` lit le `kind` (`BY_KIND`), les chaines des plugins gardant les expressions regulieres. Une erreur affichee dans une phrase passe par `errorText`, jamais `${e}`. Un echec de connexion Withings arrive sur `withings-auth-error` en objet type, ecoute par l'assistant et par Parametres. Un rate limit qui epuise les reessais reste un `RateLimit`. Guide : `docs/dev-guide.md` section Erreurs. Spec : `docs/specs/2026-10-25-erreurs-ipc-typees-design.md`

<a id="d66"></a>
## D66. Schema et maintenance SQLite (R27, 2026-10-25)

Le registre `MIGRATIONS` (`db/bootstrap.rs`) deduit `LATEST_VERSION` ; la tolerance du runner (`tolerated`) est figee jusqu'a la 16 et tracee en `warn`, la 17 et les suivantes sont strictes ; un fichier de migration n'a ni `;` en commentaire ou chaine, ni trigger. La 017 retire 7 index en double (dont les cinq `idx_legacy_*` que `legacy.rs` recreait) et confie la suppression de `alert_events` et `medication_logs` a `ON DELETE CASCADE`. `PRAGMA optimize` a l'ouverture et apres la sauvegarde du jour, `sync_log` borne a 1 000 lignes et 90 jours (derniere synchro reussie de chaque type gardee), detecteurs sur index, `triggered_at` en UTC avec `Z`. Siestes Withings : garde de detection seulement. Detail : `docs/database.md` section Maintenance et formats. Spec : `docs/specs/2026-10-25-schema-et-maintenance-sqlite-design.md`

<a id="d67"></a>
## D67. Couche metier et source unique des metriques (R28, 2026-10-25)

`services/` (`backup`, `audit`, `error_log`, `sync_log`, `insights`, `credentials`, `retention`) porte ce que `boot/` et `withings/` appellent ; les commandes sont des adaptateurs, `validation.rs` est hors de `commands/`, et `tests/integration/architecture.rs` refuse tout `crate::commands` hors de `commands/` et de `lib.rs`. Chaque metrique a une source (`metric_repository::source_of`) : sa table historique pour toutes, sauf `heart_rate` (`Split`, le pouls sans tension va dans `metrics`) ; plus de double ecriture ni de deduplication ; les lignes de `metrics` d'avant sont laissees. Migration 018 : `derived_metrics` et `metric_targets` retirees. Tableau de bord : `get_dashboard_cards` (6 lectures au premier affichage, 15 avant). Zeroize de la cle et des secrets ouverts, neutralisation des formules du CSV, import JSON verifie sur ses premiers octets. Spec : `docs/specs/2026-10-25-couche-metier-et-metriques-v2-design.md`

<a id="d68"></a>
## D68. Socle des sources (R31, 2026-10-25)

`sources/` porte le contrat (`Source` : `id`, `label`, `kind`, `capabilities`, `sync(&SyncContext) -> SyncBatch`), le registre `all_sources()` / `require_source` et `model.rs` (`SourceId`, `SourceKind`, `Domain`, `Observation`, `SyncBatch`) ; Withings vit sous `sources/withings/` (ex `withings/`, deplace par `git mv`) et `WithingsSource` enveloppe `run_sync_all` : la synchro Withings passe par le trait (`result_from` rebatit le `SyncResult` depuis le lot, dans les tests). `redact` est sorti vers `utils/redact.rs` (generique, lu par `AppError`). Garde : `tests/integration/architecture.rs` refuse tout `sources::<x>` hors de `sources/<x>/`, du registre `sources/mod.rs` et de `lib.rs` (la liste des commandes). Aucune commande, aucun evenement ni aucun ecran ne change. R32 (OAuth commun), R33 (schema), R34 (interface) construisent dessus. Fiche : `docs/plans/2026-10-22-sante-multi-sources.md`

<a id="d69"></a>
## D69. OAuth commun (R32, 2026-10-25)

`sources/oauth/` porte ce qu'une marque OAuth declare par un `OAuthSpec` (`SourceKind::OAuth(&'static OAuthSpec)`) : URL d'autorisation, base et chemin du jeton, portees, PKCE oui ou non, chemin de rappel, parametres fixes du jeton et fonction qui lit la reponse (`standard_json` pour RFC 6749, `withings::spec::parse_envelope` pour l'enveloppe Withings en HTTP 200). Il fournit le serveur de rappel (`server`), l'etat par source (`flow`), PKCE S256 (`pkce::Verifier`, 64 octets aleatoires, `Debug` masque, jamais journalise), l'echange et le rafraichissement (`token`, 3 tentatives sur 429 ou 601), le verrou par source (`refresh`) et `start_authorization` / `exchange_and_store` (`connect`). `sources/store/` lit et ecrit jetons et identifiants par `source_id`, chiffres par `CryptoVault` : il lit et ecrit `source_auth` et `source_credentials` (creees par la migration 019 de R33, qui a recopie `withings_auth` et `withings_credentials` et en a fait des vues de lecture). `invalidate_undecryptable_secrets` couvre toutes les sources et rend `InvalidatedSecrets { sources: [{ source, label, tokens, credentials }] }` ; `ReconnectNoticeCard` en liste les noms. Le garde d'architecture traite `oauth` et `store` comme des modules partages, pas comme des sources. Fiche : `docs/plans/2026-10-22-sante-multi-sources.md`

<a id="d70"></a>
## D70. Schema multi-sources et choix de la source (R33, 2026-10-25)

Migrations 019 et 020 (`sources`, `source_auth`, `source_credentials`, `source_state`, `metric_source_priority` ; cle etrangere `source` vers `sources` et `external_id` sur les huit tables de mesures ; `withings_auth`, `withings_credentials` et `sync_state` deviennent des vues en lecture seule). Une nouvelle source se declare dans le registre ET dans `sources` (un test refuse l'oubli) ; une valeur de `source` inconnue est refusee par la base, ignoree par l'import JSON. `services::metrics::select_series` rend une seule source par metrique et par jour (priorite de l'utilisateur, puis source la plus recente, puis ordre du registre) et ne somme jamais deux totaux ; `metric_repository::read_metric_series` est la lecture brute. Le runner annule une migration stricte qui ajoute des violations de cle etrangere. Detail : `docs/database.md` (019 et 020), `docs/metrics-v2.md` (choix de la source). Fiche : `docs/plans/2026-10-22-sante-multi-sources.md`

<a id="d71"></a>
## D71. Interface des sources (R34, 2026-10-25)

Parametres > Sources (`pages/settings/SourcesPanel/`, une `SourceCard` par entree de `list_sources`, onglet Priorites) remplace Withings ; `features/sources/` (api, hooks, `labels.ts` tenu par un test contre la graine de la migration 019, `SourceBadge`, `sourceColumn`, `reconnectNotice`) remplace `features/withings*`. `sources/sync.rs` (`sync_sources`) lance chaque source connectee dans l'ordre du registre, une defaillance est une issue `SourceOutcome { failed }` journalisee dans `sync_log` sous sa source et n'arrete pas les suivantes ; `sync-progress` porte `source`, `etape`, `total` (`Source::steps`). Commandes : `list_sources`, `source_start_auth`, `source_disconnect`, `source_set_credentials`, `sync_all_sources`, `get_metric_priorities`, `set_metric_priority`, `get_metric_secondary_points`, `sync_source` ; `get_invalidated_secrets` (ex `get_invalidated_withings_secrets`, R43) rend toutes les sources. Migration 021 (`sync_log.source`). Les detecteurs sommeil, tension et ECG passent par `services::metrics::keep_preferred` (une source par jour). Les listes de lignes gardent toutes les sources (un historique n'est pas une courbe), avec une colonne Source. Le bloc Import est construit par R39 (source fichier Apple Sante) et complete par R40. Fiche : `docs/plans/2026-10-22-sante-multi-sources.md`

<a id="d72"></a>
## D72. Oura (R35, 2026-10-26)

`sources/oura/` (`spec` OAuthSpec avec PKCE et portees `daily spo2 workout`, `types` serde stricts sur `day` et les dates, `mapping` pur, `client` GET pagine par `next_token` avec 401 -> un rafraichissement, 429 -> `Retry-After` plafonne a 60 s, `persist` upserts, `sync` quatre etapes, `window` 180 jours puis reprise 7 jours en arriere). Le refresh token d'Oura est a usage unique : tout passe par `oauth::refresh_locked`. Une ligne `sleep_records` par jour et par source (periode `long_sleep` la plus longue + score de `daily_sleep`, `COALESCE` a l'upsert), `activity` par jour, `workouts` par `(source, external_id)`, observations `hrv`, `readiness_score`, `temperature_deviation`, `spo2` dans `vitals_legacy` avec `external_id` = `<metrique>:<id du document>` (la cle `(source, external_id)` est commune a tous les types de constantes). Les definitions de metriques d'une source viennent de `Source::metric_definitions`, seedees par `seed_source_definitions` (Rust, jamais une migration). La serie de frequence cardiaque a 5 minutes n'est pas importee. Spec : `docs/specs/2026-10-22-sante-multi-sources-design.md` section Oura. Fiche : `docs/plans/2026-10-22-sante-multi-sources.md`

<a id="d73"></a>
## D73. Polar (R36, 2026-10-26)

`sources/polar/` (AccessLink v3 sans transactions, dépréciées par Polar depuis 2025-08-18 ; `spec` avec `client_auth: Basic` (l'authentification du client au point de jeton est un en-tête, `OAuthSpec::client_auth`), hôte du jeton `polarremote.com` distinct de l'hôte de l'API, pas de PKCE, portée `accesslink.read_all` ; jeton sans rafraichissement, un 401 arrête la source ; `client` GET Bearer, 204 = rien, 429 -> `Retry-After` ou `RateLimit-Reset` plafonne a 60 s ; `sync` : enregistrement `POST /v3/users` a la premiere synchro (domaine `registration` de `source_state`, 409 = deja fait), trois etapes, curseur avance seulement apres l'ecriture). Fenetres imposees par Polar : sommeil et Nightly Recharge 28 jours, exercices 30 jours, activite par tranches de 27 jours. `ans_charge` et `nightly_recharge_status` viennent de `Source::metric_definitions`. Spec : `docs/specs/2026-10-22-sante-multi-sources-design.md` section Polar. Fiche : `docs/plans/2026-10-22-sante-multi-sources.md`

<a id="d74"></a>
## D74. Whoop (R37, 2026-10-26)

`sources/whoop/` (API v2 `https://api.prod.whoop.com/developer/v2`, `spec` sans PKCE avec la portee `offline` sans laquelle aucun jeton de rafraichissement n'est delivre, jeton a rotation donc `oauth::refresh_locked` ; `client` GET pagine par `nextToken`, 25 par page, tout envoi passe par `pacer::Pacer` (fenetre glissante de 60 s, 90 requetes au plus) et toute attente par l'horloge injectee `Clock` (`SystemClock` en production, un temps virtuel dans les tests) ; `throttle::classify` lit `X-RateLimit-Limit` et `X-RateLimit-Reset` d'un 429 : limite a la minute = attente et rejeu (trois tentatives), quota du jour = arret sans rejeu, heure de reprise notee dans `source_state` (domaine `quota`) et synchro suivante sans reseau tant qu'elle n'est pas passee ; `sync` trois etapes : sommeil et recuperation (jointure par `sleep_id`), cycles (`activity` sans pas, plus `strain`), entrainements). Seuls les enregistrements `SCORED` sont ecrits. `recovery_score`, `resting_heart_rate` et `strain` viennent de `Source::metric_definitions`. Spec : `docs/specs/2026-10-22-sante-multi-sources-design.md` section Whoop. Fiche : `docs/plans/2026-10-22-sante-multi-sources.md`

<a id="d75"></a>
## D75. Google Health, ex-Fitbit (R38, 2026-10-26)

`sources/google_health/` (API `https://health.googleapis.com/v4`, autorisation Google avec PKCE et `authorize_params` `access_type=offline` + `prompt=consent`, trois portees `googlehealth.*.readonly`, client Google Cloud de type Application Web, rappel `http://localhost:17483/callback/google_health` pendant que le serveur ecoute sur `127.0.0.1` ; `client` verifie `users/me/identity` a chaque synchro (`ACCOUNT_NOT_LINKED` arrete la source), liste paginee par `pageToken` (25 par page pour sommeil et seances), cumuls `dailyRollUp` en tranches de 90 jours (14 pour `total-calories`) ; aucune definition de metrique neuve). Le jeton de rafraichissement vit 7 jours tant que le projet Google est en Testing : `OAuthSpec::refresh_lifetime_secs` note l'echeance a la connexion (`source_state`, domaine `refresh_expiry`), `oauth::reauth::reauth_status(echeance, maintenant)` annonce a J-1 (`list_sources` -> `reauth`, carte et bandeau du tableau de bord), et un rafraichissement reussi apres l'echeance l'efface. Spec : `docs/specs/2026-10-22-sante-multi-sources-design.md` section Google Health. Fiche : `docs/plans/2026-10-22-sante-multi-sources.md`

<a id="d76"></a>
## D76. Import Apple Sante (R39, 2026-10-26)

`sources/import/apple_health/` (source fichier `SourceKind::File(&FileSpec)`, extensions `zip` et `xml`, enregistree au registre mais jamais synchronisee). `open` lit l'entree `export.xml` du zip en flux (le plus gros `.xml` hors `*_cda*`), `reader` (quick-xml, jamais chargee entiere, les `Record` enfants d'une `Correlation` sont ignores car repetes au premier niveau), `mapping` (`Mapper` : mesures vers observations, tension appariee, activite et sommeil agreges par jour et par origine), `persist` (une seule transaction, `ObservationWriter`), `run` (lecteur dans `spawn_blocking`, lots de 2 000 par canal borne, progression toutes les 200 ms, annulation = retour arriere complet). `external_id` = empreinte SHA-256 de (type, debut, fin, valeur, origine) : reimport sans doublon. L'app d'origine va dans la colonne `device` (migration 022). Commandes `import_source_file` et `cancel_import` (`commands/import.rs`, un import a la fois), evenement `import-progress`, bloc Import de Parametres > Sources (`ImportCard`, selection par `dialog:allow-open`, jamais un chemin saisi). Nouvelles dependances `zip` (`~9.0.5`, sans fonctions par defaut) et `quick-xml` 0.42 (deja dans l'arbre). Spec : `docs/specs/2026-10-22-sante-multi-sources-design.md` section Apple Sante. Fiche : `docs/plans/2026-10-22-sante-multi-sources.md`

<a id="d77"></a>
## D77. Import FIT, GPX, TCX et CSV (R40, 2026-10-26)

`sources/import/` partage le pipeline (`pipeline.rs` : lecteur dans `spawn_blocking`, canal borne, une transaction, annulation = retour arriere), `rows.rs`, `persist.rs` (`Persister::new(source)`), `sleep.rs`, `time.rs` et `fingerprint.rs` ; chaque format n'ecrit que sa lecture (`fit/`, `trace/` pour GPX et TCX par `quick-xml`, `table/` pour le CSV). Sources `fit`, `trace` (migration 023) et `csv`, enregistrees au registre ; `import_source_file` aiguille par source, `import_csv_file`, `inspect_csv_file`, `save_csv_mapping` et `delete_csv_mapping` servent l'assistant de correspondance (`CsvAssistant`, champs par `Input` et `Select`). Crate `fitparser` `~0.11.0` (reputation et defauts releves dans la spec : un fichier tronque passe sans erreur, donc `read::check_frame` ; la lecture est sous `catch_unwind` ; un champ `weight` sort non mis a l'echelle). Le trace n'est pas stocke, seul le resume. Un seul prereglage CSV (LibreView), les autres marques sur l'assistant generique. Spec : `docs/specs/2026-10-22-sante-multi-sources-design.md` section Fichiers FIT, GPX, TCX et CSV. Fiche : `docs/plans/2026-10-22-sante-multi-sources.md`

<a id="d78"></a>
## D78. Reprises backend (R42, 2026-10-26)

Aucun appel de fichier bloquant dans une fonction `async` (`tokio::fs` ou `spawn_blocking`), tenu par `tests/integration/blocking_io.rs` qui lit les sources ; la sauvegarde quotidienne est rappelee toutes les heures par `boot::run_every` (`periodic_backup_inner` garde sa regle des 20 h) ; `utils::dates::day_range` valide les deux bornes de toute plage, un slug inconnu est `AppError::NotFoundByKey` (kind `NotFound`) ; `backup-failed` est une erreur typee (`backup_failure_payload`) ; a egalite le classement des sources suit `source_auth.created_at` (connexion la plus ancienne d'abord) ; `Source::revoke` est appele a la deconnexion (Polar, Whoop, Google Health), son echec est un avertissement (`DisconnectReport`) qui n'arrete jamais la deconnexion locale. Fiche : `docs/plans/2026-10-22-sante-multi-sources.md`

<a id="d79"></a>
## D79. Release en une commande et journal reglable (R29, 2026-10-26)

`npm run release -- X.Y.Z` (`scripts/release.mjs`, fonctions pures dans `release-lib.mjs`) enchaine controles, veille, montee de version commitee, verify complet sur l'arbre commite, couverture Rust, parcours (un nouvel essai si « aucune reponse CDP »), budget, build `CARGO_BUILD_JOBS=1`, controle de l'installeur (taille, date, version de fichier, hausse de poids au plus 10 %), copie dans `D:\Sauvegardes\Synk\installeurs\` avec son `.sha256` et journal des durees ; `--simulation` fait tout sauf build, copie et commits ; il ne pose aucune etiquette et ne pousse rien. La version vit en sept emplacements (cinq fichiers et le README) tenus par `src/__tests__/versions.test.ts` et montes par `npm run bump`. Pas de signature ni de mise a jour automatique (D9, `docs/release.md`). `SYNK_LOG` regle le niveau du journal (`logging.rs`, `info` par defaut, avertissement si illisible), cinq fichiers d'un Mo conserves ; `scripts/` est lint par `npm run verify`. `PROD_IDENTIFIER` est compare a `tauri.conf.json` par `tests/integration/app_identity.rs`.

<a id="d80"></a>
## D80. Durcissement multi-sources (R43, 2026-10-26)

`sources/import/limits.rs` plafonne les lectures XML hostiles (volume, noeud de 4 Mo, profondeur, elements ; `Guarded` mesure les octets consommes depuis le dernier evenement) ; `db/gate.rs` signale un import en cours (`LongWrite`) : delai d'occupation ramene a 250 ms pour les autres ecritures, erreur claire `REFUSAL`, `sync_all_sources` et `sync_source` refusees d'emblee, la transaction unique de l'import et son retour arriere restant intacts ; `sources/oauth/rest.rs` + `brand.rs` portent le client REST commun (`Brand` = donnees et pointeurs de fonction par marque, `Screen` pour les statuts propres a une marque, `Throttle` pour la limite de debit, `Refresh` pour le rafraichissement) ; `ObservationWriter::revising()` (sources API) met a jour une valeur revisee, le defaut garde la premiere (fichiers) ; `write_tolerant` ecrit les observations d'une etape en une transaction ; `Source::import` porte le lecteur des sources fichier et `services/import.rs` l'orchestration ; `metric_model.rs` (`SourceId`, `Observation`) casse le cycle sources et services ; `utils/http.rs` fixe https seul et aucune redirection ; Oura et Withings revoquent a la deconnexion (`utils/hmac.rs` signe la demande Withings). Evenements : noms dans `features/sources/events.ts`, tenus par `events.contract.test.ts`. Fiche : `docs/plans/2026-10-22-sante-multi-sources.md`

<a id="d81"></a>
## D81. Synk se relance hors du conteneur de l'app Claude (3.1.0, 2026-11-01)

Avant Tauri, `main` appelle `boot::container::relaunch_outside_if_needed` : une sonde vide ecrite dans `%APPDATA%\com.synk.health\` est cherchee dans `%LOCALAPPDATA%\Packages\*\LocalCache\Roaming\com.synk.health\`, puis effacee. Trouvee dans un calque, Synk pose un marqueur, se relance par `explorer.exe` (qui demarre hors du conteneur) et s'arrete avant le greffon d'instance unique. Un marqueur de moins de 60 s fait rester sur place, avec un avertissement au journal, pour ne jamais boucler. Sautee en build de dev et sous `SYNK_CONTENEUR=rester` (pose par `npm run parcours`, dont l'option `--exe` vise un build de release de `tauri.dev.conf.json` et garde son port CDP). Detecteur empirique parce que l'identite de paquet est absente dans ce conteneur (P35). Le journal de demarrage dit `lancement hors conteneur`. Tests : `boot/container.rs` (decision, calque, garde de 60 s, sonde qui ne laisse rien).

<a id="d82"></a>
## D82. L'assistant de premier lancement suit l'etat de la base (3.1.0, 2026-11-01)

`OnboardingGate`, charge a la demande avec l'assistant pour tenir le budget du chemin critique, ouvre l'assistant seulement si la cle `synk-onboarding-done` manque ET si la base n'a aucune mesure (`get_db_metrics`) ET si aucune source n'est connectee (`list_sources`). Une base deja remplie reecrit la cle sans ouvrir l'assistant ; une lecture en echec ou une reponse illisible (`needsOnboarding` valide les compteurs) l'ouvre, comme avant. La cle vit dans le stockage de la WebView, qu'un nettoyeur ou un autre profil peut perdre : la base, elle, est sauvegardee et restauree. Aucune commande IPC neuve. Tests : `AppLayout.test.tsx` (installation neuve, mesures presentes, source connectee, lecture en echec).

<a id="d83"></a>
## D83. Silhouettes : mannequin photo en PNG, PDF par image (2026-11-01)

`src/assets/body/front.png` et `back.png` remplacent les deux SVG traces par `gen-body-plate.js` : un mannequin gris d'homme, de face et de dos, issu d'une photo Pexels retouchee en local (provenance, licences et empreintes SHA-256 dans `src/assets/body/PROVENANCE.md`). **Format** : PNG niveaux de gris + alpha, 512 x 1400 px (echelle 2 du repere 256 x 700), 105 et 99 Ko ; le RGBA de depart pesait 259 et 245 Ko (au-dela des 150 Ko vises), un WebP qualite 85 en aurait pese 40 mais ajoute un decodeur Rust et ne se lit pas par `scripts/`. Le PNG reste sans perte, un seul fichier par vue sert l'ecran (`BodyFigure`, import Vite) et le PDF (`include_bytes!`), et se decode en Node par `zlib` seul. **Repere** : `plate-landmarks.json` (256 x 700, sommet du crane a y = 21, plante a y = 690) ; ses 21 reperes sont mesures sur le masque (sommet, plante, fourche, taille, mollet, cheville) ou juges a l'oeil sur une image annotee (les 15 autres), les 16 ancres du catalogue sont recalees sur leur partie du corps, `anchor.y` se compte toujours depuis les pieds (P39). **Controle**, qui remplace le generateur : `npm run plate:check` et `scripts/plate-lib.test.mjs` mesurent sur le masque le sommet, la plante, la fourche et les paliers de taille, de mollet et de cheville (tolerances dans `plate-lib.mjs`), exigent l'ordre des reperes, les proportions image / viewBox, chaque ancre sur un pixel opaque, une ancre de bras sur un bras detache du tronc, moins de 150 Ko par vue et les SHA-256 cites par `PROVENANCE.md` ; `png-lib.mjs` (lecteur PNG sans dependance) est verifie contre des fichiers ecrits par Pillow et a la main. **PDF** : printpdf 0.12.8 sans `svg` ni `html` (`features = ["png"]`), `RawImage::decode_from_bytes` puis `Op::UseXobject` (Scale puis Translate, P41), figure dimensionnee par ses pixels a l'echelle que fixe D84, libelles ecrits apres les deux figures. Plus de `figures.json`, de `Svg::parse` ni de `npm run figures:gen` ; 74 entrees quittent `Cargo.lock`, dont `bincode`, `rustybuzz` et `ttf-parser`, ce qui retire trois exceptions de `scripts/veille-exceptions.json` (`cargo audit` : 0 vulnerabilite). `canvas.rs` retire le plafond de 2 Mo de printpdf (P54). Gardes : `the_sheet_pdf_carries_both_figures_unaltered` (taille d'origine, masque de transparence, meme echelle, proportions) et `keeps_an_image_above_two_megabytes_at_its_original_size`. Remplace D40 (SVG pre-convertis) et D47 (planche generee).

<a id="d84"></a>
## D84. Fiche PDF : libelles en colonnes, mesures puis ecartes (2026-11-01)

La page 1 de la fiche ne pose plus aucun texte sur les silhouettes. `pdf/callouts.rs` (`sheet_layout`) range les 16 libelles en colonnes hors des figures : a gauche de la face, a droite de la face, a droite du dos. Chaque colonne prend la largeur de son plus long texte, mesuree par les avances de la police Helvetica que printpdf embarque (`pdf/text_metrics.rs`, `BuiltinFont::get_parsed_font` ; un caractere absent compte pour un cadratin, la largeur n'est jamais sous-estimee). Les deux figures prennent la largeur restante, a la meme echelle (48,5 mm de large et 132,6 mm de haut sur la fiche de test, mesures dans le PDF ; plafond 70 mm, plancher teste de 45 mm), et la composition est centree sur la page. Nom au-dessus du trait, valeur en gras dessous, pastille au point mesure (`Canvas::fill_discs`). Deux traits voisins restent a 7,5 mm au moins : `spread` regroupe les libelles trop proches, centre chaque groupe sur la moyenne de ses ancres, puis borne la colonne a la zone utile. Le trait part de l'ancre, reste horizontal jusqu'au bord du cadre de la figure, puis se coude dans la gouttiere vers sa colonne (`Canvas::add_polyline`). Une ancre se place du cote de son libelle, sinon son trait traverse tout le corps : epaule, tour de bras et bras tendu passent sur le bras droit de l'image (77 %, 81,8 % et 84 %), la carrure dos rejoint la colonne de droite ; `npm run plate:check` confirme qu'elles tombent sur le bras. Gardes : `figure_page.rs` (aucun libelle sur une figure ni sur un autre, fiche vide comme remplie de 199,9 ; `never_runs_a_leader_across_the_whole_body`), `callouts.rs` (`spread`, disposition), `text_metrics.rs` (avances connues de Helvetica), `the_sheet_pdf_carries_both_figures_unaltered` (meme echelle pour les deux vues, entre 45 et 70 mm). Remplace la mise en page fixe de D83 : figures de 70 mm et libelles poses a 14 mm du cadre, qui chevauchaient le mannequin.
