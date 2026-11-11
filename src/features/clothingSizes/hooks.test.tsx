import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import { useSetClothingSize, useDeleteClothingSize } from "./hooks";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

function makeWrapper() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const spy = vi.spyOn(client, "invalidateQueries");
  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return { wrapper, spy };
}

describe("clothingSizes hooks", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("useSetClothingSize appelle set_clothing_size et invalide clothingSizes", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const { wrapper, spy } = makeWrapper();
    const { result } = renderHook(() => useSetClothingSize(), { wrapper });
    await result.current.mutateAsync({ category: "jacket", value: "52", source: "override" });
    expect(mockInvoke).toHaveBeenCalledWith("set_clothing_size", {
      category: "jacket",
      value: "52",
      source: "override",
    });
    await waitFor(() => expect(spy).toHaveBeenCalledWith({ queryKey: ["clothingSizes"] }));
  });

  it("useDeleteClothingSize appelle delete_clothing_size et invalide clothingSizes", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const { wrapper, spy } = makeWrapper();
    const { result } = renderHook(() => useDeleteClothingSize(), { wrapper });
    await result.current.mutateAsync("shoes");
    expect(mockInvoke).toHaveBeenCalledWith("delete_clothing_size", { category: "shoes" });
    await waitFor(() => expect(spy).toHaveBeenCalledWith({ queryKey: ["clothingSizes"] }));
  });
});
