import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

vi.mock("@/features/sources/api", () => ({
  importSourceFile: vi.fn(),
  cancelImport: vi.fn(() => Promise.resolve(true)),
}));
vi.mock("@/features/sources/csv/api", () => ({
  inspectCsvFile: vi.fn(),
  importCsvFile: vi.fn(),
  saveCsvMapping: vi.fn(),
  deleteCsvMapping: vi.fn(),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@/lib/reportError", () => ({ reportError: vi.fn() }));
vi.mock("sonner", () => ({
  toast: { warning: vi.fn(), success: vi.fn(), error: vi.fn(), info: vi.fn() },
}));

import { open } from "@tauri-apps/plugin-dialog";
import { reportError } from "@/lib/reportError";
import {
  deleteCsvMapping,
  importCsvFile,
  inspectCsvFile,
  saveCsvMapping,
} from "@/features/sources/csv/api";
import type { CsvInspection, ImportReport, SourceInfo } from "@/features/sources/types";
import { ImportCard } from "./ImportCard";

const CSV: SourceInfo = {
  id: "csv",
  label: "Fichiers CSV",
  kind: "file",
  domains: ["body"],
  connected: false,
  credentials_configured: false,
  last_sync_at: null,
  redirect_uri: null,
  developer_url: null,
  reauth: null,
  extensions: ["csv", "txt"],
};

const CONFIG = {
  delimiter: ";",
  decimal: ",",
  skip_rows: 0,
  date_column: "Date",
  time_column: null,
  date_format: "%d/%m/%Y",
  columns: [{ column: "Poids (kg)", metric: "weight", unit: "kg" }],
};

const INSPECTION: CsvInspection = {
  header: ["Date", "Poids (kg)", "Note"],
  sample: [["25/10/2026", "72,4", "ras"]],
  row_count: 12,
  signature: "abc",
  suggestion: CONFIG,
  preset: null,
  formats: [
    { id: "%d/%m/%Y", label: "Jour/mois/année", example: "25/10/2026" },
    { id: "iso", label: "ISO", example: "2026-10-25" },
  ],
  saved: [{ id: 7, name: "Ma balance", config: CONFIG }],
  unreadable_mappings: 0,
  targets: [
    {
      metric: "weight",
      label: "Poids",
      units: [
        { id: "kg", label: "kg" },
        { id: "lb", label: "lb" },
      ],
    },
  ],
};

const REPORT: ImportReport = {
  source: "csv",
  cancelled: false,
  tally: {
    observations: 12,
    observations_ignored: 0,
    blood_pressure: 0,
    sleep: 0,
    activity: 0,
    workouts: 0,
    devices: {},
  },
  skipped: [],
  duration_ms: 5,
};

function renderCard() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={client}>
      <ImportCard sources={[CSV]} />
    </QueryClientProvider>
  );
}

async function openAssistant() {
  vi.mocked(open).mockResolvedValue("C:/balance.csv");
  vi.mocked(inspectCsvFile).mockResolvedValue(INSPECTION);
  renderCard();
  await userEvent.click(screen.getByRole("button", { name: /Choisir un fichier/ }));
  return screen.findByRole("region", { name: "Assistant de correspondance CSV" });
}

describe("assistant CSV", () => {
  it("lit le fichier choisi, propose la correspondance devinée et l'importe telle quelle", async () => {
    vi.mocked(importCsvFile).mockResolvedValue(REPORT);
    const assistant = await openAssistant();

    expect(inspectCsvFile).toHaveBeenCalledWith("C:/balance.csv");
    expect(
      within(assistant).getByRole("heading", { name: "Correspondance du fichier CSV" })
    ).toHaveFocus();
    expect(within(assistant).getByLabelText("Colonne de la date")).toHaveValue("Date");
    expect(within(assistant).getByLabelText("Mesure de la colonne Poids (kg)")).toHaveValue(
      "weight"
    );
    expect(within(assistant).getByLabelText("Mesure de la colonne Note")).toHaveValue("");

    await userEvent.click(within(assistant).getByRole("button", { name: "Importer le fichier" }));

    await waitFor(() =>
      expect(importCsvFile).toHaveBeenCalledWith("C:/balance.csv", INSPECTION.suggestion)
    );
    expect(await screen.findByText("12 mesures")).toBeInTheDocument();
    expect(
      screen.queryByRole("region", { name: "Assistant de correspondance CSV" })
    ).not.toBeInTheDocument();
  });

  it("envoie la correspondance corrigée à la main", async () => {
    vi.mocked(importCsvFile).mockResolvedValue(REPORT);
    const assistant = await openAssistant();

    await userEvent.selectOptions(
      within(assistant).getByLabelText("Unité de la colonne Poids (kg)"),
      "lb"
    );
    await userEvent.selectOptions(
      within(assistant).getByLabelText("Séparateur des décimales"),
      "."
    );
    await userEvent.click(within(assistant).getByRole("button", { name: "Importer le fichier" }));

    await waitFor(() => expect(importCsvFile).toHaveBeenCalled());
    const sent = vi.mocked(importCsvFile).mock.calls[0]?.[1];
    expect(sent?.decimal).toBe(".");
    expect(sent?.columns).toEqual([{ column: "Poids (kg)", metric: "weight", unit: "lb" }]);
  });

  it("refuse d'importer sans colonne associée et le dit", async () => {
    const assistant = await openAssistant();

    await userEvent.selectOptions(
      within(assistant).getByLabelText("Mesure de la colonne Poids (kg)"),
      ""
    );

    expect(within(assistant).getByRole("alert")).toHaveTextContent(/au moins une colonne/);
    expect(within(assistant).getByRole("button", { name: "Importer le fichier" })).toBeDisabled();
  });

  it("enregistre, réapplique et oublie une correspondance", async () => {
    vi.mocked(saveCsvMapping).mockResolvedValue(8);
    vi.mocked(deleteCsvMapping).mockResolvedValue(true);
    const assistant = await openAssistant();

    expect(within(assistant).getByText("Ma balance")).toBeInTheDocument();
    await userEvent.type(
      within(assistant).getByLabelText("Nom de cette correspondance"),
      "Balance 2"
    );
    await userEvent.click(within(assistant).getByRole("button", { name: "Enregistrer" }));
    await waitFor(() =>
      expect(saveCsvMapping).toHaveBeenCalledWith("Balance 2", "abc", INSPECTION.suggestion)
    );
    expect(await within(assistant).findByText("Balance 2")).toBeInTheDocument();

    await userEvent.click(
      within(assistant).getByRole("button", { name: "Oublier la correspondance Ma balance" })
    );
    await waitFor(() => expect(deleteCsvMapping).toHaveBeenCalledWith(7));
    await waitFor(() =>
      expect(within(assistant).queryByText("Ma balance")).not.toBeInTheDocument()
    );
  });

  it("signale un fichier illisible sans ouvrir l'assistant", async () => {
    vi.mocked(open).mockResolvedValue("C:/binaire.csv");
    vi.mocked(inspectCsvFile).mockRejectedValue(new Error("binaire"));
    renderCard();

    await userEvent.click(screen.getByRole("button", { name: /Choisir un fichier/ }));

    await waitFor(() => expect(reportError).toHaveBeenCalled());
    expect(screen.queryByRole("region", { name: /Assistant/ })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Choisir un fichier/ })).toBeEnabled();
  });

  it("dit combien de correspondances enregistrées sont illisibles au lieu de les faire disparaître", async () => {
    vi.mocked(open).mockResolvedValue("C:/balance.csv");
    vi.mocked(inspectCsvFile).mockResolvedValue({ ...INSPECTION, unreadable_mappings: 2 });
    renderCard();
    await userEvent.click(screen.getByRole("button", { name: /Choisir un fichier/ }));
    const assistant = await screen.findByRole("region", {
      name: "Assistant de correspondance CSV",
    });

    expect(
      within(assistant).getByText(/2 correspondances enregistrées sont illisibles/)
    ).toBeInTheDocument();
  });
});
