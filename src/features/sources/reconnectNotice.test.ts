import { describe, it, expect } from "vitest";
import {
  isReconnectRequired,
  reconnectReason,
  reconnectSteps,
  reconnectTitle,
  type SourceStates,
} from "./reconnectNotice";
import type { InvalidatedSecrets } from "./types";

const entry = (source: string, label: string, tokens: boolean, credentials: boolean) => ({
  source,
  label,
  tokens,
  credentials,
});
const withings = (tokens: boolean, credentials: boolean) =>
  entry("withings", "Withings", tokens, credentials);
const BOTH: InvalidatedSecrets = { sources: [withings(true, true)] };
const TOKENS_ONLY: InvalidatedSecrets = { sources: [withings(true, false)] };
const NONE: InvalidatedSecrets = { sources: [] };
const OURA_AND_WITHINGS: InvalidatedSecrets = {
  sources: [withings(true, false), entry("oura", "Oura", true, true)],
};

const state = (connected: boolean, credentialsConfigured: boolean) => ({
  connected,
  credentialsConfigured,
});
const states = (withingsState: ReturnType<typeof state>, oura = state(false, false)) =>
  ({ withings: withingsState, oura }) satisfies SourceStates;

describe("reconnectNotice", () => {
  it("n'exige rien quand aucun secret n'a été invalidé ou que la réponse est absente", () => {
    expect(isReconnectRequired(NONE)).toBe(false);
    expect(isReconnectRequired(null)).toBe(false);
    expect(isReconnectRequired(undefined)).toBe(false);
    expect(reconnectSteps(NONE, states(state(false, false)))).toEqual([]);
  });

  it("demande la ressaisie des identifiants puis la connexion quand tout a été invalidé", () => {
    const steps = reconnectSteps(BOTH, states(state(false, false)));
    expect(steps).toHaveLength(2);
    expect(steps[0]).toMatch(/Withings.*Client ID/);
    expect(steps[1]).toMatch(/Withings.*Connecter/);
  });

  it("ne demande que la connexion quand seuls les jetons ont été invalidés", () => {
    const steps = reconnectSteps(TOKENS_ONLY, states(state(false, true)));
    expect(steps).toHaveLength(1);
    expect(steps[0]).toMatch(/Connecter/);
  });

  it("retire chaque étape dès qu'elle est accomplie", () => {
    expect(reconnectSteps(BOTH, states(state(false, true)))).toHaveLength(1);
    expect(reconnectSteps(BOTH, states(state(true, true)))).toEqual([]);
  });

  it("nomme chaque source touchée dans le titre et le motif", () => {
    expect(reconnectTitle(BOTH)).toBe("Reconnexion Withings nécessaire");
    expect(reconnectTitle(OURA_AND_WITHINGS)).toBe("Reconnexion Withings, Oura nécessaire");
    expect(reconnectReason(OURA_AND_WITHINGS)).toMatch(/pour Withings, Oura/);
    expect(reconnectReason(BOTH)).toMatch(/mesures santé ne sont pas touchées/);
  });

  it("suit l'état de chaque source : Withings reconnecté ne retire pas l'étape d'Oura", () => {
    const steps = reconnectSteps(OURA_AND_WITHINGS, states(state(true, true), state(false, false)));
    expect(steps).toHaveLength(2);
    expect(steps.every((s) => s.startsWith("Oura"))).toBe(true);
  });

  it("retire l'étape d'une source dès qu'elle est reconnectée (N-62)", () => {
    const steps = reconnectSteps(OURA_AND_WITHINGS, states(state(true, true), state(true, true)));
    expect(steps).toEqual([]);
  });

  it("garde une étape pour une source dont l'état n'est pas encore connu", () => {
    const steps = reconnectSteps(OURA_AND_WITHINGS, { withings: state(true, true) });
    expect(steps.some((s) => s.startsWith("Oura"))).toBe(true);
  });
});
