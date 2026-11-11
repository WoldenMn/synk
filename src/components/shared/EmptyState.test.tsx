import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { forwardRef } from "react";
import { EmptyState } from "./EmptyState";

// Reproduit la signature des Lucide icons : forwardRef avec $$typeof.
const LucideLikeIcon = forwardRef<SVGSVGElement, { className?: string }>(
  function LucideLikeIcon(props, ref) {
    return <svg ref={ref} className={props.className} data-testid="lucide-icon" />;
  }
);

function FunctionIcon({ className }: { className?: string }) {
  return <span data-testid="function-icon" className={className} />;
}

describe("EmptyState", () => {
  it("renders title and description", () => {
    render(<EmptyState icon={FunctionIcon} title="Aucune donnée" description="Hello" />);
    expect(screen.getByText("Aucune donnée")).toBeInTheDocument();
    expect(screen.getByText("Hello")).toBeInTheDocument();
  });

  it("instancie une function component passee en prop icon", () => {
    render(<EmptyState icon={FunctionIcon} title="t" description="d" />);
    expect(screen.getByTestId("function-icon")).toBeInTheDocument();
  });

  // Regression V1.1.2 : avant le fix, EmptyState testait seulement
  // typeof icon === "function" -> echouait sur les forwardRef Lucide -> React #31.
  it("instancie un forwardRef (Lucide-like) passe en prop icon", () => {
    render(<EmptyState icon={LucideLikeIcon} title="t" description="d" />);
    expect(screen.getByTestId("lucide-icon")).toBeInTheDocument();
  });

  it("rend un ReactElement déjà instancie tel quel", () => {
    render(<EmptyState icon={<div data-testid="raw-element">x</div>} title="t" description="d" />);
    expect(screen.getByTestId("raw-element")).toBeInTheDocument();
  });

  it("rend l'action optionnelle", () => {
    render(
      <EmptyState icon={FunctionIcon} title="t" description="d" action={<button>Action</button>} />
    );
    expect(screen.getByRole("button", { name: /action/i })).toBeInTheDocument();
  });
});
