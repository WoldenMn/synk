# Historique de developpement — Synk

> Sorti de `CLAUDE.md` le 2026-09-10. Ces 29 sections y etaient chargees a chaque
> session alors qu'elles racontent le passe : ~43 000 caracteres, environ 10 800
> tokens par session. Elles sont conservees telles quelles, sans reecriture.
>
> `CHANGELOG.md` couvre 23 des 24 versions numerotees citees ici. **V2.11.0 fait
> exception** : son recit n'existe que dans ce fichier, le CHANGELOG ne la mentionne
> qu'en passant dans une entree de correctif. Ne pas supprimer ce fichier en croyant
> le CHANGELOG redondant.

## Finition 2.15.0 (branche `finition/2.15.0` — 2026-10-02)

Solde des chantiers restes ouverts apres la 2.14.3 (installee le 2026-09-26) :
correctif des plages de dates (PR #6), retrait de l'e-mail personnel, sauvegarde
par `VACUUM INTO`, puis une passe de finition. Defauts trouves en route, tous
silencieux : rapport PDF tronque deux fois (`LIMIT 15` par categorie et arret en
bas de premiere page), boutons « Reessayer » et « Restaurer » de l'ecran d'erreur
qui ne relancaient pas le boot, migration lancee meme quand sa sauvegarde
echouait, restauration qui pouvait poser une sauvegarde illisible, hydratation
affichee en % au lieu de kg, durees des records en secondes etiquetees « min »,
valeurs brutes (`21.647000000000002 %`) sur la carte d'identite, 37 resultats
ignores par `let _ =` retires (deux restaient sur les traces de suppression, retires en R09 ; le lint `clippy::let_underscore_must_use` en `deny` tient la regle depuis), liens de la barre laterale deux par ligne. Ajouts :
identite visuelle (icone generee, logo), etats vides par periode, bandeau Withings
non connecte, interface accentuee et vouvoyee, Parametres et Export sur deux
colonnes. La passe d'accents a elle-meme casse deux contrats implicites (prefixes
d'erreur Rust lus par `errors.ts`, tables indexees par titre de carte), rattrapes
et desormais gardes par un test. L'installeur NSIS affichait l'icone generique de
NSIS et ses textes anglais depuis toujours (installeurs 2.14.3 et 2.15.0 mesures) :
il recoit l'icone, deux bitmaps generes, le francais et une mention de copyright.
Run G du plan du 2026-09-25 (dependances), d'abord refuse par le garde-fou de la
session puis lance sur demande de l'utilisateur : tauri 2.11.2 et ses crates
internes poses un par un sur le verrou de la release Tauri (un `cargo update -p
tauri` seul ne bougeait rien), CLI 2.11.5, react-router 7.18.4, vite 6.4.3,
vitest 4.1.11 (deux fichiers de test retypes, aucun code applicatif touche),
openssl, serde_with. 43 des 44 alertes Dependabot fermees au verrou, glib (Linux)
restante. Le parcours des 19 pages en dev (0 erreur console) a revele 5 faux
avertissements par demarrage, nes de la sortie des `let _ =` : le rejeu de
`MIGRATION_001` cree des index sur des tables devenues vues. Classes en `debug`,
gardes par un test vu rouge puis vert. Detail et chiffres : `CHANGELOG.md` [2.15.0].

## Etat anterieur (branche `fix/integrite-donnees-et-recuperation` — integrite des donnees et recuperation — 2026-08-03)

> Depuis ce chantier, releases 2.14.0 -> 2.14.2 le 2026-08-03 sur `main`
> (bundle NSIS uniquement, plus de MSI ; correctifs et redessin de la planche
> anatomique). Version courante : **2.14.2** (dernier commit `7797a58`).

Chantier de correction, **mergee sur `main`, aucun bump de version au merge**
(2.13.0 inchange). Batie sur `main` (`7672aca`). 10 commits, aucune feature ajoutee,
aucun fichier frontend touche : quatre defauts d'integrite des donnees, tous
silencieux — rien ne plantait, les tests passaient, les fichiers produits
avaient l'air corrects.

**L'import JSON dupliquait les donnees de 4 tables sur 11.** Quatre fonctions
d'import omettaient la colonne d'identifiant Withings — `withings_grpid` pour
`body_measures_legacy`, `blood_pressure` et `vitals_legacy`, `withings_id` pour
`workouts` — precisement celle qui porte la contrainte `UNIQUE`. SQLite ne
considerant jamais deux `NULL` comme egaux dans une contrainte `UNIQUE`,
l'`INSERT OR IGNORE` ne detectait rien. Reimporter la meme sauvegarde doublait
les donnees en annoncant « 0 ignorees » ; restaurer puis resynchroniser Withings
creait une seconde ligne au lieu de mettre a jour la restauree. L'export, lui,
contenait ces colonnes depuis toujours (`SELECT *`) : seul l'import les jetait.

**La sauvegarde automatique d'avant migration pouvait etre perimee.** La base
tourne en WAL : les ecritures vivent dans `synk.db-wal` jusqu'au checkpoint.
Deux fonctions copiaient la base, une seule faisait le checkpoint —
`backup_before_migration`, celle dont la seule raison d'etre est de proteger
d'une migration ratee, ne le faisait pas. Corrige en extrayant une fonction
unique `copy_database_safely` appelee par les deux, plutot qu'en recopiant les
lignes manquantes : la cause racine etait la duplication d'une logique delicate
a deux endroits.

**La restauration echouait sur le verrou de fichier Windows.** `boot::run`
laissait le pool ouvert en sortant en erreur, et le bouton « Restaurer »
renommait un fichier encore tenu (`os error 32`). Fermer le pool ne suffit pas :
Windows ne libere pas le handle de facon synchrone. Helper
`retry_on_sharing_violation` (5 tentatives, attente doublant depuis 50 ms,
uniquement sur l'erreur 32) plus fermeture explicite du pool en defense.

**Un checkpoint WAL refuse passait sans un mot.** `PRAGMA wal_checkpoint`
renvoie son statut `busy` dans son jeu de resultats, pas comme une erreur SQL,
et `sqlx::query(...).execute(...)` jette ce jeu de resultats. Un checkpoint
refuse par une transaction concurrente passait donc inapercu. Comme
`restore_latest_backup_inner` ne recopie que le `.db` (jamais le `-wal`), la
sauvegarde perdait ses dernieres donnees en silence. Corrige par
`classify_checkpoint` / `checkpoint_wal` / `checkpoint_and_log`, comportement
non fatal preserve (mieux vaut une sauvegarde imparfaite que pas de sauvegarde
au moment de migrer) : le refus devient seulement visible dans les logs.

**Points ouverts, suivis separement** :
- La restauration renomme `synk.db` en `synk.db.broken-<horodatage>` puis
  recopie le `.db` de la sauvegarde, sans toucher aux `synk.db-wal` /
  `synk.db-shm` de l'ancienne base, qui restent a cote du fichier restaure. Le
  risque est identifie mais **n'a pas ete verifie**.
- La litiere `-wal`/`-shm` du dossier de sauvegardes, que le nettoyage rolling
  (5 fichiers) ne supprime jamais.
- Les bases deja polluees par un import anterieur au premier correctif.
- Le decoupage des fichiers au-dessus de 300 lignes (cf. tailles ci-dessous).

Chiffres comptes dans le code (et non recopies) :
- **10 commits** (`git rev-list --count main..HEAD`), 3 fichiers de production
  modifies + 2 fichiers de tests crees (`git diff --name-only main..HEAD`).
- **131 tests Rust lib inline** verts (`cargo test --lib -j 2`), dont 8 ajoutes
  par ce chantier, tous dans `backup.rs` (classification du checkpoint, retry
  sur l'erreur 32).
- **213 tests d'integration** verts, repartis sur les **35 suites** de
  `src-tauri/tests/`, lancees une par une, 0 echec. 5 ajoutes par ce chantier
  (`backup_integrity.rs` 2, `import_dedup.rs` 3). Total Rust : **344**.
- **367 tests vitest / 68 fichiers verts**, inchange — aucun fichier frontend
  touche. `tsc --noEmit` propre, eslint 0 erreur (12 warnings preexistants,
  `no-empty-function` dans `MeasurementWizard.test.tsx`).
- **101 commandes Tauri**, inchange : `copy_database_safely` est une fonction
  publique, pas une commande.
- **16 migrations**, inchange (`EXPECTED_LATEST_VERSION` = 16).
- Tailles : `commands/export/backup.rs` **111 -> 426 lignes**, dont **207 de
  tests inline** (deux modules `#[cfg(test)]`, lignes 220 a 426) — 219 lignes de
  code de production, sous le seuil.
  `commands/export/json/import_tables.rs` **337 -> 341**, au-dessus du seuil
  bloquant de 300, deja avant ce chantier. `boot.rs` **269 -> 279**. `db.rs`
  **430 lignes**, non touche ici.
- `cargo clippy --all-targets -j 2 -- -D warnings` et `cargo fmt --check`
  silencieux.
- Correction au passage : l'arborescence Architecture annoncait `db.rs` a
  418 L, mesure **430** — le fichier a grossi apres l'ecriture de la section
  precedente. Les chiffres de la section « Etat anterieur » ci-dessous
  (10 commits, 418 lignes, 329 tests) etaient exacts au commit `b5d6d7e` qui les
  a ecrits, et ont ete depasses par deux commits ulterieurs de la meme branche
  (13 commits au total, `db.rs` a 430, +2 tests dans `migration_atomicity.rs`).
  Non reecrits : une section datee vaut comme instantane.

## Etat anterieur (branche `feat/finition-fiche-et-carte` — finition fiche et carte — 2026-08-02)

Chantier de correction, **mergee sur `main`, aucun bump de version** (2.13.0
inchange). Batie sur `feat/carte-identite-chiffree`. 10 commits, aucune feature
ajoutee : corrige deux defauts silencieux — l'application ne plantait pas, les
tests passaient, les documents produits avaient l'air corrects — et solde trois
dettes techniques, plus une passe visuelle sur la planche anatomique.

**Le PDF de mensurations perdait six mesures.** `render_sheet` remplissait le
tableau de la page 2 et s'arretait net en bas de page (`if y < 20.0 { break }`)
au lieu de continuer sur une page suivante : 41 mensurations sur 47 ecrites, en
silence. Les six perdues (chevilles gauche et droite, entrejambe, longueur
exterieure, montant, bas de pantalon) sont les quatre mesures necessaires pour
couper un pantalon : la fiche destinee a un tailleur etait inutilisable, sans que
rien ne l'indique. Le PDF de la carte d'identite utilisait deja `ensure_room`
pour deborder proprement ; celui de la fiche, ecrit plus tot, ne l'avait jamais
recu. Corrige en extrayant `ensure_room`/`wrap_text`/`Cursor` dans un module
partage (`commands/export/pdf/layout.rs`) et en y migrant `sheet.rs` : le tableau
deborde desormais sur une 3e page (`/Count 3`, teste directement).

**Le wizard enregistrait des mesures non prises.** Chaque champ etait pre-rempli
avec la derniere valeur connue, et l'enregistrement ecrivait toute valeur non
vide, datee du jour. Mesurer une seule chose reenregistrait donc tout
l'historique deja connu a la date du jour : dates falsifiees, courbes
d'evolution aplaties par les doublons, recapitulatif trompeur (annoncait le
nombre de mesures deja connues, pas celui reellement retape). Corrige : la
derniere valeur s'affiche en texte fantome (`placeholder`), le champ reste vide
tant que rien n'est saisi, seul ce qui est reellement tape est envoye vers
`add_measurement_session`.

**Trois dettes soldees** :
- `audit_purge.rs` — 2 tests casses depuis que `2026-05-01` est sorti de la
  fenetre de 90 jours (dates codees en dur). Remplaces par des dates relatives a
  l'instant du test (helper `days_ago()`).
- **Migrations non atomiques** — une migration qui reconstruit une table
  (`PRAGMA foreign_keys=OFF` + `DROP`/`RENAME`) pouvait echouer a mi-chemin sans
  rollback possible. `run_versioned_migration` isole desormais les `PRAGMA` en
  tete/queue (executes hors transaction, cf. Pieges connus) et enveloppe le reste
  dans `BEGIN`/`COMMIT`/`ROLLBACK`, version ecrite dans la meme transaction que le
  reste. 4 nouveaux tests (`tests/migration_atomicity.rs`).
- **Trois promesses du design du wizard jamais implementees** — navigation
  clavier (`Entree` passe au champ suivant, puis a l'etape suivante), barre de
  progression cliquable (sauter a une etape deja visitee), et un bouton
  « passer » finalement non ajoute : le correctif de pre-remplissage l'a rendu
  inutile (plus rien a confirmer d'une touche des lors qu'il n'y a plus de
  valeur pre-remplie a valider).

**Passe visuelle** sur la planche anatomique (`src/assets/body/{front,back}.svg`)
: epaules elargies de 2,15 a 2,39 largeurs de tete, mains dessinees (le tour de
main fait partie du catalogue), ancres bras/epaule recalculees. Generateur
versionne (`scripts/gen-body-plate.js`, `npm run plate:gen`) : la planche est
produite depuis des points de controle, jamais editee a la main.

Chiffres verifies dans le code (et non recopies) :
- **101 commandes Tauri**, inchange par ce chantier. Au passage : deux mentions
  de « 102 » dans ce fichier (arborescence Architecture, titre de section) sont
  corrigees ici — deja fausses avant ce chantier, sans lien avec lui.
- **16 migrations**, inchange (`EXPECTED_LATEST_VERSION` = 16).
- **`src-tauri/src/db.rs` : 371 -> 418 lignes**, au-dessus du seuil bloquant de
  300. Deja au-dessus avant ce chantier (371 lignes). Dette assumee, non traitee
  ici : le decouper a l'occasion d'un correctif sans rapport aurait ete du
  refactoring opportuniste sur le code le plus sensible de l'application
  (migrations). A traiter separement.
- **329 tests Rust verts** (123 lib inline + 206 integration), 0 echec toutes
  suites confondues — `audit_purge` (2 echecs preexistants) est desormais vert,
  une premiere sur cette lignee de features. Le compte precedemment documente
  sur `feat/carte-identite-chiffree` (114 lib inline / 310 total) etait deja
  incorrect avant ce chantier : aucune modification de ce chantier n'ajoute ou
  ne retire de test lib inline (les 3 tests de `wrap_text` dans `layout.rs` sont
  deplaces depuis `profile/layout.rs`, pas crees ; `sheet.rs` conservait deja
  ses 5 tests inline). Mesure directe (`cargo test --lib`) : 123 tests, autant
  qu'avant ce chantier.
- **367 tests vitest / 68 fichiers verts** (362 avant, +5 : tests wizard
  pre-remplissage/enregistrement honnete, navigation clavier, progression
  cliquable).

## Etat anterieur (branche `feat/carte-identite-chiffree` — carte d'identite chiffree — 2026-08-02)

Feature de branche, **mergee sur `main`, aucun bump de version**. Batie sur
`feat/mensurations-sur-mesure` : depend de son catalogue de
mensurations et de sa table `clothing_sizes`. Remplace la page **Rapports** par une
page **Profil** : carte d'identite chiffree qui rassemble en un ecran tout ce que
Synk **peut deriver lui-meme** (corps, mensurations cles, traitements en cours,
tailles, records) et l'exporte en PDF.

Reduction de perimetre decidee par l'utilisateur le 2026-08-02 (meme jour, apres
l'implementation initiale) : les 9 champs saisis a la main (date de naissance,
lateralite, couleurs des yeux/cheveux, groupe sanguin, allergies, correction
visuelle, antecedents, vaccins) sont retires de la carte — avec eux le formulaire
d'edition, la commande `update_profile_identity` et le calcul d'age depuis la date
de naissance. L'age redevient celui saisi dans le profil des Parametres (colonne
`age`, preexistante). Les migrations 015/016 restent en place (additives, colonnes
desormais inutilisees mais sans risque a supprimer).

- **Agregation unique cote Rust** : `commands::profile_card::get_profile_card_inner`
  rassemble 5 blocs (corps, mensurations cles, traitements en cours, tailles,
  records) depuis 5 requetes paralleles (`tokio::try_join!` : profil age/sex/
  height_cm, derniere pesee, mensurations cles, traitements actifs, tailles) + le
  fetch des records. L'ecran (`pages/Profile.tsx`) ET le PDF
  (`export_profile_card_pdf`) appellent cette meme fonction : ils ne peuvent pas
  afficher des choses differentes.
- **Records via `ORDER BY value LIMIT 1`, jamais `MIN()`/`MAX()`** : un agregat SQL
  donne la bonne valeur mais perd la date a laquelle elle a ete atteinte. 6 records
  suivis (poids min/max, meilleur total de pas, plus longue nuit, meilleur score de
  sommeil, FC la plus basse, entrainement le plus long).
- **`commands/profile.rs` redevient un fichier unique** : le split `commands/profile/`
  (`mod.rs` + `identity.rs`) n'existait que parce que `identity.rs` faisait
  depasser le seuil des 300 lignes. Sans lui, plus de raison de garder un dossier
  pour un seul fichier — retour a `commands/profile.rs` (age/sex/height_cm
  uniquement, comme avant ce chantier).
- **Une seule commande d'ecriture sur `user_profile`** : `update_user_profile`
  (age/sex/height_cm, page Parametres). Pas de second point d'ecriture a proteger.
- **Tailles estimees persistees** (migration 016, CHECK `clothing_sizes.source`
  elargi a `estimated`) : l'assistant Tailles enregistre desormais ce qu'il calcule
  via `save_estimated_sizes` ; clause `WHERE source = 'estimated'` sur l'UPSERT
  protege les corrections manuelles de l'utilisateur (une estimation ne remplace
  qu'une estimation precedente).
- **PDF qui deborde plutot que de tronquer** : `wrap_text` (coupe aux espaces,
  jamais au milieu d'un mot) + `ensure_room` ajoutent une page « Suite » si le
  contenu depasse — un document medical ampute en silence est pire qu'un document
  a 2 pages.
- **Health Score supprime** : la carte est un etat des lieux factuel, pas une note
  synthetique a formule arbitraire. `HealthScoreRing`, devenu sans usage, supprime
  avec (page Rapports et ses sections retirees).
- **3 nouvelles commandes Tauri** (98 -> 101) : `get_profile_card`,
  `export_profile_card_pdf`, `save_estimated_sizes`.

Tests : 114 tests Rust lib inline (+3 vs `feat/mensurations-sur-mesure`) + 13 tests
d'integration `profile_card.rs` + `clothing_sizes.rs` 7 (tailles estimees). Suite
Rust complete : 310 tests verts / 2 echecs preexistants sans lien avec cette feature
(dates codees en dur dans `tests/audit_purge.rs`, deja signale sur
`feat/mensurations-sur-mesure`). 362 tests vitest (68 fichiers) verts. `cargo fmt
--check` silencieux sur l'ensemble du backend.

## Etat anterieur (branche `feat/mensurations-sur-mesure` — fiche de mensurations sur-mesure — 2026-07-31)

Feature de branche, **mergee sur `main`, aucun bump de version**. Ajoute une fiche de
mensurations sur-mesure : 47 mesures corporelles (tours, longueurs, largeurs, laterisation
gauche/droite sur 14 paires, 22 marquees `is_key`), saisissables via un assistant pas a pas
(2 parcours : Essentiel 16 etapes, Complet 33 etapes), consultables dans un nouvel onglet
« Fiche » (par defaut, page Mensurations passe de 2 a 3 onglets : Fiche/Tailles/Tours de
corps), exportables en PDF illustre (2 pages : silhouettes annotees + tableau detaille).

- **Catalogue partage** `src/features/bodyProfile/catalog.json` (47 entrees) : source unique
  lue par React (import Vite) et par Rust (`include_str!` dans `body_catalog.rs`) — memes
  coordonnees d'ancrage sur la fiche ecran (`BodyFigure`) et sur le PDF.
- **Seed Rust, pas migration SQL** : `seed_body_catalog` (`db.rs`) seede les 47 types
  APRES `migrate_to_v2_schema` (une migration versionnee classique echouerait :
  `measurement_types` est deja une vue a ce stade sur une base existante). Remplace
  `seed_builtin_inseam`. `EXPECTED_LATEST_VERSION` reste **14**, aucune migration 015.
  Garde-fou anti-collision avec les types personnalises de l'utilisateur (slug deja pris
  par un type `is_builtin = 0` -> `warn` + skip, sinon perte silencieuse du statut perso).
- **3 nouvelles commandes Tauri** (95 -> 98) : `add_measurement_session` (ecriture
  transactionnelle d'un releve complet, validation des bornes avant ouverture de la
  transaction), `get_body_sheet` (derniere valeur par mensuration), `export_body_sheet_pdf`.
- **PDF vectoriel** via `printpdf` feature `svg` (pas `embedded_images`) : silhouettes
  `src/assets/body/{front,back}.svg` embarquees en vectoriel, libelles ecrits cote Rust
  (`usvg`/printpdf ignore les `<text>` d'un SVG source). Frontiere fetch async (sqlx) /
  rendu sync (printpdf `Rc<RefCell>` non-Send) preservee.
- **Frontend** : `MeasurementWizard` (dialog, choix de parcours + etapes + recap),
  `WizardStepView`, `WizardRecap`, `BodyFigure` (silhouette + points d'ancrage SVG),
  page `pages/measurements/SheetView.tsx`. Confirmation (`useConfirm`) avant de perdre
  une saisie en cours a la fermeture du wizard.

Tests : 111 tests Rust lib + 16 tests d'integration `body_profile.rs` (dont seed,
idempotence, garde-fou collision, validation, PDF non vide) verts. Suite Rust complete :
291 tests verts / 2 echecs preexistants sans lien avec cette feature (dates codees en dur
dans `tests/audit_purge.rs`, deviennent fausses avec le temps qui passe — signale
separement). 356 tests vitest (66 fichiers) verts.

## Etat anterieur (V2.12.1 — correctif connexion Withings — 2026-06-28)

Patch cible : le bug « Erreur : state invalide » sur `localhost:17483/callback` lors d'une
**reconnexion** Withings (onglet ferme, bouton « Reessayer », re-clic « Connecter » dans les
120s). Cause racine = serveur callback ephemere par appel incapable de re-binder le port,
l'ancien serveur validant le callback contre son ancien `state`. Fix : serveur callback unique
pour toute la vie de l'app + `state` partage toujours egal a la derniere tentative
(`OAuthFlow`/`OnceLock`), logique de validation extraite en `evaluate_callback` pure. Un seul
fichier touche (`src-tauri/src/withings/auth.rs`), aucune API publique modifiee, aucun
changement d'usage. Revue `code-quality-reviewer` : 0 bloquant (warning spin-loop sur erreur
`recv` corrige : `break` + reset du flag). 98 tests lib Rust (87 + 11 auth) + clippy/fmt verts.
Paquet `Desktop\Synk-v2.12.1\` (MSI+NSIS), 2.12.0 archive.

## Etat anterieur (V2.12.0 — finalisation pilotee par audit — 2026-06-27)

Chantier de finalisation (branche `release/v2.12.0`) : baseline brute + audit adversarial
multi-agents (4 axes : securite, backend Rust, frontend, docs/tests) puis correction de
TOUS les findings reels. 0 CRITICAL, 1 HIGH, ~12 MED, ~22 LOW. Aucune feature, aucun
changement d'usage. Workflow brainstorming → writing-plans → subagent-driven (9 lots TDD).

- **Backend** : export `resolve_output_dir` -> `Result` (plus de panic) ; import `source`
  defaut `'manual'` au lieu de `'import'` (le CHECK legacy rejetait silencieusement workouts/ECG
  et 4 autres tables) ; deletes alertes/medicaments atomiques ; `error_log` purge bornee +
  taille limitee ; OAuth durci (`state` via OsRng, troncature body, `is_connected` verifie
  l'expiration, commande callback morte supprimee) ; helper `opt_ts_to_iso` (skip+warn sur
  timestamp corrompu).
- **Frontend** : invalidation `["trend"]` post-sync ; observabilite erreurs etendue
  (Insights/Journal/Rapports/credentials + `formatError`) ; a11y (tri tableau clavier
  `aria-sort`, titres dialogue onboarding, libelles dates, `useId` sparkline) ; perf (memo) ;
  ecouteurs surs.
- **Dette/qualite** : mise en forme integrale (prettier + `cargo fmt`) ; splits > 300 L
  (`withings/sync/measures`, `commands/export/json`, `withings/sync`) ; docs resynchronisees
  (14 migrations, **95 commandes**, **39 tables/vues**, 26 features, `bootstrap_schema`) ;
  tests etendus (roundtrip 11 tables, restore backup, dual-write, cas OAuth).

253 tests Rust (lib 87 + integration) + 319 vitest verts. clippy 0, tsc/eslint/prettier/
cargo-fmt verts. Revue `code-quality-reviewer` : 1 bloquant (import workouts/ECG `'import'`)
corrige + verifie, puis APPROVED. CI/CD exclu (decision : app desktop manuelle). MSI non signe.

## Etat anterieur (V2.11.1 — patch post-audit — 2026-06-19)

Patch suite a un audit complet adversarial (3 agents : securite/integrite, regression,
edge cases) du code 2.11.0. 0 CRITICAL/HIGH ; corrections de coherence (branche
`fix/post-audit-2.11.1`) :
- **Bornes vitaux** (migration **014**, `EXPECTED_LATEST_VERSION` 13→14) : spo2/body_temperature/
  temperature elargies aux bornes physiquement possibles. La regression introduite en 2.11.0
  (B5/FONC-04 appliquait les bornes physiologiques « normales » au dual-write, rejetant les
  valeurs cliniques pathologiques : SpO2<70, hypothermie) est corrigee. Rejet NaN/Inf conserve.
- **Validation import JSON** : `import_json_inner` valide les bornes (= `add_manual_*`) sur 6 tables
  (rollback atomique) — un backup forge ne peut plus injecter d'aberrations.
- **`update_user_profile`** aligne sur les bornes import (age/taille) — sinon un profil saisi
  age=200 via l'UI rendait son backup non restaurable (bloquant trouve en revue).
- **Validation** add_manual_workout + heart_rate de la tension. **Frontend** : `mutate`+`onSuccess`
  au lieu de `mutateAsync` (plus d'unhandled rejection) dans Alerts/Labs/Medications.

78 tests inline Rust + integration + 319 vitest verts, clippy+tsc clean, revue APPROVED.

## Etat anterieur (V2.11.0 — finition dette technique — 2026-06-19)

Chantier de finition de dette via audit cible (4 axes) + 8 lots subagent-driven (TDD),
sur branche `v3-finition`. Workflow brainstorming → writing-plans → subagent-driven.
Spec/plan dans `docs/specs/` et `docs/plans/`.

- **Validation backend** : helper `commands/validation.rs` (`check_f64_opt`/`check_i64_opt`/
  `check_f64`) applique aux 5 `add_manual_*` (seule la tension validait avant). Bornes du
  dual-write propagees au legacy body/vital (`DefMeta.min/max` `pub(super)`).
- **Integrite** : dedup insights (garde journaliere dans `insert_insight` — table `insights`
  sans UNIQUE generait des doublons a chaque sync), normalisation `SUBSTR(date,1,10)` dans 3
  detecteurs, migration **013** temperature (type 12 visible, `resolve_metric_slug(12)`,
  `EXPECTED_LATEST_VERSION` 12 → 13), dedup `read_metric_series` sur `measured_at` seul.
- **Export/import** : import JSON complete aux 11 tables (ordre FK measurement_types →
  manual_measurements), colonnes CSV/PDF body etendues (`BmRow` 5 → 9 champs).
- **Frontend** : `features/{labs,medications,alerts}/hooks.ts` (React Query + `onError:
  reportError`) + 3 pages migrees (fin du `useState`+`loadData` manuel ; erreurs enfin dans
  le journal V2.10.0). `useSync` persiste les erreurs de sync partielle.
- **Refacto/tests** : helper `measured_at_iso`, helpers colonnes `metric_repository`,
  `decide_refresh` extrait+testable (REFRESH_LOCK preserve), durcissement tests.

69 tests inline Rust + 319 tests frontend verts. Revue finale APPROVED. 4 commits sur
`v3-finition` (`370fa9d` docs, `e94e886` backend, `70bc346` tests, `4e3a72b` frontend) +
bump 2.11.0. **Pieges** : CHECK `source IN ('withings','manual')` sur tables legacy avale
`'import'` via `INSERT OR IGNORE` ; pas de `;` dans les commentaires de migration (split
naif) ; tests inline via `cargo test --lib` (package=`synk`, lib=`synk_lib`).

## Etat anterieur (feature Tailles de vetements — 2026-06-17, branche feat/tailles-vetements)

Nouvel onglet « Tailles » dans la page Mensurations : estimation des tailles homme
(haut, chemise, veste, pantalon) depuis les mensurations existantes + pointure saisie,
conversions EU/US/UK, alerte de seuil, correction manuelle persistee. Calcul en fonctions
pures frontend (`src/features/clothingSizes/sizing/`, norme EN 13402). Nouvelle table
`clothing_sizes` (migration 011, `EXPECTED_LATEST_VERSION` → 11), mensuration built-in
`inseam`, 4 commandes Tauri. 14 tests frontend sizing + 4 tests Rust `clothing_sizes`.
Livree par la branche `feat/tailles-vetements`, mergee sur `main` en 2.9.0.

## Etat anterieur (V2.8.3 — 2026-05-30)

Corrections de bugs issues d'une chasse adversariale (2 MED + 2 LOW, aucun CRITICAL/HIGH). (1) `withings/client/session.rs:145` MED-1 : `save_tokens` apres refresh OAuth ne avale plus son erreur via `let _ =` — propagation `?` evite la session corrompue silencieuse (DB gardait ancien refresh deja invalide -> boucle 401 perpetuelle). (2) `hooks/useSync.ts` MED-2 : invalidation ajoutee pour `metric-series`/`metric-latest`/`metric-definitions` (page Explorer etait perimee apres sync). (3) `session.rs:136-137` LOW-3 : `vault.open_str(...).unwrap_or_default()` remplace par `.map_err(...)?` (fail-fast clair si vault corrompu). (4) `stores/syncStore.ts:37` LOW-4 : `safeRemoveItem` sur `setLastSyncAt(null)` (bug latent).

249 tests frontend + 136 tests Rust (inchanges) verts. 0 changement comportemental observable utilisateur (sauf messages d'erreur explicites au lieu de boucles silencieuses).

## Etat anterieur (V2.8.2 — 2026-05-29)

Corrections audit (Lot C, frontend pur) : (1) systeme ConfirmDialog promise-based (`src/components/ui/confirm-dialog/` : ConfirmProvider + useConfirm + ConfirmDialog sur le Dialog existant), 14 `window.confirm()` natifs remplaces (finding UX-001). (2) 9 charts Recharts accessibles : wrapper `role="img"` + `aria-label` dynamique (helper `charts/a11y.ts`) + prop `accessibilityLayer` (finding A11Y-001).

249 tests frontend (227 V2.8.1 + 22 : 8 ConfirmDialog + 14 charts a11y) + 136 tests Rust (inchanges) verts. 0 changement backend.

## Etat anterieur (V2.8.1 — 2026-05-27)

3 quick-wins post-audit factuel : (1) `[profile.release]` Cargo avec lto+codegen-units+strip+opt-level dans `src-tauri/Cargo.toml`, binaire synk.exe -13.1% (22.1 MB -> 19.2 MB). (2) `set_credentials_pair` resilient via `INSERT OR IGNORE INTO withings_credentials (id) VALUES (1)` avant UPDATE, adresse concern V2.8.0. (3) 3 nouveaux scripts npm (typecheck, outdated:check, build:analyze).

136 tests Rust (135 V2.8.0 + 1 nouveau `set_credentials_creates_row_if_missing`) + 227 tests frontend (inchanges) verts. 0 changement comportemental observable.

## Etat anterieur (V2.8.0 — 2026-05-27)

Securite : chiffrement au repos des 4 secrets OAuth Withings (access_token, refresh_token, client_id, client_secret) via AES-256-GCM. Cle maitre 32 bytes stockee dans Windows Credential Manager (DPAPI) via crate `keyring`. Module isole `src-tauri/src/crypto/` (~270 LOC). Boot etendu de 4 a 6 etapes (`crypto_init`, `migrate_plaintext_secrets`). REFRESH_LOCK et logique de rotation OAuth strictement preserves. UX inchangee.

135 tests Rust (107 V2.7.2 + 28 crypto-related) + 227 tests frontend (224 + 3 BootErrorScreen/errors) verts.

## Etat anterieur (V2.7.2 — 2026-05-15)

Patch perf : `get_dashboard_summary` parallelise via 2 `tokio::try_join!`
(6 queries latest + 4 queries prev). 10 queries SQLite sequentielles
(~80-100 ms) -> 2 groupes paralleles (~20-30 ms). Gain ~50-70 ms par
chargement Dashboard.

Pattern inner/wrapper applique : nouvelle fn `run_dashboard_summary(pool)
-> Result<DashboardSummary, AppError>` testable en isolation, wrapper
`get_dashboard_summary` (Tauri command) map AppError -> String. 10 helpers
prives extraits (`fetch_latest_*` / `fetch_prev_*`).

+2 tests integration `tests/dashboard_summary.rs` (DB vide + DB avec
donnees verifie calcul trends).

107 tests Rust (105 V2.7.1 + 2 nouveaux) + 224 tests frontend verts.

## Etat anterieur (V2.7.1 — 2026-05-15)

Patch perf : correction du N+1 introduit en V2.7.0 dans les fns dual-write
sleep/activity. Lookup `def_id` deplace HORS des boucles via nouveaux
helpers `lookup_sleep_def_id` / `lookup_steps_def_id`. Signature des fns
`upsert_*_metric(tx, def_id, ...)` modifiee (def_id en param). Pattern
applique aussi au pre-existant `sync_metrics::upsert_metrics_group` via
preload `HashMap<i64, i64>` (`preload_def_ids`).

Gain : ~0.5-1 s par cycle sync complet (180+ queries `SELECT id WHERE
slug=?` reduites a 1 par cycle pour sleep/activity, et 1 unique HashMap
preload pour metrics).

105 tests Rust (103 V2.7.0 + 2 helper tests) + 224 tests frontend verts.

## Etat anterieur (V2.7.0 — 2026-05-15)

Stack metriques V2 enfin complet. Les 5 slugs orphelins V2.6.1
(`systolic`, `diastolic`, `heart_rate`, `sleep_score`, `steps`) sont
desormais integres au stack V2 via migration 009 (3 nouvelles categories
`vital_bp`, `vital_sleep`, `vital_activity`). `alerts.rs::get_latest_metric_value`
devient 8 LOC (60 LOC -> 8 LOC), 100% delegue au helper, plus de
frontiere V2/legacy. Struct `LatestValue` supprimee (dead code).

Sync etendue : `sync_metrics::METRIC_MEASURE_TYPES` ajoute 9/10/11
(BP ecrit dans `metrics` automatiquement via /measure existant).
`sync_sleep` et `sync_activity` font dual-write via `upsert_sleep_metric`
et `upsert_activity_metric` (fonctions pures extraites pour testabilite,
appelees dans la meme transaction que l'INSERT legacy).

`metric_repository::legacy_sql_latest_for_category` et `read_metric_series`
etendus aux 3 nouvelles categories. `sleep_records.date` et `activity.date`
utilises comme `measured_at` alias (format YYYY-MM-DD lexicographiquement
comparable). `db.rs::run_versioned_migration` modifie pour acquerir une
seule connexion (les PRAGMA sont per-connection, requis pour rebuild de
table SQLite).

Migration 009 utilise le pattern SQLite officiel : `PRAGMA foreign_keys=OFF`
+ CREATE NEW + INSERT SELECT + DROP OLD + RENAME (evite le bug du
`legacy_alter_table=ON` qui ne marche pas comme attendu via sqlx).

103 tests Rust (93 V2.6.1 + 10 nouveaux) + 224 tests frontend verts.

## Etat anterieur (V2.6.1 — 2026-05-15)

Refactor partiel : 4 slugs `body_composition` (`weight`, `fat_ratio`,
`muscle_mass`, `bmi`) delegues au nouveau helper
`metric_repository::read_metric_latest`. Comportement utilisateur
strictement identique. -25 LOC duplication chez `alerts.rs`. Performance
`get_metric_latest` Tauri command amelioree (1 row vs N rows).

Nouveau helper `read_metric_latest(pool, slug) -> Result<Option<MetricPoint>,
sqlx::Error>` : 2 queries `LIMIT 1 ORDER BY DESC` separees (metrics + legacy)
+ fusion memoire, priorite a `metrics` si meme date. Alignement complet sur
le pattern `read_metric_series`.

5 slugs `systolic`, `diastolic`, `heart_rate`, `sleep_score`, `steps`
etaient encore hardcodes dans `alerts.rs::get_latest_metric_value` :
integres en V2.7.0 via migration 009.

93 tests Rust + 224 tests frontend verts.

## Etat anterieur (V2.6.0 — 2026-05-07)

Auto-purge automatique du journal `audit_log` au demarrage de Synk. Tache
detachee `tauri::async_runtime::spawn` post-ready dans `boot.rs` (best-effort,
ne bloque jamais le boot). Conditions OR : `total_count > audit_max_entries`
OU `oldest_at < (now - audit_retention_days)`.

Migration 008 ajoute 3 colonnes a la table `settings` :
`audit_retention_days` (default 180), `audit_max_entries` (default 10000),
`audit_auto_purge_enabled` (default 1). `EXPECTED_LATEST_VERSION` 7 → 8 (rolling
backup pre-migration auto declenche pour les utilisateurs en V7).

2 nouvelles commands Tauri : `get_audit_purge_config`, `set_audit_purge_config`.
Fn pure `should_auto_purge(metrics, config, now_iso) -> Option<i64>` testable
trivialement (4 tests TDD couvrant les 4 branches).

UI : section "Auto-purge au demarrage" dans Settings > WithingsPanel >
DataMetricsCard (toggle + 2 inputs + bouton Enregistrer + toasts).

88 tests Rust + 224 tests frontend verts.

## Etat anterieur (V2.5.0 — 2026-05-07)

Splits Rust pur de 3 fichiers > 200 LOC en sous-modules par
responsabilite metier (pattern V1.2.0 `mod.rs` + `pub use sub::*`
glob preservant `tauri::generate_handler!` cache `__cmd__<name>`) :

- `commands/dashboard.rs` 212 → `dashboard/` (mod 5 + summary 156 +
  metrics 59)
- `commands/export/pdf.rs` 352 → `pdf/` (mod 37 + fetch 130 + render
  219, frontiere fetch sqlx async / render printpdf Rc<RefCell> non-Send)
- `withings/types.rs` 340 → `types/` (mod 19 + 9 sous-modules par
  domaine : deserializers 62, common 17, measures 28, activity 37,
  sleep 75, workouts 52, heart 35, devices 21, sync 25)

Refactor pur : aucun changement de comportement. 81 tests Rust + 222
tests frontend verts (inchanges).

Restent > 150 LOC (skip V2.5) : `client.rs` 353 (sensible
REFRESH_LOCK), `lib.rs` 206 (95% generate_handler! macro).

## Etat anterieur (V2.4.0 — 2026-05-07)

withings/* AppError finalise — dernier sub-projet du refactor error
handling Rust (V2.1 a V2.4). 3 nouveaux variants AppError :
`Http(reqwest::Error)` (auto via #[from]), `Unauthorized(String)`,
`RateLimit(String)`. 3 vagues : sync/* (8 fichiers, 18 sites sqlx),
client.rs (~24 sites, retry/refresh/Mutex statique strictement
preserves), auth.rs (3 sites sqlx + 3 unwrap Header::parse intacts).
Mapping Withings : HTTP 401 / code 401 → Unauthorized, HTTP 429 /
code 601 → RateLimit, autres → Internal.

Frontend `formatError` reconnait 3 nouveaux prefixes ("Erreur reseau",
"Acces expire", "Limite atteinte") → codes existants NETWORK / AUTH /
RATE_LIMIT.

81 tests Rust (79 + 2 error TDD) + 222 tests frontend (219 + 3 errors
withings) verts.

## Etat anterieur (V2.3.0 — 2026-05-07)

Audit log purge + observabilite simple. 3 nouvelles commands Tauri :
`purge_audit_log(older_than_days)`, `get_audit_metrics`,
`get_db_metrics`. Nouvelle section UI `DataMetricsCard` dans
Settings/WithingsPanel : taille DB, count audit_log avec
insert/delete + dates min/max, counts par table de mesure, form
purge (default 90 jours, `confirm()` avant execution).

79 tests Rust (76 + 3 audit_purge TDD) + 219 tests frontend (216 + 3
hooks audit/dashboard) verts.

## Etat anterieur (V2.2.0 — 2026-05-07)

Extension de V2.1.0 : `AppError` propage dans les 22 modules
`commands/*` restants (4 vagues, 79 sites migres). Wave 1 simples
(profile, withings_credentials, goals, sync_log, journal, heart,
devices, vitals, metrics). Wave 2 logique (labs, dashboard,
medications, alerts). Wave 3 sub-modules (insights/* + detectors,
manual_measurements/*). Wave 4 export (csv, pdf, json — sites sqlx
uniquement, IO/format laisses).

`withings/*` reste hors scope (refactor reporte V2.x).

76 tests Rust + 216 tests frontend verts (inchanges).

## Etat anterieur (V2.1.0 — 2026-05-07)

Error handling Rust type : enum `AppError` (`src/error.rs`) avec
6 variants (Db / Validation / NotFound / Json / Forbidden / Internal)
via `thiserror = "2"`. Les helpers `crud.rs`, `audit::record_audit`, et
les 5 modules CRUD utilisent `Result<T, AppError>` en interne, les
commands Tauri restent `Result<T, String>` (compat frontend) via
pattern inner/wrapper + `From<AppError> for String`.

Frontend `src/lib/errors.ts` etendu : `formatError` reconnait les 6
prefixes "Erreur base de donnees : ", "Erreur de validation : ", etc.
3 nouveaux codes `NOT_FOUND`, `FORBIDDEN`, `INTERNAL`.

Validation business ajoutee dans `add_manual_blood_pressure` (systolic
[0, 300], diastolic [0, 200]).

76 tests Rust (72 + 4 error TDD) + 216 tests frontend (211 + 5
errors.test) verts.

## Etat anterieur (V2.0.0 — 2026-05-06)

Refactor backend Rust : trait `CrudTable` (`commands/crud.rs`) + 3
helpers `select_range`/`select_latest`/`delete_with_audit` + helper
`record_audit_insert`. Les 5 modules CRUD (body_measures,
blood_pressure, sleep, activity, workouts) delegent les reads et le
delete aux helpers (-89 LOC sur les modules, +85 LOC crud.rs).
`add_manual_X` reste manuel mais appelle `record_audit_insert`
pour tracer les inserts dans `audit_log` (auparavant : seul DELETE
etait logge — bump major car comportement observable).

72 tests Rust (65 + 7 crud TDD) + 211 tests frontend verts.

## Etat anterieur (V1.6.1 — 2026-05-05)

Polish post-V1.6.0 : 2 warnings du code reviewer fixes.
- WithingsPanel : state `connected` consolide dans le parent (etait
  duplique dans ConnectionCard + parent). Plus de risque de
  desynchronisation entre badge et DevicesCard.
- helpers.ts : `nowDatetimeLocal`/`nowDate`/`nowTime` utilisent
  `date-fns format` au lieu de constructions manuelles (coherence
  avec `formatISO` deja importe).

211 tests frontend (inchanges) + 65 Rust verts.

## Etat anterieur (V1.6.0 — 2026-05-05)

Refactor structurel : 4 fichiers > 250 LOC splittes en sections selon
le pattern V1.2.0 (`pages/X.tsx` orchestrator + `pages/X/sections/`) :
- `WithingsPanel.tsx` 272 -> 20 LOC (5 sections)
- `createManualForm.tsx` 292 -> 132 LOC (helpers + FormFieldRenderer)
- `Alerts.tsx` 286 -> 106 LOC (2 sections + constants)
- `Medications.tsx` 299 -> 126 LOC (4 sections)

Plus middleware Zustand `devtools` sur uiStore + syncStore (Redux
DevTools en dev, no-op en prod). Plus integration Explorer aux a11y
tests (19 pages vs 18). 211 tests frontend (+1 vs V1.5.1).

## Etat anterieur (V1.5.1 — 2026-05-05)

Polish post-V1.5.0 : 11 `console.*` residuels migres vers le logger central (`hooks/appReady/*`, `App.tsx`, `BootErrorScreen`, `BootStuckScreen`). Tests `logger.test.ts` enrichis avec `vi.stubEnv` pour vraiment tester le comportement dev/prod (silenciation `logDebug`/`logInfo` en build prod). 210 tests frontend (+2 vs V1.5.0). Aucune feature touchee.

## Etat anterieur (V1.5.0 — 2026-05-04)

Couverture tests exhaustive (208 tests frontend, +66 vs V1.4.0, 100% des 24 features couvertes) + uniformisation Zustand (`uiStore` migre vers `safeStorage` pour coherence avec `syncStore`) + logger centralise env-aware (`src/lib/logger.ts`). Aucune feature touchee. Voir [CHANGELOG.md](../CHANGELOG.md) pour les notes detaillees.

Macro Rust `#[crud_command]` initialement prevue en V1.5.0 a ete differee en V2.0 (ROI faible : ~150 LOC sur 7000 LOC backend, risques Tauri). "Consolidation Zustand" recalibree en uniformisation (architecture deja propre, 2 stores bien separes).

## Etat anterieur (V1.1.0 — 2026-04-26)

Refactor complet du boot via subagent-driven-development (15 phases, ~25 dispatches). Performance percue + robustesse erreurs. 14 tests frontend + 12 tests Rust ajoutes. Voir [CHANGELOG.md](../CHANGELOG.md) pour les notes detaillees.

**Pieges nouveaux a retenir** :
- **Prefetch queryKey doit matcher hook EXACTEMENT** : si un hook utilise `["bodyMeasures", "latest"]` mais le prefetch `["body_measures_latest"]`, le cache est inutilise (refetch silencieux).
- **Single-instance avant les autres plugins** : si `tauri-plugin-single-instance` est apres `tauri_plugin_log` ou autre, la 2e instance va initialiser ces plugins avant d'exit -> race conditions sur les fichiers de log.
- **`run_versioned_migration` retourne `MigrationError`** : si tu ajoutes un nouveau site d'appel hors `bootstrap_schema`, utilise le type entierement (`MigrationError { version, source }`) au lieu de `.map_err(|e| e.source)`.

### Sync Withings
- Client avec retry exponentiel 3x, rate limiting (Semaphore 4), auto-refresh 401, timeout 30s
- Sync log automatique apres chaque sync (duree mesuree reellement, records, erreur)
- Auto-sync au demarrage + intervalle configurable (off/1h/6h/12h/24h)
- Insights auto-generes apres chaque sync
- Callback OAuth en boucle avec timeout 120s + emit evenements Tauri (`withings-auth-success`, `withings-auth-error`, `withings-auth-timeout` V1.0)

### Pages (19 routes — 18 pages + Explorer V1.0)
1. **Dashboard** — 6 KPI cards avec trends, 4 graphiques, comparaison de periodes
2. **Composition** — poids, IMC, masse grasse/musculaire, LineChart (hydration separee en StatCard)
3. **Tension** — systolique/diastolique, zones OMS, graphique
4. **Sommeil** — duree, score, phases, graphique
5. **Activite** — pas, calories, distance, graphique
6. **Entrainements** — categories, duree, FC, distance
7. **Coeur/ECG** — ECG status, FC repos
8. **Mensurations** — vue grille avec sparklines + barres progression objectif + vue detail + stats
9. **Alertes** — seuils configurables, 9 metriques, historique events
10. **Profil** — carte d'identite chiffree (identite, corps, mensurations cles, sante, tailles, records), export PDF
11. **Correlations** — correlation Pearson double-axe, 3 presets
12. **Medicaments** — suivi prises/sauts, observance
13. **Analyses labo** — vue par analyse expandable, sparklines, dots vert/rouge, trend arrows
14. **Insights** — tendances et anomalies auto-detectees, framer-motion
15. **Journal** — notes quotidiennes avec tags
16. **Timeline** — historique syncs, stats succes/erreur
17. **Export** — CSV, PDF, JSON export/import
18. **Settings** — Withings OAuth, profil, theming, sync interval, onglets masquables
19. **Explorer** (V1.0) — visualisation unifiee des 42 metriques V2, GenericMetricChart, MetricSelector, dedup metrics+legacy

### Pages supprimees (2026-04-11)
- **Humeur** — supprimee, deplacee dans `_poubelle/2026-04-10_pages-mood-nutrition/`
- **Nutrition** — supprimee, deplacee dans `_poubelle/2026-04-10_pages-mood-nutrition/`
- Tables `mood_logs` et `nutrition_logs` conservees dans la DB (pas de DROP)

### Fonctionnalites transversales
- 5 palettes couleur (indigo, emerald, rose, amber, slate)
- DataTable avancee (tri, recherche, pagination)
- Onboarding wizard 4 etapes (ecoute evenement withings-auth-success)
- ErrorBoundary global (plus d'ecran noir)
- Backup DB auto (max 5 fichiers)
- Audit logging (traces CRUD)
- Code-splitting React.lazy (48 chunks)
- Onglets masquables dans la sidebar
- Confirmation (confirm()) avant toute suppression

### Commandes Tauri (101)
- 28 modules Rust dans commands/ exposant des commandes (incluant `metrics`, `withings_credentials`, `clothing_sizes`, `error_log`, `body_profile`, `profile_card`) + `crud.rs`/`validation.rs` comme helpers partages sans commande propre
- 4 commandes Withings dans withings/ (auth x3, sync x1)
- Toutes les invoke() du frontend matchent lib.rs

### Stack Metriques V2 (V1.0 finalisee — voir docs/metrics-v2.md)
- Migration 005 : tables `metric_definitions` (36 built-in + 6 derived = 42 entries), `metrics`, `metric_targets`, `derived_metrics`
- Migration 006 : consolidation tables critiques (user_profile, medications, journal, etc.)
- Migration 007 : `withings_credentials` singleton (Client ID + Secret OAuth)
- Tables legacy : `body_measures_legacy`, `manual_measurements_legacy`, `vitals_legacy`, `measurement_types_legacy`
- Vues retrocompat : `body_measures`, `manual_measurements`, `vitals`, `measurement_types` pointent vers `*_legacy`
- Sync dual-ecriture : `sync_metrics()` + `upsert_metrics_group()` ecrivent dans `metrics` via `resolve_metric_slug`
- `metric_repository::read_metric_series` : lecture UNION ALL `metrics` + `*_legacy` avec dedup (measured_at, source)
- 3 commandes Tauri : `get_metric_definitions`, `get_metric_series`, `get_metric_latest` (V1.0)
- Hooks frontend : `useMetricDefinitions`, `useMetricSeries`, `useMetricLatest`
- Page Explorer (V1.0) : selection metrique + GenericMetricChart + stats + table 10 dernieres
- Tests d'integration : `src-tauri/tests/phase1_metrics.rs`

### Robustesse (V1.0)
- 4 `.unwrap()` fragiles refactores en pattern matching defensif (insights, sync/measures)
- `src/lib/errors.ts` : mapping uniforme erreurs Tauri/JS → messages utilisateur FR (NETWORK, AUTH, RATE_LIMIT, TIMEOUT, DATABASE, VALIDATION)
- `src/lib/storage.ts` : `safeGetItem`/`safeSetItem` avec validateurs (isIsoDate, isString, isBoolean) — clean auto des entrees corrompues
- OnboardingWizard : feedback OAuth (cancel pendant attente, retry apres timeout/error, listener `withings-auth-timeout`)
- Skeletons contextuels : `ChartSkeleton`, `StatCardGridSkeleton`, `TableSkeleton`
