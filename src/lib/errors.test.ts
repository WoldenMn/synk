import { describe, it, expect } from "vitest";
import { errorDetail, errorText, formatError, isRustError, type RustErrorKind } from "./errors";

describe("formatError", () => {
  it("classifie 401/unauthorized comme AUTH", () => {
    const err = formatError("401 unauthorized");
    expect(err.code).toBe("AUTH");
    expect(err.userMessage).toMatch(/Reconnectez la source concernée/);
  });

  it("classifie 403 forbidden comme AUTH", () => {
    expect(formatError("403 forbidden").code).toBe("AUTH");
  });

  it("classifie 429/rate.limit comme RATE_LIMIT", () => {
    expect(formatError("429 rate limit exceeded").code).toBe("RATE_LIMIT");
    expect(formatError("rate-limit hit").code).toBe("RATE_LIMIT");
  });

  it("classifie network/connection/reqwest comme NETWORK", () => {
    expect(formatError("network error").code).toBe("NETWORK");
    expect(formatError("connection refused").code).toBe("NETWORK");
    expect(formatError("reqwest::Error sending request").code).toBe("NETWORK");
  });

  it("classifie timeout comme TIMEOUT (mais doit passer apres NETWORK pour reqwest)", () => {
    expect(formatError("operation timed out").code).toBe("TIMEOUT");
  });

  it("classifie sqlx/sqlite/database/locked comme DATABASE", () => {
    expect(formatError("sqlx error: connection lost").code).toBe("DATABASE");
    expect(formatError("database is locked").code).toBe("DATABASE");
  });

  it("classifie invalid/missing/required comme VALIDATION", () => {
    expect(formatError("invalid input").code).toBe("VALIDATION");
    expect(formatError("missing field 'name'").code).toBe("VALIDATION");
  });

  it("retourne UNKNOWN pour message non reconnu", () => {
    const err = formatError("quelque chose d'inattendu");
    expect(err.code).toBe("UNKNOWN");
    expect(err.userMessage).toBe("quelque chose d'inattendu");
  });

  it("tronque userMessage sur UNKNOWN si > 200 chars", () => {
    const huge = "x".repeat(250);
    const err = formatError(huge);
    expect(err.code).toBe("UNKNOWN");
    expect(err.userMessage).toBe("Une erreur inattendue est survenue.");
  });

  it("extrait message depuis Error object", () => {
    const e = new Error("database is locked");
    const err = formatError(e);
    expect(err.code).toBe("DATABASE");
    expect(err.message).toBe("database is locked");
  });

  it("extrait message depuis objet avec champ message", () => {
    const e = { message: "401 unauthorized", code: 401 };
    expect(formatError(e).code).toBe("AUTH");
  });

  it("convertit primitif non-string en string", () => {
    expect(formatError(404).code).toBe("UNKNOWN");
    expect(formatError(null).code).toBe("UNKNOWN");
  });
});

describe("messages utilisateur des erreurs typées", () => {
  const typed = (kind: RustErrorKind, detail: string) =>
    formatError({ kind, message: `Erreur : ${detail}`, detail });

  it("Db garde sa phrase d'intégrité", () => {
    expect(typed("Db", "RowNotFound").userMessage).toContain("base de données locale");
  });

  it("Validation insère le détail", () => {
    expect(typed("Validation", "Systolique doit être dans [0, 300]").userMessage).toContain(
      "Systolique doit être dans [0, 300]"
    );
  });

  it("Http parle de connexion, Unauthorized de reconnexion, RateLimit de requêtes", () => {
    expect(typed("Http", "timeout").userMessage).toContain("connexion");
    expect(typed("Unauthorized", "token revoked").userMessage).toContain("Reconnectez");
    expect(typed("RateLimit", "429").userMessage).toContain("requêtes");
  });

  it("Unauthorized dit la même chose qu'un 401 reconnu par expression régulière", () => {
    expect(typed("Unauthorized", "x").userMessage).toBe(formatError("HTTP 401: x").userMessage);
  });

  it("un texte préfixé à l'ancienne n'est plus un protocole", () => {
    const e = formatError("Erreur reseau : echec");
    expect(e.code).toBe("UNKNOWN");
    expect(e.userMessage).toBe("Erreur reseau : echec");
  });
});

describe("expressions régulières pour les chaînes des plugins et les erreurs JavaScript", () => {
  it("variante regex 401 => AUTH", () => {
    expect(formatError("HTTP 401: unauthorized").code).toBe("AUTH");
  });
});

const CODE_BY_KIND: Record<RustErrorKind, string> = {
  Db: "DATABASE",
  Validation: "VALIDATION",
  NotFound: "NOT_FOUND",
  Json: "VALIDATION",
  Forbidden: "FORBIDDEN",
  Internal: "INTERNAL",
  Http: "NETWORK",
  Unauthorized: "AUTH",
  RateLimit: "RATE_LIMIT",
  Crypto: "CRYPTO",
};

describe("formatError - erreur Rust typée { kind, message, detail }", () => {
  it.each(Object.entries(CODE_BY_KIND))("kind %s donne le code %s", (kind, code) => {
    const e = formatError({ kind, message: "Erreur x : d", detail: "d" });
    expect(e.code).toBe(code);
    expect(e.message).toBe("Erreur x : d");
    expect(e.userMessage.length).toBeGreaterThan(0);
  });

  it("lit le kind d'une erreur Rust sérialisée", () => {
    const e = formatError({
      kind: "RateLimit",
      message: "Limite atteinte : Withings code 601: rate limit",
      detail: "Withings code 601: rate limit",
    });
    expect(e.code).toBe("RATE_LIMIT");
  });

  it("rend le détail d'une erreur de validation à l'utilisateur", () => {
    const e = formatError({
      kind: "Validation",
      message: "Erreur de validation : valeur hors bornes",
      detail: "valeur hors bornes",
    });
    expect(e.code).toBe("VALIDATION");
    expect(e.userMessage).toBe("Données invalides : valeur hors bornes");
  });

  it("rend le détail d'un refus au lieu d'une phrase unique (N-15)", () => {
    const e = formatError({
      kind: "Forbidden",
      message: "Acces refuse : type de mesure #3 integre a Synk",
      detail: "type de mesure #3 integre a Synk",
    });
    expect(e.userMessage).toContain("type de mesure #3 integre a Synk");
    expect(e.userMessage).not.toMatch(/synchronis/);
  });

  it("un kind inconnu retombe sur le message", () => {
    const e = formatError({ kind: "Nouveau", message: "quelque chose", detail: "d" });
    expect(e.code).toBe("UNKNOWN");
    expect(e.userMessage).toBe("quelque chose");
  });
});

describe("isRustError", () => {
  it("accepte l'objet complet", () => {
    expect(isRustError({ kind: "Db", message: "m", detail: "d" })).toBe(true);
  });

  it.each([null, undefined, "texte", 3, {}, { kind: 3, message: "m" }, { kind: "Db" }])(
    "refuse %j",
    (v) => {
      expect(isRustError(v)).toBe(false);
    }
  );
});

describe("errorText", () => {
  it("lit le message d'une erreur typée, jamais [object Object]", () => {
    expect(errorText({ kind: "Db", message: "Erreur base de donnees : x", detail: "x" })).toBe(
      "Erreur base de donnees : x"
    );
  });

  it("garde le texte d'une chaîne et d'un Error", () => {
    expect(errorText("brut")).toBe("brut");
    expect(errorText(new Error("boum"))).toBe("boum");
  });
});

describe("errorDetail", () => {
  it("rend le détail d'une erreur typée, sans le préfixe du type", () => {
    expect(
      errorDetail({
        kind: "Internal",
        message: "Erreur interne : disque plein",
        detail: "disque plein",
      })
    ).toBe("disque plein");
  });

  it("garde le texte d'une chaîne ou d'un Error", () => {
    expect(errorDetail("brut")).toBe("brut");
    expect(errorDetail(new Error("boum"))).toBe("boum");
  });
});
