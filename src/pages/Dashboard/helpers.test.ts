import { describe, it, expect } from "vitest";
import { buildCards, computeDeltas, DELTA_UNITS, GOAL_METRICS } from "./helpers";

const cards = buildCards({
  latestWeight: null,
  latestBP: null,
  latestSleep: null,
  latestActivity: null,
  latestWorkout: null,
  latestHeart: null,
  bodyMeasures: [],
  bpData: [],
  sleepData: [],
  activityData: [],
});
const ids = cards.map((c) => c.id);

describe("cartes du tableau de bord", () => {
  it("ont des identifiants uniques", () => {
    expect(new Set(ids).size).toBe(cards.length);
  });

  it("n'indexent leurs tables que par des identifiants de carte existants", () => {
    const deltas = computeDeltas({
      bodyMeasures: [],
      bpData: [],
      sleepData: [],
      activityData: [],
      cmpBody: [],
      cmpBP: [],
      cmpSleep: [],
      cmpActivity: [],
      compareActive: false,
    });
    for (const key of [
      ...Object.keys(DELTA_UNITS),
      ...Object.keys(GOAL_METRICS),
      ...Object.keys(deltas),
    ]) {
      expect(ids).toContain(key);
    }
  });

  it("restent indexees par identifiant quand un titre change", () => {
    const activity = cards.find((c) => c.id === "activity");
    expect(activity?.title).toBe("Activité");
    expect(DELTA_UNITS[activity!.id]).toBe("pas");
    expect(GOAL_METRICS[activity!.id]).toBe("steps");
  });
});
