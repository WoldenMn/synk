import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { CsvInspection } from "@/features/sources/types";
import { CsvAssistant } from "./CsvAssistant";

const CONFIG = {
  delimiter: ";",
  decimal: ",",
  skip_rows: 0,
  date_column: "Date",
  time_column: null,
  date_format: "%d/%m/%Y",
  columns: [{ column: "Poids", metric: "weight", unit: "kg" }],
};

const INSPECTION: CsvInspection = {
  header: ["Date", "Poids", "Poids"],
  sample: [["25/10/2026", "72,4", "72,5"]],
  row_count: 1,
  signature: "abc",
  suggestion: CONFIG,
  preset: null,
  formats: [{ id: "%d/%m/%Y", label: "Jour/mois/année", example: "25/10/2026" }],
  saved: [],
  unreadable_mappings: 0,
  targets: [
    {
      metric: "weight",
      label: "Poids",
      units: [{ id: "kg", label: "kg" }],
    },
  ],
};

function mount() {
  return render(
    <CsvAssistant
      idPrefix="t"
      inspection={INSPECTION}
      onImport={vi.fn()}
      onSave={vi.fn()}
      onForget={vi.fn()}
      onClose={vi.fn()}
    />
  );
}

describe("CsvAssistant", () => {
  it("replie les quatre listes du format sur une ou deux colonnes avant la pleine largeur", () => {
    mount();
    const grid = screen.getByLabelText("Colonne de la date").closest("div.grid");
    expect(grid?.className).toContain("grid-cols-1");
    expect(grid?.className).toContain("min-[640px]:grid-cols-2");
    expect(grid?.className).toContain("xl:grid-cols-4");
    expect(grid?.className).not.toMatch(/(^|\s)grid-cols-4/);
  });

  it("supporte deux colonnes de même nom sans clé React en double", () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => undefined);
    mount();
    const warnings = spy.mock.calls.filter((call) => String(call[0]).includes("same key"));
    spy.mockRestore();
    expect(warnings).toEqual([]);
  });

  it("dit pourquoi Enregistrer reste grisé tant que la correspondance n'a pas de nom", async () => {
    mount();
    const field = screen.getByLabelText("Nom de cette correspondance");
    const help = screen.getByText("Donnez un nom à cette correspondance pour l'enregistrer.");
    expect(field.getAttribute("aria-describedby")).toBe(help.id);
    expect(screen.getByRole("button", { name: "Enregistrer" })).toBeDisabled();

    await userEvent.type(field, "Ma balance");

    expect(
      screen.queryByText("Donnez un nom à cette correspondance pour l'enregistrer.")
    ).toBeNull();
    expect(field.getAttribute("aria-describedby")).toBeNull();
    expect(screen.getByRole("button", { name: "Enregistrer" })).toBeEnabled();
  });
});
