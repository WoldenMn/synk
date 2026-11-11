import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import type { CsvConfig, CsvDateFormat } from "@/features/sources/types";

interface Props {
  idPrefix: string;
  header: string[];
  formats: CsvDateFormat[];
  config: CsvConfig;
  onChange: (next: Partial<CsvConfig>) => void;
}

export function CsvFormatFields({ idPrefix, header, formats, config, onChange: patch }: Props) {
  return (
    <div className="grid grid-cols-1 gap-3 min-[640px]:grid-cols-2 xl:grid-cols-4">
      <div className="space-y-1">
        <Label htmlFor={`${idPrefix}-date`}>Colonne de la date</Label>
        <Select
          id={`${idPrefix}-date`}
          value={config.date_column}
          onChange={(e) => patch({ date_column: e.target.value })}
        >
          {header.map((h, i) => (
            <option key={`${i}-${h}`} value={h}>
              {h}
            </option>
          ))}
        </Select>
      </div>
      <div className="space-y-1">
        <Label htmlFor={`${idPrefix}-time`}>Colonne de l&apos;heure</Label>
        <Select
          id={`${idPrefix}-time`}
          value={config.time_column ?? ""}
          onChange={(e) => patch({ time_column: e.target.value || null })}
        >
          <option value="">Aucune (heure dans la date)</option>
          {header
            .filter((h) => h !== config.date_column)
            .map((h, i) => (
              <option key={`${i}-${h}`} value={h}>
                {h}
              </option>
            ))}
        </Select>
      </div>
      <div className="space-y-1">
        <Label htmlFor={`${idPrefix}-format`}>Forme de la date</Label>
        <Select
          id={`${idPrefix}-format`}
          value={config.date_format}
          onChange={(e) => patch({ date_format: e.target.value })}
        >
          {formats.map((f) => (
            <option key={f.id} value={f.id}>
              {f.label} ({f.example})
            </option>
          ))}
        </Select>
      </div>
      <div className="space-y-1">
        <Label htmlFor={`${idPrefix}-decimal`}>Séparateur des décimales</Label>
        <Select
          id={`${idPrefix}-decimal`}
          value={config.decimal}
          onChange={(e) => patch({ decimal: e.target.value })}
        >
          <option value=".">Point (72.4)</option>
          <option value=",">Virgule (72,4)</option>
        </Select>
      </div>
    </div>
  );
}
