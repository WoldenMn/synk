export type ErrorCode =
  | "NETWORK"
  | "AUTH"
  | "DATABASE"
  | "VALIDATION"
  | "RATE_LIMIT"
  | "TIMEOUT"
  | "NOT_FOUND"
  | "FORBIDDEN"
  | "INTERNAL"
  | "CRYPTO"
  | "UNKNOWN";

export interface AppError {
  code: ErrorCode;
  message: string;
  userMessage: string;
}

// Ordre = priorite : les codes plus specifiques (sqlx, 401, 429) sont testes
// avant les patterns generiques (network, connection) pour eviter les faux positifs
// (ex: "sqlx error: connection lost" doit etre DATABASE, pas NETWORK).
const PATTERNS: { regex: RegExp; code: ErrorCode; userMessage: string }[] = [
  {
    regex: /sqlx|sqlite|database|locked/i,
    code: "DATABASE",
    userMessage: "Erreur de base de données. Redémarrez l'application.",
  },
  {
    regex: /401|unauthorized|invalid.{0,10}token/i,
    code: "AUTH",
    userMessage: "Session expirée. Reconnectez la source concernée depuis Paramètres.",
  },
  {
    regex: /403|forbidden/i,
    code: "AUTH",
    userMessage: "Accès refusé. Vérifiez les permissions accordées à la source.",
  },
  {
    regex: /429|rate.?limit/i,
    code: "RATE_LIMIT",
    userMessage: "Limite de requêtes atteinte. Réessayez dans quelques minutes.",
  },
  { regex: /timeout|timed?.?out/i, code: "TIMEOUT", userMessage: "Délai dépassé. Réessayez." },
  {
    regex: /network|connection|fetch|reqwest/i,
    code: "NETWORK",
    userMessage: "Erreur réseau. Vérifiez votre connexion internet.",
  },
  {
    regex: /invalid|missing|required|expected/i,
    code: "VALIDATION",
    userMessage: "Données invalides ou manquantes.",
  },
];

export type RustErrorKind =
  | "Db"
  | "Validation"
  | "NotFound"
  | "Json"
  | "Forbidden"
  | "Internal"
  | "Http"
  | "Unauthorized"
  | "RateLimit"
  | "Crypto";

export interface RustError {
  kind: RustErrorKind;
  message: string;
  detail: string;
}

export function isRustError(err: unknown): err is RustError {
  return (
    typeof err === "object" &&
    err !== null &&
    typeof (err as RustError).kind === "string" &&
    typeof (err as RustError).message === "string"
  );
}

interface Classified {
  code: ErrorCode;
  userMessage: string;
}

const BY_KIND: Record<RustErrorKind, (detail: string) => Classified> = {
  Db: () => ({
    code: "DATABASE",
    userMessage:
      "Erreur de la base de données locale. Vérifiez l'intégrité ou redémarrez l'application.",
  }),
  Validation: (detail) => ({ code: "VALIDATION", userMessage: `Données invalides : ${detail}` }),
  NotFound: () => ({
    code: "NOT_FOUND",
    userMessage: "Élément supprimé ou inexistant. Rafraîchissez la liste.",
  }),
  Json: () => ({
    code: "VALIDATION",
    userMessage: "Format de données invalide. Réessayez ou contactez le support.",
  }),
  Forbidden: (detail) => ({
    code: "FORBIDDEN",
    userMessage: `Action non autorisée : ${detail}`,
  }),
  Internal: () => ({
    code: "INTERNAL",
    userMessage: "Erreur interne. Redémarrez l'application si le problème persiste.",
  }),
  Http: () => ({
    code: "NETWORK",
    userMessage: "Problème de connexion. Vérifiez votre accès internet.",
  }),
  Unauthorized: () => ({
    code: "AUTH",
    userMessage: "Session expirée. Reconnectez la source concernée depuis Paramètres.",
  }),
  RateLimit: () => ({
    code: "RATE_LIMIT",
    userMessage: "Trop de requêtes. Réessayez dans quelques minutes.",
  }),
  Crypto: () => ({
    code: "CRYPTO",
    userMessage:
      "Le stockage chiffré est corrompu ou inaccessible. Reconnectez vos comptes de santé.",
  }),
};

export function formatError(err: unknown): AppError {
  if (isRustError(err) && err.kind in BY_KIND) {
    const { code, userMessage } = BY_KIND[err.kind](String(err.detail ?? ""));
    return { code, message: err.message, userMessage };
  }

  const message = extractMessage(err);

  for (const p of PATTERNS) {
    if (p.regex.test(message)) {
      return { code: p.code, message, userMessage: p.userMessage };
    }
  }

  return {
    code: "UNKNOWN",
    message,
    userMessage: message.length > 200 ? "Une erreur inattendue est survenue." : message,
  };
}

export function errorText(err: unknown): string {
  return formatError(err).message;
}

export function errorDetail(err: unknown): string {
  return isRustError(err) ? String(err.detail ?? err.message) : errorText(err);
}

function extractMessage(err: unknown): string {
  if (typeof err === "string") return err;
  if (err instanceof Error) return err.message;
  if (err && typeof err === "object" && "message" in err) {
    return String((err as { message: unknown }).message);
  }
  return String(err);
}
