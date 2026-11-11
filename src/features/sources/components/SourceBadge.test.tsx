import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SourceBadge } from "./SourceBadge";

describe("SourceBadge", () => {
  it("nomme la source par son libellé", () => {
    render(<SourceBadge source="manual" />);
    expect(screen.getByText("Saisie manuelle")).toBeInTheDocument();
  });

  it("garde l'identifiant d'une source hors graine", () => {
    render(<SourceBadge source="inconnue" />);
    expect(screen.getByText("inconnue")).toBeInTheDocument();
  });
});
