import { useEffect, useMemo, useState } from "react";
import { LoadingSkeleton } from "@/components/shared/LoadingSkeleton";
import {
  useClothingInputs,
  useClothingSizes,
  useSaveEstimatedSizes,
} from "@/features/clothingSizes/hooks";
import { computeSizes, shoeConversion } from "@/features/clothingSizes/sizing";
import { SizeCard, type SizeBadge } from "@/features/clothingSizes/components/SizeCard";
import { SizeEditDialog } from "@/features/clothingSizes/components/SizeEditDialog";
import { BRANDS } from "@/features/clothingSizes/brands";
import { estimateForBrand } from "@/features/clothingSizes/brands/estimate";
import { SizesHeader } from "./SizesHeader";
import { useUiStore } from "@/stores/uiStore";
import type { CategoryKey, SizeEstimate, SizeSystem } from "@/features/clothingSizes/types";

interface CategoryDef {
  key: CategoryKey;
  label: string;
  sourceLabel: string;
  source: "override" | "manual";
}

const CATEGORIES: CategoryDef[] = [
  { key: "top", label: "Haut", sourceLabel: "tour de poitrine", source: "override" },
  { key: "shirt", label: "Chemise", sourceLabel: "tour de cou", source: "override" },
  { key: "jacket", label: "Veste / costume", sourceLabel: "tour de poitrine", source: "override" },
  { key: "pants", label: "Pantalon / jean", sourceLabel: "tour de taille", source: "override" },
  { key: "shoes", label: "Chaussures", sourceLabel: "pointure", source: "manual" },
];

function pick(est: SizeEstimate, system: SizeSystem): string {
  return (system === "us" ? est.us : system === "uk" ? est.uk : est.eu) ?? est.eu;
}

function secondaryConversions(est: SizeEstimate, system: SizeSystem): string | undefined {
  const parts: string[] = [];
  if (system !== "eu" && est.eu) parts.push(`EU ${est.eu}`);
  if (system !== "us" && est.us) parts.push(`US ${est.us}`);
  if (system !== "uk" && est.uk) parts.push(`UK ${est.uk}`);
  return parts.length ? parts.join(", ") : undefined;
}

interface CardModel {
  mainValue: string | null;
  badge: SizeBadge | null;
  conversions?: string;
  note?: string;
  alert?: string;
  emptyLabel: string;
}

export function SizesView() {
  const { data: inputs, isLoading } = useClothingInputs();
  const { data: sizes } = useClothingSizes();
  const sizeSystem = useUiStore((s) => s.sizeSystem);
  const setSizeSystem = useUiStore((s) => s.setSizeSystem);
  const [editing, setEditing] = useState<CategoryDef | null>(null);
  const [selectedBrand, setSelectedBrand] = useState<string | null>(null);
  const brand = selectedBrand ? (BRANDS.find((b) => b.id === selectedBrand) ?? null) : null;

  const computed: Record<CategoryKey, SizeEstimate | null> = useMemo(
    () =>
      inputs
        ? { ...computeSizes(inputs), shoes: null }
        : { top: null, shirt: null, jacket: null, pants: null, shoes: null },
    [inputs]
  );

  const saveEstimated = useSaveEstimatedSizes();

  useEffect(() => {
    if (!sizes) return;
    const entries = Object.entries(computed).flatMap(([category, est]) =>
      est ? [{ category, value: est.eu }] : []
    );
    if (entries.length > 0) saveEstimated.mutate(entries);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- on persiste quand l'estimation change, pas a chaque rendu
  }, [computed, sizes]);

  if (isLoading) return <LoadingSkeleton rows={6} height="h-24" />;

  const overrideOf = (key: string) => sizes?.find((s) => s.category === key) ?? null;

  function sourceValueFor(key: CategoryKey): number | null {
    if (!inputs) return null;
    if (key === "shirt") return inputs.neck;
    if (key === "pants") return inputs.waist;
    if (key === "shoes") return null;
    return inputs.chest;
  }

  function buildBrandModel(cat: CategoryDef, brand: (typeof BRANDS)[number]): CardModel {
    if (cat.key === "shoes") {
      return { mainValue: null, badge: null, emptyLabel: "—" };
    }
    if (!brand.grids[cat.key]) {
      return { mainValue: null, badge: null, emptyLabel: "Pas de grille pour cette marque" };
    }
    const est = inputs ? estimateForBrand(brand.id, cat.key, inputs) : null;
    if (!est) {
      return { mainValue: null, badge: null, emptyLabel: "Ajouter la mesure" };
    }
    return {
      mainValue: est.label,
      badge: "estimé",
      alert: est.nearThreshold
        ? `entre deux, le ${est.nearThreshold.size} peut aussi convenir`
        : undefined,
      note: `taille ${brand.label}`,
      emptyLabel: "Ajouter la mesure",
    };
  }

  function buildModel(cat: CategoryDef): CardModel {
    if (brand) return buildBrandModel(cat, brand);

    const override = overrideOf(cat.key);

    if (cat.key === "shoes") {
      if (!override) {
        return { mainValue: null, badge: null, emptyLabel: "Renseigner la pointure" };
      }
      const conv = shoeConversion(override.value);
      const est: SizeEstimate = { eu: override.value, us: conv?.us, uk: conv?.uk };
      return {
        mainValue: pick(est, sizeSystem),
        badge: "saisi",
        conversions: secondaryConversions(est, sizeSystem),
        note: "pointure renseignée",
        emptyLabel: "Renseigner la pointure",
      };
    }

    const est = computed[cat.key];
    if (override) {
      return {
        mainValue: override.value,
        badge: "votre taille",
        note: est ? `calcul : ${pick(est, sizeSystem)}` : undefined,
        emptyLabel: "Ajouter la mesure",
      };
    }
    if (est) {
      const sv = sourceValueFor(cat.key);
      return {
        mainValue: pick(est, sizeSystem),
        badge: "estimé",
        conversions: secondaryConversions(est, sizeSystem),
        alert: est.nearThreshold
          ? `entre deux, le ${est.nearThreshold.size} peut aussi convenir`
          : undefined,
        note: sv != null ? `d'après ${cat.sourceLabel}, ${sv} cm` : undefined,
        emptyLabel: "Ajouter la mesure",
      };
    }
    return { mainValue: null, badge: null, emptyLabel: "Ajouter la mesure" };
  }

  return (
    <div className="space-y-6">
      <SizesHeader
        brand={brand}
        selectedBrand={selectedBrand}
        setSelectedBrand={setSelectedBrand}
        sizeSystem={sizeSystem}
        setSizeSystem={setSizeSystem}
      />

      <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
        {CATEGORIES.map((cat) => {
          const m = buildModel(cat);
          return (
            <SizeCard
              key={cat.key}
              label={cat.label}
              mainValue={m.mainValue}
              badge={m.badge}
              conversions={m.conversions}
              note={m.note}
              alert={m.alert}
              emptyLabel={m.emptyLabel}
              editable={!brand}
              onEdit={() => setEditing(cat)}
            />
          );
        })}
      </div>

      {editing && (
        <SizeEditDialog
          open={!!editing}
          onOpenChange={(o) => !o && setEditing(null)}
          category={editing.key}
          label={editing.label}
          current={overrideOf(editing.key)?.value ?? null}
          source={editing.source}
        />
      )}
    </div>
  );
}
