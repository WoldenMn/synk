import { Select } from "@/components/ui/select";
import {
  columnMetric,
  columnUnit,
  setColumnMetric,
  setColumnUnit,
} from "@/features/sources/csvConfig";
import type { CsvConfig, CsvTarget } from "@/features/sources/types";

interface Props {
  idPrefix: string;
  header: string[];
  config: CsvConfig;
  targets: CsvTarget[];
  onChange: (next: CsvConfig) => void;
}

export function CsvMappingTable({ idPrefix, header, config, targets, onChange }: Props) {
  const skipped = new Set([config.date_column, config.time_column ?? ""]);
  const columns = header.filter((name) => name && !skipped.has(name));

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <caption className="sr-only">
          Correspondance entre les colonnes du fichier et les mesures
        </caption>
        <thead>
          <tr className="text-left text-xs text-muted-foreground">
            <th scope="col" className="py-1 pr-3 font-medium">
              Colonne du fichier
            </th>
            <th scope="col" className="py-1 pr-3 font-medium">
              Mesure
            </th>
            <th scope="col" className="py-1 font-medium">
              Unité dans le fichier
            </th>
          </tr>
        </thead>
        <tbody>
          {columns.map((name, index) => {
            const metric = columnMetric(config, name);
            const target = targets.find((t) => t.metric === metric);
            return (
              <tr key={`${index}-${name}`} className="border-t">
                <th scope="row" className="py-2 pr-3 text-left font-normal">
                  {name}
                </th>
                <td className="py-2 pr-3">
                  <Select
                    id={`${idPrefix}-metric-${index}`}
                    aria-label={`Mesure de la colonne ${name}`}
                    value={metric}
                    onChange={(e) =>
                      onChange(setColumnMetric(config, name, e.target.value, targets))
                    }
                  >
                    <option value="">Ignorer cette colonne</option>
                    {targets.map((t) => (
                      <option key={t.metric} value={t.metric}>
                        {t.label}
                      </option>
                    ))}
                  </Select>
                </td>
                <td className="py-2">
                  {target ? (
                    <Select
                      id={`${idPrefix}-unit-${index}`}
                      aria-label={`Unité de la colonne ${name}`}
                      value={columnUnit(config, name)}
                      onChange={(e) => onChange(setColumnUnit(config, name, e.target.value))}
                    >
                      {target.units.map((u) => (
                        <option key={u.id} value={u.id}>
                          {u.label}
                        </option>
                      ))}
                    </Select>
                  ) : (
                    <span className="text-muted-foreground">-</span>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
