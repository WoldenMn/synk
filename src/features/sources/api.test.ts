import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  cancelImport,
  getInvalidatedSecrets,
  importSourceFile,
  listSources,
  sourceDisconnect,
  sourceSetCredentials,
  sourceStartAuth,
  syncAllSources,
} from "./api";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

describe("sources/api", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("listSources lit le registre via list_sources", async () => {
    const fake = [{ id: "withings", label: "Withings", connected: true }];
    mockInvoke.mockResolvedValue(fake);

    expect(await listSources()).toEqual(fake);
    expect(mockInvoke).toHaveBeenCalledWith("list_sources");
  });

  it("sourceStartAuth nomme la source", async () => {
    mockInvoke.mockResolvedValue("https://account.withings.com/oauth2_user/authorize2?...");

    const url = await sourceStartAuth("withings");

    expect(mockInvoke).toHaveBeenCalledWith("source_start_auth", { source: "withings" });
    expect(url).toContain("account.withings.com");
  });

  it("sourceDisconnect nomme la source et rend le compte rendu de la révocation", async () => {
    mockInvoke.mockResolvedValue({ revoked: true, warning: null });

    const report = await sourceDisconnect("oura");

    expect(mockInvoke).toHaveBeenCalledWith("source_disconnect", { source: "oura" });
    expect(report).toEqual({ revoked: true, warning: null });
  });

  it("sourceSetCredentials passe la source et les deux identifiants", async () => {
    mockInvoke.mockResolvedValue(undefined);

    await sourceSetCredentials("withings", "cid", "secret");

    expect(mockInvoke).toHaveBeenCalledWith("source_set_credentials", {
      source: "withings",
      clientId: "cid",
      clientSecret: "secret",
    });
  });

  it("syncAllSources rend une issue par source synchronisée", async () => {
    const fake = [
      { source: "withings", label: "Withings", count: 10, errors: [], failed: false },
      { source: "oura", label: "Oura", count: 0, errors: ["Oura : refus"], failed: true },
    ];
    mockInvoke.mockResolvedValue(fake);

    const result = await syncAllSources();

    expect(mockInvoke).toHaveBeenCalledWith("sync_all_sources");
    expect(result).toEqual(fake);
  });

  it("getInvalidatedSecrets invoke get_invalidated_secrets", async () => {
    const fake = {
      sources: [{ source: "withings", label: "Withings", tokens: true, credentials: false }],
    };
    mockInvoke.mockResolvedValue(fake);

    expect(await getInvalidatedSecrets()).toEqual(fake);
    expect(mockInvoke).toHaveBeenCalledWith("get_invalidated_secrets");
  });

  it("importSourceFile nomme la source et le fichier choisi", async () => {
    mockInvoke.mockResolvedValue({ cancelled: false });

    await importSourceFile("apple_health", "/donnees/export.zip");

    expect(mockInvoke).toHaveBeenCalledWith("import_source_file", {
      source: "apple_health",
      path: "/donnees/export.zip",
    });
  });

  it("cancelImport demande l'arrêt de l'import en cours", async () => {
    mockInvoke.mockResolvedValue(true);

    expect(await cancelImport()).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith("cancel_import");
  });
});
