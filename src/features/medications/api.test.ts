import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  getMedications,
  addMedication,
  updateMedication,
  deleteMedication,
  logMedicationTaken,
  getMedicationLogs,
} from "./api";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

describe("medications/api", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("getMedications invoke get_medications sans args", async () => {
    const fake = [{ id: 1, name: "Paracetamol", dosage: "500mg" }];
    mockInvoke.mockResolvedValue(fake);

    const result = await getMedications();

    expect(mockInvoke).toHaveBeenCalledWith("get_medications");
    expect(result).toEqual(fake);
  });

  it("addMedication invoke add_medication avec data wrappee", async () => {
    mockInvoke.mockResolvedValue(7);
    const data = {
      name: "Aspirin",
      dosage: "100mg",
      frequency: "daily",
      start_date: "2026-04-01",
      end_date: null,
      notes: null,
    };

    const result = await addMedication(data);

    expect(mockInvoke).toHaveBeenCalledWith("add_medication", { data });
    expect(result).toBe(7);
  });

  it("updateMedication invoke update_medication", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const data = {
      id: 3,
      name: "Aspirin",
      dosage: "200mg",
      frequency: "daily",
      start_date: "2026-04-01",
      end_date: null,
      notes: null,
      active: true,
    };

    await updateMedication(data);

    expect(mockInvoke).toHaveBeenCalledWith("update_medication", { data });
  });

  it("deleteMedication invoke delete_medication avec id", async () => {
    mockInvoke.mockResolvedValue(undefined);

    await deleteMedication(12);

    expect(mockInvoke).toHaveBeenCalledWith("delete_medication", { id: 12 });
  });

  it("logMedicationTaken invoke log_medication_taken", async () => {
    mockInvoke.mockResolvedValue(99);
    const data = {
      medication_id: 3,
      taken_at: "2026-04-29T08:00:00Z",
      skipped: false,
      notes: null,
    };

    const result = await logMedicationTaken(data);

    expect(mockInvoke).toHaveBeenCalledWith("log_medication_taken", { data });
    expect(result).toBe(99);
  });

  it("getMedicationLogs invoke avec medicationId + from + to", async () => {
    const fake = [{ id: 1, medication_id: 3, taken_at: "2026-04-20T08:00:00Z" }];
    mockInvoke.mockResolvedValue(fake);

    const result = await getMedicationLogs(3, "2026-04-01", "2026-04-30");

    expect(mockInvoke).toHaveBeenCalledWith("get_medication_logs", {
      medicationId: 3,
      from: "2026-04-01",
      to: "2026-04-30",
    });
    expect(result).toEqual(fake);
  });
});
