import { describe, expect, it, vi } from "vitest";
import { act, render } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfirmProvider } from "@/components/ui/confirm-dialog";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => Promise.resolve([])),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => undefined)),
}));

import Alerts from "./Alerts";

describe("page Alertes", () => {
  it("occupe la largeur : deux colonnes a 1920 px, aucune colonne etroite centree", async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    const { container } = render(
      <QueryClientProvider client={client}>
        <ConfirmProvider>
          <MemoryRouter>
            <Alerts />
          </MemoryRouter>
        </ConfirmProvider>
      </QueryClientProvider>
    );
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 20));
    });
    expect(container.querySelector(".max-w-3xl")).toBeNull();
    expect(container.firstElementChild?.className).toContain("xl:grid-cols-2");
    expect(container.firstElementChild?.children).toHaveLength(2);
  });
});
