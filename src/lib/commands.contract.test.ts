import { describe, expect, it } from "vitest";
import libRs from "../../src-tauri/src/lib.rs?raw";
import { defined } from "@/test-utils/defined";
import {
  closing,
  registeredCommands,
  rustParams,
  splitTopLevel,
  type RustParam,
} from "./commandsCatalog";

type Call = { cmd: string; keys: string[] | null };

const rustSources = import.meta.glob("../../src-tauri/src/**/*.rs", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

const frontSources = Object.fromEntries(
  Object.entries(
    import.meta.glob("../**/*.{ts,tsx}", {
      query: "?raw",
      import: "default",
      eager: true,
    }) as Record<string, string>
  ).filter(([path]) => !/\.test\.tsx?$/.test(path) && !path.includes("test-setup"))
);

const NO_CALLER: Record<string, string> = {};

export function invokeCalls(source: string): Call[] {
  const calls: Call[] = [];
  for (const m of source.matchAll(/\binvoke(?:<[^>(]*>)?\(\s*"([a-z0-9_]+)"\s*(,)?/g)) {
    const cmd = defined(m[1]);
    if (!m[2]) {
      calls.push({ cmd, keys: [] });
      continue;
    }
    const brace = source.indexOf("{", (m.index ?? 0) + m[0].length);
    const between = source.slice((m.index ?? 0) + m[0].length, brace).trim();
    if (between !== "") {
      calls.push({ cmd, keys: null });
      continue;
    }
    const entries = splitTopLevel(source.slice(brace + 1, closing(source, brace, "{", "}")));
    calls.push({
      cmd,
      keys: entries.some((e) => e.startsWith("..."))
        ? null
        : entries.map((e) => defined(e.split(":")[0]).trim().replace(/["']/g, "")),
    });
  }
  return calls;
}

const snake = (s: string) => s.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`);

export function contractProblems(
  registered: string[],
  params: Map<string, RustParam[]>,
  calls: Call[]
): string[] {
  const problems: string[] = [];
  for (const { cmd, keys } of calls) {
    if (!registered.includes(cmd))
      problems.push(`${cmd} : appelee par le front, absente de generate_handler!`);
    const expected = params.get(cmd);
    if (!expected || keys === null) continue;
    for (const key of keys) {
      if (!expected.some((p) => p.name === snake(key))) {
        problems.push(
          `${cmd} : cle inconnue ${key} (parametres : ${expected.map((p) => p.name).join(", ") || "aucun"})`
        );
      }
    }
    for (const p of expected.filter((p) => !p.optional)) {
      if (!keys.some((k) => snake(k) === p.name))
        problems.push(`${cmd} : parametre obligatoire ${p.name} non fourni`);
    }
  }
  const called = new Set(calls.map((c) => c.cmd));
  for (const cmd of registered) {
    if (!called.has(cmd) && !(cmd in NO_CALLER))
      problems.push(`${cmd} : enregistree sans appelant`);
  }
  for (const cmd of params.keys()) {
    if (!registered.includes(cmd))
      problems.push(`${cmd} : declaree #[tauri::command] sans enregistrement`);
  }
  return problems;
}

const registered = registeredCommands(libRs);
const params = rustParams(rustSources);
const calls = Object.values(frontSources).flatMap(invokeCalls);

describe("contrat IPC front -> Rust", () => {
  it("lit les deux côtés (contrôle de l'instrument)", () => {
    expect(registered.length).toBeGreaterThan(90);
    expect(params.get("upsert_goal")?.map((p) => p.name)).toEqual(["data"]);
    expect(calls.length).toBeGreaterThan(90);
  });

  it("lit les paramètres de chaque commande enregistrée", () => {
    expect(registered.filter((cmd) => !params.has(cmd))).toEqual([]);
  });

  it("n'excuse que des commandes enregistrées et réellement sans appelant", () => {
    const called = new Set(calls.map((c) => c.cmd));
    for (const cmd of Object.keys(NO_CALLER)) {
      expect(registered).toContain(cmd);
      expect(called.has(cmd)).toBe(false);
    }
  });

  it("voit l'ancienne erreur de GoalsStep (cas connu mauvais)", () => {
    const replay = invokeCalls(`invoke("upsert_goal", { input: payload })`);
    expect(contractProblems(registered, params, replay)).toContain(
      "upsert_goal : cle inconnue input (parametres : data)"
    );
  });

  it("lit une clé abrégée, un générique et un appel sans argument", () => {
    expect(invokeCalls(`invoke<Foo[]>("get_x", { limit, codeFilter: a ?? null })`)).toEqual([
      { cmd: "get_x", keys: ["limit", "codeFilter"] },
    ]);
    expect(invokeCalls(`invoke("mark_insights_seen")`)).toEqual([
      { cmd: "mark_insights_seen", keys: [] },
    ]);
  });

  it("le front et le Rust s'accordent", () => {
    expect(contractProblems(registered, params, calls)).toEqual([]);
  });
});
