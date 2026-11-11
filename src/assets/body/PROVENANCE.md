# Provenance des figures de mensurations

`front.png` (face) et `back.png` (dos) : un mannequin gris d'homme sur fond transparent, 512 x 1400 pixels, soit l'échelle 2 du repère 256 x 700 de `src/features/bodyProfile/plate-landmarks.json`. La fiche à l'écran (`BodyFigure`) et le PDF exporté (`commands/export/pdf/figures.rs`) lisent ces deux mêmes fichiers. Le choix est décrit dans la décision D83 de `docs/decisions.md`.

## Fichiers livrés

| Fichier | Taille | SHA-256 |
|---|---|---|
| `front.png` | 107 843 octets | `31a3f6d9fe4df0bbbd973f59629464c66f3e07366d0eb061218765c954fa00c8` |
| `back.png` | 100 879 octets | `8e6ec037d192d045a375e1d82461f2137ff8f422645f8596e7e3eab7d78b75b0` |

`scripts/plate-lib.test.mjs` relit ces empreintes : remplacer une image sans mettre cette table à jour rend `npm run verify` rouge. Changer une image oblige aussi à remesurer les repères et à recaler les ancres (`npm run plate:check`, D83).

## Photo d'origine

- Photo Pexels n° 5301528 : https://www.pexels.com/photo/5301528/
- Photographe : Armin Rimoldi.
- Licence Pexels (https://www.pexels.com/license/) : usage et modification permis, attribution non requise ; ne jamais montrer une personne identifiable de façon négative.
- Fichier téléchargé : `f4-face-homme-pexels-5301528.jpg`, 3733 x 5599 pixels, SHA-256 `9a31b555710795ff563aaea968ebc00528fef7ed48070a8200415889a91f649f`.
- Ni le visage ni l'identité de la personne photographiée ne subsistent dans le résultat : le mannequin n'a aucun trait.

## Fabrication (1er novembre 2026)

1. **Retouche en mannequin gris.** FLUX.2 klein 4B (distillé, 4 pas ; licence Apache-2.0), ComfyUI 0.37.4 en local, graine 7, 2 mégapixels (1184 x 1776), par `~/.claude/scripts/editer_image.py`. Consigne de la face : « Turn this person into a smooth matte light grey store mannequin made of one single material: bald smooth rounded head without any hair, no facial features, no skin texture, no body hair, no skin spots, plain light grey boxer shorts. Keep exactly the same pose, the same body proportions, the same silhouette and the same camera framing. Plain white background. »
2. **Vue de dos.** La photo d'origine est retournée en miroir puis retouchée avec la consigne : « Show this same smooth matte light grey store mannequin seen from behind: the back of a bald smooth rounded head without hair, shoulder blades, spine and the back of the legs, plain light grey boxer shorts, no face, no skin texture. Keep exactly the same pose, the same body proportions, the same outline and the same camera framing. Plain white background. » Le dos garde donc le contour de la face, retourné.
3. **Détourage.** BiRefNet (licence MIT), par `~/.claude/scripts/finir_image.py --detourer`, en PNG RGBA 1184 x 1776.
4. **Normalisation et gris.** Recadrage sur la figure (alpha supérieur à 0,5), mise à l'échelle pour que le sommet du crâne tombe à y = 21 et la plante des pieds à y = 690 d'un repère de 700 unités, marge de 12 unités de chaque côté, figure centrée, toile de 512 x 1400 pixels (script local `variantes.py`, vue « photo », sans flou de tête). Luminance étirée entre les percentiles 2 et 98, filtre médian de 5 puis flou gaussien de 1,2, teinte 0,58 + 0,34 x luminance avec une légère dominante froide.
5. **Conversion pour le dépôt.** Luminance = moyenne de R, V et B, alpha supérieur ou égal à 250 forcé à 255 et alpha inférieur ou égal à 3 forcé à 0, PNG niveaux de gris avec alpha sur 8 bits, optimisé. Les fichiers passent de 259 et 245 Ko (RGBA) à 105 et 99 Ko, la dominante froide disparaît (écart de 7 niveaux au plus sur 255).

La conversion de l'étape 5 se rejoue avec Pillow : `Image.fromarray(np.dstack([luminance, alpha]), "LA").save(chemin, "PNG", optimize=True)`.
