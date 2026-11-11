import { render } from "@testing-library/react";
import { Scale } from "lucide-react";
import { describe, expect, it } from "vitest";

describe("icônes", () => {
  it("restent décoratives par défaut", () => {
    const { container } = render(<Scale className="h-4 w-4" />);
    expect(container.querySelector("svg")).toHaveAttribute("aria-hidden", "true");
  });
});
