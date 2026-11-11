import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useAppReady } from "./useAppReady";

type Listener<T = unknown> = (event: { payload: T }) => void;
const listeners = new Map<string, Listener<unknown>>();

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((name: string, cb: Listener<unknown>) => {
    listeners.set(name, cb);
    return Promise.resolve(() => listeners.delete(name));
  }),
}));

beforeEach(() => {
  listeners.clear();
  vi.mocked(invoke).mockReset();
  // Default : get_boot_state retourne "loading" -> on attend les events
  vi.mocked(invoke).mockResolvedValue({ status: "loading" });
});

describe("useAppReady", () => {
  it("starts in loading state", async () => {
    const { result } = renderHook(() => useAppReady());
    await waitFor(() => expect(listeners.has("app-ready")).toBe(true));
    expect(result.current.status).toBe("loading");
  });

  it("transitions to ready on app-ready event", async () => {
    const { result } = renderHook(() => useAppReady());
    await waitFor(() => expect(listeners.has("app-ready")).toBe(true));
    act(() => listeners.get("app-ready")!({ payload: undefined }));
    await waitFor(() => expect(result.current.status).toBe("ready"));
  });

  it("transitions to error on app-error event with DbBusyAfterRetry payload", async () => {
    const { result } = renderHook(() => useAppReady());
    await waitFor(() => expect(listeners.has("app-error")).toBe(true));
    act(() =>
      listeners.get("app-error")!({
        payload: { kind: "DbBusyAfterRetry", attempts: 3 },
      })
    );
    await waitFor(() => expect(result.current.status).toBe("error"));
    if (result.current.status === "error") {
      expect(result.current.error.kind).toBe("DbBusyAfterRetry");
    }
  });

  it("transitions to error on MigrationFailed payload", async () => {
    const { result } = renderHook(() => useAppReady());
    await waitFor(() => expect(listeners.has("app-error")).toBe(true));
    act(() =>
      listeners.get("app-error")!({
        payload: { kind: "MigrationFailed", version: 6, message: "table missing" },
      })
    );
    await waitFor(() => expect(result.current.status).toBe("error"));
    if (result.current.status === "error" && result.current.error.kind === "MigrationFailed") {
      expect(result.current.error.version).toBe(6);
    }
  });

  it("attaches integrityWarning to ready state if event arrives before app-ready", async () => {
    const { result } = renderHook(() => useAppReady());
    await waitFor(() => expect(listeners.has("app-integrity-warning")).toBe(true));
    act(() =>
      listeners.get("app-integrity-warning")!({
        payload: { Details: "row corrupted page 3" },
      })
    );
    act(() => listeners.get("app-ready")!({ payload: undefined }));
    await waitFor(() => expect(result.current.status).toBe("ready"));
    if (result.current.status === "ready") {
      expect(result.current.integrityWarning?.Details).toBe("row corrupted page 3");
    }
  });

  it("cleans up listeners on unmount", async () => {
    const { unmount } = renderHook(() => useAppReady());
    await waitFor(() => expect(listeners.has("app-ready")).toBe(true));
    unmount();
    await waitFor(() => expect(listeners.has("app-ready")).toBe(false));
  });

  // Regression test V1.1.1 : race condition fire-and-forget
  it("rattrape ready si get_boot_state retourne ready (event manque avant mount)", async () => {
    vi.mocked(invoke).mockResolvedValue({ status: "ready", integrity_warning: null });
    const { result } = renderHook(() => useAppReady());
    await waitFor(() => expect(result.current.status).toBe("ready"));
    expect(invoke).toHaveBeenCalledWith("get_boot_state");
  });

  it("rattrape error si get_boot_state retourne failed (event manque avant mount)", async () => {
    vi.mocked(invoke).mockResolvedValue({
      status: "failed",
      error: { kind: "DbLocked", message: "io error" },
    });
    const { result } = renderHook(() => useAppReady());
    await waitFor(() => expect(result.current.status).toBe("error"));
    if (result.current.status === "error") {
      expect(result.current.error.kind).toBe("DbLocked");
    }
  });

  it("rattrape ready avec integrityWarning si snapshot le contient", async () => {
    vi.mocked(invoke).mockResolvedValue({
      status: "ready",
      integrity_warning: { Details: "warning from rattrapage" },
    });
    const { result } = renderHook(() => useAppReady());
    await waitFor(() => expect(result.current.status).toBe("ready"));
    if (result.current.status === "ready") {
      expect(result.current.integrityWarning?.Details).toBe("warning from rattrapage");
    }
  });
});
