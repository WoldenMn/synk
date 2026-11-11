# FAQ — Synk

Resolution des problemes courants.

## Connexion Withings

### "Erreur lors de l'autorisation Withings" / Le navigateur s'ouvre mais rien ne se passe

- Verifier que **Callback URI** dans le dashboard developer Withings est exactement
  `http://localhost:17483/callback` (sans `https`, sans slash final).
- Verifier qu'aucune autre application n'utilise le port 17483 sur localhost.
  Sur Windows : `Get-NetTCPConnection -LocalPort 17483`.
- Si timeout (120s ecoulees), l'OnboardingWizard et la page Parametres affichent
  un bouton **Reessayer**. Cliquer dessus.

### "Session Withings expiree"

Le token Withings est auto-rafraichi sur 401 par le client Rust (cf.
`src-tauri/src/sources/withings/client/`). Si l'erreur persiste :

1. Parametres -> Sources -> carte Withings -> **Deconnecter**.
2. **Reconnecter** -> nouvelle OAuth.

### Sync echoue avec "rate limit" (429)

Withings limite a ~120 requetes / minute par compte. Synk implemente un Semaphore
de 4 connexions concurrentes + retry exponentiel (1s, 2s, 4s).

Si l'erreur persiste, attendre 5-10 minutes, puis relancer la sync depuis le bouton
Header.

## Donnees et synchronisation

### Pourquoi mes mesures n'apparaissent-elles pas ?

1. Verifier dans **Parametres -> Sources** que la carte de la source est "Connectee".
2. Cliquer **Synchroniser maintenant** (bouton dans le Header).
3. Aller dans **Synchronisations** : verifier que la derniere sync est bien recente et
   sans erreur.
4. Verifier que les mesures sont bien visibles dans l'app **Withings** officielle.
   Synk ne peut afficher que ce que l'API Withings retourne.

### Quels appareils Withings sont supportes ?

Tout appareil dont les donnees sont remontees a `wbsapi.withings.net` :

- Balances Body+, Body Cardio, Body Comp, Body Scan
- Tensiometres BPM Connect, BPM Core
- Sleep Analyzer, ScanWatch, Steel HR, Move
- Pulse, Thermometre Thermo

Les types de mesures (50+) sont mappes dans `src-tauri/src/sources/withings/mapping/`.

### Mes mensurations personnalisees ne sont pas synchronisees

C'est normal : Withings ne stocke pas les mensurations (tour de taille, biceps...).
Ces mesures sont saisies manuellement depuis la page **Mensurations** et restent
dans la base locale.

## Brancher une autre source

Pas à pas par marque : [guide utilisateur, section 2](./user-guide.md).

### La marque refuse la connexion (« redirect_uri », « URL de rappel invalide »)

L'URL de rappel déclarée chez la marque doit être celle que la carte affiche dans **Paramètres > Sources**, caractère pour caractère (sans slash final, en `http`) :

- Withings : `http://localhost:17483/callback`
- Oura : `http://localhost:17483/callback/oura`
- Polar : `http://localhost:17483/callback/polar`
- Whoop : `http://localhost:17483/callback/whoop`
- Google Health : `http://localhost:17483/callback/google_health`

Vérifiez aussi qu'aucun autre programme n'occupe le port 17483 (`Get-NetTCPConnection -LocalPort 17483`).

### Polar : « consentement refusé » ou des jours manquent

- Refus à l'enregistrement : acceptez tous les consentements de données sur account.polar.com, puis relancez la synchro.
- Jours manquants : Polar ne garde que 28 jours de sommeil et de Nightly Recharge et 30 jours d'entraînements. Une synchro plus espacée que cela perd les jours sortis de la fenêtre. Synchronisez au moins une fois par mois.
- « Reconnectez Polar » : le jeton de Polar ne se rafraîchit pas, il faut refaire **Connecter**.

### Whoop : « quota du jour atteint, reprise vers HH:MM »

Whoop accepte 10 000 requêtes par jour et 100 par minute. Quand le quota du jour est atteint, Synk s'arrête, garde ce qu'il a écrit et ne rappelle pas Whoop avant l'heure indiquée ; relancez la synchro après. Une première synchro (180 jours) est volontairement espacée pour rester sous la limite à la minute.

### Whoop : pas de jeton de rafraîchissement, la connexion tombe

La portée **offline** doit être cochée dans l'application Whoop : sans elle Whoop ne délivre pas de jeton de rafraîchissement. Cochez-la, puis reconnectez.

### Google Health : « compte non lié »

Le compte Google doit être lié à Google Health : ouvrez l'application mobile Google Health avec ce compte, ou migrez votre compte Fitbit vers Google. L'ancienne API Fitbit est coupée le 2 novembre 2026.

### Google Health : il faut me reconnecter toutes les semaines

Tant que le projet Google Cloud est en mode **Testing**, Google invalide l'autorisation 7 jours après chaque consentement. Synk l'annonce la veille (carte et bandeau du tableau de bord) ; cliquez sur **Connecter**, rien n'est perdu. Passer le projet **En production** supprime l'échéance (avertissement « application non vérifiée » à accepter pour votre propre compte).

### Oura : « 403 » ou abonnement

Oura répond 403 quand l'abonnement Oura est expiré ou qu'une portée manque (cochez daily, spo2 et workout dans l'application). Synk nomme la cause sans afficher le corps de la réponse. Une application Oura accepte dix utilisateurs sans validation.

### Une source échoue mais les autres se synchronisent

C'est voulu : l'erreur est nommée dans le message de synchro et dans **Synchronisations** (une ligne par source). Les analyses se calculent une fois, après toutes les sources.

### Import d'un fichier : annulé, refusé ou doublons

- **Annuler** n'écrit rien : un import est une seule transaction, une erreur fait de même.
- Un fichier tronqué ou corrompu est refusé avec son motif (fichier FIT incomplet, `export.zip` sans `export.xml`).
- Réimporter le même fichier, ou un export plus récent, n'ajoute aucun doublon.
- Apple Santé : un `export.zip` de plusieurs Go est accepté (lecture en flux). Les mesures écrites par une autre app dans Santé restent sous Apple Santé avec leur origine ; la même pesée présente aussi en connexion directe n'est montrée qu'une fois par jour.
- CSV : l'assistant propose la correspondance des colonnes, **Enregistrer** la garde pour les prochains fichiers aux mêmes colonnes. Le rapport LibreView est reconnu seul.

## Base de donnees

### Ou est stockee la base ?

`%APPDATA%/com.synk.health/synk.db` (Windows).

Pour ouvrir le dossier :

- Touche Windows + R -> `%APPDATA%/com.synk.health` -> Entree.

### "DB locked" / Plantage au demarrage

1. Fermer Synk completement (verifier dans le Gestionnaire des taches qu'aucun
   processus `synk.exe` ne reste).
2. Si le probleme persiste, redemarrer Windows (un processus Synk bloque peut tenir la base).
3. **Ne supprimer aucun fichier** `synk.db`, `synk.db-wal` ou `synk.db-shm` : le fichier `-wal` contient des ecritures deja validees qui ne sont pas encore dans `synk.db`. Si la base reste inaccessible, passer par la restauration d'une sauvegarde (section suivante).

### DB corrompue : restaurer une sauvegarde

1. Fermer Synk.
2. Renommer `synk.db` en `synk.db.broken`.
3. Copier la sauvegarde la plus recente depuis `%APPDATA%/com.synk.health/backups/daily/`
   (ou `backups/auto/`, ou `backups/`) vers `synk.db`.
4. Relancer Synk.

Quand Synk affiche un ecran d'erreur au demarrage, son bouton « Restaurer la sauvegarde »
fait tout cela seul.

Si pas de backup auto : importer un export JSON via la page **Exporter ->
Importer JSON** (l'export ne contient pas les jetons Withings : reconnectez ensuite le compte).

### Mes donnees ont disparu apres un lancement depuis l'application Claude

Avant la 3.1.0, un Synk lance depuis une session Claude ecrivait une partie de ses donnees dans le conteneur de l'application Claude, invisible depuis le raccourci du Bureau : la synchro semblait perdue et l'assistant de premier lancement revenait. Depuis la 3.1.0, Synk se relance seul hors de ce conteneur. Pour retrouver l'historique, ouvrir Parametres, Sources, reconnecter la source puis synchroniser : la synchro reprend depuis le dernier releve de la base.

### Reinitialisation complete

1. Fermer Synk.
2. Supprimer `%APPDATA%/com.synk.health/synk.db*` (db + shm + wal).
3. Relancer Synk : nouvelle base vierge, OnboardingWizard reaffiche.

> ATTENTION : cela efface TOUTES vos donnees locales. Faire un export JSON avant.

## Migrations

### "Failed to apply migration X" au demarrage

Le numero de migration est indique dans les logs. Verifier dans
`src-tauri/migrations/` si le fichier existe.

Si la base est dans un etat inconsistent (par exemple migration interrompue) :

1. Restaurer la derniere sauvegarde (commande `restore_latest_backup`, proposee par l'ecran d'erreur au demarrage).
2. Sinon, reinitialisation complete (voir ci-dessus) puis import JSON.

## Page Explorer (V1.0)

### "Aucune mesure pour cette metrique"

L'Explorer affiche des metriques **unifiees** (table `metrics` + lecture des
tables `*_legacy`). Si aucune donnee :

- La metrique n'est pas (encore) supportee par votre appareil Withings.
- Vous n'avez pas encore synchronise (Header -> Synchroniser).
- Pour les metriques **custom** ou **circumference** : il faut les saisir
  manuellement depuis la page Mensurations.

Voir `docs/metrics-v2.md` pour le catalogue des metriques.

## Logs et debug

### Voir les logs

Les logs Rust sont dans `%LOCALAPPDATA%\com.synk.health\logs\Synk.log` (dossier de
journal de Tauri sous Windows). Cinq fichiers d'un mega-octet au plus sont gardes :
le journal d'un demarrage rate survit au lancement suivant.

Augmenter la verbosite : lancer Synk avec la variable d'environnement `SYNK_LOG`
(`error`, `warn`, `info`, `debug` ou `trace` ; `info` par defaut). Une valeur
illisible garde `info` et ecrit un avertissement `SYNK_LOG=... illisible` au
demarrage. Exemple, dans PowerShell :

```powershell
$env:SYNK_LOG = "debug"; & "<dossier d'installation>\Synk.exe"
```

`RUST_LOG` n'est pas lue.

### Mode dev (developpeur)

```bash
git clone <repo> synk
cd synk
npm install
npm run tauri:dev
```

L'app demarre sur une copie de developpement (identifiant `com.synk.health.dev`, base et cle de chiffrement separees), avec rechargement a chaud et DevTools (F12). Ne jamais lancer `npm run tauri dev` nu : un build de dev sur l'identifiant de production est refuse au demarrage. Details : [dev-guide.md](./dev-guide.md).

## Performance

### L'app demarre lentement la premiere fois

Normal : creation de la base, application des migrations et semis des metriques
predefinies (quelques secondes). Les lancements suivants sont rapides.

### Sync prend > 30s

La premiere synchro d'une source remonte 180 jours d'historique. Les suivantes sont
incrementales et bien plus courtes.

Verifier dans **Synchronisations** la duree des syncs : si > 60s, probleme reseau ou
rate limiting.

## Migration echouee — comment restaurer ?

Synk V1.1.0 introduit un **filet de securite automatique** : avant chaque
mise a jour du schema DB (migration), un backup de `synk.db` est cree dans
`%APPDATA%\com.synk.health\backups\auto\synk_{date}_pre-v{n}.db`.
Les **3 backups les plus recents** sont conserves (rolling FIFO).

Si une migration echoue, l'app affiche un ecran « Mise à jour de la base
échouée (vN) » avec trois actions :

1. **Restaurer la sauvegarde** : prend la sauvegarde automatique la plus
   recente qui passe le controle d'integrite, la pose a la place de
   `synk.db` et relance le demarrage. La base cassee est preservee en
   `synk.db.broken-{ISO}` (avec ses fichiers `-wal` et `-shm`). Une
   sauvegarde illisible est ecartee et comptee dans le message affiche ; si
   aucune n'est lisible, rien n'est touche.
2. **Réessayer** : relance reellement le demarrage cote Rust (utile si le
   souci etait transitoire). Avant la 2.15.0, ce bouton ne faisait que
   recharger la page et reaffichait la meme erreur.
3. **Voir les détails techniques** : affiche le message d'erreur Rust brut.

Le meme ecran apparait, titre « Sauvegarde avant mise à jour impossible »,
si la sauvegarde qui precede une migration echoue : Synk refuse alors de
migrer sans filet, et la base reste dans son etat d'avant.

**Recommandation** : essaye d'abord « Restaurer la sauvegarde » — le pire
qui peut arriver est que tu perdes les donnees ajoutees depuis cette
sauvegarde (typiquement quelques heures, jamais plus de 24h vu le volume des
syncs).

## Que faire des fichiers `synk.db.broken-*` ?

Apres un « Restaurer la sauvegarde », l'ancienne DB cassee reste dans
`%APPDATA%\com.synk.health\synk.db.broken-{ISO}`. Tu peux :
- Les laisser : ils n'occupent pas grand chose et permettent une
  investigation post-mortem si besoin.
- Les supprimer : si tout fonctionne depuis 1-2 semaines, tu peux les
  effacer manuellement.

## La fenetre Synk ne s'ouvre pas quand je double-clique

Synk V1.1.0 utilise `tauri-plugin-single-instance` : si une instance est
deja ouverte (meme minimisee dans la barre des taches), un nouveau
double-clic **ramene la fenetre existante au front** au lieu d'ouvrir une
2eme app. Verifier la barre des taches Windows : Synk est probablement
deja la.

## "Le demarrage prend anormalement long" / Synk reste sur le spinner

V1.1.1+ inclut un **watchdog de 12 secondes** : si le boot Rust ne signale
pas "pret" dans ce delai, l'app affiche un ecran de diagnostic au lieu
d'un spinner infini.

Cet ecran montre l'etat reel du moteur Rust (`get_boot_state`) :

- **status: "ready"** → le boot est OK cote Rust, l'event a ete perdu.
  Cliquer **"Re-verifier l'etat du boot"** ou **"Recharger l'app"**
  resout le probleme instantanement.
- **status: "failed"** → le boot a echoue, l'erreur est affichee.
  Suivre la procedure de recovery (voir "Migration echouee" plus haut).
- **status: "loading"** → le boot Rust est encore en cours apres 12s :
  inhabituel. Causes possibles :
  - DB enorme et migration tres longue (rare)
  - Antivirus qui scanne `synk.exe` au premier lancement
  - Permissions refusees sur `%APPDATA%\com.synk.health`
  - `get_boot_state` indisponible (probleme Tauri runtime)

  Cliquer **"Ouvrir le dossier de donnees"** puis aller dans
  `..\..\Local\com.synk.health\logs\Synk.log` (ou directement
  `%LOCALAPPDATA%\com.synk.health\logs\Synk.log`). Filtrer sur
  `[boot` pour voir ou le boot s'est arrete. Partager les lignes
  pertinentes avec le mainteneur.

## Synk semble bloque mais aucun ecran d'erreur n'apparait

Possible si l'antivirus a quarantine `synk.exe` apres install. Verifier :

1. Quarantine de Windows Defender : ouvrir **Securite Windows** ->
   **Protection contre les virus** -> **Historique de protection** ->
   chercher des entrees recentes mentionnant `Synk` ou `synk.exe`.
2. Restaurer `synk.exe` depuis la quarantine OU ajouter une exception sur
   `C:\Program Files\Synk\` (ou le dossier d'install).
3. Relancer Synk.

Synk V1.1+ n'est pas signe (pas de certificat code-signing). C'est la cause
la plus frequente de quarantine intempestive sur premiere installation.

## Securite

### Mes tokens Withings sont-ils proteges sur mon PC ?

Depuis Synk V2.8.0, les tokens OAuth Withings (`access_token`, `refresh_token`) et les credentials d'app (`client_id`, `client_secret`) sont chiffres au repos dans la base SQLite via AES-256-GCM. La cle maitre est stockee dans le **Windows Credential Manager (DPAPI)**, liee a ton compte utilisateur Windows.

**Cela protege contre** :
- Un backup de `synk.db` qui serait exfiltre (synchronisation cloud accidentelle, copie sur USB)
- Une lecture par un autre utilisateur de ton PC Windows

**Cela ne protege pas contre** :
- Un attaquant qui a deja une session Windows active sur ton compte (acces complet au PC)
- Une attaque ciblee qui combine acces fichier + acces session utilisateur

### Qu'est-ce qui n'est pas chiffre ?

- La base de sante `synk.db` est **en clair** : elle n'est protegee que par les droits du compte Windows. Le chiffrement du disque (BitLocker) est la mesure attendue pour la proteger si le PC est perdu ou vole.
- Les exports CSV, PDF et JSON sont **en clair** : ils contiennent tes donnees de sante telles quelles. Range-les dans un dossier protege et supprime ceux dont tu n'as plus besoin.
- La cle maitre est lisible par tout programme qui tourne dans ta session Windows.
- Un secret chiffre n'est pas lie a sa colonne (pas de donnees associees) : on peut recopier un chiffre d'une colonne a l'autre. Limite acceptee, parce qu'il faut deja pouvoir ecrire dans la base pour cela.
- Les erreurs Withings (ecran, journal, `sync_log`) ne contiennent plus ni jeton ni corps de reponse : seulement une taille et une position.

### J'ai perdu mon profil Windows / reinstalle Windows / change de PC. Que se passe-t-il ?

La cle maitre etant liee au compte Windows, elle est perdue avec lui. Tes mesures sante restent intactes (elles ne sont pas chiffrees), mais Synk ne pourra plus dechiffrer tes tokens OAuth.

**Solution** : ouvre Synk, va dans **Parametres > Sources**, carte de chaque source touchee, puis **Deconnecter**, puis reconnecte ton compte Withings via le flow OAuth. Tu retrouveras toutes tes donnees historiques (Withings te les renvoie a la prochaine sync).

### Ecran "Stockage securise inaccessible" au demarrage

Cette erreur s'affiche si Synk n'arrive pas a acceder au Windows Credential Manager. Causes possibles :
- Permissions Windows insuffisantes (utiliser le mode administrateur ou verifier les droits du compte)
- Service "Credential Manager" Windows desactive (le reactiver via `services.msc`)
- Profil Windows corrompu

Cliquez sur "Reessayer" apres avoir corrige le probleme.

## Plus de questions

- [Guide utilisateur complet](./user-guide.md)
- [Architecture technique](./architecture.md)
- [Stack Metriques V2](./metrics-v2.md)

Si le probleme persiste, transmettre au mainteneur :
- Version de Synk (Parametres Windows, Applications installees, ligne Synk)
- Capture d'ecran de l'erreur
- Extrait des logs (`%LOCALAPPDATA%\com.synk.health\logs\Synk.log`, derniers 100 lignes, de preference avec `SYNK_LOG=debug`)
