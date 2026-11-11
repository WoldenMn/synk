// Wrappers localStorage avec validation pour eviter les corruptions silencieuses
// (ex: synk-sync-interval invalide -> intervalle absurde, synk-onboarding-done corrompu -> wizard force).

import { logWarn } from "./logger";

export function safeGetItem<T>(key: string, validator: (v: unknown) => v is T): T | null {
  try {
    const raw = localStorage.getItem(key);
    if (raw === null) return null;
    let parsed: unknown;
    try {
      parsed = JSON.parse(raw);
    } catch {
      parsed = raw;
    }
    if (validator(parsed)) return parsed;
    localStorage.removeItem(key);
    return null;
  } catch (e) {
    logWarn("[storage] safeGetItem échoué pour", key, e);
    return null;
  }
}

export function safeSetItem(key: string, value: unknown): boolean {
  try {
    const str = typeof value === "string" ? value : JSON.stringify(value);
    localStorage.setItem(key, str);
    return true;
  } catch (e) {
    logWarn("[storage] safeSetItem échoué pour", key, e);
    return false;
  }
}

export function safeRemoveItem(key: string): void {
  try {
    localStorage.removeItem(key);
  } catch (e) {
    logWarn("[storage] safeRemoveItem échoué pour", key, e);
  }
}

export const isIsoDate = (v: unknown): v is string =>
  typeof v === "string" && !Number.isNaN(Date.parse(v));

export const isBoolean = (v: unknown): v is boolean => typeof v === "boolean";

export const isString = (v: unknown): v is string => typeof v === "string";

export const isStringArray = (v: unknown): v is string[] =>
  Array.isArray(v) && v.every((s) => typeof s === "string");
