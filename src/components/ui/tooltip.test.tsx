import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Tooltip } from "./tooltip";

describe("Tooltip", () => {
  it("s'affiche au focus clavier et au survol", async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Synchroniser (Ctrl+Maj+S)">
        <button>Synchroniser</button>
      </Tooltip>
    );
    expect(screen.queryByRole("tooltip")).toBeNull();
    await user.tab();
    expect(screen.getByRole("tooltip")).toHaveTextContent("Synchroniser (Ctrl+Maj+S)");
    await user.tab();
    expect(screen.queryByRole("tooltip")).toBeNull();
    await user.hover(screen.getByRole("button"));
    expect(screen.getByRole("tooltip")).toBeInTheDocument();
  });

  it("reste dans un repère de page, sans quoi un contrôleur d'accessibilité signale du contenu hors repère", async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Ajout rapide">
        <button>Ajout</button>
      </Tooltip>
    );
    await user.tab();
    const region = screen.getByRole("tooltip").closest("[role='region']");
    expect(region).not.toBeNull();
    expect(region).toHaveAttribute("aria-label", "Infobulle");
  });
});
