import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { FileUp, X } from "lucide-react";
import { skippedLines, summarizeImportReport } from "@/features/sources/importSummary";
import { domainLabel } from "@/features/sources/labels";
import { importHint } from "@/features/sources/importHints";
import { useCsvImport } from "@/features/sources/useCsvImport";
import { useImportSource } from "@/features/sources/useImportSource";
import type { ImportReport, SourceInfo } from "@/features/sources/types";
import { CsvAssistant } from "./CsvAssistant";

function Progress({ percent, label }: { percent: number; label: string }) {
  return (
    <div
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={percent}
      className="h-2 w-full overflow-hidden rounded-full bg-muted"
    >
      <div className="h-full bg-primary transition-[width]" style={{ width: `${percent}%` }} />
    </div>
  );
}

interface RowViewProps {
  source: SourceInfo;
  running: boolean;
  cancelling: boolean;
  busy?: boolean;
  onStart: () => void;
  onCancel: () => void;
  progress: { percent: number; records: number } | null;
  report: ImportReport | null;
}

function RowView({
  source,
  running,
  cancelling,
  busy,
  onStart,
  onCancel,
  progress,
  report,
}: RowViewProps) {
  const hint = importHint(source.id);
  return (
    <div className="space-y-3 rounded-lg border p-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="space-y-0.5">
          <p className="font-medium">{source.label}</p>
          <p className="text-xs text-muted-foreground">
            {source.domains.map(domainLabel).join(", ")}, fichiers .{source.extensions.join(", .")}
          </p>
          {hint && <p className="max-w-3xl text-xs text-muted-foreground">{hint}</p>}
        </div>
        {running ? (
          <Button variant="outline" size="sm" onClick={onCancel} disabled={cancelling}>
            <X className="mr-2 h-4 w-4" />
            {cancelling ? "Annulation..." : "Annuler"}{" "}
            <span className="sr-only">l&apos;import {source.label}</span>
          </Button>
        ) : (
          <Button size="sm" onClick={onStart} disabled={busy}>
            <FileUp className="mr-2 h-4 w-4" />
            Choisir un fichier <span className="sr-only">{source.label}</span>
          </Button>
        )}
      </div>
      <p role="status" className="sr-only">
        {progress && progress.percent >= 10
          ? `Import ${source.label} : ${progress.percent - (progress.percent % 10)} %`
          : ""}
      </p>
      {progress && (
        <div className="space-y-1">
          <Progress percent={progress.percent} label={`Import ${source.label}`} />
          <p className="text-xs text-muted-foreground">
            {progress.percent} %, {progress.records} enregistrements écrits (rien n&apos;est gardé
            si vous annulez)
          </p>
        </div>
      )}
      <div className="space-y-1 text-sm" role="status">
        {report && <p>{summarizeImportReport(report)}</p>}
      </div>
      {report && (
        <div className="space-y-1 text-sm">
          {skippedLines(report).length > 0 && (
            <details className="text-xs text-muted-foreground">
              <summary>Ignorés ({report.skipped.length} motifs)</summary>
              <ul className="list-disc pl-5">
                {skippedLines(report).map((line) => (
                  <li key={line}>{line}</li>
                ))}
              </ul>
            </details>
          )}
        </div>
      )}
    </div>
  );
}

function ImportRow({ source }: { source: SourceInfo }) {
  const { state, start, cancel } = useImportSource(source);
  return (
    <RowView
      source={source}
      running={state.phase === "running"}
      cancelling={state.phase === "running" && state.cancelling}
      onStart={start}
      onCancel={cancel}
      progress={state.phase === "running" ? state : null}
      report={state.phase === "done" ? state.report : null}
    />
  );
}

function CsvRow({ source }: { source: SourceInfo }) {
  const { state, flow, pick, start, save, forget, close, cancel } = useCsvImport(source);
  return (
    <div className="space-y-3">
      <RowView
        source={source}
        running={state.phase === "running"}
        cancelling={state.phase === "running" && state.cancelling}
        busy={flow.step === "inspecting"}
        onStart={pick}
        onCancel={cancel}
        progress={state.phase === "running" ? state : null}
        report={state.phase === "done" ? state.report : null}
      />
      {flow.step === "assistant" && state.phase !== "running" && (
        <CsvAssistant
          key={flow.path}
          idPrefix="csv-assistant"
          inspection={flow.inspection}
          onImport={start}
          onSave={save}
          onForget={forget}
          onClose={close}
        />
      )}
    </div>
  );
}

export function ImportCard({ sources }: { sources: SourceInfo[] }) {
  if (sources.length === 0) return null;
  return (
    <Card>
      <CardHeader>
        <CardTitle as="h3">Importer un fichier</CardTitle>
        <CardDescription>
          Choisissez le fichier exporté par votre appareil ou votre application. Un import est tout
          ou rien : en cas d&apos;erreur ou d&apos;annulation, rien n&apos;est écrit. Réimporter le
          même fichier n&apos;ajoute aucun doublon.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        {sources.map((source) =>
          source.id === "csv" ? (
            <CsvRow key={source.id} source={source} />
          ) : (
            <ImportRow key={source.id} source={source} />
          )
        )}
      </CardContent>
    </Card>
  );
}
