import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { IntegrityBanner } from "./IntegrityBanner";

describe("IntegrityBanner", () => {
  it("annonce une base qui échoue au contrôle d'intégrité", () => {
    render(
      <MemoryRouter>
        <IntegrityBanner warning={{ Details: "row 12 missing from index i" }} />
      </MemoryRouter>
    );
    expect(screen.getByRole("alert")).toHaveTextContent("contrôle d'intégrité");
    expect(screen.getByText("row 12 missing from index i")).toBeInTheDocument();
  });

  it("ne rend rien sans avertissement", () => {
    const { container } = render(
      <MemoryRouter>
        <IntegrityBanner />
      </MemoryRouter>
    );
    expect(container).toBeEmptyDOMElement();
  });
});
