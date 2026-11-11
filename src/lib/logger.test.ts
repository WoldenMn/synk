import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

describe("logger", () => {
  let logSpy: ReturnType<typeof vi.spyOn>;
  let infoSpy: ReturnType<typeof vi.spyOn>;
  let warnSpy: ReturnType<typeof vi.spyOn>;
  let errorSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    /* eslint-disable @typescript-eslint/no-empty-function -- mocks console silencieux */
    logSpy = vi.spyOn(console, "log").mockImplementation(() => {});
    infoSpy = vi.spyOn(console, "info").mockImplementation(() => {});
    warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
    errorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    /* eslint-enable @typescript-eslint/no-empty-function */
  });

  afterEach(() => {
    logSpy.mockRestore();
    infoSpy.mockRestore();
    warnSpy.mockRestore();
    errorSpy.mockRestore();
    vi.unstubAllEnvs();
    vi.resetModules();
  });

  describe("en mode dev (DEV=true)", () => {
    beforeEach(() => {
      vi.stubEnv("DEV", true);
    });

    it("logDebug emet via console.log avec prefixe [Synk]", async () => {
      const { logDebug } = await import("./logger");
      logDebug("hello", { x: 1 });
      expect(logSpy).toHaveBeenCalledWith("[Synk]", "hello", { x: 1 });
    });

    it("logInfo emet via console.info avec prefixe [Synk]", async () => {
      const { logInfo } = await import("./logger");
      logInfo("info");
      expect(infoSpy).toHaveBeenCalledWith("[Synk]", "info");
    });
  });

  describe("en mode prod (DEV=false)", () => {
    beforeEach(() => {
      vi.stubEnv("DEV", false);
    });

    it("logDebug est silencieux", async () => {
      const { logDebug } = await import("./logger");
      logDebug("not visible");
      expect(logSpy).not.toHaveBeenCalled();
    });

    it("logInfo est silencieux", async () => {
      const { logInfo } = await import("./logger");
      logInfo("not visible");
      expect(infoSpy).not.toHaveBeenCalled();
    });
  });

  describe("toujours actif (peu importe DEV)", () => {
    it("logWarn emet via console.warn avec prefixe [Synk]", async () => {
      vi.stubEnv("DEV", false);
      const { logWarn } = await import("./logger");
      logWarn("alert", { code: 42 });
      expect(warnSpy).toHaveBeenCalledWith("[Synk]", "alert", { code: 42 });
    });

    it("logError emet via console.error avec prefixe [Synk]", async () => {
      vi.stubEnv("DEV", false);
      const { logError } = await import("./logger");
      const err = new Error("boom");
      logError("crash", err);
      expect(errorSpy).toHaveBeenCalledWith("[Synk]", "crash", err);
    });
  });
});
