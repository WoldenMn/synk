import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClientProvider, useQuery } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { createQueryClient } from "@/lib/queryClient";
import { RouteErrorBoundary } from "./RouteErrorBoundary";

vi.mock("@/lib/reportError", () => ({ reportError: vi.fn(), notifyError: vi.fn() }));

function Page({ load }: { load: () => Promise<string> }) {
  const { data } = useQuery({ queryKey: ["page-test"], queryFn: load, retry: false });
  return <p>{data ?? "chargement"}</p>;
}

describe("RouteErrorBoundary", () => {
  it("affiche l'erreur d'une lecture sans donnée, puis réessaie", async () => {
    const load = vi
      .fn<() => Promise<string>>()
      .mockRejectedValueOnce("Erreur base de donnees : database is locked")
      .mockResolvedValue("Nuits chargées");
    const user = userEvent.setup();
    render(
      <QueryClientProvider client={createQueryClient()}>
        <MemoryRouter>
          <RouteErrorBoundary>
            <Page load={load} />
          </RouteErrorBoundary>
        </MemoryRouter>
      </QueryClientProvider>
    );

    expect(await screen.findByText("Impossible d'afficher cette page")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Réessayer" }));
    expect(await screen.findByText("Nuits chargées")).toBeInTheDocument();
  });
});
