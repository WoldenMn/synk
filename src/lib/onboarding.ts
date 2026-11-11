import { safeGetItem, safeSetItem } from "./storage";

export const ONBOARDING_KEY = "synk-onboarding-done";

export const isOnboardingDone = (v: unknown): v is true | "true" => v === true || v === "true";

export function readOnboardingDone(): boolean {
  return safeGetItem(ONBOARDING_KEY, isOnboardingDone) !== null;
}

export function markOnboardingDone(): void {
  safeSetItem(ONBOARDING_KEY, true);
}
