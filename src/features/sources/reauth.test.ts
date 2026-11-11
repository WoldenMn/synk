import { describe, expect, it } from "vitest";
import { reauthNotices, reauthText } from "./reauth";
import type { SourceInfo } from "./types";

const NOON = Date.UTC(2026, 10, 2, 12, 0, 0) / 1000;

function source(over: Partial<SourceInfo>): SourceInfo {
  return {
    id: "google_health",
    label: "Google Health",
    kind: "oauth",
    extensions: [],
    domains: [],
    connected: true,
    credentials_configured: true,
    last_sync_at: null,
    redirect_uri: null,
    developer_url: null,
    reauth: null,
    ...over,
  };
}

describe("reauthText", () => {
  it("annonce la date d'expiration quand l'échéance approche", () => {
    expect(reauthText("Google Health", { status: "soon", expires_at: NOON })).toBe(
      "L'autorisation de Google Health expire le 2 nov. 2026 : reconnectez-la avant cette date."
    );
  });

  it("dit que l'autorisation a expiré une fois l'échéance passée", () => {
    expect(reauthText("Google Health", { status: "expired", expires_at: NOON })).toBe(
      "L'autorisation de Google Health a expiré le 2 nov. 2026 : reconnectez-la pour reprendre la synchronisation."
    );
  });
});

describe("reauthNotices", () => {
  it("ne retient que les sources connectées qui portent une annonce", () => {
    const notices = reauthNotices([
      source({ reauth: { status: "soon", expires_at: NOON } }),
      source({ id: "oura", label: "Oura", reauth: null }),
      source({
        id: "polar",
        label: "Polar",
        connected: false,
        reauth: { status: "expired", expires_at: NOON },
      }),
    ]);

    expect(notices.map((n) => n.id)).toEqual(["google_health"]);
    expect(notices[0]?.text).toContain("expire le 2 nov. 2026");
  });

  it("rend une liste vide sans données", () => {
    expect(reauthNotices(undefined)).toEqual([]);
  });
});
