import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { PrioritiesTab } from "./PrioritiesTab";

const WEIGHT = { slug: "weight", label: "Poids", sources: ["manual", "withings"], custom: false };

function renderTab(priorities: unknown[]) {
  vi.mocked(invoke).mockImplementation((cmd: string) =>
    Promise.resolve(cmd === "get_metric_priorities" ? priorities : null)
  );
  return render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <PrioritiesTab />
    </QueryClientProvider>
  );
}

describe("PrioritiesTab", () => {
  it("explique qu'il n'y a rien à choisir tant qu'aucune mesure n'a deux sources", async () => {
    renderTab([]);

    expect(
      await screen.findByText(/Aucune mesure n'a encore plusieurs sources/)
    ).toBeInTheDocument();
  });

  it("liste les sources de chaque mesure dans l'ordre retenu", async () => {
    renderTab([WEIGHT]);

    expect(await screen.findByRole("heading", { name: "Poids" })).toBeInTheDocument();
    const items = screen.getAllByRole("listitem");
    expect(items[0]).toHaveTextContent("Saisie manuelle");
    expect(items[1]).toHaveTextContent("Withings");
    const top = screen.getByRole("button", { name: "Monter Saisie manuelle pour Poids" });
    const bottom = screen.getByRole("button", { name: "Descendre Withings pour Poids" });
    expect(top).toHaveAttribute("aria-disabled", "true");
    expect(bottom).toHaveAttribute("aria-disabled", "true");
    expect(top).not.toBeDisabled();
  });

  it("enregistre le nouvel ordre quand on monte une source", async () => {
    const user = userEvent.setup();
    renderTab([WEIGHT]);

    await user.click(await screen.findByRole("button", { name: "Monter Withings pour Poids" }));

    expect(invoke).toHaveBeenCalledWith("set_metric_priority", {
      slug: "weight",
      sources: ["withings", "manual"],
    });
  });

  it("rétablit l'ordre automatique d'une mesure réglée à la main", async () => {
    const user = userEvent.setup();
    renderTab([{ ...WEIGHT, custom: true }]);

    await user.click(
      await screen.findByRole("button", { name: "Rétablir l'ordre automatique de Poids" })
    );

    expect(invoke).toHaveBeenCalledWith("set_metric_priority", { slug: "weight", sources: [] });
  });

  it("n'envoie rien quand on active un bouton en butée", async () => {
    const user = userEvent.setup();
    renderTab([WEIGHT]);
    vi.mocked(invoke).mockClear();

    await user.click(
      await screen.findByRole("button", { name: "Monter Saisie manuelle pour Poids" })
    );

    expect(invoke).not.toHaveBeenCalledWith("set_metric_priority", expect.anything());
  });

  it("annonce la nouvelle position après un déplacement", async () => {
    const user = userEvent.setup();
    renderTab([WEIGHT]);
    await screen.findByRole("heading", { name: "Poids" });
    expect(screen.getByRole("status")).toBeEmptyDOMElement();

    await user.click(await screen.findByRole("button", { name: "Monter Withings pour Poids" }));

    await waitFor(() =>
      expect(screen.getByRole("status")).toHaveTextContent("Withings, position 1 sur 2 pour Poids")
    );
  });

  it("garde le focus sur le bouton qui vient d'être activé", async () => {
    const user = userEvent.setup();
    renderTab([WEIGHT]);
    const up = await screen.findByRole("button", { name: "Monter Withings pour Poids" });

    await user.click(up);

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Monter Withings pour Poids" })).toHaveFocus()
    );
  });

  it("montre un chargement puis, en cas d'échec, une erreur avec Réessayer au lieu d'un état vide", async () => {
    const user = userEvent.setup();
    let fail = true;
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      cmd === "get_metric_priorities"
        ? fail
          ? Promise.reject(new Error("Erreur base de donnees : disk I/O error"))
          : Promise.resolve([WEIGHT])
        : Promise.resolve(null)
    );
    render(
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <PrioritiesTab />
      </QueryClientProvider>
    );

    expect(screen.getByText(/Chargement des priorités/)).toBeInTheDocument();
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent(/Impossible de lire les priorités/);
    expect(
      screen.queryByText(/Aucune mesure n'a encore plusieurs sources/)
    ).not.toBeInTheDocument();

    fail = false;
    await user.click(screen.getByRole("button", { name: "Réessayer" }));
    expect(await screen.findByRole("heading", { name: "Poids" })).toBeInTheDocument();
  });
});
