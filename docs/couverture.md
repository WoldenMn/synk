# Couverture de tests

Mesure du 2026-10-26, release 3.0.0 (V7, `npm run release`), relevée par `npm run verify` et `npm run couverture:rust`.

| Côté | Total (lignes) | Plancher | Détail |
|---|---|---|---|
| TypeScript | 77 % | `couverture_ts` | `npm run verify`, fournisseur v8 |
| Rust | 86,38 % | `couverture_rust` | `npm run couverture:rust`, `cargo llvm-cov` 0.9.1 |
| Rust, `withings/client` | 92,2 % | `couverture_withings_client` | objectif de vague 3 : 70 % ou plus, tenu |

Les planchers vivent dans `scripts/verify-floors.json`, seule source de ces nombres. Le gardien `src/__tests__/doc-drift.test.ts` refuse ce tableau si une mesure y passe sous son plancher, ou si une ligne cite un fichier Rust qui n'existe plus.

`withings/client` par fichier : `credentials.rs` 100 % (14 / 14), `oauth.rs` 100 % (11 / 11), `tokens.rs` 100 % (43 / 43), `session.rs` 89,0 % (146 / 164). Le serveur de rappel OAuth vit désormais dans `sources/oauth/server.rs`, sous test d'intégration (R21, R43).

## Méthode

- TypeScript : `vitest run --coverage` (`@vitest/coverage-v8` 5.0.5, même version que vitest), lecture de `logs/coverage/coverage-summary.json`. Inclus : `src/**/*.{ts,tsx}` hors tests, `test-setup.ts` et `.d.ts`.
- Rust : `cargo llvm-cov -j 2 --json --summary-only --output-path ../logs/llvm-cov.json` depuis `src-tauri`, `CARGO_INCREMENTAL=0`. La mesure de tout le crate en une passe a tenu en mémoire sur ce poste le 2026-10-22 (compilation instrumentée de quelques minutes), sans groupes de tests. Si `os error 1455` revient : `cargo llvm-cov --no-report -j 2 --lib`, puis `--test <nom>` par groupes de cinq fichiers, enfin `cargo llvm-cov report --json --summary-only --output-path ../logs/llvm-cov.json`.
- Le dossier mesuré se choisit par préfixe de chemin (`src/sources/withings/client/`), pondéré par les lignes de chaque fichier.
- `target/llvm-cov-target` est régénérable : `cargo llvm-cov clean --workspace` après chaque mesure est permis.

## Instrument contrôlé

- Cas connu mauvais : planchers posés à 100, les deux outils échouent avec les vraies valeurs (TypeScript 66,82 % dans `npm run verify`, Rust 69,9 % et 74,4 % dans `npm run couverture:rust`, code 1).
- Cas connu bon : planchers à l'entier inférieur de la mesure, code 0, messages « plancher atteint ».
- Les trois fichiers de `withings/client` donnent trois valeurs différentes (43,1, 83,2, 83,7) : le préfixe ne confond pas les fichiers.

## Fichiers les moins couverts

Rust (au moins 30 lignes, 2026-10-26) : `boot/post_ready.rs` 0 / 58, `commands/metrics.rs` 0 / 57, `commands/sources.rs` 0 / 100, `lib.rs` 0 / 48, `commands/import.rs` 4 / 80, `withings/mapping/workouts.rs` 18 / 70, `export/pdf/report_lines.rs` 37 / 132, `boot.rs` 48 / 147.

TypeScript : la liste des fichiers les moins couverts se relit dans `logs/coverage/coverage-summary.json` après `npm run verify` ; elle n'est pas recopiée ici (elle périmait à chaque run).

## Procédure

- `npm run verify` tient la couverture TypeScript à chaque run.
- `npm run couverture:rust` se lance en fin de vague (et avant d'ouvrir une vague de tests), pas dans `verify` : c'est une seconde compilation complète, instrumentée.
- Un plancher se relève quand la mesure le dépasse d'un point (le message dit « à relever »). Il ne baisse jamais sans une ligne datée dans ce fichier qui dit pourquoi.
- Les planchers vivent dans `scripts/verify-floors.json` : `couverture_ts`, `couverture_rust`, `couverture_withings_client`.

## Journal des planchers

- 2026-10-22 : pose initiale, 66 (TypeScript), 69 (Rust), 74 (`withings/client`).
- 2026-10-23 : relevés à la mesure du jour (re-audit run_007, N-22), 68 (TypeScript, inchangé), 74 (Rust, avant 72), 79 (`withings/client`, avant 74). `npm run couverture:rust` rend « plancher atteint » aux deux.
- 2026-10-26 : couverture.md remis d'après la mesure de la 3.0.0 (R44, N-22, N-48) et gardé par `doc-drift.test.ts`.
- 2026-10-26 : le journal ne recopie plus les planchers (R30, N-48) ; `scripts/verify-floors.json` est la source, et chaque relèvement se date dans le journal du plan.

## Reproductibilité de l'installeur (N-65, partiel, 2026-10-26)

Deux builds NSIS du même arbre (release 3.0.0, `synk.exe` reproductible par `/Brepro`) donnent deux installeurs différents : 7 007 333 octets (SHA-256 6F9457FC...) et 7 006 221 octets (594153CE...), différents dès l'offset 55 320 dans la partie compressée. L'installeur n'est donc pas reproductible ; la cause n'est pas isolée, et la comparaison depuis un dossier de build vide n'a pas été faite.
