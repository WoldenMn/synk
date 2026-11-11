import { TrendingUp, TrendingDown, Minus } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { formatNumber } from "@/lib/formatters";
import type { MetricOption } from "../helpers";

interface Correlation {
  r: number;
  direction: string;
  strength: string;
}

interface Props {
  m1: MetricOption;
  m2: MetricOption;
  correlation: Correlation;
  dataPoints: number;
}

export function CorrelationStats({ m1, m2, correlation, dataPoints }: Props) {
  return (
    <div className="grid gap-4 sm:grid-cols-3">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Direction</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            {correlation.direction === "positive" && (
              <TrendingUp className="h-5 w-5 text-success-text" />
            )}
            {correlation.direction === "negative" && (
              <TrendingDown className="h-5 w-5 text-destructive-text" />
            )}
            {(correlation.direction === "aucune" || correlation.direction === "insuffisant") && (
              <Minus className="h-5 w-5 text-muted-foreground" />
            )}
            <span className="text-lg font-semibold capitalize">
              {correlation.direction === "positive" && "Corrélation positive"}
              {correlation.direction === "negative" && "Corrélation négative"}
              {correlation.direction === "aucune" && "Pas de corrélation"}
              {correlation.direction === "insuffisant" && "Données insuffisantes"}
            </span>
          </div>
          {correlation.direction !== "insuffisant" && (
            <p className="mt-1 text-sm text-muted-foreground">
              {correlation.direction === "positive"
                ? `Quand ${m1.label} augmente, ${m2.label} tend a augmenter aussi`
                : correlation.direction === "negative"
                  ? `Quand ${m1.label} augmente, ${m2.label} tend a diminuer`
                  : `Pas de lien apparent entre ${m1.label} et ${m2.label}`}
            </p>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            Force (r de Pearson)
          </CardTitle>
        </CardHeader>
        <CardContent>
          <span className="text-2xl font-bold">
            {correlation.direction === "insuffisant" ? "—" : formatNumber(correlation.r, 3)}
          </span>
          {correlation.direction !== "insuffisant" && (
            <p className="mt-1 text-sm text-muted-foreground capitalize">
              Corrélation {correlation.strength}
            </p>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            Points de données
          </CardTitle>
        </CardHeader>
        <CardContent>
          <span className="text-2xl font-bold">{dataPoints}</span>
          <p className="mt-1 text-sm text-muted-foreground">jours avec les deux métriques</p>
        </CardContent>
      </Card>
    </div>
  );
}
