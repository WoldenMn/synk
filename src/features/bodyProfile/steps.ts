import { CATALOG } from "./catalog";
import type { BodyMeasurement, Side, Zone } from "./types";

export type Course = "essential" | "full";

export interface StepField {
  slug: string;
  side: Side;
  min: number;
  max: number;
}

export interface WizardStep {
  id: string;
  label: string;
  guidance: string;
  zone: Zone;
  order: number;
  fields: StepField[];
}

function toField(m: BodyMeasurement): StepField {
  return { slug: m.slug, side: m.side, min: m.min, max: m.max };
}

export function buildSteps(course: Course): WizardStep[] {
  const source = course === "essential" ? CATALOG.filter((m) => m.isKey) : CATALOG;
  const steps = new Map<string, WizardStep>();

  for (const m of [...source].sort((a, b) => a.order - b.order)) {
    const id = m.pairKey ?? m.slug;
    const existing = steps.get(id);
    if (existing) {
      existing.fields.push(toField(m));
      continue;
    }
    steps.set(id, {
      id,
      label: m.stepLabel,
      guidance: m.guidance,
      zone: m.zone,
      order: m.order,
      fields: [toField(m)],
    });
  }

  for (const step of steps.values()) {
    step.fields.sort((a, b) => (a.side === "left" ? -1 : b.side === "left" ? 1 : 0));
  }

  return [...steps.values()].sort((a, b) => a.order - b.order);
}
