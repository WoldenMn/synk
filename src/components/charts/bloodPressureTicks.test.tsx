import { afterEach, describe, expect, it, vi } from "vitest";
import { cloneElement, isValidElement } from "react";
import type { ReactElement } from "react";
import { render } from "@testing-library/react";
import { BloodPressureChart } from "./BloodPressureChart";
import type { BloodPressure } from "@/features/bloodPressure/types";

vi.mock("recharts", async () => {
  const actual = await vi.importActual<typeof import("recharts")>("recharts");
  return {
    ...actual,
    ResponsiveContainer: ({ children }: { children: ReactElement }) =>
      isValidElement(children)
        ? cloneElement(children, { width: 800, height: 300 } as Partial<typeof children.props>)
        : children,
  };
});

function reading(id: number, measuredAt: string): BloodPressure {
  return {
    id,
    measured_at: measuredAt,
    systolic: 120 + (id % 7),
    diastolic: 80 + (id % 5),
    heart_rate: 60 + (id % 9),
    source: "csv",
  } as BloodPressure;
}

function duplicateKeyWarnings(spy: ReturnType<typeof vi.spyOn>): string[] {
  return spy.mock.calls
    .map((call: unknown[]) => call.map(String).join(" "))
    .filter((text: string) => /same key/i.test(text));
}

describe("graduations du graphique de tension", () => {
  afterEach(() => vi.restoreAllMocks());

  it.each([
    [
      "toutes les mesures le même jour",
      Array.from({ length: 12 }, (_, i) => reading(i + 1, `2026-03-05T0${i % 9}:00:00Z`)),
    ],
    [
      "plusieurs mesures par jour sur plusieurs jours",
      [
        reading(1, "2026-03-05T07:00:00Z"),
        reading(2, "2026-03-05T19:00:00Z"),
        reading(3, "2026-03-06T07:00:00Z"),
        reading(4, "2026-03-06T08:00:00Z"),
        reading(5, "2026-03-09T07:00:00Z"),
      ],
    ],
    ["une seule mesure", [reading(1, "2026-03-05T07:00:00Z")]],
  ])("ne produit aucune clé en double : %s", (_label, rows) => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => undefined);
    render(<BloodPressureChart data={rows} />);
    expect(duplicateKeyWarnings(spy)).toEqual([]);
  });
});
