export const SOURCE_HELP: Record<string, string[]> = {
  oura: [
    "Connectez-vous au portail développeur d'Oura, ouvrez « My Applications » et créez une application.",
    "Déclarez l'URL de rappel ci-dessous à l'identique : Oura refuse toute autre adresse.",
    "Cochez les portées daily, spo2 et workout, pas davantage.",
    "Une application Oura accepte dix utilisateurs sans validation, ce qui suffit pour un usage personnel.",
  ],
  polar: [
    "Connectez-vous à admin.polaraccesslink.com avec votre compte Polar Flow et créez un client (« Create client »).",
    "Déclarez l'URL de rappel ci-dessous à l'identique dans les URL de redirection du client : Polar refuse toute autre adresse.",
    "Acceptez tous les consentements de données sur account.polar.com, sinon Polar refuse l'enregistrement de l'utilisateur.",
    "Après la connexion, synchronisez au moins une fois par mois : Polar ne garde que 28 jours de sommeil et 30 jours d'entraînements.",
  ],
  whoop: [
    "Connectez-vous à developer-dashboard.whoop.com avec votre compte Whoop, créez une équipe puis une application.",
    "Cochez les portées read:recovery, read:sleep, read:cycles et read:workout, et laissez l'accès hors ligne (offline) activé : sans lui Whoop ne délivre pas de jeton de rafraîchissement.",
    "Déclarez l'URL de rappel ci-dessous à l'identique : Whoop refuse toute autre adresse. Si le tableau de bord n'accepte pas une adresse http, signalez-le avant d'aller plus loin.",
    "Whoop limite l'application à 100 requêtes par minute et 10 000 par jour : une première synchro s'étale toute seule, et si le quota du jour est atteint Synk s'arrête et reprend à l'heure indiquée.",
  ],
  google_health: [
    "Dans Google Cloud, créez un projet puis activez Google Health API (bibliothèque d'API, lien ci-dessus). Google Health ne propose ni bac à sable ni données d'exemple : tout vient de votre compte réel.",
    "Écran de consentement : type Externe, statut Testing, et ajoutez votre propre adresse Google comme utilisateur de test (cent au plus, sans vérification d'application).",
    "Accès aux données : ajoutez les trois portées en lecture seule, sleep.readonly, activity_and_fitness.readonly et health_metrics_and_measurements.readonly, pas davantage.",
    "Clients : créez un client de type Application Web et déclarez l'URL de rappel ci-dessous à l'identique dans les URI de redirection autorisés. Copiez l'ID client et le secret ici.",
    "Votre compte Google doit être lié à Google Health : ouvrez l'application mobile Google Health avec ce compte, ou migrez votre compte Fitbit vers Google. Sans cela Google répond « compte non lié ».",
    "Tant que le projet reste en mode Testing, Google coupe l'autorisation tous les 7 jours : Synk l'annonce la veille, il suffit alors de cliquer sur Connecter. Publier le projet (En production, écran « application non vérifiée ») supprime cette échéance.",
  ],
};

export function sourceHelp(id: string): string[] {
  return SOURCE_HELP[id] ?? [];
}
