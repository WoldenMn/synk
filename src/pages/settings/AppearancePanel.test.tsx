import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import { useUiStore } from "@/stores/uiStore";
import { AppearancePanel } from "./AppearancePanel";

describe("AppearancePanel", () => {
  beforeEach(() => {
    useUiStore.setState({ hiddenRoutes: [] });
  });

  it("expose chaque page masquable comme un interrupteur", async () => {
    const user = userEvent.setup();
    render(<AppearancePanel />);
    const sommeil = screen.getByRole("switch", { name: /Sommeil/ });
    expect(sommeil).toHaveAttribute("aria-checked", "true");
    await user.click(sommeil);
    expect(screen.getByRole("switch", { name: /Sommeil/ })).toHaveAttribute(
      "aria-checked",
      "false"
    );
  });
});
