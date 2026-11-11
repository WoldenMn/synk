import { beforeAll, beforeEach, describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useUiStore } from "@/stores/uiStore";
import { QuickAddButton } from "./QuickAddButton";

function renderButton() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
  return render(
    <QueryClientProvider client={client}>
      <QuickAddButton />
    </QueryClientProvider>
  );
}

describe("QuickAddButton", () => {
  beforeAll(async () => {
    await Promise.all([
      import("@/features/bodyMeasures/components/ManualWeightForm"),
      import("@/features/bloodPressure/components/ManualBPForm"),
      import("@/features/manualMeasurements/components/ManualMeasurementForm"),
    ]);
  }, 60000);

  beforeEach(() => {
    useUiStore.setState({ quickAddOpen: false });
  });

  it("ouvre la pesée dès le premier clic", async () => {
    const user = userEvent.setup();
    renderButton();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Ajout rapide" }));
    await user.click(screen.getByRole("menuitem", { name: /Poids/ }));

    expect(screen.getByRole("button", { name: "Ajout rapide" })).toBeVisible();
    expect(await screen.findByRole("dialog", {}, { timeout: 10000 })).toBeInTheDocument();
  });

  it("ouvre la tension au premier clic", async () => {
    const user = userEvent.setup();
    renderButton();

    await user.click(screen.getByRole("button", { name: "Ajout rapide" }));
    await user.click(screen.getByRole("menuitem", { name: /Tension/ }));
    expect(await screen.findByRole("dialog", {}, { timeout: 10000 })).toBeInTheDocument();
  });

  it("se pilote au clavier : flèches, Échap, focus rendu", async () => {
    const user = userEvent.setup();
    renderButton();
    await user.click(screen.getByRole("button", { name: "Ajout rapide" }));
    const items = screen.getAllByRole("menuitem");
    expect(items).toHaveLength(3);
    expect(items[0]).toHaveFocus();
    await user.keyboard("{ArrowDown}");
    expect(items[1]).toHaveFocus();
    await user.keyboard("{ArrowUp}{ArrowUp}");
    expect(items[2]).toHaveFocus();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Ajout rapide" })).toHaveFocus();
  });
});
