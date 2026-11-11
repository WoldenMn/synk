import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { Weight } from "lucide-react";
import { StatCard } from "./StatCard";

describe("StatCard", () => {
  it("affiche label et value", () => {
    render(<StatCard label="Poids" value="72.5 kg" />);
    expect(screen.getByText("Poids")).toBeInTheDocument();
    expect(screen.getByText("72.5 kg")).toBeInTheDocument();
  });

  it("affiche un sub quand fourni", () => {
    render(<StatCard label="Poids" value="72.5 kg" sub={<span>Dernier relevé</span>} />);
    expect(screen.getByText("Dernier relevé")).toBeInTheDocument();
  });

  it("rend l'icone fournie", () => {
    render(<StatCard label="Poids" value="72.5 kg" icon={<Weight data-testid="stat-icon" />} />);
    expect(screen.getByTestId("stat-icon")).toBeInTheDocument();
  });
});
