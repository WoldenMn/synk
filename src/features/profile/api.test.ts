import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { getUserProfile, updateUserProfile } from "./api";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

describe("profile/api", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("getUserProfile invoke get_user_profile", async () => {
    const fake = { age: 35, sex: "M", height_cm: 178 };
    mockInvoke.mockResolvedValue(fake);

    const result = await getUserProfile();

    expect(mockInvoke).toHaveBeenCalledWith("get_user_profile");
    expect(result).toEqual(fake);
  });

  it("updateUserProfile invoke update_user_profile avec age + sex + heightCm", async () => {
    mockInvoke.mockResolvedValue(undefined);

    await updateUserProfile(40, "F", 165);

    expect(mockInvoke).toHaveBeenCalledWith("update_user_profile", {
      age: 40,
      sex: "F",
      heightCm: 165,
    });
  });

  it("updateUserProfile accepte les nullables", async () => {
    mockInvoke.mockResolvedValue(undefined);

    await updateUserProfile(null, null, null);

    expect(mockInvoke).toHaveBeenCalledWith("update_user_profile", {
      age: null,
      sex: null,
      heightCm: null,
    });
  });
});
