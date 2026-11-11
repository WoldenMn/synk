import { describe, it, expect, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ONBOARDING_KEY } from "@/lib/onboarding";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => undefined)),
}));
vi.mock("@/lib/db", () => ({
  updateUserProfile: vi.fn(),
}));
vi.mock("@tanstack/react-query", () => ({
  useQueryClient: () => ({ invalidateQueries: vi.fn() }),
}));
vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn(), warning: vi.fn() },
}));

import { OnboardingWizard } from "./OnboardingWizard";

describe("OnboardingWizard a11y", () => {
  it("has role=dialog and aria-modal=true", () => {
    render(<OnboardingWizard onComplete={vi.fn()} />);
    const dialog = screen.getByRole("dialog");
    expect(dialog).toHaveAttribute("aria-modal", "true");
  });

  it("has an aria-labelledby pointing to a visible title", () => {
    render(<OnboardingWizard onComplete={vi.fn()} />);
    const dialog = screen.getByRole("dialog");
    const labelledBy = dialog.getAttribute("aria-labelledby");
    expect(labelledBy).toBeTruthy();
    const titleEl = document.getElementById(labelledBy!);
    expect(titleEl).toBeInTheDocument();
    expect(titleEl?.textContent?.length).toBeGreaterThan(0);
  });

  it("se quitte d'un clic dès la première étape et se souvient du choix", async () => {
    localStorage.clear();
    const onComplete = vi.fn();
    render(<OnboardingWizard onComplete={onComplete} />);
    await userEvent.click(screen.getByRole("button", { name: "Passer l'assistant" }));
    expect(onComplete).toHaveBeenCalledTimes(1);
    expect(localStorage.getItem(ONBOARDING_KEY)).toBe("true");
  });

  it("se ferme avec Échap", () => {
    localStorage.clear();
    const onComplete = vi.fn();
    render(<OnboardingWizard onComplete={onComplete} />);
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(onComplete).toHaveBeenCalledTimes(1);
    expect(localStorage.getItem(ONBOARDING_KEY)).toBe("true");
  });
});
