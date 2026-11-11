# Guide utilisateur — Synk

Suivi sante personnel local, qui rassemble vos mesures Withings, Oura, Polar, Whoop, Google Health et vos fichiers exportes.

## 1. Installation

### Pre-requis

- Windows 10 / 11 (x64)
- Connexion internet pour synchroniser une marque (les imports de fichiers n'en ont pas besoin)
- Aucune installation de runtime (Tauri embarque tout)

### Installation

1. Recuperez l'installeur `Synk_<version>_x64-setup.exe`. Il n'y a pas de MSI. Chaque installeur est livre avec un fichier `.sha256` : comparez-le a l'empreinte du fichier si vous voulez verifier la copie.
2. Double-cliquez le fichier.
3. **Avertissement SmartScreen** : Windows affichera "Microsoft Defender SmartScreen
   a empeche le demarrage d'une application non reconnue". Cliquez sur **"Plus d'infos"**
   puis **"Executer quand meme"** (le binaire n'est pas signe, c'est normal).
4. Suivez l'assistant d'installation.

### Premier lancement

L'app cree sa base de donnees a `%APPDATA%/com.synk.health/synk.db` (Windows).
Toutes vos donnees restent locales sur ce fichier, sans telemetrie ni cloud. Seules les synchronisations que vous branchez contactent la marque concernee. La version de Synk se lit dans les Parametres Windows, Applications installees, ligne Synk.

L'assistant de bienvenue s'ouvre une fois ; vous pouvez le quitter des la premiere etape (« Passer l'assistant ») ou avec Echap, et il ne reviendra pas. Il ne s'ouvre que sur une installation vide : des que Synk a des mesures ou une source connectee, il ne se montre plus. Lance depuis l'application Claude, Synk se relance seul hors de son conteneur, pour lire et ecrire la meme base que le raccourci du Bureau.

## 2. Brancher une source

Synk lit vos mesures chez la marque qui les produit, ou dans un fichier exporté. Tout se règle dans **Paramètres > Sources** : une carte par marque (état, dernière synchronisation, identifiants, **Connecter**, **Déconnecter**) et, en dessous, le bloc **Importer un fichier**. Le bouton **Synchroniser** du bandeau lance toutes les sources connectées, celui d'une carte ne synchronise que sa marque ; une source qui échoue n'arrête pas les autres. Quand deux sources donnent la même mesure le même jour, l'écran n'en montre qu'une : l'ordre se règle dans l'onglet **Priorités**.

Chaque marque demande de créer **votre propre application** chez elle (gratuit, quelques minutes) et de déclarer l'**URL de rappel** que la carte affiche, à l'identique : un caractère de différence et la marque refuse la connexion. Synk n'écoute que sur votre PC (`localhost`, port 17483) et ne garde l'identifiant, le secret et les jetons que chiffrés, sur ce PC.

| Source | Où créer l'application | URL de rappel à déclarer | Ce qu'il faut savoir |
|---|---|---|---|
| Withings | developer.withings.com/dashboard | `http://localhost:17483/callback` | rien de particulier |
| Oura | portail développeur Oura, « My Applications » | `http://localhost:17483/callback/oura` | dix utilisateurs au plus sans validation d'Oura, ce qui suffit |
| Polar | admin.polaraccesslink.com | `http://localhost:17483/callback/polar` | Polar ne garde que 28 jours de sommeil et 30 jours d'entraînements : synchroniser au moins une fois par mois |
| Whoop | developer-dashboard.whoop.com | `http://localhost:17483/callback/whoop` | quota de 10 000 requêtes par jour, la synchro s'arrête puis reprend toute seule |
| Google Health (ex-Fitbit) | console.developers.google.com | `http://localhost:17483/callback/google_health` | en mode Testing, reconnexion tous les 7 jours |
| Apple Santé, FIT, GPX, TCX, CSV | aucune application : un fichier | sans objet | **Importer un fichier**, tout ou rien |

### Withings

Synk a besoin de credentials OAuth Withings (gratuits) pour synchroniser vos donnees.

#### Creer une application Withings developer

1. Rendez-vous sur https://developer.withings.com/dashboard/.
2. Connectez-vous avec votre compte Withings personnel.
3. Cliquez **"+ Create an application"**.
4. Renseignez :
   - **Description** : "Synk Desktop"
   - **Callback URI** : `http://localhost:17483/callback`
5. Validez et notez le **Client ID** et le **Client Secret**.

#### Renseigner dans Synk

1. Ouvrez Synk -> **Parametres** (derniere entree de la sidebar).
2. Colonne **Sources**, carte **Withings** : collez le Client ID et le Client Secret, validez.
3. Cliquez **Connecter**.
4. Votre navigateur s'ouvre sur Withings ; autorisez l'acces.
5. La fenetre du navigateur affiche "Authentification reussie" ; revenez dans Synk.
6. La premiere synchronisation demarre automatiquement.

> Si la connexion echoue ou expire (delai 120 s), l'assistant de premier lancement ou la page
> Parametres affichent un bouton **Reessayer**.

### Oura

1. Connectez-vous au portail développeur d'Oura avec votre compte Oura, ouvrez **My Applications** et créez une application.
2. Déclarez comme URL de rappel (redirect URI) exactement `http://localhost:17483/callback/oura`.
3. Cochez les portées **daily**, **spo2** et **workout**, pas davantage : ce sont les seules que Synk lit.
4. Copiez l'identifiant client et le secret dans la carte Oura de Synk, puis **Connecter**. Le navigateur s'ouvre sur Oura, autorisez l'accès.

La première synchro remonte 180 jours, les suivantes reprennent une semaine en arrière (une nuit n'apparaît chez Oura qu'à l'ouverture de l'application, parfois tard). Synk range le sommeil (score, fréquence cardiaque moyenne et minimale, respiration), l'activité du jour, les entraînements, la variabilité cardiaque, le score de préparation, l'écart de température cutanée et la SpO2 de la nuit. La fréquence cardiaque toutes les cinq minutes n'est pas importée. Si Oura limite le débit, Synk attend le délai demandé puis réessaie.

### Polar

1. Connectez-vous à admin.polaraccesslink.com avec votre compte Polar Flow et créez un client (**Create client**).
2. Déclarez `http://localhost:17483/callback/polar` à l'identique dans les URL de redirection du client.
3. Sur account.polar.com, acceptez tous les consentements de données : sans eux Polar refuse d'enregistrer l'utilisateur et Synk affiche une erreur de consentement.
4. Copiez l'identifiant et le secret dans la carte Polar de Synk, puis **Connecter**.

**Synchronisez au moins une fois par mois.** Polar ne garde que 28 jours de sommeil et de Nightly Recharge et 30 jours d'entraînements : une donnée qui sort de cette fenêtre avant d'avoir été synchronisée ne peut plus être relue par Synk (elle reste dans Polar Flow). L'activité quotidienne remonte plus loin (six mois à la première synchro). Le jeton de Polar ne se rafraîchit pas : s'il est révoqué, Synk demande de reconnecter. Synk range le sommeil, Nightly Recharge (VFC, charge du système nerveux autonome, statut, respiration), l'activité et les entraînements ; le poids de Polar Flow n'est pas lu, gardez votre balance comme source.

### Whoop

1. Connectez-vous à developer-dashboard.whoop.com avec votre compte Whoop, créez une équipe puis une application.
2. Cochez les portées **read:recovery**, **read:sleep**, **read:cycles** et **read:workout**, et laissez l'accès hors ligne (**offline**) activé : sans lui Whoop ne délivre pas de jeton de rafraîchissement et la connexion ne durerait pas.
3. Déclarez `http://localhost:17483/callback/whoop` à l'identique. Si le tableau de bord Whoop refuse une adresse en `http`, ne poursuivez pas : le cas n'est pas tranché par la documentation.
4. Copiez l'identifiant et le secret dans la carte Whoop de Synk, puis **Connecter**.

Whoop limite l'application à 100 requêtes par minute et 10 000 par jour. Synk espace ses requêtes pour ne jamais dépasser la limite à la minute (une première synchro de 180 jours est donc plus lente, c'est voulu). Si le quota du jour est atteint, la synchro s'arrête, garde ce qu'elle a écrit et indique l'heure de reprise ; elle ne rappelle pas Whoop avant. Synk range le sommeil (étapes, score de performance, efficacité, respiration), la récupération (score, fréquence cardiaque au repos, VFC, SpO2, température cutanée), les cycles (calories, effort du jour) et les entraînements. Whoop ne mesure pas de pas : la courbe des pas reste celle de vos autres sources.

### Connecter Google Health (ex-Fitbit)

L'ancienne API Fitbit est coupée le 2 novembre 2026 : les données Fitbit passent désormais par Google Health. Synk lit votre compte Google Health avec votre propre projet Google Cloud, en lecture seule (sommeil, activité, séances, fréquence cardiaque au repos, VFC, SpO2, température).

1. Votre compte Google doit être lié à Google Health : ouvrez l'application mobile Google Health avec ce compte, ou migrez votre compte Fitbit vers Google (aide Fitbit « migrer vers un compte Google »). Sans cela Google répond « compte non lié ».
2. Dans Google Cloud (console.developers.google.com), créez un projet et activez **Google Health API**.
3. Écran de consentement (Audience) : type **Externe**, statut **Testing**, puis ajoutez votre adresse Google dans **Utilisateurs de test**.
4. Accès aux données : ajoutez les portées `googlehealth.sleep.readonly`, `googlehealth.activity_and_fitness.readonly` et `googlehealth.health_metrics_and_measurements.readonly`.
5. Clients : créez un client **Application Web**, et déclarez comme URI de redirection autorisé exactement l'adresse affichée par Synk dans Paramètres > Sources > Google Health (`http://localhost:17483/callback/google_health`).
6. Copiez l'ID client et le secret dans la carte Google Health de Synk, puis cliquez sur **Connecter**.

**Pourquoi il faut se reconnecter toutes les semaines.** Tant que le projet Google reste en mode Testing, Google invalide l'autorisation 7 jours après chaque consentement, quoi que fasse Synk. La veille de l'échéance, un bandeau en haut du tableau de bord et la carte de la source l'annoncent : cliquez simplement sur **Connecter** (aucune donnée n'est perdue, la synchro reprend là où elle s'était arrêtée). Si vous voulez supprimer cette contrainte, passez le projet en **En production** dans l'écran de consentement : Google affiche alors un avertissement « application non vérifiée » que vous acceptez pour votre propre compte, et l'autorisation ne s'éteint plus (sauf révocation ou six mois sans usage). Synk constate de lui-même qu'une autorisation survit à l'échéance et cesse alors d'annoncer.

### Déconnecter une source

**Déconnecter** efface chez Synk l'identifiant, le secret et les jetons de la source ; les mesures déjà importées restent. **Déconnecter** demande une confirmation qui dit ce qui est effacé et ce qui reste. Chez les cinq marques, Synk révoque aussi l'accès chez la marque ; si la marque ne répond pas, Synk se déconnecte quand même et un avertissement vous demande de retirer l'accès depuis votre compte chez la marque.

### Importer Apple Sante

Apple ne donne aucun acces a Sante depuis un PC : la seule voie est l'export, que Synk lit sans le charger en memoire (un export de plusieurs Go est accepte).

1. Sur l'iPhone : app **Sante**, votre photo de profil en haut a droite, **Exporter toutes les donnees de sante**. L'iPhone prepare un fichier `export.zip`.
2. Transferez `export.zip` sur ce PC (cable, partage de fichiers ou stockage en ligne).
3. Dans Synk : **Parametres > Sources > Importer un fichier**, **Choisir un fichier**, puis `export.zip`.
4. Une barre de progression s'affiche ; **Annuler** arrete l'import et **rien n'est garde** (tout est ecrit en une seule fois a la fin).
5. Le resume dit ce qui a ete ajoute (mesures, tensions, nuits, jours d'activite, seances) et ce qui a ete ignore, avec la raison.

Reimporter le meme fichier, ou un export plus recent, n'ajoute aucun doublon : seules les lignes nouvelles sont ecrites. Les mesures qu'une autre app a ecrites dans Sante (balance Withings, Oura...) sont rangees sous Apple Sante avec le nom de l'app d'origine. Si la meme pesee existe aussi par la connexion directe, l'ecran n'en montre qu'une par jour (l'autre reste un point secondaire) ; l'ordre se regle dans l'onglet **Priorites**.

Non importe : la frequence cardiaque continue, la nutrition, les ECG, les itineraires et les dossiers cliniques. Le sommeil est reconstitue nuit par nuit, les siestes de 12 h a 18 h sont ecartees, et pour les pas, la distance et les calories d'une journee Synk garde l'appareil le plus complet au lieu d'additionner l'iPhone et la montre.

### Importer des fichiers FIT, GPX, TCX ou CSV

Meme endroit : **Parametres > Sources > Importer un fichier**, une ligne par type de fichier. Chaque ligne dit ou trouver le fichier. Un import est tout ou rien (une erreur ou **Annuler** n'ecrit rien) et reimporter le meme fichier n'ajoute aucun doublon.

- **Fichiers FIT** (Garmin, Coros, Suunto, Zepp via Gadgetbridge) : seances avec distance, denivele, calories et frequence cardiaque ; pesees, tensions et nuits quand le fichier les contient. Sur Garmin Connect : l'activite, puis **Exporter l'original**. Un fichier tronque ou corrompu est refuse avec son motif.
- **Traces GPX et TCX** (Strava, Komoot, Garmin Connect) : une seance par trace. Seul le resume est garde (duree, distance, denivele, frequence cardiaque), pas le trace sur une carte.
- **Fichiers CSV** (balance, tensiometre, lecteur de glycemie) : apres le choix du fichier, un assistant propose la correspondance des colonnes (colonne de la date, forme de la date, mesure et unite de chaque colonne, separateur des decimales). Corrigez ce qui est faux, **Importer le fichier**. **Enregistrer** garde la correspondance sous un nom : au prochain fichier aux memes colonnes, elle est proposee et s'applique en un clic. Le rapport LibreView (FreeStyle Libre) est reconnu seul ; pour les autres marques l'assistant devine les colonnes usuelles.

## 3. Navigation

Sidebar organisee en 5 sections (cf. CLAUDE.md `NAV_SECTIONS`) :

| Section | Pages |
|---|---|
| — | **Tableau de bord** (KPIs principaux) |
| **Corps** | Composition, Tension, Mensurations |
| **Activite** | Activite, Entrainements, Sommeil, Coeur/ECG |
| **Suivi** | Medicaments, Analyses labo, Journal |
| **Analyse** | Profil, Correlations, Constats (ex-Insights), **Explorer** (V1.0), Alertes, Synchronisations (ex-Timeline) |
| Secondaire | Exporter, Parametres |

La sidebar est repliable (bouton chevron en haut, ou `Ctrl+B`) et se souvient de son etat. Vous pouvez masquer des onglets
depuis Parametres -> Apparence (le tableau de bord, Exporter et Parametres restent toujours visibles).

### Raccourcis clavier

| Raccourci | Action |
|---|---|
| `Ctrl+Maj+S` | Synchroniser |
| `Ctrl+Maj+A` | Ajout rapide (poids, tension, mensuration) |
| `Ctrl+B` | Replier ou deplier la barre laterale |

Ils sont sans effet pendant la saisie dans un champ. Les cartes du tableau de bord sont des liens vers la page de leur mesure.

## 4. Page Explorer (nouveaute V1.0)

L'**Explorer** liste les metriques de Synk (composition
corporelle, mensurations, body scan, constantes des sources, personnalisees).

- Choisissez une categorie (chips en haut).
- Selectionnez une metrique dans le menu deroulant.
- Le graphique affiche la serie temporelle, avec options 7j / 30j / 90j / 1 an / Tout.
- Stats : derniere valeur, moyenne, min/max, delta sur la periode.
- Tableau des 10 dernieres mesures.

Quand plusieurs sources donnent la meme mesure le meme jour, la courbe n'en montre
qu'une (celle de l'onglet **Priorites**, sinon la plus recente) et dessine les autres en points
discrets (cf. `docs/metrics-v2.md`).

## 5. Sauvegarde et restauration

### Sauvegardes automatiques

Synk copie votre base **au demarrage, puis verifie chaque heure** (si la derniere copie date de plus de
20 h) dans `%APPDATA%\com.synk.health\backups\daily\` et garde les **14 plus recentes**.
Il en cree aussi une **avant chaque mise a jour du schema** (voir la section 6). Un echec de
la copie quotidienne n'empeche pas Synk de demarrer : un message rouge s'affiche et la
cause est consignee dans le journal d'erreurs.

**Parametres -> Sauvegardes** liste les cinq plus recentes (date, sorte, taille), cree une
sauvegarde a la demande et montre la plus recente dans l'Explorateur.

Ces copies restent **sur le meme disque** que la base. Pour une copie ailleurs, ajoutez
`%APPDATA%\com.synk.health` a l'outil de sauvegarde de votre poste.

### Export complet (JSON)

Page **Exporter -> Exporter JSON** : ecrit `synk_export_AAAAMMJJ_HHMMSS.json` dans le dossier
d'export. Il contient les donnees de sante : composition corporelle, tension, sommeil, activite,
entrainements, ECG, mesures vitales, mensurations et leurs types, metriques et leurs
definitions, objectifs, profil, medicaments et leurs prises, analyses de laboratoire, journal,
alertes et leurs declenchements, tailles de vetements.

Ne sont **pas** exportes : les jetons et identifiants des sources (secrets chiffres avec la cle de
votre poste, jamais ecrits en clair), l'etat de synchronisation, les journaux techniques,
les appareils (resynchronises depuis Withings), les correspondances CSV memorisees, les alertes automatiques (regenerees apres chaque
synchro), les preferences du poste et quelques tables sans lecteur dans l'application.

Ce fichier sert a lire ou migrer vos donnees ; la copie restaurable de la base est celle de
**Parametres -> Sauvegardes**. Une periode vide n'ecrit aucun fichier CSV ou PDF : l'ecran le
dit au lieu d'annoncer un succes (le CSV et le PDF proposent « Tout l'historique », coche par defaut).

**Les fichiers exportes ne sont pas chiffres** : ils contiennent vos donnees de sante en clair (la page Exporter le rappelle). Rangez-les dans un dossier protege et supprimez ceux dont vous n'avez plus besoin.

### Importer un fichier JSON

Page **Exporter -> Importer JSON** : l'ecran liste ce que le fichier contient, puis demande
confirmation. L'import **fusionne** : une ligne deja presente (meme date et meme source, meme
nom de medicament, etc.) est ignoree, rien n'est ecrase, et reimporter le meme fichier n'ajoute
rien. Le profil n'est complete que la ou un champ est vide. Les valeurs hors plage physiologique
font refuser tout le fichier, sans rien ecrire. Plafond : 64 Mo. Les fichiers produits par Synk
2.15.0 et avant restent acceptes.

### Migration entre postes

1. Exportez en JSON depuis le poste source.
2. Installez Synk sur le poste cible et reconnectez vos sources (les secrets ne voyagent pas).
3. Importez le JSON.

## 6. Recuperation automatique

Synk cree automatiquement des backups de ta base SQLite **avant chaque
mise a jour du schema DB** (migrations versionnees). Tu n'as rien a
configurer.

### Ou sont les backups ?

`%APPDATA%\com.synk.health\backups\auto\` (Windows).
Format : `synk_{YYYY-MM-DD_HH-MM-SS}_pre-v{schema_version}.db`

Les **3 backups les plus recents** sont conserves automatiquement
(rolling FIFO).

### Comment restaurer ?

Si une migration echoue au lancement, Synk affiche directement un ecran
« Mise à jour de la base échouée » avec un bouton **« Restaurer la
sauvegarde »**. Un clic suffit : Synk choisit la sauvegarde automatique la
plus recente qui passe le controle d'integrite (une sauvegarde abimee est
ecartee et signalee), met la base cassee de cote en `synk.db.broken-{ISO}`,
pose la sauvegarde a sa place puis relance le demarrage. Si aucune
sauvegarde n'est lisible, la base actuelle n'est pas touchee.

La restauration puise dans **toutes** les sauvegardes (quotidiennes, d'avant mise a jour et
manuelles) : c'est la plus recente lisible qui est posee.

Si c'est la sauvegarde d'avant mise a jour qui echoue (disque plein, base
illisible), Synk n'applique pas la mise a jour : l'ecran « Sauvegarde avant
mise à jour impossible » propose les memes boutons.

### Sauvegarde manuelle (en plus des automatiques)

Tu peux aussi faire une sauvegarde a tout moment depuis :
**Parametres -> Sauvegardes -> Creer une sauvegarde maintenant** (commande
`create_backup`). Elles vont dans `backups\` (ni `auto\` ni `daily\`) et les
**5 plus recentes** sont gardees.

### Resume

| Sorte | Dossier | Quand | Gardees |
|---|---|---|---|
| Quotidienne | `backups\daily\` | au demarrage, si la derniere a plus de 20 h | 14 |
| Avant mise a jour | `backups\auto\` | avant une migration du schema | 3 |
| Manuelle | `backups\` | bouton de Parametres | 5 |

## 7. Personnalisation

- **Theme** : clair / sombre (Parametres).
- **Couleur d'accent** : indigo, emerald, rose, amber, slate (5 palettes).
- **Intervalle d'auto-sync** : desactivee / 1 h / 6 h / 12 h / 24 h (Parametres > Sources).
- **Onglets masques** : decoche les pages que tu n'utilises pas.

## 8. Troubleshooting rapide

Voir [`docs/faq.md`](./faq.md) pour la FAQ complete (token expire, sync echoue,
DB corrompue, etc.).

## 9. Aller plus loin

- [Architecture technique](./architecture.md)
- [Base de donnees](./database.md)
- [API Withings](./api-withings.md) et [API des autres sources](./api-sources.md)
- [Stack Metriques V2](./metrics-v2.md)
- [Build et release](./release.md)
- [Guide developpeur](./dev-guide.md)
