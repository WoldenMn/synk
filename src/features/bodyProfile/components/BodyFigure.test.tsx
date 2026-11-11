import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { BodyFigure } from "./BodyFigure";
import { PLATE_HEIGHT, PLATE_WIDTH } from "../anchorGeometry";
import type { BodyMeasurement } from "../types";
import { defined } from "@/test-utils/defined";

function measurement(slug: string, y: number): BodyMeasurement {
  return {
    slug,
    label: slug,
    stepLabel: slug,
    zone: "global",
    nature: "height",
    side: "none",
    pairKey: null,
    order: 1,
    isKey: true,
    min: 0,
    max: 250,
    guidance: "",
    anchor: { view: "front", x: 50, y, labelSide: "left" },
  };
}

describe("BodyFigure", () => {
  // Le calque doit partager le repere de l'image. Avec un viewBox
  // en pourcentages et `preserveAspectRatio="none"`, il se dimensionnait sur son
  // conteneur etire par la grille : les pastilles tombaient sous la silhouette,
  // ecrasees en ellipses.
  it("calque le repere de la planche", () => {
    const { container } = render(<BodyFigure view="front" annotations={[]} />);
    const svg = container.querySelector("svg");
    expect(svg?.getAttribute("viewBox")).toBe(`0 0 ${PLATE_WIDTH} ${PLATE_HEIGHT}`);
    expect(svg?.getAttribute("preserveAspectRatio")).toBeNull();
  });

  it("place l'ancre du crane en haut de la planche", () => {
    const { container } = render(
      <BodyFigure view="front" annotations={[measurement("height", 97)]} />
    );
    const cercle = container.querySelector("circle");
    expect(Number(cercle?.getAttribute("cy"))).toBeCloseTo(21, 1);
  });

  it("place une ancre basse sous une ancre haute", () => {
    const { container } = render(
      <BodyFigure view="front" annotations={[measurement("haut", 90), measurement("bas", 20)]} />
    );
    const cys = Array.from(container.querySelectorAll("circle")).map((c) =>
      Number(c.getAttribute("cy"))
    );
    const haut = defined(cys[0]);
    const bas = defined(cys[1]);
    expect(bas).toBeGreaterThan(haut);
  });

  it("n'affiche que les ancres de la vue demandee", () => {
    const dos = measurement("dos", 50);
    dos.anchor = { view: "back", x: 50, y: 50, labelSide: "left" };
    const { container } = render(
      <BodyFigure view="front" annotations={[measurement("face", 50), dos]} />
    );
    expect(container.querySelectorAll("circle")).toHaveLength(1);
  });
});
