import { Link } from "react-router-dom";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { MiniSparkline } from "@/components/shared";
import { formatNumber } from "@/lib/formatters";
import type { Goal } from "@/features/goals/types";
import {
  TrendBadge,
  DELTA_UNITS,
  GOAL_METRICS,
  DIRECTION_ARROWS,
  type CardConfig,
  type DeltaResult,
} from "../helpers";
import type { TrendEntry } from "../useKpiState";

function GoalLine({ cardId, goals }: { cardId: string; goals: Goal[] | undefined }) {
  const metricKey = GOAL_METRICS[cardId];
  const goal = metricKey && goals?.find((g) => g.metric_key === metricKey);
  if (!goal) return null;
  return (
    <p className="text-xs text-muted-foreground mt-0.5">
      Objectif : {DIRECTION_ARROWS[goal.direction] || ""} {goal.target_value}
    </p>
  );
}

function DeltaLine({ cardId, delta }: { cardId: string; delta: DeltaResult }) {
  const unit = DELTA_UNITS[cardId] ?? "";
  const sign = delta.pct >= 0 ? "+" : "";
  const colorClass =
    Math.abs(delta.pct) < 1
      ? "text-muted-foreground"
      : delta.pct > 0
        ? "text-success-text"
        : "text-destructive-text";
  return (
    <p className={`text-xs mt-1 ${colorClass}`}>
      vs comparaison : {formatNumber(delta.current, 1)} {unit} vs {formatNumber(delta.compare, 1)}{" "}
      {unit} ({sign}
      {delta.pct.toFixed(1)}%)
    </p>
  );
}

interface Props {
  card: CardConfig;
  index: number;
  animateEntrance: boolean;
  trend: TrendEntry | undefined;
  goals: Goal[] | undefined;
  delta: DeltaResult | null | undefined;
  compareActive: boolean;
}

export function KPICard({
  card,
  index,
  animateEntrance,
  trend,
  goals,
  delta,
  compareActive,
}: Props) {
  return (
    <Link
      to={card.route}
      className="block rounded-xl focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring"
    >
      <Card
        className={`${animateEntrance ? "animate-fade-in-up " : ""}transition-shadow duration-300 ${card.glowClass}`}
        style={animateEntrance ? { animationDelay: `${index * 60}ms` } : undefined}
      >
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">{card.title}</CardTitle>
          <card.icon className={`h-5 w-5 ${card.color}`} />
        </CardHeader>
        <CardContent>
          <div className="flex items-baseline gap-2">
            <p className="text-2xl font-bold">{card.value}</p>
            {trend && <TrendBadge trend={trend.trend} metric={trend.metric} />}
          </div>
          <p className="text-xs text-muted-foreground mt-1">{card.sub}</p>
          {card.sparkData.length >= 2 && (
            <div className="mt-2">
              <MiniSparkline data={card.sparkData} color={card.sparkColor} />
            </div>
          )}
          <GoalLine cardId={card.id} goals={goals} />
          {compareActive && delta && <DeltaLine cardId={card.id} delta={delta} />}
        </CardContent>
      </Card>
    </Link>
  );
}
