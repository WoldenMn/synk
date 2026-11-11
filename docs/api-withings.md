# API Withings

L'integration avec l'API publique Withings, telle que le code la parle aujourd'hui (`src-tauri/src/sources/withings/`). Les quatre autres marques et les imports de fichiers sont dans [api-sources.md](api-sources.md) ; le mecanisme OAuth commun est decrit ici une fois pour Withings et vaut pour toutes.

---

## 1. Flux OAuth2

### URL d'autorisation

```
https://account.withings.com/oauth2_user/authorize2
```

### URL du token

```
https://wbsapi.withings.net/v2/oauth2
```

### Redirect URI

```
http://localhost:17483/callback
```

Un serveur HTTP local (`tiny_http`, `sources/oauth/server.rs`) ecoute sur le port 17483, en `127.0.0.1` et en `[::1]`. Il est partage par toutes les sources : le chemin `/callback/<source>` designe la marque, et l'ancien `/callback` va a Withings, dont l'URL de rappel enregistree ne change donc pas. Il ignore le favicon et toute requete hors rappel, et s'arrete quand plus aucune connexion n'est en attente.

### Scopes demandes

```
user.info,user.metrics,user.activity
```

### Deroulement du flux

1. **`source_start_auth`** (source `withings`) genere un `state` aleatoire et construit l'URL d'autorisation avec `response_type=code`, `client_id`, `redirect_uri`, `state` et `scope` (plus un defi PKCE S256 pour les marques qui l'utilisent ; pas Withings).
2. L'URL est retournee au frontend qui l'ouvre dans le navigateur par defaut (`opener:allow-open-url` limite aux hotes des marques).
3. Si aucun serveur de rappel ne tourne, un thread le demarre ; l'etat attendu de la source est celui de son dernier essai.
4. Withings redirige vers `/callback?code=XXX&state=YYY`.
5. Le serveur verifie que `state` correspond a l'essai en cours (protection CSRF).
6. Si le `state` est invalide ou le `code` absent, une page d'erreur HTML est renvoyee et l'evenement `withings-auth-error` part.
7. Si tout est valide, une page de succes est affichee, le code est echange contre des jetons et `withings-auth-success` part. Sans rappel en 120 secondes : `withings-auth-timeout`.

### Echange de code

L'echange se fait via un POST sur `/v2/oauth2` avec :

| Parametre | Valeur |
|---|---|
| `action` | `requesttoken` |
| `grant_type` | `authorization_code` |
| `client_id` | saisi par l'utilisateur (Parametres > Sources), chiffre au repos |
| `client_secret` | saisi par l'utilisateur, chiffre au repos |
| `code` | code recu du callback |
| `redirect_uri` | `http://localhost:17483/callback` |

### Stockage des tokens

Les jetons sont sauvegardes, chiffres (`enc:v1:`), dans `source_auth` avec un UPSERT sur `source_id = 'withings'` (`sources/store/auth.rs`). `withings_auth` n'est plus qu'une vue de lecture sur cette ligne. Colonnes :

- `access_token` et `refresh_token` (chiffres)
- `expires_at` : timestamp calcule (`now + expires_in`)
- `token_type`
- `user_ref` (l'identifiant Withings de l'utilisateur, `withings_user_id` dans la vue)
- `created_at`, `updated_at`

### Rafraichissement automatique

La fonction `get_valid_client` verifie avant chaque appel API si le token expire dans moins de **300 secondes** (5 minutes). Si oui, un refresh est effectue automatiquement via un POST sur `/v2/oauth2` avec `grant_type=refresh_token`. Les nouveaux tokens sont sauvegardes immediatement.

Tout rafraichissement passe par `oauth::refresh_locked` (le `refresh_locked` de `sources/withings/client/tokens.rs` n'en est que l'enveloppe), sous le verrou de la source (`oauth::refresh::lock_for`) : la fonction relit la base une fois le verrou pris et reutilise un jeton deja renouvele par un autre appel, sinon elle echange le refresh_token (a usage unique) et enregistre le couple chiffre. `get_valid_client` (jeton proche de l'expiration) et `WithingsClient::post` (jeton refuse) l'appellent tous les deux. Un jeton encore refuse apres un rafraichissement arrete l'appel avec « Acces expire » : pas de boucle, pas de second echange.

### Deconnexion

`source_disconnect` (source `withings`) revoque d'abord l'acces chez Withings (`sources/withings/revoke.rs` : un nonce signe en HMAC-SHA256 par `/v2/signature`, puis `action=revoke` sur `/v2/oauth2`), puis supprime la ligne de `source_auth` et remet les curseurs de `source_state` a 0. Un echec de la revocation est un avertissement (`DisconnectReport`) : la deconnexion locale a toujours lieu.

---

## 2. Endpoints utilises

Toutes les requetes sont des **POST** authentifiees via le header `Authorization: Bearer {access_token}`. Le corps est envoye en `application/x-www-form-urlencoded`.

| Endpoint | Path | Action | Parametres temporels | data_fields |
|---|---|---|---|---|
| **Mesures corporelles** | `/measure` | `getmeas` | `lastupdate` (timestamp), `meastypes`, `offset` | -- |
| **Activite** | `/v2/measure` | `getactivity` | `startdateymd`, `enddateymd`, `offset` | `steps,distance,elevation,soft,moderate,intense,active,calories,totalcalories,hr_average,hr_min,hr_max,hr_zone_0,hr_zone_1,hr_zone_2,hr_zone_3` |
| **Sommeil** | `/v2/sleep` | `getsummary` | `startdateymd`, `enddateymd`, `offset` | `total_sleep_time,deepsleepduration,lightsleepduration,remsleepduration,wakeupduration,durationtosleep,durationtowakeup,sleep_score,hr_average,hr_min,hr_max,rr_average,snoring,snoringepisodecount,sleep_efficiency,nb_rem_episodes,out_of_bed_count,waso,total_timeinbed,apnea_hypopnea_index,breathing_disturbances_intensity,asleepduration,night_events` |
| **Entrainements** | `/v2/measure` | `getworkouts` | `startdateymd`, `enddateymd`, `offset` | `steps,distance,elevation,calories,hr_average,hr_min,hr_max,hr_zone_0,hr_zone_1,hr_zone_2,hr_zone_3,spo2_average,intensity,pause_duration,effortScore,pool_laps,pool_length` |
| **Coeur / ECG** | `/v2/heart` | `list` | `startdate`, `enddate` (timestamps Unix), `offset` | -- |
| **Appareils** | `/v2/user` | `getdevice` | -- | -- |

### Lookback initial

Quand le curseur du domaine vaut 0 (premiere synchronisation), le lookback est de **180 jours** (`BACKFILL_DAYS`, `sync/helpers.rs`) ; l'endpoint Coeur remonte lui aussi de 180 jours. Les curseurs sont les lignes de `source_state` de la source `withings`.

### Validation des reponses

Chaque reponse est wrappee dans `WithingsResponse<T>` contenant `status` et `body`. Si `status != 0`, une erreur est remontee. En cas d'echec de parsing JSON, le message donne la taille de la reponse et la position de l'erreur ; un apercu masque par `utils::redact` n'est ecrit qu'au niveau `debug`. Le corps d'une reponse n'entre jamais dans une erreur.

Withings repond en HTTP 200 et met l'erreur dans le champ `status` de l'enveloppe ; le code HTTP seul ne suffit jamais. `WithingsClient::post` lit donc le statut :

| Statut d'enveloppe | Sens chez Withings | Traitement |
|---|---|---|
| `401` (ou HTTP 401) | jeton invalide ou expire | un rafraichissement sous verrou, puis un nouvel essai ; un second refus arrete l'appel |
| `503` | « Invalid Params », pas un service indisponible | erreur immediate, jamais reessayee |
| `601` (ou HTTP 429) | trop de requetes | reessaye (1 s, 2 s, 4 s) |
| autre non nul | erreur d'API | erreur immediate avec le code |

Sources (verifiees le 2026-10-03) : la liste des statuts reprise par `maximebf/python-withings` (503 « Invalid Params », 601 « Too Many Request »), le ticket `restsharp/RestSharp#934`, le ticket `the-momentum/open-wearables#1724` et le noeud `fjacquet/n8n-nodes-withings` (statut dans l'enveloppe, HTTP 200 sur un jeton expire). Les echanges de jeton (`/v2/oauth2`) lisent le meme statut avant de decoder le corps (`request_token`).

### Faux serveur de test

`src-tauri/tests/integration/withings_fake_server.rs` teste le client et les modules de synchro contre un serveur `tiny_http` local (`tests/integration/support/fake_server.rs`, generique : repond selon une fonction du test et consigne chemin, corps et en-tete `Authorization` de chaque appel). `WithingsClient::at(base, jeton)` et les variantes `*_at(base, ...)` des fonctions d'OAuth visent ce serveur ; la production lui passe `API_BASE` (`get_valid_client`). Le client HTTP vient de `utils::http::client`, comme pour les autres marques : https seulement quand la base est https, aucune redirection suivie, delai de 30 s ; le jeton d'acces reste un `Zeroizing<String>` et part par `bearer_auth` (en-tete marque sensible). Les reponses figees vivent dans `tests/fixtures/withings/` (forme de l'API, valeurs inventees). Aucun test n'appelle Withings ni le trousseau.

---

## 3. Types de mesure

### Composition corporelle (meastypes: `1,5,6,8,76,77,88`)

| ID | Constante Rust | Description | Unite |
|---|---|---|---|
| 1 | `MEASURE_WEIGHT` | Poids | kg |
| 5 | `MEASURE_FAT_FREE_MASS` | Masse maigre | kg |
| 6 | `MEASURE_FAT_RATIO` | Taux de graisse | % |
| 8 | `MEASURE_FAT_MASS` | Masse grasse | kg |
| 76 | `MEASURE_MUSCLE_MASS` | Masse musculaire | kg |
| 77 | `MEASURE_HYDRATION` | Hydratation | % |
| 88 | `MEASURE_BONE_MASS` | Masse osseuse | kg |

### Tension arterielle (meastypes: `9,10,11`)

| ID | Constante Rust | Description | Unite |
|---|---|---|---|
| 9 | `MEASURE_DIASTOLIC` | Pression diastolique | mmHg |
| 10 | `MEASURE_SYSTOLIC` | Pression systolique | mmHg |
| 11 | `MEASURE_HEART_RATE` | Frequence cardiaque | bpm |

### Vitaux (meastypes: `12,54,71,73,123,155,167,168,169,170,226`)

| ID | Constante Rust | Description | Nom interne | Unite |
|---|---|---|---|---|
| 12 | `MEASURE_TEMPERATURE` | Temperature | `temperature` | C |
| 54 | `MEASURE_SPO2` | Saturation en oxygene | `spo2` | % |
| 71 | `MEASURE_BODY_TEMPERATURE` | Temperature corporelle | `body_temperature` | C |
| 73 | `MEASURE_SKIN_TEMPERATURE` | Temperature cutanee | `skin_temperature` | C |
| 91 | `MEASURE_PULSE_WAVE_VELOCITY` | Vitesse d'onde de pouls | `pulse_wave_velocity` | m/s |
| 123 | `MEASURE_VO2MAX` | VO2 Max | `vo2max` | ml/min/kg |
| 155 | `MEASURE_VASCULAR_AGE` | Age vasculaire | `vascular_age` | annees |
| 167 | `MEASURE_NERVE_HEALTH` | Sante nerveuse | `nerve_health` | score |
| 168 | `MEASURE_EXTRACELLULAR_WATER` | Eau extracellulaire | `extracellular_water` | kg |
| 169 | `MEASURE_INTRACELLULAR_WATER` | Eau intracellulaire | `intracellular_water` | kg |
| 170 | `MEASURE_VISCERAL_FAT` | Graisse viscerale | `visceral_fat` | indice |
| 226 | `MEASURE_BMR` | Metabolisme de base | `bmr` | kcal |

### Conversion des valeurs

Les mesures Withings utilisent une notation `value * 10^unit`. La fonction `convert_measure` effectue cette conversion :

```rust
pub fn convert_measure(value: i64, unit: i64) -> f64 {
    value as f64 * 10_f64.powi(unit as i32)
}
```

Exemple : `value = 7234`, `unit = -2` donne `72.34 kg`.

### Classification

Deux fonctions de classification determinent le routage des mesures :

- **`is_body_composition()`** : types 1, 5, 6, 8, 76, 77, 88 -> table `body_measures`
- **`is_blood_pressure()`** : types 9, 10, 11 -> table `blood_pressure`
- **`vital_type_name()`** : types 12, 54, 71, 73, 91, 123, 155, 167, 168, 169, 170, 226 -> table `vitals`

---

## 4. Categories d'entrainement

| ID | Label francais |
|---|---|
| 1 | Marche |
| 2 | Course |
| 3 | Randonnee |
| 4 | Patinage |
| 5 | BMX |
| 6 | Velo |
| 7 | Natation |
| 8 | Surf |
| 9 | Kitesurf |
| 10 | Planche a voile |
| 11 | Bodyboard |
| 12 | Tennis |
| 13 | Tennis de table |
| 14 | Squash |
| 15 | Badminton |
| 16 | Musculation |
| 17 | Calisthenics |
| 18 | Elliptique |
| 19 | Pilates |
| 20 | Basketball |
| 21 | Football |
| 22 | Rugby |
| 23 | Volleyball |
| 24 | Waterpolo |
| 25 | Equitation |
| 26 | Golf |
| 27 | Yoga |
| 28 | Danse |
| 29 | Boxe |
| 30 | Escrime |
| 31 | Gymnastique |
| 32 | Voile |
| 33 | Aviron |
| 34 | Plongee |
| 35 | Ski alpin |
| 36 | Autre |
| 128 | Pas d'activite |
| 187 | Rameur |
| 188 | Zumba |
| 190 | Cricket |
| 191 | Baseball |
| 192 | Handball |
| 193 | Hockey |
| 195 | Escalade |
| 196 | Tapis de course |
| 197 | Velo d'interieur |
| 198 | Elliptique (interieur) |
| 200 | Marche nordique |
| 260 | Ski de fond |
| 261 | Snowboard |
| 262 | Raquettes |
| 263 | Patinage sur glace |
| 264 | Roller |
| 266 | HIIT |
| 272 | Stretching |
| 307 | Corde a sauter |

Tout ID non reconnu retourne `"Autre"`.

---

## 5. Deserialization tolerante

L'API Withings n'est pas strictement typee dans toutes ses reponses. Plusieurs deserialiseurs custom sont implementes dans `sources/withings/types/deserializers.rs` pour gerer les inconsistances.

### `deserialize_more` -- champ `more` (bool ou int)

Le champ `more` indique s'il y a d'autres pages de resultats. Withings retourne parfois un booleen (`true`/`false`), parfois un entier (`0`/`1`). Le deserialiseur accepte les deux formats via un enum `untagged` :

```rust
enum MoreValue {
    Int(i64),
    Bool(bool),
}
```

`true` est converti en `1`, `false` en `0`.

**Utilise par** : `MeasureBody`, `ActivityBody`, `SleepBody`, `WorkoutBody`, `HeartListBody`.

### `deserialize_lenient_i64` / `deserialize_lenient_f64` -- champs sommeil

Certains champs des donnees de sommeil peuvent etre retournes comme des strings au lieu de nombres. Ces deserialiseurs parsent la valeur JSON brute et n'extraient que les valeurs numeriques, en ignorant silencieusement les strings ou autres types inattendus.

```rust
let val = Option::<serde_json::Value>::deserialize(deserializer)?;
Ok(val.and_then(|v| match v {
    serde_json::Value::Number(n) => n.as_i64(), // ou as_f64
    _ => None,
}))
```

**Utilise par** : tous les champs de `SleepData` (22 champs), incluant `deepsleepduration`, `sleep_score`, `apnea_hypopnea_index`, etc.

### `deserialize_heart_rate` -- champ `heart_rate` (int ou struct)

Le champ `heart_rate` dans les enregistrements cardiaques peut etre :
- Un **objet** avec `value` et `qrs_duration`
- Un **nombre** simple (juste la frequence)

Le deserialiseur gere les deux cas :

```rust
serde_json::Value::Object(map) => {
    HeartHr { value: map["value"], qrs_duration: map["qrs_duration"] }
}
serde_json::Value::Number(n) => {
    HeartHr { value: n, qrs_duration: None }
}
```

**Utilise par** : `HeartEntry.heart_rate`.

---

## 6. Pagination

Chaque endpoint paginable possede un champ `more` et un champ `offset` dans la reponse.

### Mesures corporelles (`getmeas`)

- **Critere de continuation** : `more == Some(1)` **ET** `offset == Some(valeur)`
- **Parametre envoye** : `offset` (valeur directe de l'API)
- **Piege** : si `more == 1` mais `offset == None`, la boucle sort pour eviter une boucle infinie

### Activite / Sommeil / Entrainements

- **Critere de continuation** : `more == Some(1)` **ET** `batch_size > 0`
- **Parametre envoye** : `offset` incremente de `batch_size` a chaque iteration
- **Securite** : la condition `batch_size > 0` previent les boucles infinies quand l'API retourne `more = 1` avec un batch vide

### Coeur / ECG

- Meme logique que Activite/Sommeil : `more == Some(1)` + `batch_size > 0`
- Difference cle : les parametres temporels sont des **timestamps Unix** (`startdate`, `enddate`) et non des dates YMD

### Appareils

- **Pas de pagination** : un seul appel retourne tous les appareils

---

## 7. Mapping appareils

La fonction `friendly_device_name` (`sources/withings/mapping/devices.rs`) traduit les `model_id` Withings en noms lisibles :

### Balances

| model_id | Nom |
|---|---|
| 4 | Smart Body Analyzer |
| 5 | Body+ |
| 6 | Body Cardio |
| 7 | Body |
| 9 | Body Pro |
| 10 | Body Scan |

### Tensiometres

| model_id | Nom |
|---|---|
| 41 | Tensiometre iOS |
| 42 | Tensiometre sans fil |
| 43 | BPM |
| 44 | BPM Core |
| 45 | BPM Connect |
| 46 | BPM Connect Pro |

### Trackers et montres

| model_id | Nom |
|---|---|
| 51 | Pulse Ox |
| 52 | Activite |
| 53 | Activite Pop / Steel |
| 54 | Go |
| 55 | Steel HR |
| 58 | Pulse HR |
| 59 | Steel HR Sport |
| 90 | Move |
| 91 | Move ECG |
| 93 | ScanWatch |

### Sommeil et sante

| model_id | Nom |
|---|---|
| 60 | Aura Dock |
| 61 | Sleep Analyzer |
| 62 | Sleep Analyzer V2 |
| 63 | Sleep Analyzer |
| 70 | Thermo |

Si le `model_id` n'est pas reconnu, le champ `model` brut de l'API est utilise en fallback.

---

## 8. Strategie UPSERT

Toutes les donnees synchronisees utilisent des `INSERT ... ON CONFLICT ... DO UPDATE` pour garantir l'idempotence :

| Table | Cle de conflit |
|---|---|
| `body_measures` | `withings_grpid` |
| `blood_pressure` | `withings_grpid` |
| `vitals` | `withings_grpid, vital_type` |
| `activity` | `date, source` |
| `sleep_records` | `date, source` |
| `workouts` | `date, withings_id` (et `source, external_id` pour les autres sources) |
| `heart_records` | `timestamp, source` |
| `withings_devices` | `device_id` |

---

## 9. Gestion des erreurs de synchronisation

La synchronisation Withings (`WithingsSource::sync`, appelee par `sync_sources`) execute six etapes independamment (Mesures, Activite, Sommeil, Entrainements, Coeur, Appareils) ; chacune est inscrite dans `sync_log` avec sa duree et son nombre de lignes. Si un module echoue, l'erreur est capturee dans le champ `errors` du `SyncResult` et les autres modules continuent. Cela garantit qu'un probleme ponctuel sur un endpoint (ex: Heart API) n'empeche pas la synchronisation des autres donnees.

Le client et la synchro rendent `Result<_, AppError>` (R26) : un jeton refuse apres rafraichissement est `Unauthorized`, un code 601 ou un HTTP 429 `RateLimit` (y compris quand il epuise les trois reessais), un code 503 ou une reponse illisible `Internal`, un transport en echec `Http`. `log_sync` masque l'erreur par `AppError::redacted` avant de l'inscrire dans `sync_log`. Les messages de `SyncResult.errors` restent des textes.

Le `SyncResult` retourne les compteurs de chaque type :

```rust
pub struct SyncResult {
    pub body_measures_count: usize,
    pub blood_pressure_count: usize,
    pub sleep_count: usize,
    pub activity_count: usize,
    pub vitals_count: usize,
    pub workouts_count: usize,
    pub heart_count: usize,
    pub devices_count: usize,
    pub errors: Vec<String>,
}
```

---

## Evenements et stockage

### Rappel OAuth : trois evenements Tauri

Le serveur de rappel emet pour Withings :

| Evenement | Quand | Payload |
|---|---|---|
| `withings-auth-success` | Tokens echanges et sauves | `()` |
| `withings-auth-error` | Echec exchange ou save tokens | `{ kind, message, detail }` (`AppError` masque par `into_event_payload`) |
| `withings-auth-timeout` | Pas de callback recu en 120s | `()` |

Le frontend (assistant de premier lancement, carte de la source dans Parametres) ecoute ces trois evenements (`useOAuthFlow`, `useSourceEvents`) pour afficher un retour visible (indicateur, message d'erreur, bouton Reessayer).

### Une seule ecriture par mesure (R28)

La synchro ecrit chaque mesure dans **une seule** table : la table historique de sa categorie
(`body_measures_legacy`, `blood_pressure`, `vitals_legacy`, `sleep_records`, `activity`). La double
ecriture dans `metrics` a disparu en R28. Seule exception : un groupe de mesures qui porte un pouls
(type 11) sans systolique et diastolique n'a pas de ligne `blood_pressure` ; son pouls va dans
`metrics` (`upsert_pulse_alone`), et `read_metric_series` le relit (source `Split`). Voir
[`metrics-v2.md`](./metrics-v2.md).

### Identifiants de l'application Withings

Le `client_id` et le `client_secret` de l'application Withings de l'utilisateur sont saisis dans Parametres > Sources, ou par la commande `source_set_credentials(source, clientId, clientSecret)`. Ils sont chiffres dans `source_credentials` ; `withings_credentials` n'est qu'une vue (migration 019).

Voir [`docs/metrics-v2.md`](./metrics-v2.md) pour le detail Stack V2.
