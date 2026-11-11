# Pièges connus

Chaque entrée est un piège déjà payé : le symptôme, la cause, et le garde qui le voit désormais. Les identifiants (P01 à P54) sont stables ; `CLAUDE.md` en reprend l'essentiel et renvoie ici pour le détail.

<a id="p01"></a>
## P01. Une classe de `@layer utilities` n'est plus un utilitaire en Tailwind 4

(R19) : sans `@utility`, `hover:glow-*` disparaissait sans erreur ; controle par `scripts/css-classes.mjs` (liste des classes du CSS construit) et par `src/styles/tailwind-compat.test.ts`. Autres pieges de la montee : une classe collee a une interpolation (`hover:bg-muted/30${x}`) n'est plus reconnue, une couleur en dur (`bg-indigo-600`) change de teinte (palette OKLCH), `space-y-*` ne marque pas un enfant en ligne (la formule v3 est retablie), et `td`, `th`, `option` perdent leur remplissage par defaut (retabli dans `@layer base`).

<a id="p02"></a>
## P02. Lint (R18, 2026-10-23)

ESLint 10 en configuration plate (`eslint.config.mjs`), aucune regle en avertissement. `overrides` de `package.json` force le pair `eslint` de `eslint-plugin-jsx-a11y` (6.10.2 declare `^9` au plus) : le retirer quand jsx-a11y declare ESLint 10. `eslint-plugin-react-hooks` 7 : seulement `rules-of-hooks` et `exhaustive-deps` (le prereglage `recommended` ajoute les regles du compilateur React). `complexity` compte `?.`, `??` et les valeurs par defaut. Detail : `docs/dev-guide.md` section Lint.

<a id="p03"></a>
## P03. Developpement : `npm run tauri:dev` seulement

Un build de dev sur l'identifiant de production est refuse au demarrage (`ensure_not_dev_on_real_data`, `boot.rs`) et la cle keyring suit l'identifiant (`service_for`, `crypto/keyring.rs`) : `synk-dev` pour `com.synk.health.dev`, `synk` pour la production. Ne jamais lancer `npm run tauri dev` nu ni `target/release/synk.exe` depuis une session.

<a id="p04"></a>
## P04. Le build Tauri et les caracteres speciaux du chemin

Le build Tauri plante si le chemin contient certains caracteres speciaux

<a id="p05"></a>
## P05. night_events arrive en chaine JSON

L'API Withings retourne `night_events` comme string JSON au lieu de i64 → deserializer lenient

<a id="p06"></a>
## P06. heart_rate arrive en entier ou en structure

L'API Withings retourne `heart_rate` comme int brut OU struct → deserializer custom

<a id="p07"></a>
## P07. more vaut parfois false

`more` peut etre `false` (bool) au lieu de `0` (int) → deserializer custom

<a id="p08"></a>
## P08. snoringepisodecount dans les data_fields

`snoringepisodecount` (pas `snoring_episode_count`) dans les data_fields

<a id="p09"></a>
## P09. userid, entier ou chaine

`userid` peut etre int ou string → `serde_json::Value`

<a id="p10"></a>
## P10. Les curseurs Withings doivent exister

Les curseurs Withings de `source_state` (vue `sync_state`) doivent exister pour les domaines measures, activity, sleep, heart, workouts, devices (et `metrics`, historique) : `seed_sync_state` (`db/seed.rs`) les cree au premier demarrage.

<a id="p11"></a>
## P11. duration_total, pas total_sleep_duration

`duration_total` (pas `total_sleep_duration`) dans sleep_records

<a id="p12"></a>
## P12. EmptyState accepte une icone ou un noeud

EmptyState accepte `ElementType | ReactNode` pour l'icone (pas seulement ElementType)

<a id="p13"></a>
## P13. Le rappel OAuth recoit un favicon avant le vrai appel

Le callback OAuth peut recevoir des favicon.ico avant le vrai callback → boucle obligatoire

<a id="p14"></a>
## P14. Pas de page neuve sans demande

Ne jamais ajouter de nouvelle page sans demande explicite de l'utilisateur

<a id="p15"></a>
## P15. Vues retrocompat = read-only

Tenter `INSERT INTO body_measures` apres bootstrap echoue silencieusement en prod (`SQLITE_ERROR: cannot modify ... view`). Toujours pointer les ecritures sur `*_legacy`. Garde-fou : `tests/integration/phase1_metrics.rs::test_inserts_into_legacy_tables_after_bootstrap`.

<a id="p16"></a>
## P16. Withings refresh_token a usage unique

Withings revoque l'ancien refresh a chaque echange. Ne jamais paralleliser deux refresh d'une meme source : tout rafraichissement passe par `oauth::refresh_locked` (le `refresh_locked` de `sources/withings/client/tokens.rs` n'est que son enveloppe).

<a id="p17"></a>
## P17. Withings met l'erreur dans l'enveloppe, en HTTP 200

(R12) : `status` 401 jeton invalide (un rafraichissement, un seul), 503 parametres invalides (jamais reessaye), 601 trop de requetes (reessaye). Ne jamais se fier au code HTTP seul. Le client et les six modules de synchro sont testes contre le faux serveur de `tests/integration/withings_fake_server.rs` (voir `docs/api-withings.md`).

<a id="p18"></a>
## P18. Boot orchestre par `boot.rs` (V1.1.0)

Sequence typee `resolve_app_data_dir -> backup_before_migration -> init_pool_with_retry -> integrity_check` (puis crypto, steps 5 a 7). Sous-modules : `boot/auto_backup.rs`, `boot/periodic_backup.rs`, `boot/post_ready.rs`, `boot/runner.rs`, `boot/schedule.rs`. Le seed legacy + sync_state est differe en post-ready (tache detachee). Erreurs serialisables vers JS via event `app-error` avec payload `BootError { kind, ... }`.

<a id="p19"></a>
## P19. Auto-backup pre-migration (V1.1.0)

Declenche uniquement si `schema_version < EXPECTED_LATEST_VERSION` (= `LATEST_VERSION`, la derniere migration du registre). Rolling 3 dans `app_data_dir/backups/auto/`. Restorable via commande `restore_latest_backup`. Son echec arrete le boot depuis 2.15.0 (`BootError::BackupFailed`).

<a id="p20"></a>
## P20. Single-instance (V1.1.0)

`tauri-plugin-single-instance` en TETE de la chaine `.plugin(...)`. La 2e instance ramene la fenetre existante au front, evite 100% des `database is locked` causes par double-lancement.

<a id="p21"></a>
## P21. Prefetch Dashboard (V1.1.0)

`bootPrefetch.ts` rechauffe la requete `dashboardCards` (plage de 30 jours) via `qc.prefetchQuery` depuis le listener `app-ready`. La `queryKey` doit etre EXACTEMENT celle de `useDashboardCards` (`[DASHBOARD_CARDS_KEY, from, to]`), sinon le cache n'est pas reutilise.

<a id="p22"></a>
## P22. AppLayout auto-sync differe (V1.1.0)

`scheduleIdle` (`lib/scheduleIdle.ts` : `requestIdleCallback`, repli sur `setTimeout`) evite que la lecture du registre (`list_sources`) retarde le first paint.

<a id="p23"></a>
## P23. Backup DB d'un PC vers un autre = perte session OAuth

(V2.8.0+) : la cle maitre est liee au compte Windows local. Restaurer `synk.db` sur un autre PC echouera le dechiffrement des jetons OAuth de chaque source (les mesures sante restent lisibles). Au demarrage suivant, `invalidate_undecryptable_secrets` efface les secrets illisibles et l'UI demande de ressaisir les credentials puis de se reconnecter. Une restauration faite en cours de session n'est pas re-examinee avant le prochain boot : d'ici la, lectures et sync remontent l'erreur `Erreur de chiffrement : ...`.

<a id="p24"></a>
## P24. Trousseau : le magasin se pose, il ne se devine pas

(R21) : `keyring-core` n'a aucun magasin par defaut, `init_store` pose celui de Windows une fois par processus (`OnceLock`). Les tests unitaires de `crypto/keyring.rs` utilisent `keyring_core::mock::Store` (un magasin neuf par entree : deux entrees de meme service et meme utilisateur d'un meme magasin partagent leur secret) et ne peuvent pas voir un defaut de persistance ; seul `tests/keyring_persistence.rs` (`#[cfg(windows)]`, service `synk-test-persistence-<pid>` supprime par garde RAII) le prouve, avec la cible `{user}.{service}` et la persistance `Enterprise` de l'identifiant reellement stocke. Ne jamais faire tourner un test sur le vrai service `synk` / `master_key`.

<a id="p25"></a>
## P25. `save_tokens` necessite vault

La signature est `save_tokens(pool, tokens, vault)` depuis V2.8.0. Tous les call sites doivent propager le `vault: &CryptoVault` (recupere via `tauri::State<Arc<CryptoVault>>` au niveau commande, puis `vault.as_ref()` ou `vault.inner().as_ref()` pour deref vers `&CryptoVault`). Idem pour `get_valid_client`, `exchange_code`, `refresh_token`, `sources::store::get_credentials`, `set_credentials`, `load_auth` et `save_auth`.

<a id="p26"></a>
## P26. Serveur callback OAuth = singleton app-lifetime (V2.12.1, generalise en R32)

`source_start_auth` ne lance PLUS un serveur tiny_http par appel. Un seul serveur tient le port 17483 pour toute la vie de l'app et pour toutes les sources (`oauth::flow::OAuthFlow` via `OnceLock` + `compare_exchange` sur un `AtomicBool`), et le `state` attendu de chaque source est toujours celui de son DERNIER `start_authorization`. Avant : chaque tentative spawnait un serveur ; sur une relance dans les 120s, le nouveau ne pouvait pas re-bind (port tenu par l'ancien) et l'ancien validait le callback contre son ancien `state` -> « state invalide » deterministe. Le routage est une fn pure `route_callback` (`/callback/<source>`, l'ancien `/callback` va a Withings : l'URL de rappel enregistree chez Withings ne change pas), la decision de validation une fn pure `evaluate_query`, et la boucle `oauth::server::serve(server, flow, sink, poll, stop)` (`serve_both` ecoute `127.0.0.1` et `[::1]`, `bind_loopback` refuse un port deja pris sur `::1`), que les tests (R21, N-21) font tourner sur un vrai port local ephemere avec un `OAuthFlow::new()` a eux (favicon, state faux, absence de tentative, code manquant, dernier state gagnant, delai signale une fois, arret propre) ; `tiny_http` 0.12.0 est la derniere version publiee (2022) et reste. Evenements `<source>-auth-success/error/timeout` (donc `withings-auth-*` pour Withings) inchanges ; l'ancienne commande `withings_start_auth` n'existe plus (R34, `source_start_auth`).

<a id="p27"></a>
## P27. `measurement_types` ne peut PAS etre seede par migration SQL

Sur une base existante, `measurement_types` est deja une vue de retrocompat au moment ou une migration versionnee tournerait (`INSERT INTO` une vue -> `SQLITE_ERROR`, boot casse). Tout seed de type built-in doit etre une fonction Rust appelee apres `migrate_to_v2_schema`, ciblant `measurement_types_legacy`. Cf. `seed_body_catalog` dans `db/seed.rs`.

<a id="p28"></a>
## P28. Libelles de la silhouette ecrits cote Rust, jamais dans l'image

Printpdf 0.7 ignorait les `<text>` d'un SVG ; la 0.12 les rendait avec les polices systeme du poste, donc un rendu qui changeait d'une machine a l'autre. Depuis D83 la figure est un PNG : les libelles passent par `canvas.use_text(...)`, apres `place_image` des deux figures (un libelle ecrit avant la figure de dos passait derriere elle).

<a id="p29"></a>
## P29. `MIN()`/`MAX()` perdent la date de l'extreme

Une fonction d'agregat SQL donne la bonne valeur mais aucun moyen fiable de recuperer la ligne (donc la date) qui l'a produite en une seule requete sans sous-requete corrigee. `commands/profile_card/records.rs` utilise systematiquement `SELECT valeur, date ... ORDER BY valeur ASC/DESC LIMIT 1` pour ne jamais decoupler une valeur record de sa date d'obtention.

<a id="p30"></a>
## P30. `PRAGMA foreign_keys` est un no-op a l'interieur d'une transaction

SQLite l'ignore silencieusement, sans erreur. Les migrations qui reconstruisent une table (009, 016) en dependent : envelopper naivement toute la migration dans une transaction desactive la desactivation des FK sans le dire, et le `DROP TABLE` s'execute contraintes actives. `run_versioned_migration` isole desormais les `PRAGMA` en tete et en queue du fichier SQL (executes hors transaction) et n'enveloppe que le reste dans `BEGIN`/`COMMIT`/`ROLLBACK`. Verifie empiriquement au passage : contrairement a PostgreSQL, SQLite ne corrompt pas la transaction apres une erreur ignoree (`duplicate column`, etc.) — les instructions suivantes s'executent normalement. Cf. `tests/integration/migration_atomicity.rs`.

<a id="p31"></a>
## P31. Un rendu PDF qui s'arrete en bas de page tronque en silence

Le tableau de la fiche de mensurations perdait 6 mesures sur 47 (chevilles gauche/droite, entrejambe, longueur exterieure, montant, bas de pantalon — les mesures necessaires pour couper un pantalon) via un `if y < 20.0 { break }` qui arretait net le rendu en bas de page. Toute boucle de rendu PDF doit passer par `ensure_room` du module `commands/export/pdf/layout.rs`, jamais par un `break` sur la position verticale. Le PDF de la carte d'identite utilisait deja ce garde-fou ; celui de la fiche, ecrit plus tot, ne l'avait jamais recu.

<a id="p32"></a>
## P32. printpdf 0.12 ecrit un texte en litteral ou en hexadecimal selon son contenu

ASCII imprimable sans `(`, `)`, `\` ni `%` en litteral `(Synk...)`, tout le reste en hexadecimal WinAnsi (`<53796E6B...>`, `printpdf-0.12.8/src/serialize.rs` `needs_hex_encoding`). Un test qui ne cherche qu'une forme passe a vide sur une assertion negative. Chercher par `pdf_has_text` (`tests/integration/pdf_text/mod.rs`), qui teste les deux ; les figures posees se comptent par `page_one_xobject_draws` (operateurs `Do` de la page 1, lus par lopdf) et se mesurent par `page_one_images` (`/Subtype/Image`, taille d'origine, masque `SMask`) et `page_one_draw_matrices` (taille posee sur la page).

<a id="p33"></a>
## P33. Un `INSERT OR IGNORE` sans la colonne UNIQUE ne deduplique rien

Quatre fonctions d'import omettaient `withings_grpid` / `withings_id` (`commands/export/json/import_tables.rs`), or SQLite ne considere jamais deux NULL comme egaux dans une contrainte UNIQUE. Chaque reimport dupliquait donc les donnees en annoncant « 0 ignorees ». L'export (`SELECT *`) contenait pourtant ces colonnes depuis toujours. Toute requete d'import doit inclure la colonne qui porte la contrainte : `withings_grpid` (UNIQUE sur `body_measures_legacy` et `blood_pressure`, UNIQUE(withings_grpid, vital_type) sur `vitals_legacy`), `withings_id` (UNIQUE(date, withings_id) sur `workouts`). Garde-fou : `tests/integration/import_dedup.rs`.

<a id="p34"></a>
## P34. Une copie de base en mode WAL sans checkpoint peut etre vide

Les ecritures vivent dans `synk.db-wal` jusqu'au checkpoint. Toute copie passe par `copy_database_safely` (`commands/export/backup.rs`), jamais par un `std::fs::copy` direct. Corollaire : `restore_latest_backup_inner` ne recopie que le `.db`, donc le fichier principal d'une sauvegarde doit se suffire a lui-meme. A l'inverse, elle deplace le `-wal` et le `-shm` de la base courante avec elle (`synk.db.broken-<horodatage>-wal`) : laisses en place, SQLite les rejouait sur la sauvegarde restauree (mesure du 2026-09-28 : les lignes de l'ancienne base reapparaissaient). Si l'un des deplacements echoue, ceux deja faits sont annules et la restauration s'arrete (`tests/integration/restore_backup.rs`).

<a id="p35"></a>
## P35. Une base vue depuis l'app Claude n'est pas forcement la vraie

L'app Claude desktop est un paquet MSIX qui virtualise `AppData` pour tout son arbre de processus (sessions, sous-agents, serveurs MCP lances par `npx`). Un fichier cree depuis l'app sous `%APPDATA%\com.synk.health\` atterrit dans `%LOCALAPPDATA%\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\com.synk.health\` et masque le fichier reel du meme nom, alors que les `-wal`/`-shm` qui n'existent que sur le disque reel transparaissent. Le 2026-03-29, un Synk lance depuis l'app y a laisse une base vide : jusqu'au 2026-09-25, toute session lisait cette base avec le WAL reel et SQLite repondait `malformed database schema (sqlite_autoindex_heart_records_v2_1) - orphan index`, pendant que l'app installee lisait une base saine. Le diagnostic « base corrompue » de septembre en est sorti, la restauration n'y etait pour rien (note du 2026-10-04 dans `_archive/2026-10-26_plans-livres/2026-09-25-finition-chantiers-en-cours.md`). Avant de conclure a une corruption, regarder le chemin explicite du calque et relire la base depuis un processus lance par l'Explorateur (`Start-Process explorer.exe script.cmd`), hors conteneur. Ne jamais ecrire dans la vraie base depuis l'app : un checkpoint lance dans le conteneur melangerait un WAL du calque et le vrai fichier.

Mesure du 2026-11-01 : l'identite de paquet ne dit rien de ce calque, `GetCurrentPackageFullName` rend 15700 (aucun paquet) dans une session comme dehors, alors qu'un fichier neuf ecrit depuis la session n'existe, vu de dehors, que dans `LocalCache\Roaming`. La vraie base etait restee au 2026-06-28, sans jeton Withings, parce que les lancements depuis des sessions ecrivaient leur journal dans le calque. Depuis la 3.1.0, un build de release se relance seul hors du conteneur (D81) ; le piege reste entier pour la copie de dev, pour les versions anterieures et pour tout script qui lit la base depuis une session.

<a id="p36"></a>
## P36. sqlx ferme la derniere connexion APRES `pool.close()`

Avec plusieurs pools sur la meme base, `-wal` et `-shm` restent 4 a 17 ms et disparaissent seuls (mesure du 2026-10-22, 8 processus en parallele). Un code qui les teste puis les deplace doit tolerer `NotFound` (`move_aside_with`, `restore.rs`) ; c'etait N-05.

<a id="p37"></a>
## P37. Windows ne libere pas un fichier de facon synchrone

Apres fermeture d'un pool SQLite, un renommage echoue encore un instant avec l'erreur 32 (ERROR_SHARING_VIOLATION). Fermer le pool ne suffit pas, il faut reessayer — voir `retry_on_sharing_violation` (5 tentatives, attente doublant depuis 50 ms, uniquement sur l'erreur 32 : une permission refusee ne se resoudra pas d'elle-meme et doit remonter tout de suite).

<a id="p38"></a>
## P38. `PRAGMA wal_checkpoint` annonce son refus dans son jeu de resultats, pas par une erreur

Il renvoie une ligne `(busy, log, checkpointed)`, et `sqlx::query(...).execute(...)` jette cette ligne. Un `if let Err(...)` ne verra donc jamais un checkpoint refuse par une transaction concurrente. Le code de sauvegarde ne checkpointe plus (voir `VACUUM INTO` plus haut) ; tout nouvel appel doit lire le resultat avec `fetch_one`, pas `execute`.

<a id="p39"></a>
## P39. `anchor.y` du catalogue de mensurations se compte depuis les PIEDS

(0 = plante des pieds, 100 = sommet du crane), a l'inverse de l'axe SVG qui descend depuis le haut. Les deux consommateurs — le calque de `BodyFigure` et le PDF de la fiche — avaient tous les deux ajoute une inversion de trop et placaient le libelle du crane au niveau des pieds. Repere de controle : l'ancre `height` vaut 97 et le sommet du crane de la figure est a y = 21 sur 700. Conversion centralisee dans `features/bodyProfile/anchorGeometry.ts` et `anchor_page_y` (`pdf/sheet.rs`), les deux testees.

<a id="p40"></a>
## P40. Un calque superpose en `absolute inset-0` se dimensionne sur son conteneur, pas sur l'image qu'il annote

Dans une cellule de grille, ce conteneur est etire a la hauteur de la colonne voisine. Mesure sur la page Fiche : conteneur 2585 px pour une image de 750 px, donc des pastilles jetees bien sous la silhouette et, avec `preserveAspectRatio="none"`, ecrasees en ellipses. Un calque doit porter le meme `viewBox` que l'image et son conteneur doit etre en `self-start`.

<a id="p41"></a>
## P41. `printpdf` place une figure par `Scale(w, h)` puis `Translate(tx, ty)`

(`printpdf-0.12.8/src/xobject.rs`, meme ordre qu'en 0.7, pour un SVG comme pour une image) : `translate_y` est donc le bord **bas** de la figure, et l'axe PDF monte. Ne pas inverser une ordonnee deja comptee depuis le bas. Pour une image, `dpi` fixe la taille posee (`dpi_for_width`, `Scale` = pixels x 72 / dpi) : `the_sheet_pdf_carries_both_figures_unaltered` lit la matrice de chaque `Do`.

<a id="p42"></a>
## P42. Un commentaire n'est verifie par rien

Ce chantier a produit six commentaires affirmant un mecanisme faux, dont un contredit par une mesure du meme commit et un dernier trouve par la revue finale. Ils passent la compilation, les tests et la relecture, et survivent indefiniment. Un commentaire qui cite un chiffre doit pointer vers l'assertion qui le tient, ou dire qu'il n'y en a pas.

<a id="p43"></a>
## P43. Une option `undefined` écrase le défaut de TanStack Query

`useQuery({ staleTime: undefined })` vaut `staleTime: 0`, pas les 5 minutes de `createQueryClient`. `createCrudHook` ne pose `staleTime` que s'il est défini ; borne de période de `useTimeRange` = fin du jour (clé stable). Garde : `pages/Dashboard.test.tsx` (0 lecture au retour). Lectures IPC comptées au CDP par une enveloppe de `window.fetch` sur `ipc://localhost/<commande>` (`window.__TAURI_INTERNALS__.invoke` n'est pas inscriptible).

<a id="p44"></a>
## P44. CSP sans `ipc: http://ipc.localhost` = IPC replie sur postMessage en release, invisible en dev

(R17) : la CSP ne s'applique pas au serveur de dev sous Windows ; `npm run parcours -- --exe=...` sur un build `--no-bundle` de `tauri.dev.conf.json` voit le repli.

<a id="p45"></a>
## P45. Build release

NSIS uniquement depuis 2.14.0 ; une release passe par `npm run release -- X.Y.Z` (R29), jamais `npm run tauri build` a la main sauf diagnostic ; ne jamais `rm -rf src-tauri/target` (l'installeur du dernier build y est)

<a id="p46"></a>
## P46. Ne pas renseigner `bundle.publisher`

L'editeur affiche par Windows vaut « synk » (deuxieme segment de l'identifiant), et le modele NSIS de Tauri range le dossier d'installation sous `Software\<editeur>\Synk` (`MANUPRODUCTKEY`), qu'il relit pour retrouver l'installation precedente. Changer l'editeur couperait la mise a jour en place.

<a id="p47"></a>
## P47. Monter Tauri = monter ses crates internes ensemble

`cargo update -p tauri` seul ne bouge rien (« Locking 0 packages ») des que la version visee exige un nouveau `tauri-build`, `tauri-codegen` ou `tauri-macros`, et `--precise` sans autre precaution tire la ligne suivante de `tauri-runtime`, `tauri-utils` et `wry` (constate le 2026-10-02 : 2.11.1 amenait wry 0.57, tauri-runtime 2.12.1 et des doublons de `webview2-com`, `html5ever`). Poser chaque crate a la version du verrou de la release Tauri visee (lu dans l'audit cargo de la release GitHub `tauri-v2.x.y`), une par une : `tauri-utils`, `tauri-runtime`, `tauri-runtime-wry`, `tauri-macros`, `tauri-build`, `tauri`, puis redescendre `tauri-codegen` et `tauri-utils` si l'etape des macros les a montes. Le `rust-version` du `Cargo.toml` (1.94 depuis sqlx 0.9) ne freine rien : sous le resolveur 2, cargo ne l'utilise pas pour choisir les versions (simulation du 2026-10-03) ; seuls `--precise` et les exigences en `~` retiennent la ligne 2.11. CLI npm et `@tauri-apps/api` restent sur la meme mineure que le crate `tauri`.

<a id="p48"></a>
## P48. vitest 5 ne charge pas les types Node

(R22) : `tsconfig.json` limite les types globaux a `vitest/globals` ; les matchers de jest-dom 7 se declarent par l'import `@testing-library/jest-dom/vitest` de `test-setup.ts` (`Matchers<R, T>` de `vitest`, plus `jest.Matchers`), et `noUncheckedIndexedAccess` est actif (un index vaut `T | undefined` : garde ou valeur par defaut, jamais `!`). `tsconfig.node.json` declare `types: ["node"]` pour `vite.config.ts`. Vite 8 : `build.rolldownOptions`, `import.meta.dirname`, pas de groupe `codeSplitting` (il alourdit le chemin critique, `docs/dev-guide.md`). Ne pas ajouter `node` a cette liste (l'app tourne dans une WebView, `process` ou `Buffer` y typeraient sans erreur) : un test qui lit un fichier du depot passe par un import Vite `?raw` (`lib/errors.contract.test.ts`). Un mock passe a une fonction typee se type sur sa signature, `vi.fn<Config["list"]>()`, plus par `ReturnType<typeof vi.fn>`.

<a id="p49"></a>
## P49. Icone d'installeur et remplissage NSIS

`makensis` agrandit chaque image de `MUI_ICON` jusqu'a la taille de l'image correspondante de l'icone du desinstalleur, pour pouvoir l'y reecrire a l'installation. Tant que le desinstalleur gardait l'icone de NSIS, les images de 24, 48 et 64 px etaient suivies d'octets nuls : une comparaison octet pour octet les donnait pour differentes alors que Windows les rendait a l'identique (`PrivateExtractIconsW`, ecart 0). Depuis que les deux portent `icon.ico`, plus aucun remplissage (mesure du 2026-10-02). Garder la meme icone des deux cotes.

<a id="p50"></a>
## P50. Un libelle affiche peut etre une cle

(passe d'accents du 2026-10-02) : les tables du tableau de bord etaient indexees par le titre de carte (`Activite`) et les erreurs Rust reconnues a leur prefixe texte ; accentuer le libelle cassait les deux sans erreur de type ni test rouge. Depuis R26 (2026-10-25) il n'y a plus de protocole texte avec Rust : `AppError` se serialise en `{ kind, message, detail }` et `lib/errors.contract.test.ts` tient la liste des variantes. Cartes indexees par `CardConfig.id` (`pages/Dashboard/helpers.test.ts`). Avant de modifier un texte, chercher s'il est compare ou utilise comme cle.

<a id="p51"></a>
## P51. Polices integrees des PDF = WinAnsi

Printpdf encode le texte des polices Helvetica integrees en WinAnsi (`printpdf-0.12.8/src/serialize.rs` `encode_win_ansi`), qui accepte les accents francais, « œ » et « Â ». Le helper de test `pdf_hex` encode donc en Latin-1 (« é » = `E9`), pas en UTF-8.

<a id="p52"></a>
## P52. Le rapport PDF tronquait doublement jusqu'en 2.15.0

`LIMIT 15` par categorie et `if y < 40.0 { break }` en bas de premiere page. Meme famille que la fiche de mensurations : tout rendu passe par `ensure_room`.

<a id="p53"></a>
## P53. La porte d'import est commune à tous les tests d'intégration

`db/gate.rs` compte les imports en cours dans un `static`, et le binaire `tests/integration/` lance ses tests en parallèle dans un seul processus : l'import d'un test ferme la porte pour tous les autres. Sur la 3.0.0, `sync_one` répondait « Un import est en cours » au test `syncing_one_source_refuses_an_unknown_or_file_source_and_skips_a_disconnected_one` pendant le gros export Apple Santé d'un autre test (passe santé d'Intendance du 2026-10-26, rejouée le 2026-10-28). Tout test qui lance un import prend `support::import_gate::importing()`, partagé ; tout test qui lit la porte (`ensure_idle`, `busy_timeout_ms`) prend `no_import_running()`, exclusif.

<a id="p54"></a>
## P54. printpdf reduit en silence une image de plus de 2 Mo

`PdfSaveOptions::default()` porte `ImageOptimizationOptions { max_image_size: Some("2MB"), .. }` : a l'enregistrement, toute image estimee au-dela (largeur x hauteur x octets par pixel) est reechantillonnee au plus proche voisin, sans avertissement (`printpdf-0.12.8/src/image.rs`, `optimize` puis `resize_to_fit_size`). Mesure : une image RVB de 1000 x 1000 ressortait en 836 x 836 dans le PDF. Les figures de la fiche (niveaux de gris + alpha, 1,4 Mo) passent sous le plafond, mais il mord des qu'on livre une figure plus grande ou en couleur. `canvas.rs` le retire (`max_image_size: None`), tenu par `keeps_an_image_above_two_megabytes_at_its_original_size` (vu rouge avec le plafond remis). Une image RVB toute grise, elle, passe en niveaux de gris (`auto_optimize`) sous le plafond et ne prouve rien : le garde utilise des canaux differents.
