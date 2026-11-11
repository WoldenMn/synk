import { describe, expect, it } from "vitest";
import { needsOnboarding } from "./needsOnboarding";

const EMPTY = {
  db_size_bytes: 4096,
  audit_log_count: 3,
  body_measures_count: 0,
  blood_pressure_count: 0,
  sleep_count: 0,
  activity_count: 0,
  workouts_count: 0,
};

describe("needsOnboarding", () => {
  it("demande l'assistant sur une base sans mesure ni source connectée", () => {
    expect(needsOnboarding(EMPTY, [{ id: "withings", connected: false }])).toBe(true);
  });

  it("ne le demande plus dès qu'une mesure existe, le journal d'audit ne comptant pas", () => {
    expect(needsOnboarding({ ...EMPTY, sleep_count: 1 }, [])).toBe(false);
    expect(needsOnboarding({ ...EMPTY, audit_log_count: 900 }, [])).toBe(true);
  });

  it("ne le demande plus dès qu'une source est connectée", () => {
    expect(needsOnboarding(EMPTY, [{ id: "oura", connected: true }])).toBe(false);
  });

  it("refuse une réponse illisible au lieu de la prendre pour une base remplie", () => {
    expect(() => needsOnboarding(false, [])).toThrow("métriques de la base illisibles");
    expect(() => needsOnboarding({ ...EMPTY, sleep_count: Number.NaN }, [])).toThrow(
      "compteur sleep_count illisible"
    );
    expect(() => needsOnboarding(EMPTY, null)).toThrow("liste des sources illisible");
  });
});
