import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { SizesView } from "./SizesView";
import { BRANDS } from "@/features/clothingSizes/brands";
import { defined } from "@/test-utils/defined";

vi.mock("@/features/clothingSizes/hooks", () => ({
  useClothingInputs: () => ({
    data: { chest: 100, neck: 40, waist: 82, inseam: 82 },
    isLoading: false,
  }),
  useClothingSizes: () => ({ data: [] }),
  useSaveEstimatedSizes: () => ({ mutate: vi.fn() }),
}));

function mount() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={qc}>
      <SizesView />
    </QueryClientProvider>
  );
}

describe("SizesView — mode marque", () => {
  it("liste Générique + les marques dans le sélecteur", () => {
    mount();
    const select = screen.getByLabelText("Marque") as HTMLSelectElement;
    expect(select).toBeTruthy();

    const options = Array.from(select.options);
    expect(options[0]?.textContent).toBe("Générique");
    expect(options[0]?.value).toBe("");
    expect(options.length).toBe(1 + BRANDS.length);

    for (const b of BRANDS) {
      expect(options.some((o) => o.value === b.id)).toBe(true);
    }
  });

  it("bascule sur une marque et change l'affichage (sous-titre indicatif + switch EU/US/UK masqué)", () => {
    mount();
    const select = screen.getByLabelText("Marque") as HTMLSelectElement;

    // En mode générique, le switch EU/US/UK est présent.
    expect(screen.getByRole("button", { name: "eu" })).toBeInTheDocument();

    const firstBrand = defined(BRANDS[0]);
    fireEvent.change(select, { target: { value: firstBrand.id } });

    // Sous-titre passe en mode marque.
    expect(
      screen.getByText(
        new RegExp(
          `Tailles ${firstBrand.label} estimées depuis vos mensurations, à titre indicatif`,
          "i"
        )
      )
    ).toBeInTheDocument();

    // Le switch EU/US/UK est masqué en mode marque.
    expect(screen.queryByRole("button", { name: "eu" })).not.toBeInTheDocument();
  });
});
