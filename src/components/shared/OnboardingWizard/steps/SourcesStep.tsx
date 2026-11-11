import { useState } from "react";
import { Activity, RotateCcw, SkipForward } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useSources } from "@/features/sources/hooks";
import { domainLabel } from "@/features/sources/labels";
import type { SourceInfo } from "@/features/sources/types";
import { SourceConnect } from "./SourceConnect";

interface SourcesStepProps {
  onComplete: () => void;
  onSkip: () => void;
  titleId: string;
}

function SourceRow({ source, onConfigure }: { source: SourceInfo; onConfigure: () => void }) {
  return (
    <li className="flex items-center justify-between gap-3 rounded-md border p-3 text-left">
      <div className="min-w-0">
        <p className="text-sm font-medium">{source.label}</p>
        <p className="truncate text-xs text-muted-foreground">
          {source.domains.map(domainLabel).join(", ")}
        </p>
      </div>
      {source.connected ? (
        <Badge className="bg-success/10 text-success-text border-success/20">Connecté</Badge>
      ) : (
        <Button variant="outline" size="sm" onClick={onConfigure}>
          <span className="sr-only">Configurer {source.label}</span>
          <span aria-hidden="true">Configurer</span>
        </Button>
      )}
    </li>
  );
}

export function SourcesStep({ onComplete, onSkip, titleId }: SourcesStepProps) {
  const { data: sources, isError, refetch } = useSources();
  const [activeId, setActiveId] = useState<string | null>(null);
  const active = sources?.find((s) => s.id === activeId && s.kind === "oauth");
  const anyConnected = sources?.some((s) => s.connected) ?? false;

  return (
    <div className="text-center space-y-6">
      <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-primary/10">
        <Activity className="h-8 w-8 text-primary-text" />
      </div>
      <div className="space-y-2">
        <h2 id={titleId} className="text-xl font-bold">
          Choisir ses sources
        </h2>
        <p className="text-sm text-muted-foreground">
          Reliez les comptes dont Synk doit récupérer les données. Vous pouvez en ajouter plus tard
          dans Paramètres, ou n'en brancher aucune.
        </p>
      </div>
      {isError && (
        <div
          role="alert"
          className="flex flex-col items-start gap-3 rounded-md border border-destructive/30 bg-destructive/5 p-4 text-left"
        >
          <p className="text-sm text-destructive-text">Lecture des sources impossible.</p>
          <Button variant="outline" onClick={() => refetch()}>
            <RotateCcw className="mr-2 h-4 w-4" />
            Réessayer
          </Button>
        </div>
      )}
      {active ? (
        <SourceConnect
          source={active}
          onConnected={() => setActiveId(null)}
          onBack={() => setActiveId(null)}
        />
      ) : (
        <div className="space-y-3">
          <ul className="space-y-2">
            {sources
              ?.filter((s) => s.kind === "oauth")
              .map((s) => (
                <SourceRow key={s.id} source={s} onConfigure={() => setActiveId(s.id)} />
              ))}
          </ul>
          {anyConnected && (
            <Button className="w-full" size="lg" onClick={onComplete}>
              Continuer
            </Button>
          )}
          <Button variant="ghost" className="w-full" onClick={onSkip}>
            <SkipForward className="mr-2 h-4 w-4" />
            Passer cette étape
          </Button>
        </div>
      )}
    </div>
  );
}
