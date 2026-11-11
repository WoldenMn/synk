import type { LucideIcon } from "lucide-react";
import { Scale, Heart, Moon, Footprints, Dumbbell, HeartPulse } from "lucide-react";
import { formatWeight, formatBP, formatNumber, formatDuration, formatDate } from "@/lib/formatters";
import type { BodyMeasure } from "@/features/bodyMeasures/types";
import type { BloodPressure } from "@/features/bloodPressure/types";
import type { SleepRecord } from "@/features/sleep/types";
import type { Activity } from "@/features/activity/types";
import type { Workout } from "@/features/workouts/types";
import type { HeartRecord } from "@/features/heart/types";
import type { TrendData } from "@/features/dashboard/types";

const GOOD_DIRECTIONS: Record<string, string> = {
  weight: "down",
  systolic: "down",
  sleep_score: "up",
  steps: "up",
};

export function TrendBadge({ trend, metric }: { trend: TrendData | undefined; metric: string }) {
  if (!trend || trend.percent_change == null) return null;
  const pct = Math.abs(trend.percent_change);
  const arrow = trend.direction === "up" ? "↑" : trend.direction === "down" ? "↓" : "";
  const goodDir = GOOD_DIRECTIONS[metric];
  let colorClass = "text-muted-foreground";
  if (goodDir) {
    colorClass = trend.direction === goodDir ? "text-success-text" : "text-destructive-text";
  }
  return (
    <span className={`text-xs font-medium ${colorClass}`}>
      {arrow} {pct.toFixed(1)}%
    </span>
  );
}

export interface DeltaResult {
  current: number;
  compare: number;
  pct: number;
}

export function computeDelta(
  current: number | null | undefined,
  compare: number | null | undefined
): DeltaResult | null {
  if (current == null || compare == null || compare === 0) return null;
  const pct = ((current - compare) / Math.abs(compare)) * 100;
  return { current, compare, pct };
}

function avgField<T>(
  arr: T[] | undefined,
  getter: (d: T) => number | null | undefined
): number | null {
  if (!arr?.length) return null;
  const valid = arr.map(getter).filter((v): v is number => v != null);
  if (valid.length === 0) return null;
  return valid.reduce((s, v) => s + v, 0) / valid.length;
}

export interface DeltasInput {
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

export function computeDeltas(input: DeltasInput): Record<string, DeltaResult | null> {
  const { bodyMeasures, bpData, sleepData, activityData, compareActive } = input;
  const { cmpBody, cmpBP, cmpSleep, cmpActivity } = input;
  const curW = avgField(bodyMeasures, (d) => d.weight);
  const cmpW = compareActive ? avgField(cmpBody, (d) => d.weight) : null;
  const curS = avgField(bpData, (d) => d.systolic);
  const cmpS = compareActive ? avgField(cmpBP, (d) => d.systolic) : null;
  const curSl = avgField(sleepData, (d) => d.sleep_score);
  const cmpSl = compareActive ? avgField(cmpSleep, (d) => d.sleep_score) : null;
  const curSt = avgField(activityData, (d) => d.steps);
  const cmpSt = compareActive ? avgField(cmpActivity, (d) => d.steps) : null;

  return {
    weight: computeDelta(curW, cmpW),
    bp: computeDelta(curS, cmpS),
    sleep: computeDelta(curSl, cmpSl),
    activity: computeDelta(curSt, cmpSt),
  };
}

// Tables indexees par l'identifiant de carte (CardConfig.id), jamais par son
// titre : un libelle corrige (un accent ajoute) faisait disparaitre en silence
// la tendance et l'objectif de la carte.
export const DELTA_UNITS: Record<string, string> = {
  weight: "kg",
  bp: "mmHg",
  sleep: "/100",
  activity: "pas",
};

export const GOAL_METRICS: Record<string, string> = {
  weight: "weight",
  activity: "steps",
  sleep: "sleep_score",
};

export const DIRECTION_ARROWS: Record<string, string> = {
  decrease: "↓",
  increase: "↑",
  maintain: "→",
};

export type CardId = "weight" | "bp" | "sleep" | "activity" | "workout" | "heart";

export interface CardConfig {
  id: CardId;
  title: string;
  icon: LucideIcon;
  color: string;
  glowClass: string;
  route: string;
  value: string;
  sub: string;
  sparkData: number[];
  sparkColor: string;
}

export interface BuildCardsInput {
  latestWeight: BodyMeasure | null | undefined;
  latestBP: BloodPressure | null | undefined;
  latestSleep: SleepRecord | null | undefined;
  latestActivity: Activity | null | undefined;
  latestWorkout: Workout | null | undefined;
  latestHeart: HeartRecord | null | undefined;
  bodyMeasures: BodyMeasure[] | undefined;
  bpData: BloodPressure[] | undefined;
  sleepData: SleepRecord[] | undefined;
  activityData: Activity[] | undefined;
}

function ecgLabel(status: number | null | undefined): string {
  switch (status) {
    case 0:
      return "Normal";
    case 1:
      return "AFib";
    case 2:
      return "Inconclusif";
    case 3:
      return "Trop court";
    default:
      return "Aucune donnée";
  }
}

function weightCard(latest: BodyMeasure | null | undefined, series: BodyMeasure[] | undefined) {
  return {
    id: "weight" as const,
    title: "Poids",
    icon: Scale,
    color: "text-primary-text",
    glowClass: "hover:glow-indigo",
    route: "/body",
    value: latest?.weight ? formatWeight(latest.weight) : "—",
    sub: latest?.weight ? `Dernière pesée le ${formatDate(latest.measured_at)}` : "Aucune donnée",
    sparkData:
      series
        ?.slice(-7)
        .map((d) => d.weight ?? 0)
        .filter(Boolean) ?? [],
    sparkColor: "hsl(var(--series-weight))",
  };
}

function bpCard(latest: BloodPressure | null | undefined, series: BloodPressure[] | undefined) {
  return {
    id: "bp" as const,
    title: "Tension",
    icon: Heart,
    color: "text-destructive-text",
    glowClass: "hover:glow-rose",
    route: "/blood-pressure",
    value: latest ? formatBP(latest.systolic, latest.diastolic) + " mmHg" : "—",
    sub: latest ? `Dernière mesure le ${formatDate(latest.measured_at)}` : "Aucune donnée",
    sparkData: series?.slice(-7).map((d) => d.systolic) ?? [],
    sparkColor: "hsl(var(--series-systolic))",
  };
}

function sleepCard(latest: SleepRecord | null | undefined, series: SleepRecord[] | undefined) {
  return {
    id: "sleep" as const,
    title: "Sommeil",
    icon: Moon,
    color: "text-primary-text",
    glowClass: "hover:glow-violet",
    route: "/sleep",
    value: latest?.sleep_score ? `${latest.sleep_score}/100` : "—",
    sub: latest?.sleep_score ? `Score de la nuit du ${formatDate(latest.date)}` : "Aucune donnée",
    sparkData:
      series
        ?.slice(-7)
        .map((d) => d.sleep_score ?? 0)
        .filter(Boolean) ?? [],
    sparkColor: "hsl(var(--series-sleep-duration))",
  };
}

function activityCard(latest: Activity | null | undefined, series: Activity[] | undefined) {
  return {
    id: "activity" as const,
    title: "Activité",
    icon: Footprints,
    color: "text-success-text",
    glowClass: "hover:glow-emerald",
    route: "/activity",
    value: latest?.steps ? formatNumber(latest.steps, 0) : "—",
    sub: latest?.steps ? "Pas" : "Aucune donnée",
    sparkData:
      series
        ?.slice(-7)
        .map((d) => d.steps ?? 0)
        .filter(Boolean) ?? [],
    sparkColor: "hsl(var(--series-steps))",
  };
}

function workoutCard(latest: Workout | null | undefined) {
  return {
    id: "workout" as const,
    title: "Entraînement",
    icon: Dumbbell,
    color: "text-warning-text",
    glowClass: "hover:glow-amber",
    route: "/workouts",
    value: latest?.category_label
      ? `${latest.category_label}${latest.duration ? ` (${formatDuration(latest.duration)})` : ""}`
      : "—",
    sub: latest ? "Dernier entraînement" : "Aucune donnée",
    sparkData: [],
    sparkColor: "hsl(var(--series-calories))",
  };
}

function heartCard(latest: HeartRecord | null | undefined) {
  return {
    id: "heart" as const,
    title: "Cœur / ECG",
    icon: HeartPulse,
    color: "text-destructive-text",
    glowClass: "hover:glow-rose",
    route: "/heart",
    value: latest?.heart_rate ? `${latest.heart_rate} bpm` : "—",
    sub: ecgLabel(latest?.ecg_status),
    sparkData: [],
    sparkColor: "hsl(var(--series-systolic))",
  };
}

export function buildCards(input: BuildCardsInput): CardConfig[] {
  return [
    weightCard(input.latestWeight, input.bodyMeasures),
    bpCard(input.latestBP, input.bpData),
    sleepCard(input.latestSleep, input.sleepData),
    activityCard(input.latestActivity, input.activityData),
    workoutCard(input.latestWorkout),
    heartCard(input.latestHeart),
  ];
}
