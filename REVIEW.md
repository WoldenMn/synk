# Review instructions — Synk

Application Tauri : frontend TypeScript/React, backend Rust dans `src-tauri/`. **Aucune CI ne tourne sur ce dépôt** alors que les scripts existent (`lint`, `type-check`, `test`, `check`). Le hook de commit lance `npm run check` et le hook de push exige un `npm run verify` complet sur l'arbre poussé, mais rien ne tourne côté forge : la revue est donc le seul filet automatique, et elle doit couvrir ce qu'un pipeline couvrirait ailleurs — sans pour autant se transformer en linter manuel.

## Ce que « Important » veut dire ici

### La frontière Tauri

C'est la surface qui mérite le plus d'attention sur ce projet, parce qu'elle est la seule où le frontend obtient des droits qu'un navigateur n'a pas.

- Une commande `#[tauri::command]` ajoutée qui prend un **chemin de fichier venant du frontend** et l'utilise sans le contraindre à un dossier attendu. Un `../` dans une chaîne venue de l'UI est une traversée de répertoire.
- Une commande qui exécute un binaire, ouvre une URL ou écrit hors du dossier de données de l'app.
- Une permission élargie dans les capabilities Tauri : la justifier dans la PR ou c'est Important.
- Un secret rendu au frontend par une commande, alors qu'il pouvait rester côté Rust.

### Erreurs avalées

- Rust : `let _ = …` sur un `Result` porteur, `unwrap()` / `expect()` atteignable en production, `if let Err(_)` muet.
- TypeScript : `catch {}` vide, `.catch(() => {})`, `try` qui rend une valeur par défaut sans trace.

Un faux vert coûte plus cher qu'un crash — c'est vrai partout, mais surtout ici où rien ne tourne en CI pour rattraper.

### Ce que la CI absente ne rattrape pas

- Une erreur de type qui casserait `npm run type-check`.
- Un test cassé ou supprimé sans motif dit dans la PR.
- Un `console.log` ou un `dbg!` laissé dans un chemin de production.

Ces trois-là sont Important **sur ce dépôt précisément**, parce qu'aucun pipeline ne les arrête. Sur un dépôt à CI, ce ne seraient que des nits.

### Le reste

- Fichier au-delà de 300 lignes sans motif dit dans la PR.
- Chemin en dur avec séparateur POSIX : ce projet cible Windows desktop.

## Plafond de nits

Cinq. Au-delà, une ligne de décompte dans le résumé.

## Ne pas signaler

| Chemin | Motif |
|---|---|
| `src-tauri/target/**`, `node_modules/**`, `dist/**` | Générés |
| `audits/**` | Rapports datés, pas du code |
| `docs/specs/**`, `docs/plans/**`, `_archive/**` | Specs, plans et archives : de la décision datée, pas du code ni de la documentation de référence |
| `src-tauri/gen/**` | Généré par Tauri |
| `package-lock.json`, `Cargo.lock` | Lockfiles — juger la dépendance dans le manifeste, pas la résolution |

## Toujours vérifier

- **Chaque `#[tauri::command]` ajoutée ou modifiée** : ce qu'elle accepte du frontend, et ce qu'elle en fait. Nommer l'entrée non contrainte s'il y en a une.
- **Chaque dépendance ajoutée**, côté `package.json` comme `Cargo.toml` : ce qu'elle apporte que le code existant ne faisait pas. YAGNI est la règle du poste — trois lignes similaires valent mieux qu'une abstraction, et a fortiori mieux qu'une dépendance.
- **Le passage de données Rust vers TypeScript** : une structure sérialisée entière là où deux champs suffisent.
- **Plateforme** : Windows desktop uniquement. Pas de responsive, pas de media queries sous `lg`, Segoe UI, raccourcis `Ctrl`.

## Barre de preuve

Toute affirmation de comportement se cite en `fichier:ligne`. Ne jamais déduire d'un nom de commande, de hook ou de module ce qu'il fait.

Pour une affirmation de sécurité sur la frontière Tauri, citer la ligne où l'entrée arrive du frontend **et** la ligne où elle est utilisée. Sans les deux, dégrader en avertissement.

## Re-review

Deuxième passe et suivantes : Important uniquement. Les nits de la première passe ne sont pas re-postés, même non traités.

## Forme du résumé

Ouvrir par un décompte d'une ligne, du type `1 frontière Tauri, 2 erreurs avalées, 3 nits`. Quand rien de sérieux ne sort, le dire d'abord. Mentionner en une ligne si la PR aurait cassé `npm run check` — c'est l'information que l'absence de CI rend précieuse.
