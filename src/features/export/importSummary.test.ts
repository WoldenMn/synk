import { describe, expect, it } from "vitest";
import tablesRs from "../../../src-tauri/src/commands/export/json/tables.rs?raw";
import { describePreview } from "./importSummary";

const KEYS = [...tablesRs.matchAll(/^\s+key: "(\w+)"/gm)].map((m) => m[1] ?? "");

describe("describePreview", () => {
  it("reads the table keys of the export from tables.rs", () => {
    expect(KEYS.length).toBeGreaterThan(15);
    expect(KEYS).toContain("body_measures");
  });

  it("gives every exported table a readable label", () => {
    const text = describePreview(KEYS.map((key) => ({ key, rows: 2 })));
    for (const key of KEYS) {
      expect(text, key).not.toContain(key);
    }
  });

  it("falls back on the raw key for a table it does not know", () => {
    expect(describePreview([{ key: "inconnue_x", rows: 2 }])).toContain("2 inconnue_x");
  });
});
