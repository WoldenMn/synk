import { describe, expect, it } from "vitest";
import importRs from "../../../src-tauri/src/commands/import.rs?raw";
import sourcesRs from "../../../src-tauri/src/commands/sources.rs?raw";
import serverRs from "../../../src-tauri/src/sources/oauth/server.rs?raw";
import backupRs from "../../../src-tauri/src/boot/post_ready.rs?raw";
import {
  BACKUP_FAILED_EVENT,
  IMPORT_PROGRESS_EVENT,
  SOURCE_EVENTS,
  SYNC_PROGRESS_EVENT,
} from "./events";

function emitted(rust: string): string[] {
  const names = [...rust.matchAll(/\.emit\(\s*"([a-z-]+)"/g)].map((m) => m[1] as string);
  return [...new Set(names)];
}

function sourceSuffixes(rust: string): string[] {
  const body = rust.slice(0, rust.indexOf("#[cfg(test)]"));
  return [...body.matchAll(/event_name\([^"]*"([a-z-]+)"\)/g)].map((m) => m[1] as string);
}

describe("contrat des évènements Rust vers TypeScript", () => {
  it("la synchro, l'import et la sauvegarde émettent le nom que l'interface écoute", () => {
    expect(emitted(sourcesRs)).toEqual([SYNC_PROGRESS_EVENT]);
    expect(emitted(importRs)).toEqual([IMPORT_PROGRESS_EVENT]);
    expect(backupRs).toContain(`"${BACKUP_FAILED_EVENT}"`);
  });

  it("le serveur de rappel émet exactement les trois évènements par source que l'interface écoute", () => {
    expect([...new Set(sourceSuffixes(serverRs))].sort()).toEqual([...SOURCE_EVENTS].sort());
  });
});
