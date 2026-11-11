# API des autres sources

Oura, Polar, Whoop, Google Health, puis les imports de fichiers. Withings, qui sert de référence pour l'OAuth, a son document : [api-withings.md](api-withings.md). Tout ce qui est dit ici vient du code de `src-tauri/src/sources/`.

## Ce que les marques partagent

| Sujet | Règle |
|---|---|
| Rappel OAuth | `http://localhost:17483/callback/<source>`, un serveur unique pour toutes les sources, écoute sur `127.0.0.1` et `[::1]`. L'URI exacte est à enregistrer chez la marque ; Paramètres > Sources l'affiche. |
| Identifiants | L'utilisateur crée une application chez la marque et saisit son `client_id` et son `client_secret` (chiffrés dans `source_credentials`). Le lien du portail développeur est dans la carte de la source. |
| Client HTTP | `sources/oauth/rest.rs` : https seul, aucune redirection, délai de 30 s, trois tentatives, un rafraîchissement sur 401, `Retry-After` plafonné à 60 s. Une marque donne ses particularités par une `Brand` (libellés d'erreur, en-têtes d'attente, pages maximales, fonctions `screen` et `throttle`). |
| Fenêtre | Première synchro : 180 jours en arrière ; ensuite, reprise 7 jours avant la dernière réussie (les valeurs révisées par la marque sont donc réécrites). |
| Écriture | Les sources API écrivent par `ObservationWriter::revising()` : une valeur révisée met la ligne à jour. Chaque ligne porte `source` et un `external_id`. |
| Révocation | À la déconnexion, `Source::revoke` prévient la marque quand elle l'offre (Oura, Polar, Whoop, Google Health, Withings). Un échec est un avertissement, la déconnexion locale a lieu. |

## Oura

| Élément | Valeur |
|---|---|
| Autorisation | `https://cloud.ouraring.com/oauth/authorize`, PKCE S256, portées `daily spo2 workout` |
| Jeton | `https://api.ouraring.com/oauth/token`, identifiants dans le corps |
| API | `GET /v2/usercollection/<collection>?start_date=&end_date=`, pagination par `next_token` |
| Collections lues | `sleep`, `daily_sleep`, `daily_activity`, `daily_readiness`, `daily_spo2`, `workout` |
| Étapes | Sommeil, Activité, Récupération, Entraînements (4) |
| Particularités | Jeton de rafraîchissement à usage unique (tout passe par `oauth::refresh_locked`). Une ligne de sommeil par jour : la période `long_sleep` la plus longue, avec le score de `daily_sleep`. Constantes ajoutées : `hrv`, `readiness_score`, `temperature_deviation`, plus le SpO2. La série de pouls à 5 minutes n'est pas importée. |
| Erreurs | 403 : abonnement expiré ou portée non accordée. 429 : `Retry-After`. |

## Polar (AccessLink v3)

| Élément | Valeur |
|---|---|
| Autorisation | `https://flow.polar.com/oauth2/authorization`, pas de PKCE, portée `accesslink.read_all` |
| Jeton | `https://polarremote.com/v2/oauth2/token`, identifiants en en-tête `Basic` ; hôte distinct de l'API |
| API | `https://www.polaraccesslink.com/v3/…` : `users` (enregistrement), `users/sleep`, `users/nightly-recharge`, `users/activities?from=&to=`, `exercises` |
| Étapes | Sommeil et récupération, Activité, Entraînements (3) |
| Particularités | Pas de transactions (dépréciées par Polar). Le jeton n'a pas de rafraîchissement : un 401 arrête la source et demande de se reconnecter. À la première synchro, `POST /v3/users` enregistre l'utilisateur (409 = déjà fait, noté dans `source_state`). Fenêtres imposées par Polar : sommeil et Nightly Recharge 28 jours, exercices 30 jours, activité par tranches de 27 jours (364 jours au plus). 204 = rien à lire. Le curseur n'avance qu'après l'écriture. |
| Constantes ajoutées | `ans_charge`, `nightly_recharge_status` |

## Whoop (API v2)

| Élément | Valeur |
|---|---|
| Autorisation | `https://api.prod.whoop.com/oauth/oauth2/auth`, pas de PKCE, portées `offline read:recovery read:sleep read:cycles read:workout` |
| Jeton | `https://api.prod.whoop.com/oauth/oauth2/token` |
| API | `GET /developer/v2/<route>?limit=25&start=`, pagination par `nextToken` ; routes `activity/sleep`, `recovery`, `cycle`, `activity/workout` |
| Étapes | Sommeil et récupération (jointure par `sleep_id`), Cycles, Entraînements (3) |
| Particularités | La portée `offline` est indispensable : sans elle, aucun jeton de rafraîchissement n'est délivré. Rotation du jeton : `oauth::refresh_locked`. Seuls les enregistrements `SCORED` sont écrits. |
| Débit | `pacer::Pacer` : fenêtre glissante de 60 s, 90 requêtes au plus, toute attente passe par l'horloge injectée `Clock`. Un 429 est classé par `throttle::classify` : limite à la minute = attente et rejeu (trois tentatives) ; quota du jour = arrêt sans rejeu, heure de reprise notée dans `source_state` (domaine `quota`), la synchro suivante ne touche pas au réseau tant qu'elle n'est pas passée. |
| Constantes ajoutées | `recovery_score`, `resting_heart_rate`, `strain` |

## Google Health (ex-Fitbit)

| Élément | Valeur |
|---|---|
| Autorisation | `https://accounts.google.com/o/oauth2/v2/auth`, PKCE S256, `access_type=offline` et `prompt=consent`, trois portées `googlehealth.sleep.readonly`, `googlehealth.activity_and_fitness.readonly`, `googlehealth.health_metrics_and_measurements.readonly` |
| Jeton | `https://oauth2.googleapis.com/token` |
| API | `https://health.googleapis.com/v4/users/me/…` : `identity`, `dataTypes/<type>/dataPoints` (liste, paginée par `pageToken`, 25 par page pour sommeil et séances), `dataTypes/<type>/dataPoints:dailyRollUp` |
| Types lus | `sleep`, `exercise`, `steps`, `distance`, `active-energy-burned`, `total-calories`, `daily-respiratory-rate`, `daily-resting-heart-rate`, `daily-heart-rate-variability`, `daily-oxygen-saturation`, `daily-sleep-temperature-derivations` |
| Étapes | Sommeil, Activité, Entraînements, Constantes (4) |
| Particularités | Le client Google Cloud est de type « Application Web ». `users/me/identity` est lu à chaque synchro : `ACCOUNT_NOT_LINKED` arrête la source. Les cumuls `dailyRollUp` se demandent par tranches de 90 jours (14 pour `total-calories`). |
| Ré-autorisation | Tant que le projet Google est en mode Testing, le jeton de rafraîchissement meurt après 7 jours. `OAuthSpec::refresh_lifetime_secs` note l'échéance à la connexion (`source_state`, domaine `refresh_expiry`), `oauth::reauth::reauth_status` annonce la ré-autorisation la veille (`list_sources` rend `reauth`, la carte et un bandeau du tableau de bord l'affichent), et un rafraîchissement réussi après l'échéance efface l'annonce. |

## Imports de fichiers

Les sources fichier n'ont ni connexion ni synchro. `import_source_file` (ou `import_csv_file`) reçoit un chemin choisi par la boîte de dialogue du système (`dialog:allow-open`, jamais un chemin saisi), lit le fichier dans `spawn_blocking`, envoie des lots de 2 000 lignes sur un canal borné, écrit dans **une seule transaction** et émet `import-progress` toutes les 200 ms. `cancel_import` annule : retour arrière complet. Un seul import à la fois ; pendant ce temps le portail `db/gate.rs` ralentit puis refuse les autres écritures et les synchros. Chaque observation a un `external_id` qui est l'empreinte SHA-256 de (type, début, fin, valeur, origine) : réimporter le même fichier n'ajoute rien.

| Source | Extensions | Lecture | Notes |
|---|---|---|---|
| Apple Santé | `zip`, `xml` | `export.xml` lu en flux dans le zip (le plus gros `.xml` hors `*_cda*`) par `quick-xml`, jamais chargé entier | Les `Record` enfants d'une `Correlation` sont ignorés (répétés au premier niveau). Tension appariée ; activité et sommeil agrégés par jour et par origine. L'app d'origine va dans la colonne `device`. Plafonds de `limits.rs` : 16 Gio, nœud de 4 Mo, profondeur 64. |
| FIT | `fit` | crate `fitparser` | Un fichier tronqué passe sans erreur dans la crate, donc `read::check_frame` ; la lecture est sous `catch_unwind`. |
| Traces | `gpx`, `tcx` | `quick-xml`, plafonds de `limits.rs` (1 Gio) | Le tracé n'est pas stocké, seul le résumé de la séance. |
| CSV | `csv`, `txt` | module `table/` | L'assistant (`inspect_csv_file`) propose les colonnes, `save_csv_mapping` mémorise la correspondance par signature (SHA-256 des noms de colonnes). Un préréglage : LibreView. |

Les plafonds protègent d'un fichier hostile : volume total, taille d'un nœud, profondeur, nombre d'éléments ; au-delà le fichier est refusé avec un message clair.
