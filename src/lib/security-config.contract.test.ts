import { describe, expect, it } from "vitest";
import capabilitiesRaw from "../../src-tauri/capabilities/default.json?raw";
import tauriConfRaw from "../../src-tauri/tauri.conf.json?raw";

const permissions: unknown[] = JSON.parse(capabilitiesRaw).permissions;
const ids = permissions.map((p) =>
  typeof p === "string" ? p : (p as { identifier: string }).identifier
);
const csp: string = JSON.parse(tauriConfRaw).app.security.csp;
const directive = (name: string) =>
  csp
    .split(";")
    .map((d) => d.trim())
    .find((d) => d.startsWith(`${name} `)) ?? "";

describe("permissions de la fenêtre", () => {
  it("n'accorde du plugin dialog que l'ouverture de fichier", () => {
    expect(ids.filter((id) => id.startsWith("dialog:"))).toEqual(["dialog:allow-open"]);
  });

  it("n'ouvre d'URL que sur des hotes https nommes, jamais un hote generique", () => {
    const entry = permissions.find(
      (p) =>
        typeof p === "object" &&
        (p as { identifier: string }).identifier === "opener:allow-open-url"
    ) as { allow: { url: string }[] };
    expect(entry.allow.length).toBeGreaterThan(0);
    for (const { url } of entry.allow) {
      expect(url, url).toMatch(/^https:\/\/[a-z0-9-]+(\.[a-z0-9-]+)+\/\*$/);
    }
  });

  it("n'accorde ni shell ni lancement de programme", () => {
    expect(ids.some((id) => id.startsWith("shell:"))).toBe(false);
    expect(ids).not.toContain("opener:allow-open-path");
  });
});

describe("CSP", () => {
  it("laisse passer le canal IPC de Tauri", () => {
    expect(directive("connect-src")).toContain("ipc:");
    expect(directive("connect-src")).toContain("http://ipc.localhost");
  });

  it("ne laisse la WebView joindre aucune origine distante", () => {
    expect(directive("connect-src")).not.toMatch(/https:/);
  });

  it("ne charge de script et de ressource que de l'application", () => {
    expect(directive("default-src")).toBe("default-src 'self'");
    expect(directive("script-src")).toBe("script-src 'self'");
  });

  it("n'autorise ni script en ligne, ni eval, ni caractere generique de source", () => {
    expect(csp).not.toMatch(/unsafe-eval/);
    expect(directive("script-src")).not.toMatch(/unsafe-inline|\*|https?:/);
    expect(directive("default-src")).not.toMatch(/\*|https?:/);
  });

  it("ferme object-src, base-uri et form-action", () => {
    expect(directive("object-src")).toBe("object-src 'none'");
    expect(directive("base-uri")).toBe("base-uri 'self'");
    expect(directive("form-action")).toBe("form-action 'self'");
  });
});
