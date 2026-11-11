import { sourceColumn } from "@/features/sources/columns";
import { formatDuration, formatNumber, formatDate } from "@/lib/formatters";
import { avg } from "@/lib/stats";
import type { SleepRecord } from "@/features/sleep/types";

export function buildDetailedMetrics(records: SleepRecord[]) {
  const avgDeep = avg(records, "duration_deep");
  const avgLight = avg(records, "duration_light");
  const avgRem = avg(records, "duration_rem");
  const avgWaso = avg(records, "waso");
  const avgTimeinbed = avg(records, "total_timeinbed");
  const avgToSleep = avg(records, "duration_to_sleep");
  const avgBreathing = avg(records, "breathing_disturbances_intensity");
  const avgRemEpisodes = avg(records, "nb_rem_episodes");
  const avgOutOfBed = avg(records, "out_of_bed_count");
  const avgNightEvents = avg(records, "night_events");

  return [
    { label: "Sommeil profond", value: avgDeep != null ? formatDuration(avgDeep) : "\u2014" },
    { label: "Sommeil léger", value: avgLight != null ? formatDuration(avgLight) : "\u2014" },
    { label: "Sommeil REM", value: avgRem != null ? formatDuration(avgRem) : "\u2014" },
    { label: "WASO", value: avgWaso != null ? formatDuration(avgWaso) : "\u2014" },
    {
      label: "Temps au lit",
      value: avgTimeinbed != null ? formatDuration(avgTimeinbed) : "\u2014",
    },
    { label: "Endormissement", value: avgToSleep != null ? formatDuration(avgToSleep) : "\u2014" },
    {
      label: "Troubles resp.",
      value: avgBreathing != null ? formatNumber(avgBreathing, 1) : "\u2014",
    },
    {
      label: "Épisodes REM",
      value: avgRemEpisodes != null ? formatNumber(Math.round(avgRemEpisodes), 0) : "\u2014",
    },
    {
      label: "Sorties de lit",
      value: avgOutOfBed != null ? formatNumber(Math.round(avgOutOfBed), 0) : "\u2014",
    },
    {
      label: "Événements",
      value: avgNightEvents != null ? formatNumber(Math.round(avgNightEvents), 0) : "\u2014",
    },
  ];
}

export const sleepColumns = [
  {
    key: "date",
    label: "Date",
    render: (row: SleepRecord) => formatDate(row.date),
    searchText: (row: SleepRecord) => formatDate(row.date),
  },
  {
    key: "duration_total",
    label: "Durée",
    render: (row: SleepRecord) =>
      row.duration_total != null ? formatDuration(row.duration_total) : "\u2014",
    searchText: (row: SleepRecord) =>
      row.duration_total != null ? formatDuration(row.duration_total) : "",
  },
  {
    key: "sleep_score",
    label: "Score",
    render: (row: SleepRecord) => (row.sleep_score != null ? `${row.sleep_score}/100` : "\u2014"),
    searchText: (row: SleepRecord) => (row.sleep_score != null ? `${row.sleep_score}/100` : ""),
  },
  sourceColumn<SleepRecord>(),
];
