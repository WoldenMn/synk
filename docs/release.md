# Build et release — Synk

Une release est une commande : `npm run release -- X.Y.Z`. Elle verifie, monte la version, construit l'installeur NSIS, calcule sa somme SHA-256, l'archive hors de `target/` et commite. Elle ne pose aucune etiquette git et ne pousse rien : les etiquettes sont un geste du user. L'installation de Synk reste au user.

## Pre-requis machine de build

| Outil | Version | Verification |
|---|---|---|
| Node.js et npm | `engines` de `package.json` | `node --version` |
| Rust | chaine epinglee par `rust-toolchain.toml` (rustup l'installe seul) | `rustc --version` |
| Visual Studio Build Tools 2022 | >= 17 | requis pour la cible MSVC Tauri |
| NSIS | 3.x | telecharge par Tauri au premier build dans `%LOCALAPPDATA%\tauri\NSIS` |
| Disque `D:` monte | dossier `D:\Sauvegardes\Synk\installeurs\` | `Test-Path` |

### Outils cargo

Le build n'en a pas besoin ; la veille, la couverture Rust et la mutation, si. Installation : `cargo install --locked <crate>` (sauf mention).

| Crate | Version vue le 2026-10-26 | Sert à |
|---|---|---|
| `cargo-audit` | 0.22 | avis RustSec, dans `npm run veille` |
| `cargo-deny` | 0.20 | licences et interdits (`src-tauri/deny.toml`), dans `npm run veille` |
| `cargo-machete` | 0.9 | dépendances inutilisées, dans `npm run veille` |
| `cargo-llvm-cov` | 0.9 | `npm run couverture:rust` ; le composant `llvm-tools-preview` vient de `rust-toolchain.toml` |
| `cargo-mutants` | 27 | campagnes de mutation à la main, voir `mutation.md` |

### Cache NSIS, copie hors ligne

Le premier build sur un cache vide telecharge NSIS. Une copie du cache du 2026-10-26 est rangee dans `D:\Sauvegardes\Synk\outils\NSIS-2026-10-26\` : pour construire hors ligne, la recopier vers `%LOCALAPPDATA%\tauri\NSIS`. Une session lancee depuis l'app Claude voit un `%LOCALAPPDATA%` virtualise (MSIX) : le cache lu et utilise est celui du processus qui construit.

## Avant de lancer

1. Se placer sur une branche de release (`release/X.Y.Z`), jamais sur `main` ; arbre de travail propre.
2. Dater le CHANGELOG et le commiter a part : la section « Non publie » devient `## [X.Y.Z] - AAAA-MM-JJ` et « Non publie » reste, vide, au-dessus. Le script refuse un CHANGELOG sans section datee ou dont « Non publie » porte encore des notes.
3. Le script exige une version strictement superieure a celle de `package.json`.

## `npm run release -- X.Y.Z`

| Etape | Ce qu'elle fait | Arret si |
|---|---|---|
| controles | branche, arbre propre, version, CHANGELOG, disque `D:`, installeur deja present | un controle echoue |
| veille | `npm run veille` | code different de 0 (rouge = pas de release) |
| version | monte la version dans les cinq fichiers et le README, relit les sept emplacements, commite `chore(release): X.Y.Z` | cinq fichiers en desaccord au depart, emplacement non monte |
| verify | `npm run verify` complet sur l'arbre commite ; lit la derniere ligne de `logs/verify.log` et exige `arbre=<arbre du commit>` | verify rouge, ligne sans le bon arbre |
| couverture | `npm run couverture:rust` | plancher non atteint |
| parcours | `npm run parcours` ; un seul nouvel essai si la sortie dit « aucune reponse CDP », ou si le seul probleme est le repli de l'IPC sur postMessage au demarrage (poste charge, constate le 2026-10-26 : 19 pages ok, 8 avertissements, puis 0 au second passage) | tout autre probleme, ou second echec |
| budget | `npm run budget` | plafond depasse |
| build | `CARGO_BUILD_JOBS=1 npm run tauri -- build` | build en echec |
| installeur | taille (au moins 1 Mo), date posterieure au debut du build, version de fichier egale a X.Y.Z, poids au plus 10 % au-dessus de l'installeur precedent | un controle echoue |
| copie | copie dans `D:\Sauvegardes\Synk\installeurs\`, ecrit `Synk_X.Y.Z_x64-setup.exe.sha256` (`SOMME  nom`, majuscules, deux espaces), recalcule l'empreinte des deux cotes | empreintes differentes, fichier deja present (jamais ecrase) |

Chaque etape ecrit sa sortie dans `logs/release-X.Y.Z/<etape>.out`. Une ligne par release (date, version, duree de chaque etape en secondes, memoire libre minimale, arbre, octets, SHA-256) s'ajoute a `D:\Sauvegardes\Synk\installeurs\journal-des-releases.txt` : c'est la mesure de la duree du build et de la memoire (DX-009). Le build reste en `lto = true` avec un seul job : le poste manque de memoire pour l'edition de liens, la comparaison avec `lto = thin` n'a pas ete mesuree.

A la fin, le script affiche la commande `git tag vX.Y.Z` a lancer par toi, apres la fusion. Il ne pousse rien.

### Simulation

`npm run release -- X.Y.Z --simulation` fait tout sauf le build, l'installeur, la copie et les commits : controles, veille, montee de version verifiee en memoire (rien d'ecrit), verify, couverture, parcours, budget. Sa ligne de journal va dans `logs/journal-des-releases.txt`. Les fonctions pures du script (arguments, CHANGELOG, montee, somme, controle de l'installeur, journal) sont testees par `scripts/release-lib.test.mjs`.

### Verifier la somme avant d'installer

```powershell
certutil -hashfile D:\Sauvegardes\Synk\installeurs\Synk_X.Y.Z_x64-setup.exe SHA256
Get-Content D:\Sauvegardes\Synk\installeurs\Synk_X.Y.Z_x64-setup.exe.sha256
```

Les deux sommes doivent etre identiques. Puis lancer l'installeur.

## Version unique

La version vit dans `package.json`, `package-lock.json` (deux emplacements), `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json` et la ligne « Version courante » du `README.md`. `src/__tests__/versions.test.ts` echoue si un des sept emplacements differe ; `npm run bump -- X.Y.Z` les monte tous (`scripts/version-lib.mjs`). Convention SemVer : MAJOR pour un changement de schema ou de commande publique qui casse, MINOR pour une nouvelle fonction, PATCH pour une correction.

## Icones

Les icones ne s'editent pas a la main : modifier les SVG de `src-tauri/icons/source/` puis lancer `npm run icons:gen`. Le script (`scripts/build-icons.mjs`) regenere tout le jeu PNG depuis `synk-icon.svg` et assemble `icon.ico` taille par taille : 16 a 24 px depuis `synk-icon-16.svg`, 32 et 40 px depuis `synk-icon-32.svg`, 48 px et plus depuis `synk-icon.svg`. Il copie aussi le favicon dans `public/synk.svg` et compose les deux bitmaps de l'installeur (`scripts/nsis-images.mjs`) : `nsis-sidebar.bmp` (164 x 314) et `nsis-header.bmp` (150 x 57), declares sous `bundle.windows.nsis` dans `tauri.conf.json`. Une regeneration rend les PNG, `icon.ico` et les BMP a l'identique ; seul `icon.icns` (macOS, inutilise) change a chaque passage, sa sortie par `tauri icon` n'etant pas deterministe.

## Tests pre-release

`npm run verify` doit finir sur `[verify] OK (niveau complet)`. Il enchaine prettier, tsc, eslint (`src` et `scripts`) sans avertissement, la garde de taille, cargo fmt, vitest avec couverture, build, budget, clippy `-D warnings` et cargo test, et s'arrete au premier echec. Les reglages cargo `-j 2` et `incremental = false`, portes par `src-tauri/.cargo/config.toml`, evitent l'epuisement memoire (os error 1455) sur ce poste. Il echoue aussi quand le nombre de tests passe sous les planchers de `scripts/verify-floors.json`, seule source de ces nombres.

## Veille des dependances avant release

```bash
npm run veille    # doit finir en code 0 ; rapport dans logs/veille-AAAA-MM-JJ.md
```

Rouge = pas de release. La veille ne compile rien et dure une vingtaine de secondes. Elle lit `npm audit` (arbre entier et livrable), `npm outdated`, `cargo audit`, l'API de crates.io pour chaque crate directe, `cargo deny check licenses bans` (`src-tauri/deny.toml`), `cargo machete`, le `rust-version` du `Cargo.toml` contre l'arbre, et le dernier rapport d'Intendance quand il date d'apres les verrous. Elle echoue sur tout avis (haut ou critique cote npm, tout avis ou avertissement cote cargo) et toute majeure en retard (en 0.x, la mineure compte) qui n'a pas d'exception.

Une exception est un objet de `scripts/veille-exceptions.json` : `id` (identifiant de l'avis, ou `majeure:<paquet>`), `motif` (le blocage ou la preuve d'inatteignabilite, jamais vide), `date` et `echeance` (`AAAA-MM-JJ`, trois mois au plus). Une exception echue refait echouer la veille : elle se revoit, elle ne s'oublie pas. Un avis haut atteignable ou une licence a reciprocite forte ne s'excuse pas : on corrige ou on s'arrete.

## Essai manuel avant l'installation

Voir aussi `docs/user-guide.md`.

1. Installer `Synk_X.Y.Z_x64-setup.exe` (apres verification de la somme).
2. Premier lancement : assistant de premier lancement.
3. Renseigner les identifiants d'une source, la connecter, synchroniser.
4. Parcourir les 19 pages.
5. Page **Explorer** : selecteur de metrique, graphique, statistiques.
6. Export CSV, PDF et JSON, puis import JSON.
7. Theme sombre et clair, graphiques lisibles.
8. Relancer : tableau de bord direct.

## Signature : non signe (decision D9, 2026-10-26)

Le binaire et l'installeur ne sont pas signes. Raisons : un certificat de signature de code EV coute de l'ordre de 250 dollars par an (estimation de l'ancienne note de ce fichier, non reverifiee), Synk est une installation a usage personnel sur le poste de son auteur, et l'integrite entre le build et l'installation est tenue par la somme SHA-256 verifiee ci-dessus. Windows SmartScreen affichera donc un avertissement au premier lancement de l'installeur : cliquer « Plus d'infos » puis « Executer quand meme ». Si Synk sort un jour du poste de son auteur, la signature se rouvre (`bundle.windows.signCommand` et `certificateThumbprint` de `tauri.conf.json`).

## Mise a jour automatique : close (decision D9, 2026-10-26)

Aucun plugin de mise a jour (`tauri-plugin-updater`) n'est installe et aucun ne le sera tant que la distribution reste personnelle : la conception de 2026-05-07 (`docs/specs/2026-05-07-auto-update-tauri-design.md`) reposait sur des releases GitHub et sur un jeton embarque dans le binaire, que la politique du depot interdit. Mettre a jour Synk, c'est lancer l'installeur de la version suivante par-dessus : le dossier d'installation est retrouve par l'identifiant et la base n'est pas touchee (ne pas renseigner `bundle.publisher`, voir `CLAUDE.md`).

## Distribution

Le seul canal est le disque `D:` (`D:\Sauvegardes\Synk\installeurs\`, version, somme, journal) et, pour un autre poste, une copie de l'installeur et de son `.sha256` faite a la main. Ni release GitHub, ni `.msi` : seul NSIS est produit depuis la 2.14.0.

## Reproductibilite de l'executable

Mesure du 2026-10-26 (R29) : deux builds `tauri build --no-bundle` du meme arbre, l'un apres l'autre, en forcant la recompilation du crate `synk` (`touch` de `lib.rs` et `main.rs`), avec `CARGO_BUILD_JOBS=1`. Les dependances compilees sont reprises du cache ; une comparaison depuis un dossier de build vide n'a pas ete faite.

| Build | SHA-256 de `synk.exe` | Octets | Duree |
|---|---|---|---|
| 1, sans `/Brepro` | `0B5285AA...EEA769` | 26 303 488 | 234 s |
| 2, sans `/Brepro` | `80D1C88C...082032` | 26 303 488 | 234 s |
| 1, avec `/Brepro` | `1054C86A...11C745` | 26 304 000 | 220 s |
| 2, avec `/Brepro` | `1054C86A...11C745` | 26 304 000 | 215 s |

Sans l'option, les deux fichiers de meme taille different de 23 octets : l'horodatage de l'en-tete PE, des horodatages du repertoire de debogage et le GUID du chemin PDB, tous dans l'en-tete et dans `.rdata`, aucun dans le code (`.text`), les donnees ni les ressources. `src-tauri/build.rs` emet donc `cargo:rustc-link-arg-bins=/Brepro` pour la cible MSVC : l'editeur de liens remplace ces champs par une empreinte du contenu, et les deux builds donnent le meme executable octet pour octet. L'option est portee par `build.rs` et non par `RUSTFLAGS` de `src-tauri/.cargo/config.toml` : elle ne vise que l'edition de liens des binaires et ne force aucune recompilation des dependances (ni des tests, ni de clippy). Gardien : `tests/integration/reproducible_link.rs` (la ligne ne disparait pas sans qu'un test le dise).

L'installeur NSIS, mesure le 2026-10-26 (release 3.0.0, deux builds du meme arbre) : deux installeurs differents, 7 007 333 et 7 006 221 octets, differents des l'offset 55 320 (donnees compressees). Il n'est pas reproductible, cause non isolee. La somme archivee par `npm run release` est celle de l'installeur livre, pas un identifiant du code source ; `synk.exe` (dans `target/release/`) est la reference de reproductibilite.

## Nettoyage

Ne jamais supprimer `src-tauri/target` : `target/release/bundle` y garde l'installeur construit en dernier, et le dossier est une jonction vers `D:\Cache\cargo-target\synk`. Pour liberer de la place, `cargo clean --profile dev` (dans `src-tauri`) ne touche que les artefacts de developpement. `node_modules` et `dist` se regenerent par `npm ci` et `npm run build`.

## Diagnostic d'un demarrage rate

Le niveau du journal se regle par la variable d'environnement `SYNK_LOG` au lancement (`error`, `warn`, `info`, `debug`, `trace`) ; sans elle, `info`. Une valeur illisible garde `info` et ecrit un avertissement au demarrage. Le journal est dans `%LOCALAPPDATA%\com.synk.health\logs\` (`com.synk.health.dev` pour le developpement) : cinq fichiers d'un mega-octet au plus sont conserves, de sorte que le journal d'un demarrage rate survit au lancement suivant.

```powershell
$env:SYNK_LOG = "debug"; & "<dossier d'installation>\Synk.exe"
```
