import { describe, expect, it } from "vitest";
import { defined } from "@/test-utils/defined";
import { TIME_AXIS, distinctTicks, formatTick, timeAxisFor, withTimestamp } from "./timeAxis";

describe("axe du temps", () => {
  it("trie par date et garde des écarts proportionnels aux jours", () => {
    const rows = withTimestamp([
      { date: "2026-03-31", v: 3 },
      { date: "2026-01-01", v: 1 },
      { date: "2026-03-01", v: 2 },
    ]);
    expect(rows.map((r) => r.v)).toEqual([1, 2, 3]);
    const day = 86_400_000;
    const [a, b, c] = [defined(rows[0]), defined(rows[1]), defined(rows[2])];
    expect((b.ts - a.ts) / day).toBe(59);
    expect((c.ts - b.ts) / day).toBe(30);
  });

  it("ignore une date illisible", () => {
    expect(withTimestamp([{ date: "n'importe quoi" }, { date: "2026-01-01" }])).toHaveLength(1);
  });

  it("décrit un axe numérique borné aux données", () => {
    expect(TIME_AXIS).toMatchObject({
      dataKey: "ts",
      type: "number",
      scale: "time",
      domain: ["dataMin", "dataMax"],
    });
  });

  it("affiche la date du jour, sans décalage de fuseau", () => {
    const [row] = withTimestamp([{ date: "2026-03-01T23:50:00" }]);
    expect(formatTick(defined(row).ts)).toBe("1 mars 2026");
  });

  it("ne propose qu'une graduation par instant, même avec plusieurs mesures le même jour", () => {
    const rows = withTimestamp([
      { date: "2026-03-05" },
      { date: "2026-03-05" },
      { date: "2026-03-06" },
    ]);
    expect(distinctTicks(rows)).toEqual([rows[0]?.ts, rows[2]?.ts]);
    expect(timeAxisFor(rows)).toMatchObject({ dataKey: "ts", ticks: distinctTicks(rows) });
  });
});
