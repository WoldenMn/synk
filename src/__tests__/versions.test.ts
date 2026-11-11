import { describe, expect, it } from "vitest";
import pkg from "../../package.json?raw";
import lock from "../../package-lock.json?raw";
import cargoToml from "../../src-tauri/Cargo.toml?raw";
import cargoLock from "../../src-tauri/Cargo.lock?raw";
import tauriConf from "../../src-tauri/tauri.conf.json?raw";
import readme from "../../README.md?raw";
import { disagreements, readVersions, VERSION_FILES } from "../../scripts/version-lib.mjs";

const FILES: Record<string, string> = {
  "package.json": pkg,
  "package-lock.json": lock,
  "src-tauri/Cargo.toml": cargoToml,
  "src-tauri/Cargo.lock": cargoLock,
  "src-tauri/tauri.conf.json": tauriConf,
  "README.md": readme,
};

describe("version de Synk", () => {
  it("est la meme dans les cinq fichiers et le README", () => {
    expect(Object.keys(FILES).sort()).toEqual([...VERSION_FILES].sort());
    const versions = readVersions((p: string) => FILES[p] ?? "");
    expect(disagreements(versions), JSON.stringify(versions)).toEqual({});
  });
});
