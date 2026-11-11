import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { exportCsv, exportPdf, exportJson, importJson, createBackup } from "./api";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

describe("export/api", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("exportCsv invoke export_csv avec from + to + catégories", async () => {
    mockInvoke.mockResolvedValue("C:\\path\\export.csv");

    const result = await exportCsv("2026-04-01", "2026-04-30", ["body", "sleep"]);

    expect(mockInvoke).toHaveBeenCalledWith("export_csv", {
      from: "2026-04-01",
      to: "2026-04-30",
      categories: ["body", "sleep"],
    });
    expect(result).toBe("C:\\path\\export.csv");
  });

  it("exportPdf invoke export_pdf avec from + to + catégories", async () => {
    mockInvoke.mockResolvedValue("C:\\path\\report.pdf");

    const result = await exportPdf("2026-04-01", "2026-04-30", ["activity"]);

    expect(mockInvoke).toHaveBeenCalledWith("export_pdf", {
      from: "2026-04-01",
      to: "2026-04-30",
      categories: ["activity"],
    });
    expect(result).toBe("C:\\path\\report.pdf");
  });

  it("exportJson invoke export_json sans args", async () => {
    mockInvoke.mockResolvedValue("C:\\path\\export.json");

    const result = await exportJson();

    expect(mockInvoke).toHaveBeenCalledWith("export_json");
    expect(result).toBe("C:\\path\\export.json");
  });

  it("importJson invoke import_json avec path", async () => {
    mockInvoke.mockResolvedValue("imported 42 records");

    const result = await importJson("C:\\path\\backup.json");

    expect(mockInvoke).toHaveBeenCalledWith("import_json", {
      path: "C:\\path\\backup.json",
    });
    expect(result).toContain("42");
  });

  it("createBackup invoke create_backup", async () => {
    mockInvoke.mockResolvedValue("C:\\path\\synk_2026-04-29.db");

    const result = await createBackup();

    expect(mockInvoke).toHaveBeenCalledWith("create_backup");
    expect(result).toContain("synk");
  });
});
