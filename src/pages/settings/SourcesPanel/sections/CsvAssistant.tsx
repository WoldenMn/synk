import { useEffect, useRef, useState } from "react";
import { Save, Trash2, X } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { importIssue } from "@/features/sources/csvConfig";
import type { CsvConfig, CsvInspection } from "@/features/sources/types";
import { CsvFormatFields } from "./CsvFormatFields";
import { CsvMappingTable } from "./CsvMappingTable";

interface Props {
  idPrefix: string;
  inspection: CsvInspection;
  onImport: (config: CsvConfig) => void;
  onSave: (name: string, config: CsvConfig) => void;
  onForget: (id: number) => void;
  onClose: () => void;
}

const DELIMITER_NAMES: Record<string, string> = {
  ",": "virgule",
  ";": "point-virgule",
  "\t": "tabulation",
};

export function CsvAssistant({ idPrefix, inspection, onImport, onSave, onForget, onClose }: Props) {
  const [config, setConfig] = useState<CsvConfig>(inspection.suggestion);
  const [name, setName] = useState("");
  const ref = useRef<HTMLElement>(null);
  const titleRef = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    ref.current?.scrollIntoView?.({ block: "nearest" });
    titleRef.current?.focus();
  }, []);
  const issue = importIssue(config);
  const nameMissing = name.trim() === "";
  const nameHelpId = `${idPrefix}-name-help`;
  const patch = (next: Partial<CsvConfig>) => setConfig((c) => ({ ...c, ...next }));

  return (
    <section
      ref={ref}
      aria-label="Assistant de correspondance CSV"
      className="space-y-4 rounded-lg border bg-muted/20 p-4"
    >
      <h4 ref={titleRef} tabIndex={-1} className="text-sm font-semibold outline-none">
        Correspondance du fichier CSV
      </h4>
      <div className="flex flex-wrap items-center gap-2 text-sm">
        <span>
          {inspection.row_count} lignes, colonnes séparées par{" "}
          {DELIMITER_NAMES[config.delimiter] ?? config.delimiter}
        </span>
        {inspection.preset && (
          <Badge variant="secondary">Modèle reconnu : {inspection.preset}</Badge>
        )}
      </div>

      {inspection.saved.length > 0 && (
        <div className="space-y-2">
          <p className="text-sm font-medium">Correspondances enregistrées pour ces colonnes</p>
          <ul className="space-y-1">
            {inspection.saved.map((m) => (
              <li key={m.id} className="flex flex-wrap items-center gap-2 text-sm">
                <span className="font-medium">{m.name}</span>
                <Button variant="outline" size="sm" onClick={() => setConfig(m.config)}>
                  Appliquer
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  aria-label={`Oublier la correspondance ${m.name}`}
                  onClick={() => onForget(m.id)}
                >
                  <Trash2 className="h-4 w-4" aria-hidden="true" />
                </Button>
              </li>
            ))}
          </ul>
        </div>
      )}

      {inspection.unreadable_mappings > 0 && (
        <p className="text-sm text-warning-text">
          {inspection.unreadable_mappings === 1
            ? "1 correspondance enregistrée est illisible et ignorée."
            : `${inspection.unreadable_mappings} correspondances enregistrées sont illisibles et ignorées.`}
        </p>
      )}

      <CsvFormatFields
        idPrefix={idPrefix}
        header={inspection.header}
        formats={inspection.formats}
        config={config}
        onChange={patch}
      />

      <CsvMappingTable
        idPrefix={idPrefix}
        header={inspection.header}
        config={config}
        targets={inspection.targets}
        onChange={setConfig}
      />

      <details className="text-sm">
        <summary className="cursor-pointer text-muted-foreground">
          Aperçu des {inspection.sample.length} premières lignes
        </summary>
        <div className="mt-2 overflow-x-auto">
          <table className="w-full text-xs">
            <caption className="sr-only">Aperçu du fichier</caption>
            <thead>
              <tr>
                {inspection.header.map((h, j) => (
                  <th key={`${j}-${h}`} scope="col" className="py-1 pr-3 text-left font-medium">
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {inspection.sample.map((row, i) => (
                <tr key={i} className="border-t">
                  {inspection.header.map((h, j) => (
                    <td key={`${j}-${h}`} className="py-1 pr-3">
                      {row[j] ?? ""}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </details>

      {issue && (
        <p role="alert" className="text-sm text-destructive-text">
          {issue}
        </p>
      )}

      <div className="flex flex-wrap items-end gap-3">
        <div className="space-y-1">
          <Label htmlFor={`${idPrefix}-name`}>Nom de cette correspondance</Label>
          <Input
            id={`${idPrefix}-name`}
            value={name}
            maxLength={80}
            placeholder="Ma balance"
            aria-describedby={nameMissing ? nameHelpId : undefined}
            onChange={(e) => setName(e.target.value)}
          />
        </div>
        <Button
          variant="outline"
          disabled={Boolean(issue) || nameMissing}
          onClick={() => onSave(name, config)}
        >
          <Save className="mr-2 h-4 w-4" aria-hidden="true" />
          Enregistrer
        </Button>
        <Button disabled={Boolean(issue)} onClick={() => onImport(config)}>
          Importer le fichier
        </Button>
        <Button variant="ghost" onClick={onClose}>
          <X className="mr-2 h-4 w-4" aria-hidden="true" />
          Fermer
        </Button>
      </div>
      {nameMissing && (
        <p id={nameHelpId} className="text-xs text-muted-foreground">
          Donnez un nom à cette correspondance pour l'enregistrer.
        </p>
      )}
    </section>
  );
}
