import { beforeAll, describe, it, vi } from "vitest";
import { act, render } from "@testing-library/react";
import { expectNoAxeViolations } from "@/test-utils/axe";
import { MemoryRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfirmProvider } from "@/components/ui/confirm-dialog";
import React, { cloneElement, isValidElement } from "react";
import type { ReactElement } from "react";
import { getBodyMeasures } from "@/features/bodyMeasures/api";
import { getSleepRecords } from "@/features/sleep/api";
import type { BodyMeasure } from "@/features/bodyMeasures/types";
import type { SleepRecord } from "@/features/sleep/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "withings_is_connected") return Promise.resolve(false);
    if (cmd === "get_credentials_status") return Promise.resolve(false);
    if (cmd === "get_error_log") return Promise.resolve([]);
    return Promise.resolve(null);
  }),
}));
vi.mock("recharts", async () => {
  const actual = await vi.importActual<typeof import("recharts")>("recharts");
  return {
    ...actual,
    ResponsiveContainer: ({ children }: { children: ReactElement }) =>
      isValidElement(children)
        ? cloneElement(children, { width: 800, height: 300 } as Partial<typeof children.props>)
        : children,
  };
});
vi.mock("@tauri-apps/api/event", () => ({
  // eslint-disable-next-line @typescript-eslint/no-empty-function -- unsubscribe noop retourne par listen() dans les mocks
  listen: vi.fn(() => Promise.resolve(() => {})),
}));
vi.mock("@/features/profile/api", () => ({
  updateUserProfile: vi.fn(),
  getUserProfile: vi.fn(() => Promise.resolve(null)),
}));
vi.mock("@/features/manualMeasurements/api", () => ({
  getMeasurementTypes: vi.fn(() => Promise.resolve([])),
  addMeasurementType: vi.fn(),
  deleteMeasurementType: vi.fn(),
  getManualMeasurements: vi.fn(() => Promise.resolve([])),
  addManualMeasurement: vi.fn(),
  deleteManualMeasurement: vi.fn(),
  getMeasurementStats: vi.fn(() => Promise.resolve(null)),
  setMeasurementGoal: vi.fn(),
}));
vi.mock("@/features/goals/api", () => ({
  getGoals: vi.fn(() => Promise.resolve([])),
  upsertGoal: vi.fn(),
  deleteGoal: vi.fn(),
}));
vi.mock("@/features/devices/api", () => ({
  getWithingsDevices: vi.fn(() => Promise.resolve([])),
}));
vi.mock("@/features/bodyMeasures/api", () => ({
  getBodyMeasures: vi.fn(() => Promise.resolve([])),
  getLatestBodyMeasure: vi.fn(() => Promise.resolve(null)),
  addManualBodyMeasure: vi.fn(),
  deleteBodyMeasure: vi.fn(),
}));
vi.mock("@/features/bloodPressure/api", () => ({
  getBloodPressure: vi.fn(() => Promise.resolve([])),
  getLatestBloodPressure: vi.fn(() => Promise.resolve(null)),
  addManualBloodPressure: vi.fn(),
  deleteBloodPressure: vi.fn(),
}));
vi.mock("@/features/sleep/api", () => ({
  getSleepRecords: vi.fn(() => Promise.resolve([])),
  getLatestSleep: vi.fn(() => Promise.resolve(null)),
  addManualSleep: vi.fn(),
  deleteSleepRecord: vi.fn(),
}));
vi.mock("@/features/activity/api", () => ({
  getActivity: vi.fn(() => Promise.resolve([])),
  getLatestActivity: vi.fn(() => Promise.resolve(null)),
  addManualActivity: vi.fn(),
  deleteActivity: vi.fn(),
}));
vi.mock("@/features/workouts/api", () => ({
  getWorkouts: vi.fn(() => Promise.resolve([])),
  getLatestWorkout: vi.fn(() => Promise.resolve(null)),
  addManualWorkout: vi.fn(),
  deleteWorkout: vi.fn(),
}));
vi.mock("@/features/heart/api", () => ({
  getHeartRecords: vi.fn(() => Promise.resolve([])),
  getLatestHeartRecord: vi.fn(() => Promise.resolve(null)),
}));
vi.mock("@/features/metrics/api", () => ({
  getMetricDefinitions: vi.fn(() => Promise.resolve([])),
  getMetricSeries: vi.fn(() => Promise.resolve([])),
  getMetricLatest: vi.fn(() => Promise.resolve(null)),
}));
vi.mock("@/features/medications/api", () => ({
  getMedications: vi.fn(() => Promise.resolve([])),
  getMedicationLogs: vi.fn(() => Promise.resolve([])),
  logMedicationTaken: vi.fn(() => Promise.resolve(1)),
}));
vi.mock("@/features/labs/api", () => ({
  getLabResults: vi.fn(() => Promise.resolve([])),
}));
vi.mock("@/features/journal/api", () => ({
  getJournalEntries: vi.fn(() => Promise.resolve([])),
  upsertJournalEntry: vi.fn(),
  deleteJournalEntry: vi.fn(),
}));
vi.mock("@/features/insights/api", () => ({
  getInsights: vi.fn(() => Promise.resolve([])),
  generateInsights: vi.fn(() => Promise.resolve({ generated: 0, failed: [] })),
  markInsightsSeen: vi.fn(() => Promise.resolve()),
}));
vi.mock("@/features/syncLog/api", () => ({
  getSyncLog: vi.fn(() => Promise.resolve([])),
  clearSyncLog: vi.fn(() => Promise.resolve()),
}));
vi.mock("@/features/alerts/api", () => ({
  getAlerts: vi.fn(() => Promise.resolve([])),
  getAlertEvents: vi.fn(() => Promise.resolve([])),
  addAlert: vi.fn(() => Promise.resolve(1)),
  deleteAlert: vi.fn(() => Promise.resolve()),
  checkAlerts: vi.fn(() => Promise.resolve([])),
  markAlertsSeen: vi.fn(() => Promise.resolve()),
}));
vi.mock("@/features/dashboard/api", () => ({
  getDbMetrics: vi.fn(() =>
    Promise.resolve({
      db_size_bytes: 0,
      body_measures_count: 0,
      blood_pressure_count: 0,
      sleep_count: 0,
      activity_count: 0,
      workouts_count: 0,
    })
  ),
}));
vi.mock("@/features/profileCard/api", () => ({
  getProfileCard: vi.fn(() =>
    Promise.resolve({
      body: {
        age: 36,
        sex: "male",
        height_cm: 178,
        weight_kg: 74.5,
        bmi: 23.5,
        fat_ratio: 18.2,
        muscle_mass: 58.1,
        hydration: 55.4,
        measured_at: "2026-08-01",
      },
      key_measurements: [
        { slug: "waist", label: "Tour de taille", value: 86, measured_at: "2026-07-31" },
      ],
      health: {
        active_treatments: ["Doliprane"],
      },
      sizes: [{ category: "top", value: "L" }],
      records: [
        {
          key: "weight_min",
          label: "Poids minimum",
          value: 74.5,
          unit: "kg",
          achieved_at: "2026-04-20",
        },
      ],
    })
  ),
  exportProfileCardPdf: vi.fn(() => Promise.resolve("")),
}));

vi.mock("@/features/audit/api", () => ({
  getAuditMetrics: vi.fn(() =>
    Promise.resolve({
      total_count: 0,
      insert_count: 0,
      delete_count: 0,
      oldest_at: null,
      newest_at: null,
    })
  ),
  purgeAuditLog: vi.fn(() => Promise.resolve(0)),
  getAuditPurgeConfig: vi.fn(() =>
    Promise.resolve({
      retention_days: 180,
      max_entries: 10000,
      enabled: true,
    })
  ),
  setAuditPurgeConfig: vi.fn(() => Promise.resolve()),
}));

async function renderPage(Component: React.ComponentType) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  const view = render(
    <QueryClientProvider client={client}>
      <ConfirmProvider>
        <MemoryRouter>
          <h1>Page</h1>
          <Component />
        </MemoryRouter>
      </ConfirmProvider>
    </QueryClientProvider>
  );
  await act(async () => {
    await new Promise((resolve) => setTimeout(resolve, 20));
  });
  return view;
}

const PAGES: Array<[string, () => Promise<{ default: React.ComponentType }>]> = [
  ["Dashboard", () => import("@/pages/Dashboard")],
  ["BodyComposition", () => import("@/pages/BodyComposition")],
  ["BloodPressure", () => import("@/pages/BloodPressure")],
  ["Sleep", () => import("@/pages/Sleep")],
  ["Activity", () => import("@/pages/Activity")],
  ["Workouts", () => import("@/pages/Workouts")],
  ["Heart", () => import("@/pages/Heart")],
  ["Measurements", () => import("@/pages/Measurements")],
  ["Alerts", () => import("@/pages/Alerts")],
  ["Profile", () => import("@/pages/Profile")],
  ["Analyses", () => import("@/pages/Analyses")],
  ["Medications", () => import("@/pages/Medications")],
  ["Labs", () => import("@/pages/Labs")],
  ["Constats", () => import("@/pages/Insights")],
  ["Journal", () => import("@/pages/Journal")],
  ["Synchronisations", () => import("@/pages/Timeline")],
  ["Export", () => import("@/pages/Export")],
  ["Settings", () => import("@/pages/Settings")],
  ["Explorer", () => import("@/pages/Explorer")],
];

describe("WCAG AA - pages a11y", () => {
  beforeAll(async () => {
    await Promise.allSettled(PAGES.map(([, importer]) => importer()));
  }, 120000);

  it.each(PAGES)(
    "%s has no critical/serious a11y violations",
    async (_name, importer) => {
      const mod = await importer();
      const { container } = await renderPage(mod.default);
      await expectNoAxeViolations(container, {
        rules: {
          "color-contrast": { enabled: false },
          region: { enabled: false }, // FP : MemoryRouter sans landmarks
        },
      });
    },
    15000
  );
});

function bodyRow(id: number, measured_at: string, weight: number): BodyMeasure {
  return {
    id,
    measured_at,
    source: "manual",
    weight,
    fat_ratio: 18 + id,
    fat_mass: 14 + id,
    fat_free_mass: 60,
    muscle_mass: 55 + id,
    bone_mass: 3.1,
    hydration: 40,
    bmi: 23,
  };
}

function sleepRow(id: number, date: string): SleepRecord {
  return {
    id,
    date,
    source: "manual",
    start_at: `${date}T23:00:00`,
    end_at: `${date}T07:00:00`,
    duration_total: 28800,
    duration_deep: 5400,
    duration_light: 14400,
    duration_rem: 6000,
    duration_awake: 3000,
    duration_to_sleep: 600,
    duration_to_wakeup: 300,
    sleep_score: 70 + id,
    hr_average: 55,
    hr_min: 48,
    hr_max: 70,
    rr_average: 14,
    snoring: 0,
    snoring_count: 0,
    sleep_efficiency: 90,
    nb_rem_episodes: 4,
    out_of_bed_count: 0,
    waso: 20,
    total_timeinbed: 30000,
    apnea_hypopnea_index: 4 + id * 10,
    breathing_disturbances_intensity: 10,
    asleepduration: 27000,
    night_events: 0,
  };
}

describe("WCAG AA - pages avec données", () => {
  it("BodyComposition monte tableau et graphiques sans violation", async () => {
    vi.mocked(getBodyMeasures).mockResolvedValue([
      bodyRow(1, "2026-07-01T08:00:00", 76),
      bodyRow(2, "2026-07-20T08:00:00", 75),
      bodyRow(3, "2026-08-18T08:00:00", 74),
    ]);
    const mod = await import("@/pages/BodyComposition");
    const { container } = await renderPage(mod.default);
    expect(container.querySelector("table")).not.toBeNull();
    expect(container.querySelector("svg.recharts-surface")).not.toBeNull();
    await expectNoAxeViolations(container, {
      rules: { "color-contrast": { enabled: false }, region: { enabled: false } },
    });
  }, 15000);

  it("Sleep monte tableau et graphiques sans violation", async () => {
    vi.mocked(getSleepRecords).mockResolvedValue([
      sleepRow(1, "2026-07-01"),
      sleepRow(2, "2026-07-02"),
      sleepRow(3, "2026-07-03"),
    ]);
    const mod = await import("@/pages/Sleep");
    const { container } = await renderPage(mod.default);
    expect(container.querySelector("table")).not.toBeNull();
    expect(container.querySelector("svg.recharts-surface")).not.toBeNull();
    await expectNoAxeViolations(container, {
      rules: { "color-contrast": { enabled: false }, region: { enabled: false } },
    });
  }, 15000);
});
