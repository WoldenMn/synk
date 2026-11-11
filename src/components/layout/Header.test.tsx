import { describe, it, expect, vi, beforeEach } from "vitest";
import { createRef } from "react";
import { render, screen } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { MemoryRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Header } from "./Header";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => Promise.resolve(null)),
}));

function renderHeader() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
  return render(
    <QueryClientProvider client={client}>
      <MemoryRouter>
        <Header headingRef={createRef<HTMLHeadingElement>()} />
      </MemoryRouter>
    </QueryClientProvider>
  );
}

describe("Header dernière synchro", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("lit la dernière synchro réussie dans la base", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      Promise.resolve(cmd === "get_last_successful_sync" ? "2026-10-01T09:30:00Z" : null)
    );
    renderHeader();
    expect(await screen.findByText(/Synchronisé le/)).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("get_last_successful_sync");
  });

  it("dit « Jamais synchronisé » quand la base n'a aucune synchro réussie", async () => {
    vi.mocked(invoke).mockImplementation(() => Promise.resolve(null));
    renderHeader();
    expect(await screen.findByText("Jamais synchronisé")).toBeInTheDocument();
  });
});
