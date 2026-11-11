import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { createCrudHook, type CrudHookConfig } from "./createCrudHook";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

interface FakeItem {
  id: number;
  value: number;
}

interface AddInput {
  value: number;
}

type Config = CrudHookConfig<FakeItem, AddInput>;

const makeWrapper = (qc: QueryClient) => {
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={qc}>{children}</QueryClientProvider>
  );
  return Wrapper;
};

describe("createCrudHook", () => {
  let listMock: Mock<Config["list"]>;
  let latestMock: Mock<Config["latest"]>;
  let addMock: Mock<Config["add"]>;
  let removeMock: Mock<Config["remove"]>;
  let qc: QueryClient;

  beforeEach(() => {
    listMock = vi.fn<Config["list"]>().mockResolvedValue([{ id: 1, value: 42 }]);
    latestMock = vi.fn<Config["latest"]>().mockResolvedValue({ id: 1, value: 42 });
    addMock = vi.fn<Config["add"]>().mockResolvedValue(99);
    removeMock = vi.fn<Config["remove"]>().mockResolvedValue(undefined);
    qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  });

  function setup() {
    return createCrudHook<FakeItem, AddInput>({
      domain: "fake",
      list: listMock,
      latest: latestMock,
      add: addMock,
      remove: removeMock,
    });
  }

  it("useList garde les données de la plage précédente pendant le chargement de la suivante", async () => {
    const hooks = setup();
    listMock
      .mockResolvedValueOnce([{ id: 1, value: 42 }])
      .mockImplementationOnce(() => new Promise<FakeItem[]>(() => undefined));
    const { result, rerender } = renderHook(({ from, to }) => hooks.useList(from, to), {
      wrapper: makeWrapper(qc),
      initialProps: { from: "2026-04-01", to: "2026-04-30" },
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    rerender({ from: "2026-05-01", to: "2026-05-31" });

    await waitFor(() => expect(listMock).toHaveBeenCalledTimes(2));
    expect(result.current.data).toEqual([{ id: 1, value: 42 }]);
    expect(result.current.isPlaceholderData).toBe(true);
  });

  it("useList retourne les données via la fonction list", async () => {
    const hooks = setup();
    const { result } = renderHook(() => hooks.useList("2026-04-01", "2026-04-30"), {
      wrapper: makeWrapper(qc),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual([{ id: 1, value: 42 }]);
    expect(listMock).toHaveBeenCalledWith("2026-04-01", "2026-04-30");
  });

  it("useLatest utilise la queryKey [domain, 'latest']", async () => {
    const hooks = setup();
    const { result } = renderHook(() => hooks.useLatest(), {
      wrapper: makeWrapper(qc),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(qc.getQueryState(["fake", "latest"])?.status).toBe("success");
  });

  it("useAdd invalide les queries [domain] apres succès", async () => {
    const hooks = setup();
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");
    const { result } = renderHook(() => hooks.useAdd(), {
      wrapper: makeWrapper(qc),
    });
    await act(async () => {
      await result.current.mutateAsync({ value: 100 });
    });
    expect(addMock).toHaveBeenCalledWith({ value: 100 });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["fake"] });
  });

  it("useDelete invalide les queries [domain] apres succès", async () => {
    const hooks = setup();
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");
    const { result } = renderHook(() => hooks.useDelete(), {
      wrapper: makeWrapper(qc),
    });
    await act(async () => {
      await result.current.mutateAsync(7);
    });
    expect(removeMock).toHaveBeenCalledWith(7);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["fake"] });
  });

  it("useList n'execute pas si enabled = false", () => {
    const hooks = setup();
    renderHook(() => hooks.useList("from", "to", false), {
      wrapper: makeWrapper(qc),
    });
    expect(listMock).not.toHaveBeenCalled();
  });
});
