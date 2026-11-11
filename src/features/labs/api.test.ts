import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { getLabResults, addLabResult, deleteLabResult } from "./api";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

describe("labs/api", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("getLabResults invoke get_lab_results avec from/to", async () => {
    const fake = [{ id: 1, test_name: "HDL", value: 60, unit: "mg/dL", measured_at: "2026-04-15" }];
    mockInvoke.mockResolvedValue(fake);

    const result = await getLabResults("2026-04-01", "2026-04-30");

    expect(mockInvoke).toHaveBeenCalledWith("get_lab_results", {
      from: "2026-04-01",
      to: "2026-04-30",
    });
    expect(result).toEqual(fake);
  });

  it("addLabResult invoke add_lab_result avec data wrappee", async () => {
    mockInvoke.mockResolvedValue(15);
    const data = {
      date: "2026-04-29",
      test_name: "LDL",
      value: 110,
      unit: "mg/dL",
      reference_min: 0,
      reference_max: 130,
      notes: null,
    };

    const result = await addLabResult(data);

    expect(mockInvoke).toHaveBeenCalledWith("add_lab_result", { data });
    expect(result).toBe(15);
  });

  it("deleteLabResult invoke delete_lab_result avec id", async () => {
    mockInvoke.mockResolvedValue(undefined);

    await deleteLabResult(4);

    expect(mockInvoke).toHaveBeenCalledWith("delete_lab_result", { id: 4 });
  });
});
