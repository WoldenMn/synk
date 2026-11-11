import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ConfirmProvider } from "@/components/ui/confirm-dialog";
import type { ReactElement } from "react";

const mutate = vi.fn();
vi.mock("../hooks", () => ({
  useAddMeasurementSession: () => ({ mutate, isPending: false }),
  useBodySheet: () => ({
    data: [
      { slug: "waist", label: "Tour de taille", unit: "cm", value: 86, measured_at: "2026-07-01" },
    ],
  }),
}));

import { MeasurementWizard } from "./MeasurementWizard";
import { defined } from "@/test-utils/defined";

function renderWizard(ui: ReactElement) {
  return render(<ConfirmProvider>{ui}</ConfirmProvider>);
}

describe("MeasurementWizard", () => {
  beforeEach(() => mutate.mockReset());

  it("propose les deux parcours au démarrage", () => {
    renderWizard(<MeasurementWizard open onOpenChange={vi.fn()} />);
    expect(screen.getByRole("button", { name: /essentiel/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /complet/i })).toBeInTheDocument();
  });

  it("affiche la première étape et sa progression", () => {
    renderWizard(<MeasurementWizard open onOpenChange={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: /complet/i }));
    expect(screen.getByText("Hauteur totale")).toBeInTheDocument();
    expect(screen.getByText("1 / 33")).toBeInTheDocument();
  });

  it("marque sur la silhouette le point de l'étape en cours, et lui seul", () => {
    renderWizard(<MeasurementWizard open onOpenChange={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: /complet/i }));
    const points = document.querySelectorAll("svg circle");
    expect(points).toHaveLength(1);
    expect(defined(points[0]).getAttribute("class")).toContain("fill-primary");
  });

  it("laisse le champ vide et montre la dernière valeur en texte fantome", () => {
    renderWizard(<MeasurementWizard open onOpenChange={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: /essentiel/i }));
    for (let i = 0; i < 16; i++) {
      const input = screen.queryByLabelText("Tour de taille") as HTMLInputElement | null;
      if (input) {
        expect(input.value).toBe("");
        expect(input.placeholder).toBe("86");
        return;
      }
      fireEvent.click(screen.getByRole("button", { name: /suivant/i }));
    }
    throw new Error("étape Tour de taille non atteinte");
  });

  it("n'enregistre pas une mesure qui n'a pas été saisie", () => {
    renderWizard(<MeasurementWizard open onOpenChange={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: /essentiel/i }));
    fireEvent.change(screen.getByLabelText("Hauteur totale"), { target: { value: "172" } });
    fireEvent.click(screen.getByRole("button", { name: /recapitulatif/i }));
    fireEvent.click(screen.getByRole("button", { name: /enregistrer/i }));

    const payload = defined(mutate.mock.calls[0])[0];
    expect(payload.entries).toEqual([{ slug: "height", value: 172 }]);
  });

  it("le recapitulatif ne compte que ce qui a été saisi", () => {
    renderWizard(<MeasurementWizard open onOpenChange={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: /essentiel/i }));
    fireEvent.change(screen.getByLabelText("Hauteur totale"), { target: { value: "172" } });
    fireEvent.click(screen.getByRole("button", { name: /recapitulatif/i }));
    expect(screen.getByText(/^1 mesure a enregistrer/)).toBeInTheDocument();
  });

  it("affiche deux champs sur une étape latéralisée", () => {
    renderWizard(<MeasurementWizard open onOpenChange={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: /complet/i }));
    for (let i = 0; i < 33; i++) {
      if (screen.queryByLabelText("Tour de bras gauche")) {
        expect(screen.getByLabelText("Tour de bras droit")).toBeInTheDocument();
        return;
      }
      fireEvent.click(screen.getByRole("button", { name: /suivant/i }));
    }
    throw new Error("étape Tour de bras non atteinte");
  });

  it("n'enregistre que les champs remplis", () => {
    renderWizard(<MeasurementWizard open onOpenChange={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: /essentiel/i }));
    fireEvent.change(screen.getByLabelText("Hauteur totale"), { target: { value: "172" } });
    fireEvent.click(screen.getByRole("button", { name: /recapitulatif/i }));
    fireEvent.click(screen.getByRole("button", { name: /enregistrer/i }));

    expect(mutate).toHaveBeenCalledTimes(1);
    const payload = defined(mutate.mock.calls[0])[0];
    expect(payload.entries).toContainEqual({ slug: "height", value: 172 });
    expect(payload.entries.every((e: { value: number }) => Number.isFinite(e.value))).toBe(true);
  });

  it("demande confirmation avant de fermer si des champs sont remplis, et conserve la saisie si on annule", async () => {
    const onOpenChange = vi.fn();
    renderWizard(<MeasurementWizard open onOpenChange={onOpenChange} />);
    fireEvent.click(screen.getByRole("button", { name: /essentiel/i }));
    fireEvent.change(screen.getByLabelText("Hauteur totale"), { target: { value: "172" } });

    fireEvent.click(screen.getByRole("button", { name: "Fermer" }));
    expect(onOpenChange).not.toHaveBeenCalledWith(false);

    const cancelBtn = await screen.findByRole("button", { name: /continuer la saisie/i });
    fireEvent.click(cancelBtn);

    await waitFor(() => {
      expect(screen.queryByRole("button", { name: /abandonner/i })).not.toBeInTheDocument();
    });
    expect(screen.getByLabelText("Hauteur totale")).toHaveValue(172);
    expect(onOpenChange).not.toHaveBeenCalledWith(false);
  });

  it("ferme directement sans confirmation quand rien n'est rempli", () => {
    const onOpenChange = vi.fn();
    renderWizard(<MeasurementWizard open onOpenChange={onOpenChange} />);
    fireEvent.click(screen.getByRole("button", { name: "Fermer" }));
    expect(screen.queryByRole("button", { name: /abandonner/i })).not.toBeInTheDocument();
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it("desactive Enregistrer et affiche un message explicite quand aucun champ n'est rempli", () => {
    renderWizard(<MeasurementWizard open onOpenChange={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: /essentiel/i }));
    for (let i = 0; i < 16; i++) {
      const input = screen.queryByLabelText("Tour de taille");
      if (input) {
        fireEvent.change(input, { target: { value: "" } });
        break;
      }
      fireEvent.click(screen.getByRole("button", { name: /suivant/i }));
    }
    fireEvent.click(screen.getByRole("button", { name: /recapitulatif/i }));

    expect(screen.getByRole("button", { name: /enregistrer/i })).toBeDisabled();
    expect(screen.getByText(/aucune mesure/i)).toBeInTheDocument();
  });

  it("associe nativement le label au champ, seul comme latéralisé", () => {
    renderWizard(<MeasurementWizard open onOpenChange={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: /complet/i }));
    expect(screen.getByLabelText("Hauteur totale")).toBeInTheDocument();

    for (let i = 0; i < 33; i++) {
      if (screen.queryByLabelText("Tour de bras gauche")) break;
      fireEvent.click(screen.getByRole("button", { name: /suivant/i }));
    }
    expect(screen.getByLabelText("Tour de bras gauche")).toBeInTheDocument();
    expect(screen.getByLabelText("Tour de bras droit")).toBeInTheDocument();
  });

  it("Entrée passe au champ suivant sur une étape laterisee", () => {
    renderWizard(<MeasurementWizard open onOpenChange={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: /complet/i }));
    for (let i = 0; i < 33; i++) {
      const left = screen.queryByLabelText("Tour de bras gauche");
      if (left) {
        left.focus();
        fireEvent.keyDown(left, { key: "Enter" });
        expect(document.activeElement).toBe(screen.getByLabelText("Tour de bras droit"));
        return;
      }
      fireEvent.click(screen.getByRole("button", { name: /suivant/i }));
    }
    throw new Error("étape Tour de bras non atteinte");
  });

  it("Entrée sur le dernier champ passe a l'étape suivante", () => {
    renderWizard(<MeasurementWizard open onOpenChange={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: /complet/i }));
    expect(screen.getByText("1 / 33")).toBeInTheDocument();
    const input = screen.getByLabelText("Hauteur totale");
    fireEvent.keyDown(input, { key: "Enter" });
    expect(screen.getByText("2 / 33")).toBeInTheDocument();
  });

  it("un clic sur la progression ramene a l'étape visee sans perdre les saisies", () => {
    renderWizard(<MeasurementWizard open onOpenChange={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: /complet/i }));
    fireEvent.change(screen.getByLabelText("Hauteur totale"), { target: { value: "172" } });
    fireEvent.click(screen.getByRole("button", { name: /suivant/i }));
    expect(screen.getByText("2 / 33")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /aller à l'étape 1 :/i }));
    expect(screen.getByText("1 / 33")).toBeInTheDocument();
    expect((screen.getByLabelText("Hauteur totale") as HTMLInputElement).value).toBe("172");
  });
});
