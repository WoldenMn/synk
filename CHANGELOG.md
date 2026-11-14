# Changelog

Toutes les modifications notables de Synk sont documentées ici.
Format : [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), versioning [SemVer](https://semver.org/lang/fr/).

## [1.0.1] - 2026-11-14

### Corrigé

- Import FIT : un fichier dont un champ de développeur arrive avant sa description (montre ou appli Connect IQ) s'importe, au lieu d'être refusé en entier.
- Les tests ne laissent plus de bases de test dans le dossier temporaire de Windows, et le test d'une sauvegarde refusée ne tombe plus au hasard.

### Sécurité

- Dépendances Rust mises à jour, dont la correction de GHSA-jh2m-fhpj-3vr7.

## [1.0.0] - 2026-11-11

Première version publique. Synk rassemble vos données de santé sur votre machine, sans compte ni télémétrie.

### Ajouté

- Synchronisation de Withings, Oura, Polar, Whoop et Google Health par OAuth, chaque marque avec votre propre application.
- Import de fichiers exportés : Apple Santé, FIT, GPX, TCX et CSV.
- Composition corporelle, tension, sommeil, activité, entraînements, cœur et ECG, vitaux, mensurations, médicaments, analyses de laboratoire et journal.
- Constats automatiques tirés de vos mesures, carte d'identité santé et export PDF.
- Base SQLite locale, jetons OAuth chiffrés dans le Gestionnaire d'identification de Windows, sauvegardes vérifiées.
