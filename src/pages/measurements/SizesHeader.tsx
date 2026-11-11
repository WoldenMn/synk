import { Select } from "@/components/ui/select";
import { BRANDS } from "@/features/clothingSizes/brands";
import type { SizeSystem } from "@/features/clothingSizes/types";

const SYSTEMS: SizeSystem[] = ["eu", "us", "uk"];

interface Props {
  brand: (typeof BRANDS)[number] | null;
  selectedBrand: string | null;
  setSelectedBrand: (id: string | null) => void;
  sizeSystem: SizeSystem;
  setSizeSystem: (system: SizeSystem) => void;
}

export function SizesHeader({
  brand,
  selectedBrand,
  setSelectedBrand,
  sizeSystem,
  setSizeSystem,
}: Props) {
  return (
    <div className="flex items-center justify-between">
      <div>
        <h2 className="text-lg font-semibold">Tailles de vêtements</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          {brand
            ? `Tailles ${brand.label} estimées depuis vos mensurations, à titre indicatif`
            : "Estimées depuis vos mensurations, valeurs indicatives"}
        </p>
      </div>
      <div className="flex items-center gap-3">
        <Select
          value={selectedBrand ?? ""}
          onChange={(e) => setSelectedBrand(e.target.value || null)}
          aria-label="Marque"
          className="h-9 w-40"
        >
          <option value="">Générique</option>
          {BRANDS.map((b) => (
            <option key={b.id} value={b.id}>
              {b.label}
            </option>
          ))}
        </Select>
        {!brand && (
          <div className="inline-flex overflow-hidden rounded-md border">
            {SYSTEMS.map((sys) => (
              <button
                key={sys}
                type="button"
                aria-pressed={sizeSystem === sys}
                onClick={() => setSizeSystem(sys)}
                className={`px-3 py-1.5 text-xs font-medium uppercase ${
                  sizeSystem === sys
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground"
                }`}
              >
                {sys}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
