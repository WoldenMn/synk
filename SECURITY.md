# Sécurité

Synk garde des données de santé. Tout reste sur la machine : base SQLite
locale, jetons OAuth chiffrés, aucune télémétrie. Les seules sorties réseau
sont les appels aux services que l'utilisateur connecte lui-même : Withings,
Oura, Polar, Whoop, Google Health.

## Signaler une faille

Ne pas ouvrir de ticket public. Passer par le signalement privé de GitHub :
onglet **Security** du dépôt, puis **Report a vulnerability**
(<https://github.com/WoldenMn/synk/security/advisories/new>).

Décrire la version de Synk, la version de Windows, les étapes pour
reproduire et l'effet obtenu. Ne jamais joindre de vraies données de santé
ni de jeton : un jeu de données fictif suffit.

## Versions prises en charge

Seule la dernière version publiée reçoit des correctifs de sécurité.

## Ce que le modèle de menace couvre

- Un autre processus de la session qui lit les fichiers de Synk : les jetons
  et identifiants des sources sont chiffrés au repos.
- Un fichier importé piégé (export Apple Santé, FIT, GPX, TCX, CSV) : lecture
  en flux, tailles bornées, erreurs claires plutôt qu'un plantage.
- Un service distant qui répond mal : https seulement, aucune redirection
  suivie, jetons effacés de la mémoire après usage.

Hors du modèle : une machine déjà compromise, un attaquant avec un accès
physique, et la sécurité des comptes ouverts chez les marques connectées.
