import { describe, expect, it } from "vitest";
import seedSql from "../../../src-tauri/migrations/019_sources.sql?raw";
import traceSql from "../../../src-tauri/migrations/023_traces_et_correspondances_csv.sql?raw";
import modelRs from "../../../src-tauri/src/sources/model.rs?raw";
import { DOMAIN_LABELS, SOURCE_LABELS, domainLabel, sourceLabel } from "./labels";

function seededSources(sql: string): Record<string, string> {
  const block = sql.slice(sql.indexOf("INSERT INTO sources"), sql.indexOf("INSERT OR IGNORE"));
  return rowsOf(block);
}

function rowsOf(block: string): Record<string, string> {
  const found: Record<string, string> = {};
  for (const m of block.matchAll(/\('([a-z_]+)', '([^']+)', '[a-z]+', \d+\)/g)) {
    found[m[1] as string] = m[2] as string;
  }
  return found;
}

describe("libellés des sources", () => {
  it("lit les onze sources de la graine et celle de la 023 (contrôle de l'instrument)", () => {
    expect(Object.keys(seededSources(seedSql))).toHaveLength(11);
    expect(Object.keys(rowsOf(traceSql))).toEqual(["trace"]);
  });

  it("reprend exactement les libellés des migrations 019 et 023", () => {
    expect(SOURCE_LABELS).toEqual({ ...seededSources(seedSql), ...rowsOf(traceSql) });
  });

  it("nomme chaque domaine que le contrat Rust déclare", () => {
    const block = modelRs.slice(modelRs.indexOf("pub const fn as_str(self)"));
    const rust = [...block.matchAll(/=> "([a-z_]+)"/g)].map((m) => m[1] as string);
    expect(rust).toHaveLength(9);
    expect(Object.keys(DOMAIN_LABELS).sort()).toEqual([...rust].sort());
    expect(domainLabel("sleep")).toBe("Sommeil");
    expect(domainLabel("inconnu")).toBe("inconnu");
  });

  it("rend l'identifiant d'une source inconnue", () => {
    expect(sourceLabel("withings")).toBe("Withings");
    expect(sourceLabel("inconnue")).toBe("inconnue");
  });

  it("nomme l'étape des analyses, qui n'est pas une source", () => {
    expect(sourceLabel("analyses")).toBe("Analyses");
    expect(Object.keys(SOURCE_LABELS)).not.toContain("analyses");
  });
});
