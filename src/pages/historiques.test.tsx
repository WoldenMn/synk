import { beforeAll, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ComponentType } from "react";
import { ConfirmProvider } from "@/components/ui/confirm-dialog";
import type { BodyMeasure } from "@/features/bodyMeasures/types";
import type { BloodPressure } from "@/features/bloodPressure/types";
import type { SleepRecord } from "@/features/sleep/types";
import type { Activity } from "@/features/activity/types";

const body = (id: number, measured_at: string, source: string): BodyMeasure => ({
  id,
  measured_at,
  source,
  weight: 70 + id,
  fat_ratio: null,
  fat_mass: null,
  fat_free_mass: null,
  muscle_mass: null,
  bone_mass: null,
  hydration: null,
  bmi: null,
});

const pressure = (id: number, measured_at: string, source: string): BloodPressure => ({
  id,
  measured_at,
  source,
  systolic: 118 + id,
  diastolic: 74,
  heart_rate: 60,
});

const sleep = (id: number, date: string, source: string): SleepRecord => ({
  id,
  date,
  source,
  start_at: null,
  end_at: null,
  duration_total: 28800,
  duration_deep: null,
  duration_light: null,
  duration_rem: null,
  duration_awake: null,
  duration_to_sleep: null,
  duration_to_wakeup: null,
  sleep_score: 80 + id,
  hr_average: null,
  hr_min: null,
  hr_max: null,
  rr_average: null,
  snoring: null,
  snoring_count: null,
  sleep_efficiency: null,
  nb_rem_episodes: null,
  out_of_bed_count: null,
  waso: null,
  total_timeinbed: null,
  apnea_hypopnea_index: null,
  breathing_disturbances_intensity: null,
  asleepduration: null,
  night_events: null,
});

const activity = (id: number, date: string, source: string): Activity => ({
  id,
  date,
  source,
  steps: 8000 + id,
  distance: null,
  elevation: null,
  calories_active: null,
  calories_total: null,
  duration_soft: null,
  duration_moderate: null,
  duration_intense: null,
  duration_active: null,
  hr_average: null,
  hr_min: null,
  hr_max: null,
  hr_zone_0: null,
  hr_zone_1: null,
  hr_zone_2: null,
  hr_zone_3: null,
});

const DATA: Record<string, unknown[]> = {
  get_body_measures: [
    body(1, "2026-07-01T08:00:00", "manual"),
    body(2, "2026-09-01T08:00:00", "withings"),
  ],
  get_blood_pressure: [
    pressure(1, "2026-07-01T08:00:00", "manual"),
    pressure(2, "2026-09-01T08:00:00", "withings"),
  ],
  get_sleep_records: [sleep(1, "2026-07-01", "manual"), sleep(2, "2026-09-01", "withings")],
  get_activity: [activity(1, "2026-07-01", "manual"), activity(2, "2026-09-01", "withings")],
};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd in DATA) return Promise.resolve(DATA[cmd]);
    if (cmd === "get_goals") return Promise.resolve([]);
    return Promise.resolve(null);
  }),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => undefined)),
}));
vi.mock("recharts", async () => {
  const actual = await vi.importActual<typeof import("recharts")>("recharts");
  return { ...actual, ResponsiveContainer: () => null };
});

async function renderPage(Component: ComponentType, label: string) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
  render(
    <QueryClientProvider client={client}>
      <ConfirmProvider>
        <MemoryRouter>
          <Component />
        </MemoryRouter>
      </ConfirmProvider>
    </QueryClientProvider>
  );
  await screen.findByRole("table", { name: label });
}

const PAGES: Array<[string, () => Promise<{ default: ComponentType }>, string]> = [
  ["Composition", () => import("./BodyComposition"), "Pesées"],
  ["Tension", () => import("./BloodPressure"), "Mesures de tension"],
  ["Sommeil", () => import("./Sleep"), "Nuits de sommeil"],
  ["Activité", () => import("./Activity"), "Journées d'activité"],
];

beforeAll(async () => {
  await Promise.all(PAGES.map(([, load]) => load()));
}, 120000);

describe.each(PAGES)("historique %s", (_name, load, label) => {
  it("ne propose la suppression que d'une ligne saisie à la main", async () => {
    const mod = await load();
    await renderPage(mod.default, label);
    expect(screen.getAllByRole("button", { name: /^Supprimer (la|le|cette)/ })).toHaveLength(1);
  });

  it("cherche dans le texte affiché de la date", async () => {
    const user = userEvent.setup();
    const mod = await load();
    await renderPage(mod.default, label);
    const table = screen.getByRole("table", { name: label });
    expect(table.querySelectorAll("tbody tr")).toHaveLength(2);
    await user.type(screen.getByRole("textbox", { name: `Rechercher dans ${label}` }), "sept");
    expect(table.querySelectorAll("tbody tr")).toHaveLength(1);
    expect(table).toHaveTextContent("sept.");
  });

  it("nomme la source de chaque ligne et la cherche par son libellé", async () => {
    const user = userEvent.setup();
    const mod = await load();
    await renderPage(mod.default, label);
    const table = screen.getByRole("table", { name: label });
    expect(table).toHaveTextContent("Saisie manuelle");
    expect(table).toHaveTextContent("Withings");
    await user.type(screen.getByRole("textbox", { name: `Rechercher dans ${label}` }), "Withings");
    expect(table.querySelectorAll("tbody tr")).toHaveLength(1);
    expect(table).not.toHaveTextContent("Saisie manuelle");
  });
});
