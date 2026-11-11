# Campagnes de mutation

Campagne du 2026-10-22 (run R13), `cargo-mutants` 27.1.0, toolchain Rust 1.99.1. Un mutant est une altération du code (opérateur changé, fonction vidée) ; un mutant est tué si au moins un test échoue, il survit sinon. Un survivant est soit un mutant équivalent (le programme reste juste), soit un test qui manque.

## Commandes

Depuis `src-tauri`, une campagne à la fois, avec `--in-place` : le modèle de copie de `cargo-mutants` ne reprend que `src-tauri/` et le crate lit `../src/features/bodyProfile/catalog.json` et `../src/assets/body/*.png` (`include_bytes!`), la compilation de la copie échoue sur « chemin introuvable ». `--in-place` est incompatible avec `-j` (un seul mutant à la fois). Le dossier de sortie `src-tauri/mutants.out/` est ignoré par git ; copier `caught.txt`, `missed.txt`, `timeout.txt` et `unviable.txt` dans `logs/mutants/<fichier>/` après chaque campagne.

```powershell
$env:CARGO_INCREMENTAL = "0"
$env:CARGO_TARGET_DIR = "D:\Cache\cargo-target\synk-mutants"
cargo mutants --in-place --file src/crypto/cipher.rs -- --lib --test integration
cargo mutants --in-place --file src/services/backup.rs -- --lib --test integration
cargo mutants --in-place --file src/commands/export/restore.rs -- --test integration -- restore_backup::
cargo mutants --in-place --file src/commands/export/json/engine.rs -- --test integration -- export_exhaustive:: export_import_roundtrip:: import_json:: import_dedup:: import_json_validation::
```

Depuis R14, tous les tests d'integration tiennent dans un seul binaire (`--test integration`). Un filtre de modules se met apres un second `--` (`-- --test integration -- restore_backup::`) et ne convient qu'aux campagnes sans `--lib`, sinon il filtre aussi les tests unitaires ; pour `crypto/cipher.rs` et `services/backup.rs`, tout le binaire tourne (plus long par mutant que l ancienne liste de fichiers, non mesure).

Pendant une campagne, ne pas modifier `src-tauri/src` ni `src-tauri/tests` (le dépôt lui-même est muté). Après : `git status` doit ne montrer que les changements voulus. Un mutant peut laisser des fichiers dans le dossier courant (un fichier `xyzzy` de 8 Ko, base SQLite écrite par un mutant qui remplace un chemin, a été rangé le 2026-10-22).

Rappel de la nécessité de la liste de tests : l'ancienne fiche omettait `export_exhaustive` pour `engine.rs`. Sans lui, 13 mutants survivaient dont 9 étaient en réalité déjà tués ; la liste ci-dessus est la bonne. Une campagne sur un fichier dont les tests ont bougé commence par chercher qui appelle ses fonctions publiques (`Grep`).

## Résultats

| Fichier | Mutants | Tués | Non viables | Survivants avant | Survivants après |
|---|---|---|---|---|---|
| `crypto/cipher.rs` | 11 | 10 | 1 | 0 | 0 |
| `services/backup.rs` (alors `commands/export/backup.rs`, déplacé en R28) | 27 | 20, puis 26 | 1 | 6 | 0 |
| `commands/export/restore.rs` | 12 | 10 | 2 | 0 | 0 |
| `commands/export/json/engine.rs` | 61 | 56, puis 60 | 1 | 4 | 0 |
| Total | 111 | 96, puis 106 | 5 | 10 | 0 |

Aucun délai dépassé. Les mutants non viables ne compilent pas (par exemple `restore_latest_backup_inner` rendu `Ok(Default::default())`, `RestoreReport` n'implémentant pas `Default`) : ils ne comptent ni tués ni survivants.

## Survivants triés

Les dix étaient de vrais mutants (aucun équivalent), chacun tué par un test écrit dans ce run, puis rejoué par `cargo mutants -F` (backup : 11 mutants tués sur 11 ; engine : 24 sur 24).

| Mutant survivant | Ce qu'il changeait | Test qui le tue |
|---|---|---|
| `backup.rs` `<` en `<=`, `+` en `*`, `*=` en `+=` dans `retry_on_sharing_violation` | délais de reprise : dormir après la dernière tentative, ou suites 50, 100, 150 | `waits_fifty_then_doubles_and_never_sleeps_after_the_last_attempt` (horloge Tokio arrêtée, 750 ms exactement) |
| `backup.rs` `create_backup` vidé ou rendu `"xyzzy"`, `!` supprimé | la commande de sauvegarde manuelle n'était testée par rien | `a_manual_backup_without_a_database_is_refused`, `a_manual_backup_returns_the_path_of_a_verified_copy` (le corps est extrait en `create_backup_in`) |
| `engine.rs` `&&` en `\|\|` dans `data_columns` | l'export portait la colonne `id` : à l'import, un `id` déjà pris faisait ignorer la ligne sans erreur | `an_export_carries_no_row_id_so_an_import_keeps_the_rows_already_there` |
| `engine.rs` `+=` en `*=` dans `fill_profile` | une fiche de profil déjà complète n'était plus comptée en « ignorée » | `a_profile_with_nothing_to_complete_is_counted_as_ignored` |
| `engine.rs` `+=` en `-=` et `*=` dans `import_row` | les lignes orphelines n'étaient plus comptées dans le rapport | `a_row_whose_parent_is_unknown_is_counted_as_an_orphan` |

`tokio` reçoit la fonction `test-util` en dépendance de développement (horloge arrêtée) : pas de nouvelle crate, `Cargo.lock` inchangé.

## Procédure

- Une campagne est rare et ciblée : après un changement dans la sauvegarde, la restauration, le chiffrement ou l'import, ou avant une release qui les touche. Pas de script npm.
- Un survivant se traite en TDD : le test est écrit, passe sur le vrai code, et la campagne restreinte (`-F "in <fonction>"`) le voit tué.
- Les campagnes de ce run ont pris 6 min (chiffrement), 15 min (sauvegarde), 5 min (restauration) et 30 min (import) sur ce poste.
