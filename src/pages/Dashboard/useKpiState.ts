import { useMemo } from "react";
import { useGoals } from "@/features/goals/hooks";
import { useDashboardCards } from "@/features/dashboard/hooks";
import type { BodyMeasure } from "@/features/bodyMeasures/types";
import type { BloodPressure } from "@/features/bloodPressure/types";
import type { SleepRecord } from "@/features/sleep/types";
import type { Activity } from "@/features/activity/types";
import type { TrendData } from "@/features/dashboard/types";
import { buildCards, computeDeltas } from "./helpers";

export interface KpiInput {
  from: string;
  to: string;
  bodyMeasures: BodyMeasure[] | undefined;
  bpData: BloodPressure[] | undefined;
  sleepData: SleepRecord[] | undefined;
  activityData: Activity[] | undefined;
  cmpBody: BodyMeasure[] | undefined;
  cmpBP: BloodPressure[] | undefined;
  cmpSleep: SleepRecord[] | undefined;
  cmpActivity: Activity[] | undefined;
  compareActive: boolean;
}

export type TrendEntry = { trend: TrendData | undefined; metric: string };

export function useKpiState(input: KpiInput) {
  const { from, to, bodyMeasures, bpData, sleepData, activityData } = input;
  const { cmpBody, cmpBP, cmpSleep, cmpActivity, compareActive } = input;
  const { data: goals } = useGoals();
  const { data: dashboardCards } = useDashboardCards(from, to);
  const latestWeight = dashboardCards?.body;
  const latestBP = dashboardCards?.blood_pressure;
  const latestSleep = dashboardCards?.sleep;
  const latestActivity = dashboardCards?.activity;
  const latestWorkout = dashboardCards?.workout;
  const latestHeart = dashboardCards?.heart;
  const weightTrend = dashboardCards?.trends.weight;
  const systolicTrend = dashboardCards?.trends.systolic;
  const sleepTrend = dashboardCards?.trends.sleep_score;
  const stepsTrend = dashboardCards?.trends.steps;

  const deltas = useMemo(
    () =>
      computeDeltas({
        bodyMeasures,
        bpData,
        sleepData,
        activityData,
        cmpBody,
        cmpBP,
        cmpSleep,
        cmpActivity,
        compareActive,
      }),
    [
      bodyMeasures,
      bpData,
      sleepData,
      activityData,
      cmpBody,
      cmpBP,
      cmpSleep,
      cmpActivity,
      compareActive,
    ]
  );

  const trends = useMemo<Record<string, TrendEntry>>(
    () => ({
      weight: { trend: weightTrend, metric: "weight" },
      bp: { trend: systolicTrend, metric: "systolic" },
      sleep: { trend: sleepTrend, metric: "sleep_score" },
      activity: { trend: stepsTrend, metric: "steps" },
    }),
    [weightTrend, systolicTrend, sleepTrend, stepsTrend]
  );

  const cards = useMemo(
    () =>
      buildCards({
        latestWeight,
        latestBP,
        latestSleep,
        latestActivity,
        latestWorkout,
        latestHeart,
        bodyMeasures,
        bpData,
        sleepData,
        activityData,
      }),
    [
      latestWeight,
      latestBP,
      latestSleep,
      latestActivity,
      latestWorkout,
      latestHeart,
      bodyMeasures,
      bpData,
      sleepData,
      activityData,
    ]
  );

  return { goals, deltas, trends, cards };
}
