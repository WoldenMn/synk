import { useEffect, useRef, useState } from "react";
import { ArrowDown, ArrowUp } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useMetricPriorities, useSetMetricPriority } from "@/features/sources/hooks";
import { sourceLabel } from "@/features/sources/labels";
import type { MetricPriority } from "@/features/sources/types";

const INERT = "aria-disabled:pointer-events-none aria-disabled:opacity-50";

type Direction = -1 | 1;

function MetricRow({
  metric,
  onMoved,
}: {
  metric: MetricPriority;
  onMoved: (message: string) => void;
}) {
  const save = useSetMetricPriority();
  const buttons = useRef(new Map<string, HTMLButtonElement>());
  const refocus = useRef<string | null>(null);
  const order = metric.sources.join(",");
  const last = metric.sources.length - 1;

  useEffect(() => {
    if (refocus.current === null) return;
    buttons.current.get(refocus.current)?.focus();
    refocus.current = null;
  }, [order]);

  const move = (index: number, by: Direction) => {
    const target = index + by;
    if (target < 0 || target > last || save.isPending) return;
    const next = [...metric.sources];
    const [moved] = next.splice(index, 1);
    const name = sourceLabel(moved as string);
    next.splice(target, 0, moved as string);
    refocus.current = `${moved}:${by}`;
    save.mutate(
      { slug: metric.slug, sources: next },
      {
        onSuccess: () =>
          onMoved(`${name}, position ${target + 1} sur ${next.length} pour ${metric.label}`),
      }
    );
  };

  const arrow = (source: string, index: number, by: Direction, blocked: boolean) => {
    const name = sourceLabel(source);
    const Icon = by === -1 ? ArrowUp : ArrowDown;
    return (
      <Button
        ref={(el) => {
          if (el) buttons.current.set(`${source}:${by}`, el);
          else buttons.current.delete(`${source}:${by}`);
        }}
        variant="ghost"
        size="sm"
        className={INERT}
        aria-disabled={blocked || save.isPending}
        aria-label={`${by === -1 ? "Monter" : "Descendre"} ${name} pour ${metric.label}`}
        onClick={() => move(index, by)}
      >
        <Icon className="h-4 w-4" aria-hidden="true" />
      </Button>
    );
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle as="h3" className="text-base">
          {metric.label}
        </CardTitle>
        <CardDescription>
          {metric.custom
            ? "Ordre choisi par vous."
            : "Ordre automatique : la source la plus récente."}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <ol className="space-y-2">
          {metric.sources.map((source, index) => (
            <li
              key={source}
              className="flex items-center justify-between rounded-lg border px-3 py-2 text-sm"
            >
              <span>
                {index + 1}. {sourceLabel(source)}
              </span>
              <span className="flex gap-1">
                {arrow(source, index, -1, index === 0)}
                {arrow(source, index, 1, index === last)}
              </span>
            </li>
          ))}
        </ol>
        {metric.custom && (
          <Button
            variant="outline"
            size="sm"
            disabled={save.isPending}
            aria-label={`Rétablir l'ordre automatique de ${metric.label}`}
            onClick={() => save.mutate({ slug: metric.slug, sources: [] })}
          >
            Rétablir l'ordre automatique
          </Button>
        )}
      </CardContent>
    </Card>
  );
}

export function PrioritiesTab() {
  const { data: metrics, isLoading, isError, refetch } = useMetricPriorities();
  const [announcement, setAnnouncement] = useState("");

  if (isLoading) {
    return (
      <p role="status" className="text-sm text-muted-foreground">
        Chargement des priorités...
      </p>
    );
  }
  if (isError) {
    return (
      <div role="alert" className="space-y-2 text-sm">
        <p className="text-destructive-text">
          Impossible de lire les priorités des sources. Vos réglages ne sont pas perdus.
        </p>
        <Button variant="outline" size="sm" onClick={() => refetch()}>
          Réessayer
        </Button>
      </div>
    );
  }
  if (!metrics || metrics.length === 0) {
    return (
      <p className="text-sm text-muted-foreground">
        Aucune mesure n'a encore plusieurs sources. Quand deux sources fourniront la même mesure,
        vous choisirez ici celle que Synk affiche en premier.
      </p>
    );
  }
  return (
    <>
      <p role="status" className="sr-only">
        {announcement}
      </p>
      {metrics.map((metric) => (
        <MetricRow key={metric.slug} metric={metric} onMoved={setAnnouncement} />
      ))}
    </>
  );
}
