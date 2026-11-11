import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type { ReactElement, ReactNode } from "react";
import { ManualMeasurementForm } from "./ManualMeasurementForm";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

const FAKE_TYPES = [
  {
    id: 1,
    slug: "tour_taille",
    label: "Tour de taille",
    unit: "cm",
    category: "custom",
    sort_order: 1,
    is_builtin: 0,
    target_value: null,
    target_direction: null,
  },
  {
    id: 2,
    slug: "tour_hanches",
    label: "Tour de hanches",
    unit: "cm",
    category: "custom",
    sort_order: 2,
    is_builtin: 0,
    target_value: null,
    target_direction: null,
  },
];

function renderWithClient(ui: ReactElement) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(<QueryClientProvider client={client}>{ui}</QueryClientProvider>);
}

describe("ManualMeasurementForm", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Par défaut : get_measurement_types retourne la liste fake
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_measurement_types") return Promise.resolve(FAKE_TYPES);
      return Promise.resolve(undefined);
    });
  });

  it("rend les champs principaux du formulaire", async () => {
    renderWithClient(<ManualMeasurementForm open={true} onOpenChange={vi.fn()} />);

    expect(await screen.findByLabelText(/Type de mensuration/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Date et heure/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Valeur/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Notes/i)).toBeInTheDocument();
  });

  it("charge et affiche les types de mensuration dans le select", async () => {
    renderWithClient(<ManualMeasurementForm open={true} onOpenChange={vi.fn()} />);

    expect(await screen.findByText("Tour de taille (cm)")).toBeInTheDocument();
    expect(screen.getByText("Tour de hanches (cm)")).toBeInTheDocument();
  });

  it("désactive le bouton Enregistrer si aucun type ou valeur sélectionné", async () => {
    renderWithClient(<ManualMeasurementForm open={true} onOpenChange={vi.fn()} />);

    // Attendre le chargement des types
    await screen.findByText("Tour de taille (cm)");

    const submitBtn = screen.getByRole("button", { name: /Enregistrer/i });
    expect(submitBtn).toBeDisabled();
  });

  it("appelle add_manual_measurement avec le bon payload au submit", async () => {
    const onOpenChange = vi.fn();
    const user = userEvent.setup();
    renderWithClient(<ManualMeasurementForm open={true} onOpenChange={onOpenChange} />);

    // Attendre que les types soient chargés
    await screen.findByText("Tour de taille (cm)");

    // Sélectionner un type
    const select = screen.getByLabelText(/Type de mensuration/i);
    await user.selectOptions(select, "1");

    // Saisir une valeur
    const valueInput = screen.getByLabelText(/Valeur/i);
    await user.clear(valueInput);
    await user.type(valueInput, "82.5");

    // Soumettre
    const submitBtn = screen.getByRole("button", { name: /Enregistrer/i });
    await user.click(submitBtn);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("add_manual_measurement", {
        data: expect.objectContaining({
          measurement_type_id: 1,
          value: 82.5,
          notes: null,
        }),
      });
    });
  });

  it("inclut les notes dans le payload si remplies", async () => {
    const user = userEvent.setup();
    renderWithClient(<ManualMeasurementForm open={true} onOpenChange={vi.fn()} />);

    await screen.findByText("Tour de taille (cm)");

    await user.selectOptions(screen.getByLabelText(/Type de mensuration/i), "2");
    await user.type(screen.getByLabelText(/Valeur/i), "95");
    await user.type(screen.getByLabelText(/Notes/i), "après sport");

    await user.click(screen.getByRole("button", { name: /Enregistrer/i }));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("add_manual_measurement", {
        data: expect.objectContaining({
          measurement_type_id: 2,
          value: 95,
          notes: "après sport",
        }),
      });
    });
  });

  it("ferme le dialog (onOpenChange false) après un submit réussi", async () => {
    const onOpenChange = vi.fn();
    const user = userEvent.setup();
    renderWithClient(<ManualMeasurementForm open={true} onOpenChange={onOpenChange} />);

    await screen.findByText("Tour de taille (cm)");

    await user.selectOptions(screen.getByLabelText(/Type de mensuration/i), "1");
    await user.type(screen.getByLabelText(/Valeur/i), "80");

    await user.click(screen.getByRole("button", { name: /Enregistrer/i }));

    await waitFor(() => {
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  it("pré-sélectionne le type si defaultTypeId est fourni", async () => {
    renderWithClient(
      <ManualMeasurementForm open={true} onOpenChange={vi.fn()} defaultTypeId={2} />
    );

    await screen.findByText("Tour de taille (cm)");

    const select = screen.getByLabelText(/Type de mensuration/i) as HTMLSelectElement;
    expect(select.value).toBe("2");
  });

  it("n'appelle pas add_manual_measurement si le type est vide", async () => {
    const user = userEvent.setup();
    renderWithClient(<ManualMeasurementForm open={true} onOpenChange={vi.fn()} />);

    await screen.findByText("Tour de taille (cm)");

    // Saisir une valeur sans sélectionner de type
    await user.type(screen.getByLabelText(/Valeur/i), "80");

    const submitBtn = screen.getByRole("button", { name: /Enregistrer/i });
    // Le bouton reste disabled sans typeId
    expect(submitBtn).toBeDisabled();
  });

  it("affiche le bouton Annuler qui ferme le dialog", async () => {
    const onOpenChange = vi.fn();
    const user = userEvent.setup();
    renderWithClient(<ManualMeasurementForm open={true} onOpenChange={onOpenChange} />);

    await screen.findByText("Tour de taille (cm)");

    await user.click(screen.getByRole("button", { name: /Annuler/i }));

    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(mockInvoke).not.toHaveBeenCalledWith("add_manual_measurement", expect.anything());
  });
  it("propose l'heure de l'ouverture, pas celle du montage", () => {
    vi.useFakeTimers({ toFake: ["Date"] });
    try {
      vi.setSystemTime(new Date(2026, 9, 3, 9, 0));
      const client = new QueryClient({
        defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
      });
      const wrapper = ({ children }: { children: ReactNode }) => (
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
      );
      const { rerender } = render(<ManualMeasurementForm open={false} onOpenChange={vi.fn()} />, {
        wrapper,
      });

      vi.setSystemTime(new Date(2026, 9, 3, 14, 5));
      rerender(<ManualMeasurementForm open={true} onOpenChange={vi.fn()} />);

      expect(screen.getByLabelText(/Date et heure/i)).toHaveValue("2026-10-03T14:05");
    } finally {
      vi.useRealTimers();
    }
  });
});
