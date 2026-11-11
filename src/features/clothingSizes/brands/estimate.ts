import { resolveBandIndex } from "../sizing";
import { BRAND_RESOLVE_OPTS } from "./types";
import { BRANDS } from "./index";
import type { Brand, BrandSizeResult } from "./types";
import type { CategoryKey, ClothingInputs } from "../types";

function measureFor(category: CategoryKey, inputs: ClothingInputs): number | null {
  if (category === "shirt") return inputs.neck;
  if (category === "pants") return inputs.waist;
  if (category === "shoes") return null;
  return inputs.chest; // top, jacket
}

export function estimateForBrand(
  brandId: string,
  category: CategoryKey,
  inputs: ClothingInputs,
  catalog: Brand[] = BRANDS
): BrandSizeResult | null {
  const brand = catalog.find((b) => b.id === brandId);
  const bands = brand?.grids[category];
  if (!bands || bands.length === 0) return null;

  const value = measureFor(category, inputs);
  const res = resolveBandIndex(value, bands, BRAND_RESOLVE_OPTS[category]);
  if (!res) return null;

  const result: BrandSizeResult = { label: res.band.label };
  if (res.near) {
    result.nearThreshold = { size: res.near.band.label, direction: res.near.direction };
  }
  return result;
}
