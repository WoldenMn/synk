import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { expectNoAxeViolations } from "@/test-utils/axe";
import { AppLayout } from "./AppLayout";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => Promise.resolve(cmd === "list_sources" ? [] : false)),
}));

describe("AppLayout : page de bienvenue", () => {
  it("garde un repère principal et un titre de page derrière l'assistant", async () => {
    localStorage.clear();
    render(
      <QueryClientProvider client={new QueryClient()}>
        <MemoryRouter initialEntries={["/"]}>
          <Routes>
            <Route path="/" element={<AppLayout />}>
              <Route index element={<p>Accueil</p>} />
            </Route>
          </Routes>
        </MemoryRouter>
      </QueryClientProvider>
    );
    expect(await screen.findByRole("dialog", {}, { timeout: 5000 })).toBeInTheDocument();
    expect(document.querySelector("main")).not.toBeNull();
    expect(document.querySelector("h1")).not.toBeNull();
    await expectNoAxeViolations(document.body);
  });
});
