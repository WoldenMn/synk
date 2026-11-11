import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type { ReactElement } from "react";
import { SizeEditDialog } from "./SizeEditDialog";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

function renderWithClient(ui: ReactElement) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(<QueryClientProvider client={client}>{ui}</QueryClientProvider>);
}

const defaultProps = {
  open: true,
  onOpenChange: vi.fn(),
  category: "haut",
  label: "Haut / T-shirt",
  current: null,
  source: "override" as const,
};

describe("SizeEditDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
  });

  it("rend le dialog avec le label et le champ saisie", () => {
    renderWithClient(<SizeEditDialog {...defaultProps} />);

    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(screen.getByText("Haut / T-shirt")).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/Ex : 52/i)).toBeInTheDocument();
  });

  it("n'affiche pas le bouton Réinitialiser si current est null", () => {
    renderWithClient(<SizeEditDialog {...defaultProps} current={null} />);

    expect(screen.queryByRole("button", { name: /Réinitialiser/i })).not.toBeInTheDocument();
  });

  it("affiche le bouton Réinitialiser si current est défini", () => {
    renderWithClient(<SizeEditDialog {...defaultProps} current="M" />);

    expect(screen.getByRole("button", { name: /Réinitialiser/i })).toBeInTheDocument();
  });

  it("pré-remplit le champ avec la valeur current à l'ouverture", () => {
    renderWithClient(<SizeEditDialog {...defaultProps} current="L" />);

    const input = screen.getByPlaceholderText(/Ex : 52/i) as HTMLInputElement;
    expect(input.value).toBe("L");
  });

  it("désactive le bouton Enregistrer si le champ est vide", () => {
    renderWithClient(<SizeEditDialog {...defaultProps} current={null} />);

    const saveBtn = screen.getByRole("button", { name: /Enregistrer/i });
    expect(saveBtn).toBeDisabled();
  });

  it("appelle set_clothing_size avec le bon payload au submit", async () => {
    const onOpenChange = vi.fn();
    const user = userEvent.setup();
    renderWithClient(
      <SizeEditDialog
        {...defaultProps}
        onOpenChange={onOpenChange}
        category="haut"
        source="override"
      />
    );

    const input = screen.getByPlaceholderText(/Ex : 52/i);
    await user.type(input, "XL");

    const saveBtn = screen.getByRole("button", { name: /Enregistrer/i });
    await user.click(saveBtn);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("set_clothing_size", {
        category: "haut",
        value: "XL",
        source: "override",
      });
    });
  });

  it("ferme le dialog (onOpenChange false) après un save réussi", async () => {
    const onOpenChange = vi.fn();
    const user = userEvent.setup();
    renderWithClient(<SizeEditDialog {...defaultProps} onOpenChange={onOpenChange} />);

    const input = screen.getByPlaceholderText(/Ex : 52/i);
    await user.type(input, "M");

    await user.click(screen.getByRole("button", { name: /Enregistrer/i }));

    await waitFor(() => {
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  it("appelle delete_clothing_size et ferme le dialog au Réinitialiser", async () => {
    const onOpenChange = vi.fn();
    const user = userEvent.setup();
    renderWithClient(
      <SizeEditDialog
        {...defaultProps}
        onOpenChange={onOpenChange}
        category="pantalon"
        current="42"
      />
    );

    await user.click(screen.getByRole("button", { name: /Réinitialiser/i }));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("delete_clothing_size", {
        category: "pantalon",
      });
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  it("affiche le label 'Ta taille' pour source=manual", () => {
    renderWithClient(<SizeEditDialog {...defaultProps} source="manual" />);

    expect(screen.getByText("Ta taille")).toBeInTheDocument();
  });

  it("affiche le label 'Ta taille réelle' pour source=override", () => {
    renderWithClient(<SizeEditDialog {...defaultProps} source="override" />);

    expect(screen.getByText("Ta taille réelle")).toBeInTheDocument();
  });
});
