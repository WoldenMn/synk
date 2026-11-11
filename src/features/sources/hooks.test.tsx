import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import {
  useAnySourceConnected,
  useSetCredentials,
  useSourceCredentials,
  useSources,
} from "./hooks";
import type { SourceInfo } from "./types";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

function source(over: Partial<SourceInfo>): SourceInfo {
  return {
    id: "withings",
    label: "Withings",
    kind: "oauth",
    extensions: [],
    domains: ["body"],
    connected: false,
    credentials_configured: false,
    last_sync_at: null,
    redirect_uri: null,
    developer_url: null,
    reauth: null,
    ...over,
  };
}

function makeWrapper(client: QueryClient) {
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return Wrapper;
}

describe("hooks des sources", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("useSources lit le registre via list_sources", async () => {
    mockInvoke.mockResolvedValue([source({ connected: true })]);

    const { result } = renderHook(() => useSources(), { wrapper: makeWrapper(qc) });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("list_sources");
    expect(result.current.data?.[0]?.id).toBe("withings");
  });

  it("useAnySourceConnected dit vrai dès qu'une source l'est, faux sinon", async () => {
    mockInvoke.mockResolvedValue([source({}), source({ id: "oura", connected: true })]);
    const { result } = renderHook(() => useAnySourceConnected(), { wrapper: makeWrapper(qc) });
    await waitFor(() => expect(result.current.data).toBe(true));

    const other = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockResolvedValue([source({}), source({ id: "oura" })]);
    const none = renderHook(() => useAnySourceConnected(), { wrapper: makeWrapper(other) });
    await waitFor(() => expect(none.result.current.data).toBe(false));
  });

  it("useSourceCredentials lit l'état des identifiants de la source nommée", async () => {
    mockInvoke.mockResolvedValue([
      source({ credentials_configured: true }),
      source({ id: "oura", credentials_configured: false }),
    ]);

    const withings = renderHook(() => useSourceCredentials("withings"), {
      wrapper: makeWrapper(qc),
    });
    await waitFor(() => expect(withings.result.current.data).toBe(true));
    const oura = renderHook(() => useSourceCredentials("oura"), { wrapper: makeWrapper(qc) });
    await waitFor(() => expect(oura.result.current.data).toBe(false));
    expect(mockInvoke).toHaveBeenCalledTimes(1);
  });

  it("useSetCredentials nomme la source et invalide ['sources'] après succès", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useSetCredentials("oura"), { wrapper: makeWrapper(qc) });

    await act(async () => {
      await result.current.mutateAsync({ clientId: "abc123", clientSecret: "secret456" });
    });

    expect(mockInvoke).toHaveBeenCalledWith("source_set_credentials", {
      source: "oura",
      clientId: "abc123",
      clientSecret: "secret456",
    });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["sources"] });
  });
});
