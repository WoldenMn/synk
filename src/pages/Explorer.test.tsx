import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { describe, expect, it, vi } from "vitest";
import Explorer from "./Explorer";

vi.mock("@/features/metrics/api", () => ({
  getMetricDefinitions: vi.fn(() => Promise.resolve([])),
  getMetricSeries: vi.fn(() => Promise.resolve([])),
}));

describe("Explorer", () => {
  it("annonce la catégorie active", async () => {
    const user = userEvent.setup();
    render(
      <QueryClientProvider client={new QueryClient()}>
        <MemoryRouter>
          <Explorer />
        </MemoryRouter>
      </QueryClientProvider>
    );
    expect(
      screen.getByRole("button", { name: "Composition corporelle", pressed: true })
    ).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Constantes de santé" }));
    expect(
      screen.getByRole("button", { name: "Constantes de santé", pressed: true })
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Composition corporelle", pressed: false })
    ).toBeInTheDocument();
  });
});
