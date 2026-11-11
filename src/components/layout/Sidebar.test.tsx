import { beforeEach, describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { expectNoAxeViolations } from "@/test-utils/axe";
import { useUiStore } from "@/stores/uiStore";
import { Sidebar } from "./Sidebar";

function renderSidebar() {
  return render(
    <MemoryRouter>
      <Sidebar />
    </MemoryRouter>
  );
}

describe("Sidebar repliée", () => {
  beforeEach(() => {
    useUiStore.setState({ sidebarCollapsed: true });
  });

  it("donne un nom à chaque lien", async () => {
    const { container } = renderSidebar();
    expect(screen.getByRole("link", { name: "Sommeil" })).toBeInTheDocument();
    await expectNoAxeViolations(container);
  });

  it("montre l'infobulle quand un lien reçoit le focus clavier", async () => {
    const user = userEvent.setup();
    renderSidebar();
    for (let i = 0; i < 10 && !document.activeElement?.closest("a"); i++) await user.tab();
    expect(document.activeElement?.closest("a")).not.toBeNull();
    expect(screen.getByRole("tooltip")).toBeInTheDocument();
  });
});
