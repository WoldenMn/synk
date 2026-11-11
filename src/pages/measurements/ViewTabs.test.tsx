import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ViewTabs } from "./ViewTabs";
import { SizesHeader } from "./SizesHeader";

describe("bascules de Mensurations", () => {
  it("la vue active de ViewTabs est annoncée", () => {
    render(<ViewTabs view="sizes" setView={vi.fn()} />);
    expect(screen.getByRole("button", { name: "Tailles", pressed: true })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Fiche", pressed: false })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Tours de corps", pressed: false })
    ).toBeInTheDocument();
  });

  it("le système de tailles actif est annoncé", () => {
    render(
      <SizesHeader
        brand={null}
        selectedBrand={null}
        setSelectedBrand={vi.fn()}
        sizeSystem="us"
        setSizeSystem={vi.fn()}
      />
    );
    expect(screen.getByRole("button", { name: "us", pressed: true })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "eu", pressed: false })).toBeInTheDocument();
  });
});
