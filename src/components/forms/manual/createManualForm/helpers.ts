import { formatISO } from "date-fns";
import { nowDate, nowDatetimeLocal, nowTime } from "@/lib/dates";
import type { FormField } from "../types";

export function buildInitialState<TInput extends Record<string, unknown>>(
  fields: FormField[],
  defaults?: Partial<TInput>
): Record<string, string> {
  const state: Record<string, string> = {};
  for (const field of fields) {
    const initial = defaults?.[field.name];
    if (initial !== undefined && initial !== null) {
      state[field.name] = String(initial);
      continue;
    }
    if (field.type === "datetime") {
      state[field.name] = nowDatetimeLocal();
    } else if (field.type === "date") {
      state[field.name] = nowDate();
    } else if (field.type === "time") {
      state[field.name] = nowTime();
    } else if (field.type === "select" && field.options[0] !== undefined) {
      state[field.name] = String(field.options[0].value);
    } else {
      state[field.name] = "";
    }
  }
  return state;
}

export function parseFieldValue(field: FormField, raw: string): unknown {
  if (field.transform) {
    return field.transform(raw);
  }
  if (raw === "") {
    return field.required ? raw : null;
  }
  if (field.type === "number") {
    const n = parseInt(raw, 10);
    return Number.isFinite(n) ? n : null;
  }
  if (field.type === "decimal") {
    const n = parseFloat(raw);
    return Number.isFinite(n) ? n : null;
  }
  if (field.type === "datetime") {
    return formatISO(new Date(raw));
  }
  if (field.type === "select") {
    const opt = field.options.find((o) => String(o.value) === raw);
    return opt ? opt.value : raw;
  }
  return raw;
}

export function validateAll(
  fields: FormField[],
  values: Record<string, string>
): Record<string, string> {
  const errors: Record<string, string> = {};
  for (const field of fields) {
    const err = validate(field, values[field.name] ?? "");
    if (err) errors[field.name] = err;
  }
  return errors;
}

export function validate(field: FormField, raw: string): string | null {
  if (field.required && raw === "") {
    return `${field.label} est requis`;
  }
  if ((field.type === "number" || field.type === "decimal") && raw !== "") {
    const n = field.type === "decimal" ? parseFloat(raw) : parseInt(raw, 10);
    if (!Number.isFinite(n)) {
      return `${field.label} doit être un nombre`;
    }
    if (typeof field.min === "number" && n < field.min) {
      return `${field.label} doit être >= ${field.min}`;
    }
    if (typeof field.max === "number" && n > field.max) {
      return `${field.label} doit être <= ${field.max}`;
    }
  }
  return null;
}
