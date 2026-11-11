import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { StepLayout } from "./StepLayout";

describe("StepLayout", () => {
  it("annonce la progression de l'assistant", () => {
    render(
      <StepLayout step={2} totalSteps={4} titleId="t">
        <h2 id="t">Titre</h2>
      </StepLayout>
    );
    expect(screen.getByText("Étape 2 sur 4")).toBeInTheDocument();
  });
});
