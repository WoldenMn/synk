import type { ReactNode } from "react";
import { Activity as ActivityIcon, Heart, Moon, Scale } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { LoadingSkeleton } from "@/components/shared";
import { PeriodEmptyState } from "@/components/shared/PeriodEmptyState";
import { useGoals } from "@/features/goals/hooks";
import { useAnySourceConnected } from "@/features/sources/hooks";
import { historyHint } from "@/features/sources/hint";
import { WeightChart } from "@/components/charts/WeightChart";
import { BloodPressureChart } from "@/components/charts/BloodPressureChart";
import { SleepChart } from "@/components/charts/SleepChart";
import { ActivityChart } from "@/components/charts/ActivityChart";
import { useEntranceOnce } from "@/hooks/useEntranceOnce";
import type { TimeRangeValue } from "@/lib/timeRange";
import type { BodyMeasure } from "@/features/bodyMeasures/types";
import type { BloodPressure } from "@/features/bloodPressure/types";
import type { SleepRecord } from "@/features/sleep/types";
import type { Activity } from "@/features/activity/types";

interface Props {
  bodyMeasures: BodyMeasure[] | undefined;
  bpData: BloodPressure[] | undefined;
  sleepData: SleepRecord[] | undefined;
  activityData: Activity[] | undefined;
  isLoading: boolean;
  timeRange: TimeRangeValue;
  onShowAll: () => void;
}

interface TrendCardProps {
  title: string;
  delay: string;
  animate: boolean;
  isLoading: boolean;
  hasData: boolean;
  chart: ReactNode;
  empty: ReactNode;
}

function TrendCard({ title, delay, animate, isLoading, hasData, chart, empty }: TrendCardProps) {
  return (
    <Card
      className={animate ? "animate-fade-in-up" : undefined}
      style={animate ? { animationDelay: delay } : undefined}
    >
      <CardHeader>
        <CardTitle>{title}</CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? <LoadingSkeleton rows={1} height="h-[300px]" /> : hasData ? chart : empty}
      </CardContent>
    </Card>
  );
}

export function TrendCharts({
  bodyMeasures,
  bpData,
  sleepData,
  activityData,
  isLoading,
  timeRange,
  onShowAll,
}: Props) {
  const animateEntrance = useEntranceOnce("dashboard-trends");
  const { data: goals } = useGoals();
  const { data: connected } = useAnySourceConnected();
  const hint = historyHint(connected, "ou ajoutez une mesure à la main");

  return (
    <>
      <div className="grid gap-4 md:grid-cols-2">
        <TrendCard
          title="Évolution du poids"
          delay="360ms"
          animate={animateEntrance}
          isLoading={isLoading}
          hasData={!!bodyMeasures && bodyMeasures.length > 0}
          chart={
            <WeightChart
              data={bodyMeasures ?? []}
              goalWeight={goals?.find((g) => g.metric_key === "weight")?.target_value}
            />
          }
          empty={
            <PeriodEmptyState
              compact
              icon={Scale}
              periodTitle="Aucune pesée sur cette période"
              noneTitle="Aucune pesée enregistrée"
              noneDescription={hint}
              timeRange={timeRange}
              onShowAll={onShowAll}
            />
          }
        />
        <TrendCard
          title="Tension artérielle"
          delay="420ms"
          animate={animateEntrance}
          isLoading={isLoading}
          hasData={!!bpData && bpData.length > 0}
          chart={<BloodPressureChart data={bpData ?? []} />}
          empty={
            <PeriodEmptyState
              compact
              icon={Heart}
              periodTitle="Aucune mesure de tension sur cette période"
              noneTitle="Aucune mesure de tension enregistrée"
              noneDescription={hint}
              timeRange={timeRange}
              onShowAll={onShowAll}
            />
          }
        />
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <TrendCard
          title="Sommeil récent"
          delay="480ms"
          animate={animateEntrance}
          isLoading={isLoading}
          hasData={!!sleepData && sleepData.length > 0}
          chart={<SleepChart data={sleepData ?? []} />}
          empty={
            <PeriodEmptyState
              compact
              icon={Moon}
              periodTitle="Aucune nuit sur cette période"
              noneTitle="Aucune nuit enregistrée"
              noneDescription={hint}
              timeRange={timeRange}
              onShowAll={onShowAll}
            />
          }
        />
        <TrendCard
          title="Activité quotidienne"
          delay="540ms"
          animate={animateEntrance}
          isLoading={isLoading}
          hasData={!!activityData && activityData.length > 0}
          chart={<ActivityChart data={activityData ?? []} />}
          empty={
            <PeriodEmptyState
              compact
              icon={ActivityIcon}
              periodTitle="Aucune activité sur cette période"
              noneTitle="Aucune activité enregistrée"
              noneDescription={hint}
              timeRange={timeRange}
              onShowAll={onShowAll}
            />
          }
        />
      </div>
    </>
  );
}
