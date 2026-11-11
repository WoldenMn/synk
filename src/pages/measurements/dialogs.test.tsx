import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";
import { GoalDialog } from "./GoalDialog";
import { NewTypeDialog } from "./NewTypeDialog";

describe("boîtes de dialogue de Mensurations", () => {
  it("Nouveau type : chaque champ est trouvé par son libellé", () => {
    render(
      <QueryClientProvider client={new QueryClient()}>
        <NewTypeDialog newTypeDialogOpen setNewTypeDialogOpen={vi.fn()} />
      </QueryClientProvider>
    );
    expect(screen.getByLabelText(/Nom/)).toBeInTheDocument();
    expect(screen.getByLabelText(/Unité/)).toBeInTheDocument();
  });

  it("Objectif : chaque champ est trouvé par son libellé", () => {
    render(
      <GoalDialog
        goalDialogOpen
        setGoalDialogOpen={vi.fn()}
        goalValue=""
        setGoalValue={vi.fn()}
        goalDirection="decrease"
        setGoalDirection={vi.fn()}
        handleDeleteGoal={vi.fn()}
        handleSaveGoal={vi.fn()}
      />
    );
    expect(screen.getByLabelText(/Valeur cible/)).toBeInTheDocument();
    expect(screen.getByLabelText(/Direction/)).toBeInTheDocument();
  });
});
