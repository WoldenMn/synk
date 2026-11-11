import { Sparkles, TriangleAlert, Info, CircleAlert, RefreshCw, Check } from "lucide-react";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { LoadingSkeleton, EmptyState } from "@/components/shared";
import { useInsights, useGenerateInsights, useMarkInsightsSeen } from "@/features/insights/hooks";
import { describeInsightReport } from "@/features/insights/report";
import { formatDate } from "@/lib/formatters";
import { toast } from "sonner";
import { reportError } from "@/lib/reportError";

interface SeverityStyle {
  label: string;
  bg: string;
  border: string;
  icon: typeof Info;
  iconColor: string;
}

const INFO_STYLE: SeverityStyle = {
  label: "Information",
  bg: "bg-primary/5",
  border: "border-primary/30",
  icon: Info,
  iconColor: "text-primary-text",
};

const SEVERITY_STYLES: Record<string, SeverityStyle> = {
  info: INFO_STYLE,
  warning: {
    label: "Avertissement",
    bg: "bg-warning/5",
    border: "border-warning/30",
    icon: TriangleAlert,
    iconColor: "text-warning-text",
  },
  danger: {
    label: "Critique",
    bg: "bg-destructive/5",
    border: "border-destructive/30",
    icon: CircleAlert,
    iconColor: "text-destructive-text",
  },
};

export default function Insights() {
  const { data: insights = [], isLoading } = useInsights();
  const generate = useGenerateInsights();
  const markSeen = useMarkInsightsSeen();

  const unseenCount = insights.filter((i) => !i.seen).length;

  async function handleGenerate() {
    try {
      const { success, failure } = describeInsightReport(await generate.mutateAsync());
      toast.success(success);
      if (failure) {
        reportError(new Error(failure), { source: "manual", context: "generate_insights" });
      }
    } catch (e) {
      reportError(e, { source: "mutation", context: "generate_insights" });
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between flex-wrap gap-4">
        <div>
          <p className="text-muted-foreground">
            Analyses automatiques de vos données de santé
            {unseenCount > 0 && (
              <span className="ml-2 text-primary-text font-medium">
                ({unseenCount} nouveau{unseenCount > 1 ? "x" : ""})
              </span>
            )}
          </p>
        </div>
        <div className="flex gap-2">
          {unseenCount > 0 && (
            <Button variant="outline" size="sm" onClick={() => markSeen.mutate()}>
              <Check className="mr-2 h-4 w-4" />
              Tout marquer lu
            </Button>
          )}
          <Button onClick={handleGenerate} disabled={generate.isPending}>
            <RefreshCw className={`mr-2 h-4 w-4 ${generate.isPending ? "animate-spin" : ""}`} />
            Analyser
          </Button>
        </div>
      </div>

      {isLoading ? (
        <LoadingSkeleton />
      ) : insights.length === 0 ? (
        <EmptyState
          icon={<Sparkles className="h-12 w-12" />}
          title="Aucun constat pour l'instant"
          description="Lancez une analyse pour détecter les tendances et les anomalies de vos données."
          action={
            <Button onClick={handleGenerate}>
              <Sparkles className="mr-2 h-4 w-4" />
              Lancer l'analyse
            </Button>
          }
        />
      ) : (
        <div className="grid gap-3">
          {insights.map((insight, idx) => {
            const style = SEVERITY_STYLES[insight.severity] ?? INFO_STYLE;
            const Icon = style.icon;
            return (
              <div
                key={insight.id}
                className="animate-fade-in-up"
                style={{ animationDelay: `${Math.min(idx * 30, 300)}ms` }}
              >
                <Card
                  className={`${style.bg} ${style.border} border-2 ${!insight.seen ? "ring-2 ring-primary/20" : ""}`}
                >
                  <CardContent className="pt-6">
                    <div className="flex items-start gap-4">
                      <div className={`rounded-full p-2 ${style.bg} ${style.iconColor}`}>
                        <Icon className="h-5 w-5" />
                        <span className="sr-only">{style.label}</span>
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center justify-between gap-2 mb-1">
                          <h2 className="font-semibold text-base">{insight.title}</h2>
                          {!insight.seen && (
                            <span className="text-xs uppercase tracking-wider bg-primary text-primary-foreground px-2 py-0.5 rounded-full">
                              Nouveau
                            </span>
                          )}
                        </div>
                        <p className="text-sm text-muted-foreground">{insight.message}</p>
                        <div className="flex items-center gap-3 mt-2 text-xs text-muted-foreground">
                          <span>
                            <span className="capitalize">{insight.kind}</span>
                            {insight.metric && `, ${insight.metric}`},{" "}
                            {formatDate(insight.generated_at)}
                          </span>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </div>
            );
          })}
        </div>
      )}

      <Card>
        <CardHeader>
          <h2 className="text-base font-semibold leading-none tracking-tight">
            Comment fonctionnent les insights ?
          </h2>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground space-y-2">
          <p>
            <strong>Tendances</strong> : variations significatives sur 7 ou 30 jours (poids,
            activité, sommeil).
          </p>
          <p>
            <strong>Anomalies</strong> : valeurs détectées comme aberrantes via Z-score (FC élevée,
            sommeil court, tension).
          </p>
          <p>
            L'analyse tourne en local sur votre ordinateur. Aucune donnée n'est envoyée à un serveur
            externe.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
