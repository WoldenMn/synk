import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ConfirmProvider } from "@/components/ui/confirm-dialog";
import { ErrorLogSection } from "./ErrorLogSection";

const entry = (id: number) => ({
  id,
  occurred_at: "2026-10-01T09:30:00Z",
  source: "sync",
  code: "NETWORK",
  message: `Erreur ${id}`,
  context: null,
});

vi.mock("../hooks", () => ({
  useErrorLog: () => ({ data: [entry(1), entry(2)], isLoading: false }),
  usePurgeErrorLog: () => ({ mutate: vi.fn(), isPending: false }),
}));

describe("ErrorLogSection", () => {
  it("relie le bouton de chaque entrée à son panneau", async () => {
    const { default: userEvent } = await import("@testing-library/user-event");
    const user = userEvent.setup();
    render(
      <ConfirmProvider>
        <ErrorLogSection />
      </ConfirmProvider>
    );
    const toggles = screen.getAllByRole("button", { name: /NETWORK/ });
    expect(toggles).toHaveLength(2);
    for (const toggle of toggles) expect(toggle).toHaveAttribute("aria-expanded", "false");

    await user.click(toggles[0] as HTMLElement);
    const opened = screen.getAllByRole("button", { name: /NETWORK/ })[0] as HTMLElement;
    expect(opened).toHaveAttribute("aria-expanded", "true");
    expect(document.getElementById(opened.getAttribute("aria-controls") ?? "")).not.toBeNull();
  });
});
