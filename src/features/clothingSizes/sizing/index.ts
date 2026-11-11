import type { ClothingInputs, SizeEstimate } from "../types";
import {
  JACKET_BANDS,
  SHIRT_BANDS,
  TOP_BANDS,
  PANTS_WAIST_BANDS,
  SHOE_CONVERSIONS,
  type SizeBand,
} from "./tables";
import type { ResolveOpts } from "./opts";

interface BandRef {
  maxValue: number;
}

export interface ResolvedBand<B extends BandRef = BandRef> {
  idx: number;
  band: B;
  near: { idx: number; band: B; direction: "down" | "up" } | null;
}

// Cœur générique : choisit l'index de bande et détecte la proximité d'un seuil.
// Partagé entre l'estimation générique (SizeBand) et l'estimation par marque (BrandBand).
export function resolveBandIndex<B extends BandRef>(
  value: number | null,
  bands: B[],
  opts: ResolveOpts
): ResolvedBand<B> | null {
  if (value == null || Number.isNaN(value) || value < opts.min || value > opts.max) {
    return null;
  }
  let idx = bands.findIndex((b) => value <= b.maxValue);
  if (idx === -1) idx = bands.length - 1;
  const band = bands[idx];
  if (band === undefined) return null;
  const upper = bands[idx + 1];
  const lower = bands[idx - 1];

  let near: ResolvedBand<B>["near"] = null;
  if (upper !== undefined && band.maxValue - value <= opts.margin) {
    near = { idx: idx + 1, band: upper, direction: "up" };
  } else if (lower !== undefined && value - lower.maxValue <= opts.margin) {
    near = { idx: idx - 1, band: lower, direction: "down" };
  }
  return { idx, band, near };
}

function resolveBand(
  value: number | null,
  bands: SizeBand[],
  opts: ResolveOpts
): SizeEstimate | null {
  const res = resolveBandIndex(value, bands, opts);
  if (!res) return null;
  const est: SizeEstimate = { eu: res.band.eu, us: res.band.us, uk: res.band.uk };
  if (res.near) {
    est.nearThreshold = { size: res.near.band.eu, direction: res.near.direction };
  }
  return est;
}

export function estimateJacket(chestCm: number | null): SizeEstimate | null {
  return resolveBand(chestCm, JACKET_BANDS, { min: 70, max: 140, margin: 1 });
}

export function estimateTop(chestCm: number | null): SizeEstimate | null {
  return resolveBand(chestCm, TOP_BANDS, { min: 70, max: 140, margin: 2 });
}

export function estimateShirt(neckCm: number | null): SizeEstimate | null {
  return resolveBand(neckCm, SHIRT_BANDS, { min: 33, max: 50, margin: 0.5 });
}

export function estimatePants(
  waistCm: number | null,
  inseamCm?: number | null
): SizeEstimate | null {
  const base = resolveBand(waistCm, PANTS_WAIST_BANDS, { min: 55, max: 140, margin: 1.5 });
  if (!base) return null;
  if (inseamCm != null && inseamCm >= 60 && inseamCm <= 100) {
    const l = Math.round(inseamCm / 2.54);
    base.us = `${base.us} L${l}`;
    base.uk = base.us;
  }
  return base;
}

export interface ComputedSizes {
  top: SizeEstimate | null;
  shirt: SizeEstimate | null;
  jacket: SizeEstimate | null;
  pants: SizeEstimate | null;
}

export function computeSizes(inputs: ClothingInputs): ComputedSizes {
  return {
    top: estimateTop(inputs.chest),
    shirt: estimateShirt(inputs.neck),
    jacket: estimateJacket(inputs.chest),
    pants: estimatePants(inputs.waist, inputs.inseam),
  };
}

export function shoeConversion(eu: string): { us: string; uk: string } | undefined {
  return SHOE_CONVERSIONS[eu];
}
