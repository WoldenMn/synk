import { describe, expect, it } from "vitest";
import { mergeSecondary } from "./mergeSecondary";

const row = (date: string, value: number, source = "withings") => ({ date, value, source });
const other = (day: string, value: number, source: string) => ({
  measured_at: `${day}T09:00:00Z`,
  value,
  source,
});

describe("mergeSecondary", () => {
  it("rattache le point d'une autre source à la ligne du même jour", () => {
    const merged = mergeSecondary(
      [row("2026-03-01", 80), row("2026-03-02", 79.5)],
      [other("2026-03-01", 79, "manual")]
    );

    expect(merged[0]).toMatchObject({ value: 80, other: 79, otherSource: "manual" });
    expect(merged[1]).not.toHaveProperty("other");
  });

  it("ne touche pas aux lignes quand il n'y a rien d'autre", () => {
    const rows = [row("2026-03-01", 80)];
    expect(mergeSecondary(rows, [])).toEqual(rows);
  });

  it("ignore un point secondaire dont le jour n'a pas de ligne", () => {
    const merged = mergeSecondary([row("2026-03-01", 80)], [other("2026-04-01", 70, "manual")]);
    expect(merged).toEqual([row("2026-03-01", 80)]);
  });

  it("garde le premier point secondaire d'un jour qui en porte plusieurs", () => {
    const merged = mergeSecondary(
      [row("2026-03-01", 80)],
      [other("2026-03-01", 79, "manual"), other("2026-03-01", 78, "oura")]
    );
    expect(merged[0]).toMatchObject({ other: 79, otherSource: "manual" });
  });
});
