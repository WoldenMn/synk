import { describe, it, expect, vi, beforeEach } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactElement, ReactNode } from "react";
import { createManualForm } from "./createManualForm";
import { defined } from "@/test-utils/defined";

function renderWithClient(ui: ReactElement) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(<QueryClientProvider client={client}>{ui}</QueryClientProvider>);
}

interface FakeInput extends Record<string, unknown> {
  measured_at: string;
  weight: number;
  notes: string | null;
}

function makeForm(overrides: {
  submit?: (data: FakeInput) => Promise<unknown>;
  onSuccess?: () => void;
  defaultValues?: Partial<FakeInput>;
  withNumericMinMax?: boolean;
}) {
  const submit = overrides.submit ?? vi.fn().mockResolvedValue(undefined);
  return createManualForm<FakeInput>({
    fields: [
      { name: "measured_at", label: "Date", type: "datetime", required: true },
      {
        name: "weight",
        label: "Poids",
        type: "decimal",
        unit: "kg",
        decimals: 2,
        min: overrides.withNumericMinMax ? 20 : undefined,
        max: overrides.withNumericMinMax ? 300 : undefined,
        required: true,
      },
      { name: "notes", label: "Notes", type: "text" },
    ],
    submit,
    defaultValues: overrides.defaultValues,
    successMessage: "Pesée enregistrée",
    onSuccess: overrides.onSuccess,
  });
}

describe("createManualForm", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("rend tous les champs avec leurs labels", () => {
    const Form = makeForm({});
    renderWithClient(<Form open={true} onOpenChange={vi.fn()} />);

    expect(screen.getByLabelText(/Date/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Poids/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Notes/i)).toBeInTheDocument();
  });

  it("soumet les valeurs typees au handler submit", async () => {
    const submit = vi.fn().mockResolvedValue(undefined);
    const Form = makeForm({ submit });
    const user = userEvent.setup();
    renderWithClient(<Form open={true} onOpenChange={vi.fn()} />);

    const dateInput = screen.getByLabelText(/Date/i) as HTMLInputElement;
    const weightInput = screen.getByLabelText(/Poids/i) as HTMLInputElement;
    const notesInput = screen.getByLabelText(/Notes/i) as HTMLInputElement;

    await user.clear(dateInput);
    await user.type(dateInput, "2026-04-29T10:00");
    await user.type(weightInput, "75.5");
    await user.type(notesInput, "test");

    const submitBtn = screen.getByRole("button", { name: /Enregistrer/i });
    await user.click(submitBtn);

    await waitFor(() => {
      expect(submit).toHaveBeenCalledTimes(1);
    });
    const arg = defined(submit.mock.calls[0])[0] as FakeInput;
    expect(typeof arg.weight).toBe("number");
    expect(arg.weight).toBeCloseTo(75.5);
    expect(arg.notes).toBe("test");
    expect(typeof arg.measured_at).toBe("string");
  });

  it("empeche submit si un champ required est vide", async () => {
    const submit = vi.fn().mockResolvedValue(undefined);
    const Form = makeForm({ submit });
    const user = userEvent.setup();
    renderWithClient(<Form open={true} onOpenChange={vi.fn()} />);

    const submitBtn = screen.getByRole("button", { name: /Enregistrer/i });
    await user.click(submitBtn);

    expect(submit).not.toHaveBeenCalled();
  });

  it("appelle onSuccess apres succès du submit", async () => {
    const submit = vi.fn().mockResolvedValue(undefined);
    const onSuccess = vi.fn();
    const Form = makeForm({ submit, onSuccess });
    const user = userEvent.setup();
    renderWithClient(<Form open={true} onOpenChange={vi.fn()} />);

    const dateInput = screen.getByLabelText(/Date/i);
    await user.clear(dateInput);
    await user.type(dateInput, "2026-04-29T10:00");
    await user.type(screen.getByLabelText(/Poids/i), "75");

    const submitBtn = screen.getByRole("button", { name: /Enregistrer/i });
    await user.click(submitBtn);

    await waitFor(() => {
      expect(onSuccess).toHaveBeenCalledTimes(1);
    });
  });

  it("affiche un message d'erreur si submit reject", async () => {
    const submit = vi.fn().mockRejectedValue(new Error("Erreur réseau"));
    const Form = makeForm({ submit });
    const user = userEvent.setup();
    renderWithClient(<Form open={true} onOpenChange={vi.fn()} />);

    const dateInput = screen.getByLabelText(/Date/i);
    await user.clear(dateInput);
    await user.type(dateInput, "2026-04-29T10:00");
    await user.type(screen.getByLabelText(/Poids/i), "75");

    const submitBtn = screen.getByRole("button", { name: /Enregistrer/i });
    await user.click(submitBtn);

    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeInTheDocument();
    });
  });

  it("respecte min/max sur un champ numeric", () => {
    const Form = makeForm({ withNumericMinMax: true });
    renderWithClient(<Form open={true} onOpenChange={vi.fn()} />);

    const weightInput = screen.getByLabelText(/Poids/i) as HTMLInputElement;
    expect(weightInput.min).toBe("20");
    expect(weightInput.max).toBe("300");
  });

  it("applique defaultValues au mount", () => {
    const Form = makeForm({
      defaultValues: { weight: 80, notes: "default note" },
    });
    renderWithClient(<Form open={true} onOpenChange={vi.fn()} />);

    const weightInput = screen.getByLabelText(/Poids/i) as HTMLInputElement;
    const notesInput = screen.getByLabelText(/Notes/i) as HTMLInputElement;

    expect(weightInput.value).toBe("80");
    expect(notesInput.value).toBe("default note");
  });
  it("propose l'heure de l'ouverture, pas celle du montage", () => {
    vi.useFakeTimers({ toFake: ["Date"] });
    try {
      vi.setSystemTime(new Date(2026, 9, 3, 9, 0));
      const Form = makeForm({});
      const client = new QueryClient();
      const wrapper = ({ children }: { children: ReactNode }) => (
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
      );
      const { rerender } = render(<Form open={false} onOpenChange={vi.fn()} />, { wrapper });

      vi.setSystemTime(new Date(2026, 9, 3, 14, 5));
      rerender(<Form open={true} onOpenChange={vi.fn()} />);

      expect(screen.getByLabelText(/Date/i)).toHaveValue("2026-10-03T14:05");
    } finally {
      vi.useRealTimers();
    }
  });

  it("marque chaque champ invalide, lie son message et focalise le premier", async () => {
    const TwoFields = createManualForm<{ weight: number; note: string }>({
      title: "Essai",
      fields: [
        {
          name: "weight",
          label: "Poids",
          type: "decimal",
          unit: "kg",
          required: true,
          min: 1,
          max: 400,
        },
        { name: "note", label: "Note", type: "text", required: true },
      ],
      submit: vi.fn(),
    });
    renderWithClient(<TwoFields open onOpenChange={vi.fn()} />);
    const form = defined(screen.getByRole("dialog").querySelector("form"));
    fireEvent.submit(form);
    const weight = await screen.findByLabelText(/Poids/);
    const note = screen.getByLabelText(/Note/);
    expect(weight).toHaveAttribute("aria-invalid", "true");
    expect(weight).toHaveAccessibleDescription("Poids est requis");
    expect(note).toHaveAttribute("aria-invalid", "true");
    expect(note).toHaveAccessibleDescription("Note est requis");
    expect(weight).toHaveFocus();
  });
});
