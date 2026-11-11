import type { TimeRange } from "./timeRange";

export const BP_ZONES = [
  {
    label: "Optimal",
    systolicMin: 0,
    systolicMax: 120,
    diastolicMin: 0,
    diastolicMax: 80,
    color: "#22c55e",
  },
  {
    label: "Normal",
    systolicMin: 120,
    systolicMax: 130,
    diastolicMin: 80,
    diastolicMax: 85,
    color: "#84cc16",
  },
  {
    label: "Normal haut",
    systolicMin: 130,
    systolicMax: 140,
    diastolicMin: 85,
    diastolicMax: 90,
    color: "#eab308",
  },
  {
    label: "Hypertension grade 1",
    systolicMin: 140,
    systolicMax: 160,
    diastolicMin: 90,
    diastolicMax: 100,
    color: "#f97316",
  },
  {
    label: "Hypertension grade 2",
    systolicMin: 160,
    systolicMax: 180,
    diastolicMin: 100,
    diastolicMax: 110,
    color: "#ef4444",
  },
  {
    label: "Hypertension grade 3",
    systolicMin: 180,
    systolicMax: 999,
    diastolicMin: 110,
    diastolicMax: 999,
    color: "#dc2626",
  },
] as const;

export const METRIC_COLORS: Record<string, string> = {
  weight: "hsl(var(--series-weight))",
  fat_ratio: "hsl(var(--series-fat-ratio))",
  muscle_mass: "hsl(var(--series-muscle-mass))",
  hydration: "hsl(var(--series-hydration))",
  bone_mass: "hsl(var(--series-bone-mass))",
  bmi: "hsl(var(--series-bmi))",
  systolic: "hsl(var(--series-systolic))",
  diastolic: "hsl(var(--series-diastolic))",
  heart_rate: "hsl(var(--series-heart-rate))",
  steps: "hsl(var(--series-steps))",
  calories: "hsl(var(--series-calories))",
  distance: "hsl(var(--series-distance))",
  sleep_duration: "hsl(var(--series-sleep-duration))",
  sleep_score: "hsl(var(--series-sleep-score))",
  deep_sleep: "hsl(var(--series-deep-sleep))",
  light_sleep: "hsl(var(--series-light-sleep))",
  rem_sleep: "hsl(var(--series-rem-sleep))",
  awake: "hsl(var(--series-awake))",
};

export const TIME_RANGES: TimeRange[] = [
  { label: "7j", value: "7d", days: 7 },
  { label: "30j", value: "30d", days: 30 },
  { label: "90j", value: "90d", days: 90 },
  { label: "1 an", value: "1y", days: 365 },
  { label: "Tout", value: "all", days: 3650 },
];

export const NAV_SECTIONS = [
  {
    label: null,
    items: [{ route: "/", label: "Tableau de bord", icon: "LayoutDashboard" }],
  },
  {
    label: "Corps",
    items: [
      { route: "/body", label: "Composition", icon: "Scale" },
      { route: "/blood-pressure", label: "Tension", icon: "Heart" },
      { route: "/measurements", label: "Mensurations", icon: "Ruler" },
    ],
  },
  {
    label: "Activité",
    items: [
      { route: "/activity", label: "Activité", icon: "Footprints" },
      { route: "/workouts", label: "Entraînements", icon: "Dumbbell" },
      { route: "/sleep", label: "Sommeil", icon: "Moon" },
      { route: "/heart", label: "Cœur / ECG", icon: "HeartPulse" },
    ],
  },
  {
    label: "Suivi",
    items: [
      { route: "/medications", label: "Médicaments", icon: "Pill" },
      { route: "/labs", label: "Analyses labo", icon: "FlaskConical" },
      { route: "/journal", label: "Journal", icon: "BookOpen" },
    ],
  },
  {
    label: "Analyse",
    items: [
      { route: "/profile", label: "Profil", icon: "User" },
      { route: "/analyses", label: "Corrélations", icon: "ChartLine" },
      { route: "/insights", label: "Constats", icon: "Sparkles" },
      { route: "/explorer", label: "Explorer", icon: "Compass" },
      { route: "/alerts", label: "Alertes", icon: "Bell" },
      { route: "/timeline", label: "Synchronisations", icon: "Activity" },
    ],
  },
] as const;

export type NavItem = { readonly route: string; readonly label: string; readonly icon: string };

export const NAV_ITEMS: NavItem[] = NAV_SECTIONS.flatMap((s) => [...s.items]);

export const NAV_SECONDARY: NavItem[] = [
  { route: "/export", label: "Exporter", icon: "Download" },
  { route: "/settings", label: "Paramètres", icon: "Settings" },
];

export const PAGE_TITLES: Record<string, string> = Object.fromEntries(
  [...NAV_ITEMS, ...NAV_SECONDARY].map((item) => [item.route, item.label])
);

export const DIRECTION_LABELS: Record<string, string> = {
  decrease: "Diminuer",
  increase: "Augmenter",
  maintain: "Maintenir",
};

export const DIRECTION_OPTIONS = [
  { value: "decrease", label: "Diminuer" },
  { value: "increase", label: "Augmenter" },
  { value: "maintain", label: "Maintenir" },
] as const;
