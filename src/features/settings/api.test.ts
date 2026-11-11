import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { getAppDataDir } from "./api";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

describe("settings/api", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("getAppDataDir invoke get_app_data_dir", async () => {
    mockInvoke.mockResolvedValue("C:\\Users\\test\\AppData\\Roaming\\com.synk.health");

    const result = await getAppDataDir();

    expect(mockInvoke).toHaveBeenCalledWith("get_app_data_dir");
    expect(result).toContain("com.synk.health");
  });
});
