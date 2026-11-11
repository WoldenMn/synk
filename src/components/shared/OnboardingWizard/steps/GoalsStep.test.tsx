import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { GoalsStep } from "./GoalsStep";

describe("GoalsStep", () => {
  it("enregistre poids et pas avec la clé data et les sens de l'application", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    const onComplete = vi.fn();
    const user = userEvent.setup();
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    render(
      <QueryClientProvider client={client}>
        <GoalsStep onComplete={onComplete} onSkip={vi.fn()} titleId="titre-objectifs" />
      </QueryClientProvider>
    );

    await user.type(screen.getByLabelText("Poids cible (kg)"), "70");
    await user.type(screen.getByLabelText("Pas / jour"), "10000");
    await user.click(screen.getByRole("button", { name: /Terminer/ }));

    await waitFor(() => expect(onComplete).toHaveBeenCalled());
    expect(invoke).toHaveBeenCalledWith("upsert_goal", {
      data: { metric_key: "weight", target_value: 70, direction: "decrease" },
    });
    expect(invoke).toHaveBeenCalledWith("upsert_goal", {
      data: { metric_key: "steps", target_value: 10000, direction: "increase" },
    });
  });

  it("présente ses exemples comme des exemples, pas comme des valeurs", () => {
    const client = new QueryClient();
    render(
      <QueryClientProvider client={client}>
        <GoalsStep onComplete={vi.fn()} onSkip={vi.fn()} titleId="titre-objectifs" />
      </QueryClientProvider>
    );
    expect(screen.getByLabelText("Poids cible (kg)")).toHaveAttribute("placeholder", "ex. 70");
    expect(screen.getByLabelText("Pas / jour")).toHaveAttribute("placeholder", "ex. 10000");
  });
});
