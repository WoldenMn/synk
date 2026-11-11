import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";

vi.mock("sonner", () => ({ toast: { error: vi.fn() } }));
import { toast } from "sonner";
import { notifyError, reportError } from "./reportError";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;
const mockToast = toast.error as unknown as ReturnType<typeof vi.fn>;

describe("reportError", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockToast.mockReset();
  });

  it("toast + log_error avec code/source mappes", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const r = reportError("HTTP 401: unauthorized", { source: "sync", context: "sync" });
    expect(r.code).toBe("AUTH");
    expect(mockToast).toHaveBeenCalledOnce();
    expect(mockInvoke).toHaveBeenCalledWith("log_error", {
      source: "sync",
      code: "AUTH",
      message: r.userMessage,
      context: "sync",
    });
  });

  it("toast: false => pas de toast (cas ErrorBoundary)", async () => {
    mockInvoke.mockResolvedValue(undefined);
    reportError(new Error("boom"), { source: "boundary", toast: false });
    expect(mockToast).not.toHaveBeenCalled();
    expect(mockInvoke).toHaveBeenCalled();
  });

  it("un échec de log_error ne propage pas", async () => {
    mockInvoke.mockRejectedValue(new Error("db down"));
    expect(() => reportError("x", { source: "manual" })).not.toThrow();
  });

  it("le toast d'erreur reste dix secondes", () => {
    mockInvoke.mockResolvedValue(undefined);
    reportError("HTTP 401: unauthorized", { source: "mutation" });
    expect(mockToast).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({ duration: 10_000 })
    );
  });

  it("notifyError affiche sans journaliser", () => {
    notifyError("Sauvegarde impossible", "backup-failed");
    expect(mockToast).toHaveBeenCalledWith(
      "Sauvegarde impossible",
      expect.objectContaining({ duration: 10_000, id: "backup-failed" })
    );
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
