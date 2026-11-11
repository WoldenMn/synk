import { beforeEach, describe, expect, it } from "vitest";
import {
  ONBOARDING_KEY,
  isOnboardingDone,
  markOnboardingDone,
  readOnboardingDone,
} from "./onboarding";

describe("assistant de premier lancement", () => {
  beforeEach(() => localStorage.clear());

  it("reconnait la valeur ecrite par les versions installees", () => {
    localStorage.setItem(ONBOARDING_KEY, "true");
    expect(readOnboardingDone()).toBe(true);
    expect(localStorage.getItem(ONBOARDING_KEY)).toBe("true");
  });

  it("reconnait la valeur entre guillemets", () => {
    localStorage.setItem(ONBOARDING_KEY, '"true"');
    expect(readOnboardingDone()).toBe(true);
  });

  it("ne le croit pas fini sans valeur ou avec une valeur etrangere", () => {
    expect(readOnboardingDone()).toBe(false);
    localStorage.setItem(ONBOARDING_KEY, "false");
    expect(readOnboardingDone()).toBe(false);
    localStorage.setItem(ONBOARDING_KEY, "42");
    expect(readOnboardingDone()).toBe(false);
  });

  it("se relit apres ecriture, sans changer le texte stocke", () => {
    markOnboardingDone();
    expect(localStorage.getItem(ONBOARDING_KEY)).toBe("true");
    expect(readOnboardingDone()).toBe(true);
  });

  it("valide les deux formes", () => {
    expect(isOnboardingDone(true)).toBe(true);
    expect(isOnboardingDone("true")).toBe(true);
    expect(isOnboardingDone(false)).toBe(false);
    expect(isOnboardingDone(null)).toBe(false);
  });
});
