import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ConfirmProvider } from "@/components/ui/confirm-dialog";

vi.mock("@/features/bodyProfile/hooks", () => ({
  useBodySheet: () => ({
    data: [
      { slug: "waist", label: "Tour de taille", unit: "cm", value: 86, measured_at: "2026-07-31" },
      { slug: "head", label: "Tour de tête", unit: "cm", value: null, measured_at: null },
    ],
    isLoading: false,
  }),
  useAddMeasurementSession: () => ({ mutate: vi.fn(), isPending: false }),
  useExportBodySheetPdf: () => ({ mutate: vi.fn(), isPending: false }),
}));

import { SheetView } from "./SheetView";

function renderSheetView() {
  return render(
    <ConfirmProvider>
      <SheetView />
    </ConfirmProvider>
  );
}

describe("SheetView", () => {
  it("affiche les valeurs connues", () => {
    renderSheetView();
    expect(screen.getByText("Tour de taille")).toBeInTheDocument();
    expect(screen.getByText("86")).toBeInTheDocument();
  });

  it("affiche un tiret pour les mesures jamais prises", () => {
    renderSheetView();
    const row = screen.getByText("Tour de tête").closest("tr")!;
    expect(row.textContent).toContain("—");
  });

  it("propose le relevé et l'export", () => {
    renderSheetView();
    expect(screen.getByRole("button", { name: /nouveau relev/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /exporter/i })).toBeInTheDocument();
  });
});
