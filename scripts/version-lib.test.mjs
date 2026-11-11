import { describe, expect, it } from "vitest";
import { bumpText, disagreements, isSemver, readVersions, VERSION_FILES } from "./version-lib.mjs";

const FILES = {
  "package.json": '{\n  "name": "synk",\n  "version": "2.15.0",\n  "type": "module"\n}\n',
  "package-lock.json":
    '{\n  "name": "synk",\n  "version": "2.15.0",\n  "packages": {\n    "": {\n      "name": "synk",\n      "version": "2.15.0",\n      "dependencies": {\n        "zod": "^3.0.0"\n      }\n    },\n    "node_modules/zod": {\n      "version": "3.1.0"\n    }\n  }\n}\n',
  "src-tauri/Cargo.toml":
    '[package]\nname = "synk"\nversion = "2.15.0"\n\n[dependencies]\nserde = { version = "1.0" }\n',
  "src-tauri/Cargo.lock":
    '[[package]]\nname = "syn"\nversion = "2.0.1"\n\n[[package]]\nname = "synk"\nversion = "2.15.0"\n',
  "src-tauri/tauri.conf.json": '{\n  "productName": "Synk",\n  "version": "2.15.0"\n}\n',
  "README.md":
    "# Synk\n\nVersion courante : 2.15.0 (`package.json`).\n\nVersion 1.0.0 de la base.\n",
};

const at = (version) =>
  Object.fromEntries(Object.entries(FILES).map(([p, t]) => [p, bumpText(p, t, version)]));

describe("version de Synk", () => {
  it("couvre cinq fichiers de version et le README", () => {
    expect(VERSION_FILES).toHaveLength(6);
    expect(VERSION_FILES).toContain("README.md");
  });

  it("lit les sept emplacements", () => {
    const versions = readVersions((p) => FILES[p]);
    expect(Object.keys(versions)).toHaveLength(7);
    expect(new Set(Object.values(versions))).toEqual(new Set(["2.15.0"]));
  });

  it("monte chaque emplacement sans toucher aux autres versions", () => {
    const bumped = at("2.16.0");
    expect(new Set(Object.values(readVersions((p) => bumped[p])))).toEqual(new Set(["2.16.0"]));
    expect(bumped["src-tauri/Cargo.toml"]).toContain('serde = { version = "1.0" }');
    expect(bumped["src-tauri/Cargo.lock"]).toContain('name = "syn"\nversion = "2.0.1"');
    expect(bumped["package-lock.json"]).toContain(
      '"node_modules/zod": {\n      "version": "3.1.0"'
    );
    expect(bumped["README.md"]).toContain("Version 1.0.0 de la base.");
  });

  it("remonter la meme version ne change rien", () => {
    const again = at("2.15.0");
    for (const path of VERSION_FILES) expect(again[path]).toBe(FILES[path]);
  });

  it("refuse une version mal formee", () => {
    expect(isSemver("2.16.0")).toBe(true);
    expect(isSemver("2.16")).toBe(false);
    expect(isSemver("v2.16.0")).toBe(false);
    expect(() => bumpText("package.json", FILES["package.json"], "2.16")).toThrow(/mal formee/);
  });

  it("refuse un fichier ou l'emplacement de la version est introuvable", () => {
    expect(() => bumpText("README.md", "# Synk\n", "2.16.0")).toThrow(/README\.md/);
    expect(() => bumpText("inconnu.txt", "x", "2.16.0")).toThrow(/inconnu/);
  });

  it("nomme les emplacements en desaccord", () => {
    const mixed = {
      ...FILES,
      "src-tauri/tauri.conf.json": bumpText(
        "src-tauri/tauri.conf.json",
        FILES["src-tauri/tauri.conf.json"],
        "2.16.0"
      ),
    };
    const versions = readVersions((p) => mixed[p]);
    expect(disagreements(versions)).toEqual({ "tauri.conf.json": "2.16.0" });
    expect(disagreements(readVersions((p) => FILES[p]))).toEqual({});
  });

  it("signale un emplacement illisible comme une absence", () => {
    const broken = { ...FILES, "src-tauri/Cargo.lock": '[[package]]\nname = "syn"\n' };
    const versions = readVersions((p) => broken[p]);
    expect(versions["Cargo.lock"]).toBeUndefined();
    expect(Object.keys(disagreements(versions))).toContain("Cargo.lock");
  });
});
